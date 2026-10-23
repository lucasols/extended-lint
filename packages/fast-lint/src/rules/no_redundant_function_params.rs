use rustc_hash::FxHashMap;
use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor};
use tsrslint::utils::pos_i32;

use crate::utils::{node_text, skip_parens};

pub const NAME: &str = "no-redundant-function-params";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {
    functions: Vec<FunctionConfig>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FunctionConfig {
    name: String,
    defaults: Vec<serde_json::Value>,
}

pub struct NoRedundantFunctionParams {
    functions: FxHashMap<String, Vec<JsValue>>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let functions = options
        .functions
        .into_iter()
        .map(|config| {
            let defaults = config.defaults.iter().map(JsValue::from_json).collect();
            (config.name, defaults)
        })
        .collect();
    Ok(Box::new(NoRedundantFunctionParams { functions }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

impl Rule for NoRedundantFunctionParams {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoRedundantFunctionParams,
}

/// The JavaScript value the ESLint rule builds for an argument (or reads from the options): primitives, arrays and
/// plain objects (own keys in JavaScript's order), and `Complex` for anything it doesn't evaluate (a unique symbol,
/// never equal to anything).
#[derive(Clone, Debug)]
enum JsValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsValue>),
    Object(Vec<(String, JsValue)>),
    Complex,
}

impl JsValue {
    fn from_json(value: &serde_json::Value) -> JsValue {
        match value {
            serde_json::Value::Null => JsValue::Null,
            serde_json::Value::Bool(b) => JsValue::Bool(*b),
            serde_json::Value::Number(n) => JsValue::Number(n.as_f64().unwrap_or(f64::NAN)),
            serde_json::Value::String(s) => JsValue::String(s.clone()),
            serde_json::Value::Array(items) => {
                JsValue::Array(items.iter().map(JsValue::from_json).collect())
            }
            serde_json::Value::Object(map) => {
                let mut entries: Vec<(String, JsValue)> = map
                    .iter()
                    .map(|(key, value)| (key.clone(), JsValue::from_json(value)))
                    .collect();
                entries.sort_by_key(|(key, _)| array_index(key).map_or((1, 0), |i| (0, i)));
                JsValue::Object(entries)
            }
        }
    }

    fn is_object(&self) -> bool {
        matches!(self, JsValue::Array(_) | JsValue::Object(_))
    }

    /// `Object.keys(value).length`.
    fn key_count(&self) -> usize {
        match self {
            JsValue::Array(items) => items.len(),
            JsValue::Object(entries) => entries.len(),
            _ => 0,
        }
    }

    /// The keys a `for...in` loop visits.
    fn enumerable_keys(&self) -> Vec<String> {
        match self {
            JsValue::Array(items) => (0..items.len()).map(|i| i.to_string()).collect(),
            JsValue::Object(entries) => entries.iter().map(|(key, _)| key.clone()).collect(),
            _ => Vec::new(),
        }
    }

    /// `Object.getOwnPropertyDescriptor(value, key)?.value`: an array also owns its `length`.
    fn own_property(&self, key: &str) -> Option<JsValue> {
        match self {
            JsValue::Array(items) => {
                if key == "length" {
                    return Some(JsValue::Number(items.len() as f64));
                }
                items.get(array_index(key)? as usize).cloned()
            }
            JsValue::Object(entries) => entries
                .iter()
                .find(|(entry_key, _)| entry_key == key)
                .map(|(_, value)| value.clone()),
            _ => None,
        }
    }

    fn set(entries: &mut Vec<(String, JsValue)>, key: &str, value: JsValue) {
        match entries.iter_mut().find(|(entry_key, _)| entry_key == key) {
            Some(entry) => entry.1 = value,
            None => entries.push((key.to_string(), value)),
        }
    }

    /// `JSON.stringify(value)`; `None` where it returns `undefined`.
    fn stringify(&self) -> Option<String> {
        match self {
            JsValue::Undefined | JsValue::Complex => None,
            JsValue::Null => Some("null".to_string()),
            JsValue::Bool(b) => Some(b.to_string()),
            JsValue::Number(n) => Some(if n.is_finite() {
                js_number_to_string(*n)
            } else {
                "null".to_string()
            }),
            JsValue::String(s) => serde_json::to_string(s).ok(),
            JsValue::Array(items) => {
                let parts: Vec<String> = items
                    .iter()
                    .map(|item| item.stringify().unwrap_or_else(|| "null".to_string()))
                    .collect();
                Some(format!("[{}]", parts.join(",")))
            }
            JsValue::Object(entries) => {
                let parts: Vec<String> = entries
                    .iter()
                    .filter_map(|(key, value)| {
                        let value = value.stringify()?;
                        Some(format!("{}:{value}", serde_json::to_string(key).ok()?))
                    })
                    .collect();
                Some(format!("{{{}}}", parts.join(",")))
            }
        }
    }
}

/// The value of `key` as an array index (a canonical integer below 2^32 - 1), which JavaScript orders first.
fn array_index(key: &str) -> Option<u32> {
    let index: u32 = key.parse().ok()?;
    (index != u32::MAX && index.to_string() == key).then_some(index)
}

/// JavaScript's `Number.prototype.toString()`.
fn js_number_to_string(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    let magnitude = n.abs();
    if (1e-6..1e21).contains(&magnitude) {
        return n.to_string();
    }
    let exponential = format!("{n:e}");
    match exponential.split_once('e') {
        Some((mantissa, exponent)) if !exponent.starts_with('-') => {
            format!("{mantissa}e+{exponent}")
        }
        _ => exponential,
    }
}

fn deep_equal(a: &JsValue, b: &JsValue) -> bool {
    match (a, b) {
        (JsValue::Undefined, JsValue::Undefined) | (JsValue::Null, JsValue::Null) => true,
        (JsValue::Bool(a), JsValue::Bool(b)) => a == b,
        (JsValue::Number(a), JsValue::Number(b)) => a == b,
        (JsValue::String(a), JsValue::String(b)) => a == b,
        (JsValue::Array(a), JsValue::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| deep_equal(a, b))
        }
        (JsValue::Object(a), JsValue::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, a_value)| {
                    b.iter()
                        .find(|(b_key, _)| b_key == key)
                        .is_some_and(|(_, b_value)| deep_equal(a_value, b_value))
                })
        }
        _ => false,
    }
}

fn redundant_properties(arg: &JsValue, default: &JsValue) -> Vec<String> {
    if !arg.is_object() || !default.is_object() {
        return Vec::new();
    }
    arg.enumerable_keys()
        .into_iter()
        .filter(
            |key| match (default.own_property(key), arg.own_property(key)) {
                (Some(default_value), Some(arg_value)) => deep_equal(&arg_value, &default_value),
                _ => false,
            },
        )
        .collect()
}

/// The value of an ESTree `Literal`, or `None` when `node` isn't one. A regex literal is a `RegExp` object without
/// own keys.
fn literal_value(node: P<Node>) -> Option<JsValue> {
    Some(match node.kind() {
        Kind::StringLiteral => JsValue::String(node.text().to_string()),
        Kind::NumericLiteral => node
            .text()
            .parse::<f64>()
            .map_or(JsValue::Complex, JsValue::Number),
        Kind::TrueKeyword => JsValue::Bool(true),
        Kind::FalseKeyword => JsValue::Bool(false),
        Kind::NullKeyword => JsValue::Null,
        Kind::RegularExpressionLiteral => JsValue::Object(Vec::new()),
        Kind::BigIntLiteral => JsValue::Complex,
        _ => return None,
    })
}

/// The ESTree `Property` key name of an object literal member with a non-computed identifier key.
fn property_key(property: P<Node>) -> Option<&'static str> {
    if !matches!(
        property.kind(),
        Kind::PropertyAssignment
            | Kind::ShorthandPropertyAssignment
            | Kind::MethodDeclaration
            | Kind::GetAccessor
            | Kind::SetAccessor
    ) {
        return None;
    }
    property
        .name()
        .filter(|name| name.kind() == Kind::Identifier)
        .map(|name| name.text())
}

fn object_value(object: P<Node>) -> JsValue {
    let mut entries = Vec::new();
    for &property in object.properties() {
        let Some(key) = property_key(property) else {
            continue;
        };
        let value = match property.kind() {
            Kind::PropertyAssignment => match property.initializer().map(skip_parens) {
                Some(value) => match literal_value(value) {
                    Some(literal) => literal,
                    None if value.kind() == Kind::ObjectLiteralExpression => object_value(value),
                    None => JsValue::Complex,
                },
                None => JsValue::Complex,
            },
            _ => JsValue::Complex,
        };
        JsValue::set(&mut entries, key, value);
    }
    JsValue::Object(entries)
}

fn argument_value(arg: P<Node>) -> JsValue {
    if let Some(literal) = literal_value(arg) {
        return literal;
    }
    match arg.kind() {
        Kind::ObjectLiteralExpression => object_value(arg),
        Kind::ArrayLiteralExpression => JsValue::Array(
            arg.elements()
                .iter()
                .map(|&element| {
                    let element = skip_parens(element);
                    match element.kind() {
                        Kind::OmittedExpression | Kind::SpreadElement => JsValue::Undefined,
                        _ => literal_value(element).unwrap_or(JsValue::Complex),
                    }
                })
                .collect(),
        ),
        _ => JsValue::Complex,
    }
}

fn function_name(callee: P<Node>) -> Option<&'static str> {
    let callee = skip_parens(callee);
    match callee.kind() {
        Kind::Identifier => Some(callee.text()),
        Kind::PropertyAccessExpression => {
            let name = callee.as_property_access_expression().name();
            (name.kind() == Kind::Identifier).then(|| name.text())
        }
        _ => None,
    }
}

enum Redundancy {
    Simple {
        position: usize,
        value: JsValue,
    },
    Object {
        position: usize,
    },
    PartialObject {
        position: usize,
        properties: Vec<String>,
    },
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(callee) = node.expression() else {
            return;
        };
        let Some(name) = function_name(callee) else {
            return;
        };
        let Some(defaults) = self.rule.functions.get(name) else {
            return;
        };
        let args = node.arguments();
        if args.is_empty() {
            return;
        }

        let mut last_non_redundant: Option<usize> = None;
        let mut partial_objects: Vec<(usize, Vec<String>)> = Vec::new();
        let mut fully_redundant_objects: Vec<usize> = Vec::new();
        let mut details: Vec<Redundancy> = Vec::new();

        for (i, &arg) in args.iter().enumerate() {
            let Some(default) = defaults.get(i) else {
                last_non_redundant = Some(i);
                continue;
            };
            let arg = skip_parens(arg);
            if arg.kind() == Kind::SpreadElement {
                last_non_redundant = Some(i);
                continue;
            }
            let value = argument_value(arg);
            let is_object_literal = arg.kind() == Kind::ObjectLiteralExpression;

            if deep_equal(&value, default) {
                if is_object_literal {
                    fully_redundant_objects.push(i);
                    details.push(Redundancy::Object { position: i + 1 });
                } else {
                    details.push(Redundancy::Simple {
                        position: i + 1,
                        value,
                    });
                }
                continue;
            }

            if is_object_literal
                && arg.properties().is_empty()
                && default.is_object()
                && default.key_count() > 0
            {
                fully_redundant_objects.push(i);
                details.push(Redundancy::Object { position: i + 1 });
                continue;
            }

            let redundant = redundant_properties(&value, default);
            if redundant.is_empty() {
                last_non_redundant = Some(i);
                continue;
            }
            let property_count = if is_object_literal {
                arg.properties().len()
            } else {
                0
            };
            if is_object_literal
                && property_count == redundant.len()
                && property_count == default.key_count()
            {
                fully_redundant_objects.push(i);
                details.push(Redundancy::Object { position: i + 1 });
                continue;
            }
            partial_objects.push((i, redundant.clone()));
            details.push(Redundancy::PartialObject {
                position: i + 1,
                properties: redundant,
            });
            last_non_redundant = Some(i);
        }

        let has_trailing_redundant = last_non_redundant.is_none_or(|i| i + 1 < args.len());
        if !has_trailing_redundant
            && partial_objects.is_empty()
            && fully_redundant_objects.is_empty()
        {
            return;
        }

        let mut parts: Vec<String> = Vec::new();
        for detail in &details {
            match detail {
                Redundancy::Simple { position, value } => parts.push(format!(
                    "Param {position} with value {}",
                    value.stringify().unwrap_or_else(|| "undefined".to_string())
                )),
                Redundancy::Object { position } => parts.push(format!("Param {position} (object)")),
                Redundancy::PartialObject {
                    position,
                    properties,
                } => {
                    let Some(default) = defaults.get(position - 1) else {
                        continue;
                    };
                    for property in properties {
                        if let Some(value) = default.own_property(property) {
                            parts.push(format!(
                                "Param {position} property \"{property}\" with value {}",
                                value.stringify().unwrap_or_else(|| "undefined".to_string())
                            ));
                        }
                    }
                }
            }
        }
        let message = RuleMessage::new(
            "redundantParams",
            format!(
                "Function call has redundant parameters matching its default values and can be removed: {}",
                parts.join(", ")
            ),
        );

        ctx.report_node_with_fixes(node, message, |ctx| {
            let mut fixes: Vec<RuleFix> = Vec::new();
            for (index, properties) in &partial_objects {
                let Some(&arg) = args.get(*index) else {
                    continue;
                };
                let arg = skip_parens(arg);
                if arg.kind() != Kind::ObjectLiteralExpression {
                    continue;
                }
                let kept: Vec<&str> = arg
                    .properties()
                    .iter()
                    .filter(|&&property| {
                        property_key(property)
                            .is_none_or(|key| !properties.iter().any(|p| p == key))
                    })
                    .map(|&property| node_text(ctx, property))
                    .collect();
                fixes.push(ctx.fix_replace(arg, format!("{{ {} }}", kept.join(", "))));
            }

            let rightmost_non_redundant = (0..args.len()).rev().find(|i| {
                !fully_redundant_objects.contains(i)
                    && last_non_redundant.is_some_and(|last| *i <= last)
            });
            let close_paren = pos_i32(node.end()) - 1;
            match rightmost_non_redundant {
                Some(i) if i + 1 >= args.len() => {}
                Some(i) => {
                    if let Some(&arg) = args.get(i) {
                        fixes.push(ctx.fix_remove_range(pos_i32(arg.end()), close_paren));
                    }
                }
                None => {
                    if let Some(list) = node.argument_list() {
                        fixes.push(ctx.fix_remove_range(pos_i32(list.pos()), close_paren));
                    }
                }
            }
            fixes
        });
    }
}
