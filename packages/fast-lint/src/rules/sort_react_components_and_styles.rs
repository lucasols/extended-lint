use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, ModifierFlags, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleFix, RuleMessage, RuleVisitor};

use crate::utils::{
    comments_before, for_each_descendant, is_estree_chain_expression, is_jsx_name, skip_parens,
    start_with_comments_before, text_with_comments_before,
};

pub const NAME: &str = "sort-react-components-and-styles";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    main_component_regex: Option<String>,
    main_component_selector: Option<String>,
}

/// The supported subset of ESLint selectors for `mainComponentSelector`: a node type, optionally with an
/// `[id.name='Name']` attribute.
struct Selector {
    kind: SelectorKind,
    id_name: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectorKind {
    VariableDeclarator,
    FunctionDeclaration,
    ClassDeclaration,
}

pub struct SortReactComponentsAndStyles {
    main_component_regex: Option<Regex>,
    main_component_selector: Option<Selector>,
}

fn parse_selector(selector: &str) -> Result<Selector, String> {
    let unsupported = || {
        format!(
            "unsupported mainComponentSelector `{selector}`: only `VariableDeclarator`, `FunctionDeclaration` and \
             `ClassDeclaration`, optionally followed by `[id.name='Name']`, are supported"
        )
    };
    let selector = selector.trim();
    let (kind, attribute) = match selector.find('[') {
        Some(index) => (
            selector.get(..index).unwrap_or_default(),
            selector.get(index..),
        ),
        None => (selector, None),
    };
    let kind = match kind {
        "VariableDeclarator" => SelectorKind::VariableDeclarator,
        "FunctionDeclaration" => SelectorKind::FunctionDeclaration,
        "ClassDeclaration" => SelectorKind::ClassDeclaration,
        _ => return Err(unsupported()),
    };
    let id_name = match attribute {
        None => None,
        Some(attribute) => {
            let value = attribute
                .strip_prefix('[')
                .and_then(|a| a.strip_suffix(']'))
                .and_then(|a| a.trim().strip_prefix("id.name"))
                .and_then(|a| a.trim_start().strip_prefix('='))
                .map(str::trim)
                .ok_or_else(unsupported)?;
            let unquoted = ['\'', '"']
                .iter()
                .find_map(|&quote| value.strip_prefix(quote)?.strip_suffix(quote))
                .unwrap_or(value);
            Some(unquoted.to_string())
        }
    };
    Ok(Selector { kind, id_name })
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let main_component_regex = match options.main_component_regex.filter(|r| !r.is_empty()) {
        Some(pattern) => Some(Regex::new(&pattern).map_err(|e| e.to_string())?),
        None => None,
    };
    let main_component_selector = match options.main_component_selector.filter(|s| !s.is_empty()) {
        Some(selector) => Some(parse_selector(&selector)?),
        None => None,
    };
    Ok(Box::new(SortReactComponentsAndStyles {
        main_component_regex,
        main_component_selector,
    }))
}

impl Rule for SortReactComponentsAndStyles {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        if ctx.file.file_name().ends_with(".tsx") {
            self.check_file(ctx);
        }
        Box::new(Visitor)
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &[]
    }
    fn visit(&mut self, _ctx: &mut Ctx, _l: Listener, _node: P<Node>) {}
}

struct Component {
    statement: P<Node>,
    index: usize,
    is_main: bool,
    is_exported: bool,
    used_styles: FxHashSet<&'static str>,
}

struct Style {
    statement: P<Node>,
    name: &'static str,
    initializer: P<Node>,
    used: bool,
}

struct StyleMove {
    style: usize,
    target: usize,
}

#[derive(Default)]
struct Collected {
    component_names: FxHashSet<&'static str>,
    alias_of: FxHashMap<&'static str, &'static str>,
    styles: Vec<Style>,
}

fn identifier_name(node: P<Node>) -> Option<&'static str> {
    ast::is_identifier(node).then(|| node.as_identifier().text())
}

fn is_pascal_case(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase()) && chars.all(|c| c.is_ascii_alphanumeric())
}

fn is_jsx(node: P<Node>) -> bool {
    matches!(
        node.kind(),
        Kind::JsxElement | Kind::JsxSelfClosingElement | Kind::JsxFragment
    )
}

fn returns_jsx(node: P<Node>) -> bool {
    let node = skip_parens(node);
    match node.kind() {
        Kind::JsxElement | Kind::JsxSelfClosingElement | Kind::JsxFragment => true,
        Kind::Block => node
            .statements()
            .iter()
            .find_map(|&statement| {
                (statement.kind() == Kind::ReturnStatement)
                    .then(|| statement.expression())
                    .flatten()
            })
            .is_some_and(returns_jsx),
        Kind::ArrayLiteralExpression => node.elements().iter().any(|&e| is_jsx(skip_parens(e))),
        _ => false,
    }
}

fn is_function_expression(node: P<Node>) -> bool {
    matches!(node.kind(), Kind::ArrowFunction | Kind::FunctionExpression)
}

fn is_wrapper_call(node: P<Node>) -> bool {
    if node.kind() != Kind::CallExpression || is_estree_chain_expression(node) {
        return false;
    }
    let callee = skip_parens(node.as_call_expression().expression());
    if is_estree_chain_expression(callee) {
        return false;
    }
    let name = match callee.kind() {
        Kind::Identifier => Some(callee.as_identifier().text()),
        Kind::PropertyAccessExpression => {
            identifier_name(callee.as_property_access_expression().name())
        }
        Kind::ElementAccessExpression => identifier_name(skip_parens(
            callee.as_element_access_expression().argument_expression(),
        )),
        _ => None,
    };
    name.is_some_and(|name| name == "memo" || name == "forwardRef")
}

fn is_hoc_initializer(node: P<Node>) -> bool {
    if node.kind() != Kind::ArrowFunction {
        return false;
    }
    node.body()
        .map(skip_parens)
        .filter(|body| body.kind() == Kind::ArrowFunction)
        .and_then(|body| body.body())
        .is_some_and(returns_jsx)
}

fn is_styled_component(node: P<Node>) -> bool {
    if node.kind() != Kind::TaggedTemplateExpression {
        return false;
    }
    let tag = skip_parens(node.as_tagged_template_expression().tag());
    match tag.kind() {
        Kind::PropertyAccessExpression | Kind::ElementAccessExpression => {
            !is_estree_chain_expression(tag)
                && tag.expression().map(skip_parens).and_then(identifier_name) == Some("styled")
        }
        Kind::CallExpression => {
            !is_estree_chain_expression(tag)
                && identifier_name(skip_parens(tag.as_call_expression().expression()))
                    == Some("styled")
        }
        _ => false,
    }
}

fn is_css_template(node: P<Node>) -> bool {
    node.kind() == Kind::TaggedTemplateExpression
        && identifier_name(skip_parens(node.as_tagged_template_expression().tag())) == Some("css")
}

/// The ESTree `FunctionDeclaration`s: TypeScript's body-less overloads and `declare function`s are
/// `TSDeclareFunction`s there.
fn is_function_declaration(node: P<Node>) -> bool {
    node.kind() == Kind::FunctionDeclaration && node.body().is_some()
}

fn is_export_default_assignment(node: P<Node>) -> bool {
    node.kind() == Kind::ExportAssignment && !node.as_export_assignment().is_export_equals()
}

fn is_default_export(statement: P<Node>) -> bool {
    is_export_default_assignment(statement)
        || ast::has_syntactic_modifier(statement, ModifierFlags::Default)
}

/// ESTree `ExportNamedDeclaration`s and `ExportDefaultDeclaration`s.
fn is_exported(statement: P<Node>) -> bool {
    is_export_default_assignment(statement)
        || statement.kind() == Kind::ExportDeclaration
        || ast::has_syntactic_modifier(statement, ModifierFlags::Export)
}

fn variable_declarations(statement: P<Node>) -> &'static [P<Node>] {
    if statement.kind() != Kind::VariableStatement {
        return &[];
    }
    statement
        .as_variable_statement()
        .declaration_list()
        .as_variable_declaration_list()
        .declarations()
        .nodes()
}

fn collect_from_declaration(collected: &mut Collected, declaration: P<Node>, statement: P<Node>) {
    let Some(name) = declaration.name().and_then(identifier_name) else {
        return;
    };
    let Some(init) = declaration.initializer().map(skip_parens) else {
        return;
    };
    if is_styled_component(init) || is_css_template(init) {
        collected.styles.push(Style {
            statement,
            name,
            initializer: init,
            used: false,
        });
        return;
    }
    if (is_pascal_case(name) || init.kind() == Kind::ArrowFunction)
        && (is_function_expression(init) || is_wrapper_call(init))
    {
        if is_function_expression(init) {
            if init.body().is_some_and(returns_jsx) {
                collected.component_names.insert(name);
                return;
            }
        } else {
            collected.component_names.insert(name);
            return;
        }
    }
    if let Some(aliased) = identifier_name(init) {
        collected.alias_of.insert(name, aliased);
    }
    if !is_pascal_case(name) && is_hoc_initializer(init) {
        collected.component_names.insert(name);
    }
}

fn collect_top_level(collected: &mut Collected, statement: P<Node>) {
    if statement.kind() == Kind::VariableStatement {
        for &declaration in variable_declarations(statement) {
            collect_from_declaration(collected, declaration, statement);
        }
        return;
    }
    if is_function_declaration(statement) {
        let name = statement.name().and_then(identifier_name);
        if is_default_export(statement) {
            collected.component_names.insert(name.unwrap_or("default"));
        } else if let Some(name) = name {
            if is_pascal_case(name) && statement.body().is_some_and(returns_jsx) {
                collected.component_names.insert(name);
            }
        }
        return;
    }
    if is_export_default_assignment(statement) {
        let Some(expression) = statement.expression().map(skip_parens) else {
            return;
        };
        if let Some(name) = identifier_name(expression) {
            collected.component_names.insert(name);
        }
        if is_function_expression(expression) && expression.body().is_some_and(returns_jsx) {
            collected.component_names.insert("default");
        }
    }
}

fn component_name(statement: P<Node>) -> Option<&'static str> {
    if statement.kind() == Kind::VariableStatement {
        return variable_declarations(statement)
            .first()
            .and_then(|declaration| declaration.name())
            .and_then(identifier_name);
    }
    if is_function_declaration(statement) {
        let name = statement.name().and_then(identifier_name);
        if is_default_export(statement) {
            return Some(name.unwrap_or("default"));
        }
        return name;
    }
    if is_export_default_assignment(statement) {
        return statement
            .expression()
            .map(skip_parens)
            .and_then(identifier_name);
    }
    None
}

fn resolves_to_component(collected: &Collected, name: &'static str) -> bool {
    let mut seen = FxHashSet::default();
    let mut current = name;
    loop {
        if collected.component_names.contains(current) {
            return true;
        }
        if !seen.insert(current) {
            return false;
        }
        match collected.alias_of.get(current) {
            Some(&aliased) => current = aliased,
            None => return false,
        }
    }
}

/// The ESTree `Identifier` names in `node`: TypeScript identifiers, without JSX tag and attribute names.
fn estree_identifier_name(node: P<Node>) -> Option<&'static str> {
    (ast::is_identifier(node) && !is_jsx_name(node)).then(|| node.as_identifier().text())
}

fn jsx_opening_identifier_name(node: P<Node>) -> Option<&'static str> {
    matches!(
        node.kind(),
        Kind::JsxOpeningElement | Kind::JsxSelfClosingElement
    )
    .then(|| identifier_name(node.tag_name()))
    .flatten()
}

impl SortReactComponentsAndStyles {
    fn selected_statements(&self, file_node: P<Node>) -> FxHashSet<P<Node>> {
        let mut selected = FxHashSet::default();
        let Some(selector) = &self.main_component_selector else {
            return selected;
        };
        for_each_descendant(file_node, &mut |node| {
            let matches_kind = match selector.kind {
                SelectorKind::VariableDeclarator => node.kind() == Kind::VariableDeclaration,
                SelectorKind::FunctionDeclaration => is_function_declaration(node),
                SelectorKind::ClassDeclaration => node.kind() == Kind::ClassDeclaration,
            };
            if !matches_kind {
                return;
            }
            if let Some(expected) = &selector.id_name {
                if node.name().and_then(identifier_name) != Some(expected.as_str()) {
                    return;
                }
            }
            let mut current = node;
            while let Some(parent) = current.parent() {
                if parent.kind() == Kind::SourceFile {
                    break;
                }
                current = parent;
            }
            let is_statement_kind = matches!(
                current.kind(),
                Kind::VariableStatement | Kind::FunctionDeclaration
            ) || is_exported(current);
            if current
                .parent()
                .is_some_and(|p| p.kind() == Kind::SourceFile)
                && is_statement_kind
            {
                selected.insert(current);
            }
        });
        selected
    }

    fn check_file(&self, ctx: &mut Ctx) {
        let file_node = ctx.file.as_node();
        let statements = file_node.statements();
        let first_after_imports = statements
            .iter()
            .take_while(|statement| statement.kind() == Kind::ImportDeclaration)
            .count();

        let mut collected = Collected::default();
        for &statement in statements {
            collect_top_level(&mut collected, statement);
        }

        let selected = self.selected_statements(file_node);
        let mut components = Vec::new();
        for (index, &statement) in statements.iter().enumerate() {
            let Some(name) = component_name(statement) else {
                continue;
            };
            if !resolves_to_component(&collected, name) {
                continue;
            }
            let is_main = if self.main_component_selector.is_some() {
                selected.contains(&statement)
            } else if let Some(regex) = &self.main_component_regex {
                regex.is_match(name)
            } else {
                is_exported(statement)
            };
            components.push(Component {
                statement,
                index,
                is_main,
                is_exported: is_exported(statement),
                used_styles: FxHashSet::default(),
            });
        }

        if self.main_component_selector.is_none()
            && self.main_component_regex.is_some()
            && !components.iter().any(|c| c.is_main)
        {
            for component in &mut components {
                if component.is_exported {
                    component.is_main = true;
                }
            }
        }

        let mut styles = collected.styles;
        let mut styles_by_name: FxHashMap<&'static str, Vec<usize>> = FxHashMap::default();
        for (index, style) in styles.iter().enumerate() {
            styles_by_name.entry(style.name).or_default().push(index);
        }

        for component in &mut components {
            for_each_descendant(component.statement, &mut |node| {
                for name in [
                    estree_identifier_name(node),
                    jsx_opening_identifier_name(node),
                ]
                .into_iter()
                .flatten()
                {
                    if let Some(indices) = styles_by_name.get(name) {
                        component.used_styles.insert(name);
                        for &index in indices {
                            if let Some(style) = styles.get_mut(index) {
                                style.used = true;
                            }
                        }
                    }
                }
            });
        }

        let statement_index: FxHashMap<P<Node>, usize> = statements
            .iter()
            .enumerate()
            .map(|(index, &statement)| (statement, index))
            .collect();
        let mut style_index_by_name: FxHashMap<&'static str, usize> = FxHashMap::default();
        for (index, &statement) in statements.iter().enumerate() {
            for style in &styles {
                if style.statement == statement {
                    style_index_by_name.insert(style.name, index);
                }
            }
        }

        let mut depends_on: FxHashMap<&'static str, Vec<&'static str>> = FxHashMap::default();
        for style in &styles {
            let mut deps: Vec<&'static str> = Vec::new();
            for_each_descendant(style.initializer, &mut |node| {
                if let Some(name) = estree_identifier_name(node) {
                    if name != "styled"
                        && name != "css"
                        && name != style.name
                        && styles_by_name.contains_key(name)
                        && !deps.contains(&name)
                    {
                        deps.push(name);
                    }
                }
            });
            depends_on.insert(style.name, deps);
        }

        let mut referenced_by_earliest: FxHashMap<&'static str, usize> = FxHashMap::default();
        for style in &styles {
            let (Some(deps), Some(&source_index)) = (
                depends_on.get(style.name),
                style_index_by_name.get(style.name),
            ) else {
                continue;
            };
            for &dep in deps {
                let entry = referenced_by_earliest.entry(dep).or_insert(source_index);
                if source_index < *entry {
                    *entry = source_index;
                }
            }
        }

        let mut styles_to_move: Vec<StyleMove> = Vec::new();
        for (style_index, style) in styles.iter().enumerate() {
            if !style.used {
                continue;
            }
            let styled_statement_index = statement_index
                .get(&style.statement)
                .map_or(-1, |&i| to_isize(i));
            let mut first_using: isize = -1;
            for component in &components {
                if !component.used_styles.contains(style.name) {
                    continue;
                }
                let index = to_isize(component.index);
                if first_using == -1 || index < first_using {
                    first_using = index;
                }
            }
            if let Some(&reference) = referenced_by_earliest.get(style.name) {
                let reference = to_isize(reference);
                if first_using == -1 || reference < first_using {
                    first_using = reference;
                }
            }
            if first_using == -1 {
                continue;
            }
            let mut previous_component: isize = -1;
            for component in &components {
                let index = to_isize(component.index);
                if index < first_using && index > previous_component {
                    previous_component = index;
                }
            }
            let mut max_dep_index: isize = -1;
            if let Some(deps) = depends_on.get(style.name) {
                for dep in deps {
                    if let Some(&dep_index) = style_index_by_name.get(dep) {
                        max_dep_index = max_dep_index.max(to_isize(dep_index));
                    }
                }
            }
            let correctly_placed = styled_statement_index < first_using
                && styled_statement_index > previous_component
                && styled_statement_index > max_dep_index;
            if correctly_placed {
                continue;
            }
            let target = (previous_component + 1)
                .max(max_dep_index + 1)
                .max(to_isize(first_after_imports));
            if target > first_using {
                continue;
            }
            styles_to_move.push(StyleMove {
                style: style_index,
                target: usize::try_from(target).unwrap_or(0),
            });
        }

        let mut component_move: Option<(P<Node>, P<Node>)> = None;
        if components.len() > 1 {
            let first = components.iter().min_by_key(|c| c.index);
            let main = components.iter().find(|c| c.is_main);
            if let (Some(first), Some(main)) = (first, main) {
                if first.statement != main.statement {
                    component_move = Some((main.statement, first.statement));
                }
            }
        }

        if styles_to_move.is_empty() && component_move.is_none() {
            return;
        }

        let report_node = styles_to_move
            .first()
            .and_then(|m| styles.get(m.style))
            .map(|s| s.statement)
            .or(component_move.map(|(component, _)| component))
            .unwrap_or(file_node);
        let message = if styles_to_move.is_empty() {
            RuleMessage::new(
                "mainComponentShouldBeFirst",
                "Main component should be placed before other components",
            )
        } else {
            RuleMessage::new(
                "stylesShouldBeAboveUsage",
                "Style definitions should be placed above the first component that uses them",
            )
        };

        let fixes = build_fixes(ctx, &styles, styles_to_move, component_move, statements);
        match fixes {
            Some(fixes) => ctx.report_node_with_fixes(report_node, message, |_| fixes),
            None => ctx.report_node(report_node, message),
        }
    }
}

fn to_isize(value: usize) -> isize {
    isize::try_from(value).unwrap_or(isize::MAX)
}

fn removal_start(ctx: &Ctx, statement: P<Node>) -> i32 {
    if let Some(&(start, _)) = comments_before(ctx, statement).first() {
        return start;
    }
    let (start, _) = ctx.trim(statement);
    let leading = usize::try_from(start)
        .ok()
        .and_then(|start| ctx.text().get(..start))
        .unwrap_or_default();
    if leading.trim().is_empty() {
        0
    } else {
        start
    }
}

/// The fixes of the report, sorted, or `None` when they overlap (ESLint rejects such a report).
fn build_fixes(
    ctx: &Ctx,
    styles: &[Style],
    mut styles_to_move: Vec<StyleMove>,
    component_move: Option<(P<Node>, P<Node>)>,
    statements: &[P<Node>],
) -> Option<Vec<RuleFix>> {
    let mut fixes = Vec::new();
    styles_to_move.sort_by_key(|m| m.target);
    for item in &styles_to_move {
        let statement = styles.get(item.style)?.statement;
        let (_, end) = ctx.trim(statement);
        fixes.push(ctx.fix_remove_range(removal_start(ctx, statement), end));
    }
    let mut index = 0;
    while let Some(first) = styles_to_move.get(index) {
        let target = first.target;
        let group: Vec<&StyleMove> = styles_to_move
            .iter()
            .skip(index)
            .take_while(|m| m.target == target)
            .collect();
        index += group.len();
        let Some(&target_statement) = statements.get(target) else {
            continue;
        };
        let position = start_with_comments_before(ctx, target_statement);
        let texts: Vec<String> = group
            .iter()
            .filter_map(|m| styles.get(m.style))
            .map(|style| text_with_comments_before(ctx, style.statement))
            .collect();
        fixes.push(ctx.fix_replace_range(
            position,
            position,
            format!("{}\n\n", texts.join("\n\n")),
        ));
    }
    if let Some((component, target)) = component_move {
        let (_, end) = ctx.trim(component);
        let text = text_with_comments_before(ctx, component);
        let position = start_with_comments_before(ctx, target);
        fixes.push(ctx.fix_remove_range(removal_start(ctx, component), end));
        fixes.push(ctx.fix_replace_range(position, position, format!("{text}\n")));
    }
    fixes.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.end.cmp(&b.end)));
    let mut last_end = i32::MIN;
    for fix in &fixes {
        if fix.pos < last_end {
            return None;
        }
        last_end = fix.end;
    }
    Some(fixes)
}
