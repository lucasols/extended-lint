use rustc_hash::FxHashSet;
use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::skip_parens;

pub const NAME: &str = "require-lazy-execution";

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    functions: Vec<String>,
    /// ESQuery selectors of the ESLint rule. Not supported: there is no selector engine here.
    #[serde(default)]
    selectors: Vec<String>,
}

pub struct RequireLazyExecution {
    functions: FxHashSet<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    if !options.selectors.is_empty() {
        return Err(
            "the `selectors` option is not supported, list the function names in `functions` instead".to_string(),
        );
    }
    Ok(Box::new(RequireLazyExecution {
        functions: options.functions.into_iter().collect(),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::TaggedTemplateExpression),
];

impl Rule for RequireLazyExecution {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static RequireLazyExecution,
}

/// Whether `node` runs inside a function body (or parameter list) rather than at module level: an ancestor is a
/// function, arrow function, method, accessor or constructor. A method's computed key and decorators are evaluated
/// outside it, like ESTree's `MethodDefinition` key and decorators outside its `FunctionExpression` value.
fn is_inside_function_context(node: P<Node>) -> bool {
    let mut child = node;
    let mut current = node.parent();
    while let Some(ancestor) = current {
        match ancestor.kind() {
            Kind::FunctionDeclaration | Kind::FunctionExpression | Kind::ArrowFunction => {
                return true;
            }
            Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor | Kind::Constructor
                if ancestor.body() == Some(child) || ancestor.parameters().contains(&child) =>
            {
                return true;
            }
            _ => {}
        }
        child = ancestor;
        current = ancestor.parent();
    }
    false
}

/// The called name: an identifier callee or tag, or the property of a member callee or tag (`a.b` and `a[b]`, like
/// ESTree's `MemberExpression` property `Identifier`).
fn called_name(callee: P<Node>) -> Option<&'static str> {
    let callee = skip_parens(callee);
    match callee.kind() {
        Kind::Identifier => Some(callee.text()),
        Kind::PropertyAccessExpression => {
            let name = callee.as_property_access_expression().name;
            (name.kind() == Kind::Identifier).then(|| name.text())
        }
        Kind::ElementAccessExpression => {
            let argument = callee.as_element_access_expression().argument_expression;
            (argument.kind() == Kind::Identifier).then(|| argument.text())
        }
        _ => None,
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let callee = match node.kind() {
            Kind::CallExpression => node.as_call_expression().expression,
            _ => node.as_tagged_template_expression().tag,
        };
        let Some(function_name) = called_name(callee) else {
            return;
        };
        if !self.rule.functions.contains(function_name) || is_inside_function_context(node) {
            return;
        }
        ctx.report_node(
            node,
            RuleMessage::new(
                "moduleLevel",
                format!(
                    "Function \"{function_name}\" must only be called inside functions or getters, not at module level"
                ),
            ),
        );
    }
}
