use std::sync::LazyLock;

use regex::Regex;
use tsrs_ast::Node;
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

use crate::utils::comments;

pub const NAME: &str = "no-commented-out-code";

#[derive(serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    explanation_prefixes: Vec<String>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            explanation_prefixes: vec!["type-guard-safety:".to_string()],
        }
    }
}

pub struct NoCommentedOutCode {
    explanation_prefixes: Vec<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    Ok(Box::new(NoCommentedOutCode {
        explanation_prefixes: options
            .explanation_prefixes
            .into_iter()
            .filter(|prefix| !prefix.is_empty())
            .collect(),
    }))
}

impl Rule for NoCommentedOutCode {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        check_comments(ctx, &self.explanation_prefixes);
        Box::new(Visitor)
    }
}

struct Visitor;

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        &[]
    }
    fn visit(&mut self, _ctx: &mut Ctx, _listener: Listener, _node: P<Node>) {}
}

const START_PATTERNS: &[&str] = &[
    "function ",
    "class ",
    "interface ",
    "enum ",
    "namespace ",
    "import ",
    "export ",
    "const ",
    "let ",
    "var ",
    "return ",
    "throw ",
    "try {",
    "catch (",
    "finally {",
    "async ",
    "await ",
    "case ",
    "switch (",
    "while (",
    "for (",
    "default:",
];

const ALLOWED_PREFIXES: &[&str] = &[
    "INFO:",
    "TODO:",
    "DOCS:",
    "FIX:",
    "FIXME:",
    "HACK:",
    "NOTE:",
    "WARNING:",
    "WARN:",
    "BUG:",
    "ISSUE:",
    "TEMP:",
    "TEMPORARY:",
    "XXX:",
    "REVIEW:",
    "eslint",
    "cspell:",
    "spell-checker:",
];

const IGNORED_SUBSTRINGS: &[&str] = &[
    "@deprecated",
    "@example",
    "@param",
    "@returns",
    "@throws",
    "typescript-eslint",
    "@ts-",
    "prettier-ignore",
];

/// JavaScript's `\s` (WhiteSpace and LineTerminator), inside a character class.
const JS_SPACE_CLASS_ITEMS: &str = r"\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}";

/// Translates the source of a JavaScript regex (without the `u`/`v` set syntax, backreferences or `\b`) to the
/// `regex` crate's syntax, keeping JavaScript's meaning of `\s`, `\w`, `\d` and `.`.
fn js_regex(source: &str) -> Regex {
    let mut out = String::with_capacity(source.len() * 2);
    let mut in_class = false;
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let Some(escaped) = chars.next() else {
                    out.push_str(r"\\");
                    break;
                };
                match (escaped, in_class) {
                    ('s', false) => {
                        out.push('[');
                        out.push_str(JS_SPACE_CLASS_ITEMS);
                        out.push(']');
                    }
                    ('s', true) => out.push_str(JS_SPACE_CLASS_ITEMS),
                    ('w', false) => out.push_str("[A-Za-z0-9_]"),
                    ('w', true) => out.push_str("A-Za-z0-9_"),
                    ('d', false) => out.push_str("[0-9]"),
                    ('d', true) => out.push_str("0-9"),
                    ('/', _) => out.push('/'),
                    ('u', _) => {
                        let hex: String = chars.by_ref().take(4).collect();
                        out.push_str(r"\x{");
                        out.push_str(&hex);
                        out.push('}');
                    }
                    (other, _) => {
                        out.push('\\');
                        out.push(other);
                    }
                }
            }
            '[' if in_class => out.push_str(r"\["),
            '[' => {
                in_class = true;
                out.push('[');
                if chars.peek() == Some(&'^') {
                    chars.next();
                    out.push('^');
                }
            }
            ']' if in_class => {
                in_class = false;
                out.push(']');
            }
            '.' if !in_class => out.push_str(r"[^\n\r\x{2028}\x{2029}]"),
            other => out.push(other),
        }
    }
    Regex::new(&out).unwrap_or_else(|e| panic!("invalid translated regex {out}: {e}"))
}

/// A pattern reported as `regex(<JavaScript regex literal>)`, like the ESLint rule's `pattern.toString()`.
struct JsPattern {
    display: &'static str,
    regex: Regex,
}

impl JsPattern {
    fn new(display: &'static str) -> JsPattern {
        let source = display
            .strip_prefix('/')
            .and_then(|rest| rest.rsplit_once('/'))
            .map_or(display, |(source, _flags)| source);
        JsPattern {
            display,
            regex: js_regex(source),
        }
    }

    /// For the patterns `js_regex` can't translate: the JavaScript literal shown in messages and an equivalent regex.
    fn with_regex(display: &'static str, regex: &str) -> JsPattern {
        JsPattern {
            display,
            regex: Regex::new(regex).unwrap_or_else(|e| panic!("invalid regex {regex}: {e}")),
        }
    }
}

enum Pattern {
    Text(&'static str),
    Regex(JsPattern),
}

impl Pattern {
    fn find(&self, text: &str) -> Option<String> {
        match self {
            Pattern::Text(pattern) => text.contains(pattern).then(|| (*pattern).to_string()),
            Pattern::Regex(pattern) => pattern
                .regex
                .is_match(text)
                .then(|| format!("regex({})", pattern.display)),
        }
    }
}

struct Patterns {
    jsdoc_comment: Regex,
    code: Vec<Pattern>,
    jsx: Vec<Pattern>,
    header: Regex,
    header_then_text: Regex,
    lowercase_header_then_sentence: Regex,
    parenthetical_prose: Regex,
    fenced_block: Regex,
    backtick_code: Regex,
    alphanumeric: Regex,
    method_call_pattern: Regex,
    special_chars: Regex,
    colon_pattern: Regex,
    control_flow: Regex,
}

static PATTERNS: LazyLock<Patterns> = LazyLock::new(|| {
    let regex = |display| Pattern::Regex(JsPattern::new(display));
    let quoted_property_key = Pattern::Regex(JsPattern::with_regex(
        r#"/^\s*(['"`]).+?\1\s*:/"#,
        &format!(
            r#"^[{s}]*(?:'{dot}+?'|"{dot}+?"|`{dot}+?`)[{s}]*:"#,
            s = JS_SPACE_CLASS_ITEMS,
            dot = r"[^\n\r\x{2028}\x{2029}]"
        ),
    ));
    Patterns {
        jsdoc_comment: js_regex(r"^\s*[*\s]*$"),
        code: vec![
            Pattern::Text(") {"),
            Pattern::Text("return;"),
            regex(r"/^\s*return\s+/"),
            regex(r"/^\s*type\s+[\p{ID_Start}$_][\p{ID_Continue}$\u200C\u200D]*\s*(?:=|<)/u"),
            regex(r"/^\s*(?:break|continue)(?:;|\s+[A-Za-z_$][\w$]*;?)?\s*$/"),
            Pattern::Text("if ("),
            Pattern::Text("else {"),
            Pattern::Text("else if ("),
            Pattern::Text("/>"),
            Pattern::Text("</"),
            Pattern::Text("< "),
            Pattern::Text("},"),
            Pattern::Text(": {"),
            Pattern::Text(" } = "),
            Pattern::Text("={"),
            regex(r#"/\w=("|'|`)/"#),
            Pattern::Text(");"),
            regex(r#"/\w+:\s*('|"|`)/"#),
            regex(r#"/\[['"][^'"]*['"]\]:\s*('|"|`|[^'"`\s])/"#),
            regex(r"/\w+-\w+:/"),
            regex(r"/\w+_\w+:/"),
            Pattern::Text("&&"),
            Pattern::Text("||"),
            Pattern::Text("()"),
            regex(r"/\?\s+\w/"),
            regex(r"/:\s+\w/"),
            regex(r#"/^\s*('|"|`)[^'"]*('|"|`),?\s*$/"#),
            regex(r"/^\s*\d+[,}]/"),
            regex(r"/^\s*\[[^\]]*\][,}]/"),
            regex(r"/^\s*\{[^}]*\}[,}]/"),
            regex(r"/\.\w+\(/"),
            regex(r"/\[\w+\]/"),
            Pattern::Text("?.("),
            Pattern::Text("??"),
            Pattern::Text("=>"),
            quoted_property_key,
        ],
        jsx: vec![
            Pattern::Text("/>"),
            Pattern::Text("</"),
            regex(r"/^<[A-Z]\w*(\s|>|\/)/"),
            regex(r"/^<[a-z]+(\s|>|\/)/"),
            regex(r"/<[A-Z]\w*(\s.*)?>/"),
            regex(r"/<\/[A-Z]\w*>/"),
            regex(r"/<[a-z]+(\s.*)?>/"),
            regex(r"/<\/[a-z]+>/"),
        ],
        header: js_regex(r"^[a-zA-Z]"),
        header_then_text: js_regex(r"^[A-Z][A-Za-z]*(?:\s+\d+)?(?:\s+[a-z]+)*:\s+[A-Za-z]"),
        lowercase_header_then_sentence: js_regex(
            r"^[a-z][A-Za-z0-9_/-]*:\s+[A-Za-z][\w'-]*(?:\s+[A-Za-z][\w'-]*){2,}",
        ),
        parenthetical_prose: js_regex(r"\s\([A-Za-z][\w\s,.'/-]*\)"),
        fenced_block: Regex::new(r"```(?s:.)*?```").unwrap_or_else(|e| panic!("{e}")),
        backtick_code: js_regex(r"`[^`]*`"),
        alphanumeric: js_regex(r"[a-zA-Z0-9]"),
        method_call_pattern: js_regex(r"\.[A-Za-z_][A-Za-z0-9_]*\("),
        special_chars: js_regex(r"[{}[\]()`=<>]"),
        colon_pattern: js_regex(r#":\s*(['"`[{(]|\w+\s*=>)"#),
        control_flow: Regex::new(&format!(
            r"(?:^|[^A-Za-z0-9_])if[{s}]*\(|(?:^|[^A-Za-z0-9_])else(?:[^A-Za-z0-9_]|$)|=>",
            s = JS_SPACE_CLASS_ITEMS
        ))
        .unwrap_or_else(|e| panic!("{e}")),
    }
});

/// JavaScript's `String.prototype.trim` whitespace: `\s` minus U+0085, plus U+FEFF.
fn is_js_whitespace(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{FEFF}'
}

fn is_descriptive_comment_text(patterns: &Patterns, text: &str) -> bool {
    if let Some((header, _)) = text.split_once(':') {
        let header = header.trim_matches(is_js_whitespace);
        if !header.is_empty() && patterns.header.is_match(header) && header.contains(' ') {
            return true;
        }
    }

    let trimmed = text.trim_matches(is_js_whitespace);
    if trimmed.is_empty() || patterns.method_call_pattern.is_match(trimmed) {
        return false;
    }
    let without_parenthetical_prose = if trimmed.contains('(') {
        patterns.parenthetical_prose.replace_all(trimmed, "")
    } else {
        trimmed.into()
    };
    if patterns
        .special_chars
        .is_match(&without_parenthetical_prose)
        || patterns.colon_pattern.is_match(trimmed)
        || patterns.control_flow.is_match(trimmed)
    {
        return false;
    }
    trimmed.contains(':')
        && (patterns.header_then_text.is_match(trimmed)
            || patterns.lowercase_header_then_sentence.is_match(trimmed))
}

/// The pattern that makes `comment` look like code, if any.
fn commented_code_pattern(patterns: &Patterns, comment: &str, is_block: bool) -> Option<String> {
    if comment.starts_with('/') {
        return None;
    }
    let trimmed_start = comment.trim_start_matches(is_js_whitespace);
    if comment.starts_with('*')
        || trimmed_start.starts_with("eslint-disable")
        || IGNORED_SUBSTRINGS.iter().any(|s| comment.contains(s))
        || patterns.jsdoc_comment.is_match(comment)
        || ALLOWED_PREFIXES
            .iter()
            .any(|prefix| trimmed_start.starts_with(prefix))
        || comment.contains("https://")
        || is_descriptive_comment_text(patterns, trimmed_start)
    {
        return None;
    }

    if is_block {
        return patterns
            .jsx
            .iter()
            .find_map(|pattern| pattern.find(comment));
    }

    if let Some(pattern) = START_PATTERNS
        .iter()
        .find(|pattern| trimmed_start.starts_with(*pattern))
    {
        return Some((*pattern).to_string());
    }
    patterns
        .code
        .iter()
        .find_map(|pattern| pattern.find(comment))
}

/// The comment text with inline code (`` `x` ``, and fenced blocks in block comments) removed when there is prose
/// around it, so code shown as an example in a sentence isn't taken for commented-out code.
fn strip_inline_code_if_text_outside(
    patterns: &Patterns,
    original: &str,
    is_block: bool,
) -> String {
    let trimmed = original.trim_matches(is_js_whitespace);
    if trimmed.starts_with('`') && ["`,", "`;", "`"].iter().any(|end| trimmed.ends_with(end)) {
        return original.to_string();
    }

    let mut cleaned = original.to_string();
    if is_block && cleaned.contains("```") {
        let has_text = patterns
            .fenced_block
            .split(&cleaned)
            .any(|part| patterns.alphanumeric.is_match(part));
        if has_text {
            cleaned = patterns.fenced_block.replace_all(&cleaned, "").into_owned();
        }
    }

    if !cleaned.contains('`') {
        return cleaned;
    }
    let has_outer_text = patterns
        .backtick_code
        .split(&cleaned)
        .any(|segment| patterns.alphanumeric.is_match(segment));
    if has_outer_text {
        patterns
            .backtick_code
            .replace_all(&cleaned, "")
            .into_owned()
    } else {
        cleaned
    }
}

/// Whether only whitespace with a single line break separates two line comments.
fn continues_line_comment(text: &str, previous_end: usize, start: usize) -> bool {
    text.get(previous_end..start).is_some_and(|between| {
        between.chars().filter(|&c| c == '\n').count() == 1 && between.chars().all(is_js_whitespace)
    })
}

fn check_comments(ctx: &mut Ctx, explanation_prefixes: &[String]) {
    let patterns = &*PATTERNS;
    let text = ctx.text();
    let mut explanation_block_end: Option<usize> = None;
    for (start, end, is_block) in comments(ctx.file) {
        let (Ok(start_index), Ok(end_index)) = (usize::try_from(start), usize::try_from(end))
        else {
            continue;
        };
        let raw = text.get(start_index..end_index).unwrap_or_default();
        let value = if is_block {
            raw.get(2..raw.len().saturating_sub(2)).unwrap_or_default()
        } else {
            raw.get(2..).unwrap_or_default()
        };
        let continues_explanation = !is_block
            && explanation_block_end.is_some_and(|previous_end| {
                continues_line_comment(text, previous_end, start_index)
            });
        let starts_explanation = explanation_prefixes.iter().any(|prefix| {
            value
                .trim_start_matches(is_js_whitespace)
                .starts_with(prefix.as_str())
        });
        if continues_explanation || (starts_explanation && !is_block) {
            explanation_block_end = Some(end_index);
            continue;
        }
        explanation_block_end = None;
        if starts_explanation {
            continue;
        }
        let processed = strip_inline_code_if_text_outside(patterns, value, is_block);
        let Some(wrong_pattern) = commented_code_pattern(patterns, &processed, is_block) else {
            continue;
        };
        let (Ok(pos), Ok(end)) = (i32::try_from(start), i32::try_from(end)) else {
            continue;
        };
        ctx.report_range(
            pos,
            end,
            RuleMessage::new(
                "commentedOutCode",
                format!(
                    "Commented code is not allowed. Detected pattern: `{wrong_pattern}` Use a comment starting with one of these prefixes if you want to keep this code commented out: {}",
                    ALLOWED_PREFIXES.join(", ")
                ),
            ),
        );
    }
}
