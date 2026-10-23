use regex::{Captures, Regex};
use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{
    Ctx, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleSuggestion, RuleVisitor,
};

use crate::esquery::estree::{self, EsNode, EsType, EsValue};
use crate::esquery::parse::js_number_to_string;
use crate::esquery::{CompiledSelector, SelectorIndex};
use crate::utils::skip_parens;

pub const NAME: &str = "no-restricted-syntax";

/// The former name of the rule in `@ls-stack/extended-lint`.
pub const ALIAS: &str = "advanced-no-restricted-syntax";

const MESSAGE_ID: &str = "default";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    disallow: Option<Vec<DisallowOptions>>,
    disallow_fn_calls: Option<Vec<DisallowFnCallOptions>>,
    must_match_syntax: Option<Vec<MustMatchSyntaxOptions>>,
    #[serde(rename = "__dev_simulateFileName")]
    dev_simulate_file_name: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct DisallowOptions {
    selector: String,
    message: String,
    replace: Option<ReplaceOptions>,
    replace_type: Option<ReplaceType>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ReplaceOptions {
    Text(String),
    Regex(RegexReplaceOptions),
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RegexReplaceOptions {
    regex: String,
    with: String,
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ReplaceType {
    Suggestion,
    Autofix,
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
enum LiteralValue {
    Bool(bool),
    Num(f64),
    Str(String),
}

impl LiteralValue {
    fn display(&self) -> String {
        match self {
            LiteralValue::Bool(b) => b.to_string(),
            LiteralValue::Num(n) => js_number_to_string(*n),
            LiteralValue::Str(s) => s.clone(),
        }
    }

    /// The literal's source text, for fixes.
    fn source(&self) -> String {
        match self {
            LiteralValue::Str(s) => format!("'{s}'"),
            other => other.display(),
        }
    }

    /// `===` against a literal's value.
    fn equals(&self, value: &EsValue) -> bool {
        match (self, value) {
            (LiteralValue::Bool(a), EsValue::Bool(b)) => a == b,
            (LiteralValue::Num(a), EsValue::Num(b)) => a == b,
            (LiteralValue::Str(a), EsValue::Str(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgValueOptions {
    at_index: usize,
    value: LiteralValue,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct DisallowFnCallOptions {
    #[serde(rename = "fn")]
    function: String,
    with_args: Option<Vec<ArgValueOptions>>,
    message: String,
    replace_with: Option<String>,
    ignore_regex: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MustMatchSyntaxOptions {
    include_regex: String,
    exclude_regex: Option<String>,
    must_call_fn: Option<Vec<MustCallFnOptions>>,
    must_match_selector: Option<Vec<SelectorMessageOptions>>,
    must_have_export: Option<Vec<MustHaveExportOptions>>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MustCallFnOptions {
    any_call: Vec<AnyCallOptions>,
    message: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AnyCallOptions {
    #[serde(rename = "fn")]
    function: String,
    with_args: Vec<ArgLiteralOptions>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgLiteralOptions {
    at_index: usize,
    literal: LiteralValue,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectorMessageOptions {
    selector: String,
    message: String,
}

#[derive(serde::Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ExportType {
    Function,
    Variable,
    #[default]
    Any,
}

impl ExportType {
    fn name(self) -> &'static str {
        match self {
            ExportType::Function => "function",
            ExportType::Variable => "variable",
            ExportType::Any => "any",
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MustHaveExportOptions {
    name: String,
    #[serde(default, rename = "type")]
    export_type: ExportType,
    message: String,
}

enum Replacement {
    Text(String),
    Regex { regex: Regex, with: String },
}

struct Disallow {
    message: String,
    replacement: Option<Replacement>,
    autofix: bool,
}

struct DisallowFnCall {
    function: String,
    with_args: Option<Vec<(usize, LiteralValue)>>,
    message: String,
    replace_with: Option<String>,
    ignore: Option<Regex>,
}

struct MustMatchSyntax {
    include: Regex,
    exclude: Option<Regex>,
    must_call_fn: Vec<MustCallFnOptions>,
    must_match_selector: Vec<SelectorMessageOptions>,
    must_have_export: Vec<MustHaveExportOptions>,
}

pub struct NoRestrictedSyntax {
    disallow: Vec<Disallow>,
    index: SelectorIndex<usize>,
    disallow_fn_calls: Vec<DisallowFnCall>,
    must_match_syntax: Vec<MustMatchSyntax>,
    simulated_file_name: Option<String>,
    listeners: Vec<Listener>,
}

fn regex(pattern: &str) -> Result<Regex, String> {
    Regex::new(pattern).map_err(|e| format!("invalid regex \"{pattern}\": {e}"))
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut index = SelectorIndex::new();
    let mut disallow = Vec::new();
    for (i, entry) in options.disallow.unwrap_or_default().into_iter().enumerate() {
        index.add(CompiledSelector::new(&entry.selector)?, i);
        let replacement = match entry.replace {
            None => None,
            Some(ReplaceOptions::Text(text)) => Some(Replacement::Text(text)),
            Some(ReplaceOptions::Regex(options)) => Some(Replacement::Regex {
                regex: regex(&options.regex)?,
                with: options.with,
            }),
        };
        disallow.push(Disallow {
            message: entry.message,
            replacement,
            autofix: entry.replace_type == Some(ReplaceType::Autofix),
        });
    }
    let mut disallow_fn_calls = Vec::new();
    for entry in options.disallow_fn_calls.unwrap_or_default() {
        disallow_fn_calls.push(DisallowFnCall {
            function: entry.function,
            with_args: entry
                .with_args
                .map(|args| args.into_iter().map(|a| (a.at_index, a.value)).collect()),
            message: entry.message,
            replace_with: entry.replace_with,
            ignore: entry.ignore_regex.as_deref().map(regex).transpose()?,
        });
    }
    let mut must_match_syntax = Vec::new();
    for entry in options.must_match_syntax.unwrap_or_default() {
        let must_match_selector = entry.must_match_selector.unwrap_or_default();
        for selector in &must_match_selector {
            CompiledSelector::new(&selector.selector)?;
        }
        must_match_syntax.push(MustMatchSyntax {
            include: regex(&entry.include_regex)?,
            exclude: entry.exclude_regex.as_deref().map(regex).transpose()?,
            must_call_fn: entry.must_call_fn.unwrap_or_default(),
            must_match_selector,
            must_have_export: entry.must_have_export.unwrap_or_default(),
        });
    }
    let mut listeners: Vec<Listener> = index
        .kinds()
        .kinds()
        .into_iter()
        .map(Listener::Enter)
        .collect();
    if index.has_program_selectors() {
        listeners.push(Listener::Enter(Kind::EndOfFile));
    }
    if !disallow_fn_calls.is_empty() {
        listeners.extend(
            [
                Kind::CallExpression,
                Kind::ImportDeclaration,
                Kind::VariableDeclaration,
            ]
            .map(Listener::Enter),
        );
    }
    if !index.is_empty() {
        listeners.push(Listener::AllowPattern(Kind::PropertyAssignment));
    }
    dedupe(&mut listeners);
    Ok(Box::new(NoRestrictedSyntax {
        disallow,
        index,
        disallow_fn_calls,
        must_match_syntax,
        simulated_file_name: options.dev_simulate_file_name,
        listeners,
    }))
}

fn dedupe(listeners: &mut Vec<Listener>) {
    let mut seen = rustc_hash::FxHashSet::default();
    listeners.retain(|listener| seen.insert(*listener));
}

impl Rule for NoRestrictedSyntax {
    fn name(&self) -> &'static str {
        NAME
    }

    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let file_name = self
            .simulated_file_name
            .clone()
            .unwrap_or_else(|| ctx.file.file_name().to_string());
        let disallow_fn_calls = self
            .disallow_fn_calls
            .iter()
            .filter(|entry| {
                entry
                    .ignore
                    .as_ref()
                    .is_none_or(|re| !re.is_match(&file_name))
            })
            .collect();
        let file_checks = FileChecks::new(self, &file_name);
        let listeners = file_checks.as_ref().map(|checks| {
            let mut listeners = self.listeners.clone();
            listeners.extend(
                checks
                    .selectors
                    .kinds()
                    .kinds()
                    .into_iter()
                    .map(Listener::Enter),
            );
            listeners.extend(
                [
                    Kind::EndOfFile,
                    Kind::CallExpression,
                    Kind::FunctionDeclaration,
                    Kind::VariableStatement,
                    Kind::ExportDeclaration,
                ]
                .map(Listener::Enter),
            );
            if !checks.selectors.is_empty() {
                listeners.push(Listener::AllowPattern(Kind::PropertyAssignment));
            }
            dedupe(&mut listeners);
            listeners
        });
        Box::new(Visitor {
            rule: self,
            disallow_fn_calls,
            aliases: FxHashMap::default(),
            file_checks,
            listeners,
        })
    }
}

/// The variables of a file name matched by `includeRegex`, in the order they are substituted.
fn file_name_vars(captures: &Captures) -> Vec<(String, String)> {
    let mut vars = Vec::new();
    for (i, group) in captures.iter().enumerate() {
        let Some(group) = group else {
            continue;
        };
        let value = group.as_str();
        vars.push((format!("${i}_lowercase"), value.to_lowercase()));
        vars.push((format!("${i}_capitalize"), capitalize(value)));
        vars.push((format!("${i}_uncapitalize"), uncapitalize(value)));
        vars.push((format!("${i}"), value.to_string()));
    }
    vars
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn uncapitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|first| first.to_lowercase().chain(chars).collect())
        .unwrap_or_default()
}

fn replace_vars(s: &str, vars: &[(String, String)]) -> String {
    let mut result = s.to_string();
    for (name, value) in vars {
        result = result.replace(name.as_str(), value);
    }
    result
}

struct AnyCall {
    function: &'static str,
    with_args: Vec<(usize, LiteralValue)>,
}

struct MustCall {
    any_call: Vec<AnyCall>,
    message_suffix: String,
    /// Index of its "Expected file to call ..." message in `call_messages`.
    message_index: usize,
}

struct SelectorRequirement {
    raw: &'static str,
    message: String,
    matched: bool,
}

struct ExportRequirement {
    name: String,
    export_type: ExportType,
    message: String,
    found: bool,
}

/// The `mustMatchSyntax` checks of the blocks whose `includeRegex` matches the file.
struct FileChecks {
    must_calls: Vec<MustCall>,
    /// Ordered unique messages, with whether a matching call was seen.
    call_messages: Vec<(String, bool)>,
    selectors: SelectorIndex<usize>,
    selector_requirements: Vec<SelectorRequirement>,
    selector_errors: Vec<String>,
    exports: Vec<ExportRequirement>,
}

impl FileChecks {
    fn new(rule: &'static NoRestrictedSyntax, file_name: &str) -> Option<FileChecks> {
        let mut checks = FileChecks {
            must_calls: Vec::new(),
            call_messages: Vec::new(),
            selectors: SelectorIndex::new(),
            selector_requirements: Vec::new(),
            selector_errors: Vec::new(),
            exports: Vec::new(),
        };
        let mut any = false;
        for block in &rule.must_match_syntax {
            let Some(captures) = block.include.captures(file_name) else {
                continue;
            };
            if block
                .exclude
                .as_ref()
                .is_some_and(|re| re.is_match(file_name))
            {
                continue;
            }
            any = true;
            let vars = file_name_vars(&captures);
            for must_call in &block.must_call_fn {
                let functions: Vec<&str> = must_call
                    .any_call
                    .iter()
                    .map(|c| c.function.as_str())
                    .collect();
                let message = format!(
                    "Expected file to call the function: {}",
                    functions.join(" or ")
                );
                let message_index =
                    match checks.call_messages.iter().position(|(m, _)| *m == message) {
                        Some(index) => index,
                        None => {
                            checks.call_messages.push((message, false));
                            checks.call_messages.len() - 1
                        }
                    };
                checks.must_calls.push(MustCall {
                    any_call: must_call
                        .any_call
                        .iter()
                        .map(|call| AnyCall {
                            function: call.function.as_str(),
                            with_args: call
                                .with_args
                                .iter()
                                .map(|arg| {
                                    let value = match &arg.literal {
                                        LiteralValue::Str(s) => {
                                            LiteralValue::Str(replace_vars(s, &vars))
                                        }
                                        other => other.clone(),
                                    };
                                    (arg.at_index, value)
                                })
                                .collect(),
                        })
                        .collect(),
                    message_suffix: must_call
                        .message
                        .as_deref()
                        .map(|m| format!(": {}", replace_vars(m, &vars)))
                        .unwrap_or_default(),
                    message_index,
                });
            }
            for requirement in &block.must_match_selector {
                let message = replace_vars(&requirement.message, &vars);
                let owner = match checks
                    .selector_requirements
                    .iter()
                    .position(|r| r.raw == requirement.selector)
                {
                    Some(index) => {
                        if let Some(existing) = checks.selector_requirements.get_mut(index) {
                            existing.message = message;
                        }
                        index
                    }
                    None => {
                        checks.selector_requirements.push(SelectorRequirement {
                            raw: requirement.selector.as_str(),
                            message,
                            matched: false,
                        });
                        checks.selector_requirements.len() - 1
                    }
                };
                match CompiledSelector::new(&replace_vars(&requirement.selector, &vars)) {
                    Ok(selector) => checks.selectors.add(selector, owner),
                    Err(error) => checks.selector_errors.push(error),
                }
            }
            for export in &block.must_have_export {
                let name = replace_vars(&export.name, &vars);
                let message = replace_vars(&export.message, &vars);
                let duplicate = checks.exports.iter().any(|e| {
                    e.name == name && e.export_type == export.export_type && e.message == message
                });
                if !duplicate {
                    checks.exports.push(ExportRequirement {
                        name,
                        export_type: export.export_type,
                        message,
                        found: false,
                    });
                }
            }
        }
        any.then_some(checks)
    }

    fn mark_export(&mut self, name: &str, export_type: ExportType) {
        for requirement in &mut self.exports {
            if requirement.name == name
                && (requirement.export_type == export_type
                    || requirement.export_type == ExportType::Any)
            {
                requirement.found = true;
            }
        }
    }
}

struct Visitor {
    rule: &'static NoRestrictedSyntax,
    disallow_fn_calls: Vec<&'static DisallowFnCall>,
    aliases: FxHashMap<&'static str, &'static str>,
    file_checks: Option<FileChecks>,
    listeners: Option<Vec<Listener>>,
}

fn report_message(message: impl Into<String>) -> RuleMessage {
    RuleMessage::new(MESSAGE_ID, message)
}

fn diagnostic(pos: i32, end: i32, message: impl Into<String>) -> RuleDiagnostic {
    RuleDiagnostic {
        pos,
        end,
        message: report_message(message),
        labeled_ranges: Vec::new(),
    }
}

/// `text.replace(regex, replacement)` in JS: the first match, with `$&`, `$1`, `$<name>`, ... expanded.
fn js_replace(text: &str, regex: &Regex, replacement: &str) -> String {
    let Some(captures) = regex.captures(text) else {
        return text.to_string();
    };
    let Some(whole) = captures.get(0) else {
        return text.to_string();
    };
    let mut result = String::from(text.get(..whole.start()).unwrap_or_default());
    let group_count = captures.len() - 1;
    let chars: Vec<char> = replacement.chars().collect();
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        i += 1;
        if c != '$' {
            result.push(c);
            continue;
        }
        match chars.get(i) {
            Some('$') => {
                result.push('$');
                i += 1;
            }
            Some('&') => {
                result.push_str(whole.as_str());
                i += 1;
            }
            Some('`') => {
                result.push_str(text.get(..whole.start()).unwrap_or_default());
                i += 1;
            }
            Some('\'') => {
                result.push_str(text.get(whole.end()..).unwrap_or_default());
                i += 1;
            }
            Some(d) if d.is_ascii_digit() => {
                let one = d.to_digit(10).map_or(0, |n| n as usize);
                let two = chars
                    .get(i + 1)
                    .and_then(|d2| d2.to_digit(10))
                    .map(|d2| one * 10 + d2 as usize)
                    .filter(|&n| n >= 1 && n <= group_count);
                let (group, len) = match two {
                    Some(n) => (n, 2),
                    None if one >= 1 && one <= group_count => (one, 1),
                    None => {
                        result.push('$');
                        continue;
                    }
                };
                result.push_str(captures.get(group).map_or("", |m| m.as_str()));
                i += len;
            }
            Some('<') => {
                let rest: String = chars.get(i + 1..).unwrap_or_default().iter().collect();
                match rest.find('>') {
                    Some(close) if regex.capture_names().flatten().next().is_some() => {
                        let name = rest.get(..close).unwrap_or_default();
                        result.push_str(captures.name(name).map_or("", |m| m.as_str()));
                        i += 1 + rest.get(..=close).map_or(0, |s| s.chars().count());
                    }
                    _ => result.push('$'),
                }
            }
            _ => result.push('$'),
        }
    }
    result.push_str(text.get(whole.end()..).unwrap_or_default());
    result
}

fn literal_of(node: P<Node>) -> Option<EsValue> {
    let node = skip_parens(node);
    (estree::stack(node).types() == [EsType::Literal]).then(|| estree::literal_value(node))
}

impl Visitor {
    fn report_disallow(ctx: &mut Ctx, entry: &Disallow, es: EsNode) {
        let (pos, end) = estree::range(es, ctx.file);
        let message = entry.message.clone();
        let Some(replacement) = &entry.replacement else {
            ctx.report_diagnostic(diagnostic(pos, end, message));
            return;
        };
        let text = estree::source_text_range(ctx.file, pos, end);
        let (new_text, shown) = match replacement {
            Replacement::Text(text) => (text.clone(), text.as_str()),
            Replacement::Regex { regex, with } => (js_replace(text, regex, with), with.as_str()),
        };
        let fix = RuleFix {
            text: new_text,
            pos,
            end,
        };
        if entry.autofix {
            ctx.report_diagnostic_with_fixes(diagnostic(pos, end, message), |_| vec![fix]);
        } else {
            let suggestion_message = format!("Replace with \"{shown}\"");
            ctx.report_diagnostic_with_suggestions(diagnostic(pos, end, message), |_| {
                vec![RuleSuggestion {
                    message: report_message(suggestion_message),
                    fixes: vec![fix],
                }]
            });
        }
    }

    fn run_selectors(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let kind = node.kind();
        let rule = self.rule;
        if rule.index.kinds().contains(kind) {
            let mut matched = Vec::new();
            rule.index
                .visit(node, ctx.file, |owner, es| matched.push((owner, es)));
            for (owner, es) in matched {
                if let Some(entry) = rule.disallow.get(owner) {
                    Self::report_disallow(ctx, entry, es);
                }
            }
        }
        if let Some(checks) = &mut self.file_checks {
            if checks.selectors.kinds().contains(kind) {
                let requirements = &mut checks.selector_requirements;
                checks.selectors.visit(node, ctx.file, |owner, _| {
                    if let Some(requirement) = requirements.get_mut(owner) {
                        requirement.matched = true;
                    }
                });
            }
        }
    }

    /// Runs every check on `node` and its descendants: the keys of a destructuring assignment's properties,
    /// which the walker doesn't visit.
    fn enter_subtree(&mut self, ctx: &mut Ctx, node: P<Node>) {
        self.enter(ctx, node);
        let mut children = Vec::new();
        node.for_each_child(&mut |child| {
            children.push(child);
            false
        });
        for child in children {
            self.enter_subtree(ctx, child);
        }
    }

    fn enter(&mut self, ctx: &mut Ctx, node: P<Node>) {
        match node.kind() {
            Kind::CallExpression => self.check_call(ctx, node),
            Kind::ImportDeclaration => self.track_import_aliases(node),
            Kind::VariableDeclaration => self.track_variable_alias(node),
            Kind::FunctionDeclaration | Kind::VariableStatement | Kind::ExportDeclaration => {
                self.track_exports(node);
            }
            Kind::EndOfFile => {
                self.run_selectors(ctx, ctx.file.as_node());
                self.report_missing(ctx);
                return;
            }
            _ => {}
        }
        self.run_selectors(ctx, node);
    }

    fn track_import_aliases(&mut self, node: P<Node>) {
        if self.disallow_fn_calls.is_empty() {
            return;
        }
        let Some(clause) = node.as_import_declaration().import_clause else {
            return;
        };
        let Some(bindings) = clause.as_import_clause().named_bindings() else {
            return;
        };
        if bindings.kind() != Kind::NamedImports {
            return;
        }
        for &specifier in bindings.elements() {
            let Some(imported) = specifier.property_name() else {
                continue;
            };
            let local = specifier.as_import_specifier().name();
            if ast::is_identifier(imported) && imported.text() != local.text() {
                self.aliases.insert(local.text(), imported.text());
            }
        }
    }

    fn track_variable_alias(&mut self, node: P<Node>) {
        if self.disallow_fn_calls.is_empty() {
            return;
        }
        let (Some(name), Some(init)) = (node.name(), node.initializer()) else {
            return;
        };
        let init = skip_parens(init);
        if ast::is_identifier(name) && ast::is_identifier(init) {
            self.aliases.insert(name.text(), init.text());
        }
    }

    fn track_exports(&mut self, node: P<Node>) {
        let Some(checks) = &mut self.file_checks else {
            return;
        };
        if checks.exports.is_empty() {
            return;
        }
        match node.kind() {
            Kind::FunctionDeclaration => {
                let is_named_export = ast::has_syntactic_modifier(node, ModifierFlags::Export)
                    && !ast::has_syntactic_modifier(node, ModifierFlags::Default);
                if is_named_export && node.body().is_some() {
                    if let Some(name) = node.name() {
                        checks.mark_export(name.text(), ExportType::Function);
                    }
                }
            }
            Kind::VariableStatement => {
                if !ast::has_syntactic_modifier(node, ModifierFlags::Export) {
                    return;
                }
                let list = node.as_variable_statement().declaration_list;
                for &declaration in list.as_variable_declaration_list().declarations().nodes() {
                    if let Some(name) = declaration.name().filter(|&n| ast::is_identifier(n)) {
                        checks.mark_export(name.text(), ExportType::Variable);
                    }
                }
            }
            _ => {
                let Some(clause) = node.as_export_declaration().export_clause() else {
                    return;
                };
                if clause.kind() != Kind::NamedExports {
                    return;
                }
                for &specifier in clause.elements() {
                    let exported = specifier.as_export_specifier().name();
                    if ast::is_identifier(exported) {
                        for requirement in &mut checks.exports {
                            if requirement.name == exported.text()
                                && requirement.export_type == ExportType::Any
                            {
                                requirement.found = true;
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_call(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let callee = skip_parens(node.as_call_expression().expression());
        if !ast::is_identifier(callee) {
            return;
        }
        let callee_name = callee.text();
        let arguments = node.arguments();
        if let Some(checks) = &mut self.file_checks {
            for must_call in &checks.must_calls {
                let Some(call) = must_call
                    .any_call
                    .iter()
                    .find(|c| c.function == callee_name)
                else {
                    continue;
                };
                if let Some((_, called)) = checks.call_messages.get_mut(must_call.message_index) {
                    *called = true;
                }
                let suffix = &must_call.message_suffix;
                for (index, expected) in &call.with_args {
                    let shown = expected.display();
                    let Some(&argument) = arguments.get(*index) else {
                        ctx.report_node(
                            node,
                            report_message(format!(
                                "Missing argument with value \"{shown}\" at index {index}{suffix}"
                            )),
                        );
                        continue;
                    };
                    let argument = skip_parens(argument);
                    let Some(value) = literal_of(argument) else {
                        ctx.report_node(
                            argument,
                            report_message(format!(
                                "Argument at position {index} should the literal \"{shown}\"{suffix}"
                            )),
                        );
                        continue;
                    };
                    if !expected.equals(&value) {
                        let source = expected.source();
                        ctx.report_node_with_fixes(
                            argument,
                            report_message(format!(
                                "Argument should have the value \"{shown}\"{suffix}"
                            )),
                            |ctx| vec![ctx.fix_replace(argument, source)],
                        );
                    }
                }
            }
        }
        let original = self.aliases.get(callee_name).copied();
        for entry in &self.disallow_fn_calls {
            if callee_name != entry.function && original != Some(entry.function.as_str()) {
                continue;
            }
            if !Self::args_match(ctx, node, arguments, entry) {
                continue;
            }
            let message = report_message(entry.message.clone());
            match &entry.replace_with {
                Some(replace_with) => {
                    let replace_with = replace_with.clone();
                    ctx.report_node_with_suggestions(node, message, |ctx| {
                        vec![RuleSuggestion {
                            message: report_message(format!("Replace with \"{replace_with}\"")),
                            fixes: vec![ctx.fix_replace(node, replace_with.clone())],
                        }]
                    });
                }
                None => ctx.report_node(node, message),
            }
        }
    }

    /// Whether the call has `entry`'s argument values. A missing argument is reported (and doesn't match).
    fn args_match(
        ctx: &mut Ctx,
        node: P<Node>,
        arguments: &[P<Node>],
        entry: &DisallowFnCall,
    ) -> bool {
        let Some(with_args) = &entry.with_args else {
            return true;
        };
        for (index, expected) in with_args {
            let Some(&argument) = arguments.get(*index) else {
                ctx.report_node(
                    node,
                    report_message(format!(
                        "Missing argument with value \"{}\" at index {index}: {}",
                        expected.display(),
                        entry.message
                    )),
                );
                return false;
            };
            match literal_of(argument) {
                Some(value) if expected.equals(&value) => {}
                _ => return false,
            }
        }
        true
    }

    fn report_missing(&mut self, ctx: &mut Ctx) {
        let Some(checks) = &self.file_checks else {
            return;
        };
        let program = ctx.file.as_node();
        for (message, called) in &checks.call_messages {
            if !called {
                ctx.report_node(program, report_message(message.clone()));
            }
        }
        for requirement in &checks.selector_requirements {
            if !requirement.matched {
                ctx.report_node(program, report_message(requirement.message.clone()));
            }
        }
        for error in &checks.selector_errors {
            ctx.report_node(program, report_message(error.clone()));
        }
        for requirement in &checks.exports {
            if !requirement.found {
                ctx.report_node(
                    program,
                    report_message(format!(
                        "Missing required export \"{}\" of type {}: {}",
                        requirement.name,
                        requirement.export_type.name(),
                        requirement.message
                    )),
                );
            }
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        self.listeners.as_deref().unwrap_or(&self.rule.listeners)
    }

    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::AllowPattern(_) => {
                if let Some(name) = node.name() {
                    self.enter_subtree(ctx, name);
                }
            }
            _ => self.enter(ctx, node),
        }
    }
}
