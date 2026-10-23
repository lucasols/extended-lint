use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::{binding_symbol, ReferenceIndex};
use crate::utils::{
    estree_parent, is_estree_chain_expression, is_var_in_nested_block, skip_parens,
};

pub const NAME: &str = "require-reads-to-var-prop";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    vars_to_check: Vec<VarCheckOptions>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct VarCheckOptions {
    selector: Option<String>,
    from_fn_call: Option<String>,
    prop: PropOption,
    error_msg: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum PropOption {
    One(String),
    Many(Vec<String>),
}

struct Check {
    props: Vec<String>,
    error_msg: String,
}

/// A `selector` option: `VariableDeclarator` followed by attribute filters, the subset of ESLint selectors supported.
struct Selector {
    raw: String,
    attributes: Vec<Attribute>,
    check: Check,
}

struct Attribute {
    path: Vec<String>,
    test: AttributeTest,
}

enum AttributeTest {
    Exists,
    Equals(String),
    NotEquals(String),
}

pub struct RequireReadsToVarProp {
    selectors: Vec<Selector>,
    /// `*.method` patterns, by method name.
    any_object_methods: Vec<(String, String, Check)>,
    /// `object.method` patterns, by object and method name.
    object_methods: Vec<(String, String, String, Check)>,
    /// Plain function name patterns.
    functions: Vec<(String, Check)>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut rule = RequireReadsToVarProp {
        selectors: Vec::new(),
        any_object_methods: Vec::new(),
        object_methods: Vec::new(),
        functions: Vec::new(),
    };
    let check = |options: &VarCheckOptions| Check {
        props: match &options.prop {
            PropOption::One(prop) => vec![prop.clone()],
            PropOption::Many(props) => props.clone(),
        },
        error_msg: options.error_msg.clone().unwrap_or_default(),
    };
    for options in &options.vars_to_check {
        let Some(raw) = options.selector.as_deref().filter(|s| !s.is_empty()) else {
            continue;
        };
        let attributes = parse_selector(raw)?;
        rule.selectors.retain(|selector| selector.raw != raw);
        rule.selectors.push(Selector {
            raw: raw.to_string(),
            attributes,
            check: check(options),
        });
    }
    rule.selectors.sort_by(|a, b| {
        a.attributes
            .len()
            .cmp(&b.attributes.len())
            .then_with(|| a.raw.cmp(&b.raw))
    });
    for options in &options.vars_to_check {
        let Some(pattern) = options.from_fn_call.as_deref().filter(|s| !s.is_empty()) else {
            continue;
        };
        if let Some(method) = pattern.strip_prefix("*.") {
            rule.any_object_methods.retain(|(m, _, _)| m != method);
            rule.any_object_methods
                .push((method.to_string(), pattern.to_string(), check(options)));
        } else if pattern.contains('.') {
            let mut parts = pattern.split('.');
            let object = parts.next().unwrap_or_default().to_string();
            let method = parts.next().unwrap_or_default().to_string();
            rule.object_methods
                .retain(|(o, m, _, _)| *o != object || *m != method);
            rule.object_methods
                .push((object, method, pattern.to_string(), check(options)));
        } else {
            rule.functions.retain(|(f, _)| f != pattern);
            rule.functions.push((pattern.to_string(), check(options)));
        }
    }
    Ok(Box::new(rule))
}

fn unsupported_selector(selector: &str) -> String {
    format!(
        "unsupported selector \"{selector}\": only `VariableDeclarator` with attribute filters (`[init.callee.name=\"fn\"]`, `!=`, `[path]`) is supported"
    )
}

fn parse_selector(selector: &str) -> Result<Vec<Attribute>, String> {
    let error = || unsupported_selector(selector);
    let mut rest = selector
        .trim()
        .strip_prefix("VariableDeclarator")
        .ok_or_else(error)?;
    let mut attributes = Vec::new();
    while !rest.is_empty() {
        let inner = rest.strip_prefix('[').ok_or_else(error)?;
        let close = find_attribute_end(inner).ok_or_else(error)?;
        let (body, after) = inner.split_at(close);
        rest = after.get(1..).unwrap_or_default();
        attributes.push(parse_attribute(body.trim()).ok_or_else(error)?);
    }
    Ok(attributes)
}

/// The index of the `]` closing an attribute, skipping quoted values.
fn find_attribute_end(text: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, c) in text.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == ']' => return Some(i),
            None => {}
        }
    }
    None
}

fn parse_attribute(body: &str) -> Option<Attribute> {
    let (path, test) = match body.find('=') {
        None => (body, AttributeTest::Exists),
        Some(i) => {
            let (path, negated) = match body.get(..i)?.strip_suffix('!') {
                Some(path) => (path, true),
                None => (body.get(..i)?, false),
            };
            let value = parse_attribute_value(body.get(i + 1..)?.trim())?;
            let test = if negated {
                AttributeTest::NotEquals(value)
            } else {
                AttributeTest::Equals(value)
            };
            (path, test)
        }
    };
    let path: Vec<String> = path.trim().split('.').map(str::to_string).collect();
    let is_identifier = |part: &String| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    };
    path.iter()
        .all(is_identifier)
        .then_some(Attribute { path, test })
}

fn parse_attribute_value(value: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(inner) = value
            .strip_prefix(quote)
            .and_then(|v| v.strip_suffix(quote))
        {
            return Some(inner.to_string());
        }
    }
    let is_plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '-' | '.'));
    is_plain.then(|| value.to_string())
}

/// A node of the ESTree tree as the selector sees it: a TS node, possibly as the `ChainExpression` wrapping it.
#[derive(Clone, Copy)]
struct EsNode {
    node: P<Node>,
    chain: bool,
}

enum EsValue {
    Node(EsNode),
    Array(usize),
    Text(String),
    Undefined,
}

impl EsValue {
    fn expression(node: P<Node>) -> EsValue {
        let node = skip_parens(node);
        EsValue::Node(EsNode {
            node,
            chain: is_estree_chain_expression(node),
        })
    }

    fn text(text: &str) -> EsValue {
        EsValue::Text(text.to_string())
    }

    /// The value as esquery compares it, `${value}`; `None` for `undefined`.
    fn to_js_string(&self) -> Option<String> {
        match self {
            EsValue::Node(_) => Some("[object Object]".to_string()),
            EsValue::Array(_) => Some(String::new()),
            EsValue::Text(text) => Some(text.clone()),
            EsValue::Undefined => None,
        }
    }
}

fn estree_type(es: EsNode) -> &'static str {
    if es.chain {
        return "ChainExpression";
    }
    let node = es.node;
    match node.kind() {
        Kind::Identifier => "Identifier",
        Kind::PrivateIdentifier => "PrivateIdentifier",
        Kind::CallExpression => "CallExpression",
        Kind::NewExpression => "NewExpression",
        Kind::PropertyAccessExpression | Kind::ElementAccessExpression => "MemberExpression",
        Kind::AwaitExpression => "AwaitExpression",
        Kind::ArrowFunction => "ArrowFunctionExpression",
        Kind::FunctionExpression => "FunctionExpression",
        Kind::ClassExpression => "ClassExpression",
        Kind::ObjectLiteralExpression => "ObjectExpression",
        Kind::ArrayLiteralExpression => "ArrayExpression",
        Kind::ObjectBindingPattern => "ObjectPattern",
        Kind::ArrayBindingPattern => "ArrayPattern",
        Kind::StringLiteral
        | Kind::NumericLiteral
        | Kind::BigIntLiteral
        | Kind::RegularExpressionLiteral
        | Kind::TrueKeyword
        | Kind::FalseKeyword
        | Kind::NullKeyword => "Literal",
        Kind::NoSubstitutionTemplateLiteral | Kind::TemplateExpression => "TemplateLiteral",
        Kind::TaggedTemplateExpression => "TaggedTemplateExpression",
        Kind::ConditionalExpression => "ConditionalExpression",
        Kind::BinaryExpression => {
            let operator = node.as_binary_expression().operator_token.kind();
            match operator {
                Kind::AmpersandAmpersandToken | Kind::BarBarToken | Kind::QuestionQuestionToken => {
                    "LogicalExpression"
                }
                Kind::CommaToken => "SequenceExpression",
                _ if tsrs_ast::is_assignment_operator(operator) => "AssignmentExpression",
                _ => "BinaryExpression",
            }
        }
        Kind::PrefixUnaryExpression => match node.as_prefix_unary_expression().operator {
            Kind::PlusPlusToken | Kind::MinusMinusToken => "UpdateExpression",
            _ => "UnaryExpression",
        },
        Kind::PostfixUnaryExpression => "UpdateExpression",
        Kind::TypeOfExpression | Kind::VoidExpression | Kind::DeleteExpression => "UnaryExpression",
        Kind::AsExpression => "TSAsExpression",
        Kind::SatisfiesExpression => "TSSatisfiesExpression",
        Kind::NonNullExpression => "TSNonNullExpression",
        Kind::TypeAssertionExpression => "TSTypeAssertion",
        Kind::ThisKeyword => "ThisExpression",
        Kind::SuperKeyword => "Super",
        Kind::YieldExpression => "YieldExpression",
        Kind::JsxElement | Kind::JsxSelfClosingElement => "JSXElement",
        Kind::JsxFragment => "JSXFragment",
        Kind::MetaProperty => "MetaProperty",
        _ => "",
    }
}

fn literal_text(node: P<Node>) -> Option<String> {
    Some(match node.kind() {
        Kind::StringLiteral => node.text().to_string(),
        Kind::NumericLiteral => node.text().parse::<f64>().ok()?.to_string(),
        Kind::TrueKeyword => "true".to_string(),
        Kind::FalseKeyword => "false".to_string(),
        Kind::NullKeyword => "null".to_string(),
        _ => return None,
    })
}

/// ESTree property `key` of `es` (`type`, `name`, `id`, `init`, `callee`, `object`, `property`, `computed`,
/// `optional`, `arguments`, `argument`, `expression`, `value`); anything else is `undefined`.
fn estree_property(es: EsNode, key: &str) -> EsValue {
    let node = es.node;
    if key == "type" {
        return EsValue::text(estree_type(es));
    }
    if es.chain {
        return match key {
            "expression" => EsValue::Node(EsNode { node, chain: false }),
            _ => EsValue::Undefined,
        };
    }
    match (node.kind(), key) {
        (Kind::Identifier | Kind::PrivateIdentifier, "name") => {
            EsValue::text(node.text().trim_start_matches('#'))
        }
        (Kind::VariableDeclaration, "id") => node.name().map_or(EsValue::Undefined, |name| {
            EsValue::Node(EsNode {
                node: name,
                chain: false,
            })
        }),
        (Kind::VariableDeclaration, "init") => node
            .initializer()
            .map_or(EsValue::Undefined, EsValue::expression),
        (Kind::CallExpression | Kind::NewExpression, "callee") => node
            .expression()
            .map_or(EsValue::Undefined, EsValue::expression),
        (Kind::CallExpression | Kind::NewExpression, "arguments") => {
            EsValue::Array(node.arguments().len())
        }
        (Kind::PropertyAccessExpression | Kind::ElementAccessExpression, "object") => node
            .expression()
            .map_or(EsValue::Undefined, EsValue::expression),
        (Kind::PropertyAccessExpression, "property") => EsValue::Node(EsNode {
            node: node.as_property_access_expression().name(),
            chain: false,
        }),
        (Kind::ElementAccessExpression, "property") => {
            EsValue::expression(node.as_element_access_expression().argument_expression())
        }
        (Kind::PropertyAccessExpression, "computed") => EsValue::text("false"),
        (Kind::ElementAccessExpression, "computed") => EsValue::text("true"),
        (
            Kind::PropertyAccessExpression | Kind::ElementAccessExpression | Kind::CallExpression,
            "optional",
        ) => EsValue::text(if node.question_dot_token().is_some() {
            "true"
        } else {
            "false"
        }),
        (
            Kind::AwaitExpression
            | Kind::SpreadElement
            | Kind::TypeOfExpression
            | Kind::VoidExpression
            | Kind::DeleteExpression,
            "argument",
        ) => node
            .expression()
            .map_or(EsValue::Undefined, EsValue::expression),
        (Kind::PrefixUnaryExpression, "argument") => {
            EsValue::expression(node.as_prefix_unary_expression().operand)
        }
        (
            Kind::AsExpression | Kind::SatisfiesExpression | Kind::NonNullExpression,
            "expression",
        ) => node
            .expression()
            .map_or(EsValue::Undefined, EsValue::expression),
        (_, "value") => literal_text(node).map_or(EsValue::Undefined, EsValue::Text),
        _ => EsValue::Undefined,
    }
}

fn attribute_matches(declarator: P<Node>, attribute: &Attribute) -> bool {
    let mut value = EsValue::Node(EsNode {
        node: declarator,
        chain: false,
    });
    for key in &attribute.path {
        value = match value {
            EsValue::Node(es) => estree_property(es, key),
            EsValue::Array(len) if key == "length" => EsValue::Text(len.to_string()),
            _ => EsValue::Undefined,
        };
    }
    let actual = value.to_js_string();
    match &attribute.test {
        AttributeTest::Exists => actual.is_some(),
        AttributeTest::Equals(expected) => actual.as_deref().unwrap_or("undefined") == expected,
        AttributeTest::NotEquals(expected) => actual.as_deref().unwrap_or("undefined") != expected,
    }
}

/// `getFunctionName` of the ESLint rule: the callee of the declarator's call initializer.
fn function_name(declarator: P<Node>) -> String {
    let unknown = || "unknown function".to_string();
    let Some(init) = declarator.initializer().map(skip_parens) else {
        return unknown();
    };
    if init.kind() != Kind::CallExpression || is_estree_chain_expression(init) {
        return unknown();
    }
    let Some(callee) = init.expression().map(skip_parens) else {
        return unknown();
    };
    if callee.kind() == Kind::Identifier {
        return callee.text().to_string();
    }
    if is_estree_chain_expression(callee) {
        return unknown();
    }
    let (object, property) = match callee.kind() {
        Kind::PropertyAccessExpression => (
            callee.as_property_access_expression().expression(),
            callee.as_property_access_expression().name(),
        ),
        Kind::ElementAccessExpression => (
            callee.as_element_access_expression().expression(),
            skip_parens(callee.as_element_access_expression().argument_expression()),
        ),
        _ => return unknown(),
    };
    if property.kind() != Kind::Identifier {
        return unknown();
    }
    let object = skip_parens(object);
    if object.kind() == Kind::Identifier {
        format!("{}.{}", object.text(), property.text())
    } else {
        property.text().to_string()
    }
}

/// The ESTree `property` of a non-chain member expression, when it is an identifier (also `obj[identifier]`).
fn member_identifier_property(member: P<Node>) -> Option<P<Node>> {
    let property = match member.kind() {
        Kind::PropertyAccessExpression => member.as_property_access_expression().name(),
        Kind::ElementAccessExpression => {
            skip_parens(member.as_element_access_expression().argument_expression())
        }
        _ => return None,
    };
    (property.kind() == Kind::Identifier).then_some(property)
}

/// The keys of an object destructuring pattern's properties that are identifiers (also `[identifier]`), skipping
/// the rest element.
fn destructured_keys(pattern: P<Node>) -> Vec<&'static str> {
    let mut keys = Vec::new();
    for &element in pattern.elements() {
        if element.as_binding_element().dot_dot_dot_token().is_some() {
            continue;
        }
        let Some(mut key) = element.property_name_or_name() else {
            continue;
        };
        if key.kind() == Kind::ComputedPropertyName {
            let Some(expression) = key.expression() else {
                continue;
            };
            key = expression;
        }
        if key.kind() == Kind::Identifier {
            keys.push(key.text());
        }
    }
    keys
}

fn report_missing(ctx: &mut Ctx, node: P<Node>, missing: &[&str], fn_name: &str, error_msg: &str) {
    let message = match missing {
        [] => return,
        [prop] => RuleMessage::new(
            "propNotRead",
            format!("\"{fn_name}\" requires reads to the prop \"{prop}\". {error_msg}"),
        ),
        _ => {
            let props: Vec<String> = missing.iter().map(|prop| format!("\"{prop}\"")).collect();
            RuleMessage::new(
                "propsNotRead",
                format!(
                    "\"{fn_name}\" requires reads to the props {}. {error_msg}",
                    props.join(", ")
                ),
            )
        }
    };
    ctx.report_node(node, message);
}

struct Tracked {
    id: P<Node>,
    props: &'static [String],
    error_msg: &'static str,
    fn_name: String,
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::VariableDeclaration),
    Listener::Enter(Kind::EndOfFile),
];

impl Rule for RequireReadsToVarProp {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            rule: self,
            tracked: Vec::new(),
        })
    }
}

struct Visitor {
    rule: &'static RequireReadsToVarProp,
    /// Tracked variables by name, in the order they were first tracked: a later declaration of the same name
    /// replaces the entry, like the ESLint rule's `Map`.
    tracked: Vec<(&'static str, Tracked)>,
}

impl Visitor {
    fn on_match(
        &mut self,
        ctx: &mut Ctx,
        declarator: P<Node>,
        check: &'static Check,
        fn_name: String,
    ) {
        let Some(id) = declarator.name() else {
            return;
        };
        match id.kind() {
            Kind::Identifier => {
                let entry = Tracked {
                    id,
                    props: &check.props,
                    error_msg: &check.error_msg,
                    fn_name,
                };
                match self.tracked.iter_mut().find(|(name, _)| *name == id.text()) {
                    Some(existing) => existing.1 = entry,
                    None => self.tracked.push((id.text(), entry)),
                }
            }
            Kind::ObjectBindingPattern => {
                let keys = destructured_keys(id);
                let missing: Vec<&str> = check
                    .props
                    .iter()
                    .map(String::as_str)
                    .filter(|prop| !keys.contains(prop))
                    .collect();
                report_missing(ctx, id, &missing, &fn_name, &check.error_msg);
            }
            _ => {}
        }
    }

    fn visit_declarator(&mut self, ctx: &mut Ctx, declarator: P<Node>) {
        let rule = self.rule;
        for selector in &rule.selectors {
            if selector
                .attributes
                .iter()
                .all(|attribute| attribute_matches(declarator, attribute))
            {
                self.on_match(ctx, declarator, &selector.check, function_name(declarator));
            }
        }
        let Some(call) = declarator.initializer().map(skip_parens) else {
            return;
        };
        if call.kind() != Kind::CallExpression || is_estree_chain_expression(call) {
            return;
        }
        let children = call
            .expression()
            .into_iter()
            .chain(call.arguments().iter().copied());
        for child in children {
            let child = skip_parens(child);
            match child.kind() {
                Kind::Identifier => {
                    for (function, check) in &rule.functions {
                        if child.text() == function {
                            self.on_match(ctx, declarator, check, function.clone());
                        }
                    }
                }
                Kind::PropertyAccessExpression | Kind::ElementAccessExpression
                    if !is_estree_chain_expression(child) =>
                {
                    let Some(method) = member_identifier_property(child) else {
                        continue;
                    };
                    for (name, pattern, check) in &rule.any_object_methods {
                        if method.text() == name {
                            self.on_match(ctx, declarator, check, pattern.clone());
                        }
                    }
                    let Some(object) = child.expression().map(skip_parens) else {
                        continue;
                    };
                    if object.kind() != Kind::Identifier {
                        continue;
                    }
                    for (object_name, name, pattern, check) in &rule.object_methods {
                        if object.text() == object_name && method.text() == name {
                            self.on_match(ctx, declarator, check, pattern.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn check_tracked(&mut self, ctx: &mut Ctx) {
        if self.tracked.is_empty() {
            return;
        }
        let index = ReferenceIndex::new(ctx.file);
        for (_, tracked) in std::mem::take(&mut self.tracked) {
            let in_nested_block = tracked
                .id
                .parent()
                .and_then(|declarator| declarator.parent())
                .is_some_and(|list| {
                    list.kind() == Kind::VariableDeclarationList && is_var_in_nested_block(list)
                });
            if in_nested_block {
                continue;
            }
            let Some(symbol) = binding_symbol(ctx.checker, tracked.id) else {
                continue;
            };
            let mut unread: Vec<&str> = Vec::new();
            for prop in tracked.props {
                if !unread.contains(&prop.as_str()) {
                    unread.push(prop);
                }
            }
            for reference in index.references(ctx.checker, symbol) {
                let identifier = reference.identifier;
                let Some(parent) = estree_parent(identifier) else {
                    unread.clear();
                    break;
                };
                if parent.kind() == Kind::VariableDeclaration && parent.name() == Some(identifier) {
                    continue;
                }
                let member_property = match parent.kind() {
                    Kind::PropertyAccessExpression | Kind::ElementAccessExpression
                        if parent.expression().map(skip_parens) == Some(identifier) =>
                    {
                        member_identifier_property(parent)
                    }
                    _ => None,
                };
                if let Some(property) = member_property {
                    if let Some(i) = unread.iter().position(|prop| *prop == property.text()) {
                        unread.remove(i);
                        if unread.is_empty() {
                            break;
                        }
                        continue;
                    }
                }
                let destructured = parent.kind() == Kind::VariableDeclaration
                    && parent.initializer().map(skip_parens) == Some(identifier)
                    && parent
                        .name()
                        .is_some_and(|name| name.kind() == Kind::ObjectBindingPattern);
                if destructured {
                    if let Some(pattern) = parent.name() {
                        let keys = destructured_keys(pattern);
                        unread.retain(|prop| !keys.contains(prop));
                    }
                    if unread.is_empty() {
                        break;
                    }
                    continue;
                }
                if member_property.is_some_and(|property| {
                    !tracked.props.iter().any(|prop| prop == property.text())
                }) {
                    continue;
                }
                unread.clear();
                break;
            }
            report_missing(
                ctx,
                tracked.id,
                &unread,
                &tracked.fn_name,
                tracked.error_msg,
            );
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.kind() == Kind::EndOfFile {
            self.check_tracked(ctx);
        } else {
            self.visit_declarator(ctx, node);
        }
    }
}
