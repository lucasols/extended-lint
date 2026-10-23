use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
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
    ignores: Option<GlobSet>,
    rules: Vec<(&'static str, RuleEntry)>,
}

/// A rule enabled for a file, with its resolved severity and options.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnabledRule {
    pub name: &'static str,
    pub severity: Severity,
    pub options: Option<String>,
}

pub struct Config {
    /// tsconfig paths, relative to the config directory.
    pub projects: Vec<String>,
    ignores: Option<GlobSet>,
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
                    .then(|| build_glob_set(&o.ignores))
                    .transpose()?,
                rules: parse_rules(&o.rules)?,
            });
        }
        Ok(Config {
            projects: raw.projects,
            ignores: (!raw.ignores.is_empty())
                .then(|| build_glob_set(&raw.ignores))
                .transpose()?,
            blocks,
        })
    }

    /// Every rule as `error` with default options: what runs when no config file is given.
    pub fn all_rules() -> Config {
        Config {
            projects: Vec::new(),
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

    pub fn is_ignored(&self, relative_path: &str) -> bool {
        self.ignores
            .as_ref()
            .is_some_and(|set| set.is_match(relative_path))
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
    for pattern in patterns {
        let pattern = pattern.strip_prefix("./").unwrap_or(pattern);
        let glob: Glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .map_err(|e| format!("invalid glob \"{pattern}\": {e}"))?;
        builder.add(glob);
        if let Some(dir) = pattern.strip_suffix('/') {
            let glob = GlobBuilder::new(&format!("{dir}/**"))
                .literal_separator(true)
                .build()
                .map_err(|e| format!("invalid glob \"{pattern}\": {e}"))?;
            builder.add(glob);
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
        parsed.push((name, entry));
    }
    parsed.sort_by(|a, b| a.0.cmp(b.0));
    Ok(parsed)
}
