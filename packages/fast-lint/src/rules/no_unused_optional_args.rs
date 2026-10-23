use regex::Regex;
use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::scope::ReferenceIndex;
use crate::utils::{estree_identifier_key, estree_parent, skip_parens, skip_type_parens};

pub const NAME: &str = "no-unused-optional-args";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    ignore_args_matching: Option<String>,
}

pub struct NoUnusedOptionalArgs {
    ignore_args_matching: Option<Regex>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let ignore_args_matching = options
        .ignore_args_matching
        .filter(|pattern| !pattern.is_empty())
        .map(|pattern| Regex::new(&pattern).map_err(|e| format!("invalid ignoreArgsMatching: {e}")))
        .transpose()?;
    Ok(Box::new(NoUnusedOptionalArgs {
        ignore_args_matching,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::FunctionDeclaration),
    Listener::Enter(Kind::VariableDeclaration),
];

impl Rule for NoUnusedOptionalArgs {
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
    rule: &'static NoUnusedOptionalArgs,
    index: Option<ReferenceIndex>,
}

/// Where a report about an optional parameter goes: the parameter, or the `name: FC<...>` binding of a component.
#[derive(Clone, Copy)]
enum ReportTarget {
    Node(P<Node>),
    Range(i32, i32),
}

struct OptionalParam {
    target: ReportTarget,
    index: usize,
    name: &'static str,
    /// The members of an object parameter (`props: { a?: string }`) as `(name, optional)`.
    object_props: Option<Vec<(&'static str, bool)>>,
}

/// What the calls of a function pass for one of its optional parameters.
#[derive(Default)]
struct Usage {
    passed: bool,
    provided_props: FxHashSet<&'static str>,
}

fn type_literal_props(type_literal: P<Node>) -> Vec<(&'static str, bool)> {
    type_literal
        .members()
        .iter()
        .filter(|member| member.kind() == Kind::PropertySignature)
        .filter_map(|&member| {
            let name = estree_identifier_key(member.name()?)?;
            Some((name, member.question_token().is_some()))
        })
        .collect()
}

fn has_optional_prop(props: &[(&'static str, bool)]) -> bool {
    props.iter().any(|&(_, optional)| optional)
}

fn type_literal_of(node: P<Node>) -> Option<P<Node>> {
    let type_node = skip_type_parens(node.type_node()?);
    (type_node.kind() == Kind::TypeLiteral).then_some(type_node)
}

impl Visitor {
    fn optional_params(&self, params: &[P<Node>]) -> Vec<OptionalParam> {
        let mut result = Vec::new();
        for (index, &param) in params.iter().enumerate() {
            let Some(name) = param.name().filter(|name| name.kind() == Kind::Identifier) else {
                continue;
            };
            if param.initializer().is_some()
                || param
                    .as_parameter_declaration()
                    .dot_dot_dot_token()
                    .is_some()
            {
                continue;
            }
            let name = name.text();
            if self
                .rule
                .ignore_args_matching
                .as_ref()
                .is_some_and(|regex| regex.is_match(name))
            {
                continue;
            }
            let optional = param.question_token().is_some();
            let object_props = type_literal_of(param).map(type_literal_props);
            let target = ReportTarget::Node(param);
            match object_props {
                Some(props) if has_optional_prop(&props) => result.push(OptionalParam {
                    target,
                    index,
                    name,
                    object_props: Some(props),
                }),
                _ if optional => result.push(OptionalParam {
                    target,
                    index,
                    name,
                    object_props: None,
                }),
                _ => {}
            }
        }
        result
    }

    fn check(&mut self, ctx: &mut Ctx, name: P<Node>, optional_params: &[OptionalParam]) {
        let index = self
            .index
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file));
        let references: Vec<P<Node>> = index
            .references_of_binding(ctx.checker, name)
            .into_iter()
            .filter(|reference| !reference.init)
            .map(|reference| reference.identifier)
            .collect();
        if references.is_empty() || references.iter().any(|&r| is_passed_around(r)) {
            return;
        }
        let mut usages: Vec<Usage> = optional_params.iter().map(|_| Usage::default()).collect();
        let mut has_calls = false;
        for &reference in &references {
            let Some(parent) = estree_parent(reference) else {
                continue;
            };
            match parent.kind() {
                Kind::CallExpression
                    if skip_parens(parent.as_call_expression().expression) == reference =>
                {
                    let arguments = parent.arguments();
                    if arguments
                        .iter()
                        .any(|argument| argument.kind() == Kind::SpreadElement)
                    {
                        return;
                    }
                    for (param, usage) in optional_params.iter().zip(usages.iter_mut()) {
                        if arguments.len() > param.index {
                            usage.passed = true;
                        }
                        if param.object_props.is_none() {
                            continue;
                        }
                        let Some(&argument) = arguments.get(param.index) else {
                            continue;
                        };
                        let argument = skip_parens(argument);
                        if argument.kind() != Kind::ObjectLiteralExpression {
                            return;
                        }
                        for &property in argument.properties() {
                            if property.kind() == Kind::SpreadAssignment {
                                continue;
                            }
                            if let Some(key) = property.name().and_then(estree_identifier_key) {
                                usage.provided_props.insert(key);
                            }
                        }
                    }
                    has_calls = true;
                }
                Kind::JsxOpeningElement | Kind::JsxSelfClosingElement
                    if parent.tag_name() == reference =>
                {
                    let attributes = parent
                        .attributes()
                        .map(|attributes| attributes.properties())
                        .unwrap_or_default();
                    if attributes
                        .iter()
                        .any(|attribute| attribute.kind() == Kind::JsxSpreadAttribute)
                    {
                        return;
                    }
                    for (param, usage) in optional_params.iter().zip(usages.iter_mut()) {
                        if param.index == 0 {
                            usage.passed = true;
                        }
                        if param.object_props.is_none() || param.index != 0 {
                            continue;
                        }
                        for &attribute in attributes {
                            let name = attribute.as_jsx_attribute().name();
                            if name.kind() == Kind::Identifier {
                                usage.provided_props.insert(name.text());
                            }
                        }
                    }
                    has_calls = true;
                }
                _ => {}
            }
        }
        if !has_calls {
            return;
        }
        for (param, usage) in optional_params.iter().zip(&usages) {
            match &param.object_props {
                Some(props) => {
                    for &(prop, optional) in props {
                        if optional && !usage.provided_props.contains(prop) {
                            report(
                                ctx,
                                param.target,
                                RuleMessage::new(
                                    "unusedOptionalProp",
                                    format!("Optional prop '{prop}' is never provided"),
                                ),
                            );
                        }
                    }
                }
                None => {
                    if !usage.passed {
                        report(
                            ctx,
                            param.target,
                            RuleMessage::new(
                                "unusedOptionalArg",
                                format!("Optional parameter '{}' is never used", param.name),
                            ),
                        );
                    }
                }
            }
        }
    }
}

fn report(ctx: &mut Ctx, target: ReportTarget, message: RuleMessage) {
    match target {
        ReportTarget::Node(node) => ctx.report_node(node, message),
        ReportTarget::Range(pos, end) => ctx.report_range(pos, end, message),
    }
}

/// Whether the function is used other than by calling it: passed as an argument, returned (directly or as a property
/// of a returned object literal) or passed to a JSX attribute.
fn is_passed_around(reference: P<Node>) -> bool {
    let Some(parent) = estree_parent(reference) else {
        return false;
    };
    match parent.kind() {
        Kind::CallExpression => parent
            .arguments()
            .iter()
            .any(|&argument| skip_parens(argument) == reference),
        Kind::ReturnStatement | Kind::JsxExpression => true,
        _ => is_returned_object_property(parent),
    }
}

fn is_returned_object_property(parent: P<Node>) -> bool {
    let property = if parent.kind() == Kind::ComputedPropertyName {
        match parent.parent() {
            Some(property) => property,
            None => return false,
        }
    } else {
        parent
    };
    if !matches!(
        property.kind(),
        Kind::PropertyAssignment
            | Kind::ShorthandPropertyAssignment
            | Kind::MethodDeclaration
            | Kind::GetAccessor
            | Kind::SetAccessor
    ) {
        return false;
    }
    property
        .parent()
        .filter(|object| object.kind() == Kind::ObjectLiteralExpression)
        .and_then(estree_parent)
        .is_some_and(|statement| statement.kind() == Kind::ReturnStatement)
}

/// The props of `name: FC<{ a?: string }>` as a synthetic first parameter.
fn fc_props_param(ctx: &Ctx, declaration: P<Node>, name: P<Node>) -> Option<OptionalParam> {
    let annotation = declaration.type_node()?;
    let reference = skip_type_parens(annotation);
    if reference.kind() != Kind::TypeReference {
        return None;
    }
    let type_name = reference.as_type_reference_node().type_name();
    if type_name.kind() != Kind::Identifier || type_name.text() != "FC" {
        return None;
    }
    let [props_type] = reference.type_arguments() else {
        return None;
    };
    let props_type = skip_type_parens(*props_type);
    if props_type.kind() != Kind::TypeLiteral {
        return None;
    }
    let props = type_literal_props(props_type);
    if !has_optional_prop(&props) {
        return None;
    }
    Some(OptionalParam {
        target: ReportTarget::Range(ctx.trim(name).0, ctx.trim(annotation).1),
        index: 0,
        name: "props",
        object_props: Some(props),
    })
}

fn is_exported_variable(declaration: P<Node>) -> bool {
    declaration
        .parent()
        .and_then(|list| list.parent())
        .is_some_and(|statement| {
            statement.kind() == Kind::VariableStatement
                && ast::has_syntactic_modifier(statement, ModifierFlags::Export)
        })
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.kind() == Kind::FunctionDeclaration {
            if ast::has_syntactic_modifier(node, ModifierFlags::Export) {
                return;
            }
            let Some(name) = node.name() else {
                return;
            };
            let optional_params = self.optional_params(node.parameters());
            if !optional_params.is_empty() {
                self.check(ctx, name, &optional_params);
            }
            return;
        }
        let Some(name) = node.name().filter(|name| name.kind() == Kind::Identifier) else {
            return;
        };
        let Some(function) = node
            .initializer()
            .map(skip_parens)
            .filter(|init| matches!(init.kind(), Kind::ArrowFunction | Kind::FunctionExpression))
        else {
            return;
        };
        if is_exported_variable(node) {
            return;
        }
        let mut optional_params = self.optional_params(function.parameters());
        if let Some(props) = fc_props_param(ctx, node, name) {
            optional_params.push(props);
        }
        if !optional_params.is_empty() {
            self.check(ctx, name, &optional_params);
        }
    }
}
