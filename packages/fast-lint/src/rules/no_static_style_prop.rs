use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::skip_parens;

pub const NAME: &str = "no-static-style-prop";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    #[serde(default)]
    custom_message: Option<String>,
}

pub struct NoStaticStyleProp {
    message: String,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let custom = options
        .custom_message
        .filter(|message| !message.is_empty())
        .map(|message| format!(" {message}"))
        .unwrap_or_default();
    Ok(Box::new(NoStaticStyleProp {
        message: format!("Static style props are not allowed use css instead.{custom}"),
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::JsxAttribute)];

impl Rule for NoStaticStyleProp {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoStaticStyleProp,
}

/// Whether `node` is a value fixed at write time, with ESTree's node shapes: a literal, a template without
/// substitutions, or an object/array made only of those.
fn is_static_value(node: P<Node>) -> bool {
    let node = skip_parens(node);
    match node.kind() {
        Kind::StringLiteral
        | Kind::NumericLiteral
        | Kind::BigIntLiteral
        | Kind::RegularExpressionLiteral
        | Kind::TrueKeyword
        | Kind::FalseKeyword
        | Kind::NullKeyword
        | Kind::NoSubstitutionTemplateLiteral => true,
        Kind::ObjectLiteralExpression => node.properties().iter().all(|&property| {
            if !ast::is_property_assignment(property) {
                return false;
            }
            let assignment = property.as_property_assignment();
            if ast::is_computed_property_name(assignment.name()) {
                return false;
            }
            is_static_value(assignment.initializer())
        }),
        Kind::ArrayLiteralExpression => node.elements().iter().all(|&element| {
            ast::is_omitted_expression(element)
                || (!ast::is_spread_element(element) && is_static_value(element))
        }),
        _ => false,
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let attribute = node.as_jsx_attribute();
        let name = attribute.name();
        if !ast::is_identifier(name) || name.text() != "style" {
            return;
        }
        let Some(value) = attribute.initializer() else {
            return;
        };
        let value_to_check = if ast::is_jsx_expression(value) {
            value.as_jsx_expression().expression()
        } else if ast::is_string_literal(value) {
            Some(value)
        } else {
            None
        };
        if value_to_check.is_some_and(is_static_value) {
            ctx.report_node(
                node,
                RuleMessage::new("noStaticStyleProp", self.rule.message.clone()),
            );
        }
    }
}
