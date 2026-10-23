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
    pub fn parse(text: &str) -> Suppressions {
        if !PREFIXES.iter().any(|p| text.contains(p)) {
            return Suppressions::default();
        }
        let line_starts = line_starts(text);
        let mut directives = Vec::new();
        for prefix in PREFIXES {
            for (idx, _) in text.match_indices(prefix) {
                if let Some(d) = parse_directive(text, idx, prefix.len(), &line_starts) {
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

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// 1-based line of byte offset `pos`.
fn line_of(line_starts: &[usize], pos: usize) -> usize {
    line_starts.partition_point(|&s| s <= pos)
}

/// A directive must open its comment: `// eslint-disable...`, `/* eslint-disable... */` or `{/* eslint-disable... */}`.
fn parse_directive(
    text: &str,
    idx: usize,
    prefix_len: usize,
    line_starts: &[usize],
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
        after.find('\n').unwrap_or(after.len())
    };
    let body = after.get(..end)?;
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

    fn suppressed(text: &str, rule: &str, line: usize) -> bool {
        let pos = text
            .split_inclusive('\n')
            .take(line - 1)
            .map(str::len)
            .sum();
        Suppressions::parse(text).is_suppressed(rule, pos, line)
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
    fn directive_text_outside_comment_start_is_ignored() {
        let text = "const s = 'eslint-disable-next-line strict-casting'\nString(a)\n";
        assert!(!suppressed(text, "strict-casting", 2));
    }
}
