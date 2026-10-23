mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-leaked-text-in-jsx";

fn leaked(text: &str) -> String {
    let code = format!("<div>\n  <p>Hello</p>\n  {text}\n</div>\n");
    invalid(RULE, &code).errors_with_msg()
}

#[test]
fn no_leaked_text_in_jsx() {
    valid(RULE, "<div>Hello</div>");
}

#[test]
fn leaked_comma_text_in_jsx() {
    assert_snapshot!(leaked(","), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text "," should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn leaked_semicolon_text_in_jsx() {
    assert_snapshot!(leaked(";"), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text ";" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn leaked_open_bracket_text_in_jsx() {
    assert_snapshot!(leaked("["), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text "[" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn leaked_open_paren_text_in_jsx() {
    assert_snapshot!(leaked("("), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text "(" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn leaked_close_paren_text_in_jsx() {
    assert_snapshot!(leaked(")"), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text ")" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn no_leaked_text_in_jsx_2() {
    valid(
        RULE,
        indoc! {r#"
            <div>
              <p>Hello</p>
              {","}
            </div>
        "#},
    );
}

#[test]
fn no_leaked_text_in_jsx_3() {
    valid(
        RULE,
        indoc! {"
            <div>
              <p>Hello</p>
              {'[]'}
            </div>
        "},
    );
}

#[test]
fn leaked_and_text_in_jsx() {
    let result = invalid(
        RULE,
        indoc! {"
            <div>
              <p>Hello</p>
              test && (
              <div>
                <p>Hello</p>
              </div>
            </div>
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text "&&" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn leaked_or_text_in_jsx() {
    let result = invalid(
        RULE,
        indoc! {"
            <div>
              <p>Hello</p>
              test || (
              <div>
                <p>Hello</p>
              </div>
            </div>
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text "||" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn leaked_ternary_text_in_jsx() {
    let result = invalid(
        RULE,
        indoc! {"
            <div>
              <p>Hello</p>
              test ? (
              <div>
                <p>Hello</p>
              </div>
            </div>
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text "? (" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn leaked_close_bracket_text_in_jsx() {
    assert_snapshot!(leaked("]"), @r#"
    - messageId: 'leakedTextInJSX'
      msg: 'Text "]" should be wrapped in a JSX expression container.'
      line: 2
    "#);
}

#[test]
fn report_covers_the_whole_text_node() {
    let result = invalid(RULE, "<div>\n  <p>Hello</p>\n  ,\n</div>\n");
    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'leakedTextInJSX', line: 2, column: 15, endLine: 4, endColumn: 1 }");
}
