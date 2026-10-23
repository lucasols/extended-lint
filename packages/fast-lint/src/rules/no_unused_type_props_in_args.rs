use regex::Regex;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::{resolve_reference, ReferenceIndex};
use crate::utils::{estree_identifier_key, estree_property_key, skip_parens, skip_type_parens};

pub const NAME: &str = "no-unused-type-props-in-args";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    #[serde(rename = "forceCheckOnFCPropTypesWithName")]
    force_check_on_fc_prop_types_with_name: Option<Vec<String>>,
    always_check_function_option_types: Option<bool>,
}

pub struct NoUnusedTypePropsInArgs {
    force_check_on_fc_prop_types_with_name: Vec<Regex>,
    always_check_function_option_types: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let force_check_on_fc_prop_types_with_name = options
        .force_check_on_fc_prop_types_with_name
        .unwrap_or_default()
        .iter()
        .map(|pattern| {
            Regex::new(pattern)
                .map_err(|e| format!("invalid forceCheckOnFCPropTypesWithName pattern: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Box::new(NoUnusedTypePropsInArgs {
        force_check_on_fc_prop_types_with_name,
        always_check_function_option_types: options
            .always_check_function_option_types
            .unwrap_or(true),
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::VariableDeclarationList),
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::ArrowFunction),
];

impl Rule for NoUnusedTypePropsInArgs {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            rule: self,
            index: None,
        })
    }
}

struct Visitor {
    rule: &'static NoUnusedTypePropsInArgs,
    index: Option<ReferenceIndex>,
}

/// Declared properties as `(name, member)` in declaration order; a redeclared name keeps its position and takes the
/// later member.
type DeclaredProperties = Vec<(&'static str, P<Node>)>;

fn insert_property(properties: &mut DeclaredProperties, name: &'static str, member: P<Node>) {
    match properties
        .iter_mut()
        .find(|(existing, _)| *existing == name)
    {
        Some(entry) => entry.1 = member,
        None => properties.push((name, member)),
    }
}

fn extend_from_members(properties: &mut DeclaredProperties, members: &[P<Node>]) {
    for &member in members {
        if member.kind() != Kind::PropertySignature {
            continue;
        }
        if let Some(name) = member.name().and_then(estree_identifier_key) {
            insert_property(properties, name, member);
        }
    }
}

/// Whether the reference `identifier` is in a type position, like typescript-eslint's `reference.isTypeReference`.
fn is_type_reference(identifier: P<Node>) -> bool {
    let mut current = identifier;
    loop {
        let Some(parent) = current.parent() else {
            return false;
        };
        match parent.kind() {
            Kind::QualifiedName if parent.as_qualified_name().left == current => current = parent,
            Kind::PropertyAccessExpression if parent.expression() == Some(current) => {
                current = parent;
            }
            Kind::TypeReference | Kind::ExportSpecifier => return true,
            Kind::ExpressionWithTypeArguments => {
                return parent.parent().is_some_and(|clause| {
                    clause.kind() == Kind::HeritageClause
                        && (clause.as_heritage_clause().token() == Kind::ImplementsKeyword
                            || clause
                                .parent()
                                .is_some_and(|owner| owner.kind() == Kind::InterfaceDeclaration))
                });
            }
            _ => return false,
        }
    }
}

impl Visitor {
    fn type_reference_count(&mut self, ctx: &mut Ctx, type_name: P<Node>) -> usize {
        let Some(symbol) = resolve_reference(ctx.checker, type_name) else {
            return 0;
        };
        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        index
            .references(ctx.checker, symbol)
            .iter()
            .filter(|reference| !reference.init && is_type_reference(reference.identifier))
            .count()
    }

    fn extend_from_type_reference(
        &mut self,
        ctx: &mut Ctx,
        is_fc: bool,
        type_name: P<Node>,
        properties: &mut DeclaredProperties,
        ignore_exported: bool,
        force_check_function_option: bool,
    ) {
        let Some(symbol) = ctx.checker.get_symbol_at_location_exported(type_name) else {
            return;
        };
        if symbol == ctx.checker.unknown_symbol {
            return;
        }
        let Some(&declaration) = symbol.declarations().first() else {
            return;
        };
        if !matches!(
            declaration.kind(),
            Kind::TypeAliasDeclaration | Kind::InterfaceDeclaration
        ) || ast::get_source_file_of_node(declaration) != Some(ctx.file)
        {
            return;
        }
        let force_check = is_fc
            && self
                .rule
                .force_check_on_fc_prop_types_with_name
                .iter()
                .any(|regex| regex.is_match(type_name.text()));
        if !force_check && self.type_reference_count(ctx, type_name) > 1 {
            return;
        }
        let is_exported = ast::has_syntactic_modifier(declaration, ModifierFlags::Export)
            && !ast::has_syntactic_modifier(declaration, ModifierFlags::Default);
        let should_force_check_function_option =
            force_check_function_option && self.rule.always_check_function_option_types;
        if ignore_exported && is_exported && !should_force_check_function_option {
            return;
        }
        if declaration.kind() == Kind::InterfaceDeclaration {
            extend_from_members(properties, declaration.members());
            return;
        }
        if let Some(aliased) = declaration.type_node() {
            self.extend_declared_type_params(
                ctx,
                is_fc,
                properties,
                aliased,
                true,
                force_check_function_option,
            );
        }
    }

    fn extend_declared_type_params(
        &mut self,
        ctx: &mut Ctx,
        is_fc: bool,
        properties: &mut DeclaredProperties,
        type_node: P<Node>,
        ignore_references: bool,
        force_check_function_option: bool,
    ) {
        let type_node = skip_type_parens(type_node);
        match type_node.kind() {
            Kind::TypeLiteral => extend_from_members(properties, type_node.members()),
            Kind::IntersectionType => {
                for &part in type_node.as_intersection_type_node().types().nodes() {
                    self.extend_declared_type_params(
                        ctx,
                        is_fc,
                        properties,
                        part,
                        true,
                        force_check_function_option,
                    );
                }
            }
            Kind::TypeReference if !ignore_references => {
                let type_name = type_node.as_type_reference_node().type_name();
                if type_name.kind() == Kind::Identifier {
                    self.extend_from_type_reference(
                        ctx,
                        is_fc,
                        type_name,
                        properties,
                        true,
                        force_check_function_option,
                    );
                }
            }
            _ => {}
        }
    }

    fn check_params(&mut self, ctx: &mut Ctx, params: &[P<Node>]) {
        for &param in params {
            if param
                .as_parameter_declaration()
                .dot_dot_dot_token()
                .is_some()
            {
                continue;
            }
            let Some(pattern) = param
                .name()
                .filter(|name| name.kind() == Kind::ObjectBindingPattern)
            else {
                continue;
            };
            let Some(annotation) = param.type_node() else {
                continue;
            };
            let mut properties = DeclaredProperties::new();
            self.extend_declared_type_params(
                ctx,
                false,
                &mut properties,
                annotation,
                false,
                self.rule.always_check_function_option_types,
            );
            if !properties.is_empty() {
                check_properties(ctx, pattern, &properties);
            }
        }
    }

    fn check_fc_declaration(&mut self, ctx: &mut Ctx, list: P<Node>) {
        let Some(&declaration) = list
            .as_variable_declaration_list()
            .declarations()
            .nodes()
            .first()
        else {
            return;
        };
        if declaration
            .name()
            .is_none_or(|name| name.kind() != Kind::Identifier)
        {
            return;
        }
        let Some(reference) = declaration.type_node().map(skip_type_parens) else {
            return;
        };
        if reference.kind() != Kind::TypeReference {
            return;
        }
        let type_name = reference.as_type_reference_node().type_name();
        if type_name.kind() != Kind::Identifier || type_name.text() != "FC" {
            return;
        }
        let Some(&props_type) = reference.type_arguments().first() else {
            return;
        };
        let props_type = skip_type_parens(props_type);
        let mut properties = DeclaredProperties::new();
        match props_type.kind() {
            Kind::TypeReference => {
                self.extend_from_fc_type_reference(ctx, props_type, &mut properties)
            }
            Kind::TypeLiteral => extend_from_members(&mut properties, props_type.members()),
            Kind::IntersectionType => {
                for &part in props_type.as_intersection_type_node().types().nodes() {
                    let part = skip_type_parens(part);
                    if part.kind() == Kind::TypeReference
                        && part.as_type_reference_node().type_name().kind() == Kind::Identifier
                    {
                        self.extend_from_fc_type_reference(ctx, part, &mut properties);
                    } else {
                        self.extend_declared_type_params(
                            ctx,
                            true,
                            &mut properties,
                            part,
                            true,
                            false,
                        );
                    }
                }
            }
            _ => {}
        }
        if properties.is_empty() {
            return;
        }
        let Some(function) = declaration
            .initializer()
            .map(skip_parens)
            .filter(|init| init.kind() == Kind::ArrowFunction)
        else {
            return;
        };
        let Some(&first) = function.parameters().first() else {
            ctx.report_node(
                function,
                RuleMessage::new(
                    "missingComponentParam",
                    "Component has declared props but no props are used",
                ),
            );
            return;
        };
        let is_object_pattern = first
            .as_parameter_declaration()
            .dot_dot_dot_token()
            .is_none()
            && first.initializer().is_none();
        if let Some(pattern) = first
            .name()
            .filter(|name| is_object_pattern && name.kind() == Kind::ObjectBindingPattern)
        {
            check_properties(ctx, pattern, &properties);
        }
    }

    fn extend_from_fc_type_reference(
        &mut self,
        ctx: &mut Ctx,
        reference: P<Node>,
        properties: &mut DeclaredProperties,
    ) {
        let type_name = reference.as_type_reference_node().type_name();
        if type_name.kind() == Kind::Identifier {
            self.extend_from_type_reference(ctx, true, type_name, properties, false, false);
        }
    }
}

/// Reports the declared properties that `pattern` doesn't destructure, with a fix that destructures them all.
fn check_properties(ctx: &mut Ctx, pattern: P<Node>, properties: &DeclaredProperties) {
    let elements = pattern.elements();
    if elements
        .last()
        .is_some_and(|last| last.as_binding_element().dot_dot_dot_token().is_some())
    {
        return;
    }
    let destructured: Vec<&'static str> = elements
        .iter()
        .filter_map(|&element| estree_property_key(element))
        .collect();
    let missing: Vec<(&'static str, P<Node>)> = properties
        .iter()
        .copied()
        .filter(|(name, _)| !destructured.contains(name))
        .collect();
    let Some(&(_, fix_target)) = missing.last() else {
        return;
    };
    let names = missing
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ");
    let mut ordered = missing.clone();
    ordered.sort_by_key(|(_, member)| ctx.trim(*member));
    for (name, member) in ordered {
        let message = RuleMessage::new(
            "unusedObjectTypeProperty",
            format!("Object type property '{name}' is defined but never used"),
        );
        if member != fix_target {
            ctx.report_node(member, message);
            continue;
        }
        let names = names.clone();
        ctx.report_node_with_fixes(member, message, |ctx| {
            vec![match elements.last() {
                Some(&last) => ctx.fix_insert_after(last, format!(", {names}")),
                None => {
                    let start = ctx.trim(pattern).0 + 1;
                    ctx.fix_replace_range(start, start, names)
                }
            }]
        });
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::VariableDeclarationList => self.check_fc_declaration(ctx, node),
            Kind::FunctionDeclaration if node.body().is_some() => {
                self.check_params(ctx, node.parameters());
            }
            Kind::ArrowFunction => self.check_params(ctx, node.parameters()),
            _ => {}
        }
    }
}
