use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

pub const NAME: &str = "no-default-export";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoDefaultExport;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoDefaultExport))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ExportAssignment),
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::ClassDeclaration),
    Listener::Enter(Kind::InterfaceDeclaration),
    Listener::Enter(Kind::ExportDeclaration),
];

impl Rule for NoDefaultExport {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn message() -> RuleMessage {
    RuleMessage::new(
        "noDefaultExport",
        "Default exports are not allowed, use named exports instead.",
    )
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::ExportAssignment => {
                if !node.as_export_assignment().is_export_equals() {
                    ctx.report_node(node, message());
                }
            }
            Kind::ExportDeclaration => {
                let Some(clause) = node.as_export_declaration().export_clause() else {
                    return;
                };
                if !ast::is_named_exports(clause) {
                    return;
                }
                for &specifier in clause.as_named_exports().elements().nodes() {
                    let exported = specifier.as_export_specifier().name();
                    if ast::is_identifier(exported) && exported.text() == "default" {
                        ctx.report_node(node, message());
                    }
                }
            }
            _ => {
                if ast::has_syntactic_modifier(node, ModifierFlags::Default) {
                    ctx.report_node(node, message());
                }
            }
        }
    }
}
