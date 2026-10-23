use tsrs_ast::{self as ast, Kind, Node};
use tsrs_checker::{LiteralValue, Type, TypeDataTag, TypeFlags};
use tsrs_core::jsnum::BigIntSign;
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use tsrslint::utils::{get_constrained_type_at_location, union_type_parts};

use crate::utils::{node_text, skip_parens};

pub const NAME: &str = "no-leaked-conditional-rendering";

/// Guards the identifier → initializer resolution against cycles like `const a = b; const b = a;`.
const MAX_IDENTIFIER_HOPS: u32 = 16;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", default)]
struct Options {
    react_version: String,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            react_version: "19.0.0".to_string(),
        }
    }
}

pub struct NoLeakedConditionalRendering {
    /// React 18+ no longer renders empty strings, so string left operands are safe.
    allow_strings: bool,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let major = parse_major_version(&options.react_version)?;
    Ok(Box::new(NoLeakedConditionalRendering {
        allow_strings: major >= 18,
    }))
}

fn parse_major_version(version: &str) -> Result<u64, String> {
    let trimmed = version.trim().trim_start_matches(['v', 'V']);
    let major = trimmed.split('.').next().unwrap_or(trimmed);
    major
        .parse()
        .map_err(|_| format!("invalid reactVersion: {version:?}"))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::JsxExpression)];

impl Rule for NoLeakedConditionalRendering {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            allow_strings: self.allow_strings,
            enabled: ctx.text().contains("&&"),
        })
    }
}

struct Visitor {
    allow_strings: bool,
    enabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeVariant {
    Any,
    BigInt,
    Boolean,
    Enum,
    Never,
    Nullish,
    Number,
    Object,
    String,
    Unknown,
    FalsyBigInt,
    FalsyBoolean,
    FalsyNumber,
    FalsyString,
    TruthyBigInt,
    TruthyBoolean,
    TruthyNumber,
    TruthyString,
}

impl TypeVariant {
    fn is_allowed(self, allow_strings: bool) -> bool {
        match self {
            TypeVariant::Any
            | TypeVariant::Boolean
            | TypeVariant::Nullish
            | TypeVariant::Object
            | TypeVariant::FalsyBoolean
            | TypeVariant::TruthyBigInt
            | TypeVariant::TruthyBoolean
            | TypeVariant::TruthyNumber
            | TypeVariant::TruthyString => true,
            TypeVariant::String | TypeVariant::FalsyString => allow_strings,
            TypeVariant::BigInt
            | TypeVariant::Enum
            | TypeVariant::Never
            | TypeVariant::Number
            | TypeVariant::Unknown
            | TypeVariant::FalsyBigInt
            | TypeVariant::FalsyNumber => false,
        }
    }
}

fn literal_value(t: P<Type>) -> Option<LiteralValue> {
    if t.data_tag() != TypeDataTag::Literal {
        return None;
    }
    t.as_literal_type().value()
}

fn has_flag(t: P<Type>, flags: TypeFlags) -> bool {
    t.flags().intersects(flags)
}

fn is_falsy_string(t: P<Type>) -> bool {
    t.is_string_literal() && matches!(literal_value(t), Some(LiteralValue::String("")))
}

fn is_truthy_string(t: P<Type>) -> bool {
    t.is_string_literal() && !matches!(literal_value(t), Some(LiteralValue::String("")))
}

fn is_zero_number(t: P<Type>) -> bool {
    matches!(literal_value(t), Some(LiteralValue::Number(n)) if n.0 == 0.0)
}

fn is_falsy_number(t: P<Type>) -> bool {
    t.is_number_literal() && is_zero_number(t)
}

fn is_truthy_number(t: P<Type>) -> bool {
    t.is_number_literal() && !is_zero_number(t)
}

fn big_int_sign(t: P<Type>) -> Option<BigIntSign> {
    let is_literal = has_flag(
        t,
        TypeFlags::StringLiteral | TypeFlags::NumberLiteral | TypeFlags::BigIntLiteral,
    );
    if !is_literal {
        return None;
    }
    match literal_value(t) {
        Some(LiteralValue::BigInt(value)) => Some(value.sign()),
        _ => None,
    }
}

fn is_falsy_big_int(t: P<Type>) -> bool {
    matches!(big_int_sign(t), Some(BigIntSign::Zero))
}

fn is_truthy_big_int(t: P<Type>) -> bool {
    matches!(
        big_int_sign(t),
        Some(BigIntSign::Negative | BigIntSign::Positive)
    )
}

fn is_boolean_literal(t: P<Type>, value: bool) -> bool {
    has_flag(t, TypeFlags::BooleanLiteral)
        && matches!(literal_value(t), Some(LiteralValue::Boolean(v)) if v == value)
}

fn is_object_like(t: P<Type>) -> bool {
    !has_flag(
        t,
        TypeFlags::Null
            | TypeFlags::Undefined
            | TypeFlags::VoidLike
            | TypeFlags::BooleanLike
            | TypeFlags::StringLike
            | TypeFlags::NumberLike
            | TypeFlags::BigIntLike
            | TypeFlags::TypeParameter
            | TypeFlags::Any
            | TypeFlags::Unknown
            | TypeFlags::Never,
    )
}

/// Classifies a literal-capable primitive group (`strings`, `numbers`, `bigints`) as all-truthy, all-falsy or mixed.
fn literal_group_variant(
    types: &[P<Type>],
    is_truthy: fn(P<Type>) -> bool,
    is_falsy: fn(P<Type>) -> bool,
    variants: [TypeVariant; 3],
) -> TypeVariant {
    let [truthy, falsy, mixed] = variants;
    if types.iter().all(|&t| is_truthy(t)) {
        truthy
    } else if types.iter().all(|&t| is_falsy(t)) {
        falsy
    } else {
        mixed
    }
}

fn filter_flag(types: &[P<Type>], flags: TypeFlags) -> Vec<P<Type>> {
    types
        .iter()
        .copied()
        .filter(|&t| has_flag(t, flags))
        .collect()
}

fn get_type_variants(types: &[P<Type>]) -> Vec<TypeVariant> {
    let mut variants = Vec::new();
    if types.iter().any(|&t| has_flag(t, TypeFlags::Unknown)) {
        variants.push(TypeVariant::Unknown);
        return variants;
    }
    if types.iter().any(|&t| {
        has_flag(
            t,
            TypeFlags::Null | TypeFlags::Undefined | TypeFlags::VoidLike,
        )
    }) {
        variants.push(TypeVariant::Nullish);
    }
    let booleans = filter_flag(types, TypeFlags::BooleanLike);
    match booleans.as_slice() {
        [single] if is_boolean_literal(*single, false) => variants.push(TypeVariant::FalsyBoolean),
        [single] if is_boolean_literal(*single, true) => variants.push(TypeVariant::TruthyBoolean),
        [_, _] => variants.push(TypeVariant::Boolean),
        _ => {}
    }
    let strings = filter_flag(types, TypeFlags::StringLike);
    if !strings.is_empty() {
        variants.push(literal_group_variant(
            &strings,
            is_truthy_string,
            is_falsy_string,
            [
                TypeVariant::TruthyString,
                TypeVariant::FalsyString,
                TypeVariant::String,
            ],
        ));
    }
    let big_ints = filter_flag(types, TypeFlags::BigIntLike);
    if !big_ints.is_empty() {
        variants.push(literal_group_variant(
            &big_ints,
            is_truthy_big_int,
            is_falsy_big_int,
            [
                TypeVariant::TruthyBigInt,
                TypeVariant::FalsyBigInt,
                TypeVariant::BigInt,
            ],
        ));
    }
    let numbers = filter_flag(types, TypeFlags::NumberLike);
    if !numbers.is_empty() {
        variants.push(literal_group_variant(
            &numbers,
            is_truthy_number,
            is_falsy_number,
            [
                TypeVariant::TruthyNumber,
                TypeVariant::FalsyNumber,
                TypeVariant::Number,
            ],
        ));
    }
    if types.iter().any(|&t| has_flag(t, TypeFlags::EnumLike)) {
        variants.push(TypeVariant::Enum);
    }
    if types.iter().any(|&t| is_object_like(t)) {
        variants.push(TypeVariant::Object);
    }
    if types
        .iter()
        .any(|&t| has_flag(t, TypeFlags::TypeParameter | TypeFlags::Any))
    {
        variants.push(TypeVariant::Any);
    }
    if types.iter().any(|&t| has_flag(t, TypeFlags::Never)) {
        variants.push(TypeVariant::Never);
    }
    variants
}

/// Parentheses plus ESLint-React's `AST.isTypeExpression` (`as`, `satisfies`, `!`, `<T>x`, `f<T>`).
fn skip_type_expressions(mut node: P<Node>) -> P<Node> {
    loop {
        node = skip_parens(node);
        match node.kind() {
            Kind::AsExpression => node = node.as_as_expression().expression,
            Kind::SatisfiesExpression => node = node.as_satisfies_expression().expression,
            Kind::NonNullExpression => node = node.as_non_null_expression().expression,
            Kind::TypeAssertionExpression => node = node.as_type_assertion().expression,
            Kind::ExpressionWithTypeArguments => {
                node = node.as_expression_with_type_arguments().expression;
            }
            _ => return node,
        }
    }
}

fn is_logical_not(node: P<Node>) -> bool {
    node.kind() == Kind::PrefixUnaryExpression
        && node.as_prefix_unary_expression().operator == Kind::ExclamationToken
}

fn is_identifier_named(node: P<Node>, name: &str) -> bool {
    node.kind() == Kind::Identifier && node.text() == name
}

/// The `VariableDeclaration` an identifier's first declaration belongs to, if it is declared in the linted file. For
/// destructured bindings this is the enclosing declaration, like ESLint's variable def node (the declarator).
fn variable_declaration_of(ctx: &mut Ctx, identifier: P<Node>) -> Option<P<Node>> {
    let symbol = ctx.checker.get_symbol_at_location_exported(identifier)?;
    let mut declaration = *symbol.declarations().first()?;
    while declaration.kind() == Kind::BindingElement {
        let pattern = declaration.parent()?;
        declaration = pattern.parent()?;
    }
    if declaration.kind() != Kind::VariableDeclaration {
        return None;
    }
    if ast::get_source_file_of_node(declaration) != Some(ctx.file) {
        return None;
    }
    Some(declaration)
}

fn variable_initializer(ctx: &mut Ctx, identifier: P<Node>) -> Option<P<Node>> {
    let declaration = variable_declaration_of(ctx, identifier)?;
    declaration.as_variable_declaration().initializer.get()
}

/// `getStaticValue(left)?.value === "NaN"`: the static value is the string `"NaN"` (a string or template literal,
/// possibly behind type expressions or a `const` binding).
fn is_static_nan_string(ctx: &mut Ctx, node: P<Node>, hops: u32) -> bool {
    let node = skip_type_expressions(node);
    match node.kind() {
        Kind::StringLiteral | Kind::NoSubstitutionTemplateLiteral => node.text() == "NaN",
        Kind::Identifier if hops < MAX_IDENTIFIER_HOPS => {
            let Some(declaration) = variable_declaration_of(ctx, node) else {
                return false;
            };
            let variable = declaration.as_variable_declaration();
            if variable.name.kind() != Kind::Identifier || !ast::is_var_const(declaration) {
                return false;
            }
            let Some(initializer) = variable.initializer.get() else {
                return false;
            };
            is_static_nan_string(ctx, initializer, hops + 1)
        }
        _ => false,
    }
}

impl Visitor {
    /// The left operand of the first `&&` whose value could be rendered, following the branches React renders.
    fn find_leaked_operand(&self, ctx: &mut Ctx, node: P<Node>, hops: u32) -> Option<P<Node>> {
        let node = skip_type_expressions(node);
        match node.kind() {
            Kind::BinaryExpression => self.check_logical_and(ctx, node, hops),
            Kind::ConditionalExpression => {
                let conditional = node.as_conditional_expression();
                self.find_leaked_operand(ctx, conditional.when_true, hops)
                    .or_else(|| self.find_leaked_operand(ctx, conditional.when_false, hops))
            }
            Kind::Identifier if hops < MAX_IDENTIFIER_HOPS => {
                let initializer = variable_initializer(ctx, node)?;
                self.find_leaked_operand(ctx, initializer, hops + 1)
            }
            _ => None,
        }
    }

    fn check_logical_and(&self, ctx: &mut Ctx, node: P<Node>, hops: u32) -> Option<P<Node>> {
        let binary = node.as_binary_expression();
        if binary.operator_token.kind() != Kind::AmpersandAmpersandToken {
            return None;
        }
        let left = skip_parens(binary.left);
        let right = binary.right.get();
        if is_logical_not(left) {
            return self.find_leaked_operand(ctx, right, hops);
        }
        if is_identifier_named(left, "NaN") || !self.is_safe_left_operand(ctx, left) {
            return Some(left);
        }
        if is_static_nan_string(ctx, left, 0) {
            return Some(left);
        }
        self.find_leaked_operand(ctx, right, hops)
    }

    fn is_safe_left_operand(&self, ctx: &mut Ctx, left: P<Node>) -> bool {
        let left_type = get_constrained_type_at_location(ctx.checker, left);
        get_type_variants(&union_type_parts(left_type))
            .into_iter()
            .all(|variant| variant.is_allowed(self.allow_strings))
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        if self.enabled {
            LISTENERS
        } else {
            &[]
        }
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let jsx_expression = node.as_jsx_expression();
        if jsx_expression.dot_dot_dot_token.is_some() {
            return;
        }
        let Some(expression) = jsx_expression.expression else {
            return;
        };
        let Some(leaked) = self.find_leaked_operand(ctx, expression, 0) else {
            return;
        };
        let value = node_text(ctx, leaked);
        ctx.report_node(
            leaked,
            RuleMessage::new(
                "noLeakedConditionalRendering",
                format!(
                    "Potential leaked value {value} that might cause unintentionally rendered values or rendering crashes."
                ),
            ),
        );
    }
}
