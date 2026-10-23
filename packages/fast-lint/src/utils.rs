use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::Ctx;

/// Source text of `node` without its leading trivia, like ESLint's `sourceCode.getText(node)`.
pub fn node_text(ctx: &Ctx, node: P<Node>) -> &'static str {
    let (pos, end) = ctx.trim(node);
    let start = usize::try_from(pos.min(end)).unwrap_or(0);
    let end = usize::try_from(end).unwrap_or(0);
    ctx.text().get(start..end).unwrap_or_default()
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum LeadingPosition {
    Statement(P<Node>),
    ArrowBody,
}

/// Whether `node`'s text starts an expression statement or an arrow function's expression body.
fn leading_position(ctx: &Ctx, node: P<Node>) -> Option<LeadingPosition> {
    let start = ctx.trim(node).0;
    let mut current = node;
    loop {
        let parent = current.parent()?;
        if ast::is_arrow_function(parent) {
            return (parent.body() == Some(current)).then_some(LeadingPosition::ArrowBody);
        }
        if ctx.trim(parent).0 != start {
            return None;
        }
        if ast::is_expression_statement(parent) {
            return Some(LeadingPosition::Statement(parent));
        }
        if ast::is_statement(parent) || ast::is_function_like(Some(parent)) {
            return None;
        }
        current = parent;
    }
}

fn starts_with_keyword(text: &str, keyword: &str) -> bool {
    text.strip_prefix(keyword).is_some_and(|rest| {
        rest.chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '$')
    })
}

/// Text that replaces `node`, adjusted so it keeps parsing as an expression where `node` started a statement or an
/// arrow body: an object literal, function or class is parenthesized, and a statement starting with `(`, `[`, `` ` ``,
/// `+`, `-` or `/` gets a leading `;` unless the previous token ends a statement, so it doesn't continue the previous
/// line.
pub fn replacement_at(ctx: &Ctx, node: P<Node>, replacement: String) -> String {
    let Some(position) = leading_position(ctx, node) else {
        return replacement;
    };
    let starts_like_statement = match position {
        LeadingPosition::ArrowBody => replacement.starts_with('{'),
        LeadingPosition::Statement(_) => {
            replacement.starts_with('{')
                || ["function", "class", "async"]
                    .iter()
                    .any(|keyword| starts_with_keyword(&replacement, keyword))
        }
    };
    let replacement = if starts_like_statement {
        format!("({replacement})")
    } else {
        replacement
    };
    let LeadingPosition::Statement(statement) = position else {
        return replacement;
    };
    if !replacement.starts_with(['(', '[', '`', '+', '-', '/']) {
        return replacement;
    }
    let statement_start = u32::try_from(ctx.trim(statement).0).unwrap_or(0);
    let previous = tsrs_astnav::find_preceding_token(ctx.file, statement_start);
    let ends_statement = previous
        .is_none_or(|token| matches!(token.kind(), Kind::SemicolonToken | Kind::OpenBraceToken));
    if ends_statement {
        replacement
    } else {
        format!(";{replacement}")
    }
}
