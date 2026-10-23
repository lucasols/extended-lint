mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-type-guards";

fn with_options(code: &str, options: serde_json::Value) -> Case<'_> {
    Case {
        code,
        options: Some(options),
        filename: None,
    }
}

#[test]
fn allows_type_guard_in_type_guards_file() {
    valid(
        RULE,
        with_options(
            indoc! {"
                function isString(value: unknown): value is string {
                  return typeof value === 'string'
                }
            "},
            json!({ "__dev_simulateFileName": "utils.typeGuards.ts" }),
        ),
    );
}

#[test]
fn allows_type_guard_in_type_guards_tsx_file() {
    valid(
        RULE,
        with_options(
            indoc! {"
                function isNumber(value: unknown): value is number {
                  return typeof value === 'number'
                }
            "},
            json!({ "__dev_simulateFileName": "helpers.typeGuards.tsx" }),
        ),
    );
}

#[test]
fn allows_type_guard_in_real_type_guards_file() {
    valid(
        RULE,
        Case {
            code: "function isString(value: unknown): value is string { return true }",
            options: None,
            filename: Some("utils.typeGuards.ts"),
        },
    );
}

#[test]
fn allows_normal_function_without_type_guard() {
    valid(
        RULE,
        indoc! {"
            function getString(value: unknown): string {
              return String(value)
            }
        "},
    );
}

#[test]
fn allows_function_with_regular_return_type_annotation() {
    valid(
        RULE,
        indoc! {"
            function process(): boolean {
              return true
            }
        "},
    );
}

#[test]
fn disallows_type_guard_in_regular_file() {
    let result = invalid(
        RULE,
        indoc! {"
            function isString(value: unknown): value is string {
              return typeof value === 'string'
            }
        "},
    );

    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'typeGuardNotAllowed', line: 1, column: 34, endLine: 1, endColumn: 51 }");
}

#[test]
fn disallows_type_guard_in_method() {
    let result = invalid(
        RULE,
        indoc! {"
            class Validator {
              isString(value: unknown): value is string {
                return typeof value === 'string'
              }
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'typeGuardNotAllowed', line: 2 }");
}

#[test]
fn disallows_type_guard_in_arrow_function() {
    let result = invalid(
        RULE,
        "const isNumber = (value: unknown): value is number => typeof value === 'number'",
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'typeGuardNotAllowed'
      msg: 'Check if the type guard can be inferred by typescript, in most cases it can, e.g. `.filter((nullable) => nullable !== null)`. If not, type guards are only allowed in *.typeGuards.(ts|tsx) or *.type-guards.(ts|tsx) files'
      line: 1
    ");
}

#[test]
fn disallows_multiple_type_guards() {
    let result = invalid(
        RULE,
        indoc! {"
            function isString(value: unknown): value is string {
              return typeof value === 'string'
            }

            function isNumber(value: unknown): value is number {
              return typeof value === 'number'
            }
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'typeGuardNotAllowed', line: 1 }
    - { messageId: 'typeGuardNotAllowed', line: 5 }
    ");
}

#[test]
fn allows_type_guard_in_type_guards_kebab_file() {
    valid(
        RULE,
        with_options(
            indoc! {"
                function isBoolean(value: unknown): value is boolean {
                  return typeof value === 'boolean'
                }
            "},
            json!({ "__dev_simulateFileName": "utils.type-guards.ts" }),
        ),
    );
}

#[test]
fn allows_type_guard_when_suggestions_disabled() {
    valid(
        RULE,
        with_options(
            indoc! {"
                const items = ['a', 1, 'b']
                const strings = items.filter((item): item is string => typeof item === 'string')
            "},
            json!({ "__dev_simulateFileName": "utils.typeGuards.ts" }),
        ),
    );
}

#[test]
fn shows_custom_message_for_filter_with_type_guard() {
    let result = invalid(
        RULE,
        with_options(
            indoc! {"
                const items = ['a', 1, 'b']
                const strings = items.filter((item): item is string => typeof item === 'string')
            "},
            json!({ "alternativeMsgs": { "inArrayFilter": "Replace type guard with type annotation" } }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'useFilterWithTypeCheck'
      msg: 'Replace type guard with type annotation'
      line: 2
    ");
}

#[test]
fn shows_custom_message_for_find_with_type_guard() {
    let result = invalid(
        RULE,
        with_options(
            indoc! {"
                const items = ['a', 1, 'b']
                const firstString = items.find((item): item is string => typeof item === 'string')
            "},
            json!({ "alternativeMsgs": { "inArrayFind": "Replace type guard with type annotation" } }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'useFindWithTypeCheck'
      msg: 'Replace type guard with type annotation'
      line: 2
    ");
}

#[test]
fn no_suggestions_when_specific_option_not_set() {
    let result = invalid(
        RULE,
        with_options(
            indoc! {"
                const items = ['a', 1, 'b']
                const strings = items.filter((item): item is string => typeof item === 'string')
            "},
            json!({ "alternativeMsgs": { "inArrayFind": "Find suggestion" } }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'typeGuardNotAllowed', line: 2 }");
}

#[test]
fn no_suggestions_without_alternative_msgs_in_nested_filter() {
    let result = invalid(
        RULE,
        indoc! {"
            const test = {
              appLeftSidebarMenu: apiAppConfig.bundle?.menu_bar ? {
                type: 'left_sidebar' as const,
                title: apiAppConfig.bundle.menu_bar.title ?? '',
                background_color: apiAppConfig.bundle.menu_bar.top_bg_color,
                items: apiAppConfig.bundle.menu_bar.items
                  .map(convertMenuBarItemToSimplified)
                  .filter((item): item is MenuItem => item !== null),
                show_shortcuts: {
                  apps: apiAppConfig.bundle.menu_bar.shortcuts.apps,
                  favorites: apiAppConfig.bundle.menu_bar.shortcuts.favorites,
                  settings: apiAppConfig.bundle.menu_bar.shortcuts.settings,
                },
              } : null,
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'typeGuardNotAllowed', line: 8 }");
}

#[test]
fn asserts_predicates_and_ambient_signatures_are_ignored() {
    valid(
        RULE,
        indoc! {"
            function assertString(value: unknown): asserts value is string {}
            declare function isString(value: unknown): value is string
            type Guard = (value: unknown) => value is string
            interface Checker { isString(value: unknown): value is string }
        "},
    );
}

#[test]
fn object_literal_methods_and_function_expressions_are_ignored() {
    valid(
        RULE,
        indoc! {"
            const obj = { isString(value: unknown): value is string { return true } }
            const fn = function (value: unknown): value is string { return true }
        "},
    );
}
