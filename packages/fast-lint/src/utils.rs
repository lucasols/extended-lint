use tsrs_ast::{self as ast, Kind, Node, SourceFile, Symbol};
use tsrs_checker::{LiteralValue, Type, TypeFlags};
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

/// Whether `node` has comments outside its descendant `inner`, which replacing `node` with `inner`'s text would drop.
pub fn has_comments_outside(ctx: &Ctx, node: P<Node>, inner: P<Node>) -> bool {
    let (node_start, node_end) = ctx.trim(node);
    let (inner_start, inner_end) = ctx.trim(inner);
    let text = ctx.text();
    [(node_start, inner_start), (inner_end, node_end)]
        .iter()
        .any(|&(start, end)| {
            let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
                return false;
            };
            let segment = text.get(start..end).unwrap_or_default();
            ["//", "/*"].iter().any(|opener| {
                segment.match_indices(opener).any(|(i, _)| {
                    u32::try_from(start + i).is_ok_and(|pos| comment_at(ctx.file, pos).is_some())
                })
            })
        })
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

/// Every comment of `file` in source order, as its byte range and whether it is a block comment, like ESLint's
/// `sourceCode.getAllComments()`: `//` and `/*` inside strings, templates, regexes and JSX text are not comments.
pub fn comments(file: P<SourceFile>) -> Vec<(u32, u32, bool)> {
    let bytes = file.text().as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while let Some(offset) = bytes
        .get(i..)
        .and_then(|rest| rest.iter().position(|&b| b == b'/'))
    {
        let start = i + offset;
        i = start + 1;
        if !matches!(bytes.get(start + 1), Some(b'/' | b'*')) {
            continue;
        }
        let Ok(pos) = u32::try_from(start) else {
            break;
        };
        if let Some(comment) =
            comment_at(file, pos).filter(|&(comment_start, _, _)| comment_start == pos)
        {
            found.push(comment);
            i = usize::try_from(comment.1).unwrap_or(bytes.len());
        }
    }
    found
}

/// The name of an ESTree `MemberExpression`'s `Identifier` property: `b` in `a.b`, and also in the computed `a[b]`,
/// where ESTree keeps the identifier as the property too.
pub fn member_identifier_property(node: P<Node>) -> Option<&'static str> {
    match node.kind() {
        Kind::PropertyAccessExpression => {
            let name = node.as_property_access_expression().name;
            ast::is_identifier(name).then(|| name.text())
        }
        Kind::ElementAccessExpression => {
            let argument = skip_parens(node.as_element_access_expression().argument_expression);
            ast::is_identifier(argument).then(|| argument.text())
        }
        _ => None,
    }
}

/// The range of the ESTree `TSTypeAnnotation` holding `type_node` (a parameter's or a function's return type): from
/// the `:` before it to its end.
pub fn type_annotation_range(ctx: &Ctx, type_node: P<Node>) -> (i32, i32) {
    let (start, end) = ctx.trim(type_node);
    let colon = type_node
        .pos()
        .source_offset()
        .and_then(|full_start| i32::try_from(full_start).ok())
        .map(|full_start| full_start - 1)
        .filter(|&colon| {
            usize::try_from(colon)
                .ok()
                .and_then(|colon| ctx.text().get(colon..=colon))
                == Some(":")
        });
    (colon.unwrap_or(start), end)
}

/// A type node without the parentheses around it; ESTree has no parenthesized type nodes.
pub fn skip_type_parens(node: P<Node>) -> P<Node> {
    let mut current = node;
    while ast::is_parenthesized_type_node(current) {
        current = current.as_parenthesized_type_node().type_;
    }
    current
}

/// The name of a property key that ESTree represents as an `Identifier`: `a` in `a: 1`, `{ a }`, `a() {}`,
/// `a?: string` or `[a]: 1` (a computed identifier key is still an `Identifier`). `None` for string, numeric,
/// private and other computed keys.
pub fn estree_identifier_key(name: P<Node>) -> Option<&'static str> {
    match name.kind() {
        Kind::Identifier => Some(name.text()),
        Kind::ComputedPropertyName => {
            let expression = skip_parens(name.expression()?);
            (expression.kind() == Kind::Identifier).then(|| expression.text())
        }
        _ => None,
    }
}

/// The identifier key name of an ESTree `Property`: an object literal property, method or accessor, or a non-rest
/// element of an object binding pattern. `None` for spread and rest elements and non-identifier keys.
pub fn estree_property_key(node: P<Node>) -> Option<&'static str> {
    match node.kind() {
        Kind::PropertyAssignment
        | Kind::ShorthandPropertyAssignment
        | Kind::MethodDeclaration
        | Kind::GetAccessor
        | Kind::SetAccessor => estree_identifier_key(node.name()?),
        Kind::BindingElement if node.as_binding_element().dot_dot_dot_token().is_none() => {
            estree_identifier_key(node.property_name().or(node.name())?)
        }
        _ => None,
    }
}

/// The function an object literal property holds as its ESTree `Property.value`: the arrow function or function
/// expression of `a: () => {}`, or the method or accessor itself for `a() {}`.
pub fn estree_property_function(property: P<Node>) -> Option<P<Node>> {
    match property.kind() {
        Kind::PropertyAssignment => {
            let value = skip_parens(property.initializer()?);
            matches!(value.kind(), Kind::ArrowFunction | Kind::FunctionExpression).then_some(value)
        }
        Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor => Some(property),
        _ => None,
    }
}

/// The object literal a function returns: its arrow expression body, or the argument of the only `return` among
/// the top-level statements of its block body.
pub fn returned_object_literal(function: P<Node>) -> Option<P<Node>> {
    let body = function.body()?;
    let returned = if body.kind() == Kind::Block {
        let mut returns = body
            .statements()
            .iter()
            .filter(|statement| statement.kind() == Kind::ReturnStatement);
        let only = returns.next()?;
        if returns.next().is_some() {
            return None;
        }
        skip_parens(only.expression()?)
    } else {
        skip_parens(body)
    };
    (returned.kind() == Kind::ObjectLiteralExpression).then_some(returned)
}

/// Whether the `var` declaration list `list` is inside a block scope of its function (a nested block, a `switch` or
/// a `with`), where ESLint's `sourceCode.getScope(list)` is that block scope, which doesn't hold the hoisted variable.
/// Always false for `let`/`const`/`using` lists.
pub fn is_var_in_nested_block(list: P<Node>) -> bool {
    if list.flags().intersects(ast::NodeFlags::BlockScoped) {
        return false;
    }
    let mut node = list.parent();
    while let Some(current) = node {
        match current.kind() {
            Kind::Block => {
                return current.parent().is_some_and(|parent| {
                    !ast::is_function_like(Some(parent))
                        && parent.kind() != Kind::ClassStaticBlockDeclaration
                });
            }
            Kind::CaseBlock | Kind::WithStatement => return true,
            Kind::SourceFile | Kind::ModuleBlock | Kind::ClassStaticBlockDeclaration => {
                return false;
            }
            _ if ast::is_function_like(Some(current)) => return false,
            _ => node = current.parent(),
        }
    }
    false
}

/// The comments between the previous token and `node`, like ESLint's `sourceCode.getCommentsBefore(node)`, as byte
/// ranges. Unlike TypeScript's leading comments, this includes a comment on the same line as the previous token.
pub fn comments_before(ctx: &Ctx, node: P<Node>) -> Vec<(i32, i32)> {
    let Some(full_start) = node.pos().source_offset() else {
        return Vec::new();
    };
    let (start, _) = ctx.trim(node);
    let (Ok(full_start), Ok(start)) = (usize::try_from(full_start), usize::try_from(start)) else {
        return Vec::new();
    };
    let Some(trivia) = ctx.text().get(full_start..start) else {
        return Vec::new();
    };
    let mut comments = Vec::new();
    let mut offset = 0;
    if full_start == 0 && trivia.starts_with("#!") {
        offset = trivia.find(is_line_terminator).unwrap_or(trivia.len());
    }
    while let Some(rest) = trivia.get(offset..).filter(|rest| !rest.is_empty()) {
        let len = if rest.starts_with("//") {
            rest.find(is_line_terminator).unwrap_or(rest.len())
        } else if rest.starts_with("/*") {
            rest.get(2..)
                .and_then(|body| body.find("*/"))
                .map_or(rest.len(), |end| end + 4)
        } else {
            offset += rest.chars().next().map_or(1, char::len_utf8);
            continue;
        };
        if let (Ok(comment_start), Ok(comment_end)) = (
            i32::try_from(full_start + offset),
            i32::try_from(full_start + offset + len),
        ) {
            comments.push((comment_start, comment_end));
        }
        offset += len;
    }
    comments
}

/// Whether the Identifier `node` names a JSX tag or attribute (`<Foo>`, `<a.b>`, `<x:y>`, `<div id>`): ESTree has
/// `JSXIdentifier`s there, not `Identifier`s.
pub fn is_jsx_name(node: P<Node>) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        match parent.kind() {
            Kind::PropertyAccessExpression | Kind::JsxNamespacedName => current = parent,
            Kind::JsxOpeningElement | Kind::JsxSelfClosingElement | Kind::JsxClosingElement => {
                return parent.tag_name() == current;
            }
            Kind::JsxAttribute => return parent.name() == Some(current),
            _ => return false,
        }
    }
    false
}

/// Where `node` starts once the comments directly before it (see `comments_before`) are included.
pub fn start_with_comments_before(ctx: &Ctx, node: P<Node>) -> i32 {
    comments_before(ctx, node)
        .first()
        .map_or_else(|| ctx.trim(node).0, |&(start, _)| start)
}

/// The text of `node` preceded by the comments directly before it, one per line, the way ESLint rules that move
/// statements rebuild it: `comments.map(getText).join('\n') + '\n' + getText(node)`.
pub fn text_with_comments_before(ctx: &Ctx, node: P<Node>) -> String {
    let comments: Vec<&str> = comments_before(ctx, node)
        .iter()
        .filter_map(|&(start, end)| {
            let (start, end) = (usize::try_from(start).ok()?, usize::try_from(end).ok()?);
            ctx.text().get(start..end)
        })
        .collect();
    let text = node_text(ctx, node);
    if comments.is_empty() {
        text.to_string()
    } else {
        format!("{}\n{text}", comments.join("\n"))
    }
}

/// Where ESTree's node for the declaration `node` starts: TypeScript attaches `export` and `default` to the
/// declaration, while ESTree wraps it in an `ExportNamedDeclaration`/`ExportDefaultDeclaration` that owns them.
pub fn estree_declaration_start(ctx: &Ctx, node: P<Node>) -> i32 {
    let (start, _) = ctx.trim(node);
    let Some(last_export_modifier) = node
        .modifier_nodes()
        .iter()
        .rev()
        .find(|modifier| matches!(modifier.kind(), Kind::ExportKeyword | Kind::DefaultKeyword))
    else {
        return start;
    };
    last_export_modifier
        .end()
        .source_offset()
        .map(|end| tsrs_scanner::skip_trivia(ctx.text(), end))
        .and_then(|start| i32::try_from(start).ok())
        .unwrap_or(start)
}

/// Calls `f` on `node` and each of its descendants, in source order.
pub fn for_each_descendant<F: FnMut(P<Node>)>(node: P<Node>, f: &mut F) {
    f(node);
    node.for_each_child(&mut |child| {
        for_each_descendant(child, f);
        false
    });
}

/// The value of a string literal type.
pub fn literal_string_value(t: P<Type>) -> Option<&'static str> {
    match t.as_literal_type().value() {
        Some(LiteralValue::String(value)) => Some(value),
        _ => None,
    }
}

/// The values of a string literal type or a union of string literal types; `None` for any other type.
pub fn string_literal_values_of_type(t: P<Type>) -> Option<Vec<&'static str>> {
    if t.flags().intersects(TypeFlags::Any | TypeFlags::Unknown) {
        return None;
    }
    if t.flags().intersects(TypeFlags::Union) {
        let mut values = Vec::with_capacity(t.types().len());
        for part in t.types() {
            if !part.flags().intersects(TypeFlags::StringLiteral) {
                return None;
            }
            values.push(literal_string_value(*part)?);
        }
        return Some(values);
    }
    if t.flags().intersects(TypeFlags::StringLiteral) {
        return Some(vec![literal_string_value(t)?]);
    }
    None
}
