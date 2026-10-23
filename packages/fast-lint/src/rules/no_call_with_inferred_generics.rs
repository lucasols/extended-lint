use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{skip_parens, skip_type_parens};

pub const NAME: &str = "no-call-with-inferred-generics";

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct FunctionConfig {
    name: String,
    min_generics: Option<usize>,
    #[serde(default)]
    allow_any: bool,
    disallow_types: Option<Vec<String>>,
    #[serde(default)]
    disallow_type_of: bool,
    #[serde(default)]
    disallow_unknown: bool,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    functions: Vec<FunctionConfig>,
    disallow_types: Option<Vec<String>>,
    /// Accepted by the ESLint rule's schema but never used.
    #[serde(rename = "anyAliases")]
    _any_aliases: Option<serde::de::IgnoredAny>,
}

pub struct NoCallWithInferredGenerics {
    functions: FxHashMap<String, FunctionConfig>,
    disallow_types: Option<Vec<String>>,
    has_disallow_type_of: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let has_disallow_type_of = options.functions.iter().any(|f| f.disallow_type_of);
    Ok(Box::new(NoCallWithInferredGenerics {
        functions: options
            .functions
            .into_iter()
            .map(|config| (config.name.clone(), config))
            .collect(),
        disallow_types: options.disallow_types,
        has_disallow_type_of,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::TypeAliasDeclaration),
    Listener::Enter(Kind::CallExpression),
];

impl Rule for NoCallWithInferredGenerics {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            rule: self,
            type_of_aliases: FxHashSet::default(),
        })
    }
}

struct Visitor {
    rule: &'static NoCallWithInferredGenerics,
    /// Type aliases seen so far whose type is a `typeof` query.
    type_of_aliases: FxHashSet<&'static str>,
}

/// `typeof x`, also `typeof import('x')`, which ESTree represents as a `TSTypeQuery` too.
fn is_type_query(type_node: P<Node>) -> bool {
    ast::is_type_query_node(type_node)
        || (type_node.kind() == Kind::ImportType && type_node.as_import_type_node().is_type_of())
}

/// The name of a type reference to a plain identifier, like `User` (not `ns.User`).
fn type_reference_identifier(type_node: P<Node>) -> Option<&'static str> {
    if !ast::is_type_reference_node(type_node) {
        return None;
    }
    let type_name = type_node.as_type_reference_node().type_name();
    ast::is_identifier(type_name).then(|| type_name.text())
}

fn function_message(id: &'static str, function_name: &str, text: &str) -> RuleMessage {
    RuleMessage::new(
        id,
        format!("Function '{function_name}' should not be called with {text} in generics"),
    )
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.kind() == Kind::TypeAliasDeclaration {
            if self.rule.has_disallow_type_of
                && node
                    .type_node()
                    .map(skip_type_parens)
                    .is_some_and(is_type_query)
            {
                self.type_of_aliases
                    .insert(node.as_type_alias_declaration().name.text());
            }
            return;
        }

        let callee = skip_parens(node.as_call_expression().expression);
        if !ast::is_identifier(callee) {
            return;
        }
        let function_name = callee.text();
        let Some(config) = self.rule.functions.get(function_name) else {
            return;
        };
        let min_generics = config.min_generics.unwrap_or(1);
        let disallow_types = config
            .disallow_types
            .as_ref()
            .or(self.rule.disallow_types.as_ref());
        let type_arguments: Vec<P<Node>> = node
            .type_arguments()
            .iter()
            .map(|&type_argument| skip_type_parens(type_argument))
            .collect();

        if type_arguments.len() < min_generics {
            ctx.report_node(
                node,
                RuleMessage::new(
                    "missingGenericDeclaration",
                    format!(
                        "Function '{function_name}' should be called with at least {min_generics} generic(s) (ex: \
                         `fn<Generic>()`) defined"
                    ),
                ),
            );
        }

        if config.allow_any && disallow_types.is_none() {
            return;
        }

        let has_disallowed_type = type_arguments.iter().any(|&type_argument| {
            (!config.allow_any && type_argument.kind() == Kind::AnyKeyword)
                || disallow_types.is_some_and(|disallowed| {
                    type_reference_identifier(type_argument)
                        .is_some_and(|name| disallowed.iter().any(|d| d == name))
                })
        });
        if has_disallowed_type {
            ctx.report_node(
                node,
                function_message("anyUsedInGenerics", function_name, "'any'"),
            );
        }

        if config.disallow_unknown
            && type_arguments
                .iter()
                .any(|type_argument| type_argument.kind() == Kind::UnknownKeyword)
        {
            ctx.report_node(
                node,
                function_message("unknownUsedInGenerics", function_name, "'unknown'"),
            );
        }

        if config.disallow_type_of
            && type_arguments.iter().any(|&type_argument| {
                is_type_query(type_argument)
                    || type_reference_identifier(type_argument)
                        .is_some_and(|name| self.type_of_aliases.contains(name))
            })
        {
            ctx.report_node(
                node,
                function_message("typeOfUsedInGenerics", function_name, "'typeof'"),
            );
        }
    }
}
