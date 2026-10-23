use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::skip_type_parens;

pub const NAME: &str = "no-non-camel-case-functions";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoNonCamelCaseFunctions;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoNonCamelCaseFunctions))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::FunctionDeclaration)];

impl Rule for NoNonCamelCaseFunctions {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

/// `^\$?[a-z][a-zA-Z0-9]*$`
fn is_camel_case(name: &str) -> bool {
    let rest = name.strip_prefix('$').unwrap_or(name);
    let mut chars = rest.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase()) && chars.all(|c| c.is_ascii_alphanumeric())
}

/// `JSX.Element`
fn is_jsx_element_type(type_node: P<Node>) -> bool {
    if !ast::is_type_reference_node(type_node) {
        return false;
    }
    let type_name = type_node.as_type_reference_node().type_name();
    if !ast::is_qualified_name(type_name) {
        return false;
    }
    let qualified = type_name.as_qualified_name();
    ast::is_identifier(qualified.left())
        && qualified.left().text() == "JSX"
        && qualified.right().text() == "Element"
}

fn has_jsx_return_type(node: P<Node>) -> bool {
    let Some(return_type) = node.type_node() else {
        return false;
    };
    let return_type = skip_type_parens(return_type);
    if ast::is_union_type_node(return_type) {
        return return_type
            .as_union_type_node()
            .types()
            .nodes()
            .iter()
            .any(|&member| is_jsx_element_type(skip_type_parens(member)));
    }
    is_jsx_element_type(return_type)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.body().is_none() {
            return;
        }
        let Some(name) = node.as_function_declaration().name else {
            return;
        };
        let function_name = name.text();
        if is_camel_case(function_name) || has_jsx_return_type(node) {
            return;
        }
        ctx.report_node(
            name,
            RuleMessage::new(
                "nonCamelCaseFunction",
                format!(
                    "Function name \"{function_name}\" should be in camelCase format. If this fn is a React \
                     component and can't use the `FC` type, add a explicit \"JSX.Element\" return type to it"
                ),
            ),
        );
    }
}
