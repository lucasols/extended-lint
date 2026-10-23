mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "naming-convention";

#[test]
fn pascal_case_type_names_are_valid() {
    valid(
        RULE,
        indoc! {"
            class UserStore {}
            const Store = class StoreImpl {}
            interface UserData { name: string }
            type UserId = string
            enum Status { Active }
            function identity<TValue>(value: TValue): TValue {
              return value
            }
            type Keys<T> = { [k in keyof T]: T[k] }
        "},
    );
}

#[test]
fn type_names_starting_lowercase_or_with_underscore() {
    let result = invalid(
        RULE,
        indoc! {"
            class userStore {}
            const Store = class storeImpl {}
            interface User_Data { name: string }
            type userId = string
            enum status { Active }
            function identity<value>(arg: value): value {
              return arg
            }
            type Unwrap<T> = T extends Promise<infer inner> ? inner : T
        "},
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'typeName', line: 1, column: 7, endLine: 1, endColumn: 16 }
    - { messageId: 'typeName', line: 2, column: 21, endLine: 2, endColumn: 30 }
    - { messageId: 'typeName', line: 3, column: 11, endLine: 3, endColumn: 20 }
    - { messageId: 'typeName', line: 4, column: 6, endLine: 4, endColumn: 12 }
    - { messageId: 'typeName', line: 5, column: 6, endLine: 5, endColumn: 12 }
    - { messageId: 'typeName', line: 6, column: 19, endLine: 6, endColumn: 24 }
    - { messageId: 'typeName', line: 9, column: 42, endLine: 9, endColumn: 47 }
    "###);
    assert_snapshot!(result.diagnostics[0].message, @"Type names must be PascalCase");
}

#[test]
fn mapped_type_keys_are_not_type_names() {
    valid(RULE, "type Flags<T> = { [key in keyof T]: boolean }");
}

#[test]
fn values_not_starting_with_double_underscore_are_valid() {
    valid(
        RULE,
        indoc! {"
            const _private = 1
            let value = 2
            function run(_unused: number, ...rest: number[]) {}
            const { a, b: c = 1, ...others } = { a: 1, b: 2 }
            const [first, , second = 2, ...tail] = [1, 2, 3]
            class Store {
              __method(param: number) {}
              constructor(private readonly service: string) {}
            }
            const obj = { __key: 1 }
            const { __key: renamed } = obj
        "},
    );
}

#[test]
fn variables_and_functions_starting_with_double_underscore() {
    let result = invalid(
        RULE,
        indoc! {"
            const __a = 1
            let __b: number
            for (const __c of [1]) {}
            function __d() {}
            const fn = function __e() {}
        "},
    );
    assert_snapshot!(result.errors_with_columns(), @r"
    - { messageId: 'valueName', line: 1, column: 7, endLine: 1, endColumn: 10 }
    - { messageId: 'valueName', line: 2, column: 5, endLine: 2, endColumn: 8 }
    - { messageId: 'valueName', line: 3, column: 12, endLine: 3, endColumn: 15 }
    - { messageId: 'valueName', line: 4, column: 10, endLine: 4, endColumn: 13 }
    - { messageId: 'valueName', line: 5, column: 21, endLine: 5, endColumn: 24 }
    ");
    assert_snapshot!(
        result.diagnostics[0].message,
        @"Variable, function and parameter names must not start with `__`"
    );
}

#[test]
fn parameters_starting_with_double_underscore() {
    let result = invalid(
        RULE,
        indoc! {"
            function a(__a: number, __b = 1, ...__c: number[]) {}
            const b = (__d: number) => __d
            class C {
              constructor(private __e: string, readonly __f = '') {}
              method(__g: number) {}
              set value(__h: number) {}
            }
        "},
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'valueName', line: 1, column: 12, endLine: 1, endColumn: 15 }
    - { messageId: 'valueName', line: 1, column: 25, endLine: 1, endColumn: 28 }
    - { messageId: 'valueName', line: 1, column: 37, endLine: 1, endColumn: 40 }
    - { messageId: 'valueName', line: 2, column: 12, endLine: 2, endColumn: 15 }
    - { messageId: 'valueName', line: 4, column: 23, endLine: 4, endColumn: 26 }
    - { messageId: 'valueName', line: 4, column: 45, endLine: 4, endColumn: 48 }
    - { messageId: 'valueName', line: 5, column: 10, endLine: 5, endColumn: 13 }
    - { messageId: 'valueName', line: 6, column: 13, endLine: 6, endColumn: 16 }
    "###);
}

#[test]
fn destructured_names_starting_with_double_underscore() {
    let result = invalid(
        RULE,
        indoc! {"
            const { __a, b: __b, c: __c = 1, __d = 2, ...__e } = obj
            const [__f, __g = 1, ...__h] = arr
            const { nested: { __i } } = obj
            function f({ __j }: Props, [__k]: number[]) {}
        "},
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'valueName', line: 1, column: 9, endLine: 1, endColumn: 12 }
    - { messageId: 'valueName', line: 1, column: 17, endLine: 1, endColumn: 20 }
    - { messageId: 'valueName', line: 1, column: 25, endLine: 1, endColumn: 28 }
    - { messageId: 'valueName', line: 1, column: 34, endLine: 1, endColumn: 37 }
    - { messageId: 'valueName', line: 1, column: 46, endLine: 1, endColumn: 49 }
    - { messageId: 'valueName', line: 2, column: 8, endLine: 2, endColumn: 11 }
    - { messageId: 'valueName', line: 2, column: 13, endLine: 2, endColumn: 16 }
    - { messageId: 'valueName', line: 2, column: 25, endLine: 2, endColumn: 28 }
    - { messageId: 'valueName', line: 3, column: 19, endLine: 3, endColumn: 22 }
    - { messageId: 'valueName', line: 4, column: 14, endLine: 4, endColumn: 17 }
    - { messageId: 'valueName', line: 4, column: 29, endLine: 4, endColumn: 32 }
    "###);
}

#[test]
fn destructuring_assignment_targets_starting_with_double_underscore() {
    let result = invalid(
        RULE,
        indoc! {"
            let __a, __b, __c, __d, __e
            ;({ a: __a, __b, c: __c = 1, ...__d } = obj)
            ;[__e] = arr
        "},
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'valueName', line: 1, column: 5, endLine: 1, endColumn: 8 }
    - { messageId: 'valueName', line: 1, column: 10, endLine: 1, endColumn: 13 }
    - { messageId: 'valueName', line: 1, column: 15, endLine: 1, endColumn: 18 }
    - { messageId: 'valueName', line: 1, column: 20, endLine: 1, endColumn: 23 }
    - { messageId: 'valueName', line: 1, column: 25, endLine: 1, endColumn: 28 }
    - { messageId: 'valueName', line: 2, column: 8, endLine: 2, endColumn: 11 }
    - { messageId: 'valueName', line: 2, column: 13, endLine: 2, endColumn: 16 }
    - { messageId: 'valueName', line: 2, column: 21, endLine: 2, endColumn: 24 }
    - { messageId: 'valueName', line: 2, column: 33, endLine: 2, endColumn: 36 }
    - { messageId: 'valueName', line: 3, column: 3, endLine: 3, endColumn: 6 }
    "###);
}

#[test]
fn bodiless_signatures_catch_params_and_other_names_are_not_values() {
    valid(
        RULE,
        indoc! {"
            declare function __a(__b: number): void
            function overload(__c: string): void
            function overload(value: string) {}
            type Fn = (__d: number) => void
            interface Api { call(__e: number): void }
            try {} catch (__f) {}
            import { __g } from 'mod'
            const o = { __i() {} }
        "},
    );
}

#[test]
fn rest_and_destructured_params_of_signatures_are_values() {
    let result = invalid(
        RULE,
        indoc! {"
            type Fn = (...__a: number[]) => void
            interface Api { call({ __b }: Props): void }
        "},
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'valueName', line: 1, column: 15, endLine: 1, endColumn: 18 }
    - { messageId: 'valueName', line: 2, column: 24, endLine: 2, endColumn: 27 }
    "###);
}

#[test]
fn checks_can_be_disabled() {
    valid(
        RULE,
        Case {
            code: "class lower {}",
            options: Some(json!({ "typeNames": false })),
            ..Default::default()
        },
    );
    valid(
        RULE,
        Case {
            code: "const __a = 1",
            options: Some(json!([{ "values": false }])),
            ..Default::default()
        },
    );
}

#[test]
fn custom_regex_and_message() {
    let result = invalid(
        RULE,
        Case {
            code: "type IUser = { name: string }\nconst $value = 1",
            options: Some(json!({
                "typeNames": { "disallow": "^I[A-Z]", "message": "No I prefix" },
                "values": { "disallow": "^\\$" },
            })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'typeName'
      msg: 'No I prefix'
      line: 1
    - messageId: 'valueName'
      msg: 'Variable, function and parameter names must not start with `__`'
      line: 2
    ");
}

#[test]
fn invalid_regex_is_an_options_error() {
    let error =
        fast_lint::rules::create_rule(RULE, Some(&json!({ "values": { "disallow": "(" } })))
            .err()
            .unwrap_or_default();
    assert!(
        error.starts_with("invalid options for rule naming-convention: invalid regex \"(\""),
        "{error}"
    );
}
