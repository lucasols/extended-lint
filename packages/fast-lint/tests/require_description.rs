mod common;

use common::{invalid, valid, Case, LintResult};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "require-description";

/// The start line and column of each diagnostic, like the ESLint tests' `getErrorsFromResult(result, { column: true })`.
fn errors_with_column(result: &LintResult) -> String {
    result
        .diagnostics
        .iter()
        .map(|d| {
            format!(
                "- {{ messageId: '{}', line: {}, column: {} }}\n",
                d.message_id, d.line, d.column
            )
        })
        .collect()
}

#[test]
fn eslint_and_oxlint_directives_with_descriptions() {
    valid(
        RULE,
        indoc! {r#"
        // eslint-disable-next-line no-console -- logs the dev tools output
        console.log(1)
        // oxlint-disable-next-line no-console -- logs the dev tools output
        console.log(2)
        console.log(3) // oxlint-disable-line no-console -- logs the dev tools output
        /* oxlint-disable no-console -- dev script */
        /* oxlint-enable no-console -- dev script */
    "#},
    );
}

#[test]
fn comments_that_are_not_directives() {
    valid(
        RULE,
        indoc! {r#"
        // oxlint is used to lint the project
        // the oxlint-disable comments are checked
        const value = 1
    "#},
    );
}

#[test]
fn eslint_and_oxlint_directives_without_descriptions() {
    let result = invalid(
        RULE,
        indoc! {r#"
        // eslint-disable-next-line no-console
        console.log(1)
        // oxlint-disable-next-line no-console
        console.log(2)
        console.log(3) // oxlint-disable-line no-console
        /* oxlint-disable no-console */
        /* oxlint-enable no-console */
        /* oxlint-disable */
    "#},
    );
    assert_snapshot!(errors_with_column(&result), @r#"
    - { messageId: 'missingDescription', line: 1, column: 1 }
    - { messageId: 'missingDescription', line: 3, column: 1 }
    - { messageId: 'missingDescription', line: 5, column: 1 }
    - { messageId: 'missingDescription', line: 6, column: 1 }
    - { messageId: 'missingDescription', line: 7, column: 1 }
    - { messageId: 'missingDescription', line: 8, column: 1 }
    "#);
}

#[test]
fn ignored_oxlint_directive_kinds() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            // oxlint-disable-next-line no-console
            console.log(1)
            /* oxlint-disable no-console */
        "#},
            options: Some(json!([{"ignore":["oxlint-disable-next-line","oxlint-disable"]}])),
            filename: None,
        },
    );
}

#[test]
fn fast_lint_directives() {
    let result = invalid(
        RULE,
        indoc! {r#"
        // fast-lint-disable-next-line strict-casting
        const a = 1
        const b = 2 // fast-lint-disable-line strict-casting -- reason
        /* fast-lint-disable strict-casting */
        /* fast-lint-enable strict-casting -- reason */
        // fast-lint-disable strict-casting
    "#},
    );
    assert_snapshot!(errors_with_column(&result), @r#"
    - { messageId: 'missingDescription', line: 1, column: 1 }
    - { messageId: 'missingDescription', line: 4, column: 1 }
    "#);
}

#[test]
fn other_directives_and_empty_descriptions() {
    let result = invalid(
        RULE,
        indoc! {r#"
        /* eslint no-console: off */
        /* global foo */
        /* exported bar -- used by the page */
        // eslint-disable-line no-console --  x
        // eslint-disable-line no-console --  -- x
        /* eslint-disable-next-line
           no-console */
        const value = `/* eslint-disable */`
    "#},
    );
    assert_snapshot!(errors_with_column(&result), @r#"
    - { messageId: 'missingDescription', line: 1, column: 1 }
    - { messageId: 'missingDescription', line: 2, column: 1 }
    - { messageId: 'missingDescription', line: 5, column: 1 }
    "#);
}

#[test]
fn ignored_fast_lint_directive_kinds() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            // fast-lint-disable-next-line strict-casting
            const a = 1
        "#},
            options: Some(json!([{ "ignore": ["fast-lint-disable-next-line"] }])),
            filename: None,
        },
    );
}
