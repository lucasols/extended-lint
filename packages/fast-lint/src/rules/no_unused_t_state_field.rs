use rustc_hash::FxHashSet;
use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::{declared_names, ReferenceIndex};
use crate::utils::{
    estree_parent, estree_property_function, estree_property_key, returned_object_literal,
    skip_parens,
};

pub const NAME: &str = "no-unused-t-state-field";

pub struct NoUnusedTStateField;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<NoOptions>(options)?;
    Ok(Box::new(NoUnusedTStateField))
}

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct NoOptions {}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for NoUnusedTStateField {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let file_name = ctx.file.file_name();
        let is_jsx_file = file_name.ends_with(".jsx") || file_name.ends_with(".tsx");
        Box::new(Visitor {
            done: !is_jsx_file || !imports_t_state_form(ctx.file.as_node()),
            declared_fields: None,
            index: None,
        })
    }
}

struct Visitor {
    done: bool,
    /// The fields of the first `useForm({ initialConfig })`, as `(name, property)` in declaration order.
    declared_fields: Option<Vec<(&'static str, P<Node>)>>,
    index: Option<ReferenceIndex>,
}

fn imports_t_state_form(file: P<Node>) -> bool {
    file.statements().iter().any(|&statement| {
        statement.kind() == Kind::ImportDeclaration
            && statement.module_specifier().is_some_and(|specifier| {
                specifier.kind() == Kind::StringLiteral && specifier.text() == "t-state-form"
            })
    })
}

fn is_call_of(call: P<Node>, name: &str) -> bool {
    let callee = skip_parens(call.as_call_expression().expression);
    callee.kind() == Kind::Identifier && callee.text() == name
}

/// The fields of `useForm({ initialConfig: { ... } })`, where `initialConfig` may also be a function returning the
/// object.
fn declared_fields(call: P<Node>) -> Option<Vec<(&'static str, P<Node>)>> {
    if !is_call_of(call, "useForm") {
        return None;
    }
    let options = skip_parens(*call.arguments().first()?);
    if options.kind() != Kind::ObjectLiteralExpression {
        return None;
    }
    let initial_config = options.properties().iter().find_map(|&property| {
        if estree_property_key(property) != Some("initialConfig") {
            return None;
        }
        if property.kind() == Kind::PropertyAssignment {
            let value = skip_parens(property.initializer()?);
            if value.kind() == Kind::ObjectLiteralExpression {
                return Some(value);
            }
        }
        returned_object_literal(estree_property_function(property)?)
    })?;
    let mut fields: Vec<(&'static str, P<Node>)> = Vec::new();
    for &property in initial_config.properties() {
        let Some(name) = estree_property_key(property) else {
            continue;
        };
        match fields.iter_mut().find(|(field, _)| *field == name) {
            Some(field) => field.1 = property,
            None => fields.push((name, property)),
        }
    }
    Some(fields)
}

/// The `formFields` binding of `const { formFields } = useFormState(...)` (or its alias in
/// `{ formFields: fields }`).
fn form_fields_binding(declaration: P<Node>) -> Option<P<Node>> {
    declared_names(declaration).into_iter().find(|&name| {
        name.text() == "formFields"
            || name.parent().is_some_and(|element| {
                element.kind() == Kind::BindingElement
                    && element.initializer().is_none()
                    && element.property_name().is_some()
                    && estree_property_key(element) == Some("formFields")
            })
    })
}

fn is_jsx_tag_name(identifier: P<Node>) -> bool {
    let mut current = identifier;
    while let Some(parent) = current.parent() {
        match parent.kind() {
            Kind::PropertyAccessExpression if parent.expression() == Some(current) => {
                current = parent;
            }
            Kind::JsxOpeningElement | Kind::JsxSelfClosingElement | Kind::JsxClosingElement => {
                return parent.tag_name() == current;
            }
            _ => return false,
        }
    }
    false
}

fn is_in_returned_object(parent: P<Node>) -> bool {
    matches!(
        parent.kind(),
        Kind::PropertyAssignment | Kind::ShorthandPropertyAssignment
    ) && parent
        .parent()
        .filter(|object| object.kind() == Kind::ObjectLiteralExpression)
        .and_then(estree_parent)
        .is_some_and(|statement| statement.kind() == Kind::ReturnStatement)
}

/// The fields read as `formFields.<field>` from `const { formFields } = useFormState(...)`; `None` when the call
/// isn't one or `formFields` escapes (returned) or is accessed in a way that can't be followed.
fn used_fields(
    ctx: &mut Ctx,
    index: &ReferenceIndex,
    call: P<Node>,
) -> Option<FxHashSet<&'static str>> {
    if !is_call_of(call, "useFormState") {
        return None;
    }
    let declaration =
        estree_parent(call).filter(|parent| parent.kind() == Kind::VariableDeclaration)?;
    let pattern = declaration
        .name()
        .filter(|name| name.kind() == Kind::ObjectBindingPattern)?;
    if !pattern
        .elements()
        .iter()
        .any(|&element| estree_property_key(element) == Some("formFields"))
    {
        return None;
    }
    let references = match form_fields_binding(declaration) {
        Some(binding) => index.references_of_binding(ctx.checker, binding),
        None => Vec::new(),
    };
    let mut used = FxHashSet::default();
    for reference in references.iter().filter(|reference| !reference.init) {
        let identifier = reference.identifier;
        if is_jsx_tag_name(identifier) {
            return None;
        }
        let Some(parent) = estree_parent(identifier) else {
            continue;
        };
        if is_in_returned_object(parent) || parent.kind() == Kind::ReturnStatement {
            return None;
        }
        let property = match parent.kind() {
            Kind::PropertyAccessExpression => parent.as_property_access_expression().name(),
            Kind::ElementAccessExpression => {
                skip_parens(parent.as_element_access_expression().argument_expression())
            }
            _ => continue,
        };
        let object = skip_parens(parent.expression()?);
        if object.kind() != Kind::Identifier || property.kind() != Kind::Identifier {
            return None;
        }
        used.insert(property.text());
    }
    Some(used)
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if self.done {
            return;
        }
        let Some(fields) = &self.declared_fields else {
            self.declared_fields = declared_fields(node);
            return;
        };
        if !is_call_of(node, "useFormState") {
            return;
        }
        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        let Some(used) = used_fields(ctx, index, node) else {
            return;
        };
        self.done = true;
        for &(name, property) in fields {
            if used.contains(name) {
                continue;
            }
            ctx.report_node(
                property,
                RuleMessage::new(
                    "unusedField",
                    format!("Field \"{name}\" is not being used, you can safely remove it"),
                ),
            );
        }
    }
}
