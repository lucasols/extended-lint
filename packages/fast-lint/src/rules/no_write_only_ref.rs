use rustc_hash::FxHashSet;
use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::ReferenceIndex;
use crate::utils::{estree_parent, is_estree_chain_expression, skip_parens};

pub const NAME: &str = "no-write-only-ref";

pub struct NoWriteOnlyRef;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<NoOptions>(options)?;
    Ok(Box::new(NoWriteOnlyRef))
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct NoOptions {}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VariableDeclaration)];

impl Rule for NoWriteOnlyRef {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            use_ref_names: react_use_ref_imports(ctx.file.as_node()),
            index: None,
        })
    }
}

struct Visitor {
    /// Local names of `useRef` imported from `react`.
    use_ref_names: FxHashSet<&'static str>,
    index: Option<ReferenceIndex>,
}

fn react_use_ref_imports(file: P<Node>) -> FxHashSet<&'static str> {
    let mut names = FxHashSet::default();
    for &statement in file.statements() {
        if statement.kind() != Kind::ImportDeclaration {
            continue;
        }
        let declaration = statement.as_import_declaration();
        let specifier = declaration.module_specifier;
        if specifier.kind() != Kind::StringLiteral || specifier.text() != "react" {
            continue;
        }
        let Some(bindings) = declaration
            .import_clause()
            .and_then(|clause| clause.as_import_clause().named_bindings())
        else {
            continue;
        };
        if bindings.kind() != Kind::NamedImports {
            continue;
        }
        for &element in bindings.elements() {
            let import = element.as_import_specifier();
            let imported = import.property_name().unwrap_or(import.name());
            if imported.kind() == Kind::Identifier && imported.text() == "useRef" {
                names.insert(import.name().text());
            }
        }
    }
    names
}

impl Visitor {
    /// `useRef(...)` with an imported `useRef`, or `X.useRef(...)`.
    fn is_use_ref_call(&self, node: P<Node>) -> bool {
        if node.kind() != Kind::CallExpression || is_estree_chain_expression(node) {
            return false;
        }
        let callee = skip_parens(node.as_call_expression().expression);
        match callee.kind() {
            Kind::Identifier => self.use_ref_names.contains(callee.text()),
            Kind::PropertyAccessExpression => {
                let access = callee.as_property_access_expression();
                skip_parens(access.expression).kind() == Kind::Identifier
                    && access.name.kind() == Kind::Identifier
                    && access.name.text() == "useRef"
            }
            _ => false,
        }
    }
}

/// `<div ref={identifier} />`: passing the ref to a `ref` prop doesn't read it.
fn is_ref_prop_value(identifier: P<Node>) -> bool {
    let Some(container) = estree_parent(identifier) else {
        return false;
    };
    if container.kind() != Kind::JsxExpression {
        return false;
    }
    container.parent().is_some_and(|attribute| {
        attribute.kind() == Kind::JsxAttribute
            && attribute
                .name()
                .is_some_and(|name| name.kind() == Kind::Identifier && name.text() == "ref")
    })
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(name) = node.name().filter(|name| name.kind() == Kind::Identifier) else {
            return;
        };
        let Some(init) = node.initializer() else {
            return;
        };
        if !self.is_use_ref_call(skip_parens(init)) {
            return;
        }
        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        let is_read = index
            .references_of_binding(ctx.checker, name)
            .iter()
            .any(|reference| !reference.init && !is_ref_prop_value(reference.identifier));
        if is_read {
            return;
        }
        ctx.report_node(
            name,
            RuleMessage::new(
                "refNotRead",
                format!(
                    "Ref \"{}\" is never read. Consider removing it if not needed.",
                    name.text()
                ),
            ),
        );
    }
}
