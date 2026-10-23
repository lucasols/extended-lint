use std::sync::Arc;
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;
use tsrs_ast::SourceFile;
use tsrs_compiler::{new_compiler_host, new_program, Program, ProgramOptions};
use tsrs_core::tspath::{self, Path};
use tsrs_core::{Tristate, P};
use tsrs_vfs::{bundled, cachedvfs, osvfs, FS};
use tsrslint::linter::{lint_files, Options};
use tsrslint::rule::{ReportedDiagnostic, Rule, RuleFix, RuleSuggestion};
use tsrslint::tsconfig::{ExtendedConfigCache, ParseHost, TsConfigResolver};

use crate::config::{Config, EnabledRule, Severity};
use crate::suppressions::Suppressions;

pub struct LintOptions {
    /// Absolute, normalized directory that config globs and printed paths are relative to.
    pub root: String,
    /// Absolute tsconfig paths.
    pub projects: Vec<String>,
    /// Absolute, normalized files or directories to lint; empty means every file of the projects.
    pub paths: Vec<String>,
    pub fix: bool,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub rule: &'static str,
    pub severity: Severity,
    pub message_id: &'static str,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub fixes: Vec<RuleFix>,
    pub suggestions: Vec<RuleSuggestion>,
}

pub struct FileResult {
    /// Absolute, normalized path.
    pub path: String,
    pub diagnostics: Vec<Diagnostic>,
    /// The fixed text, when `fix` is on and fixes were applied.
    pub fixed_text: Option<String>,
}

#[derive(Default)]
pub struct Timings {
    pub program: Duration,
    pub lint: Duration,
}

pub struct LintOutput {
    pub files: Vec<FileResult>,
    /// tsconfig and program diagnostics, as printable text.
    pub warnings: Vec<String>,
    pub linted_files: usize,
    pub timings: Timings,
}

/// The run's shared FS: one cache for config parsing and every program.
fn base_fs() -> Arc<dyn FS> {
    Arc::new(bundled::wrap_fs(cachedvfs::from(Arc::new(osvfs::fs()))))
}

struct ProgramExtCache(&'static ExtendedConfigCache);

impl tsrs_tsoptions::ExtendedConfigCache for ProgramExtCache {
    fn get_extended_config(
        &self,
        file_name: &str,
        path: &Path,
        resolution_stack: &[Path],
        host: &'static dyn tsrs_tsoptions::ParseConfigHost,
    ) -> P<tsrs_tsoptions::ExtendedConfigCacheEntry> {
        self.0
            .get_extended_config(file_name, path, resolution_stack, host)
    }
}

fn diagnostic_text(d: P<tsrs_ast::Diagnostic>) -> String {
    let message = tsrs_compiler::diagnosticwriter::flatten_diagnostic_message(d, "\n");
    match d.file() {
        Some(file) => format!("{}: TS{} {message}", file.file_name(), d.code()),
        None => format!("TS{} {message}", d.code()),
    }
}

pub fn run(config: &Config, opts: &LintOptions) -> Result<LintOutput, String> {
    let fs = base_fs();
    let host: &'static ParseHost = Box::leak(Box::new(ParseHost {
        fs: Arc::clone(&fs),
        cwd: opts.root.clone(),
    }));
    let ext_cache: &'static ExtendedConfigCache = Box::leak(Box::default());
    let resolver = TsConfigResolver::new(host, ext_cache);
    let mut output = LintOutput {
        files: Vec::new(),
        warnings: Vec::new(),
        linted_files: 0,
        timings: Timings::default(),
    };

    for tsconfig in &opts.projects {
        let start = Instant::now();
        let parsed = resolver.load(tsconfig);
        let Some(parsed_config) = parsed.config else {
            let errors: Vec<String> = parsed.errors.iter().map(|&d| diagnostic_text(d)).collect();
            return Err(format!("failed to load {tsconfig}:\n{}", errors.join("\n")));
        };
        output
            .warnings
            .extend(parsed.errors.iter().map(|&d| diagnostic_text(d)));
        output
            .warnings
            .extend(parsed_config.errors.iter().map(|&d| diagnostic_text(d)));

        let dir = tspath::get_directory_path(tsconfig);
        let compiler_host = new_compiler_host(
            &dir,
            Arc::clone(&fs),
            &bundled::lib_path(),
            Some(Arc::new(ProgramExtCache(ext_cache))),
            None,
            None,
        );
        let mut popts = ProgramOptions::new(parsed_config, compiler_host);
        popts.use_source_of_project_reference = true;
        popts.single_threaded = Tristate::False;
        let program = new_program(popts);
        output.warnings.extend(
            program
                .get_program_diagnostics()
                .iter()
                .map(|&d| diagnostic_text(d)),
        );
        program.bind_source_files();
        output.timings.program += start.elapsed();

        let start = Instant::now();
        let files = select_files(program, parsed_config.file_names_by_path(), config, opts);
        output.linted_files += files.iter().map(|(_, group)| group.len()).sum::<usize>();
        for (rules, group) in &files {
            output
                .files
                .extend(lint_group(program, group, rules, opts.fix));
        }
        output.timings.lint += start.elapsed();
    }

    output.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(output)
}

fn relative_to(root: &str, path: &str) -> String {
    let prefix = format!("{}/", root.trim_end_matches('/'));
    path.strip_prefix(&prefix).unwrap_or(path).to_string()
}

fn is_selected(file_name: &str, paths: &[String]) -> bool {
    paths.is_empty()
        || paths.iter().any(|p| {
            file_name == p
                || file_name
                    .strip_prefix(p.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
        })
}

/// The project's own files to lint, grouped by the rules enabled for them.
fn select_files(
    program: &'static Program,
    root_files: &FxHashMap<Path, String>,
    config: &Config,
    opts: &LintOptions,
) -> Vec<(Vec<EnabledRule>, Vec<P<SourceFile>>)> {
    let mut groups: FxHashMap<Vec<EnabledRule>, Vec<P<SourceFile>>> = FxHashMap::default();
    for &file in program.source_files() {
        if file.is_declaration_file() || !root_files.contains_key(file.path()) {
            continue;
        }
        let file_name = file.file_name();
        if file_name.contains("/node_modules/") || !is_selected(file_name, &opts.paths) {
            continue;
        }
        let relative = relative_to(&opts.root, file_name);
        if config.is_ignored(&relative) {
            continue;
        }
        let rules = config.rules_for(&relative);
        if rules.is_empty() {
            continue;
        }
        groups.entry(rules).or_default().push(file);
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_by(|a, b| a.0.cmp(&b.0));
    groups
}

fn create_rules(rules: &[EnabledRule]) -> Vec<&'static dyn Rule> {
    rules
        .iter()
        .filter_map(|r| {
            let options: Option<serde_json::Value> = r
                .options
                .as_deref()
                .and_then(|o| serde_json::from_str(o).ok());
            crate::rules::create_rule(r.name, options.as_ref())
                .ok()
                .map(|rule| -> &'static dyn Rule { Box::leak(rule) })
        })
        .collect()
}

fn lint_group(
    program: &'static Program,
    files: &[P<SourceFile>],
    rules: &[EnabledRule],
    fix: bool,
) -> Vec<FileResult> {
    let rule_instances = create_rules(rules);
    let severities: FxHashMap<&str, Severity> =
        rules.iter().map(|r| (r.name, r.severity)).collect();
    let opts = Options {
        fix: true,
        fix_suggestions: true,
        debug_timings: false,
    };
    let reported = lint_files(program, files, &rule_instances, &opts);

    let mut by_file: FxHashMap<String, (P<SourceFile>, Vec<ReportedDiagnostic>)> =
        FxHashMap::default();
    for d in reported {
        by_file
            .entry(d.file.file_name().to_string())
            .or_insert_with(|| (d.file, Vec::new()))
            .1
            .push(d);
    }

    let mut results = Vec::with_capacity(by_file.len());
    for (_, (file, mut reported)) in by_file {
        reported.sort_by_key(|d| (d.pos, d.end));
        let text = file.text();
        let suppressions = Suppressions::parse(text);
        let mut diagnostics: Vec<Diagnostic> = reported
            .into_iter()
            .filter_map(|d| {
                let (line, column) = line_col(file, d.pos);
                if suppressions.is_suppressed(d.rule_name, d.pos as usize, line as usize) {
                    return None;
                }
                let (end_line, end_column) = line_col(file, d.end);
                Some(Diagnostic {
                    rule: d.rule_name,
                    severity: severities
                        .get(d.rule_name)
                        .copied()
                        .unwrap_or(Severity::Error),
                    message_id: d.message.id,
                    message: d.message.description,
                    line,
                    column,
                    end_line,
                    end_column,
                    fixes: d.fixes.unwrap_or_default(),
                    suggestions: d.suggestions.unwrap_or_default(),
                })
            })
            .collect();
        if diagnostics.is_empty() {
            continue;
        }
        let fixed_text = if fix {
            apply_fixes(text, &mut diagnostics)
        } else {
            None
        };
        results.push(FileResult {
            path: file.file_name().to_string(),
            diagnostics,
            fixed_text,
        });
    }
    results
}

/// Applies the non-overlapping autofixes in position order and drops the diagnostics they fixed. Returns the new
/// text when anything changed.
fn apply_fixes(text: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    let mut order: Vec<usize> = (0..diagnostics.len())
        .filter(|&i| diagnostics.get(i).is_some_and(|d| !d.fixes.is_empty()))
        .collect();
    if order.is_empty() {
        return None;
    }
    for d in diagnostics.iter_mut() {
        d.fixes.sort_by_key(|f| (f.pos, f.end));
    }
    let span = |d: &Diagnostic| {
        (
            d.fixes.first().map_or(0, |f| f.pos),
            d.fixes.last().map_or(0, |f| f.end),
        )
    };
    order.sort_by_key(|&i| diagnostics.get(i).map(span));

    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;
    let mut applied = vec![false; diagnostics.len()];
    for i in order {
        let Some(d) = diagnostics.get(i) else {
            continue;
        };
        let (start, _) = span(d);
        if (start as usize) < last {
            continue;
        }
        for f in &d.fixes {
            out.push_str(text.get(last..f.pos as usize).unwrap_or_default());
            out.push_str(&f.text);
            last = f.end as usize;
        }
        if let Some(a) = applied.get_mut(i) {
            *a = true;
        }
    }
    out.push_str(text.get(last..).unwrap_or_default());

    let mut index = 0;
    diagnostics.retain(|_| {
        let keep = !applied.get(index).copied().unwrap_or(false);
        index += 1;
        keep
    });
    Some(out)
}

fn line_col(file: P<SourceFile>, pos: i32) -> (u32, u32) {
    let (line, col) = tsrs_scanner::get_ecma_line_and_utf16_character_of_position(
        &*file,
        u32::try_from(pos).unwrap_or(0),
    );
    (u32::try_from(line + 1).unwrap_or(u32::MAX), col + 1)
}
