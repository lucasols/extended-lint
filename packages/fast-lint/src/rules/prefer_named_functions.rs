use regex::Regex;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};

use crate::utils::{node_text, skip_parens};

pub const NAME: &str = "prefer-named-functions";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    ignore_regex: Option<String>,
    disallow_arrow_fn_with_implicit_returns: bool,
}

pub struct PreferNamedFunctions {
    ignore_regex: Option<Regex>,
    disallow_arrow_fn_with_implicit_returns: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let ignore_regex = options
        .ignore_regex
        .filter(|pattern| !pattern.is_empty())
        .map(|pattern| Regex::new(&pattern).map_err(|e| format!("invalid ignoreRegex: {e}")))
        .transpose()?;
    Ok(Box::new(PreferNamedFunctions {
        ignore_regex,
        disallow_arrow_fn_with_implicit_returns: options.disallow_arrow_fn_with_implicit_returns,
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VariableDeclaration)];

impl Rule for PreferNamedFunctions {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static PreferNamedFunctions,
}

/// The range of the ESTree `VariableDeclaration` holding `declaration`: from its `const`/`let`/`var` keyword through
/// the statement's `;`, without `export`/`declare` modifiers.
fn estree_declaration_range(ctx: &Ctx, declaration: P<Node>) -> Option<(i32, i32)> {
    let list = declaration.parent()?;
    let (start, list_end) = ctx.trim(list);
    let end = list
        .parent()
        .filter(|&statement| ast::is_variable_statement(statement))
        .map_or(list_end, |statement| ctx.trim(statement).1);
    Some((start, end))
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let declaration = node.as_variable_declaration();
        let Some(init) = declaration.initializer().map(skip_parens) else {
            return;
        };
        let name = declaration.name();
        if !ast::is_arrow_function(init)
            || !ast::is_identifier(name)
            || declaration.type_().is_some()
        {
            return;
        }
        let function_name = name.text();
        if self
            .rule
            .ignore_regex
            .as_ref()
            .is_some_and(|regex| regex.is_match(function_name))
        {
            return;
        }
        let Some(body) = init.body() else {
            return;
        };
        if !self.rule.disallow_arrow_fn_with_implicit_returns && !ast::is_block(body) {
            return;
        }
        let message = if self.rule.ignore_regex.is_some() {
            RuleMessage::new(
                "withIgnoreRegex",
                format!(
                    "Function {function_name} should be defined as a named function \"function {function_name} () \
                     {{}}\", if not possible to use `function` you can also use a name that matches the rule ignore \
                     regex"
                ),
            )
        } else {
            RuleMessage::new(
                "default",
                format!(
                    "Function {function_name} should be defined as a named function \"function {function_name} () \
                     {{}}\" instead of an arrow function"
                ),
            )
        };
        let Some((start, end)) = estree_declaration_range(ctx, node) else {
            return;
        };
        ctx.report_node_with_suggestions(name, message, |ctx| {
            let is_async = ast::has_syntactic_modifier(init, ModifierFlags::Async);
            let params = init
                .parameters()
                .iter()
                .map(|&param| node_text(ctx, param))
                .collect::<Vec<_>>()
                .join(", ");
            let replacement = format!(
                "{}function {function_name}({params}) {}",
                if is_async { "async " } else { "" },
                node_text(ctx, body),
            );
            vec![RuleSuggestion {
                message: RuleMessage::new("suggestion", "Convert to named function"),
                fixes: vec![ctx.fix_replace_range(start, end, replacement)],
            }]
        });
    }
}
