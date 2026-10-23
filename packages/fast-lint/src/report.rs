use std::fmt::Write as _;

use serde::Serialize;

use crate::config::Severity;
use crate::linter::{Diagnostic, FileResult};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Stylish,
    Compact,
    Json,
}

impl Format {
    pub fn parse(value: &str) -> Result<Format, String> {
        match value {
            "stylish" => Ok(Format::Stylish),
            "compact" => Ok(Format::Compact),
            "json" => Ok(Format::Json),
            other => Err(format!(
                "unknown format \"{other}\", expected stylish, compact or json"
            )),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Counts {
    pub errors: usize,
    pub warnings: usize,
    pub fixable_errors: usize,
    pub fixable_warnings: usize,
}

pub fn count(files: &[FileResult]) -> Counts {
    let mut counts = Counts::default();
    for d in files.iter().flat_map(|f| &f.diagnostics) {
        let fixable = !d.fixes.is_empty();
        match d.severity {
            Severity::Error => {
                counts.errors += 1;
                counts.fixable_errors += usize::from(fixable);
            }
            Severity::Warn => {
                counts.warnings += 1;
                counts.fixable_warnings += usize::from(fixable);
            }
            Severity::Off => {}
        }
    }
    counts
}

fn display_path(root: &str, path: &str) -> String {
    let prefix = format!("{}/", root.trim_end_matches('/'));
    path.strip_prefix(&prefix).unwrap_or(path).to_string()
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

struct Style {
    color: bool,
}

impl Style {
    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

pub fn render(format: Format, root: &str, files: &[FileResult], color: bool) -> String {
    match format {
        Format::Stylish => stylish(root, files, &Style { color }),
        Format::Compact => compact(root, files),
        Format::Json => json(files),
    }
}

fn stylish(root: &str, files: &[FileResult], style: &Style) -> String {
    let mut out = String::new();
    for file in files.iter().filter(|f| !f.diagnostics.is_empty()) {
        let _ = writeln!(
            out,
            "\n{}",
            style.paint("4", &display_path(root, &file.path))
        );
        let position_width = file
            .diagnostics
            .iter()
            .map(|d| format!("{}:{}", d.line, d.column).len())
            .max()
            .unwrap_or(0);
        let message_width = file
            .diagnostics
            .iter()
            .map(|d| d.message.chars().count())
            .max()
            .unwrap_or(0);
        for d in &file.diagnostics {
            let position = format!("{}:{}", d.line, d.column);
            let severity = match d.severity {
                Severity::Error => style.paint("31", "error  "),
                _ => style.paint("33", "warning"),
            };
            let padding = message_width.saturating_sub(d.message.chars().count());
            let _ = writeln!(
                out,
                "  {}  {severity}  {}{}  {}",
                style.paint("2", &format!("{position:<position_width$}")),
                d.message,
                " ".repeat(padding),
                style.paint("2", d.rule)
            );
        }
    }
    let counts = count(files);
    let total = counts.errors + counts.warnings;
    if total > 0 {
        let summary = format!(
            "\n✖ {} ({}, {})\n",
            plural(total, "problem"),
            plural(counts.errors, "error"),
            plural(counts.warnings, "warning")
        );
        let code = if counts.errors > 0 { "1;31" } else { "1;33" };
        out.push_str(&style.paint(code, &summary));
        if counts.fixable_errors + counts.fixable_warnings > 0 {
            let _ = writeln!(
                out,
                "  {} and {} potentially fixable with the `--fix` option.",
                plural(counts.fixable_errors, "error"),
                plural(counts.fixable_warnings, "warning")
            );
        }
    }
    out
}

fn compact(root: &str, files: &[FileResult]) -> String {
    let mut out = String::new();
    for file in files {
        let path = display_path(root, &file.path);
        for d in &file.diagnostics {
            let severity = match d.severity {
                Severity::Error => "Error",
                _ => "Warning",
            };
            let _ = writeln!(
                out,
                "{path}:{}:{}: {severity} - {} ({})",
                d.line, d.column, d.message, d.rule
            );
        }
    }
    out
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonFix<'a> {
    range: [i32; 2],
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonSuggestion<'a> {
    message_id: &'a str,
    desc: &'a str,
    fixes: Vec<JsonFix<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonMessage<'a> {
    rule_id: &'a str,
    severity: u8,
    message_id: &'a str,
    message: &'a str,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
    fixes: Vec<JsonFix<'a>>,
    suggestions: Vec<JsonSuggestion<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonFile<'a> {
    file_path: &'a str,
    messages: Vec<JsonMessage<'a>>,
}

/// Converts byte offsets of a file's text to the UTF-16 indices ESLint uses for fix ranges.
struct Utf16Offsets<'a> {
    /// `None` for ASCII text, where both are the same.
    text: Option<&'a str>,
}

impl Utf16Offsets<'_> {
    fn new(text: &str) -> Utf16Offsets<'_> {
        Utf16Offsets {
            text: (!text.is_ascii()).then_some(text),
        }
    }

    fn get(&self, offset: i32) -> i32 {
        let Some(text) = self.text else {
            return offset;
        };
        let offset = usize::try_from(offset).unwrap_or(0);
        let units = text
            .get(..offset)
            .map_or(0, |prefix| prefix.encode_utf16().count());
        i32::try_from(units).unwrap_or(i32::MAX)
    }
}

fn json_fixes<'a>(text: &Utf16Offsets, fixes: &'a [tsrslint::rule::RuleFix]) -> Vec<JsonFix<'a>> {
    fixes
        .iter()
        .map(|f| JsonFix {
            range: [text.get(f.pos), text.get(f.end)],
            text: &f.text,
        })
        .collect()
}

fn json_message<'a>(text: &Utf16Offsets, d: &'a Diagnostic) -> JsonMessage<'a> {
    JsonMessage {
        rule_id: d.rule,
        severity: match d.severity {
            Severity::Error => 2,
            Severity::Warn => 1,
            Severity::Off => 0,
        },
        message_id: d.message_id,
        message: &d.message,
        line: d.line,
        column: d.column,
        end_line: d.end_line,
        end_column: d.end_column,
        fixes: json_fixes(text, &d.fixes),
        suggestions: d
            .suggestions
            .iter()
            .map(|s| JsonSuggestion {
                message_id: s.message.id,
                desc: &s.message.description,
                fixes: json_fixes(text, &s.fixes),
            })
            .collect(),
    }
}

fn json(files: &[FileResult]) -> String {
    let files: Vec<JsonFile> = files
        .iter()
        .map(|f| JsonFile {
            file_path: &f.path,
            messages: {
                let offsets = Utf16Offsets::new(f.text());
                f.diagnostics
                    .iter()
                    .map(|d| json_message(&offsets, d))
                    .collect()
            },
        })
        .collect();
    serde_json::to_string(&files).unwrap_or_else(|_| "[]".to_string())
}
