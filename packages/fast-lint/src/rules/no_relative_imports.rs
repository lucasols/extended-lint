use tsrs_ast::{Kind, Node};
use tsrs_core::tspath;
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

pub const NAME: &str = "no-relative-imports";

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Alias {
    find: String,
    replacement: String,
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    aliases: Vec<Alias>,
    #[serde(default)]
    root_dir: Option<String>,
    #[serde(default)]
    allow_not_found_aliases: bool,
    #[serde(default, rename = "_dev_simulateFileName")]
    dev_simulate_file_name: Option<String>,
}

pub struct NoRelativeImports {
    options: Options,
    root_dir: String,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let cwd = std::env::current_dir()
        .map(|dir| tspath::normalize_path(&dir.to_string_lossy()))
        .unwrap_or_else(|_| "/".to_string());
    let root_dir = match &options.root_dir {
        Some(dir) => resolve(&cwd, dir),
        None => cwd,
    };
    Ok(Box::new(NoRelativeImports { options, root_dir }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::ImportDeclaration)];

impl Rule for NoRelativeImports {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoRelativeImports,
}

/// Node's `path.resolve(base, path)`: an absolute, normalized path without a trailing separator.
fn resolve(base: &str, path: &str) -> String {
    let resolved = tspath::get_normalized_absolute_path(path, base);
    match resolved.strip_suffix('/') {
        Some(stripped) if !stripped.is_empty() && !stripped.ends_with(':') => stripped.to_string(),
        _ => resolved,
    }
}

/// Node's `path.relative(from, to)` for absolute, normalized paths.
fn relative(from: &str, to: &str) -> String {
    let from_parts: Vec<&str> = from.split('/').filter(|part| !part.is_empty()).collect();
    let to_parts: Vec<&str> = to.split('/').filter(|part| !part.is_empty()).collect();
    let common = from_parts
        .iter()
        .zip(&to_parts)
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts: Vec<&str> = vec![".."; from_parts.len() - common];
    parts.extend(to_parts.get(common..).unwrap_or_default());
    parts.join("/")
}

impl Visitor {
    /// The alias whose replacement prefixes `absolute_path` (relative to `rootDir`, with a leading `/`), and the
    /// import path rewritten with it.
    fn find_matching_alias(&self, absolute_path: &str) -> Option<(&'static str, String)> {
        let mut relative_to_root = relative(&self.rule.root_dir, absolute_path);
        if relative_to_root.starts_with('.') {
            return None;
        }
        if !relative_to_root.starts_with('/') {
            relative_to_root.insert(0, '/');
        }
        self.rule.options.aliases.iter().find_map(|alias| {
            relative_to_root
                .strip_prefix(alias.replacement.as_str())
                .map(|rest| (alias.find.as_str(), format!("{}{rest}", alias.find)))
        })
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let source = node.as_import_declaration().module_specifier();
        let import_path = source.text();
        if !import_path.starts_with('.') {
            return;
        }
        let file_name = self
            .rule
            .options
            .dev_simulate_file_name
            .as_deref()
            .unwrap_or_else(|| ctx.file.file_name());
        let directory = tspath::get_directory_path(&tspath::normalize_path(file_name));
        let absolute_path = resolve(&directory, import_path);
        let Some((alias, new_path)) = self.find_matching_alias(&absolute_path) else {
            if self.rule.options.allow_not_found_aliases {
                return;
            }
            ctx.report_node(
                node,
                RuleMessage::new(
                    "noRelativeImports",
                    "Relative imports are not allowed. Use one of the valid aliases in tsconfig.json instead.",
                ),
            );
            return;
        };
        let message = RuleMessage::new(
            "noRelativeImportsWithAlias",
            format!("Relative imports are not allowed. Use the \"{alias}\" alias instead."),
        );
        ctx.report_node_with_fixes(node, message, |ctx| {
            vec![ctx.fix_replace(source, format!("'{new_path}'"))]
        });
    }
}
