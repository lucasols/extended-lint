use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};

use crate::utils::{estree_parent, is_estree_chain_expression, skip_parens};

pub const NAME: &str = "prefer-react-hook-alternative";

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct DisallowedFunction {
    name: String,
    #[serde(default)]
    allow_using_with_args: bool,
    hook_alternative: Option<String>,
    message: Option<String>,
    #[serde(default)]
    allow_use_inside: Vec<String>,
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    disallowed_functions: Vec<DisallowedFunction>,
}

pub struct PreferReactHookAlternative {
    disallowed: FxHashMap<String, DisallowedFunction>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut disallowed = FxHashMap::default();
    for function in options.disallowed_functions {
        disallowed.insert(function.name.clone(), function);
    }
    Ok(Box::new(PreferReactHookAlternative { disallowed }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for PreferReactHookAlternative {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static PreferReactHookAlternative,
}

fn is_hook_name(name: &str) -> bool {
    name.strip_prefix("use")
        .and_then(|rest| rest.chars().next())
        .is_some_and(|c| c.is_ascii_uppercase())
}

fn is_component_name(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

fn identifier_name(node: P<Node>) -> Option<&'static str> {
    ast::is_identifier(node).then(|| node.as_identifier().text())
}

/// The ESTree `MemberExpression.property` of `callee` when it is an Identifier (`a.b` and `a[b]`).
fn member_property_identifier(callee: P<Node>) -> Option<P<Node>> {
    let property = match callee.kind() {
        Kind::PropertyAccessExpression => callee.as_property_access_expression().name(),
        Kind::ElementAccessExpression => {
            skip_parens(callee.as_element_access_expression().argument_expression())
        }
        _ => return None,
    };
    ast::is_identifier(property).then_some(property)
}

/// The callee as ESTree sees it: a parenthesized optional chain becomes a `ChainExpression`.
fn estree_callee(call: P<Node>) -> Option<P<Node>> {
    let callee = skip_parens(call.as_call_expression().expression());
    (!is_estree_chain_expression(callee)).then_some(callee)
}

/// The callee Identifier, or the member property Identifier, that names the called function.
fn call_name_node(call: P<Node>) -> Option<P<Node>> {
    let callee = estree_callee(call)?;
    if ast::is_identifier(callee) {
        return Some(callee);
    }
    member_property_identifier(callee)
}

fn call_name(call: P<Node>) -> Option<&'static str> {
    call_name_node(call).and_then(identifier_name)
}

fn is_hook_callee(call: P<Node>) -> bool {
    call_name(call).is_some_and(is_hook_name)
}

fn is_react_function(call: P<Node>, function_name: &str) -> bool {
    let Some(callee) = estree_callee(call) else {
        return false;
    };
    if let Some(name) = identifier_name(callee) {
        return name == function_name;
    }
    let object = match callee.kind() {
        Kind::PropertyAccessExpression => callee.as_property_access_expression().expression(),
        Kind::ElementAccessExpression => callee.as_element_access_expression().expression(),
        _ => return false,
    };
    identifier_name(skip_parens(object)) == Some("React")
        && member_property_identifier(callee).and_then(identifier_name) == Some(function_name)
}

/// ESTree `FunctionDeclaration`, `FunctionExpression` and `ArrowFunctionExpression`: methods, accessors and
/// constructors are `FunctionExpression`s there.
fn is_estree_function(node: P<Node>) -> bool {
    matches!(
        node.kind(),
        Kind::FunctionDeclaration
            | Kind::FunctionExpression
            | Kind::ArrowFunction
            | Kind::MethodDeclaration
            | Kind::GetAccessor
            | Kind::SetAccessor
            | Kind::Constructor
    )
}

fn is_forward_ref_or_memo_callback(function: P<Node>) -> bool {
    estree_parent(function).is_some_and(|parent| {
        parent.kind() == Kind::CallExpression
            && (is_react_function(parent, "forwardRef") || is_react_function(parent, "memo"))
    })
}

fn non_computed_name(node: P<Node>) -> Option<P<Node>> {
    node.name()
        .filter(|name| name.kind() != Kind::ComputedPropertyName)
}

/// The name of an ESTree function: its own id, or the binding/assignment/property it is the value of.
fn function_name(function: P<Node>) -> Option<P<Node>> {
    match function.kind() {
        Kind::FunctionDeclaration => return function.name(),
        Kind::FunctionExpression if function.name().is_some() => return function.name(),
        Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor => {
            let parent = function.parent()?;
            return if parent.kind() == Kind::ObjectLiteralExpression {
                non_computed_name(function)
            } else {
                None
            };
        }
        Kind::FunctionExpression | Kind::ArrowFunction => {}
        _ => return None,
    }
    let parent = estree_parent(function)?;
    match parent.kind() {
        Kind::VariableDeclaration => parent.name(),
        Kind::BinaryExpression => {
            let binary = parent.as_binary_expression();
            (binary.operator_token().kind() == Kind::EqualsToken
                && skip_parens(binary.right()) == function)
                .then(|| binary.left())
        }
        Kind::PropertyAssignment => non_computed_name(parent),
        Kind::ShorthandPropertyAssignment | Kind::Parameter | Kind::BindingElement => parent.name(),
        _ => None,
    }
}

fn function_identifier_name(function: P<Node>) -> Option<&'static str> {
    function_name(function).and_then(identifier_name)
}

fn is_component_or_hook_name(name: &str) -> bool {
    is_component_name(name) || is_hook_name(name)
}

fn ancestors(node: P<Node>) -> impl Iterator<Item = P<Node>> {
    std::iter::successors(node.parent(), |current| current.parent())
}

fn is_inside_component_or_hook(node: P<Node>) -> bool {
    ancestors(node)
        .filter(|&a| is_estree_function(a))
        .any(|function| match function_name(function) {
            Some(name) if ast::is_identifier(name) => {
                is_component_or_hook_name(name.as_identifier().text())
            }
            _ => is_forward_ref_or_memo_callback(function),
        })
}

fn is_directly_inside_component_or_hook(node: P<Node>) -> bool {
    let mut found_nested = false;
    for function in ancestors(node).filter(|&a| is_estree_function(a)) {
        if let Some(name) = function_identifier_name(function) {
            if is_component_or_hook_name(name) {
                return !found_nested;
            }
            found_nested = true;
        } else if is_forward_ref_or_memo_callback(function) {
            return !found_nested;
        } else {
            let parent = estree_parent(function);
            let is_transparent = parent.is_some_and(|parent| match parent.kind() {
                Kind::CallExpression => is_hook_callee(parent),
                Kind::JsxExpression => true,
                _ => false,
            });
            if !is_transparent {
                found_nested = true;
            }
        }
    }
    false
}

fn is_inside_allowed_function(node: P<Node>, allow_use_inside: &[String]) -> bool {
    let allowed = |name: &str| allow_use_inside.iter().any(|allowed| allowed == name);
    ancestors(node).any(|ancestor| {
        if is_estree_function(ancestor) {
            function_identifier_name(ancestor).is_some_and(allowed)
        } else if ancestor.kind() == Kind::CallExpression {
            call_name(ancestor).is_some_and(allowed)
        } else {
            false
        }
    })
}

fn should_report(node: P<Node>, allow_use_inside: &[String]) -> bool {
    if !is_inside_component_or_hook(node) {
        return false;
    }
    if allow_use_inside.is_empty() {
        return is_directly_inside_component_or_hook(node);
    }
    !is_inside_allowed_function(node, allow_use_inside)
}

fn has_meaningful_arguments(call: P<Node>) -> bool {
    let arguments = call.arguments();
    !arguments.is_empty()
        && !arguments
            .iter()
            .all(|&arg| identifier_name(skip_parens(arg)) == Some("undefined"))
}

fn report_message(id: &'static str, message: String) -> RuleMessage {
    RuleMessage::new(
        id,
        format!("This function should not be used in react{message}."),
    )
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(name_node) = call_name_node(node) else {
            return;
        };
        let Some(function) = self.rule.disallowed.get(name_node.as_identifier().text()) else {
            return;
        };
        if function.allow_using_with_args && has_meaningful_arguments(node) {
            return;
        }
        if !should_report(node, &function.allow_use_inside) {
            return;
        }
        let hook_alternative = function
            .hook_alternative
            .as_deref()
            .filter(|h| !h.is_empty());
        let detail = match function.message.as_deref().filter(|m| !m.is_empty()) {
            Some(message) => format!(" {message}"),
            None => format!(
                " use {} instead",
                function.hook_alternative.as_deref().unwrap_or("undefined")
            ),
        };
        let message = report_message("preferHookAlternative", detail);
        let Some(hook_alternative) = hook_alternative else {
            ctx.report_node(node, message);
            return;
        };
        ctx.report_node_with_suggestions(node, message, |ctx| {
            vec![RuleSuggestion {
                message: report_message(
                    "preferHookAlternative",
                    format!("Replace with {hook_alternative}"),
                ),
                fixes: vec![ctx.fix_replace(name_node, hook_alternative.to_string())],
            }]
        });
    }
}
