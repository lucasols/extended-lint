use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node, Symbol, SymbolFlags};
use tsrs_checker::{Checker, SignatureKind, Type, TypeFlags};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};
use tsrslint::utils::is_symbol_flag_set;

use crate::utils::{
    estree_parent, has_comments_outside, is_default_lib_declaration, is_default_lib_symbol,
    is_in_optional_chain, node_text, replacement_at, skip_parens,
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

/// Whether `node` contains an `in` operator: unparenthesized in a `for` initializer, it would read as a `for...in`.
fn contains_in_operator(node: P<Node>) -> bool {
    if ast::is_binary_expression(node)
        && node.as_binary_expression().operator_token.kind() == Kind::InKeyword
    {
        return true;
    }
    let mut found = false;
    node.for_each_child(&mut |child| {
        found = contains_in_operator(child);
        found
    });
    found
}

/// Whether `node` is part of a `for` statement's initializer.
fn is_in_for_initializer(node: P<Node>) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if ast::is_for_statement(parent) {
            return parent.as_for_statement().initializer() == Some(current);
        }
        if ast::is_statement(parent)
            || ast::is_function_like(Some(parent))
            || ast::is_class_like(parent)
        {
            return false;
        }
        current = parent;
    }
    false
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
    if contains_in_operator(arg) && is_in_for_initializer(node) {
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

/// Primitives that `Number()` may turn into a number (`null` gives 0), and types that may hold one.
const NUMBERISH_FLAGS: TypeFlags = TypeFlags::StringLike
    .union(TypeFlags::NumberLike)
    .union(TypeFlags::BigIntLike)
    .union(TypeFlags::BooleanLike)
    .union(TypeFlags::EnumLike)
    .union(TypeFlags::Null)
    .union(TypeFlags::Any)
    .union(TypeFlags::Unknown);

/// Primitives that a branded intersection (`string & { __brand: 'id' }`) holds.
const PRIMITIVE_FLAGS: TypeFlags = TypeFlags::StringLike
    .union(TypeFlags::NumberLike)
    .union(TypeFlags::BigIntLike)
    .union(TypeFlags::BooleanLike)
    .union(TypeFlags::EnumLike);

/// `{}` (also `NonNullable<unknown>` and the constraint of `T extends {}`): any value but `null` and `undefined`.
fn is_empty_object_type(checker: &mut Checker, t: P<Type>) -> bool {
    checker.get_properties_of_type(t).is_empty()
        && checker.get_index_infos_of_type(t).is_empty()
        && checker
            .get_signatures_of_type(t, SignatureKind::Call)
            .is_empty()
        && checker
            .get_signatures_of_type(t, SignatureKind::Construct)
            .is_empty()
}

/// Whether a value of type `t` may be a primitive that `Number()` turns into a number: type parameters and other
/// instantiable types by their constraint.
fn may_be_numberish(checker: &mut Checker, t: P<Type>) -> bool {
    let flags = t.flags();
    if flags.intersects(TypeFlags::Union) {
        return t
            .types()
            .iter()
            .any(|&part| may_be_numberish(checker, part));
    }
    if flags.intersects(NUMBERISH_FLAGS) {
        return true;
    }
    if flags.intersects(TypeFlags::Intersection) {
        return t
            .types()
            .iter()
            .any(|part| part.flags().intersects(PRIMITIVE_FLAGS));
    }
    if flags.intersects(TypeFlags::Instantiable) {
        return match checker.get_base_constraint_of_type(t) {
            Some(constraint) => constraint != t && may_be_numberish(checker, constraint),
            None => true,
        };
    }
    false
}

/// A conversion method of an object (`[Symbol.toPrimitive]`, `valueOf`, `toString`).
struct PrimitiveHook {
    /// A call may return a primitive that converts to a number.
    may_return_number: bool,
    /// The method may be missing (optional, `null` or `undefined`), so conversion falls back to the next method.
    may_be_absent: bool,
}

/// `None` when the method is always missing (typed `null` or `undefined`).
fn primitive_hook(
    checker: &mut Checker,
    method: P<Symbol>,
    location: P<Node>,
) -> Option<PrimitiveHook> {
    let method_type = checker.get_type_of_symbol_at_location(method, Some(location))?;
    let parts: &[P<Type>] = if method_type.flags().intersects(TypeFlags::Union) {
        method_type.types()
    } else {
        std::slice::from_ref(&method_type)
    };
    let present: Vec<P<Type>> = parts
        .iter()
        .copied()
        .filter(|part| !part.flags().intersects(NULLISH_FLAGS))
        .collect();
    if present.is_empty() {
        return None;
    }
    let may_return_number = present.iter().any(|&part| {
        if part.flags().intersects(TypeFlags::Any | TypeFlags::Unknown) {
            return true;
        }
        let signatures = checker.get_signatures_of_type(part, SignatureKind::Call);
        signatures.iter().any(|&signature| {
            let return_type = checker.get_return_type_of_signature(signature);
            may_be_numberish(checker, return_type)
        })
    });
    Some(PrimitiveHook {
        may_return_number,
        may_be_absent: is_symbol_flag_set(Some(method), SymbolFlags::Optional)
            || present.len() < parts.len(),
    })
}

/// Whether `Number()` of an object type can give a number, following `ToPrimitive`: a branded primitive
/// (`string & { __brand: 'id' }`), `{}` or `Object` (which hold primitives too), or an object whose
/// `[Symbol.toPrimitive]`, `valueOf` or own `toString` (the default ones give `"[object Object]"`, a function's source
/// and so on) may return a number-like primitive. A `[Symbol.toPrimitive]` that is always there decides alone.
fn may_convert_to_number(ctx: &mut Ctx, t: P<Type>, location: P<Node>) -> bool {
    if t.flags().intersects(TypeFlags::Intersection)
        && t.types()
            .iter()
            .any(|part| part.flags().intersects(PRIMITIVE_FLAGS))
    {
        return true;
    }
    if t == ctx.checker.global_object_type || is_empty_object_type(ctx.checker, t) {
        return true;
    }
    let apparent_type = ctx.checker.get_apparent_type(t);
    let to_primitive = ctx
        .checker
        .get_property_name_for_known_symbol_name("toPrimitive");
    if let Some(hook) = ctx
        .checker
        .get_property_of_type(apparent_type, &to_primitive)
        .and_then(|symbol| primitive_hook(ctx.checker, symbol, location))
    {
        if hook.may_return_number {
            return true;
        }
        if !hook.may_be_absent {
            return false;
        }
    }
    if ctx
        .checker
        .get_property_of_type(apparent_type, "valueOf")
        .and_then(|symbol| primitive_hook(ctx.checker, symbol, location))
        .is_some_and(|hook| hook.may_return_number)
    {
        return true;
    }
    let Some(to_string) = ctx.checker.get_property_of_type(apparent_type, "toString") else {
        return false;
    };
    if !to_string
        .declarations()
        .iter()
        .any(|&declaration| !is_default_lib_declaration(ctx.program, declaration))
    {
        return false;
    }
    primitive_hook(ctx.checker, to_string, location).is_some_and(|hook| hook.may_return_number)
}

/// Whether removing the cast would leave the argument with a narrower type than the cast's `string`/`number`: an enum
/// or a type parameter, which can break later assignments or change inferred return types.
fn keeps_narrower_type(t: P<Type>) -> bool {
    let parts: &[P<Type>] = if t.flags().intersects(TypeFlags::Union) {
        t.types()
    } else {
        std::slice::from_ref(&t)
    };
    parts.iter().any(|p| {
        p.flags()
            .intersects(TypeFlags::EnumLike | TypeFlags::TypeParameter)
    })
}

fn report_unnecessary(
    ctx: &mut Ctx,
    node: P<Node>,
    arg: P<Node>,
    name: &str,
    expected: ExpectedType,
    arg_type: Option<P<Type>>,
) {
    let message = RuleMessage::new(
        "unnecessaryCasting",
        format!(
            "Unnecessary {name}() cast of a value that is already of type {}",
            expected.as_str()
        ),
    );
    if has_comments_outside(ctx, node, arg) || arg_type.is_some_and(keeps_narrower_type) {
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

/// `castToString?.(value)` of an optional cast function evaluates to `undefined` when the function is missing, which
/// an `any` or `unknown` function may be too.
fn is_possibly_nullish(ctx: &mut Ctx, node: P<Node>) -> bool {
    let t = ctx.checker.get_type_at_location(node);
    let parts: &[P<Type>] = if t.flags().intersects(TypeFlags::Union) {
        t.types()
    } else {
        std::slice::from_ref(&t)
    };
    parts.iter().any(|p| {
        p.flags()
            .intersects(NULLISH_FLAGS | TypeFlags::Any | TypeFlags::Unknown)
    })
}

/// Whether `callee` resolves to a declaration of the default library, not a local `String`/`Number`.
fn is_global_builtin(ctx: &mut Ctx, callee: P<Node>) -> bool {
    ctx.checker
        .get_symbol_at_location_exported(callee)
        .is_some_and(|symbol| is_default_lib_symbol(ctx.program, symbol))
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
            report_unnecessary(ctx, node, arg, name, cast.expected_type, None);
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
            report_unnecessary(ctx, node, arg, name, cast.expected_type, Some(arg_type));
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
                && !may_convert_to_number(ctx, p, arg)
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
