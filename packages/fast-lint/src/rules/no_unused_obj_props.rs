use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_checker::Checker;
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::{resolve_reference, ReferenceIndex};
use crate::utils::{estree_parent, skip_parens, string_literal_values_of_type};

pub const NAME: &str = "no-unused-obj-props";

pub struct NoUnusedObjProps;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<NoOptions>(options)?;
    Ok(Box::new(NoUnusedObjProps))
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct NoOptions {}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::VariableDeclaration),
    Listener::Enter(Kind::FunctionDeclaration),
];

impl Rule for NoUnusedObjProps {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { index: None })
    }
}

struct Visitor {
    index: Option<ReferenceIndex>,
}

type Keys = FxHashSet<&'static str>;

/// `export const obj = ...`.
fn is_exported_declarator(declarator: P<Node>) -> bool {
    declarator
        .parent()
        .and_then(|list| list.parent())
        .is_some_and(|statement| {
            statement.kind() == Kind::VariableStatement
                && ast::has_syntactic_modifier(statement, ModifierFlags::Export)
        })
}

/// The object literal of an initializer, looking through parentheses and `as const`.
fn object_literal(expression: P<Node>) -> Option<P<Node>> {
    let mut current = skip_parens(expression);
    if ast::is_const_assertion(current) {
        current = skip_parens(current.expression()?);
    }
    (current.kind() == Kind::ObjectLiteralExpression).then_some(current)
}

fn literal_text(node: P<Node>) -> Option<&'static str> {
    matches!(
        node.kind(),
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral | Kind::NumericLiteral
    )
    .then(|| node.text())
}

/// The key of a property name: an identifier, a string or numeric literal, or a computed literal.
fn static_key(name: P<Node>) -> Option<&'static str> {
    match name.kind() {
        Kind::Identifier => Some(name.text()),
        Kind::ComputedPropertyName => literal_text(skip_parens(name.expression()?)),
        _ => literal_text(name),
    }
}

/// The keys an expression used as a computed key may have: a literal, or a string literal (union) type.
fn dynamic_keys(checker: &mut Checker, expression: P<Node>) -> Option<Vec<&'static str>> {
    let expression = skip_parens(expression);
    if let Some(key) = literal_text(expression) {
        return Some(vec![key]);
    }
    let t = checker.get_type_at_location(expression);
    string_literal_values_of_type(t)
}

/// The key of an object literal property the rule can report. Spreads, accessors (which may have side effects),
/// `__proto__` and dynamic computed keys are not tracked.
fn tracked_key(property: P<Node>) -> Option<&'static str> {
    if !matches!(
        property.kind(),
        Kind::PropertyAssignment | Kind::ShorthandPropertyAssignment | Kind::MethodDeclaration
    ) {
        return None;
    }
    let name = property.name()?;
    let key = static_key(name)?;
    let is_prototype_setter = property.kind() == Kind::PropertyAssignment
        && name.kind() != Kind::ComputedPropertyName
        && key == "__proto__";
    (!is_prototype_setter).then_some(key)
}

/// The tracked properties of an object literal by key, a later duplicate replacing an earlier one.
fn tracked_properties(object: P<Node>) -> Vec<(&'static str, P<Node>)> {
    let mut properties: Vec<(&'static str, P<Node>)> = Vec::new();
    for &property in object.properties() {
        let Some(key) = tracked_key(property) else {
            continue;
        };
        match properties.iter_mut().find(|(name, _)| *name == key) {
            Some(entry) => entry.1 = property,
            None => properties.push((key, property)),
        }
    }
    properties
}

fn is_this_boundary(node: P<Node>) -> bool {
    matches!(
        node.kind(),
        Kind::FunctionDeclaration
            | Kind::FunctionExpression
            | Kind::MethodDeclaration
            | Kind::GetAccessor
            | Kind::SetAccessor
            | Kind::Constructor
            | Kind::ClassDeclaration
            | Kind::ClassExpression
    )
}

/// Whether `node` uses `this` or `super` of its own function (arrow functions included).
fn uses_own_this(node: P<Node>) -> bool {
    fn visit(node: P<Node>, found: &mut bool) -> bool {
        if matches!(node.kind(), Kind::ThisKeyword | Kind::SuperKeyword) {
            *found = true;
            return true;
        }
        if is_this_boundary(node) {
            return false;
        }
        node.for_each_child(&mut |child| visit(child, found))
    }
    let mut found = false;
    node.for_each_child(&mut |child| visit(child, &mut found));
    found
}

/// Whether the function an identifier refers to (a function declaration, or a variable holding a function
/// expression) uses `this`.
fn referenced_function_uses_this(checker: &mut Checker, identifier: P<Node>) -> bool {
    let Some(symbol) = resolve_reference(checker, identifier) else {
        return false;
    };
    symbol.declarations().iter().any(|&declaration| {
        let function = match declaration.kind() {
            Kind::FunctionDeclaration => Some(declaration),
            Kind::VariableDeclaration => declaration
                .initializer()
                .map(skip_parens)
                .filter(|value| value.kind() == Kind::FunctionExpression),
            _ => None,
        };
        function.is_some_and(uses_own_this)
    })
}

/// Whether a method, an accessor or a `function` value of the object uses `this`, which can read any property.
fn has_property_using_this(checker: &mut Checker, object: P<Node>) -> bool {
    object
        .properties()
        .iter()
        .any(|&property| match property.kind() {
            Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor => {
                uses_own_this(property)
            }
            Kind::PropertyAssignment => {
                property
                    .initializer()
                    .map(skip_parens)
                    .is_some_and(|value| match value.kind() {
                        Kind::FunctionExpression => uses_own_this(value),
                        Kind::Identifier => referenced_function_uses_this(checker, value),
                        _ => false,
                    })
            }
            Kind::ShorthandPropertyAssignment => property
                .name()
                .is_some_and(|name| referenced_function_uses_this(checker, name)),
            _ => false,
        })
}

/// The keys an object binding pattern reads, or `None` when it has a rest element or a dynamic key.
fn binding_pattern_keys(checker: &mut Checker, pattern: P<Node>) -> Option<Vec<&'static str>> {
    let mut keys = Vec::new();
    for &element in pattern.elements() {
        if element.as_binding_element().dot_dot_dot_token().is_some() {
            return None;
        }
        match element.property_name() {
            Some(name) if name.kind() == Kind::ComputedPropertyName => {
                keys.extend(dynamic_keys(checker, name.expression()?)?);
            }
            Some(name) => keys.push(static_key(name)?),
            None => {
                let name = element
                    .name()
                    .filter(|name| name.kind() == Kind::Identifier)?;
                keys.push(name.text());
            }
        }
    }
    Some(keys)
}

/// The keys an object destructuring assignment target (`({ a, b: c } = obj)`) reads.
fn assignment_pattern_keys(checker: &mut Checker, pattern: P<Node>) -> Option<Vec<&'static str>> {
    let mut keys = Vec::new();
    for &property in pattern.properties() {
        match property.kind() {
            Kind::ShorthandPropertyAssignment => keys.push(property.name()?.text()),
            Kind::PropertyAssignment => {
                let name = property.name()?;
                if name.kind() == Kind::ComputedPropertyName {
                    keys.extend(dynamic_keys(checker, name.expression()?)?);
                } else {
                    keys.push(static_key(name)?);
                }
            }
            _ => return None,
        }
    }
    Some(keys)
}

/// The keys a member access on the object reads: none for a plain assignment target, `None` for a dynamic key.
fn member_access_keys(checker: &mut Checker, member: P<Node>) -> Option<Vec<&'static str>> {
    let keys = match member.kind() {
        Kind::PropertyAccessExpression => {
            let name = member.as_property_access_expression().name;
            (name.kind() == Kind::Identifier).then(|| vec![name.text()])?
        }
        _ => dynamic_keys(
            checker,
            member.as_element_access_expression().argument_expression,
        )?,
    };
    Some(if ast::is_write_only_access(member) {
        Vec::new()
    } else {
        keys
    })
}

/// The keys a use of the object reads, or `None` when the object escapes (passed, returned, spread, aliased,
/// typed with `typeof`, exported, enumerated...).
fn use_keys(checker: &mut Checker, identifier: P<Node>) -> Option<Vec<&'static str>> {
    let parent = estree_parent(identifier)?;
    match parent.kind() {
        Kind::PropertyAccessExpression | Kind::ElementAccessExpression
            if parent.expression().map(skip_parens) == Some(identifier) =>
        {
            member_access_keys(checker, parent)
        }
        Kind::VariableDeclaration if parent.initializer().map(skip_parens) == Some(identifier) => {
            let pattern = parent
                .name()
                .filter(|pattern| pattern.kind() == Kind::ObjectBindingPattern)?;
            binding_pattern_keys(checker, pattern)
        }
        Kind::BinaryExpression => {
            let binary = parent.as_binary_expression();
            let pattern = skip_parens(binary.left);
            if binary.operator_token.kind() != Kind::EqualsToken
                || skip_parens(binary.right()) != identifier
                || pattern.kind() != Kind::ObjectLiteralExpression
            {
                return None;
            }
            assignment_pattern_keys(checker, pattern)
        }
        _ => None,
    }
}

/// The keys read through the variable declared by `name`, or `None` when it escapes, is reassigned or is unused.
fn read_keys(ctx: &mut Ctx, index: &ReferenceIndex, name: P<Node>) -> Option<Keys> {
    let references = index.references_of_binding(ctx.checker, name);
    let mut keys = Keys::default();
    let mut has_uses = false;
    for reference in references {
        if reference.init {
            continue;
        }
        if reference.is_write() {
            return None;
        }
        has_uses = true;
        keys.extend(use_keys(ctx.checker, reference.identifier)?);
    }
    has_uses.then_some(keys)
}

/// The object literals a function returns, when every `return` of its own returns one.
fn returned_objects(function: P<Node>) -> Option<Vec<P<Node>>> {
    fn visit(node: P<Node>, objects: &mut Vec<P<Node>>, other: &mut bool) -> bool {
        if *other {
            return true;
        }
        if ast::is_function_like(node) || ast::is_class_like(node) {
            return false;
        }
        if node.kind() == Kind::ReturnStatement {
            match node.expression().and_then(object_literal) {
                Some(object) => objects.push(object),
                None => *other = true,
            }
            return *other;
        }
        node.for_each_child(&mut |child| visit(child, objects, other))
    }
    let body = function.body()?;
    if body.kind() != Kind::Block {
        return object_literal(body).map(|object| vec![object]);
    }
    let mut objects = Vec::new();
    let mut other = false;
    body.for_each_child(&mut |child| visit(child, &mut objects, &mut other));
    (!other && !objects.is_empty()).then_some(objects)
}

/// A function whose callers can be followed: not async or a generator and without a return type.
fn is_plain_function(function: P<Node>) -> bool {
    let flags = ast::get_function_flags(Some(function));
    !flags.intersects(ast::FunctionFlags::Async | ast::FunctionFlags::Generator)
        && function.type_node().is_none()
}

impl Visitor {
    fn check_object_variable(&mut self, ctx: &mut Ctx, name: P<Node>, object: P<Node>) {
        let properties = tracked_properties(object);
        if properties.is_empty() || has_property_using_this(ctx.checker, object) {
            return;
        }
        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        if let Some(keys) = read_keys(ctx, index, name) {
            report_unused(ctx, &properties, &keys, false);
        }
    }

    /// Reports the properties of the objects `function` returns that none of its callers read. Every reference to
    /// the function must be a call initializing a `const`/`let` (destructured without rest, or whose variable only
    /// reads properties).
    fn check_returning_function(&mut self, ctx: &mut Ctx, name: P<Node>, function: P<Node>) {
        if !is_plain_function(function) {
            return;
        }
        let Some(objects) = returned_objects(function) else {
            return;
        };
        if objects
            .iter()
            .any(|&object| has_property_using_this(ctx.checker, object))
        {
            return;
        }
        let properties: Vec<Vec<(&'static str, P<Node>)>> = objects
            .iter()
            .map(|&object| tracked_properties(object))
            .collect();
        if properties.iter().all(Vec::is_empty) {
            return;
        }
        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        let mut keys = Keys::default();
        let mut has_calls = false;
        for reference in index.references_of_binding(ctx.checker, name) {
            if reference.init {
                continue;
            }
            if reference.is_write() {
                return;
            }
            let Some(call_keys) = call_site_keys(ctx, index, reference.identifier) else {
                return;
            };
            has_calls = true;
            keys.extend(call_keys);
        }
        if !has_calls {
            return;
        }
        for properties in &properties {
            report_unused(ctx, properties, &keys, true);
        }
    }
}

/// The keys read from the result of a call to the function `identifier` refers to: `const { a } = f()` or
/// `const result = f()` followed by property reads.
fn call_site_keys(ctx: &mut Ctx, index: &ReferenceIndex, identifier: P<Node>) -> Option<Keys> {
    let call = estree_parent(identifier)?;
    if call.kind() != Kind::CallExpression
        || skip_parens(call.as_call_expression().expression) != identifier
    {
        return None;
    }
    let declaration = estree_parent(call)?;
    let is_plain_initializer = declaration.kind() == Kind::VariableDeclaration
        && declaration.initializer().map(skip_parens) == Some(call)
        && declaration.type_node().is_none()
        && !is_exported_declarator(declaration);
    if !is_plain_initializer {
        return None;
    }
    let name = declaration.name()?;
    match name.kind() {
        Kind::ObjectBindingPattern => Some(
            binding_pattern_keys(ctx.checker, name)?
                .into_iter()
                .collect(),
        ),
        Kind::Identifier => read_keys(ctx, index, name),
        _ => None,
    }
}

fn report_unused(
    ctx: &mut Ctx,
    properties: &[(&'static str, P<Node>)],
    keys: &Keys,
    returned: bool,
) {
    for &(key, property) in properties {
        if keys.contains(key) {
            continue;
        }
        let message = if returned {
            RuleMessage::new(
                "unusedReturnObjectProperty",
                format!(
                    "The object property \"{key}\" returned by this function is not being used"
                ),
            )
        } else {
            RuleMessage::new(
                "unusedObjectProperty",
                format!("The object property \"{key}\" is not being used"),
            )
        };
        ctx.report_node(property, message);
    }
}

/// Whether the function declaration can be referenced from outside the file or has overloads.
fn is_exported_or_overloaded(function: P<Node>) -> bool {
    ast::has_syntactic_modifier(function, ModifierFlags::Export | ModifierFlags::Default)
        || function.body().is_none()
        || function
            .symbol()
            .is_some_and(|symbol| symbol.declarations().len() > 1)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(name) = node.name().filter(|name| name.kind() == Kind::Identifier) else {
            return;
        };
        if node.kind() == Kind::FunctionDeclaration {
            if !is_exported_or_overloaded(node) {
                self.check_returning_function(ctx, name, node);
            }
            return;
        }
        let Some(init) = node.initializer() else {
            return;
        };
        if node.type_node().is_some() || is_exported_declarator(node) {
            return;
        }
        if let Some(object) = object_literal(init) {
            self.check_object_variable(ctx, name, object);
            return;
        }
        let function = skip_parens(init);
        let is_anonymous_function = function.kind() == Kind::ArrowFunction
            || (function.kind() == Kind::FunctionExpression && function.name().is_none());
        if is_anonymous_function {
            self.check_returning_function(ctx, name, function);
        }
    }
}
