use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::resolve_reference;
use crate::utils::{estree_parent, member_identifier_property, node_text, skip_parens};

pub const NAME: &str = "no-duplicate-key";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoDuplicateKey;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoDuplicateKey))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::JsxAttribute),
    Listener::Enter(Kind::EndOfFile),
];

impl Rule for NoDuplicateKey {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            enabled: ctx.text().contains("key="),
            entries: Vec::new(),
            entry_index: FxHashMap::default(),
        })
    }
}

/// The value a `key` attribute compares by: a static string (string, number and expression-less template literals,
/// as React stringifies keys) or the whitespace-normalized source text of any other expression.
#[derive(PartialEq, Eq, Hash)]
enum KeyValue {
    Static(String),
    Expression(String),
}

/// The `key` attributes seen under one root: the sibling elements of an array literal or JSX parent, or the constant
/// keys of the elements an iteration callback returns.
struct Entry {
    is_callback: bool,
    keys: Vec<(P<Node>, KeyValue)>,
}

struct Visitor {
    enabled: bool,
    entries: Vec<Entry>,
    entry_index: FxHashMap<P<Node>, usize>,
}

impl Visitor {
    fn push(&mut self, root: P<Node>, is_callback: bool, key: P<Node>, value: KeyValue) {
        if let Some(&index) = self.entry_index.get(&root) {
            self.entries[index].keys.push((key, value));
        } else {
            self.entry_index.insert(root, self.entries.len());
            self.entries.push(Entry {
                is_callback,
                keys: vec![(key, value)],
            });
        }
    }

    fn check_attribute(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let name = node.as_jsx_attribute().name();
        if !ast::is_identifier(name) || name.text() != "key" {
            return;
        }
        let Some(jsx_element) = jsx_element_of(node) else {
            return;
        };
        let Some(parent) = estree_parent(jsx_element) else {
            return;
        };
        let Some(value) = key_value(ctx, node) else {
            return;
        };
        if matches!(
            parent.kind(),
            Kind::ArrayLiteralExpression | Kind::JsxElement | Kind::JsxFragment
        ) {
            self.push(parent, false, node, value);
            return;
        }
        let Some(call) = find_ancestor(jsx_element, |n| callback_argument_index(n).is_some())
        else {
            return;
        };
        let Some(iter) = find_ancestor(jsx_element, |n| n == call || is_function(n)) else {
            return;
        };
        if !is_function(iter) {
            return;
        }
        let Some(index) = callback_argument_index(call) else {
            return;
        };
        let Some(&callback) = call.arguments().get(index) else {
            return;
        };
        if skip_type_expressions(callback) != iter {
            return;
        }
        if matches!(value, KeyValue::Static(_)) || is_outer_const(ctx, node, iter) {
            self.push(call, true, node, value);
        }
    }

    fn report(&self, ctx: &mut Ctx) {
        for entry in &self.entries {
            let mut counts: FxHashMap<&KeyValue, usize> = FxHashMap::default();
            if !entry.is_callback {
                for (_, value) in &entry.keys {
                    *counts.entry(value).or_default() += 1;
                }
            }
            for (key, value) in &entry.keys {
                if !entry.is_callback && counts.get(value).is_none_or(|&count| count < 2) {
                    continue;
                }
                let message = format!(
                    "A key must be unique. '{}' is duplicated.",
                    node_text(ctx, *key)
                );
                ctx.report_node(*key, RuleMessage::new("noDuplicateKey", message));
            }
        }
    }
}

/// The ESTree `JSXElement` owning the attribute: the self-closing element, or the element of the opening tag.
fn jsx_element_of(attribute: P<Node>) -> Option<P<Node>> {
    let tag = attribute.parent()?.parent()?;
    match tag.kind() {
        Kind::JsxSelfClosingElement => Some(tag),
        Kind::JsxOpeningElement => tag.parent(),
        _ => None,
    }
}

/// ESLint-React's `AST.findParentNode`: the nearest ancestor passing `test`, stopping before the program.
fn find_ancestor(node: P<Node>, test: impl Fn(P<Node>) -> bool) -> Option<P<Node>> {
    let mut current = node.parent()?;
    while current.kind() != Kind::SourceFile {
        if test(current) {
            return Some(current);
        }
        current = current.parent()?;
    }
    None
}

/// The index of the callback argument of an iteration call: `0` for a call of a member named `map` or ending with
/// `Map` (ESLint-React's loose `AST.isArrayMapCall`), `1` for `Array.from(items, callback)`.
fn callback_argument_index(node: P<Node>) -> Option<usize> {
    if node.kind() != Kind::CallExpression {
        return None;
    }
    let callee = skip_parens(node.as_call_expression().expression);
    let name = member_identifier_property(callee)?;
    if name == "map" || name.ends_with("Map") {
        return Some(0);
    }
    let object = skip_parens(callee.expression()?);
    (name == "from" && ast::is_identifier(object) && object.text() == "Array").then_some(1)
}

/// ESTree's function nodes: arrows, function declarations and expressions, and the function expressions of methods,
/// accessors and constructors.
fn is_function(node: P<Node>) -> bool {
    matches!(
        node.kind(),
        Kind::ArrowFunction
            | Kind::FunctionDeclaration
            | Kind::FunctionExpression
            | Kind::MethodDeclaration
            | Kind::GetAccessor
            | Kind::SetAccessor
            | Kind::Constructor
    )
}

/// Parentheses plus ESLint-React's `AST.getUnderlyingExpression` (`as`, `satisfies`, `!`, `<T>x`, `f<T>`).
fn skip_type_expressions(mut node: P<Node>) -> P<Node> {
    loop {
        node = skip_parens(node);
        match node.kind() {
            Kind::AsExpression => node = node.as_as_expression().expression,
            Kind::SatisfiesExpression => node = node.as_satisfies_expression().expression,
            Kind::NonNullExpression => node = node.as_non_null_expression().expression,
            Kind::TypeAssertionExpression => node = node.as_type_assertion().expression,
            Kind::ExpressionWithTypeArguments => {
                node = node.as_expression_with_type_arguments().expression;
            }
            _ => return node,
        }
    }
}

/// The value of a `key` attribute, `None` for `key` without a value or with an empty expression.
fn key_value(ctx: &Ctx, attribute: P<Node>) -> Option<KeyValue> {
    let initializer = attribute.as_jsx_attribute().initializer()?;
    if ast::is_string_literal(initializer) {
        return Some(KeyValue::Static(initializer.text().to_string()));
    }
    if !ast::is_jsx_expression(initializer) {
        return None;
    }
    let expression = skip_parens(initializer.as_jsx_expression().expression?);
    if ast::is_string_literal(expression) || ast::is_no_substitution_template_literal(expression) {
        return Some(KeyValue::Static(expression.text().to_string()));
    }
    if ast::is_numeric_literal(expression) {
        let text = expression.text();
        let number = text
            .replace('_', "")
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .map_or_else(|| text.to_string(), |number| number.to_string());
        return Some(KeyValue::Static(number));
    }
    Some(KeyValue::Expression(normalize_whitespace(node_text(
        ctx, expression,
    ))))
}

/// `text` without whitespace, except for one space between two identifier characters (`typeof a`, `a in b`).
fn normalize_whitespace(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;
    for char in text.chars() {
        if char.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space
            && is_identifier_char(char)
            && normalized
                .chars()
                .next_back()
                .is_some_and(is_identifier_char)
        {
            normalized.push(' ');
        }
        pending_space = false;
        normalized.push(char);
    }
    normalized
}

fn is_identifier_char(char: char) -> bool {
    char.is_alphanumeric() || char == '_' || char == '$'
}

/// Whether the `key` attribute's value is an identifier bound to a `const` declared outside `callback`, so it has the
/// same value for every element the callback returns.
fn is_outer_const(ctx: &mut Ctx, attribute: P<Node>, callback: P<Node>) -> bool {
    let Some(initializer) = attribute.as_jsx_attribute().initializer() else {
        return false;
    };
    if !ast::is_jsx_expression(initializer) {
        return false;
    }
    let Some(expression) = initializer.as_jsx_expression().expression else {
        return false;
    };
    let identifier = skip_parens(expression);
    if !ast::is_identifier(identifier) {
        return false;
    }
    let Some(symbol) = resolve_reference(ctx.checker, identifier) else {
        return false;
    };
    let [declaration] = symbol.declarations() else {
        return false;
    };
    let mut declaration = *declaration;
    while matches!(
        declaration.kind(),
        Kind::BindingElement | Kind::ObjectBindingPattern | Kind::ArrayBindingPattern
    ) {
        let Some(parent) = declaration.parent() else {
            return false;
        };
        declaration = parent;
    }
    declaration.kind() == Kind::VariableDeclaration
        && ast::is_var_const(declaration)
        && !is_descendant_of(declaration, callback)
}

fn is_descendant_of(node: P<Node>, ancestor: P<Node>) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent == ancestor {
            return true;
        }
        current = parent.parent();
    }
    false
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        if self.enabled {
            LISTENERS
        } else {
            &[]
        }
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        if listener == Listener::Enter(Kind::EndOfFile) {
            self.report(ctx);
            return;
        }
        self.check_attribute(ctx, node);
    }
}
