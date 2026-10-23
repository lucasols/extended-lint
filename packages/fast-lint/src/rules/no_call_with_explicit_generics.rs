use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{member_identifier_property, skip_parens};

pub const NAME: &str = "no-call-with-explicit-generics";

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum FunctionEntry {
    Name(String),
    WithMessage(FunctionWithMessage),
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionWithMessage {
    name: String,
    message: String,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Options {
    functions: Vec<FunctionEntry>,
}

pub struct NoCallWithExplicitGenerics {
    exact_matches: FxHashMap<String, Option<String>>,
    wildcard_matches: FxHashMap<String, Option<String>>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut exact_matches = FxHashMap::default();
    let mut wildcard_matches = FxHashMap::default();
    for entry in options.functions {
        let (name, message) = match entry {
            FunctionEntry::Name(name) => (name, None),
            FunctionEntry::WithMessage(entry) => (entry.name, Some(entry.message)),
        };
        match name.strip_prefix("*.") {
            Some(method) => wildcard_matches.insert(method.to_string(), message),
            None => exact_matches.insert(name, message),
        };
    }
    Ok(Box::new(NoCallWithExplicitGenerics {
        exact_matches,
        wildcard_matches,
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for NoCallWithExplicitGenerics {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoCallWithExplicitGenerics,
}

/// `fn` or `object.method`, like the ESTree callee is an `Identifier` or a `MemberExpression` of identifiers.
fn callee_name(callee: P<Node>) -> Option<String> {
    let callee = skip_parens(callee);
    if ast::is_identifier(callee) {
        return Some(callee.text().to_string());
    }
    let property = member_identifier_property(callee)?;
    let object = skip_parens(callee.expression()?);
    ast::is_identifier(object).then(|| format!("{}.{property}", object.text()))
}

impl NoCallWithExplicitGenerics {
    fn find_match(&self, function_name: &str) -> Option<&Option<String>> {
        if let Some(message) = self.exact_matches.get(function_name) {
            return Some(message);
        }
        let (_, method) = function_name.split_once('.')?;
        self.wildcard_matches.get(method)
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.type_argument_list().is_none() {
            return;
        }
        let Some(function_name) = callee_name(node.as_call_expression().expression) else {
            return;
        };
        let Some(message) = self.rule.find_match(&function_name) else {
            return;
        };
        let message = match message {
            Some(message) => RuleMessage::new("noExplicitGenericsCustom", message.clone()),
            None => RuleMessage::new(
                "noExplicitGenerics",
                format!(
                    "Function '{function_name}' should be called with inferred generics (remove the explicit type \
                     parameters)"
                ),
            ),
        };
        ctx.report_node(node, message);
    }
}
