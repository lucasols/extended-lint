use rustc_hash::FxHashSet;

use crate::rules::normalize_rule_name;

const PREFIXES: &[&str] = &["eslint-", "oxlint-"];

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
    pos: usize,
    line: usize,
    /// None means every rule.
    rules: Option<Vec<String>>,
}

/// The `eslint-disable*` / `oxlint-disable*` directives of a file.
#[derive(Debug, Default)]
pub struct Suppressions {
    directives: Vec<Directive>,
}

impl Suppressions {
    /// `is_comment(pos)` tells whether the `//` or `/*` at byte offset `pos` opens a comment, as opposed to being
    /// part of a string, template or JSX text.
    pub fn parse(text: &str, is_comment: impl Fn(usize) -> bool) -> Suppressions {
        if !PREFIXES.iter().any(|p| text.contains(p)) {
            return Suppressions::default();
        }
        let line_starts = line_starts(text);
        let mut directives = Vec::new();
        for prefix in PREFIXES {
            for (idx, _) in text.match_indices(prefix) {
                if let Some(d) = parse_directive(text, idx, prefix.len(), &line_starts, &is_comment)
                {
                    directives.push(d);
                }
            }
        }
        directives.sort_by_key(|d| d.pos);
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

fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// 1-based line of byte offset `pos`.
fn line_of(line_starts: &[usize], pos: usize) -> usize {
    line_starts.partition_point(|&s| s <= pos)
}

/// A directive must open its comment: `// eslint-disable...`, `/* eslint-disable... */` or `{/* eslint-disable... */}`.
/// Like ESLint, a `disable-line` or `disable-next-line` block comment spanning several lines is ignored.
fn parse_directive(
    text: &str,
    idx: usize,
    prefix_len: usize,
    line_starts: &[usize],
    is_comment: &impl Fn(usize) -> bool,
) -> Option<Directive> {
    let line = line_of(line_starts, idx);
    let line_start = line_starts.get(line - 1).copied().unwrap_or(0);
    let before = text.get(line_start..idx)?.trim_end();
    let is_block = if before.ends_with("//") {
        false
    } else if before.ends_with("/*") {
        true
    } else {
        return None;
    };
    if !is_comment(line_start + before.len() - 2) {
        return None;
    }
    let rest = text.get(idx + prefix_len..)?;
    let (keyword, kind) = KINDS.iter().find(|(k, _)| rest.starts_with(k))?;
    let after = rest.get(keyword.len()..)?;
    if after
        .chars()
        .next()
        .is_some_and(|c| !c.is_whitespace() && c != '*')
    {
        return None;
    }
    let end = if is_block {
        after.find("*/").unwrap_or(after.len())
    } else {
        after.find(is_line_terminator).unwrap_or(after.len())
    };
    let body = after.get(..end)?;
    if matches!(kind, Kind::DisableLine | Kind::DisableNextLine)
        && body.contains(is_line_terminator)
    {
        return None;
    }
    let body = body.split(" --").next().unwrap_or(body);
    let rules: Vec<String> = body
        .split(',')
        .map(|r| normalize_rule_name(r.trim()).to_string())
        .filter(|r| !r.is_empty())
        .collect();
    Some(Directive {
        kind: *kind,
        pos: idx,
        line,
        rules: (!rules.is_empty()).then_some(rules),
    })
}

#[cfg(test)]
mod tests {
    use super::Suppressions;

    fn suppressed_with(
        text: &str,
        rule: &str,
        line: usize,
        is_comment: impl Fn(usize) -> bool,
    ) -> bool {
        let pos = text
            .split_inclusive('\n')
            .take(line - 1)
            .map(str::len)
            .sum();
        Suppressions::parse(text, is_comment).is_suppressed(rule, pos, line)
    }

    fn suppressed(text: &str, rule: &str, line: usize) -> bool {
        suppressed_with(text, rule, line, |_| true)
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
        assert!(!suppressed_with(text, "strict-casting", 2, |pos| pos != 11));
    }

    #[test]
    fn multiline_next_line_block_comment_is_ignored() {
        let text = "/* eslint-disable-next-line\n strict-casting */\nString(a)\nString(b)\n";
        assert!(!suppressed(text, "strict-casting", 2));
        assert!(!suppressed(text, "strict-casting", 3));
    }

    #[test]
    fn carriage_return_line_breaks() {
        let text = "// eslint-disable-next-line strict-casting\rString(a)\rString(b)\r";
        let suppressions = Suppressions::parse(text, |_| true);
        assert!(suppressions.is_suppressed("strict-casting", 43, 2));
        assert!(!suppressions.is_suppressed("strict-casting", 53, 3));
    }

    #[test]
    fn directive_text_outside_comment_start_is_ignored() {
        let text = "const s = 'eslint-disable-next-line strict-casting'\nString(a)\n";
        assert!(!suppressed(text, "strict-casting", 2));
    }
}
