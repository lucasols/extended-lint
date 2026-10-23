use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node, Symbol, SymbolFlags};
use tsrs_checker::{
    is_tuple_type_exported as is_tuple_type, Checker, LiteralValue, ObjectFlags, SignatureKind,
    Type, TypeFlags,
};
use tsrs_compiler::Program;
use tsrs_core::{jsnum, P};
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};
use tsrslint::utils::{get_call_signatures, is_symbol_flag_set, type_to_string, union_type_parts};

use crate::utils::{
    is_default_lib_declaration, is_default_lib_symbol, is_estree_chain_expression, node_text,
    skip_parens,
};

pub const NAME: &str = "improved-no-unnecessary-condition";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    alternative_for_in_condition: Option<String>,
}

pub struct ImprovedNoUnnecessaryCondition {
    /// The `{{alternative}}` suffix of the `in` check messages.
    in_condition_alternative: String,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let in_condition_alternative = match options.alternative_for_in_condition {
        Some(alternative) if !alternative.is_empty() => {
            format!(" If this is a valid condition, you can use as alternative: {alternative}")
        }
        _ => String::new(),
    };
    Ok(Box::new(ImprovedNoUnnecessaryCondition {
        in_condition_alternative,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::BinaryExpression),
    Listener::Enter(Kind::CallExpression),
];

impl Rule for ImprovedNoUnnecessaryCondition {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            rule: self,
            annotated_literals: None,
        })
    }
}

/// String literal values of variables annotated with string literal types, by name, as found by the first
/// declarator that decides the name. Built lazily, at most once per file.
type AnnotatedLiterals = FxHashMap<&'static str, Option<Vec<&'static str>>>;

struct Visitor {
    rule: &'static ImprovedNoUnnecessaryCondition,
    annotated_literals: Option<AnnotatedLiterals>,
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if ast::is_binary_expression(node) {
            let operator = node.as_binary_expression().operator_token.kind();
            check_typeof_comparison(ctx, node, operator);
            check_length_comparison(ctx, node, operator);
            if operator == Kind::InKeyword {
                self.check_in_condition(ctx, node);
            }
        } else {
            self.check_string_assertion(ctx, node);
            check_array_is_array(ctx, node);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeofValue {
    String,
    Number,
    Bigint,
    Boolean,
    Symbol,
    Undefined,
    Object,
    Function,
    Never,
}

impl TypeofValue {
    fn parse(value: &str) -> Option<TypeofValue> {
        Some(match value {
            "string" => TypeofValue::String,
            "number" => TypeofValue::Number,
            "bigint" => TypeofValue::Bigint,
            "boolean" => TypeofValue::Boolean,
            "symbol" => TypeofValue::Symbol,
            "undefined" => TypeofValue::Undefined,
            "object" => TypeofValue::Object,
            "function" => TypeofValue::Function,
            "never" => TypeofValue::Never,
            _ => return None,
        })
    }

    fn as_str(self) -> &'static str {
        match self {
            TypeofValue::String => "string",
            TypeofValue::Number => "number",
            TypeofValue::Bigint => "bigint",
            TypeofValue::Boolean => "boolean",
            TypeofValue::Symbol => "symbol",
            TypeofValue::Undefined => "undefined",
            TypeofValue::Object => "object",
            TypeofValue::Function => "function",
            TypeofValue::Never => "never",
        }
    }
}

/// The `typeof` results of the `object` type, which functions are assignable to.
const NON_PRIMITIVE_TYPEOFS: &[TypeofValue] = &[TypeofValue::Object, TypeofValue::Function];

/// The `typeof` results of the `{}` type: anything but `undefined` and `null`.
const NON_NULLISH_TYPEOFS: &[TypeofValue] = &[
    TypeofValue::String,
    TypeofValue::Number,
    TypeofValue::Bigint,
    TypeofValue::Boolean,
    TypeofValue::Symbol,
    TypeofValue::Object,
    TypeofValue::Function,
];

fn typeof_values_of_type(
    checker: &mut Checker,
    program: &Program,
    t: P<Type>,
) -> Option<&'static [TypeofValue]> {
    let flags = t.flags();
    if flags.intersects(TypeFlags::Any | TypeFlags::Unknown) {
        return None;
    }
    if flags.intersects(TypeFlags::StringLike) {
        return Some(&[TypeofValue::String]);
    }
    if flags.intersects(TypeFlags::NumberLike) {
        return Some(&[TypeofValue::Number]);
    }
    if flags.intersects(TypeFlags::BigIntLike) {
        return Some(&[TypeofValue::Bigint]);
    }
    if flags.intersects(TypeFlags::BooleanLike) {
        return Some(&[TypeofValue::Boolean]);
    }
    if flags.intersects(TypeFlags::ESSymbolLike) {
        return Some(&[TypeofValue::Symbol]);
    }
    if flags.intersects(TypeFlags::Undefined | TypeFlags::Void) {
        return Some(&[TypeofValue::Undefined]);
    }
    if flags.intersects(TypeFlags::Null) {
        return Some(&[TypeofValue::Object]);
    }
    if !get_call_signatures(checker, t).is_empty()
        || !checker
            .get_signatures_of_type(t, SignatureKind::Construct)
            .is_empty()
    {
        return Some(&[TypeofValue::Function]);
    }
    if is_function_interface_type(checker, program, t) {
        return Some(&[TypeofValue::Function]);
    }
    if t == checker.global_object_type {
        return Some(NON_NULLISH_TYPEOFS);
    }
    if flags.intersects(TypeFlags::Object) {
        if checker.get_properties_of_type(t).is_empty() {
            return Some(NON_NULLISH_TYPEOFS);
        }
        return Some(&[TypeofValue::Object]);
    }
    if flags.intersects(TypeFlags::NonPrimitive) {
        return Some(NON_PRIMITIVE_TYPEOFS);
    }
    if flags.intersects(TypeFlags::Never) {
        return Some(&[TypeofValue::Never]);
    }
    None
}

/// `Function`, `CallableFunction`, `NewableFunction` and the interfaces extending them (`interface AbstractType<T>
/// extends Function`) have no signatures but are functions: they have the `bind` method of the library's `Function`
/// interfaces.
fn is_function_interface_type(checker: &mut Checker, program: &Program, t: P<Type>) -> bool {
    checker.get_property_of_type(t, "bind").is_some_and(|bind| {
        bind.declarations().iter().any(|&declaration| {
            declaration.parent().is_some_and(|parent| {
                ast::is_interface_declaration(parent)
                    && parent.name().is_some_and(|name| {
                        matches!(
                            name.text(),
                            "Function" | "CallableFunction" | "NewableFunction"
                        )
                    })
                    && is_default_lib_declaration(program, parent)
            })
        })
    })
}

/// The distinct `typeof` results of the union members, in order.
fn typeof_values_of_operand(
    checker: &mut Checker,
    program: &Program,
    operand: P<Node>,
) -> Option<Vec<TypeofValue>> {
    let t = checker.get_type_at_location(operand);
    let mut values = Vec::new();
    for part in union_type_parts(t) {
        for &value in typeof_values_of_type(checker, program, part)? {
            if !values.contains(&value) {
                values.push(value);
            }
        }
    }
    Some(values)
}

fn describe_typeof_values(values: &[TypeofValue]) -> String {
    let mut unique: Vec<&str> = Vec::with_capacity(values.len());
    for value in values {
        if !unique.contains(&value.as_str()) {
            unique.push(value.as_str());
        }
    }
    if unique.is_empty() {
        return "never".to_string();
    }
    unique.join(" | ")
}

/// The operand of `typeof` as ESLint prints it: an identifier's name or the operand's source text.
fn typeof_operand_name(ctx: &Ctx, operand: P<Node>) -> &'static str {
    if ast::is_identifier(operand) {
        operand.text()
    } else {
        node_text(ctx, operand)
    }
}

fn string_literal_text(node: P<Node>) -> Option<&'static str> {
    ast::is_string_literal(node).then(|| node.text())
}

fn numeric_literal_value(node: P<Node>) -> Option<f64> {
    ast::is_numeric_literal(node).then(|| jsnum::from_string(node.text()).0)
}

fn unnecessary_typeof_message(name: &str, t: &str) -> RuleMessage {
    RuleMessage::new(
        "unnecessaryTypeofCondition",
        format!("This condition is unnecessary. The type of \"{name}\" is always \"{t}\"."),
    )
}

fn always_false_typeof_message(name: &str, actual_type: &str, condition_type: &str) -> RuleMessage {
    RuleMessage::new(
        "alwaysFalseTypeofCondition",
        format!(
            "This condition will always be false. The type of \"{name}\" is \"{actual_type}\" so the condition has no overlap with \"{condition_type}\"."
        ),
    )
}

fn check_typeof_comparison(ctx: &mut Ctx, node: P<Node>, operator: Kind) {
    let negated = match operator {
        Kind::EqualsEqualsEqualsToken => false,
        Kind::ExclamationEqualsEqualsToken => true,
        _ => return,
    };
    let binary = node.as_binary_expression();
    let left = skip_parens(binary.left());
    let right = skip_parens(binary.right());
    let (typeof_node, compared) = if ast::is_type_of_expression(left) {
        (left, right)
    } else if ast::is_type_of_expression(right) {
        (right, left)
    } else {
        return;
    };
    let Some(condition) = string_literal_text(compared).and_then(TypeofValue::parse) else {
        return;
    };
    let operand = skip_parens(typeof_node.as_type_of_expression().expression());
    let Some(possible) = typeof_values_of_operand(ctx.checker, ctx.program, operand) else {
        return;
    };
    let included = possible.contains(&condition);
    let message = match (possible.as_slice(), included, negated) {
        ([_], true, false) => {
            unnecessary_typeof_message(typeof_operand_name(ctx, operand), condition.as_str())
        }
        ([only], false, true) => {
            unnecessary_typeof_message(typeof_operand_name(ctx, operand), only.as_str())
        }
        ([_], _, _) | (_, false, false) => always_false_typeof_message(
            typeof_operand_name(ctx, operand),
            &describe_typeof_values(&possible),
            condition.as_str(),
        ),
        (_, false, true) => unnecessary_typeof_message(
            typeof_operand_name(ctx, operand),
            &describe_typeof_values(&possible),
        ),
        (_, true, _) => return,
    };
    ctx.report_node(node, message);
}

fn literal_string_value(t: P<Type>) -> Option<&'static str> {
    match t.as_literal_type().value() {
        Some(LiteralValue::String(value)) => Some(value),
        _ => None,
    }
}

fn string_literal_values_of_type(t: P<Type>) -> Option<Vec<&'static str>> {
    if t.flags().intersects(TypeFlags::Any | TypeFlags::Unknown) {
        return None;
    }
    if t.flags().intersects(TypeFlags::Union) {
        let mut values = Vec::with_capacity(t.types().len());
        for part in t.types() {
            if !part.flags().intersects(TypeFlags::StringLiteral) {
                return None;
            }
            values.push(literal_string_value(*part)?);
        }
        return Some(values);
    }
    if t.flags().intersects(TypeFlags::StringLiteral) {
        return Some(vec![literal_string_value(t)?]);
    }
    None
}

fn string_literal_values_of_expression(
    checker: &mut Checker,
    expression: P<Node>,
) -> Option<Vec<&'static str>> {
    let t = checker.get_type_at_location(expression);
    if let Some(values) = string_literal_values_of_type(t) {
        return Some(values);
    }
    let symbol = checker.get_symbol_at_location_exported(expression)?;
    let symbol_type = checker.get_type_of_symbol_at_location(symbol, Some(expression))?;
    string_literal_values_of_type(symbol_type)
}

/// The string literal values of a variable declaration's type annotation: `Some` for a string literal type or
/// a union of them, `None` for a union with other members, and no decision (outer `None`) for other annotations.
fn annotated_string_literals(declaration: P<Node>) -> Option<Option<Vec<&'static str>>> {
    let annotation = ast::skip_type_parentheses(declaration.type_node()?);
    if ast::is_union_type_node(annotation) {
        let members = annotation.as_union_type_node().types().nodes();
        let values: Option<Vec<&'static str>> = members
            .iter()
            .map(|member| literal_type_string(*member))
            .collect();
        return Some(values);
    }
    literal_type_string(annotation).map(|value| Some(vec![value]))
}

fn literal_type_string(type_node: P<Node>) -> Option<&'static str> {
    let type_node = ast::skip_type_parentheses(type_node);
    if !ast::is_literal_type_node(type_node) {
        return None;
    }
    string_literal_text(type_node.as_literal_type_node().literal())
}

/// The annotated literals of the file's top-level variables.
fn collect_annotated_literals(node: P<Node>, found: &mut AnnotatedLiterals) {
    if ast::is_variable_declaration(node)
        && node.parent().is_some_and(ast::is_variable_declaration_list)
    {
        let name = node.as_variable_declaration().name;
        if ast::is_identifier(name) && !found.contains_key(name.text()) {
            if let Some(values) = annotated_string_literals(node) {
                found.insert(name.text(), values);
            }
        }
        return;
    }
    if ast::is_source_file(node)
        || ast::is_variable_statement(node)
        || ast::is_variable_declaration_list(node)
    {
        node.for_each_child(&mut |child| {
            collect_annotated_literals(child, found);
            false
        });
    }
}

/// Whether `identifier` refers to something declared in this file, whose type the checker already gave.
fn is_declared_in_file(ctx: &mut Ctx, identifier: P<Node>) -> bool {
    ctx.checker
        .get_symbol_at_location_exported(identifier)
        .is_some_and(|symbol| {
            symbol
                .declarations()
                .iter()
                .any(|&declaration| ast::get_source_file_of_node(declaration) == Some(ctx.file))
        })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StringMethod {
    StartsWith,
    EndsWith,
    Includes,
}

impl StringMethod {
    fn parse(name: &str) -> Option<StringMethod> {
        match name {
            "startsWith" => Some(StringMethod::StartsWith),
            "endsWith" => Some(StringMethod::EndsWith),
            "includes" => Some(StringMethod::Includes),
            _ => None,
        }
    }

    fn evaluate(self, value: &str, search: &str) -> bool {
        match self {
            StringMethod::StartsWith => value.starts_with(search),
            StringMethod::EndsWith => value.ends_with(search),
            StringMethod::Includes => value.contains(search),
        }
    }

    fn message(self, always_true: bool) -> RuleMessage {
        match (self, always_true) {
            (StringMethod::StartsWith, true) => RuleMessage::new(
                "unnecessaryStartsWithCondition",
                "This startsWith check is unnecessary as it always evaluates to true.",
            ),
            (StringMethod::StartsWith, false) => RuleMessage::new(
                "alwaysFalseStartsWithCondition",
                "This startsWith check will always be false.",
            ),
            (StringMethod::EndsWith, true) => RuleMessage::new(
                "unnecessaryEndsWithCondition",
                "This endsWith check is unnecessary as it always evaluates to true.",
            ),
            (StringMethod::EndsWith, false) => RuleMessage::new(
                "alwaysFalseEndsWithCondition",
                "This endsWith check will always be false.",
            ),
            (StringMethod::Includes, true) => RuleMessage::new(
                "unnecessaryIncludesCondition",
                "This includes check is unnecessary as it always evaluates to true.",
            ),
            (StringMethod::Includes, false) => RuleMessage::new(
                "alwaysFalseIncludesCondition",
                "This includes check will always be false.",
            ),
        }
    }
}

/// `Some(result)` when `predicate` gives the same result for every item, `None` when the results are mixed.
fn constant_result<T>(items: &[T], mut predicate: impl FnMut(&T) -> bool) -> Option<bool> {
    let (first, rest) = items.split_first()?;
    let result = predicate(first);
    rest.iter()
        .all(|item| predicate(item) == result)
        .then_some(result)
}

/// The non-computed member expression callee of a call, as ESTree sees it.
fn member_callee(call: P<Node>) -> Option<P<Node>> {
    let callee = skip_parens(call.expression()?);
    if !ast::is_property_access_expression(callee) || is_estree_chain_expression(callee) {
        return None;
    }
    Some(callee)
}

impl Visitor {
    fn check_string_assertion(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let Some(callee) = member_callee(node) else {
            return;
        };
        let access = callee.as_property_access_expression();
        if !ast::is_identifier(access.name) {
            return;
        }
        let Some(method) = StringMethod::parse(access.name.text()) else {
            return;
        };
        let [argument] = node.arguments() else {
            return;
        };
        let Some(search) = string_literal_text(skip_parens(*argument)) else {
            return;
        };
        let object = skip_parens(access.expression);
        let Some(values) = string_literal_values_of_expression(ctx.checker, object)
            .or_else(|| self.annotated_literals_of(ctx, object))
        else {
            return;
        };
        if method == StringMethod::Includes && values.len() > 1 {
            return;
        }
        let Some(always_true) = constant_result(&values, |value| method.evaluate(value, search))
        else {
            return;
        };
        ctx.report_node(node, method.message(always_true));
    }

    /// Fallback for identifiers that resolve to no declaration of this file (a script's `const status: 'a'` clashing
    /// with the DOM's global `status: string`): the type annotation of the first top-level variable declared with that
    /// name.
    fn annotated_literals_of(
        &mut self,
        ctx: &mut Ctx,
        object: P<Node>,
    ) -> Option<Vec<&'static str>> {
        if !ast::is_identifier(object) || is_declared_in_file(ctx, object) {
            return None;
        }
        let found = self.annotated_literals.get_or_insert_with(|| {
            let mut found = AnnotatedLiterals::default();
            collect_annotated_literals(ctx.file.as_node(), &mut found);
            found
        });
        found.get(object.text()).cloned().flatten()
    }

    fn check_in_condition(&self, ctx: &mut Ctx, node: P<Node>) {
        let binary = node.as_binary_expression();
        let left = skip_parens(binary.left());
        if ast::is_private_identifier(left) {
            return;
        }
        let Some(property) = literal_key_of_expression(ctx.checker, left) else {
            return;
        };
        if property.is_empty() || property == "__proto__" {
            return;
        }
        let right = skip_parens(binary.right());
        let t = ctx.checker.get_type_at_location(right);
        let message = match property_presence(ctx.checker, t, property) {
            PropertyPresence::Required => RuleMessage::new(
                "unnecessaryInCondition",
                format!(
                    "This in check is unnecessary. Property \"{property}\" always exists on type \"{}\".{}",
                    type_to_string(ctx.checker, t),
                    self.rule.in_condition_alternative
                ),
            ),
            PropertyPresence::Absent => RuleMessage::new(
                "alwaysFalseInCondition",
                format!(
                    "This in check will always be false. Property \"{property}\" does not exist on type \"{}\".{}",
                    type_to_string(ctx.checker, t),
                    self.rule.in_condition_alternative
                ),
            ),
            PropertyPresence::Optional | PropertyPresence::Unknown => return,
        };
        ctx.report_node(node, message);
    }
}

/// The library's `Array` or `ReadonlyArray`, not a local type with the same name (`Data.Array`).
fn is_array_symbol(program: &Program, symbol: P<Symbol>) -> bool {
    matches!(symbol.name(), "Array" | "ReadonlyArray") && is_default_lib_symbol(program, symbol)
}

fn is_array_or_tuple_type(checker: &Checker, t: P<Type>) -> bool {
    checker.is_array_type(t) || is_tuple_type(t)
}

fn is_runtime_array_type(checker: &mut Checker, program: &Program, t: P<Type>) -> bool {
    if is_array_or_tuple_type(checker, t) {
        return true;
    }
    let apparent = checker.get_apparent_type(t);
    if apparent != t && is_array_or_tuple_type(checker, apparent) {
        return true;
    }
    if let Some(alias_symbol) = t.alias().and_then(|alias| alias.symbol()) {
        if is_array_symbol(program, alias_symbol) {
            return true;
        }
    }
    t.symbol()
        .is_some_and(|symbol| is_array_symbol(program, symbol))
}

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

fn is_definitely_not_array_type(checker: &mut Checker, program: &Program, t: P<Type>) -> bool {
    let flags = t.flags();
    if flags.intersects(
        TypeFlags::StringLike
            | TypeFlags::NumberLike
            | TypeFlags::BigIntLike
            | TypeFlags::BooleanLike
            | TypeFlags::ESSymbolLike
            | TypeFlags::EnumLike
            | TypeFlags::Null
            | TypeFlags::Undefined
            | TypeFlags::Void
            | TypeFlags::Never,
    ) {
        return true;
    }
    flags.intersects(TypeFlags::Object)
        && !t.object_flags().intersects(ObjectFlags::ClassOrInterface)
        && !is_class_or_interface_reference(t)
        && !is_runtime_array_type(checker, program, t)
        && !is_empty_object_type(checker, t)
}

/// An instantiated generic class or interface (`Iterable<T>`, `ArrayLike<T>`), which an array may implement, like the
/// non-generic ones.
fn is_class_or_interface_reference(t: P<Type>) -> bool {
    t.object_flags().intersects(ObjectFlags::Reference)
        && is_symbol_flag_set(t.symbol(), SymbolFlags::Class | SymbolFlags::Interface)
}

/// `Some(result)` when `Array.isArray` always gives `result` for a value of type `t`.
fn evaluate_array_is_array(checker: &mut Checker, program: &Program, t: P<Type>) -> Option<bool> {
    if t.flags().intersects(TypeFlags::Any | TypeFlags::Unknown) {
        return None;
    }
    if t.flags().intersects(TypeFlags::Union) {
        let mut all_true = true;
        let mut all_false = true;
        for part in t.types() {
            let result = evaluate_array_is_array(checker, program, *part)?;
            all_true &= result;
            all_false &= !result;
        }
        return match (all_true, all_false) {
            (true, _) => Some(true),
            (_, true) => Some(false),
            _ => None,
        };
    }
    if is_runtime_array_type(checker, program, t) {
        return Some(true);
    }
    is_definitely_not_array_type(checker, program, t).then_some(false)
}

/// `Array.isArray` with the global `Array`, not a local binding named `Array`.
fn is_array_is_array_callee(ctx: &mut Ctx, callee: P<Node>) -> bool {
    let access = callee.as_property_access_expression();
    let object = skip_parens(access.expression);
    ast::is_identifier(object)
        && object.text() == "Array"
        && ast::is_identifier(access.name)
        && access.name.text() == "isArray"
        && ctx
            .checker
            .get_symbol_at_location_exported(object)
            .is_some_and(|symbol| is_default_lib_symbol(ctx.program, symbol))
}

fn check_array_is_array(ctx: &mut Ctx, node: P<Node>) {
    let Some(callee) = member_callee(node) else {
        return;
    };
    if !is_array_is_array_callee(ctx, callee) {
        return;
    }
    let Some(first) = node.arguments().first() else {
        return;
    };
    if ast::is_spread_element(*first) {
        return;
    }
    let t = ctx.checker.get_type_at_location(skip_parens(*first));
    let message = match evaluate_array_is_array(ctx.checker, ctx.program, t) {
        Some(true) => RuleMessage::new(
            "unnecessaryArrayIsArrayCondition",
            "This Array.isArray check is unnecessary as it always evaluates to true.",
        ),
        Some(false) => RuleMessage::new(
            "alwaysFalseArrayIsArrayCondition",
            "This Array.isArray check will always be false.",
        ),
        None => return,
    };
    ctx.report_node(node, message);
}

fn compare_numbers(operator: Kind, left: f64, right: f64) -> Option<bool> {
    Some(match operator {
        Kind::EqualsEqualsEqualsToken => left == right,
        Kind::ExclamationEqualsEqualsToken => left != right,
        Kind::GreaterThanToken => left > right,
        Kind::GreaterThanEqualsToken => left >= right,
        Kind::LessThanToken => left < right,
        Kind::LessThanEqualsToken => left <= right,
        _ => return None,
    })
}

/// The possible lengths of `expression` when it is `x.length` with `x` a string literal union.
fn string_literal_lengths(checker: &mut Checker, expression: P<Node>) -> Option<Vec<f64>> {
    if !ast::is_property_access_expression(expression) || is_estree_chain_expression(expression) {
        return None;
    }
    let access = expression.as_property_access_expression();
    if !ast::is_identifier(access.name) || access.name.text() != "length" {
        return None;
    }
    let values = string_literal_values_of_expression(checker, skip_parens(access.expression))?;
    Some(
        values
            .iter()
            .map(|value| value.encode_utf16().count() as f64)
            .collect(),
    )
}

fn check_length_comparison(ctx: &mut Ctx, node: P<Node>, operator: Kind) {
    if compare_numbers(operator, 0.0, 0.0).is_none() {
        return;
    }
    let binary = node.as_binary_expression();
    let left = skip_parens(binary.left());
    let right = skip_parens(binary.right());
    let (length_expression, literal, length_on_left) =
        if let Some(value) = numeric_literal_value(right) {
            (left, value, true)
        } else if let Some(value) = numeric_literal_value(left) {
            (right, value, false)
        } else {
            return;
        };
    let Some(lengths) = string_literal_lengths(ctx.checker, length_expression) else {
        return;
    };
    let always_true = constant_result(&lengths, |&length| {
        let (l, r) = if length_on_left {
            (length, literal)
        } else {
            (literal, length)
        };
        compare_numbers(operator, l, r).unwrap_or(false)
    });
    let message = match always_true {
        Some(true) => RuleMessage::new(
            "unnecessaryLengthCondition",
            "This length comparison is unnecessary as it always evaluates to true.",
        ),
        Some(false) => RuleMessage::new(
            "alwaysFalseLengthCondition",
            "This length comparison will always be false.",
        ),
        None => return,
    };
    ctx.report_node(node, message);
}

fn literal_key_of_expression(checker: &mut Checker, expression: P<Node>) -> Option<&'static str> {
    if let Some(value) = string_literal_text(expression) {
        return Some(value);
    }
    match string_literal_values_of_expression(checker, expression)?.as_slice() {
        [value] => Some(value),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PropertyPresence {
    Required,
    Optional,
    Absent,
    Unknown,
}

fn property_presence_in_object(
    checker: &mut Checker,
    t: P<Type>,
    property: &str,
) -> PropertyPresence {
    if !checker.get_index_infos_of_type(t).is_empty() {
        return PropertyPresence::Unknown;
    }
    if checker.get_properties_of_type(t).is_empty() {
        return PropertyPresence::Unknown;
    }
    let Some(symbol) = checker.get_property_of_type(t, property) else {
        return PropertyPresence::Absent;
    };
    if is_symbol_flag_set(Some(symbol), SymbolFlags::Optional) {
        PropertyPresence::Optional
    } else {
        PropertyPresence::Required
    }
}

fn property_presence(checker: &mut Checker, t: P<Type>, property: &str) -> PropertyPresence {
    let flags = t.flags();
    if flags.intersects(
        TypeFlags::Any
            | TypeFlags::Unknown
            | TypeFlags::Never
            | TypeFlags::TypeParameter
            | TypeFlags::IndexedAccess
            | TypeFlags::StringLike
            | TypeFlags::NumberLike
            | TypeFlags::BigIntLike
            | TypeFlags::BooleanLike
            | TypeFlags::EnumLike
            | TypeFlags::ESSymbolLike
            | TypeFlags::NonPrimitive
            | TypeFlags::Null
            | TypeFlags::Undefined
            | TypeFlags::Void,
    ) {
        return PropertyPresence::Unknown;
    }
    if flags.intersects(TypeFlags::Union) {
        let mut all_required = true;
        let mut all_absent = true;
        for part in t.types() {
            match property_presence(checker, *part, property) {
                PropertyPresence::Unknown | PropertyPresence::Optional => {
                    return PropertyPresence::Unknown
                }
                PropertyPresence::Required => all_absent = false,
                PropertyPresence::Absent => all_required = false,
            }
        }
        return match (all_required, all_absent) {
            (true, _) => PropertyPresence::Required,
            (_, true) => PropertyPresence::Absent,
            _ => PropertyPresence::Unknown,
        };
    }
    if flags.intersects(TypeFlags::Intersection) {
        let mut has_required = false;
        let mut has_optional = false;
        for part in t.types() {
            match property_presence(checker, *part, property) {
                PropertyPresence::Unknown => return PropertyPresence::Unknown,
                PropertyPresence::Required => has_required = true,
                PropertyPresence::Optional => has_optional = true,
                PropertyPresence::Absent => {}
            }
        }
        return match (has_required, has_optional) {
            (true, _) => PropertyPresence::Required,
            (_, true) => PropertyPresence::Optional,
            _ => PropertyPresence::Absent,
        };
    }
    if !flags.intersects(TypeFlags::Object) {
        return PropertyPresence::Unknown;
    }
    property_presence_in_object(checker, t, property)
}
