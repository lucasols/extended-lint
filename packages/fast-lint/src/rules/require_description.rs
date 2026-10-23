use tsrs_ast::Node;
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::comment_at;

pub const NAME: &str = "require-description";

/// The directive kinds, as the first word of the comment, including fast-lint's own `fast-lint-*` directives.
const KINDS: &[&str] = &[
    "eslint",
    "eslint-disable",
    "eslint-disable-line",
    "eslint-disable-next-line",
    "eslint-enable",
    "eslint-env",
    "oxlint-disable",
    "oxlint-disable-line",
    "oxlint-disable-next-line",
    "oxlint-enable",
    "fast-lint-disable",
    "fast-lint-disable-line",
    "fast-lint-disable-next-line",
    "fast-lint-enable",
    "exported",
    "global",
    "globals",
];

/// Text a directive comment contains: every kind starts with one of these.
const NEEDLES: &[&str] = &["eslint", "oxlint-", "fast-lint-", "exported", "global"];

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    ignore: Vec<String>,
}

pub struct RequireDescription {
    ignore: Vec<&'static str>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut ignore = Vec::with_capacity(options.ignore.len());
    for kind in &options.ignore {
        let Some(&known) = KINDS.iter().find(|&&k| k == kind) else {
            return Err(format!("unknown directive kind in `ignore`: {kind}"));
        };
        ignore.push(known);
    }
    Ok(Box::new(RequireDescription { ignore }))
}

impl Rule for RequireDescription {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        self.check_comments(ctx);
        Box::new(Visitor)
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &[]
    }
    fn visit(&mut self, _ctx: &mut Ctx, _l: Listener, _node: P<Node>) {}
}

/// JavaScript's `\s`: Unicode `White_Space` and U+FEFF, but not U+0085.
fn is_js_whitespace(c: char) -> bool {
    c == '\u{FEFF}' || (c != '\u{85}' && c.is_whitespace())
}

fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// The byte length of a `\s-{2,}\s` separator starting at `rest`, if one does.
fn separator_len(rest: &str) -> Option<usize> {
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if !is_js_whitespace(first) {
        return None;
    }
    let after_space = first.len_utf8();
    let dashes = rest
        .get(after_space..)?
        .bytes()
        .take_while(|&b| b == b'-')
        .count();
    if dashes < 2 {
        return None;
    }
    let after_dashes = after_space + dashes;
    let last = rest.get(after_dashes..)?.chars().next()?;
    is_js_whitespace(last).then_some(after_dashes + last.len_utf8())
}

/// Splits a comment value like ESLint's `value.split(/\s-{2,}\s/u)`: the directive text and, when there is a
/// separator, the description (the part up to the next separator).
fn divide(value: &str) -> (&str, Option<&str>) {
    let mut parts = Vec::with_capacity(2);
    let mut start = 0;
    let mut i = 0;
    while i < value.len() && parts.len() < 2 {
        let Some(rest) = value.get(i..) else {
            break;
        };
        if let Some(len) = separator_len(rest) {
            parts.push(value.get(start..i).unwrap_or_default());
            i += len;
            start = i;
            continue;
        }
        i += rest.chars().next().map_or(1, char::len_utf8);
    }
    if parts.len() < 2 {
        parts.push(value.get(start..).unwrap_or_default());
    }
    let text = parts.first().copied().unwrap_or_default();
    (
        text.trim_matches(is_js_whitespace),
        parts.get(1).map(|d| d.trim_matches(is_js_whitespace)),
    )
}

impl RequireDescription {
    fn check_comments(&self, ctx: &mut Ctx) {
        let text = ctx.text();
        let mut comments: Vec<(u32, u32, bool)> = NEEDLES
            .iter()
            .flat_map(|needle| text.match_indices(needle))
            .filter_map(|(idx, _)| comment_at(ctx.file, u32::try_from(idx).ok()?))
            .collect();
        if comments.is_empty() {
            return;
        }
        comments.sort_unstable();
        comments.dedup();
        for (start, end, is_block) in comments {
            let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
                continue;
            };
            let Some(comment) = text.get(start..end) else {
                continue;
            };
            let value = comment.get(2..).unwrap_or_default();
            let value = if is_block {
                value.strip_suffix("*/").unwrap_or(value)
            } else {
                value
            };
            if !self.is_undescribed_directive(value, is_block, comment) {
                continue;
            }
            let line_start = text
                .get(..start)
                .and_then(|before| before.rfind(is_line_terminator))
                .map_or(0, |i| i + 1);
            let (Ok(pos), Ok(end)) = (i32::try_from(line_start), i32::try_from(end)) else {
                continue;
            };
            ctx.report_range(
                pos,
                end,
                RuleMessage::new(
                    "missingDescription",
                    "Unexpected undescribed directive comment. Include descriptions to explain why the comment is necessary.",
                ),
            );
        }
    }

    fn is_undescribed_directive(&self, value: &str, is_block: bool, comment: &str) -> bool {
        let (directive, description) = divide(value);
        let kind = directive.split(is_js_whitespace).next().unwrap_or_default();
        if !KINDS.contains(&kind) {
            return false;
        }
        let line_comment_supported =
            kind.ends_with("-disable-line") || kind.ends_with("-disable-next-line");
        if !is_block && !line_comment_supported {
            return false;
        }
        if line_comment_supported && comment.contains(is_line_terminator) {
            return false;
        }
        if self.ignore.contains(&kind) {
            return false;
        }
        description.is_none_or(str::is_empty)
    }
}
