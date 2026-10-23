use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{is_estree_chain_expression, skip_parens};

pub const NAME: &str = "no-reexport";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoReexport;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoReexport))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ImportDeclaration),
    Listener::Enter(Kind::ExportDeclaration),
    Listener::Enter(Kind::ExportAssignment),
    Listener::Enter(Kind::VariableStatement),
    Listener::Enter(Kind::TypeAliasDeclaration),
];

impl Rule for NoReexport {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            imported: FxHashSet::default(),
        })
    }
}

/// Local names bound by the imports seen so far (in source order, like the ESLint rule).
struct Visitor {
    imported: FxHashSet<&'static str>,
}

fn message() -> RuleMessage {
    RuleMessage::new(
        "noReexport",
        "Re-exports are not allowed. Use direct exports only.",
    )
}

impl Visitor {
    fn track_import(&mut self, node: P<Node>) {
        let Some(clause) = node.as_import_declaration().import_clause() else {
            return;
        };
        let clause = clause.as_import_clause();
        if let Some(name) = clause.name() {
            self.imported.insert(name.text());
        }
        let Some(bindings) = clause.named_bindings() else {
            return;
        };
        if ast::is_namespace_import(bindings) {
            self.imported
                .insert(bindings.as_namespace_import().name().text());
        } else if ast::is_named_imports(bindings) {
            for &specifier in bindings.as_named_imports().elements().nodes() {
                self.imported
                    .insert(specifier.as_import_specifier().name().text());
            }
        }
    }

    /// An imported identifier, or a member access on one (`obj.prop`, `obj['prop']`). An optional chain is an ESTree
    /// `ChainExpression`, which the ESLint rule doesn't look into.
    fn contains_imported_identifier(&self, node: P<Node>) -> bool {
        let node = skip_parens(node);
        if ast::is_identifier(node) {
            return self.imported.contains(node.text());
        }
        if (ast::is_property_access_expression(node) || ast::is_element_access_expression(node))
            && !is_estree_chain_expression(node)
        {
            return node
                .expression()
                .is_some_and(|object| self.contains_imported_identifier(object));
        }
        false
    }

    fn is_imported_type_reference(&self, type_node: P<Node>) -> bool {
        let mut type_node = type_node;
        while ast::is_parenthesized_type_node(type_node) {
            let Some(inner) = type_node.type_node() else {
                return false;
            };
            type_node = inner;
        }
        if !ast::is_type_reference_node(type_node) {
            return false;
        }
        let reference = type_node.as_type_reference_node();
        if reference.type_arguments().is_some() {
            return false;
        }
        let name = reference.type_name();
        ast::is_identifier(name) && self.imported.contains(name.text())
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::ImportDeclaration => self.track_import(node),
            Kind::ExportDeclaration => {
                let declaration = node.as_export_declaration();
                let has_specifiers = declaration.export_clause().is_some_and(|clause| {
                    !ast::is_named_exports(clause)
                        || !clause.as_named_exports().elements().nodes().is_empty()
                });
                if declaration.module_specifier().is_some() || has_specifiers {
                    ctx.report_node(node, message());
                }
            }
            Kind::ExportAssignment => {
                let assignment = node.as_export_assignment();
                if assignment.is_export_equals() {
                    return;
                }
                let expression = skip_parens(assignment.expression());
                if ast::is_identifier(expression) && self.imported.contains(expression.text()) {
                    ctx.report_node(node, message());
                }
            }
            Kind::VariableStatement => {
                if !ast::has_syntactic_modifier(node, ModifierFlags::Export) {
                    return;
                }
                let list = node.as_variable_statement().declaration_list();
                for &declarator in list.as_variable_declaration_list().declarations().nodes() {
                    if declarator
                        .initializer()
                        .is_some_and(|init| self.contains_imported_identifier(init))
                    {
                        ctx.report_node(node, message());
                    }
                }
            }
            Kind::TypeAliasDeclaration
                if ast::has_syntactic_modifier(node, ModifierFlags::Export)
                    && node
                        .type_node()
                        .is_some_and(|type_node| self.is_imported_type_reference(type_node)) =>
            {
                ctx.report_node(node, message());
            }
            _ => {}
        }
    }
}
