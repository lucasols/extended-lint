use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

use fast_lint::config::Config;
use fast_lint::linter::{run, LintOptions, LintOutput};
use fast_lint::report::{render, Format};
use insta::assert_snapshot;

/// A temporary project directory, removed on drop.
struct Project {
    root: PathBuf,
}

impl Project {
    fn new(name: &str, files: &[(&str, &str)]) -> Project {
        let base = std::fs::canonicalize(std::env::temp_dir()).expect("temp dir");
        let root = base.join(format!("fast-lint-cli-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (path, text) in files {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create dir");
            std::fs::write(path, text).expect("write file");
        }
        Project { root }
    }

    fn root(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }

    fn path(&self, relative: &str) -> String {
        format!("{}/{relative}", self.root())
    }

    fn options(&self, projects: &[&str], paths: &[&str], fix: bool) -> LintOptions {
        LintOptions {
            root: self.root(),
            projects: projects.iter().map(|p| self.path(p)).collect(),
            paths: paths.iter().map(|p| vec![self.path(p)]).collect(),
            fix,
            errors_only: false,
            fix_errors_only: false,
        }
    }

    fn lint(
        &self,
        config: &str,
        projects: &[&str],
        paths: &[&str],
        fix: bool,
    ) -> Result<LintOutput, String> {
        let config = Config::from_json(config)?;
        run(&config, &self.options(projects, paths, fix))
    }

    /// Runs the fast-lint binary in the project directory: exit code and stdout.
    fn cli(&self, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_fast-lint"))
            .args(args)
            .arg("--no-color")
            .current_dir(&self.root)
            .output()
            .expect("run fast-lint");
        (
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stdout).into_owned(),
        )
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// One line per diagnostic: `path:line:column rule`.
fn summary(project: &Project, output: &LintOutput) -> String {
    let mut out = String::new();
    for file in &output.files {
        let path = file
            .path
            .strip_prefix(&format!("{}/", project.root()))
            .unwrap_or(&file.path);
        for d in &file.diagnostics {
            let _ = writeln!(
                out,
                "{path}:{}:{} {} {}",
                d.line,
                d.column,
                d.rule,
                d.severity.label()
            );
        }
    }
    out
}

const TSCONFIG: &str = r#"{ "compilerOptions": { "strict": true }, "include": ["src"] }"#;
const ALL_RULES: &str = r#"{ "rules": {
    "improved-no-unnecessary-condition": "error",
    "no-leaked-conditional-rendering": "error",
    "no-unnecessary-void-on-promise": "error",
    "strict-casting": "error"
} }"#;

#[test]
fn solution_style_tsconfig_lints_referenced_projects() {
    let project = Project::new(
        "solution",
        &[
            (
                "tsconfig.json",
                r#"{ "files": [], "references": [{ "path": "./tsconfig.app.json" }, { "path": "./tsconfig.node.json" }] }"#,
            ),
            (
                "tsconfig.app.json",
                r#"{ "compilerOptions": { "strict": true, "composite": true }, "include": ["src"] }"#,
            ),
            (
                "tsconfig.node.json",
                r#"{ "compilerOptions": { "strict": true, "composite": true }, "include": ["vite.config.ts"] }"#,
            ),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
            (
                "vite.config.ts",
                "export const b = (n: number) => Number(n)\n",
            ),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .expect("lint");

    assert_eq!(output.linted_files, 2);
    assert_snapshot!(summary(&project, &output), @r###"
    src/a.ts:1:33 strict-casting error
    vite.config.ts:1:33 strict-casting error
    "###);
}

#[test]
fn files_in_several_projects_are_linted_once() {
    let project = Project::new(
        "overlap",
        &[
            ("tsconfig.json", TSCONFIG),
            ("tsconfig.build.json", TSCONFIG),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    let output = project
        .lint(
            ALL_RULES,
            &["tsconfig.json", "tsconfig.build.json"],
            &[],
            false,
        )
        .expect("lint");

    assert_eq!(output.linted_files, 1);
    assert_snapshot!(summary(&project, &output), @"src/a.ts:1:33 strict-casting error");
}

#[test]
fn path_matching_no_project_file_is_an_error() {
    let project = Project::new(
        "unmatched",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/a.ts", "export const a = 1\n"),
        ],
    );

    let error = project
        .lint(ALL_RULES, &["tsconfig.json"], &["src/typo"], false)
        .err()
        .expect("unmatched path should fail");

    assert_snapshot!(error, @"no project files match src/typo");
}

#[test]
fn every_file_ignored_is_an_error() {
    let project = Project::new(
        "ignored",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/a.ts", "export const a = 1\n"),
        ],
    );

    let error = project
        .lint(
            r#"{ "ignores": ["src/**"] }"#,
            &["tsconfig.json"],
            &[],
            false,
        )
        .err()
        .expect("no files should fail");

    assert_snapshot!(error.replace(&project.root(), "<root>"), @"no files to lint: every file of tsconfig.json is ignored or the projects are empty");
}

#[test]
fn missing_files_entry_is_an_error() {
    let project = Project::new(
        "missing-file",
        &[
            ("tsconfig.json", r#"{ "files": ["src/missing.ts"] }"#),
            ("src/a.ts", "export const a = 1\n"),
        ],
    );

    let error = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .err()
        .expect("missing file should fail");

    assert_snapshot!(error.replace(&project.root(), "<root>"), @r###"
    failed to load <root>/tsconfig.json:
    TS6053 File '<root>/src/missing.ts' not found.
      The file is in the program because:
        Part of 'files' list in tsconfig.json
    "###);
}

#[test]
fn missing_extends_is_an_error() {
    let project = Project::new(
        "missing-extends",
        &[
            (
                "tsconfig.json",
                r#"{ "extends": "./tsconfig.base.json", "include": ["src"] }"#,
            ),
            ("src/a.ts", "export const a = 1\n"),
        ],
    );

    let error = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .err()
        .expect("missing extends should fail");

    assert_snapshot!(error.replace(&project.root(), "<root>"), @r###"
    failed to load <root>/tsconfig.json:
    TS5083 Cannot read file '<root>/tsconfig.base.json'.
    "###);
}

#[test]
fn overrides_and_ignores_apply_by_path() {
    let project = Project::new(
        "overrides",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
            (
                "src/b.test.ts",
                "export const b = (s: string) => String(s)\n",
            ),
            (
                "src/gen/c.ts",
                "export const c = (s: string) => String(s)\n",
            ),
        ],
    );
    let config = r#"{
        "ignores": ["src/gen/**"],
        "rules": { "strict-casting": "warn" },
        "overrides": [{ "files": ["**/*.test.ts"], "rules": { "strict-casting": "error" } }]
    }"#;

    let output = project
        .lint(config, &["tsconfig.json"], &[], false)
        .expect("lint");

    assert_snapshot!(summary(&project, &output), @r###"
    src/a.ts:1:33 strict-casting warning
    src/b.test.ts:1:33 strict-casting error
    "###);
}

#[test]
fn disable_directives_inside_strings_are_ignored() {
    let project = Project::new(
        "directives",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/a.ts",
                "export const banner = '/* eslint-disable */'\nexport const a = (s: string) => String(s)\n// eslint-disable-next-line strict-casting\nexport const b = (s: string) => String(s)\n",
            ),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .expect("lint");

    assert_snapshot!(summary(&project, &output), @"src/a.ts:2:33 strict-casting error");
}

#[test]
fn fix_reports_remaining_problems_at_their_fixed_position() {
    let project = Project::new(
        "fix",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/a.ts",
                "declare const s: string\ndeclare const o: { a: string }\nexport const x = [String(s), Number(o)]\n",
            ),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], true)
        .expect("lint");

    let file = output.files.first().expect("a file result");
    assert_snapshot!(file.fixed_text.as_deref().unwrap_or_default(), @r###"
    declare const s: string
    declare const o: { a: string }
    export const x = [s, Number(o)]
    "###);
    assert_snapshot!(summary(&project, &output), @"src/a.ts:3:22 strict-casting error");
}

#[test]
fn json_fix_ranges_are_utf16_offsets() {
    let project = Project::new(
        "json",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/a.ts",
                "const emoji = '😀'\nexport const a = (s: string) => String(s)\n",
            ),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .expect("lint");

    let json = render(Format::Json, &project.root(), &output.files, false);
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_snapshot!(value[0]["messages"][0]["fix"].to_string(), @r###"{"range":[51,60],"text":"s"}"###);
}

#[test]
fn json_output_has_the_eslint_result_shape() {
    let project = Project::new(
        "json-shape",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .expect("lint");

    let json = render(Format::Json, &project.root(), &output.files, false);
    assert_snapshot!(json.replace(&project.root(), "<root>"), @r###"[{"filePath":"<root>/src/a.ts","messages":[{"ruleId":"strict-casting","severity":2,"message":"Unnecessary String() cast of a value that is already of type string","line":1,"column":33,"messageId":"unnecessaryCasting","endLine":1,"endColumn":42,"fix":{"range":[32,41],"text":"s"}}],"suppressedMessages":[],"errorCount":1,"fatalErrorCount":0,"warningCount":0,"fixableErrorCount":1,"fixableWarningCount":0,"usedDeprecatedRules":[]}]"###);
}

#[test]
fn trailing_slash_path_argument_selects_the_directory() {
    let project = Project::new(
        "trailing-slash",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    let (code, stdout) = project.cli(&["--format", "compact", "src/"]);
    assert_eq!(code, 1);
    assert_snapshot!(stdout, @r"
    src/a.ts: line 1, col 33, Error - Unnecessary String() cast of a value that is already of type string (strict-casting)

    1 problem
    ");

    let (code, _) = project.cli(&["--format", "compact", "./"]);
    assert_eq!(code, 1);
}

#[test]
fn explicit_declaration_file_argument_is_skipped() {
    let project = Project::new(
        "declaration-arg",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/env.d.ts", "declare const env: string\n"),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    let output = project
        .lint(
            ALL_RULES,
            &["tsconfig.json"],
            &["src/env.d.ts", "src/a.ts"],
            false,
        )
        .expect("lint");

    assert_snapshot!(output.warnings.join("\n"), @"src/env.d.ts: file ignored: it matches `ignores`, is a declaration file or is in node_modules");
    assert_snapshot!(summary(&project, &output), @"src/a.ts:1:33 strict-casting error");
}

#[test]
fn explicit_ignored_file_argument_is_a_warning() {
    let project = Project::new(
        "ignored-arg",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/gen/c.ts",
                "export const c = (s: string) => String(s)\n",
            ),
        ],
    );

    let output = project
        .lint(
            r#"{ "ignores": ["src/gen"] }"#,
            &["tsconfig.json"],
            &["src/gen/c.ts"],
            false,
        )
        .expect("lint");

    assert!(output.files.is_empty());
    assert_snapshot!(output.warnings.join("\n"), @"src/gen/c.ts: file ignored: it matches `ignores`, is a declaration file or is in node_modules");

    let error = project
        .lint(
            r#"{ "ignores": ["src/gen"] }"#,
            &["tsconfig.json"],
            &["src/gen"],
            false,
        )
        .err()
        .expect("an ignored directory argument should fail");
    assert_snapshot!(error, @"all files matched by src/gen are ignored");
}

#[test]
fn ignores_match_directories_and_support_negation() {
    let project = Project::new(
        "ignores",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/gen/a.ts",
                "export const a = (s: string) => String(s)\n",
            ),
            (
                "src/gen/b.ts",
                "export const b = (s: string) => String(s)\n",
            ),
            (
                "src/out/c.ts",
                "export const c = (s: string) => String(s)\n",
            ),
            (
                "src/lib/d.ts",
                "export const d = (s: string) => String(s)\n",
            ),
            (
                "src/lib/e.ts",
                "export const e = (s: string) => String(s)\n",
            ),
            (
                "src/vendor/f.ts",
                "export const f = (s: string) => String(s)\n",
            ),
            (
                "src/dist/g.ts",
                "export const g = (s: string) => String(s)\n",
            ),
        ],
    );
    let config = r#"{
        "ignores": [
            "src/gen",
            "**/out",
            "src/lib/**/*",
            "!src/lib/e.ts",
            "**/vendor/**",
            "!**/vendor/f.ts",
            "src/dist/**/"
        ],
        "rules": { "strict-casting": "error" }
    }"#;

    let output = project
        .lint(config, &["tsconfig.json"], &[], false)
        .expect("lint");

    assert_snapshot!(summary(&project, &output), @"src/lib/e.ts:1:33 strict-casting error");
}

#[test]
fn globs_match_files_outside_the_config_directory() {
    let project = Project::new(
        "outside-config",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
            ("src/b.ts", "export const b = (s: string) => String(s)\n"),
        ],
    );
    let config = Config::from_json(
        r#"{
            "ignores": ["../src/b.ts"],
            "rules": { "strict-casting": "warn" },
            "overrides": [{ "files": ["../src/**"], "rules": { "strict-casting": "error" } }]
        }"#,
    )
    .expect("config");
    let mut options = project.options(&["tsconfig.json"], &[], false);
    options.root = project.path("config");

    let output = run(&config, &options).expect("lint");

    assert_snapshot!(summary(&project, &output), @"src/a.ts:1:33 strict-casting error");
}

#[test]
fn parsing_errors_are_reported() {
    let project = Project::new(
        "parse-error",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/a.ts",
                "const broken: = ;\nexport const a = (s: string) => String(s)\n",
            ),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .expect("lint");

    let compact = render(Format::Compact, &project.root(), &output.files, false);
    assert_snapshot!(compact, @r"
    src/a.ts: line 1, col 15, Error - Parsing error: Type expected.

    1 problem
    ");
}

#[test]
fn directives_follow_eslint_comment_rules() {
    let project = Project::new(
        "directive-comments",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/a.ts",
                "// add /* eslint-disable */ at the top to silence this\nexport const a = (s: string) => String(s)\n/* eslint-disable-next-line\n   strict-casting */\nexport const b = (s: string) => String(s)\n// eslint-disable strict-casting\nexport const c = (s: string) => String(s)\n",
            ),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .expect("lint");

    assert_snapshot!(summary(&project, &output), @r###"
    src/a.ts:2:33 strict-casting error
    src/a.ts:7:33 strict-casting error
    "###);
}

#[test]
fn quiet_counts_warnings_for_max_warnings_and_fixes_only_errors() {
    let project = Project::new(
        "quiet",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "fast-lint.config.json",
                r#"{ "rules": { "strict-casting": "warn" } }"#,
            ),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    let (code, stdout) = project.cli(&["--quiet", "--max-warnings", "0"]);
    assert_eq!((code, stdout.as_str()), (1, ""));

    let (code, stdout) = project.cli(&["--quiet", "--fix"]);
    assert_eq!((code, stdout.as_str()), (0, ""));
    assert_eq!(
        std::fs::read_to_string(project.path("src/a.ts")).expect("read"),
        "export const a = (s: string) => String(s)\n"
    );
}

#[test]
fn fix_keeps_the_utf8_bom() {
    let project = Project::new(
        "bom",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/a.ts",
                "\u{FEFF}export const a = (s: string) => String(s)\n",
            ),
        ],
    );

    let (code, _) = project.cli(&["--fix"]);
    assert_eq!(code, 0);
    assert_eq!(
        std::fs::read_to_string(project.path("src/a.ts")).expect("read"),
        "\u{FEFF}export const a = (s: string) => s\n"
    );
}

#[test]
fn printed_paths_are_relative_to_the_current_directory() {
    let project = Project::new(
        "cwd-paths",
        &[
            ("tsconfig.json", TSCONFIG),
            ("fast-lint.config.json", ALL_RULES),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_fast-lint"))
        .args([
            "--format",
            "compact",
            "-c",
            "../fast-lint.config.json",
            "-p",
            "../tsconfig.json",
        ])
        .current_dir(project.root.join("src"))
        .output()
        .expect("run fast-lint");

    assert_snapshot!(String::from_utf8_lossy(&output.stdout), @r"
    a.ts: line 1, col 33, Error - Unnecessary String() cast of a value that is already of type string (strict-casting)

    1 problem
    ");
}

#[test]
fn fix_skips_files_that_are_not_utf8_and_keeps_their_problems() {
    let project = Project::new("latin1", &[("tsconfig.json", TSCONFIG)]);
    std::fs::create_dir_all(project.path("src")).expect("create dir");
    std::fs::write(
        project.path("src/a.ts"),
        b"// caf\xE9\nexport const a = (s: string) => String(s)\n",
    )
    .expect("write file");

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], true)
        .expect("lint");

    assert!(output.files.iter().all(|f| f.fixed_text.is_none()));
    assert_snapshot!(summary(&project, &output), @"src/a.ts:2:33 strict-casting error");
    assert_eq!(
        output
            .warnings
            .join("\n")
            .replace(&project.root(), "<root>"),
        "<root>/src/a.ts: not fixed, the file isn't UTF-8"
    );

    let (code, _) = project.cli(&["--fix"]);
    assert_eq!(code, 1);
}

#[test]
fn write_errors_are_reported_after_every_file() {
    let project = Project::new(
        "read-only",
        &[
            ("tsconfig.json", TSCONFIG),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
            ("src/b.ts", "export const b = (s: string) => String(s)\n"),
        ],
    );
    let a = project.path("src/a.ts");
    let mut permissions = std::fs::metadata(&a).expect("metadata").permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&a, permissions).expect("set read-only");

    let (code, _) = project.cli(&["--fix"]);

    assert_eq!(code, 2);
    assert_eq!(
        std::fs::read_to_string(project.path("src/b.ts")).expect("read"),
        "export const b = (s: string) => s\n"
    );
}

#[cfg(unix)]
#[test]
fn symlinked_path_arguments_select_their_files() {
    let project = Project::new(
        "symlinks",
        &[
            ("tsconfig.json", TSCONFIG),
            ("real/a.ts", "export const a = (s: string) => String(s)\n"),
            ("src/b.ts", "export const b = (s: string) => String(s)\n"),
        ],
    );
    std::os::unix::fs::symlink("../real/a.ts", project.path("src/a.ts")).expect("symlink");
    std::os::unix::fs::symlink(&project.root, project.path("link")).expect("symlink");

    let (code, stdout) = project.cli(&["--format", "compact", "src/a.ts"]);
    assert_eq!(code, 1);
    assert_snapshot!(stdout, @r"
    src/a.ts: line 1, col 33, Error - Unnecessary String() cast of a value that is already of type string (strict-casting)

    1 problem
    ");

    let through_link = project.path("link/src/b.ts");
    let (code, stdout) = project.cli(&["--format", "compact", &through_link]);
    assert_eq!(code, 1);
    assert_snapshot!(stdout, @r"
    src/b.ts: line 1, col 33, Error - Unnecessary String() cast of a value that is already of type string (strict-casting)

    1 problem
    ");
}

#[test]
fn negative_max_warnings_means_no_limit() {
    let project = Project::new(
        "max-warnings",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "fast-lint.config.json",
                r#"{ "rules": { "strict-casting": "warn" } }"#,
            ),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    assert_eq!(project.cli(&["--max-warnings", "-1"]).0, 0);
    assert_eq!(project.cli(&["--max-warnings", "0"]).0, 1);
}

#[test]
fn tsconfig_syntax_errors_are_warnings() {
    let project = Project::new(
        "tsconfig-syntax",
        &[
            (
                "tsconfig.json",
                r#"{ "compilerOptions": { "strict": true } "include": ["src"] }"#,
            ),
            ("src/a.ts", "export const a = (s: string) => String(s)\n"),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], false)
        .expect("lint");

    assert!(
        output.warnings.iter().any(|w| w.contains("TS1005")),
        "{:?}",
        output.warnings
    );
}

#[test]
fn fix_drops_suggestions_that_enclose_an_applied_fix() {
    let project = Project::new(
        "enclosing-suggestion",
        &[
            ("tsconfig.json", TSCONFIG),
            (
                "src/a.ts",
                "declare function run(s: string): string\nexport function f(s: string) {\n  void run(String(s))\n}\n",
            ),
        ],
    );

    let output = project
        .lint(ALL_RULES, &["tsconfig.json"], &[], true)
        .expect("lint");

    let file = output.files.first().expect("a file result");
    assert_snapshot!(file.fixed_text.as_deref().unwrap_or_default(), @r"
    declare function run(s: string): string
    export function f(s: string) {
      void run(s)
    }
    ");
    let void_diagnostic = file
        .diagnostics
        .iter()
        .find(|d| d.rule == "no-unnecessary-void-on-promise")
        .expect("the void diagnostic");
    assert!(void_diagnostic.suggestions.is_empty());
}
