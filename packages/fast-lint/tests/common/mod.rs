#![expect(
    dead_code,
    reason = "each test binary uses a different subset of the harness"
)]

use std::cell::RefCell;
use std::fmt::Write as _;
use std::sync::Arc;

use rustc_hash::FxHashMap;
use tsrs_ast::{SourceFile, SourceFileParseOptions};
use tsrs_compiler::{new_compiler_host, new_program, CompilerHost, ProgramOptions};
use tsrs_core::tspath::{self, Path};
use tsrs_core::{CompilerOptions, ScriptKind, Tristate, P};
use tsrs_tsoptions::{self as tsoptions, ParsedCommandLine};
use tsrs_vfs::{bundled, cachedvfs, osvfs, FS};
use tsrslint::linter::{apply_rule_fixes, lint_files, Options};
use tsrslint::overlayfs::OverlayFS;
use tsrslint::rule::{ReportedDiagnostic, Rule, RuleFix};
use tsrslint::tsconfig::ParseHost;

const MAX_FIX_PASSES: usize = 10;

#[derive(Default)]
pub struct Case<'a> {
    pub code: &'a str,
    pub options: Option<serde_json::Value>,
    pub filename: Option<&'a str>,
}

impl<'a> From<&'a str> for Case<'a> {
    fn from(code: &'a str) -> Case<'a> {
        Case {
            code,
            ..Default::default()
        }
    }
}

impl<'a> From<&'a String> for Case<'a> {
    fn from(code: &'a String) -> Case<'a> {
        Case {
            code,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message_id: &'static str,
    pub message: String,
    pub fixes: Vec<RuleFix>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message_id: &'static str,
    pub message: String,
    pub line: i32,
    pub column: i32,
    pub end_line: i32,
    pub end_column: i32,
    pub fixes: Vec<RuleFix>,
    pub suggestions: Vec<Suggestion>,
}

pub struct LintResult {
    pub code: String,
    pub diagnostics: Vec<Diagnostic>,
    /// The code after applying every autofix, re-linting up to ten passes like ESLint's `verifyAndFix`.
    pub output: String,
}

impl LintResult {
    /// One line per diagnostic: `- { messageId: 'x', line: 1 }`.
    pub fn errors(&self) -> String {
        let mut out = String::new();
        for d in &self.diagnostics {
            let _ = writeln!(
                out,
                "- {{ messageId: '{}', line: {} }}",
                d.message_id, d.line
            );
        }
        out
    }

    /// Like `errors` with the column range.
    pub fn errors_with_columns(&self) -> String {
        let mut out = String::new();
        for d in &self.diagnostics {
            let _ = writeln!(
                out,
                "- {{ messageId: '{}', line: {}, column: {}, endLine: {}, endColumn: {} }}",
                d.message_id, d.line, d.column, d.end_line, d.end_column
            );
        }
        out
    }

    /// Each diagnostic with its full message.
    pub fn errors_with_msg(&self) -> String {
        let mut out = String::new();
        for d in &self.diagnostics {
            let _ = writeln!(out, "- messageId: '{}'", d.message_id);
            let _ = writeln!(out, "  msg: '{}'", d.message);
            let _ = writeln!(out, "  line: {}", d.line);
        }
        out
    }

    /// The code after applying suggestion `index` of the first diagnostic.
    pub fn suggestion_output(&self, index: usize) -> String {
        let diagnostic = self.diagnostics.first().expect("no diagnostics");
        self.suggestion_output_of(diagnostic, index)
    }

    pub fn suggestion_output_of(&self, diagnostic: &Diagnostic, index: usize) -> String {
        let suggestion = diagnostic
            .suggestions
            .get(index)
            .unwrap_or_else(|| panic!("no suggestion {index}"));
        apply_rule_fixes(&self.code, vec![suggestion.fixes.clone()]).0
    }
}

/// Asserts `case` produces no diagnostics.
pub fn valid<'a>(rule: &str, case: impl Into<Case<'a>>) {
    let case = case.into();
    let result = lint(rule, &case);
    assert!(
        result.diagnostics.is_empty(),
        "expected no errors, got:{}\ncode:\n{}",
        result.errors_with_msg(),
        case.code
    );
}

/// Lints `case` and asserts it produces at least one diagnostic.
pub fn invalid<'a>(rule: &str, case: impl Into<Case<'a>>) -> LintResult {
    let case = case.into();
    let result = lint(rule, &case);
    assert!(
        !result.diagnostics.is_empty(),
        "expected errors, got none\ncode:\n{}",
        case.code
    );
    result
}

pub fn lint(rule_name: &str, case: &Case) -> LintResult {
    let rule: &'static dyn Rule = Box::leak(
        fast_lint::rules::create_rule(rule_name, case.options.as_ref())
            .unwrap_or_else(|e| panic!("{e}")),
    );
    lint_rule(rule, case)
}

/// Lints `case` with an already created rule, e.g. a test-only rule exercising a helper.
pub fn lint_rule(rule: &'static dyn Rule, case: &Case) -> LintResult {
    let file_name = case.filename.unwrap_or_else(|| {
        if case.code.contains("</") || case.code.contains("/>") {
            "react.tsx"
        } else {
            "file.ts"
        }
    });
    let (diagnostics, file) = run(rule, case.code, file_name);
    let diagnostics: Vec<Diagnostic> = diagnostics.iter().map(|d| to_diagnostic(file, d)).collect();

    let mut output = case.code.to_string();
    let mut fix_sets: Vec<Vec<RuleFix>> = diagnostics
        .iter()
        .filter(|d| !d.fixes.is_empty())
        .map(|d| d.fixes.clone())
        .collect();
    for _ in 0..MAX_FIX_PASSES {
        if fix_sets.is_empty() {
            break;
        }
        let (fixed, changed) = apply_rule_fixes(&output, fix_sets);
        if !changed {
            break;
        }
        output = fixed;
        let (next, _) = run(rule, &output, file_name);
        fix_sets = next
            .into_iter()
            .filter_map(|d| d.fixes.filter(|f| !f.is_empty()))
            .collect();
    }

    LintResult {
        code: case.code.to_string(),
        diagnostics,
        output,
    }
}

fn to_diagnostic(file: P<SourceFile>, d: &ReportedDiagnostic) -> Diagnostic {
    let (line, column) = line_col(file, d.pos);
    let (end_line, end_column) = line_col(file, d.end);
    Diagnostic {
        message_id: d.message.id,
        message: d.message.description.clone(),
        line,
        column,
        end_line,
        end_column,
        fixes: d.fixes.clone().unwrap_or_default(),
        suggestions: d
            .suggestions
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|s| Suggestion {
                message_id: s.message.id,
                message: s.message.description.clone(),
                fixes: s.fixes.clone(),
            })
            .collect(),
    }
}

fn line_col(file: P<SourceFile>, pos: i32) -> (i32, i32) {
    let (line, col) =
        tsrs_scanner::get_ecma_line_and_utf16_character_of_position(&*file, pos as u32);
    (line as i32 + 1, col as i32 + 1)
}

/// File name, the two external-module-indicator flags and the text identify a parse.
type SourceFileKey = (String, bool, bool, String);

thread_local! {
    static SOURCE_FILE_CACHE: RefCell<FxHashMap<SourceFileKey, P<SourceFile>>> =
        RefCell::new(FxHashMap::default());
}

/// Parses each lib file once per thread instead of once per test program.
struct CachedCompilerHost {
    inner: Arc<dyn CompilerHost>,
}

impl CompilerHost for CachedCompilerHost {
    fn fs(&self) -> &dyn FS {
        self.inner.fs()
    }
    fn default_library_path(&self) -> &str {
        self.inner.default_library_path()
    }
    fn get_current_directory(&self) -> &str {
        self.inner.get_current_directory()
    }
    fn trace(&self, msg: &'static tsrs_diagnostics::Message, args: &[&dyn std::fmt::Display]) {
        self.inner.trace(msg, args)
    }
    fn get_source_file(&self, opts: SourceFileParseOptions) -> Option<P<SourceFile>> {
        let text = self.fs().read_file(&opts.file_name)?;
        let script_kind = tsrs_core::get_script_kind_from_file_name(&opts.file_name);
        if script_kind == ScriptKind::Unknown {
            return None;
        }
        let emi = opts.external_module_indicator_options;
        let key = (opts.file_name.clone(), emi.jsx, emi.force, text.clone());
        if let Some(cached) = SOURCE_FILE_CACHE.with(|c| c.borrow().get(&key).copied()) {
            return Some(cached);
        }
        let sf = tsrs_parser::parse_source_file(opts, &text, script_kind);
        SOURCE_FILE_CACHE.with(|c| c.borrow_mut().insert(key, sf));
        Some(sf)
    }
    fn get_content_mapped_source_files(
        &self,
        opts: SourceFileParseOptions,
        mapper: &tsrs_contentmapper::Mapper,
    ) -> Result<tsrs_contentmapper::SourceFiles, tsrs_contentmapper::Error> {
        self.inner.get_content_mapped_source_files(opts, mapper)
    }
    fn content_mapper_project(&self) -> Option<Arc<dyn tsrs_contentmapper::Project>> {
        self.inner.content_mapper_project()
    }
    fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: Path,
    ) -> Option<P<ParsedCommandLine>> {
        self.inner.get_resolved_project_reference(file_name, path)
    }
}

fn fixture_root() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture");
    let dir = std::fs::canonicalize(dir).expect("tests/fixture exists");
    tspath::normalize_path(&dir.to_string_lossy())
}

fn base_fs() -> Arc<dyn FS> {
    thread_local! {
        static BASE: Arc<dyn FS> =
            Arc::new(cachedvfs::from(Arc::new(bundled::wrap_fs(osvfs::fs()))));
    }
    BASE.with(Arc::clone)
}

fn run(
    rule: &'static dyn Rule,
    code: &str,
    file_name: &str,
) -> (Vec<ReportedDiagnostic>, P<SourceFile>) {
    let root = fixture_root();
    let resolved = tspath::resolve_path(&root, &[file_name]);
    let mut virtual_files: FxHashMap<String, String> = FxHashMap::default();
    virtual_files.insert(resolved.clone(), code.to_string());
    let fs: Arc<dyn FS> = Arc::new(OverlayFS::new(base_fs(), virtual_files));
    let host: &'static ParseHost = Box::leak(Box::new(ParseHost {
        fs: Arc::clone(&fs),
        cwd: root.clone(),
    }));
    let tsconfig = tspath::resolve_path(&root, &["tsconfig.json"]);
    let (parsed, errors) = tsoptions::get_parsed_command_line_of_config_file(
        &tsconfig,
        Some(&CompilerOptions::default()),
        None,
        host,
        None,
    );
    assert!(errors.is_empty(), "tsconfig errors: {}", errors.len());
    let config = P::new(parsed.expect("parsed tsconfig"));
    let inner = new_compiler_host(&root, fs, &bundled::lib_path(), None, None, None);
    let compiler_host: Arc<dyn CompilerHost> = Arc::new(CachedCompilerHost { inner });
    let mut popts = ProgramOptions::new(config, compiler_host);
    popts.single_threaded = Tristate::True;
    let program = new_program(popts);
    program.bind_source_files();
    let path = tspath::to_path(&resolved, &root, host.fs.use_case_sensitive_file_names());
    let file = program
        .source_files()
        .iter()
        .copied()
        .find(|sf| *sf.path() == path)
        .unwrap_or_else(|| panic!("{resolved} not in program"));
    let opts = Options {
        fix: true,
        fix_suggestions: true,
        debug_timings: false,
    };
    (lint_files(program, &[file], &[rule], &opts), file)
}
