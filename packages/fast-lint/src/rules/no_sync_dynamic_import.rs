use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::{estree_parent, skip_parens};

pub const NAME: &str = "no-sync-dynamic-import";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoSyncDynamicImport;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoSyncDynamicImport))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::CallExpression),
    Listener::Enter(Kind::ImportType),
];

impl Rule for NoSyncDynamicImport {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn sync_dynamic_import() -> RuleMessage {
    RuleMessage::new(
        "syncDynamicImport",
        "Synchronous dynamic import() is not allowed. Use regular import statement or await import() for dynamic imports.",
    )
}

fn is_identifier_named(node: P<Node>, name: &str) -> bool {
    ast::is_identifier(node) && node.text() == name
}

/// `Promise.all([...])`, `Promise.allSettled([...])` or `Promise.race([...])`.
fn is_promise_combinator_call(call: P<Node>) -> bool {
    if !ast::is_call_expression(call) {
        return false;
    }
    let Some(callee) = call.expression().map(skip_parens) else {
        return false;
    };
    if !ast::is_property_access_expression(callee) {
        return false;
    }
    let access = callee.as_property_access_expression();
    is_identifier_named(skip_parens(access.expression()), "Promise")
        && matches!(access.name().text(), "all" | "allSettled" | "race")
}

/// Whether the dynamic `import()` is awaited, chained with `.then`, returned by an arrow function or passed in an
/// array to `Promise.all`/`allSettled`/`race`. Any ancestor `await` counts, like in the ESLint rule.
fn is_in_async_context(import: P<Node>) -> bool {
    let mut current = import;
    while let Some(parent) = current.parent() {
        if ast::is_await_expression(parent) {
            return true;
        }
        if ast::is_property_access_expression(parent) {
            let access = parent.as_property_access_expression();
            if skip_parens(access.expression()) == import && access.name().text() == "then" {
                return true;
            }
        }
        if ast::is_arrow_function(parent) && parent.body().map(skip_parens) == Some(import) {
            return true;
        }
        if ast::is_array_literal_expression(parent)
            && estree_parent(parent).is_some_and(is_promise_combinator_call)
        {
            return true;
        }
        current = parent;
    }
    false
}

/// `require(...)` or a call of a `require` member (`require.resolve(...)`).
fn is_require_call(call: P<Node>) -> bool {
    let Some(callee) = call.expression().map(skip_parens) else {
        return false;
    };
    if is_identifier_named(callee, "require") {
        return true;
    }
    (ast::is_property_access_expression(callee) || ast::is_element_access_expression(callee))
        && callee
            .expression()
            .is_some_and(|object| is_identifier_named(skip_parens(object), "require"))
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        if node.kind() == Kind::ImportType {
            ctx.report_node(node, sync_dynamic_import());
            return;
        }
        if ast::is_import_call(node) {
            if !is_in_async_context(node) {
                ctx.report_node(node, sync_dynamic_import());
            }
            return;
        }
        if is_require_call(node) {
            ctx.report_node(
                node,
                RuleMessage::new(
                    "requireNotAllowed",
                    "require() statements are not allowed. Use ES6 import statements instead.",
                ),
            );
        }
    }
}
