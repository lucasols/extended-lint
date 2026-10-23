use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{Checker, SignatureKind, Type, TypeFlags};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};

use crate::utils::{
    comment_at, estree_parent, is_in_optional_chain, node_text, replacement_at, skip_parens,
};

pub const NAME: &str = "strict-casting";

#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ExpectedType {
    String,
    Number,
}

impl ExpectedType {
    fn as_str(self) -> &'static str {
        match self {
            ExpectedType::String => "string",
            ExpectedType::Number => "number",
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AdditionalCastFunction {
    name: String,
    expected_type: ExpectedType,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    additional_cast_functions: Vec<AdditionalCastFunction>,
    allow_nullish_casting: bool,
    allow_invalid_casting: bool,
    allow_any_casting: bool,
    allow_unknown_casting: bool,
}

#[derive(Clone, Copy)]
struct CastFunction {
    expected_type: ExpectedType,
    builtin: bool,
}

pub struct StrictCasting {
    cast_functions: FxHashMap<String, CastFunction>,
    allow_nullish_casting: bool,
    allow_invalid_casting: bool,
    allow_any_casting: bool,
    allow_unknown_casting: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut cast_functions = FxHashMap::default();
    cast_functions.insert(
        "String".to_string(),
        CastFunction {
            expected_type: ExpectedType::String,
            builtin: true,
        },
    );
    cast_functions.insert(
        "Number".to_string(),
        CastFunction {
            expected_type: ExpectedType::Number,
            builtin: true,
        },
    );
    for function in options.additional_cast_functions {
        cast_functions.insert(
            function.name,
            CastFunction {
                expected_type: function.expected_type,
                builtin: false,
            },
        );
    }
    Ok(Box::new(StrictCasting {
        cast_functions,
        allow_nullish_casting: options.allow_nullish_casting,
        allow_invalid_casting: options.allow_invalid_casting,
        allow_any_casting: options.allow_any_casting,
        allow_unknown_casting: options.allow_unknown_casting,
    }))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::CallExpression)];

const NULLISH_FLAGS: TypeFlags = TypeFlags::Null
    .union(TypeFlags::Undefined)
    .union(TypeFlags::Void);

impl Rule for StrictCasting {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static StrictCasting,
}

/// ESTree's `simpleArgTypes`: Identifier, Literal, TemplateLiteral, MemberExpression and CallExpression. Optional
/// chains are wrapped in an ESTree `ChainExpression`, so they are not simple.
fn is_simple_arg(arg: P<Node>) -> bool {
    match arg.kind() {
        Kind::Identifier
        | Kind::StringLiteral
        | Kind::NumericLiteral
        | Kind::BigIntLiteral
        | Kind::RegularExpressionLiteral
        | Kind::TrueKeyword
        | Kind::FalseKeyword
        | Kind::NullKeyword
        | Kind::NoSubstitutionTemplateLiteral
        | Kind::TemplateExpression => true,
        Kind::PropertyAccessExpression | Kind::ElementAccessExpression | Kind::CallExpression => {
            !is_in_optional_chain(arg)
        }
        _ => false,
    }
}

/// A computed key of an object literal or object pattern member, whose ESTree parent is a `Property`.
fn is_object_member_computed_name(computed_name: P<Node>) -> bool {
    computed_name
        .parent()
        .and_then(|member| member.parent())
        .is_some_and(|container| {
            matches!(
                container.kind(),
                Kind::ObjectLiteralExpression | Kind::ObjectBindingPattern
            )
        })
}

/// `Number(1).toString()`: a bare `1.toString()` would read the `.` as a decimal point.
fn is_numeric_literal_member_object(arg: P<Node>, node: P<Node>) -> bool {
    ast::is_numeric_literal(arg)
        && estree_parent(node).is_some_and(|parent| {
            ast::is_property_access_expression(parent)
                && parent.expression().map(skip_parens) == Some(node)
        })
}

fn unnecessary_fix_needs_parens(arg: P<Node>, node: P<Node>) -> bool {
    if is_numeric_literal_member_object(arg, node) {
        return true;
    }
    if is_simple_arg(arg) {
        return false;
    }
    if ast::is_comma_expression(arg) {
        return true;
    }
    if is_in_optional_chain(node) {
        return true;
    }
    let Some(parent) = estree_parent(node) else {
        return true;
    };
    match parent.kind() {
        Kind::VariableDeclaration
        | Kind::ExpressionStatement
        | Kind::ReturnStatement
        | Kind::ArrowFunction
        | Kind::ArrayLiteralExpression => false,
        Kind::PropertyAssignment => false,
        Kind::ComputedPropertyName => !is_object_member_computed_name(parent),
        Kind::JsxExpression => parent.as_jsx_expression().dot_dot_dot_token().is_some(),
        Kind::BinaryExpression => {
            !ast::is_assignment_operator(parent.as_binary_expression().operator_token.kind())
        }
        Kind::CallExpression | Kind::NewExpression => {
            parent.expression().map(skip_parens) == Some(node)
        }
        _ => true,
    }
}

fn is_literal_of_expected_type(arg: P<Node>, expected_type: ExpectedType) -> bool {
    match expected_type {
        ExpectedType::String => matches!(
            arg.kind(),
            Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral | Kind::TemplateExpression
        ),
        ExpectedType::Number => ast::is_numeric_literal(arg),
    }
}

/// The union parts of `t`, with type parameters replaced by the parts of their base constraint. `None` when a type
/// parameter has no constraint.
fn resolve_union_parts(checker: &mut Checker, t: P<Type>) -> Option<Vec<P<Type>>> {
    let raw_parts: &[P<Type>] = if t.flags().intersects(TypeFlags::Union) {
        t.types()
    } else {
        std::slice::from_ref(&t)
    };
    let mut parts = Vec::with_capacity(raw_parts.len());
    for &raw_part in raw_parts {
        if !raw_part.flags().intersects(TypeFlags::TypeParameter) {
            parts.push(raw_part);
            continue;
        }
        let constraint = checker.get_base_constraint_of_type(raw_part)?;
        if constraint.flags().intersects(TypeFlags::Union) {
            parts.extend_from_slice(constraint.types());
        } else {
            parts.push(constraint);
        }
    }
    Some(parts)
}

/// Whether `Number()` of an object type can give a number: a branded primitive (`string & { __brand: 'id' }`), an
/// object with `[Symbol.toPrimitive]` or a `valueOf` returning a number.
fn may_convert_to_number(checker: &mut Checker, t: P<Type>, location: P<Node>) -> bool {
    if t.flags().intersects(TypeFlags::Intersection)
        && t.types().iter().any(|part| {
            part.flags().intersects(
                TypeFlags::StringLike
                    | TypeFlags::NumberLike
                    | TypeFlags::BigIntLike
                    | TypeFlags::BooleanLike
                    | TypeFlags::EnumLike,
            )
        })
    {
        return true;
    }
    let apparent_type = checker.get_apparent_type(t);
    let to_primitive = checker.get_property_name_for_known_symbol_name("toPrimitive");
    if checker
        .get_property_of_type(apparent_type, &to_primitive)
        .is_some()
    {
        return true;
    }
    has_numeric_value_of(checker, apparent_type, location)
}

fn has_numeric_value_of(checker: &mut Checker, apparent_type: P<Type>, location: P<Node>) -> bool {
    let Some(value_of_symbol) = checker.get_property_of_type(apparent_type, "valueOf") else {
        return false;
    };
    let Some(value_of_type) =
        checker.get_type_of_symbol_at_location(value_of_symbol, Some(location))
    else {
        return false;
    };
    let signatures = checker.get_signatures_of_type(value_of_type, SignatureKind::Call);
    signatures.iter().any(|&signature| {
        checker
            .get_return_type_of_signature(signature)
            .flags()
            .intersects(TypeFlags::NumberLike)
    })
}

/// Whether the call has comments outside its argument, which removing the call would drop.
fn has_comments_outside_arg(ctx: &Ctx, node: P<Node>, arg: P<Node>) -> bool {
    let (node_start, node_end) = ctx.trim(node);
    let (arg_start, arg_end) = ctx.trim(arg);
    let text = ctx.text();
    [(node_start, arg_start), (arg_end, node_end)]
        .iter()
        .any(|&(start, end)| {
            let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
                return false;
            };
            let segment = text.get(start..end).unwrap_or_default();
            ["//", "/*"].iter().any(|opener| {
                segment.match_indices(opener).any(|(i, _)| {
                    u32::try_from(start + i).is_ok_and(|pos| comment_at(ctx.file, pos).is_some())
                })
            })
        })
}

fn report_unnecessary(
    ctx: &mut Ctx,
    node: P<Node>,
    arg: P<Node>,
    name: &str,
    expected: ExpectedType,
) {
    let message = RuleMessage::new(
        "unnecessaryCasting",
        format!(
            "Unnecessary {name}() cast of a value that is already of type {}",
            expected.as_str()
        ),
    );
    if has_comments_outside_arg(ctx, node, arg) {
        ctx.report_node(node, message);
        return;
    }
    ctx.report_node_with_fixes(node, message, |ctx| {
        let arg_text = node_text(ctx, arg);
        let replacement = if unnecessary_fix_needs_parens(arg, node) {
            format!("({arg_text})")
        } else {
            arg_text.to_string()
        };
        vec![ctx.fix_replace(node, replacement_at(ctx, node, replacement))]
    });
}

/// `castToString?.(value)` of an optional cast function evaluates to `undefined` when the function is missing.
fn is_possibly_nullish(ctx: &mut Ctx, node: P<Node>) -> bool {
    let t = ctx.checker.get_type_at_location(node);
    let parts: &[P<Type>] = if t.flags().intersects(TypeFlags::Union) {
        t.types()
    } else {
        std::slice::from_ref(&t)
    };
    parts.iter().any(|p| p.flags().intersects(NULLISH_FLAGS))
}

/// Whether `callee` resolves to a declaration of the default library, not a local `String`/`Number`.
fn is_global_builtin(ctx: &mut Ctx, callee: P<Node>) -> bool {
    let Some(symbol) = ctx.checker.get_symbol_at_location_exported(callee) else {
        return false;
    };
    symbol.declarations().iter().any(|&declaration| {
        ast::get_source_file_of_node(declaration)
            .is_some_and(|file| ctx.program.is_source_file_default_library(file.path()))
    })
}

fn report_nullish(ctx: &mut Ctx, node: P<Node>, arg: P<Node>, expected: ExpectedType) {
    let (message, fallback) = match expected {
        ExpectedType::String => (
            RuleMessage::new(
                "nullishStringCasting",
                "String() cast of a possibly nullish value results in the strings 'null' or 'undefined', handle the nullish case explicitly",
            ),
            "''",
        ),
        ExpectedType::Number => (
            RuleMessage::new(
                "nullishNumberCasting",
                "Number() cast of a possibly nullish value results in 0 for null and NaN for undefined, handle the nullish case explicitly",
            ),
            "0",
        ),
    };
    ctx.report_node_with_suggestions(node, message, |ctx| {
        let arg_text = node_text(ctx, arg);
        let replacement = if is_simple_arg(arg) {
            format!("{arg_text} ?? {fallback}")
        } else {
            format!("({arg_text}) ?? {fallback}")
        };
        vec![RuleSuggestion {
            message: RuleMessage::new(
                "addNullishFallback",
                format!("Add a `?? {fallback}` fallback to handle the nullish case"),
            ),
            fixes: vec![ctx.fix_replace(arg, replacement)],
        }]
    });
}

impl StrictCasting {
    fn check(&self, ctx: &mut Ctx, node: P<Node>, arg: P<Node>, name: &str, cast: CastFunction) {
        if is_literal_of_expected_type(arg, cast.expected_type) {
            report_unnecessary(ctx, node, arg, name, cast.expected_type);
            return;
        }

        let arg_type = ctx.checker.get_type_at_location(arg);
        let Some(parts) = resolve_union_parts(ctx.checker, arg_type) else {
            return;
        };

        let mut has_any_part = false;
        let mut has_unknown_part = false;
        for part in &parts {
            if part.flags().intersects(TypeFlags::Any) {
                has_any_part = true;
            } else if part.flags().intersects(TypeFlags::Unknown) {
                has_unknown_part = true;
            }
        }
        if has_any_part || has_unknown_part {
            if !cast.builtin {
                return;
            }
            if has_any_part && !self.allow_any_casting {
                ctx.report_node(
                    node,
                    RuleMessage::new(
                        "anyCasting",
                        format!("{name}() cast of an `any` value is unsafe, type the value before casting"),
                    ),
                );
            } else if has_unknown_part && !self.allow_unknown_casting {
                ctx.report_node(
                    node,
                    RuleMessage::new(
                        "unknownCasting",
                        format!("{name}() cast of an `unknown` value is unsafe, narrow the value before casting"),
                    ),
                );
            }
            return;
        }

        if cast.builtin
            && !self.allow_nullish_casting
            && parts.iter().any(|p| p.flags().intersects(NULLISH_FLAGS))
        {
            report_nullish(ctx, node, arg, cast.expected_type);
            return;
        }

        if cast.builtin
            && cast.expected_type == ExpectedType::Number
            && !self.allow_invalid_casting
            && self.report_invalid_number_cast(ctx, node, arg, arg_type, &parts)
        {
            return;
        }

        let target_flags = match cast.expected_type {
            ExpectedType::String => TypeFlags::StringLike,
            ExpectedType::Number => TypeFlags::NumberLike,
        };
        if parts.iter().all(|p| p.flags().intersects(target_flags)) {
            report_unnecessary(ctx, node, arg, name, cast.expected_type);
        }
    }

    fn report_invalid_number_cast(
        &self,
        ctx: &mut Ctx,
        node: P<Node>,
        arg: P<Node>,
        arg_type: P<Type>,
        parts: &[P<Type>],
    ) -> bool {
        if parts
            .iter()
            .any(|p| p.flags().intersects(TypeFlags::ESSymbolLike))
        {
            ctx.report_node(
                node,
                RuleMessage::new(
                    "symbolNumberCasting",
                    "Number() cast of a symbol throws at runtime",
                ),
            );
            return true;
        }

        let has_nan_part = parts.iter().any(|&p| {
            p.flags()
                .intersects(TypeFlags::Object | TypeFlags::Intersection)
                && !may_convert_to_number(ctx.checker, p, arg)
        });
        if !has_nan_part {
            return false;
        }
        let type_string = tsrslint::utils::type_to_string(ctx.checker, arg_type);
        ctx.report_node(
            node,
            RuleMessage::new(
                "nanNumberCasting",
                format!("Number() cast of a value of type \"{type_string}\" will result in NaN"),
            ),
        );
        true
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let callee = skip_parens(node.as_call_expression().expression);
        if !ast::is_identifier(callee) {
            return;
        }
        let name = callee.as_identifier().text();
        let Some(&cast) = self.rule.cast_functions.get(name) else {
            return;
        };
        let &[arg] = node.arguments() else {
            return;
        };
        if ast::is_spread_element(arg) {
            return;
        }
        if cast.builtin && !is_global_builtin(ctx, callee) {
            return;
        }
        if node.question_dot_token().is_some() && is_possibly_nullish(ctx, callee) {
            return;
        }
        self.rule.check(ctx, node, skip_parens(arg), name, cast);
    }
}
