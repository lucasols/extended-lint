mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-redundant-function-params";

#[test]
fn allows_calls_without_redundant_default_parameters() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                inline({ align: 'left' })
                createElement(App, 'span')
                configure({ debug: true })
            "},
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center" }] }, { "name": "createElement", "defaults": [null, "div"] }, { "name": "configure", "defaults": [{ "debug": false }] }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn allows_calls_with_no_parameters_when_function_has_defaults() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                inline()
                createElement(App)
                configure()
            "},
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center" }] }, { "name": "createElement", "defaults": [null, "div"] }, { "name": "configure", "defaults": [{ "debug": false }] }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn allows_calls_with_partial_parameters() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                complex(1, { enabled: false })
                complex(2)
            "},
            options: Some(
                json!({ "functions": [{ "name": "complex", "defaults": [null, { "enabled": true }, "default"] }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn disallows_calls_with_simple_default_value() {
    let result = invalid(
        RULE,
        Case {
            code: r"createElement(App, 'div')",
            options: Some(
                json!({ "functions": [{ "name": "createElement", "defaults": [null, "div"] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 2 with value "div"'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"createElement(App)");
}

#[test]
fn disallows_calls_with_object_default_value() {
    let result = invalid(
        RULE,
        Case {
            code: r"inline({ align: 'center' })",
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center" }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 (object)'
      line: 1
    ");
    assert_snapshot!(result.output, @r"inline()");
}

#[test]
fn disallows_calls_with_complex_object_default_value() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                setup({
                  theme: 'dark',
                  features: { analytics: true, auth: false }
                })
            "},
            options: Some(
                json!({ "functions": [{ "name": "setup", "defaults": [{ "theme": "dark", "features": { "analytics": true, "auth": false } }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 (object)'
      line: 1
    ");
    assert_snapshot!(result.output, @r"setup()");
}

#[test]
fn disallows_multiple_redundant_parameters() {
    let result = invalid(
        RULE,
        Case {
            code: r"configure(true, 'dev', { verbose: false })",
            options: Some(
                json!({ "functions": [{ "name": "configure", "defaults": [true, "dev", { "verbose": false }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value true, Param 2 with value "dev", Param 3 (object)'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"configure()");
}

#[test]
fn removes_only_trailing_redundant_parameters() {
    let result = invalid(
        RULE,
        Case {
            code: r"advanced(1, 'default', true)",
            options: Some(
                json!({ "functions": [{ "name": "advanced", "defaults": [null, "default", true] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 2 with value "default", Param 3 with value true'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"advanced(1)");
}

#[test]
fn handles_mixed_redundant_and_non_redundant_parameters() {
    let result = invalid(
        RULE,
        Case {
            code: r"mixed(42, 'custom', 'default')",
            options: Some(
                json!({ "functions": [{ "name": "mixed", "defaults": [null, null, "default"] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 3 with value "default"'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"mixed(42, 'custom')");
}

#[test]
fn ignores_functions_not_in_configuration() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                unknownFunction({ align: 'center' })
                anotherFunction('div')
            "},
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center" }] }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn works_with_member_expressions() {
    let result = invalid(
        RULE,
        Case {
            code: r"obj.method('default')",
            options: Some(json!({ "functions": [{ "name": "method", "defaults": ["default"] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value "default"'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"obj.method()");
}

#[test]
fn handles_boolean_default_values() {
    let result = invalid(
        RULE,
        Case {
            code: r"toggle(false)",
            options: Some(json!({ "functions": [{ "name": "toggle", "defaults": [false] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value false'
      line: 1
    ");
    assert_snapshot!(result.output, @r"toggle()");
}

#[test]
fn handles_null_and_undefined_default_values() {
    let result = invalid(
        RULE,
        Case {
            code: r"nullable(null)",
            options: Some(json!({ "functions": [{ "name": "nullable", "defaults": [null] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value null'
      line: 1
    ");
    assert_snapshot!(result.output, @r"nullable()");
}

#[test]
fn reproduce_bug() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const Component = styled.div`
                  color: red;
                  ${inline({ align: 'center', justify: 'center' })}
                `
            "},
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center", "justify": "left", "gap": 0 }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "align" with value "center"'
      line: 3
    "#);
    assert_snapshot!(result.output, @r"
    const Component = styled.div`
      color: red;
      ${inline({ justify: 'center' })}
    `
    ");
}

#[test]
fn reproduce_bug_2() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const Component = styled.div`
                  color: red;
                  ${inline({ align: 'center', justify: 'left', gap: 5 })}
                `
            "},
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center", "justify": "left", "gap": 0 }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "align" with value "center", Param 1 property "justify" with value "left"'
      line: 3
    "#);
    assert_snapshot!(result.output, @r"
    const Component = styled.div`
      color: red;
      ${inline({ gap: 5 })}
    `
    ");
}

#[test]
fn dont_keep_empty_objects() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const Component = styled.div`
                  color: red;
                  ${inline({ align: 'center' })};
                `
            "},
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center", "justify": "left", "gap": 0 }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "align" with value "center"'
      line: 3
    "#);
    assert_snapshot!(result.output, @r"
    const Component = styled.div`
      color: red;
      ${inline()};
    `
    ");
}

#[test]
fn dont_allow_empty_objects() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const Component = styled.div`
                  color: red;
                  ${inline({})};
                `
            "},
            options: Some(
                json!({ "functions": [{ "name": "inline", "defaults": [{ "align": "center", "justify": "left", "gap": 0 }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 (object)'
      line: 3
    ");
    assert_snapshot!(result.output, @r"
    const Component = styled.div`
      color: red;
      ${inline()};
    `
    ");
}

#[test]
fn extra_regex_default_object() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn(/a/)",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [{}] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value {}'
      line: 1
    ");
    assert_snapshot!(result.output, @r"fn()");
}

#[test]
fn extra_array_default() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn([1, 2])",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [[1, 2]] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value [1,2]'
      line: 1
    ");
    assert_snapshot!(result.output, @r"fn()");
}

#[test]
fn extra_array_partial() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn([1, 3])",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [[1, 2]] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "0" with value 1'
      line: 1
    "#);
}

#[test]
fn extra_length_of_array_default() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn({ length: 2 })",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [[1, 2]] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "length" with value 2'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"fn()");
}

#[test]
fn extra_spread_arguments() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                fn(...args)
                fn(1, ...args)
            "},
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": ["a", 1] }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_nested_default_object_value() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn({ a: { x: 1 }, b: 2 })",
            options: Some(
                json!({ "functions": [{ "name": "fn", "defaults": [{ "a": { "x": 1 }, "b": 1 }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "a" with value {"x":1}'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"fn({ b: 2 })");
}

#[test]
fn extra_numbers() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn(1e21, 0x10, 1_000, 0.5, 1e-7)",
            options: Some(
                json!({ "functions": [{ "name": "fn", "defaults": [1e+21, 16, 1000, 0.5, 1e-7] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value 1e+21, Param 2 with value 16, Param 3 with value 1000, Param 4 with value 0.5, Param 5 with value 1e-7'
      line: 1
    ");
    assert_snapshot!(result.output, @r"fn()");
}

#[test]
fn extra_string_escapes() {
    let result = invalid(
        RULE,
        Case {
            code: r#"fn('a"b\n')"#,
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": ["a\"b\n"] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value "a\"b\n"'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"fn()");
}

#[test]
fn extra_computed_and_shorthand_keys() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                fn({ ['a']: 1 })
                fn({ a })
            "},
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [{ "a": 1 }] }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_duplicate_keys() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn({ a: 2, a: 1 })",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [{ "a": 1 }] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 (object)'
      line: 1
    ");
    assert_snapshot!(result.output, @r"fn()");
}

#[test]
fn extra_computed_member_callee() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                obj['fn'](1)
                obj.fn(1)
            "},
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [1] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value 1'
      line: 2
    ");
    assert_snapshot!(result.output, @r"
    obj['fn'](1)
    obj.fn()
    ");
}

#[test]
fn extra_redundant_object_before_non_redundant() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn({}, 5)",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [{ "a": 1 }, 1] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 (object)'
      line: 1
    ");
}

#[test]
fn extra_trailing_comma_and_comment() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn(1, 'a', /* c */)",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [null, "a"] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 2 with value "a"'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"fn(1)");
}

#[test]
fn extra_negative_and_undefined() {
    valid(
        RULE,
        Case {
            code: r"fn(-1, undefined)",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [-1, null] }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_partial_with_spread_and_method() {
    let result = invalid(
        RULE,
        Case {
            code: r"fn({ ...x, a: 1, m() {}, b: 2 })",
            options: Some(
                json!({ "functions": [{ "name": "fn", "defaults": [{ "a": 1, "b": 1 }] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "a" with value 1'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"fn({ ...x, m() {}, b: 2 })");
}

#[test]
fn extra_array_holes() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                fn([, 1])
                fn([null, 1])
            "},
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [[null, 1]] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 property "1" with value 1'
      line: 1
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value [null,1]'
      line: 2
    "#);
    assert_snapshot!(result.output, @r"
    fn([, 1])
    fn()
    ");
}

#[test]
fn extra_optional_member_call() {
    let result = invalid(
        RULE,
        Case {
            code: r"obj?.fn('a')",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": ["a"] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'redundantParams'
      msg: 'Function call has redundant parameters matching its default values and can be removed: Param 1 with value "a"'
      line: 1
    "#);
    assert_snapshot!(result.output, @r"obj?.fn()");
}

#[test]
fn extra_template_literal_argument() {
    valid(
        RULE,
        Case {
            code: r"fn(`a`)",
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": ["a"] }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_last_config_wins() {
    valid(
        RULE,
        Case {
            code: r"fn(1)",
            options: Some(
                json!({ "functions": [{ "name": "fn", "defaults": [1] }, { "name": "fn", "defaults": [2] }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn removes_arguments_between_the_call_parentheses() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                fn<string>('a')
                fn?.('a')
                fn((1), 'a')
            "},
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": [null, "a"] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.output, @r"
    fn<string>('a')
    fn?.('a')
    fn((1))
    ");

    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                fn<string>('a')
                fn?.('a')
            "},
            options: Some(json!({ "functions": [{ "name": "fn", "defaults": ["a"] }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.output, @r"
    fn<string>()
    fn?.()
    ");
}
