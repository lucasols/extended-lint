use regex::Regex;
use tsrs_ast::{Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::{binding_symbol, ReferenceIndex};
use crate::utils::{is_estree_chain_expression, skip_parens};

pub const NAME: &str = "react-no-module-level-single-use-values";

const LONG_STRING_LENGTH_MIN: usize = 80;

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    ignore_const_case_var_regex: Option<String>,
}

pub struct ReactNoModuleLevelSingleUseValues {
    ignore_const_case_var_pattern: Option<Regex>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let ignore_const_case_var_pattern = options
        .ignore_const_case_var_regex
        .filter(|pattern| !pattern.is_empty())
        .and_then(|pattern| Regex::new(&pattern).ok());
    Ok(Box::new(ReactNoModuleLevelSingleUseValues {
        ignore_const_case_var_pattern,
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VariableDeclaration)];

impl Rule for ReactNoModuleLevelSingleUseValues {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let file_name = ctx.file.file_name();
        let components = if file_name.ends_with(".tsx") || file_name.ends_with(".jsx") {
            top_level_components(ctx)
        } else {
            Vec::new()
        };
        Box::new(Visitor {
            rule: self,
            components,
            references: None,
        })
    }
}

struct Component {
    name: &'static str,
    pos: i32,
    end: i32,
}

struct Visitor {
    rule: &'static ReactNoModuleLevelSingleUseValues,
    components: Vec<Component>,
    references: Option<ReferenceIndex>,
}

fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase()) && chars.all(|c| c.is_ascii_alphanumeric())
}

fn is_constant_case(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// The PascalCase functions declared at the top level (also exported, but not `export default`) and the PascalCase
/// variables initialized with a function, or with a function wrapped in `memo`/`forwardRef`. The first declaration of
/// a name wins.
fn top_level_components(ctx: &Ctx) -> Vec<Component> {
    let mut components: Vec<Component> = Vec::new();
    let mut add = |name: &'static str, function: P<Node>| {
        if !is_pascal_case(name) || components.iter().any(|c| c.name == name) {
            return;
        }
        let (pos, end) = ctx.trim(function);
        components.push(Component { name, pos, end });
    };
    for &statement in ctx.file.statements().nodes() {
        match statement.kind() {
            Kind::VariableStatement => {
                let list = statement.as_variable_statement().declaration_list();
                for &declaration in list.as_variable_declaration_list().declarations().nodes() {
                    let Some(name) = declaration
                        .name()
                        .filter(|name| name.kind() == Kind::Identifier)
                    else {
                        continue;
                    };
                    if let Some(function) = declaration.initializer().and_then(component_function) {
                        add(name.text(), function);
                    }
                }
            }
            Kind::FunctionDeclaration => {
                if statement.body().is_none()
                    || statement
                        .modifier_flags()
                        .intersects(ModifierFlags::Default)
                {
                    continue;
                }
                if let Some(name) = statement.name() {
                    add(name.text(), statement);
                }
            }
            _ => {}
        }
    }
    components
}

/// ESTree `TSAsExpression`, `TSSatisfiesExpression` and `TSNonNullExpression` unwrapping (parentheses have no ESTree
/// node).
fn unwrap_expression(mut node: P<Node>) -> P<Node> {
    loop {
        node = skip_parens(node);
        match node.kind() {
            Kind::AsExpression | Kind::SatisfiesExpression | Kind::NonNullExpression => {
                let Some(inner) = node.expression() else {
                    return node;
                };
                node = inner;
            }
            _ => return node,
        }
    }
}

fn is_function(node: P<Node>) -> bool {
    matches!(node.kind(), Kind::ArrowFunction | Kind::FunctionExpression)
}

fn is_plain_call(node: P<Node>) -> bool {
    node.kind() == Kind::CallExpression && !is_estree_chain_expression(node)
}

fn component_function(initializer: P<Node>) -> Option<P<Node>> {
    let node = unwrap_expression(initializer);
    if is_function(node) {
        return Some(node);
    }
    if !is_plain_call(node) || !node.expression().is_some_and(is_component_wrapper_callee) {
        return None;
    }
    let first = skip_parens(*node.arguments().first()?);
    if first.kind() == Kind::SpreadElement {
        return None;
    }
    if is_plain_call(first) {
        return component_function(first);
    }
    let first = unwrap_expression(first);
    is_function(first).then_some(first)
}

fn is_component_wrapper_callee(callee: P<Node>) -> bool {
    let callee = skip_parens(callee);
    let name = match callee.kind() {
        Kind::Identifier => callee,
        Kind::PropertyAccessExpression => callee.as_property_access_expression().name(),
        Kind::ElementAccessExpression => {
            skip_parens(callee.as_element_access_expression().argument_expression())
        }
        _ => return false,
    };
    name.kind() == Kind::Identifier && matches!(name.text(), "memo" | "forwardRef")
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Arrays, objects, classes, `new Set`/`new Map` and strings of 80 characters or more, but not aliases of other
/// values (`a`, `a.b`).
fn is_target_initializer(initializer: P<Node>) -> bool {
    let node = unwrap_expression(initializer);
    match node.kind() {
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => {
            utf16_len(node.text()) >= LONG_STRING_LENGTH_MIN
        }
        Kind::ArrayLiteralExpression | Kind::ObjectLiteralExpression | Kind::ClassExpression => {
            true
        }
        Kind::NewExpression => node.expression().is_some_and(|callee| {
            let callee = skip_parens(callee);
            let name = match callee.kind() {
                Kind::Identifier => callee,
                Kind::PropertyAccessExpression => callee.as_property_access_expression().name(),
                _ => return false,
            };
            name.kind() == Kind::Identifier && matches!(name.text(), "Set" | "Map")
        }),
        _ => false,
    }
}

fn is_module_level_declarator(declaration: P<Node>) -> bool {
    declaration
        .parent()
        .and_then(|list| list.parent())
        .is_some_and(|statement| {
            statement.kind() == Kind::VariableStatement
                && !statement.modifier_flags().intersects(ModifierFlags::Export)
                && statement
                    .parent()
                    .is_some_and(|parent| parent.kind() == Kind::SourceFile)
        })
}

impl Visitor {
    /// The innermost component whose function contains `[pos, end)`.
    fn containing_component(&self, pos: i32, end: i32) -> Option<&'static str> {
        let mut smallest: Option<&Component> = None;
        for component in &self.components {
            if pos < component.pos || end > component.end {
                continue;
            }
            if smallest.is_none_or(|s| component.end - component.pos < s.end - s.pos) {
                smallest = Some(component);
            }
        }
        smallest.map(|component| component.name)
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        if self.components.is_empty() {
            &[]
        } else {
            LISTENERS
        }
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(name) = node.name().filter(|name| name.kind() == Kind::Identifier) else {
            return;
        };
        if !is_module_level_declarator(node) {
            return;
        }
        if !node.initializer().is_some_and(is_target_initializer) {
            return;
        }
        let value_name = name.text();
        if self
            .rule
            .ignore_const_case_var_pattern
            .as_ref()
            .is_some_and(|pattern| is_constant_case(value_name) && pattern.is_match(value_name))
        {
            return;
        }
        let Some(symbol) = binding_symbol(ctx.checker, name) else {
            return;
        };
        let references = self
            .references
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file))
            .references(ctx.checker, symbol);

        let mut component_name: Option<&'static str> = None;
        let mut has_usage = false;
        for reference in references.iter().filter(|reference| !reference.init) {
            has_usage = true;
            let (pos, end) = ctx.trim(reference.identifier);
            let Some(component) = self.containing_component(pos, end) else {
                return;
            };
            if component_name.is_some_and(|name| name != component) {
                return;
            }
            component_name = Some(component);
        }
        let Some(component_name) = component_name.filter(|_| has_usage) else {
            return;
        };
        ctx.report_node(
            name,
            RuleMessage::new(
                "moveValueInsideComponent",
                format!(
                    "Module-level `{value_name}` is only used in React component `{component_name}`. Move it inside the component to improve memory usage."
                ),
            ),
        );
    }
}
