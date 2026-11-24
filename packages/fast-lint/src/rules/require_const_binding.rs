use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{estree_parent, skip_parens};

pub const NAME: &str = "require-const-binding";

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    functions: Vec<String>,
}

pub struct RequireConstBinding {
    functions: FxHashSet<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    Ok(Box::new(RequireConstBinding {
        functions: options.functions.into_iter().collect(),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for RequireConstBinding {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static RequireConstBinding,
}

/// Whether `parent` calls `node`: `node()`, or `node.method()` via the property access `node.method`.
fn chained_call(node: P<Node>, parent: P<Node>) -> Option<P<Node>> {
    if ast::is_call_expression(parent) {
        return (skip_parens(parent.as_call_expression().expression()) == node).then_some(parent);
    }
    if ast::is_property_access_expression(parent) && parent.expression() == Some(node) {
        let call = estree_parent(parent)?;
        return (ast::is_call_expression(call)
            && skip_parens(call.as_call_expression().expression()) == parent)
            .then_some(call);
    }
    None
}

/// The outermost expression built from `call`: curried calls (`fn()()`), chained method calls (`fn().orNull()`) and
/// type-only wrappers (`as`, `satisfies`, `!`) are part of the same value.
fn bound_expression(call: P<Node>) -> P<Node> {
    let mut current = call;
    while let Some(parent) = estree_parent(current) {
        current = match parent.kind() {
            Kind::AsExpression | Kind::SatisfiesExpression | Kind::NonNullExpression => parent,
            _ => match chained_call(current, parent) {
                Some(call) => call,
                None => break,
            },
        };
    }
    current
}

fn is_const_binding_value(expression: P<Node>) -> bool {
    estree_parent(expression).is_some_and(|declaration| {
        declaration.kind() == Kind::VariableDeclaration
            && ast::is_identifier(declaration.as_variable_declaration().name())
            && declaration
                .initializer()
                .is_some_and(|initializer| skip_parens(initializer) == expression)
            && ast::is_var_const(declaration)
    })
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let callee = skip_parens(node.as_call_expression().expression());
        if !ast::is_identifier(callee) {
            return;
        }
        let name = callee.text();
        if !self.rule.functions.contains(name) || is_const_binding_value(bound_expression(node)) {
            return;
        }
        ctx.report_node(
            node,
            RuleMessage::new(
                "requireConstBinding",
                format!(
                    "`{name}` must be the value of its own `const` binding, declare it as a `const` and reference it"
                ),
            ),
        );
    }
}
