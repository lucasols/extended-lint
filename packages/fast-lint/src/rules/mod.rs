use tsrslint::rule::Rule;

pub mod improved_no_unnecessary_condition;
pub mod naming_convention;
pub mod no_call_with_explicit_generics;
pub mod no_call_with_inferred_generics;
pub mod no_commented_out_code;
pub mod no_default_export;
pub mod no_duplicate_key;
pub mod no_leaked_conditional_rendering;
pub mod no_leaked_text_in_jsx;
pub mod no_non_camel_case_functions;
pub mod no_optional_root_props;
pub mod no_redundant_function_params;
pub mod no_reexport;
pub mod no_relative_imports;
pub mod no_restricted_syntax;
pub mod no_static_style_prop;
pub mod no_sync_dynamic_import;
pub mod no_type_guards;
pub mod no_unnecessary_iife;
pub mod no_unnecessary_typing;
pub mod no_unnecessary_void_on_promise;
pub mod no_unused_optional_args;
pub mod no_unused_selected_values;
pub mod no_unused_t_state_field;
pub mod no_unused_type_props_in_args;
pub mod no_write_only_ref;
pub mod prefer_named_functions;
pub mod prefer_react_hook_alternative;
pub mod react_compiler_extra;
pub mod react_no_module_level_single_use_values;
pub mod react_single_export;
pub mod require_description;
pub mod require_lazy_execution;
pub mod require_reads_to_var_prop;
pub mod sort_react_components_and_styles;
pub mod strict_casting;
pub mod use_state_setter_naming;
pub mod use_types_directly_above_usage;

/// The rules that run when no config file is given: the type-aware ones. Every other rule only runs when configured.
pub const DEFAULT_RULE_NAMES: &[&str] = &[
    improved_no_unnecessary_condition::NAME,
    no_leaked_conditional_rendering::NAME,
    no_unnecessary_void_on_promise::NAME,
    strict_casting::NAME,
];

pub const RULE_NAMES: &[&str] = &[
    improved_no_unnecessary_condition::NAME,
    naming_convention::NAME,
    no_call_with_explicit_generics::NAME,
    no_call_with_inferred_generics::NAME,
    no_commented_out_code::NAME,
    no_default_export::NAME,
    no_duplicate_key::NAME,
    no_leaked_conditional_rendering::NAME,
    no_leaked_text_in_jsx::NAME,
    no_non_camel_case_functions::NAME,
    no_optional_root_props::NAME,
    no_redundant_function_params::NAME,
    no_reexport::NAME,
    no_relative_imports::NAME,
    no_restricted_syntax::NAME,
    no_static_style_prop::NAME,
    no_sync_dynamic_import::NAME,
    no_type_guards::NAME,
    no_unnecessary_iife::NAME,
    no_unnecessary_typing::NAME,
    no_unnecessary_void_on_promise::NAME,
    no_unused_optional_args::NAME,
    no_unused_selected_values::NAME,
    no_unused_t_state_field::NAME,
    no_unused_type_props_in_args::NAME,
    no_write_only_ref::NAME,
    prefer_named_functions::NAME,
    prefer_react_hook_alternative::NAME,
    react_compiler_extra::NAME,
    react_no_module_level_single_use_values::NAME,
    react_single_export::NAME,
    require_description::NAME,
    require_lazy_execution::NAME,
    require_reads_to_var_prop::NAME,
    sort_react_components_and_styles::NAME,
    strict_casting::NAME,
    use_state_setter_naming::NAME,
    use_types_directly_above_usage::NAME,
];

pub fn create_rule(
    name: &str,
    options: Option<&serde_json::Value>,
) -> Result<Box<dyn Rule>, String> {
    let rule = match name {
        improved_no_unnecessary_condition::NAME => {
            improved_no_unnecessary_condition::create(options)
        }
        naming_convention::NAME => naming_convention::create(options),
        no_call_with_explicit_generics::NAME => no_call_with_explicit_generics::create(options),
        no_call_with_inferred_generics::NAME => no_call_with_inferred_generics::create(options),
        no_commented_out_code::NAME => no_commented_out_code::create(options),
        no_default_export::NAME => no_default_export::create(options),
        no_duplicate_key::NAME => no_duplicate_key::create(options),
        no_leaked_conditional_rendering::NAME => no_leaked_conditional_rendering::create(options),
        no_leaked_text_in_jsx::NAME => no_leaked_text_in_jsx::create(options),
        no_non_camel_case_functions::NAME => no_non_camel_case_functions::create(options),
        no_optional_root_props::NAME => no_optional_root_props::create(options),
        no_redundant_function_params::NAME => no_redundant_function_params::create(options),
        no_reexport::NAME => no_reexport::create(options),
        no_relative_imports::NAME => no_relative_imports::create(options),
        no_restricted_syntax::NAME => no_restricted_syntax::create(options),
        no_static_style_prop::NAME => no_static_style_prop::create(options),
        no_sync_dynamic_import::NAME => no_sync_dynamic_import::create(options),
        no_type_guards::NAME => no_type_guards::create(options),
        no_unnecessary_iife::NAME => no_unnecessary_iife::create(options),
        no_unnecessary_typing::NAME => no_unnecessary_typing::create(options),
        no_unnecessary_void_on_promise::NAME => no_unnecessary_void_on_promise::create(options),
        no_unused_optional_args::NAME => no_unused_optional_args::create(options),
        no_unused_selected_values::NAME => no_unused_selected_values::create(options),
        no_unused_t_state_field::NAME => no_unused_t_state_field::create(options),
        no_unused_type_props_in_args::NAME => no_unused_type_props_in_args::create(options),
        no_write_only_ref::NAME => no_write_only_ref::create(options),
        prefer_named_functions::NAME => prefer_named_functions::create(options),
        prefer_react_hook_alternative::NAME => prefer_react_hook_alternative::create(options),
        react_compiler_extra::NAME => react_compiler_extra::create(options),
        react_no_module_level_single_use_values::NAME => {
            react_no_module_level_single_use_values::create(options)
        }
        react_single_export::NAME => react_single_export::create(options),
        require_description::NAME => require_description::create(options),
        require_lazy_execution::NAME => require_lazy_execution::create(options),
        require_reads_to_var_prop::NAME => require_reads_to_var_prop::create(options),
        sort_react_components_and_styles::NAME => sort_react_components_and_styles::create(options),
        strict_casting::NAME => strict_casting::create(options),
        use_state_setter_naming::NAME => use_state_setter_naming::create(options),
        use_types_directly_above_usage::NAME => use_types_directly_above_usage::create(options),
        _ => return Err(format!("unknown rule: {name}")),
    };
    rule.map_err(|e| format!("invalid options for rule {name}: {e}"))
}

/// Rule names as they appear in configs and disable directives may carry a plugin prefix
/// (`@ls-stack/strict-casting`, `@eslint-react/no-leaked-conditional-rendering`); only the last segment identifies
/// the rule. Former names map to the current one (`advanced-no-restricted-syntax` to `no-restricted-syntax`).
pub fn normalize_rule_name(name: &str) -> &str {
    let name = name.rsplit('/').next().unwrap_or(name).trim();
    if name == no_restricted_syntax::ALIAS {
        no_restricted_syntax::NAME
    } else {
        name
    }
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
