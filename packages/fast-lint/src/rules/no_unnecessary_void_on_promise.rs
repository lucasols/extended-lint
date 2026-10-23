use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, SignatureKind, Type, TypeFlags};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};

use crate::utils::{is_in_optional_chain, node_text, replacement_at, skip_parens};

pub const NAME: &str = "no-unnecessary-void-on-promise";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoUnnecessaryVoidOnPromise;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoUnnecessaryVoidOnPromise))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VoidExpression)];

impl Rule for NoUnnecessaryVoidOnPromise {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn is_promise_like(checker: &mut Checker, t: P<Type>) -> bool {
    if t.flags().intersects(TypeFlags::Any | TypeFlags::Unknown) {
        return false;
    }
    if t.flags().intersects(TypeFlags::Union) {
        return t.types().iter().any(|&part| is_promise_like(checker, part));
    }
    let Some(then_symbol) = checker.get_property_of_type(t, "then") else {
        return false;
    };
    let then_type = checker.get_type_of_symbol(then_symbol);
    !checker
        .get_signatures_of_type(then_type, SignatureKind::Call)
        .is_empty()
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(expression) = node.expression() else {
            return;
        };
        let argument = skip_parens(expression);
        if !ast::is_call_expression(argument) || is_in_optional_chain(argument) {
            return;
        }
        let t = ctx.checker.get_type_at_location(argument);
        if is_promise_like(ctx.checker, t) {
            return;
        }
        let argument_text = node_text(ctx, argument);
        ctx.report_node_with_suggestions(
            node,
            RuleMessage::new(
                "unnecessaryVoid",
                "Unnecessary void operator on non-Promise function call",
            ),
            |ctx| {
                let replacement = replacement_at(ctx, node, argument_text.to_string());
                vec![RuleSuggestion {
                    message: RuleMessage::new("removeVoid", "Remove void operator"),
                    fixes: vec![ctx.fix_replace(node, replacement)],
                }]
            },
        );
    }
}
