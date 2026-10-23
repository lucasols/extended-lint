use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};

use crate::utils::{is_in_optional_chain, skip_parens};

pub const NAME: &str = "use-state-setter-naming";

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    ignore_prefixes: Vec<String>,
}

pub struct UseStateSetterNaming {
    ignore_prefixes: Vec<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    Ok(Box::new(UseStateSetterNaming {
        ignore_prefixes: options.ignore_prefixes,
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VariableDeclaration)];

impl Rule for UseStateSetterNaming {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static UseStateSetterNaming,
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn strip_prefixes<'a>(name: &'a str, prefixes: &[String]) -> &'a str {
    prefixes
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix.as_str()))
        .unwrap_or(name)
}

/// `useState(...)` or `React.useState(...)`, not in an optional chain (an ESTree `ChainExpression`).
fn is_use_state_call(node: P<Node>) -> bool {
    if node.kind() != Kind::CallExpression || is_in_optional_chain(node) {
        return false;
    }
    let callee = node.as_call_expression().expression;
    match callee.kind() {
        Kind::Identifier => callee.text() == "useState",
        Kind::PropertyAccessExpression => {
            let access = callee.as_property_access_expression();
            access.expression.kind() == Kind::Identifier
                && access.expression.text() == "React"
                && access.name.kind() == Kind::Identifier
                && access.name.text() == "useState"
        }
        _ => false,
    }
}

/// The name of an array-pattern element that ESTree represents as a plain `Identifier`: no default value, no rest.
fn plain_identifier_element(element: P<Node>) -> Option<P<Node>> {
    if element.kind() != Kind::BindingElement {
        return None;
    }
    let binding = element.as_binding_element();
    if binding.dot_dot_dot_token().is_some() || binding.initializer().is_some() {
        return None;
    }
    element
        .name()
        .filter(|name| name.kind() == Kind::Identifier)
}

impl Visitor {
    fn report(ctx: &mut Ctx, setter: P<Node>, expected_name: &str, expected_setter_name: String) {
        let message = RuleMessage::new(
            "incorrectSetterName",
            format!(
                "useState setter should follow the pattern \"set{expected_name}\" with a \"set\" prefix but got \"{}\"",
                setter.text()
            ),
        );
        ctx.report_node_with_suggestions(setter, message, |ctx| {
            vec![RuleSuggestion {
                message: RuleMessage::new(
                    "useSuggestedName",
                    format!("Use \"{expected_setter_name}\" instead"),
                ),
                fixes: vec![ctx.fix_replace(setter, expected_setter_name.clone())],
            }]
        });
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(init) = node.initializer() else {
            return;
        };
        if !is_use_state_call(skip_parens(init)) {
            return;
        }
        let Some(pattern) = node.name() else {
            return;
        };
        if pattern.kind() != Kind::ArrayBindingPattern {
            return;
        }
        let [value_element, setter_element] = pattern.elements() else {
            return;
        };
        let (Some(value), Some(setter)) = (
            plain_identifier_element(*value_element),
            plain_identifier_element(*setter_element),
        ) else {
            return;
        };

        let value_name = value.text();
        let prefixes = &self.rule.ignore_prefixes;
        let stripped_value_name = strip_prefixes(value_name, prefixes);
        let stripped_setter_name = strip_prefixes(setter.text(), prefixes);

        if stripped_value_name.is_empty() {
            if !stripped_setter_name.starts_with("set") {
                let expected_name = capitalize(value_name);
                Self::report(ctx, setter, &expected_name, format!("set{expected_name}"));
            }
            return;
        }

        let expected_name = capitalize(stripped_value_name);
        let expected_setter_name = format!("set{expected_name}");
        if stripped_setter_name != expected_setter_name {
            Self::report(ctx, setter, &expected_name, expected_setter_name);
        }
    }
}
