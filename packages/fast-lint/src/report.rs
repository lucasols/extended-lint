use std::fmt::Write as _;

use serde::Serialize;

use crate::config::Severity;
use crate::linter::{Diagnostic, FileResult};
use tsrslint::rule::RuleFix;

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

/// Renders the results; `cwd` is what stylish and compact paths are relative to.
pub fn render(format: Format, cwd: &str, files: &[FileResult], color: bool) -> String {
    match format {
        Format::Stylish => stylish(cwd, files, &Style { color }),
        Format::Compact => compact(cwd, files),
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

/// ESLint's `compact` format (`eslint-formatter-compact`), with paths relative to the current directory.
fn compact(root: &str, files: &[FileResult]) -> String {
    let mut out = String::new();
    let mut total = 0;
    for file in files {
        let path = display_path(root, &file.path);
        for d in &file.diagnostics {
            total += 1;
            let severity = if d.fatal || d.severity == Severity::Error {
                "Error"
            } else {
                "Warning"
            };
            let rule = if d.rule.is_empty() {
                String::new()
            } else {
                format!(" ({})", d.rule)
            };
            let _ = writeln!(
                out,
                "{path}: line {}, col {}, {severity} - {}{rule}",
                d.line, d.column, d.message
            );
        }
    }
    if total > 0 {
        let _ = writeln!(
            out,
            "\n{total} problem{}",
            if total == 1 { "" } else { "s" }
        );
    }
    out
}

#[derive(Serialize)]
struct JsonFix {
    range: [i32; 2],
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonSuggestion<'a> {
    message_id: &'a str,
    desc: &'a str,
    fix: JsonFix,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonMessage<'a> {
    rule_id: Option<&'a str>,
    severity: u8,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    fatal: bool,
    message: &'a str,
    line: u32,
    column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_id: Option<&'a str>,
    end_line: u32,
    end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<JsonFix>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    suggestions: Vec<JsonSuggestion<'a>>,
}

/// One file of ESLint's JSON format.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonFile<'a> {
    file_path: &'a str,
    messages: Vec<JsonMessage<'a>>,
    suppressed_messages: [(); 0],
    error_count: usize,
    fatal_error_count: usize,
    warning_count: usize,
    fixable_error_count: usize,
    fixable_warning_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<&'a str>,
    used_deprecated_rules: [(); 0],
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

/// The fixes of one diagnostic merged into a single fix, like ESLint does: from the first fix's start to the last
/// fix's end, keeping the text between them.
fn merged_fix(offsets: &Utf16Offsets, text: &str, fixes: &[RuleFix]) -> Option<JsonFix> {
    let mut sorted: Vec<&RuleFix> = fixes.iter().collect();
    sorted.sort_by_key(|f| (f.pos, f.end));
    let (first, last) = (sorted.first()?, sorted.last()?);
    let mut merged = String::new();
    let mut last_end = first.pos;
    for f in &sorted {
        if f.pos < last_end {
            return None;
        }
        merged.push_str(text.get(usize::try_from(last_end).ok()?..usize::try_from(f.pos).ok()?)?);
        merged.push_str(&f.text);
        last_end = f.end;
    }
    Some(JsonFix {
        range: [offsets.get(first.pos), offsets.get(last.end)],
        text: merged,
    })
}

fn json_message<'a>(offsets: &Utf16Offsets, text: &str, d: &'a Diagnostic) -> JsonMessage<'a> {
    JsonMessage {
        rule_id: (!d.rule.is_empty()).then_some(d.rule),
        severity: match d.severity {
            Severity::Error => 2,
            Severity::Warn => 1,
            Severity::Off => 0,
        },
        fatal: d.fatal,
        message: &d.message,
        line: d.line,
        column: d.column,
        message_id: (!d.message_id.is_empty()).then_some(d.message_id),
        end_line: d.end_line,
        end_column: d.end_column,
        fix: merged_fix(offsets, text, &d.fixes),
        suggestions: d
            .suggestions
            .iter()
            .filter_map(|s| {
                Some(JsonSuggestion {
                    message_id: s.message.id,
                    desc: &s.message.description,
                    fix: merged_fix(offsets, text, &s.fixes)?,
                })
            })
            .collect(),
    }
}

fn json(files: &[FileResult]) -> String {
    let files: Vec<JsonFile> = files
        .iter()
        .map(|f| {
            let text = f.text();
            let offsets = Utf16Offsets::new(text);
            let counts = count(std::slice::from_ref(f));
            JsonFile {
                file_path: &f.path,
                messages: f
                    .diagnostics
                    .iter()
                    .map(|d| json_message(&offsets, text, d))
                    .collect(),
                suppressed_messages: [],
                error_count: counts.errors,
                fatal_error_count: f.diagnostics.iter().filter(|d| d.fatal).count(),
                warning_count: counts.warnings,
                fixable_error_count: counts.fixable_errors,
                fixable_warning_count: counts.fixable_warnings,
                output: f.fixed_text.as_deref(),
                used_deprecated_rules: [],
            }
        })
        .collect();
    serde_json::to_string(&files).unwrap_or_else(|_| "[]".to_string())
}
