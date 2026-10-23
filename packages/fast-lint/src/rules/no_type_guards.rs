use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{member_identifier_property, skip_parens, type_annotation_range};

pub const NAME: &str = "no-type-guards";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct AlternativeMsgs {
    in_array_find: Option<String>,
    in_array_filter: Option<String>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    alternative_msgs: Option<AlternativeMsgs>,
    #[serde(rename = "__dev_simulateFileName")]
    dev_simulate_file_name: Option<String>,
}

pub struct NoTypeGuards {
    alternative_msgs: Option<AlternativeMsgs>,
    dev_simulate_file_name: Option<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    Ok(Box::new(NoTypeGuards {
        alternative_msgs: options.alternative_msgs,
        dev_simulate_file_name: options.dev_simulate_file_name,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::ArrowFunction),
    Listener::Enter(Kind::MethodDeclaration),
    Listener::Enter(Kind::GetAccessor),
    Listener::Enter(Kind::SetAccessor),
];

impl Rule for NoTypeGuards {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let file_name = self
            .dev_simulate_file_name
            .as_deref()
            .unwrap_or_else(|| ctx.file.file_name());
        Box::new(Visitor {
            rule: self,
            enabled: !is_type_guards_file(file_name),
        })
    }
}

struct Visitor {
    rule: &'static NoTypeGuards,
    enabled: bool,
}

/// `/(typeGuards|type-guards)\.(ts|tsx)$/`
fn is_type_guards_file(file_name: &str) -> bool {
    [
        "typeGuards.ts",
        "typeGuards.tsx",
        "type-guards.ts",
        "type-guards.tsx",
    ]
    .iter()
    .any(|suffix| file_name.ends_with(suffix))
}

/// The functions whose return type ESLint's `FunctionDeclaration > TSTypeAnnotation`,
/// `ArrowFunctionExpression > TSTypeAnnotation` and `MethodDefinition > FunctionExpression > TSTypeAnnotation`
/// selectors reach: declarations and class methods with a body (bodiless ones are `TSDeclareFunction` and
/// `TSEmptyBodyFunctionExpression` in ESTree), and arrow functions. Object literal methods are `Property` values.
fn is_checked_function(node: P<Node>) -> bool {
    match node.kind() {
        Kind::ArrowFunction => true,
        Kind::FunctionDeclaration => node.body().is_some(),
        _ => node.body().is_some() && node.parent().is_some_and(ast::is_class_like),
    }
}

/// `filter` or `find` when `node` is inside a call to a method with that name.
fn enclosing_filter_or_find(node: P<Node>) -> Option<&'static str> {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        if ast::is_call_expression(ancestor) {
            let callee = skip_parens(ancestor.as_call_expression().expression);
            if let Some(method @ ("filter" | "find")) = member_identifier_property(callee) {
                return Some(method);
            }
        }
        current = ancestor.parent();
    }
    None
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
        let Some(return_type) = node.type_node() else {
            return;
        };
        if !ast::is_type_predicate_node(return_type)
            || return_type
                .as_type_predicate_node()
                .asserts_modifier()
                .is_some()
            || !is_checked_function(node)
        {
            return;
        }
        let (pos, end) = type_annotation_range(ctx, return_type);

        if let Some(alternative_msgs) = &self.rule.alternative_msgs {
            let alternative = match enclosing_filter_or_find(node) {
                Some("filter") => alternative_msgs
                    .in_array_filter
                    .as_deref()
                    .map(|message| ("useFilterWithTypeCheck", message)),
                Some(_) => alternative_msgs
                    .in_array_find
                    .as_deref()
                    .map(|message| ("useFindWithTypeCheck", message)),
                None => None,
            };
            if let Some((id, message)) = alternative.filter(|(_, message)| !message.is_empty()) {
                ctx.report_range(pos, end, RuleMessage::new(id, message.to_string()));
                return;
            }
        }

        ctx.report_range(
            pos,
            end,
            RuleMessage::new(
                "typeGuardNotAllowed",
                "Check if the type guard can be inferred by typescript, in most cases it can, e.g. \
                 `.filter((nullable) => nullable !== null)`. If not, type guards are only allowed in \
                 *.typeGuards.(ts|tsx) or *.type-guards.(ts|tsx) files",
            ),
        );
    }
}
