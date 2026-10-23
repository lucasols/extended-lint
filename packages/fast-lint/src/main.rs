use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use fast_lint::config::{Config, Severity};
use fast_lint::linter::{self, LintOptions};
use fast_lint::report::{self, Format};
use tsrs_core::tspath;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const DEFAULT_CONFIG: &str = "fast-lint.config.json";

/// The crate version. The npm launcher answers `--version` itself with the package version, so release binaries
/// don't embed it and stay identical across releases without code changes.
const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
fast-lint: type-aware lint rules on a native TypeScript checker

Usage: fast-lint [options] [paths...]

Paths limit linting to those files or directories (default: every file of the projects).

Options:
  -p, --project <tsconfig>  tsconfig to lint, repeatable (default: config `projects`, else tsconfig.json)
  -c, --config <file>       JSON config file (default: fast-lint.config.json when present)
      --config-json <json>  Inline JSON config (the npm launcher passes .ts/.js configs this way)
      --config-root <dir>   Directory --config-json paths resolve from (default: current directory)
      --fix                 Apply autofixes
  -f, --format <format>     stylish (default), compact or json
      --quiet               Report errors only
      --max-warnings <n>    Fail when there are more than n warnings (-1: no limit)
      --report-unused-disable-directives
                            Warn about fast-lint-disable directives that suppress nothing
      --timing              Print program and lint times to stderr
      --color, --no-color   Force or disable colors
  -h, --help                Print this help
  -v, --version             Print the version
";

struct Args {
    projects: Vec<String>,
    config: Option<String>,
    config_json: Option<String>,
    config_root: Option<String>,
    fix: bool,
    format: Format,
    quiet: bool,
    max_warnings: Option<usize>,
    report_unused_disable_directives: bool,
    timing: bool,
    color: bool,
    paths: Vec<String>,
}

enum Parsed {
    Run(Args),
    Exit(&'static str),
}

fn parse_args(raw: Vec<String>) -> Result<Parsed, String> {
    let mut args = Args {
        projects: Vec::new(),
        config: None,
        config_json: None,
        config_root: None,
        fix: false,
        format: Format::Stylish,
        quiet: false,
        max_warnings: None,
        report_unused_disable_directives: false,
        timing: false,
        color: (std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none())
            || std::env::var_os("FORCE_COLOR").is_some_and(|v| v != "0"),
        paths: Vec::new(),
    };
    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--" {
            args.paths.extend(iter.by_ref());
            break;
        }
        let (name, inline_value) = match arg.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n.to_string(), Some(v.to_string())),
            _ => (arg.clone(), None),
        };
        if inline_value.is_some()
            && matches!(
                name.as_str(),
                "--help"
                    | "--version"
                    | "--fix"
                    | "--quiet"
                    | "--report-unused-disable-directives"
                    | "--timing"
                    | "--color"
                    | "--no-color"
            )
        {
            return Err(format!("{name} doesn't take a value"));
        }
        let mut value = |flag: &str| -> Result<String, String> {
            inline_value
                .clone()
                .or_else(|| iter.next())
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match name.as_str() {
            "-h" | "--help" => return Ok(Parsed::Exit(HELP)),
            "-v" | "--version" => return Ok(Parsed::Exit(VERSION)),
            "-p" | "--project" => args.projects.push(value(&name)?),
            "-c" | "--config" => args.config = Some(value(&name)?),
            "--config-json" => args.config_json = Some(value(&name)?),
            "--config-root" => args.config_root = Some(value(&name)?),
            "--fix" => args.fix = true,
            "-f" | "--format" => args.format = Format::parse(&value(&name)?)?,
            "--quiet" => args.quiet = true,
            "--max-warnings" => {
                let v = value(&name)?;
                let max: i64 = v
                    .parse()
                    .map_err(|_| format!("--max-warnings expects a number, got \"{v}\""))?;
                args.max_warnings = usize::try_from(max).ok();
            }
            "--report-unused-disable-directives" => args.report_unused_disable_directives = true,
            "--timing" => args.timing = true,
            "--color" => args.color = true,
            "--no-color" => args.color = false,
            _ if name.starts_with('-') && name.len() > 1 => {
                return Err(format!("unknown option {name}\n\n{HELP}"))
            }
            _ => args.paths.push(arg),
        }
    }
    Ok(Parsed::Run(args))
}

fn canonical(path: &str) -> Option<String> {
    let canonical = std::fs::canonicalize(path).ok()?;
    Some(tspath::normalize_path(&without_verbatim_prefix(
        &canonical.to_string_lossy(),
    )))
}

/// Windows' `canonicalize` gives verbatim paths: `\\?\C:\dir` for `C:\dir` and `\\?\UNC\server\share` for
/// `\\server\share`.
fn without_verbatim_prefix(path: &str) -> String {
    if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{unc}");
    }
    path.strip_prefix(r"\\?\").unwrap_or(path).to_string()
}

/// The absolute, normalized form of `path` with the symlinks of its directory resolved, so it compares equal to the
/// physical current directory. The last component is kept: a symlinked tsconfig or source file is used by the path
/// it was given, like tsc does.
fn absolute(cwd: &str, path: &str) -> String {
    let resolved = tspath::normalize_path(&tspath::resolve_path(cwd, &[path]));
    let resolved = without_trailing_slash(&resolved);
    let dir = tspath::get_directory_path(&resolved);
    if dir.len() >= resolved.len() {
        return canonical(&resolved).unwrap_or(resolved);
    }
    match canonical(&dir) {
        Some(dir) => tspath::combine_paths(&dir, &[&tspath::get_base_file_name(&resolved)]),
        None => resolved,
    }
}

/// The absolute, normalized form of directory `path`, with every symlink resolved like the current directory's.
fn absolute_dir(cwd: &str, path: &str) -> String {
    let resolved = tspath::normalize_path(&tspath::resolve_path(cwd, &[path]));
    canonical(&resolved).unwrap_or_else(|| without_trailing_slash(&resolved))
}

/// `path` without a trailing `/`, unless it is a root.
fn without_trailing_slash(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.len() == path.len() || tspath::get_root_length(path) >= path.len() {
        path.to_string()
    } else {
        trimmed.to_string()
    }
}

/// The forms a path argument may take in the projects' file names, which keep symlinks below the tsconfig
/// directory: as given (resolved against the physical current directory), with its directory's symlinks resolved
/// and fully resolved.
fn path_arg(cwd: &str, path: &str) -> Vec<String> {
    let logical =
        without_trailing_slash(&tspath::normalize_path(&tspath::resolve_path(cwd, &[path])));
    let first = absolute(cwd, path);
    let physical = canonical(&first);
    let mut forms = vec![first];
    for form in [Some(logical), physical].into_iter().flatten() {
        if !forms.contains(&form) {
            forms.push(form);
        }
    }
    forms
}

/// Writes fixed text back, keeping a UTF-8 BOM. The linter only fixes UTF-8 files. The text goes to a temporary file
/// next to the target (a symlink's target), which then replaces it: an interrupted write never truncates the source.
fn write_fixed(path: &str, text: &str) -> Result<(), String> {
    const BOM: &[u8] = b"\xEF\xBB\xBF";
    let target = std::fs::canonicalize(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let permissions = std::fs::metadata(&target)
        .map_err(|e| format!("failed to read {path}: {e}"))?
        .permissions();
    if permissions.readonly() {
        return Err(format!("failed to write {path}: the file is read-only"));
    }
    let original = std::fs::read(&target).map_err(|e| format!("failed to read {path}: {e}"))?;
    let mut bytes = Vec::with_capacity(text.len() + BOM.len());
    if original.starts_with(BOM) {
        bytes.extend_from_slice(BOM);
    }
    bytes.extend_from_slice(text.as_bytes());
    let file_name = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = target.with_file_name(format!(".{file_name}.fast-lint-{}.tmp", std::process::id()));
    let result = std::fs::write(&temp, bytes)
        .and_then(|()| std::fs::set_permissions(&temp, permissions))
        .and_then(|()| std::fs::rename(&temp, &target));
    result.map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("failed to write {path}: {e}")
    })
}

/// A tsconfig path; a directory means its `tsconfig.json`, like `tsc -p`.
fn project_path(base: &str, path: &str) -> String {
    let resolved = absolute(base, path);
    if std::path::Path::new(&resolved).is_dir() {
        tspath::combine_paths(&resolved, &["tsconfig.json"])
    } else {
        resolved
    }
}

/// The config and the directory its relative paths resolve from.
fn load_config(args: &Args, cwd: &str) -> Result<(Config, String), String> {
    if let Some(json) = &args.config_json {
        let root = args
            .config_root
            .as_deref()
            .map_or_else(|| cwd.to_string(), |r| absolute_dir(cwd, r));
        return Ok((Config::from_json(json)?, root));
    }
    let path = match &args.config {
        Some(path) => absolute(cwd, path),
        None => {
            let default = absolute(cwd, DEFAULT_CONFIG);
            if !std::path::Path::new(&default).is_file() {
                return Ok((Config::default_rules(), cwd.to_string()));
            }
            default
        }
    };
    let text = std::fs::read_to_string(&path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let config = Config::from_json(&text).map_err(|e| format!("{path}: {e}"))?;
    Ok((config, tspath::get_directory_path(&path)))
}

fn run(args: Args) -> Result<ExitCode, String> {
    let cwd = std::env::current_dir()
        .map(|c| tspath::normalize_path(&c.to_string_lossy()))
        .map_err(|e| format!("failed to read the current directory: {e}"))?;
    let (config, root) = load_config(&args, &cwd)?;

    let projects: Vec<String> = if !args.projects.is_empty() {
        args.projects
            .iter()
            .map(|p| project_path(&cwd, p))
            .collect()
    } else if !config.projects.is_empty() {
        config
            .projects
            .iter()
            .map(|p| project_path(&root, p))
            .collect()
    } else {
        vec![absolute(&root, "tsconfig.json")]
    };
    let opts = LintOptions {
        root: root.clone(),
        projects,
        paths: args.paths.iter().map(|p| path_arg(&cwd, p)).collect(),
        fix: args.fix,
        errors_only: args.quiet && args.max_warnings.is_none(),
        fix_errors_only: args.quiet,
        report_unused_disable_directives: args.report_unused_disable_directives
            || config.report_unused_disable_directives,
    };

    let mut output = linter::run(&config, &opts)?;

    let write_errors: Vec<String> = output
        .files
        .iter()
        .filter_map(|file| {
            let text = file.fixed_text.as_ref()?;
            write_fixed(&file.path, text).err()
        })
        .collect();
    let mut stderr = std::io::stderr().lock();
    for warning in &output.warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }
    let counts = report::count(&output.files);
    if args.quiet {
        for file in &mut output.files {
            file.diagnostics.retain(|d| d.severity == Severity::Error);
        }
    }
    output
        .files
        .retain(|f| !f.diagnostics.is_empty() || f.fixed_text.is_some());

    let rendered = report::render(args.format, &cwd, &output.files, args.color);
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(rendered.as_bytes());
    if args.format == Format::Json {
        let _ = writeln!(stdout);
    }
    let _ = stdout.flush();

    if args.timing {
        let _ = writeln!(
            stderr,
            "fast-lint: {} files, program {:.2}s, lint {:.2}s",
            output.linted_files,
            output.timings.program.as_secs_f64(),
            output.timings.lint.as_secs_f64()
        );
    }

    if !write_errors.is_empty() {
        for error in &write_errors {
            let _ = writeln!(stderr, "fast-lint: {error}");
        }
        return Ok(ExitCode::from(2));
    }

    let too_many_warnings = args.max_warnings.is_some_and(|max| counts.warnings > max);
    if counts.errors > 0 || too_many_warnings {
        if too_many_warnings && counts.errors == 0 {
            let _ = writeln!(
                stderr,
                "fast-lint found too many warnings (maximum: {}).",
                args.max_warnings.unwrap_or_default()
            );
        }
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_hook(info);
        if let Some(file) = linter::current_file() {
            eprintln!("fast-lint: crashed while linting {file}");
        }
    }));
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(raw) {
        Ok(Parsed::Run(args)) => args,
        Ok(Parsed::Exit(text)) => {
            println!("{}", text.trim_end());
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("fast-lint: {e}");
            return ExitCode::from(2);
        }
    };
    // Type checking recurses deeply: run on a thread with a large stack, like tsrslint's headless binary.
    let handle = std::thread::Builder::new()
        .name("fast-lint".to_string())
        .stack_size(512 << 20)
        .spawn(move || run(args));
    let result = match handle {
        Ok(handle) => handle
            .join()
            .unwrap_or_else(|_| Err("internal error: the lint thread panicked".to_string())),
        Err(e) => Err(format!("failed to start the lint thread: {e}")),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("fast-lint: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_args, without_verbatim_prefix, Parsed};

    #[test]
    fn verbatim_prefixes_are_removed() {
        assert_eq!(without_verbatim_prefix(r"\\?\C:\dir"), r"C:\dir");
        assert_eq!(
            without_verbatim_prefix(r"\\?\UNC\server\share\dir"),
            r"\\server\share\dir"
        );
        assert_eq!(without_verbatim_prefix("/home/dir"), "/home/dir");
    }

    fn parse(args: &[&str]) -> Result<Vec<String>, String> {
        match parse_args(args.iter().map(ToString::to_string).collect())? {
            Parsed::Run(args) => Ok(args.paths),
            Parsed::Exit(_) => Err("exit".to_string()),
        }
    }

    #[test]
    fn flags_reject_inline_values_and_double_dash_ends_options() {
        assert_eq!(
            parse(&["--fix=false"]),
            Err("--fix doesn't take a value".to_string())
        );
        assert_eq!(
            parse(&["--color", "--", "--fix", "src"]),
            Ok(vec!["--fix".to_string(), "src".to_string()])
        );
    }
}
