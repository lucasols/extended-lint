use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{
    comment_at, estree_parent, is_estree_chain_expression, node_text, replacement_at, skip_parens,
};

pub const NAME: &str = "no-unnecessary-iife";

pub struct NoUnnecessaryIife;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<NoOptions>(options)?;
    Ok(Box::new(NoUnnecessaryIife))
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct NoOptions {}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for NoUnnecessaryIife {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn message() -> RuleMessage {
    RuleMessage::new(
        "unnecessaryIife",
        "Unnecessary IIFE. Can be simplified to the returned expression.",
    )
}

/// Whether the source text of `node` contains a comment, like ESLint's `sourceCode.getCommentsInside(node)`.
fn has_comments_inside(ctx: &Ctx, node: P<Node>) -> bool {
    let (start, end) = ctx.trim(node);
    let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
        return false;
    };
    let segment = ctx.text().get(start..end).unwrap_or_default();
    ["//", "/*"].iter().any(|opener| {
        segment.match_indices(opener).any(|(i, _)| {
            u32::try_from(start + i).is_ok_and(|pos| {
                comment_at(ctx.file, pos).is_some_and(|(comment_start, _, _)| comment_start == pos)
            })
        })
    })
}

/// A function whose ESTree body is the block `block`: ESTree represents methods, accessors and constructors as
/// `FunctionExpression`s.
fn is_function_with_body(node: P<Node>, block: P<Node>) -> bool {
    matches!(
        node.kind(),
        Kind::FunctionDeclaration
            | Kind::FunctionExpression
            | Kind::ArrowFunction
            | Kind::MethodDeclaration
            | Kind::GetAccessor
            | Kind::SetAccessor
            | Kind::Constructor
    ) && node.body() == Some(block)
}

/// The body of the function `call` is the only statement of, as `return call()`.
fn returned_from_function_body(call: P<Node>) -> Option<P<Node>> {
    if is_estree_chain_expression(call) {
        return None;
    }
    let return_statement = estree_parent(call)?;
    if return_statement.kind() != Kind::ReturnStatement {
        return None;
    }
    let block = return_statement.parent()?;
    if block.kind() != Kind::Block || block.statements().len() != 1 {
        return None;
    }
    let function = block.parent()?;
    is_function_with_body(function, block).then_some(block)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if !node.arguments().is_empty() {
            return;
        }
        let callee = skip_parens(node.as_call_expression().expression);
        if !matches!(
            callee.kind(),
            Kind::FunctionExpression | Kind::ArrowFunction
        ) || ast::has_syntactic_modifier(callee, ModifierFlags::Async)
            || !callee.parameters().is_empty()
        {
            return;
        }
        let Some(body) = callee.body() else {
            return;
        };

        let returned = if body.kind() == Kind::Block {
            match body.statements() {
                [statement] if statement.kind() == Kind::ReturnStatement => statement.expression(),
                [_] => return,
                _ => {
                    let Some(function_body) = returned_from_function_body(node) else {
                        return;
                    };
                    if has_comments_inside(ctx, callee) {
                        return;
                    }
                    let iife_body = node_text(ctx, body).to_string();
                    ctx.report_node_with_fixes(node, message(), |ctx| {
                        vec![ctx.fix_replace(function_body, iife_body)]
                    });
                    return;
                }
            }
        } else {
            Some(body)
        };

        if has_comments_inside(ctx, callee) {
            return;
        }
        let replacement = match returned {
            Some(expression) => node_text(ctx, skip_parens(expression)).to_string(),
            None => "undefined".to_string(),
        };
        let replacement = replacement_at(ctx, node, replacement);
        ctx.report_node_with_fixes(node, message(), |ctx| {
            vec![ctx.fix_replace(node, replacement)]
        });
    }
}
