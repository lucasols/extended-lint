use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleDiagnostic, RuleFix, RuleMessage, RuleVisitor};

use crate::utils::{
    estree_declaration_start, skip_type_parens, start_with_comments_before,
    text_with_comments_before,
};

pub const NAME: &str = "use-types-directly-above-usage";

#[derive(serde::Deserialize, Clone, Copy, PartialEq, Eq)]
enum CheckContext {
    #[serde(rename = "function-args")]
    FunctionArgs,
    #[serde(rename = "FC")]
    Fc,
    #[serde(rename = "generic-args-at-fn-calls")]
    GenericArgsAtFnCalls,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    check_only: Vec<CheckContext>,
    /// Accepted for config compatibility; the ESLint rule never used it.
    #[serde(rename = "checkTypesFromSelectors")]
    _check_types_from_selectors: Option<Vec<String>>,
}

pub struct UseTypesDirectlyAboveUsage {
    check_only: Vec<CheckContext>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    Ok(Box::new(UseTypesDirectlyAboveUsage {
        check_only: options.check_only,
    }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::TypeReference),
    Listener::Enter(Kind::EndOfFile),
];

impl Rule for UseTypesDirectlyAboveUsage {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            rule: self,
            references: Vec::new(),
        })
    }
}

struct TypeReference {
    name: &'static str,
    statement: Option<P<Node>>,
    position: i32,
    in_function_args: bool,
    in_fc_props: bool,
    in_generic_arg_at_call: bool,
    in_type_declaration: bool,
}

struct Visitor {
    rule: &'static UseTypesDirectlyAboveUsage,
    references: Vec<TypeReference>,
}

struct TypeDefinition {
    index: usize,
}

struct ToProcess {
    name: &'static str,
    first_usage: usize,
    position: i32,
    in_function_args: bool,
}

struct ToMove {
    definition: usize,
    target: usize,
    position: i32,
}

/// The parent of a type node, skipping parenthesized types (which ESTree doesn't have).
fn estree_type_parent(node: P<Node>) -> Option<P<Node>> {
    let mut parent = node.parent()?;
    while parent.kind() == Kind::ParenthesizedType {
        parent = parent.parent()?;
    }
    Some(parent)
}

fn is_part_of_indexed_access(node: P<Node>) -> bool {
    estree_type_parent(node).is_some_and(|parent| {
        parent.kind() == Kind::IndexedAccessType
            && skip_type_parens(parent.as_indexed_access_type_node().object_type()) == node
    })
}

/// `(child, parent)` pairs from `node` up to the root.
fn ancestor_pairs(node: P<Node>) -> impl Iterator<Item = (P<Node>, P<Node>)> {
    std::iter::successors(
        node.parent().map(|parent| (node, parent)),
        |&(_, current)| current.parent().map(|parent| (current, parent)),
    )
}

/// ESTree `FunctionDeclaration`, `FunctionExpression` and `ArrowFunctionExpression`: methods, accessors and
/// constructors are `FunctionExpression`s there, while body-less ones are `TSDeclareFunction`s or
/// `TSEmptyBodyFunctionExpression`s.
fn is_estree_function(node: P<Node>) -> bool {
    match node.kind() {
        Kind::ArrowFunction | Kind::FunctionExpression => true,
        Kind::FunctionDeclaration
        | Kind::MethodDeclaration
        | Kind::GetAccessor
        | Kind::SetAccessor
        | Kind::Constructor => node.body().is_some(),
        _ => false,
    }
}

fn is_in_function_argument(node: P<Node>) -> bool {
    ancestor_pairs(node)
        .find(|&(_, parent)| is_estree_function(parent))
        .is_some_and(|(child, _)| child.kind() == Kind::Parameter)
}

fn is_fc_type_name(type_name: P<Node>) -> bool {
    match type_name.kind() {
        Kind::Identifier => type_name.as_identifier().text() == "FC",
        Kind::QualifiedName => {
            let right = type_name.as_qualified_name().right;
            ast::is_identifier(right) && right.as_identifier().text() == "FC"
        }
        _ => false,
    }
}

fn is_in_fc_props(node: P<Node>) -> bool {
    ancestor_pairs(node).any(|(child, parent)| {
        parent.kind() == Kind::TypeReference
            && parent.type_arguments().contains(&child)
            && is_fc_type_name(parent.as_type_reference_node().type_name())
    })
}

fn is_in_generic_arg_at_function_call(node: P<Node>) -> bool {
    ancestor_pairs(node).any(|(child, parent)| {
        matches!(parent.kind(), Kind::CallExpression | Kind::NewExpression)
            && parent.type_arguments().contains(&child)
    })
}

fn is_in_type_declaration(node: P<Node>) -> bool {
    ancestor_pairs(node).any(|(_, parent)| {
        matches!(
            parent.kind(),
            Kind::TypeAliasDeclaration | Kind::InterfaceDeclaration
        )
    })
}

fn top_level_statement(node: P<Node>) -> Option<P<Node>> {
    let mut current = node;
    loop {
        let parent = current.parent()?;
        if parent.kind() == Kind::SourceFile {
            return Some(current);
        }
        current = parent;
    }
}

impl Visitor {
    fn matches_check_only(&self, reference: &TypeReference) -> bool {
        let check_only = &self.rule.check_only;
        check_only.is_empty()
            || check_only.iter().any(|context| match context {
                CheckContext::FunctionArgs => reference.in_function_args,
                CheckContext::Fc => reference.in_fc_props,
                CheckContext::GenericArgsAtFnCalls => reference.in_generic_arg_at_call,
            })
    }

    fn check_file(&self, ctx: &mut Ctx) {
        let statements = ctx.file.as_node().statements();
        let statement_index: FxHashMap<P<Node>, usize> = statements
            .iter()
            .enumerate()
            .map(|(index, &statement)| (statement, index))
            .collect();
        let index_of = |statement: Option<P<Node>>| {
            statement.and_then(|statement| statement_index.get(&statement).copied())
        };

        let mut definitions: FxHashMap<&'static str, TypeDefinition> = FxHashMap::default();
        for (index, &statement) in statements.iter().enumerate() {
            if !matches!(
                statement.kind(),
                Kind::TypeAliasDeclaration | Kind::InterfaceDeclaration
            ) || ast::has_syntactic_modifier(statement, ModifierFlags::Default)
            {
                continue;
            }
            let Some(name) = statement.name().filter(|name| ast::is_identifier(*name)) else {
                continue;
            };
            definitions.insert(name.as_identifier().text(), TypeDefinition { index });
        }

        let mut usages: Vec<(&'static str, Vec<usize>)> = Vec::new();
        let mut usage_slot: FxHashMap<&'static str, usize> = FxHashMap::default();
        for reference in &self.references {
            let Some(definition) = definitions.get(reference.name) else {
                continue;
            };
            if !self.matches_check_only(reference) {
                continue;
            }
            let Some(index) = index_of(reference.statement) else {
                continue;
            };
            if index == definition.index {
                continue;
            }
            let slot = *usage_slot.entry(reference.name).or_insert_with(|| {
                usages.push((reference.name, Vec::new()));
                usages.len() - 1
            });
            if let Some((_, statements)) = usages.get_mut(slot) {
                if !statements.contains(&index) {
                    statements.push(index);
                }
            }
        }

        let has_check_only = !self.rule.check_only.is_empty();
        let mut to_process: Vec<ToProcess> = Vec::new();
        for (name, usage_statements) in &usages {
            let Some(&first_usage) = usage_statements.iter().min() else {
                continue;
            };
            let mut position = i32::MAX;
            let mut in_function_args = false;
            for reference in &self.references {
                if reference.name != *name || !self.matches_check_only(reference) {
                    continue;
                }
                if index_of(reference.statement) == Some(first_usage)
                    && reference.position < position
                {
                    position = reference.position;
                    in_function_args = reference.in_function_args;
                }
            }

            let earliest_blocking_usage = self
                .references
                .iter()
                .filter(|r| r.name == *name && (has_check_only || !r.in_type_declaration))
                .filter_map(|r| index_of(r.statement))
                .min();
            if earliest_blocking_usage.is_some_and(|earliest| earliest < first_usage) {
                continue;
            }

            to_process.push(ToProcess {
                name,
                first_usage,
                position,
                in_function_args,
            });
        }

        if to_process.is_empty() {
            return;
        }
        self.report(ctx, &definitions, &to_process, statements);
    }

    fn report(
        &self,
        ctx: &mut Ctx,
        definitions: &FxHashMap<&'static str, TypeDefinition>,
        to_process: &[ToProcess],
        statements: &[P<Node>],
    ) {
        let mut types_by_target: FxHashMap<usize, usize> = FxHashMap::default();
        for item in to_process {
            *types_by_target.entry(item.first_usage).or_default() += 1;
        }
        let lenient_function_args = self.rule.check_only.contains(&CheckContext::FunctionArgs);

        let mut to_move: Vec<ToMove> = Vec::new();
        for item in to_process {
            let Some(definition) = definitions.get(item.name) else {
                continue;
            };
            let (definition_index, usage_index) = (definition.index, item.first_usage);
            if definition_index == usage_index {
                continue;
            }
            let lenient = (lenient_function_args && item.in_function_args)
                || types_by_target
                    .get(&usage_index)
                    .is_some_and(|&count| count > 1);
            let should_move = if lenient {
                definition_index > usage_index
            } else {
                definition_index > usage_index || definition_index + 1 != usage_index
            };
            if should_move {
                to_move.push(ToMove {
                    definition: definition_index,
                    target: usage_index,
                    position: item.position,
                });
            }
        }
        to_move.sort_by(|a, b| {
            a.position
                .cmp(&b.position)
                .then(a.definition.cmp(&b.definition))
        });

        let Some(first) = to_move.first() else {
            return;
        };
        let Some(&report_node) = statements.get(first.definition) else {
            return;
        };
        let pos = estree_declaration_start(ctx, report_node);
        let (_, end) = ctx.trim(report_node);
        let fixes = build_fixes(ctx, &to_move, statements);
        ctx.report_diagnostic_with_fixes(
            RuleDiagnostic {
                pos,
                end,
                message: RuleMessage::new(
                    "moveTypeAboveUsage",
                    "Type definition should be placed directly above its first usage.",
                ),
                labeled_ranges: Vec::new(),
            },
            |_| fixes,
        );
    }
}

/// The fixes of the report, sorted, or none when they overlap (ESLint rejects such a report).
fn build_fixes(ctx: &Ctx, to_move: &[ToMove], statements: &[P<Node>]) -> Vec<RuleFix> {
    let mut fixes = Vec::new();
    let mut targets: Vec<(usize, Vec<&ToMove>)> = Vec::new();
    for item in to_move {
        match targets
            .iter_mut()
            .find(|(target, _)| *target == item.target)
        {
            Some((_, group)) => group.push(item),
            None => targets.push((item.target, vec![item])),
        }
    }
    for item in to_move {
        let Some(&definition) = statements.get(item.definition) else {
            continue;
        };
        let (_, end) = ctx.trim(definition);
        fixes.push(ctx.fix_remove_range(start_with_comments_before(ctx, definition), end));
    }
    for (target, group) in &targets {
        let Some(&target) = statements.get(*target) else {
            continue;
        };
        let position = start_with_comments_before(ctx, target);
        let texts: Vec<String> = group
            .iter()
            .filter_map(|item| statements.get(item.definition))
            .map(|&definition| text_with_comments_before(ctx, definition))
            .collect();
        fixes.push(ctx.fix_replace_range(
            position,
            position,
            format!("{}\n\n", texts.join("\n\n")),
        ));
    }
    fixes.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.end.cmp(&b.end)));
    let mut last_end = i32::MIN;
    for fix in &fixes {
        if fix.pos < last_end {
            return Vec::new();
        }
        last_end = fix.end;
    }
    fixes
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        if listener == Listener::Enter(Kind::EndOfFile) {
            self.check_file(ctx);
            return;
        }
        let type_name = node.as_type_reference_node().type_name();
        if !ast::is_identifier(type_name) || is_part_of_indexed_access(node) {
            return;
        }
        self.references.push(TypeReference {
            name: type_name.as_identifier().text(),
            statement: top_level_statement(node),
            position: ctx.trim(node).0,
            in_function_args: is_in_function_argument(node),
            in_fc_props: is_in_fc_props(node),
            in_generic_arg_at_call: is_in_generic_arg_at_function_call(node),
            in_type_declaration: is_in_type_declaration(node),
        });
    }
}
