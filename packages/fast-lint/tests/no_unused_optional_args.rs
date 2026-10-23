mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-unused-optional-args";

#[test]
fn exported_function_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export function f(a?: string, b?: string) {
                  return "+" + a + b
                }
                f('x')
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn unused_args_inside_function_are_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, b?: string) {
                  return "+" + a
                }
                f('x', 'y')
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn skip_when_usage_cannot_be_inferred() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, b?: string) { return '' + a + b }
                const g = f
                g('x', 'y')
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn do_not_flag_when_at_least_one_call_provides_arg() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, b?: string) { return '' + a + b }
                f('x')
                f('x', 'y')
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn ignore_params_by_regex() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(_a?: string, b?: string) { return '' + _a + b }
                f('x', 'y')
            "#},
            options: Some(json!([{"ignoreArgsMatching":"^_"}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn report_param_never_passed_by_any_call() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, b?: string) {
                  return "+" + a + b
                }
                f('ok')
                f('test')
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalArg', line: 1 }");
}

#[test]
fn report_for_arrow_function_variable() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const f = (a?: number, b?: number) => a && b ? a + b : 0
                f(1)
                f(2)
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalArg', line: 1 }");
}

#[test]
fn report_optional_object_param_never_passed() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(props?: { x: number }) { return props?.x ?? 0 }
                f()
                f()
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalArg', line: 1 }");
}

#[test]
fn skip_if_any_call_uses_spread_args() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, b?: string) { return '' + a + b }
                const args = ['x', 'y'] as const
                f(...args)
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn jsx_usage_counts_as_providing_first_arg() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const Component = (props?: { x: number }) => null

                const App = () => {
                  return <Component />
                }
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn jsx_usage_with_fc_typed_variable() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const Component: FC<{ x: number }> = (props?: { x: number }) => null

                const App = () => <Component />
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn report_unused_optional_prop_for_jsx_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const Component = (props: { a?: string; b?: number }) => null

                const App = () => {
                  return <Component a="x" />
                }
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalProp', line: 1 }");
}

#[test]
fn report_unused_optional_prop_for_fc_typed_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const Component: FC<{ a?: string; b?: number }> = () => null

                const App = () => <Component a="x" />
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalProp', line: 1 }");
}

#[test]
fn do_not_report_when_prop_provided_somewhere() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const Component = (props: { a?: string; b?: number }) => null

                const App = () => {
                  return (
                    <>
                      <Component a="x" />
                      <Component b={1} />
                    </>
                  )
                }
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn bail_when_jsx_spread_props_present() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const Component = (props: { a?: string; b?: number }) => null

                const App = () => {
                  const p = { a: 'x' }
                  return <Component {...p} />
                }
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn report_unused_optional_prop_in_normal_call() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function withProps(props: { x?: number; b?: string }) {
                  return null
                }

                withProps({ x: 1 })
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalProp', line: 1 }");
}

#[test]
fn report_unused_optional_prop_in_second_param_object() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, options?: { b?: string; c?: number }) {}

                f('ok', { b: 'ok' })
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalProp', line: 1 }");
}

#[test]
fn bail_when_non_object_arg_is_passed() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, options?: { b?: string; c?: number }) {}
                const opts = { b: 'x' }
                f('ok', opts)
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn do_not_report_when_some_calls_provide_missing_prop() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, options?: { b?: string; c?: number }) {}
                f('ok', { b: 'ok' })
                f('ok', { c: 1 })
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn pass_function_as_argument_to_another_function_skip() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: string, b?: string) { return '' + a + b }

                function run(g: (x?: string, y?: string) => void) {
                  g('a')
                }

                run(f)
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn pass_function_as_jsx_prop_skip() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const f = (a?: number, b?: number) => {}
                const Btn = () => <button onClick={f} />
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn return_function_from_another_function_skip() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function createHandler(a?: string, b?: string) { return '' + a + b }

                function getHandler() {
                  return createHandler
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn return_arrow_function_skip() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const handler = (a?: string, b?: string) => a + b

                const getHandler = () => handler
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn return_function_as_object_property_skip() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function load(options?: { delay?: number }) {
                  return options?.delay || 0
                }

                function createApi() {
                  return {
                    load,
                    other: 'value'
                  }
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn return_function_as_object_property_with_different_name_skip() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function load(options?: { delay?: number }) {
                  return options?.delay || 0
                }

                function createApi() {
                  return {
                    handler: load,
                    other: 'value'
                  }
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn functions_with_same_name_in_different_scopes_inner_function_returned_still_skipped() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function handler(a?: string) { 
                  return a || 'default'
                }
                handler('test')  // This call exists

                function createHandler() {
                  function handler(b?: number) {  // Same name, different scope
                    return b || 0
                  }
                  // No calls to inner handler - would report error but skipped because returned
                  return handler
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn functions_with_same_name_in_different_scopes_both_analyzed_independently() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function process(data?: string) {
                  return data
                }
                process('outer')

                function wrapper() {
                  function process(value?: number) {  // Same name
                    return value
                  }
                  process(42)  // Inner call
                  return process
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn unique_function_names_still_report_errors_correctly() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function uniqueHandler(a?: string, b?: string) {
                  return a + b
                }
                uniqueHandler('test')  // Only first param used

                function anotherUnique(x?: number) {
                  return x || 0
                }
                anotherUnique()  // No params used
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedOptionalArg', line: 1 }
    - { messageId: 'unusedOptionalArg', line: 6 }
    ");
}

#[test]
fn exported_fc_component_with_unused_optional_props_should_be_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const FieldAndAppOriginsForm: FC<{
                  tabsMock: UseMockAllTabs;
                  creationTestsOnly?: boolean;
                }> = ({ tabsMock, creationTestsOnly }) => {
                  return null
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn reproduce_false_positive_exported_fc_component_should_not_report_unused_optional_props() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const Component: FC<{
                  required: string;
                  optional?: boolean;
                }> = ({ required, optional }) => {
                  return <div>{required}</div>
                }

                const App = () => <Component required="test" />
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn parenthesized_callee_and_object_argument() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(options?: { a?: number; b?: number }) {
                  return options
                }
                (f)(({ a: 1 }))
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalProp', line: 1 }");
}

#[test]
fn method_call_on_object_argument_with_computed_identifier_key() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const key = 1
                function f(options: { key?: number; b?: number }) {
                  return options
                }
                f({ [key]: 1 })
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedOptionalProp', line: 2 }");
}

#[test]
fn function_returned_in_an_object_property_is_skipped() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f(a?: number) {
                  return a
                }
                f()
                function g() {
                  return ({ f })
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}
