use rustc_hash::FxHashSet;

use crate::rules::normalize_rule_name;

/// `fast-lint-` directives are ignored by ESLint, so `eslint --fix` doesn't remove them as unused when these rules are
/// off in ESLint.
const PREFIXES: &[&str] = &["eslint-", "oxlint-", "fast-lint-"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    DisableNextLine,
    DisableLine,
    Disable,
    Enable,
}

const KINDS: &[(&str, Kind)] = &[
    ("disable-next-line", Kind::DisableNextLine),
    ("disable-line", Kind::DisableLine),
    ("disable", Kind::Disable),
    ("enable", Kind::Enable),
];

#[derive(Debug)]
struct Directive {
    kind: Kind,
    /// Byte offset of the comment start: `disable` and `enable` apply from there.
    pos: usize,
    /// The 1-based line `disable-line` applies to; `disable-next-line` applies to the line after it.
    line: usize,
    /// None means every rule.
    rules: Option<Vec<String>>,
}

/// A comment: its byte range and whether it is a block comment.
pub type Comment = (usize, usize, bool);

/// The `eslint-disable*` / `oxlint-disable*` / `fast-lint-disable*` directives of a file.
#[derive(Debug, Default)]
pub struct Suppressions {
    directives: Vec<Directive>,
}

impl Suppressions {
    /// `comment_at(pos)` gives the comment containing byte offset `pos`, if any: directive text in a string, a
    /// template or JSX text is not a directive.
    pub fn parse(text: &str, comment_at: impl Fn(usize) -> Option<Comment>) -> Suppressions {
        if !PREFIXES.iter().any(|p| text.contains(p)) {
            return Suppressions::default();
        }
        let line_starts = line_starts(text);
        let mut comments: Vec<Comment> = PREFIXES
            .iter()
            .flat_map(|prefix| text.match_indices(prefix))
            .filter_map(|(idx, _)| comment_at(idx))
            .collect();
        comments.sort_unstable();
        comments.dedup();
        let directives = comments
            .into_iter()
            .filter_map(|comment| parse_directive(text, comment, &line_starts))
            .collect();
        Suppressions { directives }
    }

    /// Whether a diagnostic of `rule` starting at byte offset `pos` on 1-based `line` is disabled.
    pub fn is_suppressed(&self, rule: &str, pos: usize, line: usize) -> bool {
        let mut disabled_all = false;
        let mut disabled: FxHashSet<&str> = FxHashSet::default();
        let mut reenabled: FxHashSet<&str> = FxHashSet::default();
        for d in &self.directives {
            let matches = |rules: &Option<Vec<String>>| {
                rules
                    .as_ref()
                    .is_none_or(|names| names.iter().any(|n| n == rule))
            };
            match d.kind {
                Kind::DisableNextLine => {
                    if d.line + 1 == line && matches(&d.rules) {
                        return true;
                    }
                }
                Kind::DisableLine => {
                    if d.line == line && matches(&d.rules) {
                        return true;
                    }
                }
                Kind::Disable if d.pos < pos => match &d.rules {
                    None => {
                        disabled_all = true;
                        reenabled.clear();
                    }
                    Some(names) => {
                        for n in names {
                            disabled.insert(n);
                            reenabled.remove(n.as_str());
                        }
                    }
                },
                Kind::Enable if d.pos < pos => match &d.rules {
                    None => {
                        disabled_all = false;
                        disabled.clear();
                        reenabled.clear();
                    }
                    Some(names) => {
                        for n in names {
                            disabled.remove(n.as_str());
                            if disabled_all {
                                reenabled.insert(n);
                            }
                        }
                    }
                },
                Kind::Disable | Kind::Enable => {}
            }
        }
        (disabled_all && !reenabled.contains(rule)) || disabled.contains(rule)
    }
}

/// Line start byte offsets with ECMAScript line terminators (`\n`, `\r\n`, `\r`, U+2028, U+2029), like tsrs's line map.
pub fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '\r' => {
                if chars.next_if(|&(_, next)| next == '\n').is_some() {
                    starts.push(i + 2);
                } else {
                    starts.push(i + 1);
                }
            }
            '\n' | '\u{2028}' | '\u{2029}' => starts.push(i + c.len_utf8()),
            _ => {}
        }
    }
    starts
}

/// 1-based line of byte offset `pos`.
fn line_of(line_starts: &[usize], pos: usize) -> usize {
    line_starts.partition_point(|&s| s <= pos)
}

/// The comment text without `//`, `/*` and `*/`.
fn comment_value(text: &str, (start, end, is_block): Comment) -> Option<&str> {
    let comment = text.get(start..end)?;
    let value = comment.get(2..)?;
    if is_block {
        Some(value.strip_suffix("*/").unwrap_or(value))
    } else {
        Some(value)
    }
}

/// JavaScript's whitespace (`\s`, what ESLint's directive parsing uses): Unicode `White_Space` and U+FEFF, but not
/// U+0085.
fn is_js_whitespace(c: char) -> bool {
    c == '\u{FEFF}' || (c != '\u{85}' && c.is_whitespace())
}

/// The directive part of a comment value, before a `--` description separator surrounded by whitespace.
fn directive_part(value: &str) -> &str {
    for (i, c) in value.char_indices() {
        if !is_js_whitespace(c) {
            continue;
        }
        let rest = value.get(i + c.len_utf8()..).unwrap_or_default();
        let dashes = rest.len() - rest.trim_start_matches('-').len();
        if dashes >= 2
            && rest
                .get(dashes..)
                .and_then(|after| after.chars().next())
                .is_some_and(is_js_whitespace)
        {
            return value
                .get(..i)
                .unwrap_or(value)
                .trim_matches(is_js_whitespace);
        }
    }
    value.trim_matches(is_js_whitespace)
}

/// The directive label: lowercase words joined by single hyphens, followed by whitespace or the end.
fn label_of(directive: &str) -> Option<&str> {
    let end = directive.find(is_js_whitespace).unwrap_or(directive.len());
    let label = directive.get(..end)?;
    let valid = label
        .split('-')
        .all(|word| !word.is_empty() && word.bytes().all(|b| b.is_ascii_lowercase()));
    valid.then_some(label)
}

/// Parses a comment like ESLint does: the trimmed comment text must start with the directive, file-wide `disable`
/// and `enable` need a block comment, and a `disable-line` comment may not span several lines. `disable-next-line`
/// applies to the line after the comment's last line.
fn parse_directive(text: &str, comment: Comment, line_starts: &[usize]) -> Option<Directive> {
    let directive = directive_part(comment_value(text, comment)?);
    let label = label_of(directive)?;
    let name = PREFIXES
        .iter()
        .find_map(|prefix| label.strip_prefix(prefix))?;
    let &(_, kind) = KINDS.iter().find(|(k, _)| *k == name)?;
    let (start, end, is_block) = comment;
    if !is_block && matches!(kind, Kind::Disable | Kind::Enable) {
        return None;
    }
    let start_line = line_of(line_starts, start);
    let end_line = line_of(line_starts, end.saturating_sub(1).max(start));
    if kind == Kind::DisableLine && start_line != end_line {
        return None;
    }
    let rules: Vec<String> = directive
        .get(label.len()..)?
        .split(',')
        .map(|r| {
            let r = r.trim_matches(is_js_whitespace);
            let unquoted = ['"', '\'']
                .iter()
                .find_map(|&q| r.strip_prefix(q).and_then(|r| r.strip_suffix(q)))
                .unwrap_or(r);
            normalize_rule_name(unquoted).to_string()
        })
        .filter(|r| !r.is_empty())
        .collect();
    Some(Directive {
        kind,
        pos: start,
        line: if kind == Kind::DisableNextLine {
            end_line
        } else {
            start_line
        },
        rules: (!rules.is_empty()).then_some(rules),
    })
}

#[cfg(test)]
mod tests {
    use super::{Comment, Suppressions};

    /// Comments of test texts, which have no strings containing `//` or `/*`.
    fn naive_comment_at(text: &str, pos: usize) -> Option<Comment> {
        let mut offset = 0;
        while offset < text.len() {
            let rest = text.get(offset..)?;
            let (len, is_block) = if rest.starts_with("//") {
                (rest.find(['\n', '\r']).unwrap_or(rest.len()), false)
            } else if rest.starts_with("/*") {
                (rest.find("*/").map_or(rest.len(), |end| end + 2), true)
            } else {
                offset += rest.chars().next().map_or(1, char::len_utf8);
                continue;
            };
            if offset <= pos && pos < offset + len {
                return Some((offset, offset + len, is_block));
            }
            offset += len;
        }
        None
    }

    fn parse(text: &str) -> Suppressions {
        Suppressions::parse(text, |pos| naive_comment_at(text, pos))
    }

    fn suppressed(text: &str, rule: &str, line: usize) -> bool {
        let pos = text
            .split_inclusive('\n')
            .take(line - 1)
            .map(str::len)
            .sum();
        parse(text).is_suppressed(rule, pos, line)
    }

    #[test]
    fn next_line_with_plugin_prefix_and_description() {
        let text = "// eslint-disable-next-line @ls-stack/strict-casting -- reason\nString(a)\nString(b)\n";
        assert!(suppressed(text, "strict-casting", 2));
        assert!(!suppressed(text, "strict-casting", 3));
        assert!(!suppressed(text, "no-unnecessary-void-on-promise", 2));
    }

    #[test]
    fn same_line_and_jsx_comment() {
        let text = "a // oxlint-disable-line strict-casting\n{/* eslint-disable-next-line @eslint-react/no-leaked-conditional-rendering */}\nb\n";
        assert!(suppressed(text, "strict-casting", 1));
        assert!(suppressed(text, "no-leaked-conditional-rendering", 3));
    }

    #[test]
    fn fast_lint_directives() {
        let text = "// fast-lint-disable-next-line strict-casting\nString(a)\n/* fast-lint-disable */\nb\n";
        assert!(suppressed(text, "strict-casting", 2));
        assert!(suppressed(text, "no-unnecessary-void-on-promise", 4));
    }

    #[test]
    fn block_disable_and_enable() {
        let text = "/* eslint-disable @ls-stack/strict-casting */\na\n/* eslint-enable @ls-stack/strict-casting */\nb\n";
        assert!(suppressed(text, "strict-casting", 2));
        assert!(!suppressed(text, "strict-casting", 4));
    }

    #[test]
    fn disable_all_with_single_reenabled_rule() {
        let text = "/* eslint-disable */\na\n/* eslint-enable strict-casting */\nb\n";
        assert!(suppressed(text, "no-unnecessary-void-on-promise", 4));
        assert!(!suppressed(text, "strict-casting", 4));
    }

    #[test]
    fn directive_outside_a_comment_is_ignored() {
        let text = "const s = '/* eslint-disable */'\nString(a)\n";
        let suppressions = Suppressions::parse(text, |_| None);
        assert!(!suppressions.is_suppressed("strict-casting", 33, 2));
    }

    #[test]
    fn multiline_next_line_block_comment_applies_after_its_end() {
        let text = "/* eslint-disable-next-line\n strict-casting */\nString(a)\nString(b)\n";
        assert!(!suppressed(text, "strict-casting", 2));
        assert!(suppressed(text, "strict-casting", 3));
        assert!(!suppressed(text, "strict-casting", 4));
    }

    #[test]
    fn multiline_disable_line_block_comment_is_ignored() {
        let text = "String(a) /* eslint-disable-line\n strict-casting */\n";
        assert!(!suppressed(text, "strict-casting", 1));
        assert!(!suppressed(text, "strict-casting", 2));
    }

    #[test]
    fn directive_on_a_later_line_of_a_block_comment() {
        let text = "/*\n eslint-disable strict-casting\n */\nString(a)\n";
        assert!(suppressed(text, "strict-casting", 4));
    }

    #[test]
    fn directive_inside_comment_text_is_ignored() {
        let text = "// to silence it, add /* eslint-disable */ at the top\nString(a)\n// /* eslint-disable-next-line */\nString(b)\n";
        assert!(!suppressed(text, "strict-casting", 2));
        assert!(!suppressed(text, "strict-casting", 4));
    }

    #[test]
    fn file_wide_directives_need_a_block_comment() {
        let text = "// eslint-disable strict-casting\nString(a)\n/* eslint-disable strict-casting */\n// eslint-enable strict-casting\nString(b)\n";
        assert!(!suppressed(text, "strict-casting", 2));
        assert!(suppressed(text, "strict-casting", 5));
    }

    #[test]
    fn description_separator_needs_surrounding_whitespace() {
        let text = "// eslint-disable-next-line strict-casting\t-- why\nString(a)\n// eslint-disable-next-line strict-casting --why\nString(b)\n";
        assert!(suppressed(text, "strict-casting", 2));
        assert!(!suppressed(text, "strict-casting", 4));
    }

    #[test]
    fn carriage_return_line_breaks() {
        let text = "// eslint-disable-next-line strict-casting\rString(a)\rString(b)\r";
        let suppressions = parse(text);
        assert!(suppressions.is_suppressed("strict-casting", 43, 2));
        assert!(!suppressions.is_suppressed("strict-casting", 53, 3));
    }

    #[test]
    fn directive_whitespace_is_javascript_whitespace() {
        let text = "//\u{FEFF}eslint-disable-next-line strict-casting\nString(a)\n//\u{85}eslint-disable-next-line strict-casting\nString(b)\n";
        assert!(suppressed(text, "strict-casting", 2));
        assert!(!suppressed(text, "strict-casting", 4));
    }

    #[test]
    fn directive_text_outside_comment_start_is_ignored() {
        let text = "const s = 'eslint-disable-next-line strict-casting'\nString(a)\n";
        assert!(!suppressed(text, "strict-casting", 2));
    }
}
