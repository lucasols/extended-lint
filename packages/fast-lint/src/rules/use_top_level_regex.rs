use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{estree_parent, skip_parens};

pub const NAME: &str = "use-top-level-regex";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct UseTopLevelRegex;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(UseTopLevelRegex))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::RegularExpressionLiteral)];

impl Rule for UseTopLevelRegex {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

/// The `g` and `y` flags make a regex stateful (`lastIndex`), so hoisting it would change behavior.
fn has_stateful_flags(regex: P<Node>) -> bool {
    let text = regex.text();
    text.rfind('/')
        .and_then(|index| text.get(index + 1..))
        .is_some_and(|flags| flags.contains(['g', 'y']))
}

fn is_in_regex_constructor(regex: P<Node>) -> bool {
    estree_parent(regex).is_some_and(|parent| {
        ast::is_new_expression(parent)
            && parent.expression().map(skip_parens).is_some_and(|callee| {
                ast::is_identifier(callee) && callee.as_identifier().text() == "RegExp"
            })
    })
}

fn is_method_like(node: P<Node>) -> bool {
    matches!(
        node.kind(),
        Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor | Kind::Constructor
    )
}

/// Whether `ancestor`, reached from its child `child`, is one of the ESTree `FunctionDeclaration`,
/// `FunctionExpression`, `ArrowFunctionExpression` or `MethodDefinition` nodes the ESLint rule's selector matches.
/// A class method is a `MethodDefinition` that also spans its key and decorators, while an object literal method is a
/// `Property` whose `FunctionExpression` value starts at the parameters, so its key is outside the function.
fn is_estree_function_ancestor(ancestor: P<Node>, child: P<Node>) -> bool {
    match ancestor.kind() {
        Kind::ArrowFunction | Kind::FunctionExpression => true,
        Kind::FunctionDeclaration => ancestor.body().is_some(),
        _ if is_method_like(ancestor) => {
            let in_object_literal = ancestor
                .parent()
                .is_some_and(ast::is_object_literal_expression);
            if in_object_literal {
                ancestor.name() != Some(child)
            } else {
                !ast::has_syntactic_modifier(ancestor, ModifierFlags::Abstract)
            }
        }
        _ => false,
    }
}

fn is_inside_function(regex: P<Node>) -> bool {
    let mut child = regex;
    while let Some(ancestor) = child.parent() {
        if is_estree_function_ancestor(ancestor, child) {
            return true;
        }
        child = ancestor;
    }
    false
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if has_stateful_flags(node) || is_in_regex_constructor(node) || !is_inside_function(node) {
            return;
        }
        ctx.report_node(
            node,
            RuleMessage::new(
                "regexShouldBeTopLevel",
                "Move this regex literal to the top level to avoid creating new RegExp objects on each function call.",
            ),
        );
    }
}
