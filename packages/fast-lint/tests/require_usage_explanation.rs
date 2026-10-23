mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "require-usage-explanation";

fn case(code: &str, matches: serde_json::Value) -> Case<'_> {
    Case {
        code,
        options: Some(json!([{ "matches": matches }])),
        filename: None,
    }
}

fn workaround(code: &str) -> Case<'_> {
    case(
        code,
        json!([{ "fn": "dangerousFn", "commentPrefix": "WORKAROUND:" }]),
    )
}

#[test]
fn valid_fn_call_with_line_comment_prefix_above() {
    valid(
        RULE,
        workaround(indoc! {r#"
            // WORKAROUND: legacy API requires this call
            dangerousFn()
        "#}),
    );
}

#[test]
fn valid_fn_call_with_block_comment_prefix_above() {
    valid(
        RULE,
        workaround(indoc! {r#"
            /* WORKAROUND: legacy API requires this call */
            dangerousFn()
        "#}),
    );
}

#[test]
fn valid_fn_call_inside_variable_declaration_with_comment_above() {
    valid(
        RULE,
        workaround(indoc! {r#"
            // WORKAROUND: needed for backwards compatibility
            const result = dangerousFn()
        "#}),
    );
}

#[test]
fn valid_no_matching_function_call() {
    valid(
        RULE,
        workaround(indoc! {r#"
            otherFn()
        "#}),
    );
}

#[test]
fn valid_selector_match_with_correct_comment_above() {
    valid(
        RULE,
        case(
            indoc! {r#"
                // CAST: needed for external library compatibility
                const x = value as string
            "#},
            json!([{ "selector": "TSAsExpression", "commentPrefix": "CAST:" }]),
        ),
    );
}

#[test]
fn valid_member_expression_fn_call_with_comment_above() {
    valid(
        RULE,
        workaround(indoc! {r#"
            // WORKAROUND: required by the SDK
            obj.dangerousFn()
        "#}),
    );
}

#[test]
fn invalid_fn_call_with_no_comment() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            dangerousFn()
        "#}),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'default'
      msg: 'Add a comment with prefix "WORKAROUND:" to explain why "dangerousFn" is being used.'
      line: 1
    "#);
}

#[test]
fn invalid_fn_call_with_wrong_prefix() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            // NOTE: some note
            dangerousFn()
        "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 2 }");
}

#[test]
fn invalid_fn_call_with_prefix_only_no_explanation_text() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            // WORKAROUND:
            dangerousFn()
        "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 2 }");
}

#[test]
fn invalid_selector_match_with_no_comment() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
                const x = value as string
            "#},
            json!([{ "selector": "TSAsExpression", "commentPrefix": "CAST:" }]),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'default'
      msg: 'Add a comment with prefix "CAST:" to explain why "TSAsExpression" is being used.'
      line: 1
    "#);
}

#[test]
fn invalid_multiple_configs_one_missing_comment() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
                // WORKAROUND: needed
                dangerousFn()
                unsafeFn()
            "#},
            json!([
                { "fn": "dangerousFn", "commentPrefix": "WORKAROUND:" },
                { "fn": "unsafeFn", "commentPrefix": "HACK:" },
            ]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 3 }");
}

#[test]
fn invalid_member_expression_fn_call_with_no_comment() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            obj.dangerousFn()
        "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 1 }");
}

#[test]
fn invalid_fn_call_with_custom_message_appended() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
                dangerousFn()
            "#},
            json!([{
                "fn": "dangerousFn",
                "commentPrefix": "WORKAROUND:",
                "message": "This function has security implications.",
            }]),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'default'
      msg: 'Add a comment with prefix "WORKAROUND:" to explain why "dangerousFn" is being used. This function has security implications.'
      line: 1
    "#);
}

#[test]
fn valid_comment_above_return_statement_containing_fn_call() {
    valid(
        RULE,
        workaround(indoc! {r#"
            function foo() {
              // WORKAROUND: needed for edge case
              return dangerousFn()
            }
        "#}),
    );
}

#[test]
fn valid_comment_inside_ternary_above_fn_call() {
    valid(
        RULE,
        workaround(indoc! {r#"
            const data = selector
              ? selector(state.data)
              : (
                  // WORKAROUND: Without a selector the hook returns full state
                  dangerousFn(state.data)
                )
        "#}),
    );
}

#[test]
fn valid_comment_inside_ternary_above_fn_call_2() {
    valid(
        RULE,
        workaround(indoc! {r#"
            const data = selector
              ? selector(state.data)
              : // WORKAROUND: Without a selector the hook returns full state
              dangerousFn(state.data)

        "#}),
    );
}

#[test]
fn valid_inline_comment_in_ternary_without_parens_above_fn_call() {
    valid(
        RULE,
        workaround(indoc! {r#"
            const data = selector
              ? selector(state.data)
              : // WORKAROUND: Without a selector the hook returns full state
                dangerousFn(state.data)
        "#}),
    );
}

#[test]
fn invalid_fn_call_inside_ternary_with_no_comment() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            const data = selector
              ? selector(state.data)
              : dangerousFn(state.data)
        "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 3 }");
}

#[test]
fn valid_comment_inside_array_above_fn_call() {
    valid(
        RULE,
        workaround(indoc! {r#"
            const items = [
              // WORKAROUND: needed for legacy support
              dangerousFn(),
              otherFn(),
            ]
        "#}),
    );
}

#[test]
fn valid_comment_inside_function_argument_above_fn_call() {
    valid(
        RULE,
        workaround(indoc! {r#"
            foo(
              // WORKAROUND: needed for legacy support
              dangerousFn()
            )
        "#}),
    );
}

#[test]
fn invalid_fn_call_inside_return_with_no_comment() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            function foo() {
              return dangerousFn()
            }
        "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 2 }");
}

#[test]
fn valid_comment_before_an_outer_expression_part() {
    valid(
        RULE,
        workaround(indoc! {r#"
            const value =
              // WORKAROUND: the chained call needs the legacy client
              client.dangerousFn().then(done)
        "#}),
    );
}

#[test]
fn invalid_comment_in_an_enclosing_function_does_not_count() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            // WORKAROUND: this explains the function, not the call inside it
            function foo() {
              dangerousFn()
            }
        "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'default', line: 3 }");
}

#[test]
fn invalid_optional_call_and_switch_case() {
    let result = invalid(
        RULE,
        workaround(indoc! {r#"
            obj?.dangerousFn()

            switch (kind) {
              case 1:
                // WORKAROUND: explained
                dangerousFn()
                break
              case 2:
                dangerousFn()
            }
        "#}),
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'default', line: 1 }
    - { messageId: 'default', line: 9 }
    "#);
}

#[test]
fn invalid_selector_option_is_validated() {
    let result = std::panic::catch_unwind(|| {
        valid(
            RULE,
            case(
                "a",
                json!([{ "selector": "TSAsExpression[", "commentPrefix": "CAST:" }]),
            ),
        );
    });
    assert!(result.is_err());
}
