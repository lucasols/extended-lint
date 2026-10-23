use tsrs_ast::{Kind, Node, SourceFile};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::esquery::estree::{self, EsNode, EsType, EsValue};
use crate::esquery::{CompiledSelector, SelectorIndex};
use crate::utils::comments;

pub const NAME: &str = "require-usage-explanation";

const MESSAGE_ID: &str = "default";

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Options {
    matches: Vec<MatchOptions>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MatchOptions {
    #[serde(rename = "fn")]
    function: Option<String>,
    selector: Option<String>,
    comment_prefix: String,
    message: Option<String>,
}

struct Check {
    prefix: String,
    message: String,
}

impl Check {
    fn new(prefix: &str, label: &str, custom_message: Option<&str>) -> Check {
        let base = format!(
            "Add a comment with prefix \"{prefix}\" to explain why \"{label}\" is being used."
        );
        let message = match custom_message.filter(|message| !message.is_empty()) {
            Some(custom) => format!("{base} {custom}"),
            None => base,
        };
        Check {
            prefix: prefix.to_string(),
            message,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Owner {
    Calls,
    Selector(usize),
}

pub struct RequireUsageExplanation {
    fn_checks: Vec<(String, Check)>,
    selector_checks: Vec<Check>,
    index: SelectorIndex<Owner>,
    listeners: Vec<Listener>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut index = SelectorIndex::new();
    let mut fn_checks = Vec::new();
    let mut selector_checks = Vec::new();
    for entry in &options.matches {
        let message = entry.message.as_deref();
        if let Some(function) = &entry.function {
            fn_checks.push((
                function.clone(),
                Check::new(&entry.comment_prefix, function, message),
            ));
        }
        if let Some(selector) = &entry.selector {
            index.add(
                CompiledSelector::new(selector)?,
                Owner::Selector(selector_checks.len()),
            );
            selector_checks.push(Check::new(&entry.comment_prefix, selector, message));
        }
    }
    if !fn_checks.is_empty() {
        index.add(CompiledSelector::new("CallExpression")?, Owner::Calls);
    }
    let mut listeners: Vec<Listener> = index
        .kinds()
        .kinds()
        .into_iter()
        .map(Listener::Enter)
        .collect();
    if index.has_program_selectors() {
        listeners.push(Listener::Enter(Kind::EndOfFile));
    }
    if !index.is_empty() {
        listeners.push(Listener::AllowPattern(Kind::PropertyAssignment));
    }
    Ok(Box::new(RequireUsageExplanation {
        fn_checks,
        selector_checks,
        index,
        listeners,
    }))
}

impl Rule for RequireUsageExplanation {
    fn name(&self) -> &'static str {
        NAME
    }

    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor {
            rule: self,
            comments: None,
        })
    }
}

struct Visitor {
    rule: &'static RequireUsageExplanation,
    comments: Option<Vec<(u32, u32, bool)>>,
}

/// The name a call's callee has for ESLint's `callee.name` / `callee.property.name`: an `Identifier` callee or the
/// `Identifier` property of a `MemberExpression` callee.
fn callee_name(es: EsNode, file: P<SourceFile>) -> Option<String> {
    let EsValue::Node(callee) = estree::field(es, "callee", file) else {
        return None;
    };
    let identifier = match callee.es_type() {
        EsType::Identifier => callee,
        EsType::MemberExpression => match estree::field(callee, "property", file) {
            EsValue::Node(property) if property.es_type() == EsType::Identifier => property,
            _ => return None,
        },
        _ => return None,
    };
    match estree::field(identifier, "name", file) {
        EsValue::Str(name) => Some(name.into_owned()),
        _ => None,
    }
}

/// The text of a comment without its `//` or `/* */` delimiters, like ESLint's `comment.value`.
fn comment_value(text: &str, (start, end, block): (u32, u32, bool)) -> &str {
    let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
        return "";
    };
    let value_end = if block { end.saturating_sub(2) } else { end };
    text.get(start + 2..value_end).unwrap_or("")
}

fn is_explanation(value: &str, prefix: &str) -> bool {
    value
        .trim()
        .strip_prefix(prefix)
        .is_some_and(|rest| !rest.trim().is_empty())
}

impl Visitor {
    /// Whether a comment between the previous token and `pos` explains the usage, like checking
    /// `sourceCode.getCommentsBefore(node)` for a node starting at `pos`.
    fn has_explanation_before(&mut self, ctx: &Ctx, pos: i32, prefix: &str) -> bool {
        let text = ctx.text();
        let comments = self.comments.get_or_insert_with(|| comments(ctx.file));
        let Ok(mut cursor) = usize::try_from(pos) else {
            return false;
        };
        loop {
            let Some(before) = text.get(..cursor) else {
                return false;
            };
            let Ok(end) = u32::try_from(before.trim_end().len()) else {
                return false;
            };
            let Ok(found) = comments.binary_search_by_key(&end, |&(_, end, _)| end) else {
                return false;
            };
            let Some(&comment) = comments.get(found) else {
                return false;
            };
            if is_explanation(comment_value(text, comment), prefix) {
                return true;
            }
            let Ok(start) = usize::try_from(comment.0) else {
                return false;
            };
            cursor = start;
        }
    }

    /// Whether a comment before `es` or before one of its ancestors up to the statement holding it explains the
    /// usage.
    fn has_explanation(&mut self, ctx: &Ctx, es: EsNode, prefix: &str) -> bool {
        let mut chain = vec![es];
        let mut current = es;
        loop {
            let Some(parent) = estree::parent(current) else {
                return false;
            };
            if matches!(
                parent.es_type(),
                EsType::Program | EsType::BlockStatement | EsType::SwitchCase
            ) {
                break;
            }
            chain.push(parent);
            current = parent;
        }
        let mut last_start = None;
        for node in chain {
            let start = estree::range(node, ctx.file).0;
            if last_start == Some(start) {
                continue;
            }
            last_start = Some(start);
            if self.has_explanation_before(ctx, start, prefix) {
                return true;
            }
        }
        false
    }

    fn check(&mut self, ctx: &mut Ctx, es: EsNode, check: &Check) {
        if self.has_explanation(ctx, es, &check.prefix) {
            return;
        }
        let (pos, end) = estree::range(es, ctx.file);
        ctx.report_range(
            pos,
            end,
            RuleMessage::new(MESSAGE_ID, check.message.clone()),
        );
    }

    fn run_selectors(&mut self, ctx: &mut Ctx, node: P<Node>) {
        let rule = self.rule;
        if !rule.index.kinds().contains(node.kind()) {
            return;
        }
        let mut matched = Vec::new();
        rule.index
            .visit(node, ctx.file, |owner, es| matched.push((owner, es)));
        for (owner, es) in matched {
            match owner {
                Owner::Selector(index) => {
                    if let Some(check) = rule.selector_checks.get(index) {
                        self.check(ctx, es, check);
                    }
                }
                Owner::Calls => {
                    let Some(name) = callee_name(es, ctx.file) else {
                        continue;
                    };
                    for (function, check) in &rule.fn_checks {
                        if *function == name {
                            self.check(ctx, es, check);
                        }
                    }
                }
            }
        }
    }

    /// Runs the selectors on `node` and its descendants: the keys of a destructuring assignment's properties, which
    /// the walker doesn't visit.
    fn enter_subtree(&mut self, ctx: &mut Ctx, node: P<Node>) {
        self.run_selectors(ctx, node);
        let mut children = Vec::new();
        node.for_each_child(&mut |child| {
            children.push(child);
            false
        });
        for child in children {
            self.enter_subtree(ctx, child);
        }
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &self.rule.listeners
    }

    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>) {
        match listener {
            Listener::AllowPattern(_) => {
                if let Some(name) = node.name() {
                    self.enter_subtree(ctx, name);
                }
            }
            _ if node.kind() == Kind::EndOfFile => self.run_selectors(ctx, ctx.file.as_node()),
            _ => self.run_selectors(ctx, node),
        }
    }
}
