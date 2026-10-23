use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::ReferenceIndex;
use crate::utils::{
    estree_parent, estree_property_function, estree_property_key, is_estree_chain_expression,
    returned_object_literal, skip_parens,
};

pub const NAME: &str = "no-unused-selected-values";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    selectors: Vec<SelectorOptions>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SelectorOptions {
    name: String,
    selector_prop: Option<String>,
    selector_arg_pos: Option<f64>,
    return_prop: Option<String>,
}

struct Selector {
    selector_prop: Option<String>,
    /// `None` when `selectorArgPos` isn't a valid argument index.
    selector_arg_pos: Option<usize>,
    return_prop: Option<String>,
}

pub struct NoUnusedSelectedValues {
    selectors: FxHashMap<String, Selector>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut selectors = FxHashMap::default();
    for selector in options.selectors {
        let position = selector.selector_arg_pos.unwrap_or(0.0);
        let selector_arg_pos =
            (position >= 0.0 && position.fract() == 0.0).then_some(position as usize);
        selectors.insert(
            selector.name,
            Selector {
                selector_prop: selector.selector_prop.filter(|prop| !prop.is_empty()),
                selector_arg_pos,
                return_prop: selector.return_prop.filter(|prop| !prop.is_empty()),
            },
        );
    }
    Ok(Box::new(NoUnusedSelectedValues { selectors }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::VariableDeclaration)];

impl Rule for NoUnusedSelectedValues {
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
    rule: &'static NoUnusedSelectedValues,
    index: Option<ReferenceIndex>,
}

/// `name` of `name(...)`, `a.name(...)` or `a[name](...)`.
fn callee_name(call: P<Node>) -> Option<&'static str> {
    let callee = skip_parens(call.as_call_expression().expression);
    let name = match callee.kind() {
        Kind::Identifier => callee,
        Kind::PropertyAccessExpression => callee.as_property_access_expression().name(),
        Kind::ElementAccessExpression => {
            skip_parens(callee.as_element_access_expression().argument_expression())
        }
        _ => return None,
    };
    (name.kind() == Kind::Identifier).then(|| name.text())
}

fn selector_function(call: P<Node>, selector: &Selector) -> Option<P<Node>> {
    let argument = skip_parens(*call.arguments().get(selector.selector_arg_pos?)?);
    if matches!(
        argument.kind(),
        Kind::ArrowFunction | Kind::FunctionExpression
    ) {
        return Some(argument);
    }
    let selector_prop = selector.selector_prop.as_deref()?;
    if argument.kind() != Kind::ObjectLiteralExpression {
        return None;
    }
    let property = argument
        .properties()
        .iter()
        .copied()
        .find(|&property| estree_property_key(property) == Some(selector_prop))?;
    estree_property_function(property)
}

/// The properties of an object literal or object binding pattern by name, `None` when one is a spread or rest
/// element or has a non-identifier key, or there are none.
fn properties_by_name(properties: &[P<Node>]) -> Option<Vec<(&'static str, P<Node>)>> {
    let mut result: Vec<(&'static str, P<Node>)> = Vec::new();
    for &property in properties {
        let name = estree_property_key(property)?;
        match result.iter_mut().find(|(existing, _)| *existing == name) {
            Some(entry) => entry.1 = property,
            None => result.push((name, property)),
        }
    }
    (!result.is_empty()).then_some(result)
}

/// The names destructured from the selected value, or from its `returnProp` property.
fn destructured_names(
    pattern: P<Node>,
    return_prop: Option<&str>,
) -> Option<FxHashSet<&'static str>> {
    let mut elements = pattern.elements();
    if let Some(return_prop) = return_prop {
        let nested = elements
            .iter()
            .copied()
            .find(|&element| estree_property_key(element) == Some(return_prop))?;
        let value = nested
            .name()
            .filter(|value| value.kind() == Kind::ObjectBindingPattern)?;
        if nested.initializer().is_some() {
            return None;
        }
        elements = value.elements();
    }
    let names = properties_by_name(elements)?;
    Some(names.into_iter().map(|(name, _)| name).collect())
}

fn report_unused(
    ctx: &mut Ctx,
    declared: &[(&'static str, P<Node>)],
    used: &FxHashSet<&'static str>,
) {
    for &(name, property) in declared {
        if used.contains(name) {
            continue;
        }
        ctx.report_node(
            property,
            RuleMessage::new(
                "unusedSelectedValue",
                format!("The selected value \"{name}\" is not being used"),
            ),
        );
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(call) = node.initializer().map(skip_parens).filter(|init| {
            init.kind() == Kind::CallExpression && !is_estree_chain_expression(*init)
        }) else {
            return;
        };
        let Some(selector) = callee_name(call).and_then(|name| self.rule.selectors.get(name))
        else {
            return;
        };
        let Some(declared) = selector_function(call, selector)
            .and_then(returned_object_literal)
            .and_then(|object| properties_by_name(object.properties()))
        else {
            return;
        };
        let name = node.name();
        let Some(name) = name else {
            return;
        };
        if name.kind() == Kind::ObjectBindingPattern {
            if let Some(used) = destructured_names(name, selector.return_prop.as_deref()) {
                report_unused(ctx, &declared, &used);
            }
            return;
        }
        if name.kind() != Kind::Identifier || selector.return_prop.is_some() {
            return;
        }
        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        let references = index.references_of_binding(ctx.checker, name);
        let mut used = FxHashSet::default();
        let mut has_references = false;
        for reference in references.iter().filter(|reference| !reference.init) {
            has_references = true;
            let Some(parent) = estree_parent(reference.identifier) else {
                return;
            };
            let property = match parent.kind() {
                Kind::PropertyAccessExpression => parent.as_property_access_expression().name(),
                Kind::ElementAccessExpression => {
                    skip_parens(parent.as_element_access_expression().argument_expression())
                }
                _ => return,
            };
            if property.kind() != Kind::Identifier {
                return;
            }
            used.insert(property.text());
        }
        if has_references {
            report_unused(ctx, &declared, &used);
        }
    }
}
