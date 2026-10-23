mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-non-camel-case-functions";

#[test]
fn camel_case_function_declarations() {
    valid(
        RULE,
        indoc! {"
            function camelCase() {}
            function anotherCamelCase() {}
            function validateUserInput() {}
        "},
    );
}

#[test]
fn arrow_functions_are_not_checked() {
    valid(
        RULE,
        indoc! {"
            const PascalCase = () => {}
            const snake_case = () => {}
        "},
    );
}

#[test]
fn function_expressions_are_not_checked() {
    valid(
        RULE,
        indoc! {"
            const myFunc = function PascalCase() {}
            const myOtherFunc = function snake_case() {}
        "},
    );
}

#[test]
fn function_with_jsx_element_return_type_is_ignored() {
    valid(
        RULE,
        indoc! {"
            function PascalCase(): JSX.Element {}
            function snake_case(): JSX.Element | null {}
        "},
    );
}

#[test]
fn pascal_case_function_declaration() {
    let result = invalid(RULE, "function PascalCase() {}");
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'nonCamelCaseFunction'
      msg: 'Function name "PascalCase" should be in camelCase format. If this fn is a React component and can't use the `FC` type, add a explicit "JSX.Element" return type to it'
      line: 1
    "#);
}

#[test]
fn snake_case_function_declaration() {
    let result = invalid(RULE, "function snake_case() {}");
    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'nonCamelCaseFunction', line: 1, column: 10, endLine: 1, endColumn: 20 }");
}

#[test]
fn uppercase_function_declaration() {
    let result = invalid(RULE, "function UPPERCASE() {}");
    assert_snapshot!(result.errors(), @"- { messageId: 'nonCamelCaseFunction', line: 1 }");
}

#[test]
fn dollar_prefixed_functions_are_allowed() {
    valid(RULE, "function $camelCase() {}");
}

#[test]
fn invalid_dollar_prefixed_functions() {
    let result = invalid(RULE, "function $UPPERCASE() {}");
    assert_snapshot!(result.errors(), @"- { messageId: 'nonCamelCaseFunction', line: 1 }");
}

#[test]
fn overloads_and_ambient_declarations_are_not_checked() {
    valid(
        RULE,
        indoc! {"
            declare function Ambient(): void
            export function overloaded(a: string): void
            export function overloaded(a: number): void
            export function overloaded(a: unknown) {}
        "},
    );
}

#[test]
fn parenthesized_jsx_element_return_types_are_ignored() {
    valid(
        RULE,
        indoc! {"
            function PascalCase(): (JSX.Element) {}
            function Other(): (JSX.Element) | null {}
        "},
    );
}
