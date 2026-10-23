mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-restricted-comments";

fn case(code: &str, options: serde_json::Value) -> Case<'_> {
    Case {
        code,
        options: Some(options),
        filename: None,
    }
}

#[test]
fn no_patterns_configured() {
    valid(
        RULE,
        case(
            indoc! {r#"
            // This is a comment
            const x = 1
        "#},
            json!([{"patterns": []}]),
        ),
    );
}

#[test]
fn comment_does_not_match_any_pattern() {
    valid(
        RULE,
        case(
            indoc! {r#"
            // This is a regular comment
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "TODO:"}]}]),
        ),
    );
}

#[test]
fn line_comment_matches_includes_pattern() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // TODO: fix this later
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "TODO:"}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'restrictedComment', line: 1 }");
    assert_snapshot!(result.suggestion_output(0), @"

    const x = 1
    ");
}

#[test]
fn line_comment_matches_includes_pattern_with_custom_message() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // FIXME: broken code
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "FIXME:", "message": "FIXMEs must be resolved before merge"}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'restrictedCommentWithMessage', line: 1 }");
}

#[test]
fn line_comment_matches_regex_pattern() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // eslint-disable-next-line
            const x = 1
        "#},
            json!([{"patterns": [{"regex": "^\\s*eslint-disable"}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'restrictedComment', line: 1 }");
}

#[test]
fn block_comment_matches_includes_pattern() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            /* TODO: implement this */
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "TODO:"}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'restrictedComment', line: 1 }");
    assert_snapshot!(result.suggestion_output(0), @"

    const x = 1
    ");
}

#[test]
fn multiple_patterns_only_one_matches() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // HACK: workaround for bug
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "TODO:"}, {"includes": "FIXME:"}, {"includes": "HACK:"}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'restrictedComment', line: 1 }");
}

#[test]
fn multiple_comments_match_patterns() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // TODO: first task
            const x = 1
            // FIXME: second issue
            const y = 2
        "#},
            json!([{"patterns": [{"includes": "TODO:"}, {"includes": "FIXME:"}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'restrictedComment', line: 1 }
    - { messageId: 'restrictedComment', line: 3 }
    "#);
}

#[test]
fn regex_pattern_does_not_match() {
    valid(
        RULE,
        case(
            indoc! {r#"
            // eslint is great
            const x = 1
        "#},
            json!([{"patterns": [{"regex": "^\\s*eslint-disable"}]}]),
        ),
    );
}

#[test]
fn auto_fix_removes_comment_automatically() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // TODO: fix this
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "TODO:", "autoFix": true}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'restrictedComment', line: 1 }");
    assert_snapshot!(result.output, @"

    const x = 1
    ");
}

#[test]
fn auto_fix_with_custom_message() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // FIXME: broken
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "FIXME:", "message": "Fix this before merge", "autoFix": true}]}]),
        ),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'restrictedCommentWithMessage', line: 1 }");
    assert_snapshot!(result.output, @"

    const x = 1
    ");
}

#[test]
fn messages() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // TODO: first task
            /* FIXME: second issue */
            const x = 1 // eslint-disable-line
        "#},
            json!([{"patterns": [
                {"includes": "TODO:"},
                {"includes": "FIXME:", "message": "FIXMEs must be resolved before merge"},
                {"regex": "^\\s*eslint-disable"}
            ]}]),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'restrictedComment'
      msg: 'Comment matches restricted pattern 'TODO:''
      line: 1
    - messageId: 'restrictedCommentWithMessage'
      msg: 'FIXMEs must be resolved before merge'
      line: 2
    - messageId: 'restrictedComment'
      msg: 'Comment matches restricted pattern '^\s*eslint-disable''
      line: 3
    "#);
}

#[test]
fn only_the_first_matching_pattern_is_reported() {
    let result = invalid(
        RULE,
        case(
            indoc! {r#"
            // TODO: FIXME: both
            const x = 1
        "#},
            json!([{"patterns": [{"includes": "FIXME:", "message": "first"}, {"includes": "TODO:"}]}]),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'restrictedCommentWithMessage'
      msg: 'first'
      line: 1
    "#);
}

#[test]
fn comment_like_text_is_not_a_comment() {
    valid(
        RULE,
        case(
            indoc! {r#"
            const url = 'https://example.com/TODO:'
            const regex = /TODO:/
            const element = <div>// TODO: not a comment</div>
        "#},
            json!([{"patterns": [{"includes": "TODO:"}]}]),
        ),
    );
}
