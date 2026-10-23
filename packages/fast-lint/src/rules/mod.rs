use tsrslint::rule::Rule;

pub mod improved_no_unnecessary_condition;
pub mod no_leaked_conditional_rendering;
pub mod no_unnecessary_void_on_promise;
pub mod strict_casting;

pub const RULE_NAMES: &[&str] = &[
    improved_no_unnecessary_condition::NAME,
    no_leaked_conditional_rendering::NAME,
    no_unnecessary_void_on_promise::NAME,
    strict_casting::NAME,
];

pub fn create_rule(
    name: &str,
    options: Option<&serde_json::Value>,
) -> Result<Box<dyn Rule>, String> {
    let rule = match name {
        improved_no_unnecessary_condition::NAME => {
            improved_no_unnecessary_condition::create(options)
        }
        no_leaked_conditional_rendering::NAME => no_leaked_conditional_rendering::create(options),
        no_unnecessary_void_on_promise::NAME => no_unnecessary_void_on_promise::create(options),
        strict_casting::NAME => strict_casting::create(options),
        _ => return Err(format!("unknown rule: {name}")),
    };
    rule.map_err(|e| format!("invalid options for rule {name}: {e}"))
}

/// Rule names as they appear in configs and disable directives may carry a plugin prefix
/// (`@ls-stack/strict-casting`, `@eslint-react/no-leaked-conditional-rendering`); only the last segment identifies
/// the rule.
pub fn normalize_rule_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name).trim()
}

/// Parses rule options the way ESLint passes them: either the options object itself or a one-element array holding
/// it. A missing value means the defaults.
pub fn parse_options<T: serde::de::DeserializeOwned + Default>(
    options: Option<&serde_json::Value>,
) -> Result<T, String> {
    let value = match options {
        None | Some(serde_json::Value::Null) => return Ok(T::default()),
        Some(serde_json::Value::Array(items)) => match items.as_slice() {
            [] => return Ok(T::default()),
            [single] => single,
            _ => return Err("expected a single options object".to_string()),
        },
        Some(value) => value,
    };
    serde_json::from_value(value.clone()).map_err(|e| e.to_string())
}
