use tsrs_ast::{self as ast, FunctionFlags, Kind, Node, Symbol};
use tsrs_checker::{Checker, Type, TypeFlags};
use tsrs_core::P;
use tsrslint::rule::{
    Ctx, Listener, Rule, RuleDiagnostic, RuleMessage, RuleSuggestion, RuleVisitor,
};

use crate::esquery::estree::{self, EsType};
use crate::utils::{
    comment_value, comments, comments_directly_before, estree_parent, explanation_anchors,
    skip_parens, type_annotation_range,
};

pub const NAME: &str = "require-type-guard-explanation";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct AlternativeMsgs {
    default: Option<String>,
    unknown_input: Option<String>,
    in_array_find: Option<String>,
    in_array_filter: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    comment_prefix: String,
    min_length: usize,
    alternative_msgs: AlternativeMsgs,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            comment_prefix: "type-guard-safety:".to_string(),
            min_length: 20,
            alternative_msgs: AlternativeMsgs::default(),
        }
    }
}

pub struct RequireTypeGuardExplanation {
    comment_prefix: String,
    min_length: usize,
    missing_message: String,
    missing_message_unknown_input: Option<String>,
    missing_message_in_find: Option<String>,
    missing_message_in_filter: Option<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let missing_message = format!(
        "Type guards are not checked by TypeScript. Add a \"// {} ...\" comment (at least {} characters) explaining \
         why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a \
         discriminated union, a schema parser) can't be used.",
        options.comment_prefix.trim_end(),
        options.min_length
    );
    let with_hint = |hint: Option<String>| {
        hint.filter(|hint| !hint.is_empty())
            .map(|hint| format!("{missing_message} {hint}"))
    };
    let msgs = options.alternative_msgs;
    let default = with_hint(msgs.default);
    let unknown_input = with_hint(msgs.unknown_input).or_else(|| default.clone());
    let missing_message_in_find = with_hint(msgs.in_array_find);
    let missing_message_in_filter = with_hint(msgs.in_array_filter);
    Ok(Box::new(RequireTypeGuardExplanation {
        comment_prefix: options.comment_prefix,
        min_length: options.min_length,
        missing_message: default.unwrap_or(missing_message),
        missing_message_unknown_input: unknown_input,
        missing_message_in_find,
        missing_message_in_filter,
    }))
}

const MESSAGE_INFERABLE: &str =
    "TypeScript infers this type predicate, remove the return type annotation.";
const MESSAGE_REMOVE_ANNOTATION: &str = "Remove the return type annotation";

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::FunctionExpression),
    Listener::Enter(Kind::ArrowFunction),
    Listener::Enter(Kind::MethodDeclaration),
];

impl Rule for RequireTypeGuardExplanation {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            rule: self,
            comments: None,
        })
    }
}

struct Visitor {
    rule: &'static RequireTypeGuardExplanation,
    comments: Option<Vec<(u32, u32, bool)>>,
}

/// The type predicate `function` declares as its return type: `x is T`, `this is T` or `asserts x is T`, but not
/// `asserts x`, which only narrows to truthy.
fn checked_type_predicate(function: P<Node>) -> Option<P<Node>> {
    function.body()?;
    let return_type = function.type_node()?;
    (ast::is_type_predicate_node(return_type)
        && return_type.as_type_predicate_node().type_().is_some())
    .then_some(return_type)
}

/// The ESTree parents that end the search for an explanation: a comment above the statement, class member or object
/// member holding the guard counts, not one above the block, class or object holding that.
fn is_container(parent: EsType) -> bool {
    matches!(
        parent,
        EsType::Program
            | EsType::BlockStatement
            | EsType::StaticBlock
            | EsType::SwitchCase
            | EsType::ClassBody
            | EsType::ObjectExpression
    )
}

/// The explanation in a comment block (one block comment or consecutive line comments): the text after the first
/// line starting with `prefix`, with the lines joined and the whitespace collapsed.
fn block_explanation(lines: &[&str], prefix: &str) -> Option<String> {
    let start = lines.iter().position(|line| line.starts_with(prefix))?;
    let mut words = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(start) {
        let text = if index == start {
            line.get(prefix.len()..).unwrap_or("")
        } else {
            line
        };
        words.extend(text.split_whitespace());
    }
    Some(words.join(" "))
}

/// The lines of a comment, trimmed and without the leading `*` of JSDoc-style block comment lines.
fn comment_lines(text: &str, comment: (u32, u32, bool)) -> Vec<&str> {
    let value = comment_value(text, comment);
    if !comment.2 {
        return vec![value.trim()];
    }
    value
        .lines()
        .map(|line| {
            let line = line.trim();
            line.strip_prefix('*').map_or(line, str::trim_start)
        })
        .collect()
}

/// Whether the text between two comments has at most one line break: consecutive `//` lines.
fn is_next_line(text: &str, end: u32, start: u32) -> bool {
    let (Ok(end), Ok(start)) = (usize::try_from(end), usize::try_from(start)) else {
        return false;
    };
    text.get(end..start)
        .is_some_and(|between| between.matches('\n').count() <= 1)
}

impl Visitor {
    fn has_explanation(&mut self, ctx: &Ctx, function: P<Node>) -> bool {
        let Some(es) = estree::node_of(function) else {
            return false;
        };
        let text = ctx.text();
        let comments = self.comments.get_or_insert_with(|| comments(ctx.file));
        let prefix = self.rule.comment_prefix.as_str();
        let min_length = self.rule.min_length;
        explanation_anchors(ctx.file, es, is_container)
            .into_iter()
            .any(|start| {
                let mut before = comments_directly_before(text, comments, start);
                before.reverse();
                let mut blocks: Vec<Vec<&str>> = Vec::new();
                let mut previous: Option<(u32, u32, bool)> = None;
                for comment in before {
                    let joins_previous = previous.is_some_and(|previous| {
                        !previous.2 && !comment.2 && is_next_line(text, previous.1, comment.0)
                    });
                    let lines = comment_lines(text, comment);
                    match blocks.last_mut() {
                        Some(block) if joins_previous => block.extend(lines),
                        _ => blocks.push(lines),
                    }
                    previous = Some(comment);
                }
                blocks.iter().any(|block| {
                    block_explanation(block, prefix)
                        .is_some_and(|explanation| explanation.chars().count() >= min_length)
                })
            })
    }
}

/// The expression a function returns when it is its only return: an arrow function's expression body, or the
/// `return` of a block body holding nothing else.
fn single_return_expression(function: P<Node>) -> Option<P<Node>> {
    let body = function.body()?;
    if body.kind() != Kind::Block {
        return Some(skip_parens(body));
    }
    match body.statements() {
        [statement] if statement.kind() == Kind::ReturnStatement => {
            statement.expression().map(skip_parens)
        }
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum Narrowing {
    /// `x !== null`
    NotNull,
    /// `x !== undefined`
    NotUndefined,
    /// `x != null` or `x != undefined`
    NotNullish,
    /// `x === null`
    IsNull,
    /// `x === undefined`
    IsUndefined,
    /// `x == null` or `x == undefined`
    IsNullish,
    /// `typeof x === 'name'` (`true`) or `typeof x !== 'name'` (`false`)
    Typeof(&'static str, bool),
}

fn is_parameter_reference(checker: &mut Checker, node: P<Node>, parameter: P<Symbol>) -> bool {
    let node = skip_parens(node);
    ast::is_identifier(node) && checker.get_symbol_at_location_exported(node) == Some(parameter)
}

fn is_undefined(checker: &mut Checker, node: P<Node>) -> bool {
    let node = skip_parens(node);
    ast::is_identifier(node)
        && node.as_identifier().text() == "undefined"
        && checker.get_symbol_at_location_exported(node) == Some(checker.get_undefined_symbol())
}

fn typeof_operand(node: P<Node>) -> Option<P<Node>> {
    let node = skip_parens(node);
    (node.kind() == Kind::TypeOfExpression).then(|| node.as_type_of_expression().expression)
}

fn string_literal(node: P<Node>) -> Option<&'static str> {
    let node = skip_parens(node);
    matches!(
        node.kind(),
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral
    )
    .then(|| node.text())
}

const TYPEOF_NAMES: &[&str] = &["string", "number", "bigint", "boolean", "symbol"];

/// The narrowing of a single comparison of the parameter (in either operand order).
fn comparison_narrowing(
    checker: &mut Checker,
    expression: P<Node>,
    parameter: P<Symbol>,
) -> Option<Narrowing> {
    let binary = expression.as_binary_expression();
    let operator = binary.operator_token().kind();
    let (left, right) = (binary.left(), binary.right());
    for (subject, other) in [(left, right), (right, left)] {
        if is_parameter_reference(checker, subject, parameter) {
            let is_null = skip_parens(other).kind() == Kind::NullKeyword;
            let is_undefined = is_undefined(checker, other);
            return match operator {
                Kind::ExclamationEqualsEqualsToken if is_null => Some(Narrowing::NotNull),
                Kind::ExclamationEqualsEqualsToken if is_undefined => Some(Narrowing::NotUndefined),
                Kind::ExclamationEqualsToken if is_null || is_undefined => {
                    Some(Narrowing::NotNullish)
                }
                Kind::EqualsEqualsEqualsToken if is_null => Some(Narrowing::IsNull),
                Kind::EqualsEqualsEqualsToken if is_undefined => Some(Narrowing::IsUndefined),
                Kind::EqualsEqualsToken if is_null || is_undefined => Some(Narrowing::IsNullish),
                _ => None,
            };
        }
        let Some(operand) = typeof_operand(subject) else {
            continue;
        };
        if !is_parameter_reference(checker, operand, parameter) {
            return None;
        }
        let name = string_literal(other)?;
        let name = TYPEOF_NAMES.iter().copied().find(|&known| known == name)?;
        return match operator {
            Kind::EqualsEqualsEqualsToken | Kind::EqualsEqualsToken => {
                Some(Narrowing::Typeof(name, true))
            }
            Kind::ExclamationEqualsEqualsToken | Kind::ExclamationEqualsToken => {
                Some(Narrowing::Typeof(name, false))
            }
            _ => None,
        };
    }
    None
}

/// The narrowings the returned expression applies to the parameter when it is true: all of them for a comparison
/// or an `&&` chain of `!==`/`!=` null checks, any of them for an `||` chain of `===`/`==` null checks.
struct Condition {
    narrowings: Vec<Narrowing>,
    any: bool,
}

/// The operands of a chain of `operator` (`a && b && c`), looking through parentheses.
fn chain_operands(expression: P<Node>, operator: Kind, operands: &mut Vec<P<Node>>) {
    let expression = skip_parens(expression);
    if expression.kind() == Kind::BinaryExpression
        && expression.as_binary_expression().operator_token().kind() == operator
    {
        let binary = expression.as_binary_expression();
        chain_operands(binary.left(), operator, operands);
        chain_operands(binary.right(), operator, operands);
    } else {
        operands.push(expression);
    }
}

fn condition(
    checker: &mut Checker,
    expression: P<Node>,
    parameter: P<Symbol>,
) -> Option<Condition> {
    let expression = skip_parens(expression);
    if expression.kind() != Kind::BinaryExpression {
        return None;
    }
    let operator = expression.as_binary_expression().operator_token().kind();
    let any = match operator {
        Kind::AmpersandAmpersandToken => false,
        Kind::BarBarToken => true,
        _ => {
            return Some(Condition {
                narrowings: vec![comparison_narrowing(checker, expression, parameter)?],
                any: false,
            })
        }
    };
    let mut operands = Vec::new();
    chain_operands(expression, operator, &mut operands);
    let mut narrowings = Vec::with_capacity(operands.len());
    for operand in operands {
        if operand.kind() != Kind::BinaryExpression {
            return None;
        }
        let narrowing = comparison_narrowing(checker, operand, parameter)?;
        let fits = if any {
            matches!(
                narrowing,
                Narrowing::IsNull | Narrowing::IsUndefined | Narrowing::IsNullish
            )
        } else {
            matches!(
                narrowing,
                Narrowing::NotNull | Narrowing::NotUndefined | Narrowing::NotNullish
            )
        };
        if !fits {
            return None;
        }
        narrowings.push(narrowing);
    }
    Some(Condition { narrowings, any })
}

/// Whether TypeScript knows exactly which `typeof` and nullish checks `t` passes: primitives, literals, `null`,
/// `undefined` and object types, but not `any`, `unknown`, `void`, type parameters or other instantiable types.
fn has_definite_facts(t: P<Type>) -> bool {
    t.flags().intersects(
        TypeFlags::StringLike
            | TypeFlags::NumberLike
            | TypeFlags::BigIntLike
            | TypeFlags::BooleanLike
            | TypeFlags::ESSymbolLike
            | TypeFlags::Null
            | TypeFlags::Undefined
            | TypeFlags::Object
            | TypeFlags::NonPrimitive,
    ) && !t.flags().intersects(
        TypeFlags::Any
            | TypeFlags::Unknown
            | TypeFlags::Void
            | TypeFlags::Never
            | TypeFlags::Instantiable
            | TypeFlags::Intersection,
    )
}

fn typeof_flags(name: &str) -> TypeFlags {
    match name {
        "string" => TypeFlags::StringLike,
        "number" => TypeFlags::NumberLike,
        "bigint" => TypeFlags::BigIntLike,
        "boolean" => TypeFlags::BooleanLike,
        _ => TypeFlags::ESSymbolLike,
    }
}

fn typeof_type(checker: &Checker, name: &str) -> P<Type> {
    match name {
        "string" => checker.get_string_type(),
        "number" => checker.get_number_type(),
        "bigint" => checker.get_big_int_type(),
        "boolean" => checker.get_boolean_type(),
        _ => checker.get_es_symbol_type(),
    }
}

/// Whether constituent `t` passes `narrowing`, or `None` when TypeScript would narrow it to something else (e.g.
/// `{}` to `string` for `typeof x === 'string'`).
fn passes(checker: &mut Checker, t: P<Type>, narrowing: Narrowing) -> Option<bool> {
    let flags = t.flags();
    match narrowing {
        Narrowing::NotNull => Some(!flags.intersects(TypeFlags::Null)),
        Narrowing::NotUndefined => Some(!flags.intersects(TypeFlags::Undefined)),
        Narrowing::NotNullish => Some(!flags.intersects(TypeFlags::Null | TypeFlags::Undefined)),
        Narrowing::IsNull => Some(flags.intersects(TypeFlags::Null)),
        Narrowing::IsUndefined => Some(flags.intersects(TypeFlags::Undefined)),
        Narrowing::IsNullish => Some(flags.intersects(TypeFlags::Null | TypeFlags::Undefined)),
        Narrowing::Typeof(name, equals) => {
            if flags.intersects(TypeFlags::Object | TypeFlags::NonPrimitive) {
                let implied = typeof_type(checker, name);
                if checker.is_type_assignable_to(implied, t) {
                    return None;
                }
                return Some(!equals);
            }
            Some(flags.intersects(typeof_flags(name)) == equals)
        }
    }
}

/// What `unknown` narrows to when `condition` holds: the `typeof` type, or `null`/`undefined` for positive null
/// checks.
fn narrowed_unknown(checker: &mut Checker, condition: &Condition) -> Option<P<Type>> {
    if let [Narrowing::Typeof(name, true)] = condition.narrowings.as_slice() {
        return Some(typeof_type(checker, name));
    }
    let mut types = Vec::new();
    for narrowing in &condition.narrowings {
        match narrowing {
            Narrowing::IsNull => types.push(checker.get_null_type()),
            Narrowing::IsUndefined => types.push(checker.get_undefined_type()),
            Narrowing::IsNullish => {
                types.push(checker.get_null_type());
                types.push(checker.get_undefined_type());
            }
            _ => return None,
        }
    }
    Some(checker.get_union_type_exported(&types))
}

/// The type of the parameter when `condition` holds, when TypeScript would infer a predicate for it: something was
/// narrowed away, and nothing of the narrowed type can fail the checks.
fn narrowed_type(
    checker: &mut Checker,
    declared: P<Type>,
    condition: &Condition,
) -> Option<P<Type>> {
    if declared.flags().intersects(TypeFlags::Boolean) {
        return None;
    }
    if declared.flags().intersects(TypeFlags::Unknown) {
        return narrowed_unknown(checker, condition);
    }
    let constituents: Vec<P<Type>> = if declared.flags().intersects(TypeFlags::Union) {
        declared.types().to_vec()
    } else {
        vec![declared]
    };
    let mut kept = Vec::with_capacity(constituents.len());
    for t in constituents.iter().copied() {
        if !has_definite_facts(t) {
            return None;
        }
        let mut passing = 0;
        for &narrowing in &condition.narrowings {
            if passes(checker, t, narrowing)? {
                passing += 1;
            }
        }
        let keep = if condition.any {
            passing > 0
        } else {
            passing == condition.narrowings.len()
        };
        if keep {
            kept.push(t);
        }
    }
    if kept.is_empty() || kept.len() == constituents.len() {
        return None;
    }
    Some(checker.get_union_type_exported(&kept))
}

/// Whether TypeScript would infer `predicate` itself if the annotation were removed, like its
/// `getTypePredicateFromBody`, for the narrowings it can be sure about.
fn is_inferable(checker: &mut Checker, function: P<Node>, predicate: P<Node>) -> bool {
    let predicate_node = predicate.as_type_predicate_node();
    if predicate_node.asserts_modifier().is_some() {
        return false;
    }
    let name = predicate_node.parameter_name();
    let Some(predicate_type) = predicate_node.type_() else {
        return false;
    };
    if !ast::is_identifier(name) || ast::get_function_flags(Some(function)) != FunctionFlags::Normal
    {
        return false;
    }
    let target = name.as_identifier().text();
    let Some(parameter) = function.parameters().iter().copied().find(|&parameter| {
        parameter.name().is_some_and(|parameter_name| {
            ast::is_identifier(parameter_name) && parameter_name.as_identifier().text() == target
        })
    }) else {
        return false;
    };
    if parameter
        .as_parameter_declaration()
        .dot_dot_dot_token()
        .is_some()
    {
        return false;
    }
    let Some(symbol) = parameter.symbol() else {
        return false;
    };
    let Some(expression) = single_return_expression(function) else {
        return false;
    };
    let Some(condition) = condition(checker, expression, symbol) else {
        return false;
    };
    let declared = checker.get_type_of_symbol(symbol);
    let Some(narrowed) = narrowed_type(checker, declared, &condition) else {
        return false;
    };
    let predicate_type = checker.get_type_from_type_node_exported(predicate_type);
    checker.is_type_identical_to(narrowed, predicate_type)
}

/// `filter` or `find` when `function` is passed directly to a call of a method with that name.
fn array_method_of_callback(function: P<Node>) -> Option<&'static str> {
    let call = estree_parent(function).filter(|&parent| ast::is_call_expression(parent))?;
    if !call
        .arguments()
        .iter()
        .any(|&arg| skip_parens(arg) == function)
    {
        return None;
    }
    let callee = skip_parens(call.as_call_expression().expression);
    let method = match callee.kind() {
        Kind::PropertyAccessExpression => {
            let name = callee.as_property_access_expression().name;
            ast::is_identifier(name).then(|| name.text())
        }
        Kind::ElementAccessExpression => {
            let key = skip_parens(callee.as_element_access_expression().argument_expression);
            ast::is_string_literal_like(key).then(|| key.text())
        }
        _ => None,
    }?;
    matches!(method, "filter" | "find").then_some(method)
}

/// Whether the parameter `predicate` narrows is declared as `unknown`.
fn narrows_unknown_parameter(checker: &mut Checker, function: P<Node>, predicate: P<Node>) -> bool {
    let name = predicate.as_type_predicate_node().parameter_name();
    if !ast::is_identifier(name) {
        return false;
    }
    let target = name.as_identifier().text();
    function
        .parameters()
        .iter()
        .find(|&&parameter| {
            parameter.name().is_some_and(|name| {
                ast::is_identifier(name) && name.as_identifier().text() == target
            })
        })
        .and_then(|&parameter| parameter.symbol())
        .is_some_and(|symbol| {
            checker
                .get_type_of_symbol(symbol)
                .flags()
                .intersects(TypeFlags::Unknown)
        })
}

impl RequireTypeGuardExplanation {
    /// The message with the most specific configured hint: `.filter()`/`.find()` callback, then `unknown` input, then
    /// the default one.
    fn missing_message(
        &self,
        checker: &mut Checker,
        function: P<Node>,
        predicate: P<Node>,
    ) -> String {
        let in_array = match array_method_of_callback(function) {
            Some("filter") => self.missing_message_in_filter.as_ref(),
            Some(_) => self.missing_message_in_find.as_ref(),
            None => None,
        };
        in_array
            .or_else(|| {
                self.missing_message_unknown_input
                    .as_ref()
                    .filter(|_| narrows_unknown_parameter(checker, function, predicate))
            })
            .unwrap_or(&self.missing_message)
            .clone()
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }

    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(predicate) = checked_type_predicate(node) else {
            return;
        };
        let (pos, end) = type_annotation_range(ctx, predicate);
        if is_inferable(ctx.checker, node, predicate) {
            let diagnostic = RuleDiagnostic {
                pos,
                end,
                message: RuleMessage::new("inferablePredicate", MESSAGE_INFERABLE),
                labeled_ranges: Vec::new(),
            };
            ctx.report_diagnostic_with_suggestions(diagnostic, |ctx| {
                vec![RuleSuggestion {
                    message: RuleMessage::new("removeAnnotation", MESSAGE_REMOVE_ANNOTATION),
                    fixes: vec![ctx.fix_remove_range(pos, end)],
                }]
            });
            return;
        }
        if self.has_explanation(ctx, node) {
            return;
        }
        let message = self.rule.missing_message(ctx.checker, node, predicate);
        ctx.report_range(pos, end, RuleMessage::new("missingExplanation", message));
    }
}
