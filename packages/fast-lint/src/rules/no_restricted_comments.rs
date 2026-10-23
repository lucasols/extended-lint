use regex::Regex;
use tsrs_ast::Node;
use tsrs_core::P;
use tsrslint::rule::{
    Ctx, Listener, Rule, RuleDiagnostic, RuleMessage, RuleSuggestion, RuleVisitor,
};

use crate::utils::comments;

pub const NAME: &str = "no-restricted-comments";

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PatternOptions {
    regex: Option<String>,
    includes: Option<String>,
    message: Option<String>,
    #[serde(default)]
    auto_fix: bool,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Options {
    patterns: Vec<PatternOptions>,
}

enum Matcher {
    Regex(Regex),
    Includes(String),
    Never,
}

struct Pattern {
    matcher: Matcher,
    /// The pattern as the default message shows it: the regex source, else the `includes` text.
    display: String,
    message: Option<String>,
    auto_fix: bool,
}

pub struct NoRestrictedComments {
    patterns: Vec<Pattern>,
}

fn non_empty(value: Option<&String>) -> Option<&str> {
    value.map(String::as_str).filter(|value| !value.is_empty())
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let mut patterns = Vec::with_capacity(options.patterns.len());
    for pattern in options.patterns {
        let matcher = if let Some(regex) = non_empty(pattern.regex.as_ref()) {
            Matcher::Regex(
                Regex::new(regex).map_err(|e| format!("invalid regex \"{regex}\": {e}"))?,
            )
        } else if let Some(includes) = non_empty(pattern.includes.as_ref()) {
            Matcher::Includes(includes.to_string())
        } else {
            Matcher::Never
        };
        patterns.push(Pattern {
            matcher,
            display: pattern
                .regex
                .clone()
                .or_else(|| pattern.includes.clone())
                .unwrap_or_default(),
            message: pattern.message.filter(|message| !message.is_empty()),
            auto_fix: pattern.auto_fix,
        });
    }
    Ok(Box::new(NoRestrictedComments { patterns }))
}

impl Rule for NoRestrictedComments {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        if !self.patterns.is_empty() {
            self.check_comments(ctx);
        }
        Box::new(Visitor)
    }
}

impl Pattern {
    fn matches(&self, value: &str) -> bool {
        match &self.matcher {
            Matcher::Regex(regex) => regex.is_match(value),
            Matcher::Includes(includes) => value.contains(includes.as_str()),
            Matcher::Never => false,
        }
    }

    fn message(&self) -> RuleMessage {
        match &self.message {
            Some(message) => RuleMessage::new("restrictedCommentWithMessage", message.clone()),
            None => RuleMessage::new(
                "restrictedComment",
                format!("Comment matches restricted pattern '{}'", self.display),
            ),
        }
    }
}

/// The ESTree comment `value`: the text without the `//`, or without the `/*` and `*/`.
fn comment_value(text: &str, is_block: bool) -> &str {
    if is_block {
        let body = text.get(2..).unwrap_or_default();
        body.strip_suffix("*/").unwrap_or(body)
    } else {
        text.get(2..).unwrap_or_default()
    }
}

impl NoRestrictedComments {
    fn check_comments(&self, ctx: &mut Ctx) {
        for (start, end, is_block) in comments(ctx.file) {
            let (Ok(pos), Ok(end)) = (i32::try_from(start), i32::try_from(end)) else {
                continue;
            };
            let Some(text) = usize::try_from(start)
                .ok()
                .zip(usize::try_from(end).ok())
                .and_then(|(start, end)| ctx.text().get(start..end))
            else {
                continue;
            };
            let value = comment_value(text, is_block);
            let Some(pattern) = self.patterns.iter().find(|pattern| pattern.matches(value)) else {
                continue;
            };
            let diagnostic = RuleDiagnostic {
                pos,
                end,
                message: pattern.message(),
                labeled_ranges: Vec::new(),
            };
            if pattern.auto_fix {
                ctx.report_diagnostic_with_fixes(diagnostic, |ctx| {
                    vec![ctx.fix_remove_range(pos, end)]
                });
            } else {
                ctx.report_diagnostic_with_suggestions(diagnostic, |ctx| {
                    vec![RuleSuggestion {
                        message: RuleMessage::new("removeComment", "Remove this comment"),
                        fixes: vec![ctx.fix_remove_range(pos, end)],
                    }]
                });
            }
        }
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &[]
    }
    fn visit(&mut self, _ctx: &mut Ctx, _listener: Listener, _node: P<Node>) {}
}
