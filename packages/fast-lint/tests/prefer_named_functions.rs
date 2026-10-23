mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "prefer-named-functions";

fn ignore_fn_suffix() -> Option<serde_json::Value> {
    Some(json!({ "ignoreRegex": "Fn$" }))
}

#[test]
fn named_function_exports() {
    valid(
        RULE,
        indoc! {"
            export function test() {
            }
            export function anotherTest() {
            }
        "},
    );
}

#[test]
fn exported_function_expressions() {
    valid(
        RULE,
        indoc! {"
            export const test = function() {
            }
            export const anotherTest = function namedFn() {
            }
        "},
    );
}

#[test]
fn iife() {
    valid(
        RULE,
        indoc! {"
            (() => {})();

            (function () {})();

            (async () => {})();
        "},
    );
}

#[test]
fn typed_arrow_functions_are_allowed() {
    valid(
        RULE,
        indoc! {"
            export const test: VoidFunction = () => {
            }
        "},
    );
}

#[test]
fn arrow_functions_matching_ignore_regex_are_allowed() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                export const testFn = (a: T) => {
                }
            "},
            options: ignore_fn_suffix(),
            filename: None,
        },
    );
}

#[test]
fn one_line_arrow_function_with_implicit_return() {
    valid(RULE, "const test = () => true");
}

#[test]
fn arrow_function_with_implicit_return_in_multiple_lines() {
    valid(
        RULE,
        indoc! {"
            const test = (a: number, b: string) =>
              a + b;
        "},
    );
}

#[test]
fn exported_arrow_function() {
    let result = invalid(
        RULE,
        indoc! {"
            export const test = (arg1: string) => {
              console.log('test');
              return null;
            }
        "},
    );

    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'default'
      msg: 'Function test should be defined as a named function "function test () {}" instead of an arrow function'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    export function test(arg1: string) {
      console.log('test');
      return null;
    }
    ");
}

#[test]
fn multiple_exported_arrow_functions() {
    let result = invalid(
        RULE,
        indoc! {"
            export const test = () => {
            }
            export const anotherTest = () => {
            }
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'default', line: 1 }
    - { messageId: 'default', line: 3 }
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[0], 0), @r"
    export function test() {
    }
    export const anotherTest = () => {
    }
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[1], 0), @r"
    export const test = () => {
    }
    export function anotherTest() {
    }
    ");
}

#[test]
fn exported_arrow_function_with_parameters() {
    let result = invalid(
        RULE,
        indoc! {"
            export const test = (a: number, b: string) => {
            }
        "},
    );

    assert_snapshot!(result.suggestion_output(0), @r"
    export function test(a: number, b: string) {
    }
    ");
}

#[test]
fn exported_async_arrow_function() {
    let result = invalid(
        RULE,
        indoc! {"
            export const test = async () => {
            }
        "},
    );

    assert_snapshot!(result.suggestion_output(0), @r"
    export async function test() {
    }
    ");
}

#[test]
fn name_not_matching_ignore_regex() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                export const test = (a: T) => {
                }
            "},
            options: ignore_fn_suffix(),
            filename: None,
        },
    );

    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'withIgnoreRegex'
      msg: 'Function test should be defined as a named function "function test () {}", if not possible to use `function` you can also use a name that matches the rule ignore regex'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    export function test(a: T) {
    }
    ");
}

#[test]
fn arrow_function() {
    let result = invalid(
        RULE,
        indoc! {"
            const test = () => {
            }
            const anotherTest = () => {
            }
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'default', line: 1 }
    - { messageId: 'default', line: 3 }
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[0], 0), @r"
    function test() {
    }
    const anotherTest = () => {
    }
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[1], 0), @r"
    const test = () => {
    }
    function anotherTest() {
    }
    ");
}

#[test]
fn async_arrow_function() {
    let result = invalid(
        RULE,
        indoc! {"
            const test = async () => {
            }
        "},
    );

    assert_snapshot!(result.suggestion_output(0), @r"
    async function test() {
    }
    ");
}

#[test]
fn suggestion_replaces_the_declaration_through_its_semicolon() {
    let result = invalid(
        RULE,
        indoc! {"
            const test = (a = 1, ...rest: number[]) => {
            };
            const other = 1;
        "},
    );

    assert_snapshot!(result.suggestion_output(0), @r"
    function test(a = 1, ...rest: number[]) {
    }
    const other = 1;
    ");
}

#[test]
fn implicit_returns_are_reported_when_disallowed() {
    let result = invalid(
        RULE,
        Case {
            code: "const test = () => true",
            options: Some(json!({ "disallowArrowFnWithImplicitReturns": true })),
            filename: None,
        },
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 1 }");
}
