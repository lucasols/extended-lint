use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use fast_lint::config::{Config, Severity};
use fast_lint::linter::{self, LintOptions};
use fast_lint::report::{self, Format};
use tsrs_core::tspath;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const DEFAULT_CONFIG: &str = "fast-lint.config.json";

/// The npm package version when built by `scripts/build.mjs`, so `--version` matches the published package.
const VERSION: &str = match option_env!("FAST_LINT_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

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
      --max-warnings <n>    Fail when there are more than n warnings
      --timing              Print program and lint times to stderr
      --no-color            Disable colors
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
        timing: false,
        color: (std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none())
            || std::env::var_os("FORCE_COLOR").is_some_and(|v| v != "0"),
        paths: Vec::new(),
    };
    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        let (name, inline_value) = match arg.split_once('=') {
            Some((n, v)) if n.starts_with("--") => (n.to_string(), Some(v.to_string())),
            _ => (arg.clone(), None),
        };
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
                args.max_warnings = Some(
                    v.parse()
                        .map_err(|_| format!("--max-warnings expects a number, got \"{v}\""))?,
                );
            }
            "--timing" => args.timing = true,
            "--no-color" => args.color = false,
            _ if name.starts_with('-') && name.len() > 1 => {
                return Err(format!("unknown option {name}\n\n{HELP}"))
            }
            _ => args.paths.push(arg),
        }
    }
    Ok(Parsed::Run(args))
}

fn absolute(cwd: &str, path: &str) -> String {
    tspath::normalize_path(&tspath::resolve_path(cwd, &[path]))
}

/// The config and the directory its relative paths resolve from.
fn load_config(args: &Args, cwd: &str) -> Result<(Config, String), String> {
    if let Some(json) = &args.config_json {
        let root = args
            .config_root
            .as_deref()
            .map_or_else(|| cwd.to_string(), |r| absolute(cwd, r));
        return Ok((Config::from_json(json)?, root));
    }
    let path = match &args.config {
        Some(path) => absolute(cwd, path),
        None => {
            let default = absolute(cwd, DEFAULT_CONFIG);
            if !std::path::Path::new(&default).is_file() {
                return Ok((Config::all_rules(), cwd.to_string()));
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
        args.projects.iter().map(|p| absolute(&cwd, p)).collect()
    } else if !config.projects.is_empty() {
        config.projects.iter().map(|p| absolute(&root, p)).collect()
    } else {
        vec![absolute(&root, "tsconfig.json")]
    };
    let opts = LintOptions {
        root: root.clone(),
        projects,
        paths: args.paths.iter().map(|p| absolute(&cwd, p)).collect(),
        fix: args.fix,
    };

    let mut output = linter::run(&config, &opts)?;

    let mut stderr = std::io::stderr().lock();
    for warning in &output.warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }
    if args.fix {
        for file in &output.files {
            if let Some(text) = &file.fixed_text {
                std::fs::write(&file.path, text)
                    .map_err(|e| format!("failed to write {}: {e}", file.path))?;
            }
        }
    }
    if args.quiet {
        for file in &mut output.files {
            file.diagnostics.retain(|d| d.severity == Severity::Error);
        }
    }
    output.files.retain(|f| !f.diagnostics.is_empty());

    let rendered = report::render(args.format, &root, &output.files, args.color);
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

    let counts = report::count(&output.files);
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
