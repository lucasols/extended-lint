use regex::Regex;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::skip_parens;

pub const NAME: &str = "naming-convention";

const DEFAULT_TYPE_NAMES_DISALLOW: &str = "^[a-z]|_";
const DEFAULT_TYPE_NAMES_MESSAGE: &str = "Type names must be PascalCase";
const DEFAULT_VALUES_DISALLOW: &str = "^__";
const DEFAULT_VALUES_MESSAGE: &str =
    "Variable, function and parameter names must not start with `__`";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CheckOptions {
    disallow: Option<String>,
    message: Option<String>,
}

/// `false` turns a check off; an object (possibly empty) keeps it on with the given overrides.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CheckSetting {
    Enabled(bool),
    Options(CheckOptions),
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    type_names: Option<CheckSetting>,
    values: Option<CheckSetting>,
}

struct Check {
    disallow: Regex,
    message: String,
}

impl Check {
    fn new(
        setting: Option<CheckSetting>,
        default_disallow: &str,
        default_message: &str,
    ) -> Result<Option<Check>, String> {
        let options = match setting {
            None | Some(CheckSetting::Enabled(true)) => CheckOptions::default(),
            Some(CheckSetting::Enabled(false)) => return Ok(None),
            Some(CheckSetting::Options(options)) => options,
        };
        let pattern = options.disallow.as_deref().unwrap_or(default_disallow);
        let disallow =
            Regex::new(pattern).map_err(|e| format!("invalid regex \"{pattern}\": {e}"))?;
        Ok(Some(Check {
            disallow,
            message: options
                .message
                .unwrap_or_else(|| default_message.to_string()),
        }))
    }
}

pub struct NamingConvention {
    type_names: Option<Check>,
    values: Option<Check>,
    listeners: Vec<Listener>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let type_names = Check::new(
        options.type_names,
        DEFAULT_TYPE_NAMES_DISALLOW,
        DEFAULT_TYPE_NAMES_MESSAGE,
    )?;
    let values = Check::new(
        options.values,
        DEFAULT_VALUES_DISALLOW,
        DEFAULT_VALUES_MESSAGE,
    )?;
    let mut listeners = Vec::new();
    if type_names.is_some() {
        listeners.extend(TYPE_NAME_KINDS.iter().map(|&k| Listener::Enter(k)));
    }
    if values.is_some() {
        listeners.extend(VALUE_KINDS.iter().map(|&k| Listener::Enter(k)));
        listeners.push(Listener::AllowPattern(Kind::ObjectLiteralExpression));
        listeners.push(Listener::AllowPattern(Kind::ArrayLiteralExpression));
    }
    Ok(Box::new(NamingConvention {
        type_names,
        values,
        listeners,
    }))
}

const TYPE_NAME_KINDS: &[Kind] = &[
    Kind::ClassDeclaration,
    Kind::ClassExpression,
    Kind::InterfaceDeclaration,
    Kind::TypeAliasDeclaration,
    Kind::EnumDeclaration,
    Kind::TypeParameter,
];

const VALUE_KINDS: &[Kind] = &[
    Kind::VariableDeclaration,
    Kind::FunctionDeclaration,
    Kind::FunctionExpression,
    Kind::Parameter,
    Kind::BindingElement,
];

impl Rule for NamingConvention {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NamingConvention,
}

/// ESLint's `:function`: a function with a body. Bodiless overloads and `declare function`s are `TSDeclareFunction`s
/// and bodiless methods `TSEmptyBodyFunctionExpression`s in ESTree.
fn is_estree_function(node: P<Node>) -> bool {
    match node.kind() {
        Kind::ArrowFunction | Kind::FunctionExpression => true,
        Kind::FunctionDeclaration
        | Kind::MethodDeclaration
        | Kind::Constructor
        | Kind::GetAccessor
        | Kind::SetAccessor => node.body().is_some(),
        _ => false,
    }
}

/// The type parameter of a mapped type (`[K in Keys]`) is not visited by ESLint: ESTree models it as the mapped
/// type's `key` and `constraint`.
fn is_type_name_declaration(node: P<Node>) -> bool {
    node.kind() != Kind::TypeParameter
        || node
            .parent()
            .is_none_or(|parent| parent.kind() != Kind::MappedType)
}

impl Visitor {
    fn check(ctx: &mut Ctx, check: &Check, id: &'static str, name: Option<P<Node>>) {
        let Some(name) = name.map(skip_parens) else {
            return;
        };
        if !ast::is_identifier(name) || !check.disallow.is_match(name.text()) {
            return;
        }
        ctx.report_node(name, RuleMessage::new(id, check.message.clone()));
    }

    fn check_value(ctx: &mut Ctx, check: &Check, name: Option<P<Node>>) {
        Self::check(ctx, check, "valueName", name);
    }

    /// The binding targets of a destructuring assignment (`({ a: __b } = x)`, `[__a = 1] = x`): ESTree turns the
    /// object and array literals into patterns, whose identifiers are declared names for this check.
    fn check_assignment_pattern(ctx: &mut Ctx, check: &Check, node: P<Node>) {
        let targets: &[P<Node>] = match node.kind() {
            Kind::ObjectLiteralExpression => node.properties(),
            _ => node.elements(),
        };
        for &target in targets {
            let value = match target.kind() {
                Kind::PropertyAssignment => target.initializer(),
                Kind::ShorthandPropertyAssignment => target.name(),
                Kind::SpreadAssignment | Kind::SpreadElement => target.expression(),
                _ => Some(target),
            };
            let Some(value) = value.map(skip_parens) else {
                continue;
            };
            let value = if ast::is_assignment_expression(value, true) {
                value.as_binary_expression().left()
            } else {
                value
            };
            Self::check_value(ctx, check, Some(value));
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &self.rule.listeners
    }

    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        if let Listener::AllowPattern(_) = listener {
            if let Some(check) = &self.rule.values {
                Self::check_assignment_pattern(ctx, check, node);
            }
            return;
        }
        let kind = node.kind();
        if TYPE_NAME_KINDS.contains(&kind) {
            if let Some(check) = &self.rule.type_names {
                if is_type_name_declaration(node) {
                    Self::check(ctx, check, "typeName", node.name());
                }
            }
            return;
        }
        let Some(check) = &self.rule.values else {
            return;
        };
        match kind {
            Kind::VariableDeclaration => {
                let in_catch_clause = node
                    .parent()
                    .is_some_and(|parent| parent.kind() == Kind::CatchClause);
                if !in_catch_clause {
                    Self::check_value(ctx, check, node.name());
                }
            }
            Kind::FunctionDeclaration | Kind::FunctionExpression => {
                if is_estree_function(node) {
                    Self::check_value(ctx, check, node.name());
                }
            }
            Kind::Parameter => {
                let parameter = node.as_parameter_declaration();
                let is_plain = parameter.dot_dot_dot_token().is_none()
                    && parameter.initializer().is_none()
                    && !ast::has_syntactic_modifier(node, ModifierFlags::ParameterPropertyModifier);
                if !is_plain || node.parent().is_some_and(is_estree_function) {
                    Self::check_value(ctx, check, Some(parameter.name()));
                }
            }
            Kind::BindingElement => Self::check_value(ctx, check, node.name()),
            _ => {}
        }
    }
}
