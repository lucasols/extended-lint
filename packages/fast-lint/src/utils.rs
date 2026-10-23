use tsrs_ast::{self as ast, Node};
use tsrs_core::P;
use tsrslint::rule::Ctx;

/// Source text of `node` without its leading trivia, like ESLint's `sourceCode.getText(node)`.
pub fn node_text(ctx: &Ctx, node: P<Node>) -> &'static str {
    let (pos, end) = ctx.trim(node);
    &ctx.text()[pos as usize..end as usize]
}

/// ESTree has no parenthesized expression nodes, so rules ported from ESLint look through them.
pub fn skip_parens(node: P<Node>) -> P<Node> {
    ast::skip_parentheses(node)
}

/// The parent of `node`, skipping parenthesized expressions (the ESTree parent).
pub fn estree_parent(node: P<Node>) -> Option<P<Node>> {
    let mut parent = node.parent()?;
    while ast::is_parenthesized_expression(parent) {
        parent = parent.parent()?;
    }
    Some(parent)
}

/// `a?.b()`, `a?.b` and their continuations are wrapped in an ESTree `ChainExpression`, so ESLint rules that match
/// on `CallExpression`/`MemberExpression` children never see them directly.
pub fn is_in_optional_chain(node: P<Node>) -> bool {
    ast::is_optional_chain(node)
}

/// Whether ESTree wraps `node` in a `ChainExpression`: it is the outermost link of an optional chain (`a?.b` in
/// `a?.b > 1`), so ESLint rules matching a `MemberExpression`/`CallExpression` there see a `ChainExpression` instead.
pub fn is_estree_chain_expression(node: P<Node>) -> bool {
    if !is_in_optional_chain(node) {
        return false;
    }
    node.parent()
        .is_none_or(|parent| !is_in_optional_chain(parent) || parent.expression() != Some(node))
}
