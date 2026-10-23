use std::fmt::Write as _;
use std::path::PathBuf;

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

    fn lint(
        &self,
        config: &str,
        projects: &[&str],
        paths: &[&str],
        fix: bool,
    ) -> Result<LintOutput, String> {
        let config = Config::from_json(config)?;
        let opts = LintOptions {
            root: self.root(),
            projects: projects.iter().map(|p| self.path(p)).collect(),
            paths: paths.iter().map(|p| self.path(p)).collect(),
            fix,
        };
        run(&config, &opts)
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
    assert_snapshot!(value[0]["messages"][0]["fixes"].to_string(), @r###"[{"range":[51,60],"text":"s"}]"###);
}
