use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

pub const NAME: &str = "no-leaked-text-in-jsx";

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Options {}

pub struct NoLeakedTextInJsx;

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    super::parse_options::<Options>(options)?;
    Ok(Box::new(NoLeakedTextInJsx))
}

const LISTENERS: &[Listener] = &[Listener::Enter(Kind::JsxText)];

const DISALLOWED_TEXTS: &[&str] = &[",", ";", "[", "]", "(", ")"];

impl Rule for NoLeakedTextInJsx {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor)
    }
}

struct Visitor;

fn leaked_text(text: &str) -> Option<&str> {
    if let Some(&disallowed) = DISALLOWED_TEXTS.iter().find(|&&d| d == text) {
        return Some(disallowed);
    }
    if text.contains("&&") {
        Some("&&")
    } else if text.contains("||") {
        Some("||")
    } else if text.ends_with("? (") {
        Some("? (")
    } else {
        None
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let Some(invalid) = leaked_text(node.as_jsx_text().text().trim()) else {
            return;
        };
        let message = RuleMessage::new(
            "leakedTextInJSX",
            format!("Text \"{invalid}\" should be wrapped in a JSX expression container."),
        );
        let (Some(pos), Some(end)) = (node.pos().source_offset(), node.end().source_offset())
        else {
            return;
        };
        let (Ok(pos), Ok(end)) = (i32::try_from(pos), i32::try_from(end)) else {
            return;
        };
        ctx.report_range(pos, end, message);
    }
}
