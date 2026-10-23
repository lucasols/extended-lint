use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{is_estree_chain_expression, skip_parens};

pub const NAME: &str = "react-single-export";

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    #[serde(default = "default_extensions")]
    extensions: Vec<String>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            extensions: default_extensions(),
        }
    }
}

fn default_extensions() -> Vec<String> {
    vec!["tsx".to_string()]
}

pub struct ReactSingleExport {
    extensions: Vec<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    Ok(Box::new(ReactSingleExport {
        extensions: options.extensions,
    }))
}

impl Rule for ReactSingleExport {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let file_name = ctx.file.file_name();
        if self
            .extensions
            .iter()
            .any(|extension| file_name.ends_with(extension.as_str()))
        {
            check_exports(ctx);
        }
        Box::new(Visitor)
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &[]
    }
    fn visit(&mut self, _ctx: &mut Ctx, _listener: Listener, _node: P<Node>) {}
}

fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase()) && chars.all(|c| c.is_ascii_alphanumeric())
}

fn identifier_name(node: P<Node>) -> Option<&'static str> {
    ast::is_identifier(node).then(|| node.text())
}

/// A call ESLint sees as a `CallExpression` (not an optional chain) of `memo`/`forwardRef`, also as a member
/// (`React.memo(...)`, `React[memo](...)`).
fn is_wrapper_call(node: P<Node>) -> bool {
    if !ast::is_call_expression(node) || is_estree_chain_expression(node) {
        return false;
    }
    let Some(callee) = node.expression().map(skip_parens) else {
        return false;
    };
    let name = if ast::is_identifier(callee) {
        Some(callee)
    } else if is_estree_chain_expression(callee) {
        None
    } else if ast::is_property_access_expression(callee) {
        callee.name()
    } else if ast::is_element_access_expression(callee) {
        Some(skip_parens(
            callee.as_element_access_expression().argument_expression(),
        ))
    } else {
        None
    };
    name.and_then(identifier_name)
        .is_some_and(|name| name == "memo" || name == "forwardRef")
}

fn is_jsx(node: P<Node>) -> bool {
    ast::is_jsx_element(node)
        || ast::is_jsx_self_closing_element(node)
        || ast::is_jsx_fragment(node)
}

fn returns_jsx(node: P<Node>) -> bool {
    is_jsx(node)
        || (ast::is_array_literal_expression(node)
            && node
                .elements()
                .iter()
                .any(|&element| is_jsx(skip_parens(element))))
}

/// `(...) => (...) => <jsx />`
fn is_hoc_initializer(node: P<Node>) -> bool {
    if !ast::is_arrow_function(node) {
        return false;
    }
    node.body()
        .map(skip_parens)
        .filter(|&inner| ast::is_arrow_function(inner))
        .and_then(|inner| inner.body())
        .is_some_and(|body| returns_jsx(skip_parens(body)))
}

#[derive(Default)]
struct Components {
    names: FxHashSet<&'static str>,
    alias_of: FxHashMap<&'static str, &'static str>,
}

impl Components {
    fn collect_declarator(&mut self, declarator: P<Node>) {
        let Some(name) = declarator.name().and_then(identifier_name) else {
            return;
        };
        let Some(init) = declarator.initializer().map(skip_parens) else {
            return;
        };
        let pascal = is_pascal_case(name);
        if pascal
            && (ast::is_arrow_function(init)
                || ast::is_function_expression(init)
                || is_wrapper_call(init))
        {
            self.names.insert(name);
            return;
        }
        if let Some(aliased) = identifier_name(init) {
            self.alias_of.insert(name, aliased);
        }
        if !pascal && is_hoc_initializer(init) {
            self.names.insert(name);
        }
    }

    fn resolves(&self, name: &str) -> bool {
        let mut seen = FxHashSet::default();
        let mut current = name;
        loop {
            if self.names.contains(current) {
                return true;
            }
            if !seen.insert(current) {
                return false;
            }
            let Some(&aliased) = self.alias_of.get(current) else {
                return false;
            };
            current = aliased;
        }
    }
}

fn declarators(statement: P<Node>) -> &'static [P<Node>] {
    statement
        .as_variable_statement()
        .declaration_list()
        .as_variable_declaration_list()
        .declarations()
        .nodes()
}

enum ExportModifier {
    None,
    Named(P<Node>),
    Default,
}

/// How ESTree wraps a declaration with modifiers: typescript-estree turns it into an `ExportNamedDeclaration` or
/// `ExportDefaultDeclaration` only when its first non-decorator modifier is `export`.
fn export_modifier(statement: P<Node>) -> ExportModifier {
    let mut modifiers = statement
        .modifier_nodes()
        .iter()
        .filter(|modifier| modifier.kind() != Kind::Decorator);
    let Some(&first) = modifiers.next() else {
        return ExportModifier::None;
    };
    if first.kind() != Kind::ExportKeyword {
        return ExportModifier::None;
    }
    if modifiers
        .next()
        .is_some_and(|modifier| modifier.kind() == Kind::DefaultKeyword)
    {
        ExportModifier::Default
    } else {
        ExportModifier::Named(first)
    }
}

fn is_function_with_body(node: P<Node>) -> bool {
    ast::is_function_declaration(node) && node.body().is_some()
}

struct ExportEntry {
    range: (i32, i32),
    is_type: bool,
    is_component: bool,
}

fn check_exports(ctx: &mut Ctx) {
    let statements = ctx.file.statements().nodes();
    let mut components = Components::default();
    for &statement in statements {
        if ast::is_variable_statement(statement) {
            for &declarator in declarators(statement) {
                components.collect_declarator(declarator);
            }
        } else if is_function_with_body(statement)
            && !matches!(export_modifier(statement), ExportModifier::Default)
        {
            if let Some(name) = statement.name().and_then(identifier_name) {
                if is_pascal_case(name) {
                    components.names.insert(name);
                }
            }
        }
    }

    let text = ctx.text();
    let mut exports = Vec::new();
    for &statement in statements {
        if ast::is_export_declaration(statement) {
            let export = statement.as_export_declaration();
            let Some(clause) = export.export_clause() else {
                continue;
            };
            if clause.kind() != Kind::NamedExports {
                continue;
            }
            for &specifier in clause.elements() {
                let Some(local) = specifier.property_name_or_name().and_then(identifier_name)
                else {
                    continue;
                };
                exports.push(ExportEntry {
                    range: ctx.trim(specifier),
                    is_type: export.is_type_only(),
                    is_component: components.resolves(local),
                });
            }
            continue;
        }
        if ast::is_export_assignment(statement) {
            let assignment = statement.as_export_assignment();
            if assignment.is_export_equals() {
                continue;
            }
            let expression = skip_parens(assignment.expression());
            let is_component = if let Some(name) = identifier_name(expression) {
                components.resolves(name) || is_pascal_case(name)
            } else {
                ast::is_arrow_function(expression)
                    || ast::is_function_expression(expression)
                    || (ast::is_call_expression(expression)
                        && !is_estree_chain_expression(expression))
            };
            exports.push(ExportEntry {
                range: ctx.trim(statement),
                is_type: false,
                is_component,
            });
            continue;
        }

        let statement_end = ctx.trim(statement).1;
        match export_modifier(statement) {
            ExportModifier::None => {}
            ExportModifier::Default => exports.push(ExportEntry {
                range: ctx.trim(statement),
                is_type: false,
                is_component: is_function_with_body(statement),
            }),
            ExportModifier::Named(export_keyword) => {
                let start = u32::try_from(ctx.trim(export_keyword).1)
                    .map(|end| tsrs_scanner::skip_trivia(text, end))
                    .ok()
                    .and_then(|start| i32::try_from(start).ok())
                    .unwrap_or(statement_end);
                let range = (start, statement_end);
                if ast::is_interface_declaration(statement)
                    || ast::is_type_alias_declaration(statement)
                {
                    exports.push(ExportEntry {
                        range,
                        is_type: true,
                        is_component: false,
                    });
                    continue;
                }
                let is_type = statement
                    .modifier_flags()
                    .contains(ast::ModifierFlags::Ambient);
                if ast::is_variable_statement(statement) {
                    for &declarator in declarators(statement) {
                        let Some(name) = declarator.name().and_then(identifier_name) else {
                            continue;
                        };
                        exports.push(ExportEntry {
                            range: ctx.trim(declarator),
                            is_type,
                            is_component: components.resolves(name),
                        });
                    }
                    continue;
                }
                let is_component = is_function_with_body(statement)
                    && statement
                        .name()
                        .and_then(identifier_name)
                        .is_some_and(|name| components.resolves(name) || is_pascal_case(name));
                exports.push(ExportEntry {
                    range,
                    is_type,
                    is_component,
                });
            }
        }
    }

    let value_exports: Vec<&ExportEntry> = exports.iter().filter(|entry| !entry.is_type).collect();
    let Some(first_component) = value_exports.iter().position(|entry| entry.is_component) else {
        return;
    };
    if let Some((_, entry)) = value_exports
        .iter()
        .enumerate()
        .find(|&(index, _)| index != first_component)
    {
        ctx.report_range(
            entry.range.0,
            entry.range.1,
            RuleMessage::new(
                "multipleExports",
                "React component files should have only one export to support fast refresh, if you want to export multiple components, use a separate file for each component. Type-only exports are allowed.",
            ),
        );
    }
}
