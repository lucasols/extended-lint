use tsrs_ast::{self as ast, Kind, Node, SourceFile, Symbol};
use tsrs_compiler::Program;
use tsrs_core::P;
use tsrslint::rule::Ctx;

/// Whether `declaration` is in a default library file (`lib.*.d.ts`).
pub fn is_default_lib_declaration(program: &Program, declaration: P<Node>) -> bool {
    ast::get_source_file_of_node(declaration)
        .is_some_and(|file| program.is_source_file_default_library(file.path()))
}

/// Whether `symbol` is declared in a default library file: the global `Array`, `Function`, ..., not a local one with
/// the same name.
pub fn is_default_lib_symbol(program: &Program, symbol: P<Symbol>) -> bool {
    symbol
        .declarations()
        .iter()
        .any(|&declaration| is_default_lib_declaration(program, declaration))
}

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
    ExportDefault,
}

/// Whether `node`'s text starts an expression statement, an arrow function's expression body or an `export default`
/// expression.
fn leading_position(ctx: &Ctx, node: P<Node>) -> Option<LeadingPosition> {
    let start = ctx.trim(node).0;
    let mut current = node;
    loop {
        let parent = current.parent()?;
        if ast::is_arrow_function(parent) {
            return (parent.body() == Some(current)).then_some(LeadingPosition::ArrowBody);
        }
        if ast::is_export_assignment(parent) {
            return (parent.expression() == Some(current))
                .then_some(LeadingPosition::ExportDefault);
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

fn starts_with_declaration_keyword(text: &str) -> bool {
    ["function", "class", "async"]
        .iter()
        .any(|keyword| starts_with_keyword(text, keyword))
}

/// Whether `statement` is directly in a statement list, where the previous line can continue into it. The body of
/// a braceless `if`/`else`/loop/label isn't: a `;` there would become the whole body.
fn is_in_statement_list(statement: P<Node>) -> bool {
    statement.parent().is_some_and(|parent| {
        ast::is_source_file(parent)
            || ast::is_block(parent)
            || ast::is_module_block(parent)
            || ast::is_case_clause(parent)
            || ast::is_default_clause(parent)
    })
}

/// Text that replaces `node`, adjusted so it keeps parsing as the same expression where `node` started a statement,
/// an arrow body or an `export default` expression:
/// - an object literal, function or class is parenthesized, and so is a string literal that would become the whole
///   statement (a directive like `'use strict'`);
/// - a statement of a statement list starting with `(`, `[`, `` ` ``, `+`, `-`, `/` or `<` gets a leading `;` unless
///   the previous token ends a statement, so it doesn't continue the previous line.
pub fn replacement_at(ctx: &Ctx, node: P<Node>, replacement: String) -> String {
    let Some(position) = leading_position(ctx, node) else {
        return replacement;
    };
    let needs_parens = match position {
        LeadingPosition::ArrowBody => replacement.starts_with('{'),
        LeadingPosition::ExportDefault => starts_with_declaration_keyword(&replacement),
        LeadingPosition::Statement(statement) => {
            replacement.starts_with('{')
                || starts_with_declaration_keyword(&replacement)
                || (statement.expression() == Some(node) && replacement.starts_with(['\'', '"']))
        }
    };
    let replacement = if needs_parens {
        format!("({replacement})")
    } else {
        replacement
    };
    let LeadingPosition::Statement(statement) = position else {
        return replacement;
    };
    if !replacement.starts_with(['(', '[', '`', '+', '-', '/', '<'])
        || !is_in_statement_list(statement)
    {
        return replacement;
    }
    let statement_start = u32::try_from(ctx.trim(statement).0).unwrap_or(0);
    let previous = tsrs_astnav::find_preceding_token_ex(ctx.file, statement_start, None, true);
    let ends_statement = previous
        .is_none_or(|token| matches!(token.kind(), Kind::SemicolonToken | Kind::OpenBraceToken));
    if ends_statement {
        replacement
    } else {
        format!(";{replacement}")
    }
}

/// The comment containing byte offset `pos`, as its byte range and whether it is a block comment. `None` when `pos`
/// is not inside a comment: in a token (a string, template, regex or JSX text) or in whitespace.
pub fn comment_at(file: P<SourceFile>, pos: u32) -> Option<(u32, u32, bool)> {
    let token = tsrs_astnav::get_token_at_position(file, pos);
    if matches!(token.kind(), Kind::JsxText | Kind::JsxTextAllWhiteSpaces) {
        return None;
    }
    let full_start = token.pos().source_offset()?;
    let text = file.text();
    let token_start = tsrs_scanner::skip_trivia(text, full_start);
    if pos < full_start || pos >= token_start {
        return None;
    }
    let trivia = text.get(full_start as usize..token_start as usize)?;
    let mut offset = 0;
    if full_start == 0 && trivia.starts_with("#!") {
        offset = trivia.find(is_line_terminator).unwrap_or(trivia.len());
    }
    while let Some(rest) = trivia.get(offset..).filter(|rest| !rest.is_empty()) {
        let (len, is_block) = if rest.starts_with("//") {
            (rest.find(is_line_terminator).unwrap_or(rest.len()), false)
        } else if rest.starts_with("/*") {
            let len = rest
                .get(2..)
                .and_then(|body| body.find("*/"))
                .map_or(rest.len(), |end| end + 4);
            (len, true)
        } else {
            offset += rest.chars().next().map_or(1, char::len_utf8);
            continue;
        };
        let start = full_start + u32::try_from(offset).ok()?;
        let end = start + u32::try_from(len).ok()?;
        if pos < end {
            return (start <= pos).then_some((start, end, is_block));
        }
        offset += len;
    }
    None
}

fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}
