use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::ReferenceIndex;
use crate::utils::{estree_identifier_key, estree_parent, is_jsx_name, skip_parens};

pub const NAME: &str = "no-unused-obj-props";

pub struct NoUnusedObjProps;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<NoOptions>(options)?;
    Ok(Box::new(NoUnusedObjProps))
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct NoOptions {}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VariableDeclaration)];

impl Rule for NoUnusedObjProps {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { index: None })
    }
}

struct Visitor {
    index: Option<ReferenceIndex>,
}

/// `export const obj = ...`: ESTree wraps the declaration in an `ExportNamedDeclaration`.
fn is_exported_declarator(declarator: P<Node>) -> bool {
    declarator
        .parent()
        .and_then(|list| list.parent())
        .is_some_and(|statement| {
            statement.kind() == Kind::VariableStatement
                && ast::has_syntactic_modifier(statement, ModifierFlags::Export)
        })
}

/// ESTree `Property` nodes of an object literal: everything but spread elements.
fn is_estree_property(property: P<Node>) -> bool {
    property.kind() != Kind::SpreadAssignment
}

/// A property whose ESTree value is a `FunctionExpression`, which may use `this`: a method, an accessor or
/// `a: function () {}`.
fn has_function_expression_value(property: P<Node>) -> bool {
    match property.kind() {
        Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor => true,
        Kind::PropertyAssignment => property
            .initializer()
            .is_some_and(|value| skip_parens(value).kind() == Kind::FunctionExpression),
        _ => false,
    }
}

/// The property name a member access on the object reads, or `None` when it isn't a trackable access.
fn accessed_property(member: P<Node>) -> Option<&'static str> {
    match member.kind() {
        Kind::PropertyAccessExpression => {
            let name = member.as_property_access_expression().name;
            (name.kind() == Kind::Identifier).then(|| name.text())
        }
        Kind::ElementAccessExpression => {
            let argument = skip_parens(member.as_element_access_expression().argument_expression);
            (argument.kind() == Kind::StringLiteral).then(|| argument.text())
        }
        _ => None,
    }
}

fn member_object(member: P<Node>) -> Option<P<Node>> {
    match member.kind() {
        Kind::PropertyAccessExpression => Some(member.as_property_access_expression().expression),
        Kind::ElementAccessExpression => Some(member.as_element_access_expression().expression),
        _ => None,
    }
    .map(skip_parens)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(name) = node.name().filter(|name| name.kind() == Kind::Identifier) else {
            return;
        };
        let Some(object) = node
            .initializer()
            .map(skip_parens)
            .filter(|init| init.kind() == Kind::ObjectLiteralExpression)
        else {
            return;
        };
        if node.type_node().is_some() || is_exported_declarator(node) {
            return;
        }

        let mut properties: Vec<(&'static str, P<Node>)> = Vec::new();
        for &property in object.properties() {
            if !is_estree_property(property) {
                continue;
            }
            let Some(key) = property.name().and_then(estree_identifier_key) else {
                continue;
            };
            match properties.iter_mut().find(|(name, _)| *name == key) {
                Some(entry) => entry.1 = property,
                None => properties.push((key, property)),
            }
        }
        if properties.is_empty() {
            return;
        }

        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        let references: Vec<_> = index
            .references_of_binding(ctx.checker, name)
            .into_iter()
            .filter(|reference| !reference.init)
            .collect();
        if references.is_empty() {
            return;
        }

        let has_methods_using_this = object
            .properties()
            .iter()
            .any(|&property| is_estree_property(property) && has_function_expression_value(property));

        let mut used: FxHashSet<&'static str> = FxHashSet::default();
        for reference in references {
            let identifier = reference.identifier;
            if is_jsx_name(identifier) {
                return;
            }
            let Some(member) = estree_parent(identifier) else {
                return;
            };
            if member_object(member) != Some(identifier) {
                return;
            }
            let Some(property) = accessed_property(member) else {
                return;
            };
            used.insert(property);
            if has_methods_using_this
                && estree_parent(member).is_some_and(|parent| parent.kind() == Kind::CallExpression)
            {
                return;
            }
        }

        for (key, property) in properties {
            if !used.contains(key) {
                ctx.report_node(
                    property,
                    RuleMessage::new(
                        "unusedObjectProperty",
                        format!("The object property \"{key}\" is not being used"),
                    ),
                );
            }
        }
    }
}
