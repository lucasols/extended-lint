use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

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

/// The `key` attributes seen under one root (an array literal or JSX parent of keyed siblings, or a `.map()`-like
/// call), like ESLint-React's `keyedEntries` map values.
struct Entry {
    has_duplicate: bool,
    keys: Vec<P<Node>>,
}

struct Visitor {
    enabled: bool,
    entries: Vec<Entry>,
    entry_index: FxHashMap<P<Node>, usize>,
}

impl Visitor {
    /// `keyedEntries.set(root, entry)`: replaces the entry but keeps the root's first insertion order.
    fn set(&mut self, root: P<Node>, entry: Entry) {
        if let Some(&index) = self.entry_index.get(&root) {
            self.entries[index] = entry;
        } else {
            self.entry_index.insert(root, self.entries.len());
            self.entries.push(entry);
        }
    }

    fn check_attribute(&mut self, node: P<Node>) {
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
        if matches!(
            parent.kind(),
            Kind::ArrayLiteralExpression | Kind::JsxElement | Kind::JsxFragment
        ) {
            let prev_keys = self
                .entry_index
                .get(&parent)
                .map(|&index| std::mem::take(&mut self.entries[index].keys))
                .unwrap_or_default();
            let has_duplicate = prev_keys
                .iter()
                .any(|&prev_key| is_key_value_equal(prev_key, node));
            let mut keys = prev_keys;
            keys.push(node);
            self.set(
                parent,
                Entry {
                    has_duplicate,
                    keys,
                },
            );
            return;
        }
        let Some(call) = find_ancestor(jsx_element, is_array_map_call) else {
            return;
        };
        let Some(iter) = find_ancestor(jsx_element, |n| n == call || is_function(n)) else {
            return;
        };
        if !is_function(iter) {
            return;
        }
        let Some(&arg0) = call.arguments().first() else {
            return;
        };
        if skip_type_expressions(arg0) != iter {
            return;
        }
        let has_duplicate = node
            .as_jsx_attribute()
            .initializer()
            .is_some_and(ast::is_string_literal);
        self.set(
            call,
            Entry {
                has_duplicate,
                keys: vec![node],
            },
        );
    }

    fn report(&self, ctx: &mut Ctx) {
        for entry in &self.entries {
            if !entry.has_duplicate {
                continue;
            }
            for &key in &entry.keys {
                let message = format!(
                    "A key must be unique. '{}' is duplicated.",
                    node_text(ctx, key)
                );
                ctx.report_node(key, RuleMessage::new("noDuplicateKey", message));
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

/// ESLint-React's loose `AST.isArrayMapCall`: a call of a member named `map` or ending with `Map`.
fn is_array_map_call(node: P<Node>) -> bool {
    if node.kind() != Kind::CallExpression {
        return false;
    }
    member_identifier_property(skip_parens(node.as_call_expression().expression))
        .is_some_and(|name| name == "map" || name.ends_with("Map"))
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

/// ESLint-React's `isKeyValueEqual`: `AST.isNodeEqual` of the two attribute values, which for JSX attribute values
/// only holds for string literals with the same value.
fn is_key_value_equal(a: P<Node>, b: P<Node>) -> bool {
    let (Some(a), Some(b)) = (
        a.as_jsx_attribute().initializer(),
        b.as_jsx_attribute().initializer(),
    ) else {
        return false;
    };
    ast::is_string_literal(a) && ast::is_string_literal(b) && a.text() == b.text()
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
        self.check_attribute(node);
    }
}
