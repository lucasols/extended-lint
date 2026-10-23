use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleSuggestion, RuleVisitor};

use crate::scope::{binding_symbol, ReferenceIndex};
use crate::utils::{
    is_estree_chain_expression, is_var_in_nested_block, node_text, skip_parens, skip_type_parens,
};

pub const NAME: &str = "no-optional-root-props";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoOptionalRootProps;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoOptionalRootProps))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::TypeAliasDeclaration),
    Listener::Enter(Kind::InterfaceDeclaration),
    Listener::Enter(Kind::TypeReference),
];

impl Rule for NoOptionalRootProps {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { references: None })
    }
}

struct Visitor {
    references: Option<ReferenceIndex>,
}

/// The ESTree node types the rule looks at while walking up from a reference; everything else is `Other`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EsKind {
    Program,
    ExportNamed,
    ExportDefault,
    VariableDeclaration,
    VariableDeclarator,
    AssignmentPattern,
    ArrowFunction,
    FunctionDeclaration,
    /// Any other node with a `returnType` key: function expressions, `TSDeclareFunction`, signatures and function
    /// types.
    FunctionLike,
    TypeParameterInstantiation,
    TypeReference,
    CallExpression,
    Other,
}

impl EsKind {
    fn has_return_type(self) -> bool {
        matches!(
            self,
            EsKind::ArrowFunction | EsKind::FunctionDeclaration | EsKind::FunctionLike
        )
    }
}

/// An ESTree node: its type and the TS node it was built from (for a `TSTypeParameterInstantiation`, the node
/// owning the type arguments).
#[derive(Clone, Copy)]
struct EsNode {
    kind: EsKind,
    node: P<Node>,
}

/// The ESTree node of `node` and its ESTree ancestors up to `Program`, emulating typescript-estree's tree on top of
/// the TS AST: parentheses, `NamedExports`, `JsxAttributes`, ... have no ESTree node, while type annotations, type
/// argument lists, export declarations, method values and class/interface bodies are extra nodes.
fn estree_chain(start: P<Node>) -> Vec<EsNode> {
    let mut chain = vec![EsNode {
        kind: primary_kind(start),
        node: start,
    }];
    push_chain_expression(&mut chain, start);
    let mut child = start;
    while let Some(parent) = child.parent() {
        push_parent(&mut chain, child, parent);
        child = parent;
    }
    chain
}

fn primary_kind(node: P<Node>) -> EsKind {
    match node.kind() {
        Kind::SourceFile => EsKind::Program,
        Kind::VariableDeclarationList => EsKind::VariableDeclaration,
        Kind::VariableDeclaration => EsKind::VariableDeclarator,
        Kind::ArrowFunction => EsKind::ArrowFunction,
        Kind::FunctionDeclaration if node.body().is_some() => EsKind::FunctionDeclaration,
        Kind::FunctionDeclaration
        | Kind::FunctionExpression
        | Kind::MethodSignature
        | Kind::CallSignature
        | Kind::ConstructSignature
        | Kind::FunctionType
        | Kind::ConstructorType => EsKind::FunctionLike,
        Kind::CallExpression => EsKind::CallExpression,
        Kind::TypeReference => EsKind::TypeReference,
        Kind::ExportDeclaration => EsKind::ExportNamed,
        Kind::ExportAssignment if !node.as_export_assignment().is_export_equals() => {
            EsKind::ExportDefault
        }
        _ => EsKind::Other,
    }
}

fn push(chain: &mut Vec<EsNode>, kind: EsKind, node: P<Node>) {
    chain.push(EsNode { kind, node });
}

fn push_chain_expression(chain: &mut Vec<EsNode>, node: P<Node>) {
    if matches!(
        node.kind(),
        Kind::PropertyAccessExpression
            | Kind::ElementAccessExpression
            | Kind::CallExpression
            | Kind::NonNullExpression
    ) && is_estree_chain_expression(node)
    {
        push(chain, EsKind::Other, node);
    }
}

fn push_export_wrapper(chain: &mut Vec<EsNode>, node: P<Node>) {
    let flags = node.modifier_flags();
    if flags.intersects(ModifierFlags::Default) {
        push(chain, EsKind::ExportDefault, node);
    } else if flags.intersects(ModifierFlags::Export) {
        push(chain, EsKind::ExportNamed, node);
    }
}

fn push_parent(chain: &mut Vec<EsNode>, child: P<Node>, parent: P<Node>) {
    let is_return_type = || parent.type_node() == Some(child);
    match parent.kind() {
        Kind::ParenthesizedExpression
        | Kind::ParenthesizedType
        | Kind::NamedExports
        | Kind::NamedImports
        | Kind::ImportClause
        | Kind::JsxAttributes
        | Kind::HeritageClause
        | Kind::CaseBlock
        | Kind::TemplateSpan
        | Kind::ComputedPropertyName => {}
        Kind::VariableStatement => push_export_wrapper(chain, parent),
        Kind::FunctionDeclaration
        | Kind::ClassDeclaration
        | Kind::InterfaceDeclaration
        | Kind::TypeAliasDeclaration
        | Kind::EnumDeclaration
        | Kind::ModuleDeclaration => {
            let has_inner_node = child.kind() == Kind::TypeParameter
                || (parent.kind() == Kind::FunctionDeclaration && is_return_type())
                || (matches!(
                    parent.kind(),
                    Kind::ClassDeclaration | Kind::InterfaceDeclaration
                ) && parent.members().contains(&child));
            if has_inner_node {
                push(chain, EsKind::Other, parent);
            }
            push(chain, primary_kind(parent), parent);
            push_export_wrapper(chain, parent);
        }
        Kind::ClassExpression => {
            if child.kind() == Kind::TypeParameter || parent.members().contains(&child) {
                push(chain, EsKind::Other, parent);
            }
            push(chain, EsKind::Other, parent);
        }
        Kind::FunctionExpression
        | Kind::ArrowFunction
        | Kind::MethodSignature
        | Kind::CallSignature
        | Kind::ConstructSignature
        | Kind::FunctionType
        | Kind::ConstructorType => {
            if child.kind() == Kind::TypeParameter || is_return_type() {
                push(chain, EsKind::Other, parent);
            }
            push(chain, primary_kind(parent), parent);
        }
        Kind::MethodDeclaration | Kind::Constructor | Kind::GetAccessor | Kind::SetAccessor => {
            let is_value_part = parent.name() != Some(child)
                && !matches!(child.kind(), Kind::Decorator)
                && !parent.modifier_nodes().contains(&child);
            if is_value_part {
                if child.kind() == Kind::TypeParameter || is_return_type() {
                    push(chain, EsKind::Other, parent);
                }
                push(chain, EsKind::FunctionLike, parent);
            }
            push(chain, EsKind::Other, parent);
        }
        Kind::Parameter => push_parameter_parent(chain, child, parent),
        Kind::VariableDeclaration => {
            if parent.type_node() == Some(child) {
                push(chain, EsKind::Other, parent);
                push(chain, EsKind::Other, parent);
            }
            push(chain, EsKind::VariableDeclarator, parent);
        }
        Kind::BindingElement => push_binding_element_parent(chain, child, parent),
        Kind::PropertySignature
        | Kind::PropertyDeclaration
        | Kind::IndexSignature
        | Kind::TypePredicate => {
            if parent.type_node() == Some(child) {
                push(chain, EsKind::Other, parent);
            }
            push(chain, EsKind::Other, parent);
        }
        Kind::TypeReference
        | Kind::CallExpression
        | Kind::NewExpression
        | Kind::TaggedTemplateExpression
        | Kind::ExpressionWithTypeArguments
        | Kind::TypeQuery
        | Kind::ImportType
        | Kind::JsxOpeningElement
        | Kind::JsxSelfClosingElement => {
            if parent.type_arguments().contains(&child) {
                push(chain, EsKind::TypeParameterInstantiation, parent);
            }
            push(chain, primary_kind(parent), parent);
            if parent.kind() == Kind::JsxSelfClosingElement {
                push(chain, EsKind::Other, parent);
            }
            push_chain_expression(chain, parent);
        }
        _ => {
            push(chain, primary_kind(parent), parent);
            push_chain_expression(chain, parent);
        }
    }
}

/// A parameter is its name (an identifier or a pattern) in ESTree, wrapped in an `AssignmentPattern` when it has a
/// default value, a `RestElement` when it is a rest parameter (which then holds the type annotation) and a
/// `TSParameterProperty` when it is a parameter property.
fn push_parameter_parent(chain: &mut Vec<EsNode>, child: P<Node>, parent: P<Node>) {
    let is_rest = parent
        .as_parameter_declaration()
        .dot_dot_dot_token()
        .is_some();
    if parent.type_node() == Some(child) {
        push(chain, EsKind::Other, parent);
        if !is_rest {
            push(chain, EsKind::Other, parent);
        }
    }
    if is_rest {
        push(chain, EsKind::Other, parent);
    } else if parent.initializer().is_some() {
        push(chain, EsKind::AssignmentPattern, parent);
    }
    if ast::has_syntactic_modifier(parent, ModifierFlags::ParameterPropertyModifier) {
        push(chain, EsKind::Other, parent);
    }
}

fn push_binding_element_parent(chain: &mut Vec<EsNode>, child: P<Node>, parent: P<Node>) {
    let in_object_pattern = parent
        .parent()
        .is_some_and(|pattern| pattern.kind() == Kind::ObjectBindingPattern);
    if parent.as_binding_element().dot_dot_dot_token().is_some() {
        push(chain, EsKind::Other, parent);
        return;
    }
    let is_key = parent.property_name() == Some(child);
    if !is_key && parent.initializer().is_some() {
        push(chain, EsKind::AssignmentPattern, parent);
    }
    if in_object_pattern {
        push(chain, EsKind::Other, parent);
    }
}

/// typescript-eslint's `findParentNode(chain[index], kind, maxDepth)`.
fn find_parent(
    chain: &[EsNode],
    mut index: usize,
    kind: EsKind,
    mut max_depth: u32,
) -> Option<usize> {
    loop {
        if max_depth == 0 {
            return None;
        }
        let node = chain.get(index)?;
        if index + 1 >= chain.len() {
            return None;
        }
        if node.kind == kind {
            return Some(index);
        }
        index += 1;
        if max_depth != u32::MAX {
            max_depth -= 1;
        }
    }
}

fn is_exported(chain: &[EsNode], index: usize) -> bool {
    chain
        .get(index + 1)
        .is_some_and(|parent| matches!(parent.kind, EsKind::ExportNamed | EsKind::ExportDefault))
}

fn is_memo_call(ref_chain: &[EsNode]) -> bool {
    ref_chain.get(1).is_some_and(|parent| {
        parent.kind == EsKind::CallExpression
            && parent.node.expression().is_some_and(|callee| {
                let callee = skip_parens(callee);
                callee.kind() == Kind::Identifier && callee.text() == "memo"
            })
    })
}

impl Visitor {
    fn references(&mut self, ctx: &Ctx) -> &ReferenceIndex {
        self.references
            .get_or_insert_with(|| ReferenceIndex::new(ctx.file))
    }

    /// The ESTree chain of the only reference to the variable or function declared by `chain[index]`, unless the
    /// declaration or that reference is exported.
    fn valid_single_ref(
        &mut self,
        ctx: &mut Ctx,
        chain: &[EsNode],
        index: usize,
    ) -> Option<Vec<EsNode>> {
        if is_exported(chain, index) {
            return None;
        }
        let declaration = chain.get(index)?;
        let name = match declaration.kind {
            EsKind::VariableDeclaration => {
                let list = declaration.node;
                let [single] = list.as_variable_declaration_list().declarations().nodes() else {
                    return None;
                };
                if is_var_in_nested_block(list) {
                    return None;
                }
                single
                    .name()
                    .filter(|name| name.kind() == Kind::Identifier)?
            }
            _ => declaration.node.name()?,
        };
        let symbol = binding_symbol(ctx.checker, name)?;
        let references = self.references(ctx).references(ctx.checker, symbol);
        let mut uses = references
            .iter()
            .filter(|reference| reference.identifier != name);
        let reference = uses.next()?;
        if uses.next().is_some() {
            return None;
        }
        let ref_chain = estree_chain(reference.identifier);
        if is_exported(&ref_chain, 2) {
            return None;
        }
        Some(ref_chain)
    }

    fn is_fc_props_of_valid_component(
        &mut self,
        ctx: &mut Ctx,
        chain: &[EsNode],
        index: usize,
    ) -> bool {
        let Some(declaration) = find_parent(chain, index, EsKind::VariableDeclaration, 4) else {
            return false;
        };
        self.valid_single_ref(ctx, chain, declaration)
            .is_some_and(|ref_chain| !is_memo_call(&ref_chain))
    }

    fn is_referenced_only_once(&mut self, ctx: &mut Ctx, declaration: P<Node>) -> bool {
        let Some(name) = declaration.name() else {
            return false;
        };
        let Some(symbol) = binding_symbol(ctx.checker, name) else {
            return false;
        };
        let references = self.references(ctx).references(ctx.checker, symbol);
        let [reference] = references.as_slice() else {
            return false;
        };
        let chain = estree_chain(reference.identifier);
        for (index, node) in chain.iter().enumerate() {
            if node.kind.has_return_type() || node.kind == EsKind::ExportNamed {
                return false;
            }
            let Some(parent) = chain.get(index + 1) else {
                return false;
            };
            if parent.kind == EsKind::TypeParameterInstantiation
                && chain
                    .get(index + 2)
                    .is_some_and(|reference| reference.kind == EsKind::TypeReference)
                && is_fc_reference(parent.node)
            {
                return self.is_fc_props_of_valid_component(ctx, &chain, index + 3);
            }
            if node.kind == EsKind::AssignmentPattern {
                return false;
            }
            if parent.kind == EsKind::ArrowFunction {
                let Some(declaration) =
                    find_parent(&chain, index + 1, EsKind::VariableDeclaration, u32::MAX)
                else {
                    return false;
                };
                return self.valid_single_ref(ctx, &chain, declaration).is_some();
            }
            if parent.kind == EsKind::FunctionDeclaration {
                return self.valid_single_ref(ctx, &chain, index + 1).is_some();
            }
        }
        false
    }
}

/// Whether `node` is a `FC<...>` type reference.
fn is_fc_reference(node: P<Node>) -> bool {
    node.kind() == Kind::TypeReference && {
        let type_name = node.as_type_reference_node().type_name;
        type_name.kind() == Kind::Identifier && type_name.text() == "FC"
    }
}

/// The key of an optional property signature, when it is an identifier (or a computed `[identifier]`).
fn optional_identifier_key(member: P<Node>) -> Option<P<Node>> {
    if member.kind() != Kind::PropertySignature || member.question_token().is_none() {
        return None;
    }
    let mut key = member.name()?;
    if key.kind() == Kind::ComputedPropertyName {
        key = key.expression()?;
    }
    (key.kind() == Kind::Identifier).then_some(key)
}

fn report_optional_properties(ctx: &mut Ctx, members: &[P<Node>]) {
    for &member in members {
        let Some(key) = optional_identifier_key(member) else {
            continue;
        };
        let message = RuleMessage::new(
            "optionalNotAllowed",
            format!(
                "Optional property \"{}\" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.",
                key.text()
            ),
        );
        ctx.report_node_with_suggestions(key, message, |ctx| {
            let replacement = node_text(ctx, member).replacen("?:", ": undefined |", 1);
            vec![RuleSuggestion {
                message: RuleMessage::new("suggestion", "Use `prop: undefined | ...` instead."),
                fixes: vec![ctx.fix_replace(member, replacement)],
            }]
        });
    }
}

fn has_optional_property(members: &[P<Node>]) -> bool {
    members
        .iter()
        .any(|&member| optional_identifier_key(member).is_some())
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        match node.kind() {
            Kind::TypeAliasDeclaration | Kind::InterfaceDeclaration => {
                let members = if node.kind() == Kind::InterfaceDeclaration {
                    node.members()
                } else {
                    let Some(type_node) = node.type_node().map(skip_type_parens) else {
                        return;
                    };
                    if type_node.kind() != Kind::TypeLiteral {
                        return;
                    }
                    type_node.members()
                };
                if !has_optional_property(members)
                    || node
                        .modifier_flags()
                        .intersects(ModifierFlags::Export | ModifierFlags::Default)
                    || !self.is_referenced_only_once(ctx, node)
                {
                    return;
                }
                report_optional_properties(ctx, members);
            }
            Kind::TypeReference => {
                if !is_fc_reference(node) {
                    return;
                }
                let Some(&props) = node.type_arguments().first() else {
                    return;
                };
                let props = skip_type_parens(props);
                if props.kind() != Kind::TypeLiteral || !has_optional_property(props.members()) {
                    return;
                }
                let chain = estree_chain(node);
                if !self.is_fc_props_of_valid_component(ctx, &chain, 1) {
                    return;
                }
                report_optional_properties(ctx, props.members());
            }
            _ => {}
        }
    }
}
