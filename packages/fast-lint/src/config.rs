use std::borrow::Cow;

use globset::{Glob, GlobBuilder, GlobMatcher, GlobSet, GlobSetBuilder};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::rules::{normalize_rule_name, RULE_NAMES};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Off,
    Warn,
    Error,
}

impl Severity {
    fn parse(value: &serde_json::Value) -> Result<Severity, String> {
        match value {
            serde_json::Value::String(s) => match s.as_str() {
                "off" => Ok(Severity::Off),
                "warn" => Ok(Severity::Warn),
                "error" => Ok(Severity::Error),
                other => Err(format!("invalid severity \"{other}\"")),
            },
            serde_json::Value::Number(n) => match n.as_u64() {
                Some(0) => Ok(Severity::Off),
                Some(1) => Ok(Severity::Warn),
                Some(2) => Ok(Severity::Error),
                _ => Err(format!("invalid severity {n}")),
            },
            other => Err(format!("invalid severity {other}")),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Severity::Off => "off",
            Severity::Warn => "warning",
            Severity::Error => "error",
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RawConfig {
    #[serde(default, rename = "$schema")]
    _schema: Option<String>,
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    ignores: Vec<String>,
    #[serde(default)]
    rules: FxHashMap<String, serde_json::Value>,
    #[serde(default)]
    overrides: Vec<RawOverride>,
    #[serde(default)]
    report_unused_disable_directives: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RawOverride {
    files: Vec<String>,
    #[serde(default)]
    ignores: Vec<String>,
    #[serde(default)]
    rules: FxHashMap<String, serde_json::Value>,
}

/// One rule entry of a config block. `options` is None when the entry only sets the severity, which keeps the options
/// of earlier blocks, as in ESLint flat configs.
#[derive(Clone, Debug)]
struct RuleEntry {
    severity: Severity,
    options: Option<serde_json::Value>,
}

struct Block {
    files: Option<GlobSet>,
    ignores: Option<FileIgnores>,
    rules: Vec<(&'static str, RuleEntry)>,
}

/// The `ignores` of an override: patterns matched against the file in order, `!pattern` un-ignoring what earlier
/// ones ignored, like a config object's `ignores` in ESLint.
struct FileIgnores {
    patterns: Vec<(GlobSet, bool)>,
}

impl FileIgnores {
    fn new(patterns: &[String]) -> Result<FileIgnores, String> {
        let patterns = patterns
            .iter()
            .map(|raw| {
                let (negated, pattern) = match raw.strip_prefix('!') {
                    Some(pattern) => (true, pattern),
                    None => (false, raw.as_str()),
                };
                Ok((build_glob_set(&[pattern.to_string()])?, negated))
            })
            .collect::<Result<_, String>>()?;
        Ok(FileIgnores { patterns })
    }

    fn is_match(&self, path: &str) -> bool {
        let mut ignored = false;
        for (set, negated) in &self.patterns {
            if set.is_match(path) {
                ignored = !negated;
            }
        }
        ignored
    }
}

/// minimatch's `@(a|b)` extglob as a globset alternative; the other extglobs (`?(…)`, `*(…)`, `+(…)`, `!(…)`) have no
/// equivalent and are rejected rather than matched literally.
fn translate_extglobs(pattern: &str) -> Result<Cow<'_, str>, String> {
    let bytes = pattern.as_bytes();
    let has_extglob = bytes
        .windows(2)
        .any(|pair| matches!(pair, [b'@' | b'?' | b'*' | b'+' | b'!', b'(']));
    if !has_extglob {
        return Ok(Cow::Borrowed(pattern));
    }
    let unsupported = || {
        format!(
            "invalid glob \"{pattern}\": only the @(a|b) extglob is supported, use {{a,b}} instead"
        )
    };
    let mut out = String::with_capacity(pattern.len());
    let mut rest = pattern;
    while let Some(open) = rest.find('(') {
        let (before, after) = rest.split_at(open);
        let after = after.get(1..).unwrap_or_default();
        match before.chars().last() {
            Some('@') => {}
            Some('?' | '*' | '+' | '!') => return Err(unsupported()),
            _ => {
                out.push_str(before);
                out.push('(');
                rest = after;
                continue;
            }
        };
        let Some(close) = after.find(')') else {
            return Err(unsupported());
        };
        let (inner, tail) = after.split_at(close);
        if inner.contains(['(', ',', '{', '}']) {
            return Err(unsupported());
        }
        out.push_str(before.get(..before.len() - 1).unwrap_or_default());
        out.push('{');
        out.push_str(&inner.replace('|', ","));
        out.push('}');
        rest = tail.get(1..).unwrap_or_default();
    }
    out.push_str(rest);
    Ok(Cow::Owned(out))
}

fn glob(raw: &str, pattern: &str) -> Result<Glob, String> {
    GlobBuilder::new(&translate_extglobs(pattern)?)
        .literal_separator(true)
        .build()
        .map_err(|e| format!("invalid glob \"{raw}\": {e}"))
}

/// A rule enabled for a file, with its resolved severity and options.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnabledRule {
    pub name: &'static str,
    pub severity: Severity,
    pub options: Option<String>,
}

/// One pattern of the global `ignores`.
struct IgnorePattern {
    matcher: GlobMatcher,
    /// `!pattern`: un-ignores what earlier patterns ignored.
    negated: bool,
    /// `dir/`: matches directories only.
    directory_only: bool,
    /// For `dir/**` (and `dir/**/`): matches `dir` itself as a directory, as minimatch's `**` matches nothing.
    directory_itself: Option<GlobMatcher>,
}

/// Global ignores with ESLint flat config semantics: a pattern matching a directory ignores everything in it, and
/// `!` patterns, applied in order, un-ignore files (but not files in an ignored directory).
struct Ignores {
    patterns: Vec<IgnorePattern>,
}

impl Ignores {
    fn new(patterns: &[String]) -> Result<Ignores, String> {
        let patterns = patterns
            .iter()
            .map(|raw| {
                let (negated, pattern) = match raw.strip_prefix('!') {
                    Some(pattern) => (true, pattern),
                    None => (false, raw.as_str()),
                };
                let pattern = pattern.strip_prefix("./").unwrap_or(pattern);
                let (directory_only, pattern) = match pattern.strip_suffix('/') {
                    Some(dir) => (true, dir),
                    None => (false, pattern),
                };
                let build = |pattern: &str| glob(raw, pattern).map(|glob| glob.compile_matcher());
                let directory_itself = pattern
                    .strip_suffix("/**")
                    .filter(|dir| !dir.is_empty())
                    .map(build)
                    .transpose()?;
                Ok(IgnorePattern {
                    matcher: build(pattern)?,
                    negated,
                    directory_only,
                    directory_itself,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Ignores { patterns })
    }

    fn matches(&self, path: &str, is_directory: bool) -> bool {
        let mut ignored = false;
        for pattern in &self.patterns {
            let matches_directory_itself = is_directory
                && pattern
                    .directory_itself
                    .as_ref()
                    .is_some_and(|matcher| matcher.is_match(path));
            if (is_directory || !pattern.directory_only) && pattern.matcher.is_match(path)
                || matches_directory_itself
            {
                ignored = !pattern.negated;
            }
        }
        ignored
    }

    fn is_ignored(&self, relative_path: &str) -> bool {
        let is_ignored_directory = |dir: &str| {
            !dir.is_empty() && dir != ".." && !dir.ends_with("/..") && self.matches(dir, true)
        };
        relative_path
            .match_indices('/')
            .any(|(i, _)| relative_path.get(..i).is_some_and(is_ignored_directory))
            || self.matches(relative_path, false)
    }
}

pub struct Config {
    /// tsconfig paths, relative to the config directory.
    pub projects: Vec<String>,
    /// Report `fast-lint-disable*` directives that suppressed nothing.
    pub report_unused_disable_directives: bool,
    ignores: Option<Ignores>,
    blocks: Vec<Block>,
}

impl Config {
    pub fn from_json(text: &str) -> Result<Config, String> {
        let raw: RawConfig =
            serde_json::from_str(text).map_err(|e| format!("invalid config: {e}"))?;
        let mut blocks = vec![Block {
            files: None,
            ignores: None,
            rules: parse_rules(&raw.rules)?,
        }];
        for o in &raw.overrides {
            if o.files.is_empty() {
                return Err(
                    "invalid config: an override needs at least one `files` pattern".into(),
                );
            }
            blocks.push(Block {
                files: Some(build_glob_set(&o.files)?),
                ignores: (!o.ignores.is_empty())
                    .then(|| FileIgnores::new(&o.ignores))
                    .transpose()?,
                rules: parse_rules(&o.rules)?,
            });
        }
        Ok(Config {
            projects: raw.projects,
            report_unused_disable_directives: raw.report_unused_disable_directives,
            ignores: (!raw.ignores.is_empty())
                .then(|| Ignores::new(&raw.ignores))
                .transpose()?,
            blocks,
        })
    }

    /// Every rule as `error` with default options: what runs when no config file is given.
    pub fn all_rules() -> Config {
        Config {
            projects: Vec::new(),
            report_unused_disable_directives: false,
            ignores: None,
            blocks: vec![Block {
                files: None,
                ignores: None,
                rules: RULE_NAMES
                    .iter()
                    .map(|&name| {
                        (
                            name,
                            RuleEntry {
                                severity: Severity::Error,
                                options: None,
                            },
                        )
                    })
                    .collect(),
            }],
        }
    }

    /// Whether the global `ignores` exclude a file (path relative to the config directory, `/`-separated).
    pub fn is_ignored(&self, relative_path: &str) -> bool {
        self.ignores
            .as_ref()
            .is_some_and(|ignores| ignores.is_ignored(relative_path))
    }

    /// The rules enabled for a file (path relative to the config directory, `/`-separated), sorted by name.
    pub fn rules_for(&self, relative_path: &str) -> Vec<EnabledRule> {
        let mut resolved: FxHashMap<&'static str, RuleEntry> = FxHashMap::default();
        for block in &self.blocks {
            if let Some(files) = &block.files {
                if !files.is_match(relative_path) {
                    continue;
                }
            }
            if let Some(ignores) = &block.ignores {
                if ignores.is_match(relative_path) {
                    continue;
                }
            }
            for (name, entry) in &block.rules {
                let current = resolved.entry(name).or_insert(RuleEntry {
                    severity: Severity::Off,
                    options: None,
                });
                current.severity = entry.severity;
                if entry.options.is_some() {
                    current.options.clone_from(&entry.options);
                }
            }
        }
        let mut enabled: Vec<EnabledRule> = resolved
            .into_iter()
            .filter(|(_, e)| e.severity != Severity::Off)
            .map(|(name, e)| EnabledRule {
                name,
                severity: e.severity,
                options: e.options.map(|o| o.to_string()),
            })
            .collect();
        enabled.sort_by(|a, b| a.name.cmp(b.name));
        enabled
    }
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for raw in patterns {
        let pattern = raw.strip_prefix("./").unwrap_or(raw);
        builder.add(glob(raw, pattern)?);
        if let Some(dir) = pattern.strip_suffix('/') {
            builder.add(glob(raw, &format!("{dir}/**"))?);
        }
    }
    builder.build().map_err(|e| e.to_string())
}

fn parse_rules(
    rules: &FxHashMap<String, serde_json::Value>,
) -> Result<Vec<(&'static str, RuleEntry)>, String> {
    let mut parsed = Vec::with_capacity(rules.len());
    for (key, value) in rules {
        let normalized = normalize_rule_name(key);
        let Some(&name) = RULE_NAMES.iter().find(|&&n| n == normalized) else {
            return Err(format!(
                "invalid config: unknown rule \"{key}\", available rules: {}",
                RULE_NAMES.join(", ")
            ));
        };
        let entry = match value {
            serde_json::Value::Array(items) => {
                let Some(severity) = items.first() else {
                    return Err(format!("invalid config: rule \"{key}\" has an empty entry"));
                };
                let severity =
                    Severity::parse(severity).map_err(|e| format!("rule \"{key}\": {e}"))?;
                let options = match items.len() {
                    1 => None,
                    2 => items.get(1).cloned(),
                    _ => {
                        return Err(format!(
                            "invalid config: rule \"{key}\" takes a single options object"
                        ))
                    }
                };
                RuleEntry { severity, options }
            }
            value => RuleEntry {
                severity: Severity::parse(value).map_err(|e| format!("rule \"{key}\": {e}"))?,
                options: None,
            },
        };
        crate::rules::create_rule(name, entry.options.as_ref())?;
        if parsed.iter().any(|(n, _)| *n == name) {
            return Err(format!(
                "invalid config: rule \"{name}\" is configured twice in one block (with and without a plugin prefix)"
            ));
        }
        parsed.push((name, entry));
    }
    parsed.sort_by(|a, b| a.0.cmp(b.0));
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::Config;

    fn rule_names(config: &Config, path: &str) -> Vec<&'static str> {
        config.rules_for(path).iter().map(|r| r.name).collect()
    }

    #[test]
    fn override_ignores_apply_negation_in_order() {
        let config = Config::from_json(
            r#"{ "overrides": [{ "files": ["**/*.ts"], "ignores": ["**/*.ts", "!src/*.ts"], "rules": { "strict-casting": "error" } }] }"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(rule_names(&config, "src/a.ts"), ["strict-casting"]);
        assert!(rule_names(&config, "lib/a.ts").is_empty());
    }

    #[test]
    fn at_extglobs_are_alternatives() {
        let config = Config::from_json(
            r#"{ "ignores": ["**/*.@(test|spec).ts"], "overrides": [{ "files": ["src/@(a|b).ts"], "rules": { "strict-casting": "error" } }] }"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert!(config.is_ignored("src/a.test.ts"));
        assert!(config.is_ignored("src/a.spec.ts"));
        assert!(!config.is_ignored("src/a.ts"));
        assert_eq!(rule_names(&config, "src/b.ts"), ["strict-casting"]);
        assert!(rule_names(&config, "src/c.ts").is_empty());
    }

    #[test]
    fn other_extglobs_are_rejected() {
        let error = Config::from_json(r#"{ "ignores": ["**/?(a|b).ts"] }"#)
            .err()
            .unwrap_or_default();
        assert_eq!(
            error,
            r#"invalid glob "**/?(a|b).ts": only the @(a|b) extglob is supported, use {a,b} instead"#
        );
    }

    #[test]
    fn rule_configured_with_and_without_prefix_is_rejected() {
        let error = Config::from_json(
            r#"{ "rules": { "strict-casting": "off", "@ls-stack/strict-casting": "error" } }"#,
        )
        .err()
        .unwrap_or_default();
        assert_eq!(
            error,
            r#"invalid config: rule "strict-casting" is configured twice in one block (with and without a plugin prefix)"#
        );
    }
}
