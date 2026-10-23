use regex::Regex;
use tsrs_ast::{self as ast, FunctionFlags, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{
    Ctx, Listener, Rule, RuleDiagnostic, RuleMessage, RuleSuggestion, RuleVisitor,
};

use crate::utils::{
    comment_at, estree_declaration_start, estree_parent, is_estree_chain_expression, node_text,
    skip_parens,
};

pub const NAME: &str = "react-compiler-extra";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    run_only_with_enable_compiler_directive: bool,
}

pub struct ReactCompilerExtra {
    run_only_with_enable_compiler_directive: bool,
    enable_compiler_directive: Regex,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let enable_compiler_directive =
        Regex::new(r#"eslint +react-compiler/react-compiler: +\["error"#)
            .map_err(|e| e.to_string())?;
    Ok(Box::new(ReactCompilerExtra {
        run_only_with_enable_compiler_directive: options.run_only_with_enable_compiler_directive,
        enable_compiler_directive,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::VariableDeclaration),
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Enter(Kind::GetAccessor),
    Listener::Enter(Kind::SetAccessor),
    Listener::Enter(Kind::Constructor),
    Listener::Enter(Kind::ReturnStatement),
];

impl Rule for ReactCompilerExtra {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let enabled = !self.run_only_with_enable_compiler_directive
            || self.has_enable_compiler_directive(ctx);
        Box::new(Visitor { enabled })
    }
}

impl ReactCompilerExtra {
    /// Whether a comment holds the `eslint react-compiler/react-compiler: ["error"` directive.
    fn has_enable_compiler_directive(&self, ctx: &Ctx) -> bool {
        self.enable_compiler_directive
            .find_iter(ctx.text())
            .any(|found| {
                u32::try_from(found.start())
                    .ok()
                    .and_then(|start| comment_at(ctx.file, start))
                    .is_some_and(|(_, end, _)| {
                        usize::try_from(end).is_ok_and(|end| found.end() <= end)
                    })
            })
    }
}

struct Visitor {
    enabled: bool,
}

const MESSAGE_OBJECT_METHOD: &str = "Object methods such as `{ method() {} }` have limited support to optimizations in the React compiler, use a function expression `{ method: () => {} }` instead.";
const MESSAGE_THIS_IN_METHOD: &str = "Object method uses `this` keyword which would have different behavior if converted to an arrow function. Fix this manually.";
const MESSAGE_SHOULD_RETURN_JSX: &str = "React components and hooks should create JSX elements, call other hooks or use the \"use memo\" directive for optimal React compiler detection.";
const MESSAGE_ADD_USE_MEMO: &str =
    "Add \"use memo\" directive to opt into React compiler memoization";
const MESSAGE_CALLING_HOOKS: &str = "Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with \"use\").";
const MESSAGE_USE_MEMO_NAMING: &str = "Functions using \"use memo\" directive must follow React naming conventions (PascalCase for components or start with \"use\" for hooks).";

fn identifier_name(node: P<Node>) -> Option<&'static str> {
    ast::is_identifier(node).then(|| node.as_identifier().text())
}

fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase()) && chars.all(|c| c.is_ascii_alphanumeric())
}

fn is_hook_name(name: &str) -> bool {
    name.starts_with("use") && name.len() > 3
}

fn is_valid_react_function_name(name: &str) -> bool {
    is_pascal_case(name) || is_hook_name(name)
}

/// ESTree `FunctionDeclaration`, `FunctionExpression` and `ArrowFunctionExpression`: methods, accessors and
/// constructors are `FunctionExpression`s there, while body-less ones are `TSDeclareFunction`s or
/// `TSEmptyBodyFunctionExpression`s.
fn is_estree_function(node: P<Node>) -> bool {
    match node.kind() {
        Kind::ArrowFunction | Kind::FunctionExpression => true,
        Kind::FunctionDeclaration
        | Kind::MethodDeclaration
        | Kind::GetAccessor
        | Kind::SetAccessor
        | Kind::Constructor => node.body().is_some(),
        _ => false,
    }
}

fn is_method_like(node: P<Node>) -> bool {
    matches!(
        node.kind(),
        Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor | Kind::Constructor
    )
}

/// The callee as ESTree sees it: a parenthesized optional chain becomes a `ChainExpression`.
fn estree_callee(call: P<Node>) -> Option<P<Node>> {
    let callee = skip_parens(call.as_call_expression().expression());
    (!is_estree_chain_expression(callee)).then_some(callee)
}

/// The ESTree `MemberExpression` object and Identifier property of `node` (`a.b` and `a[b]`).
fn member_parts(node: P<Node>) -> Option<(P<Node>, &'static str)> {
    match node.kind() {
        Kind::PropertyAccessExpression => {
            let access = node.as_property_access_expression();
            Some((
                skip_parens(access.expression()),
                identifier_name(access.name())?,
            ))
        }
        Kind::ElementAccessExpression => {
            let access = node.as_element_access_expression();
            Some((
                skip_parens(access.expression()),
                identifier_name(skip_parens(access.argument_expression()))?,
            ))
        }
        _ => None,
    }
}

fn is_hook_callee(call: P<Node>) -> bool {
    estree_callee(call)
        .and_then(identifier_name)
        .is_some_and(|name| name.starts_with("use"))
}

fn is_hook_callee_including_members(call: P<Node>) -> bool {
    let Some(callee) = estree_callee(call) else {
        return false;
    };
    identifier_name(callee)
        .or_else(|| member_parts(callee).map(|(_, property)| property))
        .is_some_and(|name| name.starts_with("use"))
}

fn creates_jsx(node: P<Node>) -> bool {
    let node = skip_parens(node);
    match node.kind() {
        Kind::JsxElement | Kind::JsxSelfClosingElement | Kind::JsxFragment => true,
        Kind::ConditionalExpression => {
            let conditional = node.as_conditional_expression();
            creates_jsx(conditional.when_true) || creates_jsx(conditional.when_false)
        }
        Kind::BinaryExpression => {
            let binary = node.as_binary_expression();
            match binary.operator_token().kind() {
                Kind::AmpersandAmpersandToken => creates_jsx(binary.right()),
                Kind::BarBarToken => creates_jsx(binary.left()) || creates_jsx(binary.right()),
                _ => false,
            }
        }
        Kind::CallExpression => {
            if is_estree_chain_expression(node) {
                return false;
            }
            let is_create_element =
                estree_callee(node)
                    .and_then(member_parts)
                    .is_some_and(|(object, property)| {
                        identifier_name(object) == Some("React") && property == "createElement"
                    });
            is_create_element
                || node
                    .arguments()
                    .iter()
                    .any(|&arg| !ast::is_spread_element(arg) && creates_jsx(arg))
        }
        _ => false,
    }
}

fn is_fc_type(type_node: P<Node>) -> bool {
    if type_node.kind() != Kind::TypeReference {
        return false;
    }
    let type_name = type_node.as_type_reference_node().type_name();
    match type_name.kind() {
        Kind::Identifier => type_name.as_identifier().text() == "FC",
        Kind::QualifiedName => {
            let qualified = type_name.as_qualified_name();
            identifier_name(qualified.left) == Some("React")
                && identifier_name(qualified.right) == Some("FC")
        }
        _ => false,
    }
}

#[derive(Default)]
struct ReactAnalysis {
    contains_jsx: bool,
    contains_hook_calls: bool,
}

fn analyze_react_behavior(node: P<Node>) -> ReactAnalysis {
    fn visit(node: P<Node>, result: &mut ReactAnalysis) -> bool {
        if result.contains_jsx && result.contains_hook_calls {
            return true;
        }
        match node.kind() {
            Kind::JsxElement | Kind::JsxSelfClosingElement | Kind::JsxFragment => {
                result.contains_jsx = true;
            }
            Kind::CallExpression => {
                if is_hook_callee(node) {
                    result.contains_hook_calls = true;
                }
                if creates_jsx(node) {
                    result.contains_jsx = true;
                }
            }
            Kind::ReturnStatement if node.expression().is_some_and(creates_jsx) => {
                result.contains_jsx = true;
            }
            Kind::VariableDeclaration if node.initializer().is_some_and(creates_jsx) => {
                result.contains_jsx = true;
            }
            Kind::BinaryExpression
                if ast::is_assignment_expression(node, false)
                    && creates_jsx(node.as_binary_expression().right()) =>
            {
                result.contains_jsx = true;
            }
            _ => {}
        }
        node.for_each_child(&mut |child| visit(child, result))
    }
    let mut result = ReactAnalysis::default();
    visit(node, &mut result);
    result
}

/// Whether `body` calls a hook before the traversal reaches a nested function: like the ESLint rule's traversal, a
/// nested function stops the whole search, not only its own subtree.
fn calls_hooks_directly(body: P<Node>) -> bool {
    fn visit(node: P<Node>, root: P<Node>, found: &mut bool) -> bool {
        if *found {
            return true;
        }
        if node != root && is_estree_function(node) {
            if is_method_like(node) {
                for &modifier in node.modifier_nodes() {
                    if visit(modifier, root, found) {
                        return true;
                    }
                }
                if let Some(name) = node.name() {
                    visit(name, root, found);
                }
            }
            return true;
        }
        if node.kind() == Kind::CallExpression && is_hook_callee_including_members(node) {
            *found = true;
        }
        node.for_each_child(&mut |child| visit(child, root, found))
    }
    let mut found = false;
    visit(body, body, &mut found);
    found
}

fn string_literal_value(node: P<Node>) -> Option<&'static str> {
    let node = skip_parens(node);
    (node.kind() == Kind::StringLiteral).then(|| node.text())
}

fn is_string_statement(statement: P<Node>, value: &str) -> bool {
    statement.kind() == Kind::ExpressionStatement
        && statement.expression().and_then(string_literal_value) == Some(value)
}

fn block_body(function: P<Node>) -> Option<P<Node>> {
    function.body().filter(|body| body.kind() == Kind::Block)
}

fn has_use_no_memo_directive(function: P<Node>) -> bool {
    block_body(function)
        .and_then(|body| body.statements().first().copied())
        .is_some_and(|statement| is_string_statement(statement, "use no memo"))
}

fn has_use_memo_directive(function: P<Node>) -> bool {
    block_body(function).is_some_and(|body| {
        body.statements()
            .iter()
            .any(|&statement| is_string_statement(statement, "use memo"))
    })
}

fn containing_function(node: P<Node>) -> Option<P<Node>> {
    std::iter::successors(node.parent(), |current| current.parent())
        .find(|&a| is_estree_function(a))
}

fn should_skip_due_to_use_no_memo(node: P<Node>) -> bool {
    containing_function(node).is_some_and(has_use_no_memo_directive)
}

fn function_declaration_name(function: P<Node>) -> Option<&'static str> {
    if function.kind() != Kind::FunctionDeclaration {
        return None;
    }
    function.name().and_then(identifier_name)
}

fn is_react_component_or_hook(
    function: P<Node>,
    identifier: Option<&str>,
    type_annotation: Option<P<Node>>,
) -> bool {
    if type_annotation.is_some_and(is_fc_type) {
        if let Some(identifier) = identifier {
            return is_pascal_case(identifier);
        }
        if let Some(name) = function_declaration_name(function) {
            return is_pascal_case(name);
        }
        return true;
    }
    if identifier.is_some_and(is_hook_name) {
        return true;
    }
    if function_declaration_name(function).is_some_and(is_hook_name) {
        return true;
    }
    let function_name = identifier.or_else(|| function_declaration_name(function));
    if function_name.is_some_and(is_pascal_case) {
        return function
            .body()
            .is_some_and(|body| analyze_react_behavior(body).contains_jsx);
    }
    false
}

fn behaves_like_react_component_or_hook(function: P<Node>) -> bool {
    function.body().is_some_and(|body| {
        let analysis = analyze_react_behavior(body);
        analysis.contains_jsx || analysis.contains_hook_calls
    })
}

fn calls_hooks_but_not_valid_component(
    function: P<Node>,
    identifier: Option<&str>,
    type_annotation: Option<P<Node>>,
) -> bool {
    if !function.body().is_some_and(calls_hooks_directly) {
        return false;
    }
    let mut is_valid = is_react_component_or_hook(function, identifier, type_annotation);
    if !is_valid && type_annotation.is_some() {
        let function_name = identifier.or_else(|| function_declaration_name(function));
        if function_name.is_some_and(is_pascal_case) {
            is_valid = true;
        }
    }
    !is_valid
}

/// `\bthis[.[]`
fn contains_this_keyword(text: &str) -> bool {
    text.match_indices("this").any(|(index, _)| {
        let before = text.get(..index).and_then(|t| t.chars().next_back());
        let after = text.get(index + 4..).and_then(|t| t.chars().next());
        !before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            && matches!(after, Some('.' | '['))
    })
}

/// The range of the ESTree function node: a method's `FunctionExpression` starts at its type parameters or
/// parameter list, not at its key, and an exported function declaration starts after `export`.
fn estree_function_range(ctx: &Ctx, function: P<Node>) -> (i32, i32) {
    let (start, end) = ctx.trim(function);
    if function.kind() == Kind::FunctionDeclaration {
        return (estree_declaration_start(ctx, function), end);
    }
    if !is_method_like(function) {
        return (start, end);
    }
    let list = function
        .type_parameter_list()
        .or_else(|| function.parameter_list());
    let list_start = list
        .and_then(|list| list.pos().source_offset())
        .and_then(|pos| i32::try_from(pos).ok())
        .map(|pos| pos - 1);
    (list_start.unwrap_or(start), end)
}

fn report_function(ctx: &mut Ctx, function: P<Node>, message: RuleMessage) {
    let (pos, end) = estree_function_range(ctx, function);
    ctx.report_range(pos, end, message);
}

fn report_should_return_jsx(ctx: &mut Ctx, function: P<Node>) {
    let message = RuleMessage::new("fcComponentShouldReturnJsx", MESSAGE_SHOULD_RETURN_JSX);
    let Some(body) = block_body(function) else {
        report_function(ctx, function, message);
        return;
    };
    let (pos, end) = estree_function_range(ctx, function);
    let diagnostic = RuleDiagnostic {
        pos,
        end,
        message,
        labeled_ranges: Vec::new(),
    };
    ctx.report_diagnostic_with_suggestions(diagnostic, |ctx| {
        let open_brace = ctx.trim(body).0 + 1;
        vec![RuleSuggestion {
            message: RuleMessage::new("addUseMemoDirective", MESSAGE_ADD_USE_MEMO),
            fixes: vec![ctx.fix_replace_range(open_brace, open_brace, "\n  \"use memo\"\n")],
        }]
    });
}

fn method_body(method: P<Node>) -> Option<P<Node>> {
    (method.kind() == Kind::MethodDeclaration)
        .then(|| method.body())
        .flatten()
}

fn check_for_object_methods(ctx: &mut Ctx, object: P<Node>) {
    for &property in object.properties() {
        let Some(body) = method_body(property) else {
            continue;
        };
        let body_text = node_text(ctx, body);
        if contains_this_keyword(body_text) {
            ctx.report_node(
                property,
                RuleMessage::new("thisKeywordInMethod", MESSAGE_THIS_IN_METHOD),
            );
            continue;
        }
        let Some(name) = property.name() else {
            continue;
        };
        let key = if name.kind() == Kind::ComputedPropertyName {
            name.expression().unwrap_or(name)
        } else {
            name
        };
        let method_name = node_text(ctx, key);
        let params: Vec<&str> = property
            .parameters()
            .iter()
            .map(|&parameter| node_text(ctx, parameter))
            .collect();
        let params = params.join(", ");
        let flags = ast::get_function_flags(Some(property));
        let replacement = if flags.contains(FunctionFlags::Generator) {
            format!("{method_name}: function* ({params}) {body_text}")
        } else if flags.contains(FunctionFlags::Async) {
            format!("{method_name}: async ({params}) => {body_text}")
        } else {
            format!("{method_name}: ({params}) => {body_text}")
        };
        ctx.report_node_with_fixes(
            property,
            RuleMessage::new("objectMethodIsNotSupported", MESSAGE_OBJECT_METHOD),
            |ctx| vec![ctx.fix_replace(property, replacement)],
        );
    }
}

fn check_nested_this_usage(ctx: &mut Ctx, object: P<Node>) {
    for &property in object.properties() {
        if property.kind() != Kind::PropertyAssignment {
            continue;
        }
        let Some(value) = property.initializer().map(skip_parens) else {
            continue;
        };
        if value.kind() != Kind::ObjectLiteralExpression {
            continue;
        }
        for &nested in value.properties() {
            let Some(body) = method_body(nested) else {
                continue;
            };
            if contains_this_keyword(node_text(ctx, body)) {
                ctx.report_node(
                    nested,
                    RuleMessage::new("thisKeywordInMethod", MESSAGE_THIS_IN_METHOD),
                );
            }
        }
    }
}

fn check_object(ctx: &mut Ctx, object: P<Node>) {
    check_for_object_methods(ctx, object);
    check_nested_this_usage(ctx, object);
}

fn returned_object(statement: P<Node>) -> Option<P<Node>> {
    if statement.kind() != Kind::ReturnStatement {
        return None;
    }
    statement
        .expression()
        .map(skip_parens)
        .filter(|e| e.kind() == Kind::ObjectLiteralExpression)
}

fn check_call_expression(ctx: &mut Ctx, node: P<Node>) {
    if !is_hook_callee_including_members(node) {
        return;
    }
    for &arg in node.arguments() {
        let arg = skip_parens(arg);
        let mut objects = Vec::new();
        match arg.kind() {
            Kind::ObjectLiteralExpression => objects.push(arg),
            Kind::ArrowFunction | Kind::FunctionExpression => {
                if let Some(body) = arg.body() {
                    let body = if arg.kind() == Kind::ArrowFunction {
                        skip_parens(body)
                    } else {
                        body
                    };
                    if arg.kind() == Kind::ArrowFunction
                        && body.kind() == Kind::ObjectLiteralExpression
                    {
                        objects.push(body);
                    }
                    if body.kind() == Kind::Block {
                        objects
                            .extend(body.statements().iter().filter_map(|&s| returned_object(s)));
                    }
                }
            }
            _ => {}
        }
        for object in objects {
            if should_skip_due_to_use_no_memo(node) {
                return;
            }
            check_object(ctx, object);
        }
    }
}

fn check_variable_declaration(ctx: &mut Ctx, node: P<Node>) {
    let Some(identifier) = node.name().and_then(identifier_name) else {
        return;
    };
    let Some(function) = node.initializer().map(skip_parens) else {
        return;
    };
    if !matches!(
        function.kind(),
        Kind::ArrowFunction | Kind::FunctionExpression
    ) {
        return;
    }
    let type_annotation = node.type_node();

    if has_use_memo_directive(function) && !is_valid_react_function_name(identifier) {
        report_function(
            ctx,
            function,
            RuleMessage::new("useMemoDirectiveNaming", MESSAGE_USE_MEMO_NAMING),
        );
        return;
    }

    if calls_hooks_but_not_valid_component(function, Some(identifier), type_annotation) {
        if has_use_no_memo_directive(function) {
            return;
        }
        report_function(
            ctx,
            function,
            RuleMessage::new("functionCallingHooksMustBeComponent", MESSAGE_CALLING_HOOKS),
        );
        return;
    }

    if is_react_component_or_hook(function, Some(identifier), type_annotation)
        && !behaves_like_react_component_or_hook(function)
        && !has_use_memo_directive(function)
        && !has_use_no_memo_directive(function)
    {
        report_should_return_jsx(ctx, function);
    }
}

fn check_function_declaration(ctx: &mut Ctx, node: P<Node>) {
    if node.body().is_none() {
        return;
    }
    if let Some(name) = node.name().and_then(identifier_name) {
        if has_use_memo_directive(node) && !is_valid_react_function_name(name) {
            report_function(
                ctx,
                node,
                RuleMessage::new("useMemoDirectiveNaming", MESSAGE_USE_MEMO_NAMING),
            );
            return;
        }
    }

    if calls_hooks_but_not_valid_component(node, None, None) {
        if has_use_no_memo_directive(node) {
            return;
        }
        report_function(
            ctx,
            node,
            RuleMessage::new("functionCallingHooksMustBeComponent", MESSAGE_CALLING_HOOKS),
        );
        return;
    }

    if is_react_component_or_hook(node, None, None)
        && !behaves_like_react_component_or_hook(node)
        && !has_use_memo_directive(node)
        && !has_use_no_memo_directive(node)
    {
        report_should_return_jsx(ctx, node);
    }
}

fn check_function_expression(ctx: &mut Ctx, node: P<Node>) {
    if !is_estree_function(node) {
        return;
    }
    if has_use_memo_directive(node) {
        report_function(
            ctx,
            node,
            RuleMessage::new("useMemoDirectiveNaming", MESSAGE_USE_MEMO_NAMING),
        );
        return;
    }
    if calls_hooks_but_not_valid_component(node, None, None) && !has_use_no_memo_directive(node) {
        report_function(
            ctx,
            node,
            RuleMessage::new("functionCallingHooksMustBeComponent", MESSAGE_CALLING_HOOKS),
        );
    }
}

fn check_return_statement(ctx: &mut Ctx, node: P<Node>) {
    let Some(object) = returned_object(node) else {
        return;
    };
    let Some(function) = containing_function(node) else {
        return;
    };
    let function_name = match function.kind() {
        Kind::FunctionDeclaration => function.name().and_then(identifier_name),
        Kind::ArrowFunction | Kind::FunctionExpression => estree_parent(function)
            .filter(|parent| parent.kind() == Kind::VariableDeclaration)
            .and_then(|parent| parent.name())
            .and_then(identifier_name),
        _ => None,
    };
    if !function_name.is_some_and(is_hook_name) {
        return;
    }
    if should_skip_due_to_use_no_memo(node) {
        return;
    }
    check_object(ctx, object);
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        if self.enabled {
            LISTENERS
        } else {
            &[]
        }
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::CallExpression => check_call_expression(ctx, node),
            Kind::VariableDeclaration => check_variable_declaration(ctx, node),
            Kind::FunctionDeclaration => check_function_declaration(ctx, node),
            Kind::ReturnStatement => check_return_statement(ctx, node),
            _ => check_function_expression(ctx, node),
        }
    }
}
