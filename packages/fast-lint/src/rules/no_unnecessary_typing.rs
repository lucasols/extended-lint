use rustc_hash::FxHashSet;
use tsrs_ast::{self as ast, Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleDiagnostic, RuleMessage, RuleVisitor};

use crate::utils::{estree_parent, member_identifier_property, skip_parens, type_annotation_range};

pub const NAME: &str = "no-unnecessary-typing";

const DEFAULT_ARRAY_METHODS: &[&str] = &[
    "find",
    "filter",
    "map",
    "forEach",
    "some",
    "every",
    "reduce",
    "findIndex",
    "sort",
];

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Methods {
    List(Vec<String>),
    Mode(MethodsMode),
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum MethodsMode {
    Array,
}

impl Default for Methods {
    fn default() -> Self {
        Methods::Mode(MethodsMode::Array)
    }
}

#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Options {
    methods: Methods,
}

pub struct NoUnnecessaryTyping {
    methods: FxHashSet<String>,
}

pub fn create(options: Option<&serde_json::Value>) -> Result<Box<dyn Rule>, String> {
    let options = super::parse_options::<Options>(options)?;
    let methods = match options.methods {
        Methods::List(methods) => methods.into_iter().collect(),
        Methods::Mode(MethodsMode::Array) => DEFAULT_ARRAY_METHODS
            .iter()
            .map(|&method| method.to_string())
            .collect(),
    };
    Ok(Box::new(NoUnnecessaryTyping { methods }))
}

const LISTENERS: &[Listener] = &[
    Listener::Enter(Kind::ArrowFunction),
    Listener::Enter(Kind::FunctionExpression),
];

impl Rule for NoUnnecessaryTyping {
    fn name(&self) -> &'static str {
        NAME
    }
    fn create_visitor(&'static self, _ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        Box::new(Visitor { rule: self })
    }
}

struct Visitor {
    rule: &'static NoUnnecessaryTyping,
}

impl Visitor {
    /// Whether `function` is the first argument of a call to one of the checked methods (`items.find(fn)`).
    fn is_checked_callback(&self, function: P<Node>) -> bool {
        let Some(call) = estree_parent(function).filter(|&parent| ast::is_call_expression(parent))
        else {
            return false;
        };
        let callee = skip_parens(call.as_call_expression().expression);
        let Some(method) = member_identifier_property(callee) else {
            return false;
        };
        self.rule.methods.contains(method)
            && call
                .arguments()
                .first()
                .is_some_and(|&first| skip_parens(first) == function)
    }
}

impl RuleVisitor for Visitor {
    fn listeners(&self) -> &[Listener] {
        LISTENERS
    }
    fn visit(&mut self, ctx: &mut Ctx, _l: Listener, node: P<Node>) {
        let annotated_params: Vec<P<Node>> = node
            .parameters()
            .iter()
            .filter_map(|&param| {
                let declaration = param.as_parameter_declaration();
                let is_plain_identifier = ast::is_identifier(declaration.name())
                    && declaration.dot_dot_dot_token().is_none()
                    && declaration.initializer().is_none();
                declaration.type_().filter(|_| is_plain_identifier)
            })
            .collect();
        if annotated_params.is_empty() || !self.is_checked_callback(node) {
            return;
        }
        for type_node in annotated_params {
            let (pos, end) = type_annotation_range(ctx, type_node);
            ctx.report_diagnostic_with_fixes(
                RuleDiagnostic {
                    pos,
                    end,
                    message: RuleMessage::new(
                        "unnecessaryTypeAnnotation",
                        "Unnecessary type annotation. TypeScript can infer this type from context.",
                    ),
                    labeled_ranges: Vec::new(),
                },
                |ctx| vec![ctx.fix_remove_range(pos, end)],
            );
        }
    }
}
