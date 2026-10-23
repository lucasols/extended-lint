use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::SourceFile;
use tsrs_compiler::{new_compiler_host, new_program, Program, ProgramOptions};
use tsrs_core::tspath::{self, Path};
use tsrs_core::{Tristate, P};
use tsrs_vfs::{bundled, cachedvfs, osvfs, FS};
use tsrslint::linter::{lint_files, Options};
use tsrslint::rule::{ReportedDiagnostic, Rule, RuleFix, RuleSuggestion};
use tsrslint::tsconfig::{ExtendedConfigCache, ParseHost, TsConfigResolver};

use crate::config::{Config, EnabledRule, Severity};
use crate::suppressions::{line_starts, Suppressions};
use crate::utils::comment_at;

pub struct LintOptions {
    /// Absolute, normalized directory that config globs and printed paths are relative to.
    pub root: String,
    /// Absolute tsconfig paths.
    pub projects: Vec<String>,
    /// Absolute, normalized files or directories to lint, without a trailing `/`; empty means every file of the
    /// projects.
    pub paths: Vec<String>,
    pub fix: bool,
    /// Run only the rules set to `error` (`--quiet` without `--max-warnings`, like ESLint).
    pub errors_only: bool,
    /// Apply only the fixes of `error` diagnostics (`--quiet`).
    pub fix_errors_only: bool,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Empty for a parsing error.
    pub rule: &'static str,
    /// A parsing error: the file's rules didn't run.
    pub fatal: bool,
    /// Byte offsets of the reported range.
    pub pos: u32,
    pub end: u32,
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
    /// The original text.
    pub source: &'static str,
    /// The fixed text, when `fix` is on and fixes were applied.
    pub fixed_text: Option<String>,
}

impl FileResult {
    /// The text that diagnostic positions and fix ranges refer to: the fixed text when fixes were applied.
    pub fn text(&self) -> &str {
        self.fixed_text.as_deref().unwrap_or(self.source)
    }
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

/// tsconfig diagnostics that mean the project can't be loaded as written: a `files` entry or an `extends` target that
/// doesn't exist.
const FATAL_CONFIG_CODES: &[i32] = &[5083, 6053];

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
    let mut selection = Selection {
        seen: FxHashSet::default(),
        paths: vec![PathMatch::None; opts.paths.len()],
        selected: 0,
    };

    let mut queue: Vec<String> = opts.projects.iter().rev().cloned().collect();
    let mut loaded: FxHashSet<String> = FxHashSet::default();
    while let Some(tsconfig) = queue.pop() {
        if !loaded.insert(tsconfig.clone()) {
            continue;
        }
        let start = Instant::now();
        let parsed = resolver.load(&tsconfig);
        let Some(parsed_config) = parsed.config else {
            let errors: Vec<String> = parsed.errors.iter().map(|&d| diagnostic_text(d)).collect();
            return Err(format!("failed to load {tsconfig}:\n{}", errors.join("\n")));
        };
        let config_errors: Vec<P<tsrs_ast::Diagnostic>> = parsed
            .errors
            .iter()
            .chain(parsed_config.errors.iter())
            .copied()
            .collect();
        if config_errors
            .iter()
            .any(|d| FATAL_CONFIG_CODES.contains(&d.code()))
        {
            let errors: Vec<String> = config_errors.iter().map(|&d| diagnostic_text(d)).collect();
            return Err(format!("failed to load {tsconfig}:\n{}", errors.join("\n")));
        }
        output
            .warnings
            .extend(config_errors.iter().map(|&d| diagnostic_text(d)));

        if parsed_config.file_names().is_empty()
            && !parsed_config.resolved_project_reference_paths().is_empty()
        {
            queue.extend(
                parsed_config
                    .resolved_project_reference_paths()
                    .iter()
                    .rev()
                    .cloned(),
            );
            continue;
        }

        let selected = select_files(
            parsed_config.file_names_by_path(),
            config,
            opts,
            &mut selection,
        );
        if selected.is_empty() {
            output.timings.program += start.elapsed();
            continue;
        }

        let dir = tspath::get_directory_path(&tsconfig);
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
        let program_errors = program.get_program_diagnostics();
        if program_errors
            .iter()
            .any(|d| FATAL_CONFIG_CODES.contains(&d.code()))
        {
            let errors: Vec<String> = program_errors.iter().map(|&d| diagnostic_text(d)).collect();
            return Err(format!("failed to load {tsconfig}:\n{}", errors.join("\n")));
        }
        output
            .warnings
            .extend(program_errors.iter().map(|&d| diagnostic_text(d)));
        program.bind_source_files();
        output.timings.program += start.elapsed();

        let start = Instant::now();
        let mut groups: FxHashMap<&[EnabledRule], Vec<P<SourceFile>>> = FxHashMap::default();
        for &file in program.source_files() {
            let Some(rules) = selected.get(file.path()) else {
                continue;
            };
            output.linted_files += 1;
            if !file.diagnostics().is_empty() {
                output.files.push(parsing_error(file));
                continue;
            }
            if !rules.is_empty() {
                groups.entry(rules.as_slice()).or_default().push(file);
            }
        }
        let mut groups: Vec<_> = groups.into_iter().collect();
        groups.sort_by(|a, b| a.0.cmp(b.0));
        for (rules, group) in &groups {
            output.files.extend(lint_group(program, group, rules, opts));
        }
        output.timings.lint += start.elapsed();
    }

    let mut unmatched = Vec::new();
    for (path, state) in opts.paths.iter().zip(&selection.paths) {
        match state {
            PathMatch::None => unmatched.push(display_path(&opts.root, path)),
            PathMatch::Ignored if std::path::Path::new(path).is_file() => {
                output.warnings.push(format!(
                    "{}: file ignored: it matches `ignores`, is a declaration file or is in node_modules",
                    display_path(&opts.root, path)
                ));
            }
            PathMatch::Ignored => {
                return Err(format!(
                    "all files matched by {} are ignored",
                    display_path(&opts.root, path)
                ));
            }
            PathMatch::Selected => {}
        }
    }
    if !unmatched.is_empty() {
        return Err(format!("no project files match {}", unmatched.join(", ")));
    }
    if selection.selected == 0 && opts.paths.is_empty() {
        return Err(format!(
            "no files to lint: every file of {} is ignored or the projects are empty",
            opts.projects
                .iter()
                .map(|p| display_path(&opts.root, p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    output.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(output)
}

/// `path` relative to `root`, with `..` segments when it is outside, for config globs.
pub fn relative_path<'a>(root: &str, path: &'a str) -> Cow<'a, str> {
    if let Some(rest) = path
        .strip_prefix(root.trim_end_matches('/'))
        .and_then(|rest| rest.strip_prefix('/'))
    {
        return Cow::Borrowed(rest);
    }
    let options = tspath::ComparePathsOptions {
        use_case_sensitive_file_names: true,
        current_directory: root.to_string(),
    };
    if tspath::get_root_length(root) == 0 || tspath::get_root_length(path) == 0 {
        return Cow::Borrowed(path);
    }
    Cow::Owned(tspath::get_relative_path_from_directory(
        root, path, &options,
    ))
}

/// `path` relative to `root` when it is inside, else as is: for messages.
fn display_path<'a>(root: &str, path: &'a str) -> &'a str {
    path.strip_prefix(root.trim_end_matches('/'))
        .and_then(|rest| rest.strip_prefix('/'))
        .unwrap_or(path)
}

/// Whether `file_name` is `path` or in it. `path` has no trailing `/` unless it is a root (`/`, `c:/`).
fn is_under(file_name: &str, path: &str) -> bool {
    file_name
        .strip_prefix(path)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/') || path.ends_with('/'))
}

/// How a path argument matched the project files.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PathMatch {
    None,
    /// Only files that aren't linted: ignored, declaration or node_modules files.
    Ignored,
    Selected,
}

/// File selection state shared by every project of a run.
struct Selection {
    /// Files already selected by an earlier project, linted only once.
    seen: FxHashSet<String>,
    /// How each path argument matched.
    paths: Vec<PathMatch>,
    /// Files selected and not ignored, including those with every rule off.
    selected: usize,
}

/// The project's root files to lint, with the rules enabled for each.
fn select_files(
    root_files: &FxHashMap<Path, String>,
    config: &Config,
    opts: &LintOptions,
    selection: &mut Selection,
) -> FxHashMap<Path, Vec<EnabledRule>> {
    let mut selected = FxHashMap::default();
    for (path, file_name) in root_files {
        if !opts.paths.is_empty() && !opts.paths.iter().any(|p| is_under(file_name, p)) {
            continue;
        }
        let relative = relative_path(&opts.root, file_name);
        let skipped = tspath::is_declaration_file_name(file_name)
            || file_name.contains("/node_modules/")
            || config.is_ignored(&relative);
        for (p, state) in opts.paths.iter().zip(selection.paths.iter_mut()) {
            if is_under(file_name, p) {
                if !skipped {
                    *state = PathMatch::Selected;
                } else if *state == PathMatch::None {
                    *state = PathMatch::Ignored;
                }
            }
        }
        if skipped || !selection.seen.insert(file_name.clone()) {
            continue;
        }
        selection.selected += 1;
        let mut rules = config.rules_for(&relative);
        if opts.errors_only {
            rules.retain(|r| r.severity == Severity::Error);
        }
        selected.insert(path.clone(), rules);
    }
    selected
}

/// The result of a file with a syntax error: like ESLint, a fatal "Parsing error" and no rule diagnostics.
fn parsing_error(file: P<SourceFile>) -> FileResult {
    let diagnostics = file
        .diagnostics()
        .first()
        .map(|&d| {
            let pos = d.pos().unwrap_or(0);
            let (line, column) = line_col(file, i32::try_from(pos).unwrap_or(0));
            Diagnostic {
                rule: "",
                fatal: true,
                pos,
                end: pos,
                severity: Severity::Error,
                message_id: "",
                message: format!(
                    "Parsing error: {}",
                    tsrs_compiler::diagnosticwriter::flatten_diagnostic_message(d, "\n")
                ),
                line,
                column,
                end_line: line,
                end_column: column,
                fixes: Vec::new(),
                suggestions: Vec::new(),
            }
        })
        .into_iter()
        .collect();
    FileResult {
        path: file.file_name().to_string(),
        diagnostics,
        source: file.text(),
        fixed_text: None,
    }
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
    lint_opts: &LintOptions,
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
        let suppressions = Suppressions::parse(text, |pos| {
            let (start, end, is_block) = comment_at(file, u32::try_from(pos).ok()?)?;
            Some((start as usize, end as usize, is_block))
        });
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
                    fatal: false,
                    pos: u32::try_from(d.pos).unwrap_or(0),
                    end: u32::try_from(d.end).unwrap_or(0),
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
        let fixed_text = if lint_opts.fix {
            apply_fixes(text, &mut diagnostics, lint_opts.fix_errors_only)
        } else {
            None
        };
        results.push(FileResult {
            path: file.file_name().to_string(),
            diagnostics,
            source: text,
            fixed_text,
        });
    }
    results
}

/// A replacement applied to the text: the original byte range and the length of its new text.
#[derive(Clone, Copy)]
struct Edit {
    pos: u32,
    end: u32,
    len: u32,
}

/// Maps an offset of the original text to the fixed text. `None` when it falls strictly inside a replaced range.
fn map_offset(edits: &[Edit], offset: u32) -> Option<u32> {
    let mut delta: i64 = 0;
    for edit in edits {
        if offset <= edit.pos {
            break;
        }
        if offset < edit.end {
            return None;
        }
        delta += i64::from(edit.len) - i64::from(edit.end - edit.pos);
    }
    u32::try_from(i64::from(offset) + delta).ok()
}

fn map_fixes(edits: &[Edit], fixes: &[RuleFix]) -> Option<Vec<RuleFix>> {
    fixes
        .iter()
        .map(|f| {
            let pos = map_offset(edits, u32::try_from(f.pos).ok()?)?;
            let end = map_offset(edits, u32::try_from(f.end).ok()?)?;
            Some(RuleFix {
                pos: i32::try_from(pos).ok()?,
                end: i32::try_from(end).ok()?,
                ..f.clone()
            })
        })
        .collect()
}

/// Applies the non-overlapping autofixes (only those of errors when `errors_only`) in position order and drops the
/// diagnostics they fixed. The remaining diagnostics are moved to their place in the fixed text; their fixes and
/// suggestions that overlapped an applied fix are dropped. Returns the new text when anything changed.
fn apply_fixes(text: &str, diagnostics: &mut Vec<Diagnostic>, errors_only: bool) -> Option<String> {
    let mut order: Vec<usize> = (0..diagnostics.len())
        .filter(|&i| {
            diagnostics.get(i).is_some_and(|d| {
                !d.fixes.is_empty() && (!errors_only || d.severity == Severity::Error)
            })
        })
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
    let mut edits: Vec<Edit> = Vec::new();
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
            edits.push(Edit {
                pos: u32::try_from(f.pos).unwrap_or(0),
                end: u32::try_from(f.end).unwrap_or(0),
                len: u32::try_from(f.text.len()).unwrap_or(u32::MAX),
            });
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

    let starts = line_starts(&out);
    for d in diagnostics.iter_mut() {
        let pos = map_offset(&edits, d.pos).unwrap_or_else(|| {
            edits
                .iter()
                .find(|e| e.pos < d.pos && d.pos < e.end)
                .map_or(0, |e| map_offset(&edits, e.pos).unwrap_or(0))
        });
        let end = map_offset(&edits, d.end).unwrap_or(pos).max(pos);
        d.pos = pos;
        d.end = end;
        (d.line, d.column) = text_line_col(&out, &starts, pos);
        (d.end_line, d.end_column) = text_line_col(&out, &starts, end);
        d.fixes = map_fixes(&edits, &d.fixes).unwrap_or_default();
        d.suggestions = std::mem::take(&mut d.suggestions)
            .into_iter()
            .filter_map(|s| {
                Some(RuleSuggestion {
                    fixes: map_fixes(&edits, &s.fixes)?,
                    ..s
                })
            })
            .collect();
    }
    Some(out)
}

/// 1-based line and UTF-16 column of byte offset `pos` in `text`.
fn text_line_col(text: &str, starts: &[usize], pos: u32) -> (u32, u32) {
    let pos = pos as usize;
    let line = starts.partition_point(|&s| s <= pos).max(1);
    let line_start = starts.get(line - 1).copied().unwrap_or(0);
    let column = text
        .get(line_start..pos)
        .map_or(0, |prefix| prefix.encode_utf16().count());
    (
        u32::try_from(line).unwrap_or(u32::MAX),
        u32::try_from(column + 1).unwrap_or(u32::MAX),
    )
}

fn line_col(file: P<SourceFile>, pos: i32) -> (u32, u32) {
    let (line, col) = tsrs_scanner::get_ecma_line_and_utf16_character_of_position(
        &*file,
        u32::try_from(pos).unwrap_or(0),
    );
    (u32::try_from(line + 1).unwrap_or(u32::MAX), col + 1)
}
