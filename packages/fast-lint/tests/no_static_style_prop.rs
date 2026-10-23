mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-static-style-prop";

#[test]
fn variable_reference() {
    valid(
        RULE,
        indoc! {"
            const styles = { color: 'red' }
            function Component() {
              return <div style={styles}>Content</div>
            }
        "},
    );
}

#[test]
fn function_call() {
    valid(
        RULE,
        indoc! {"
            function getStyles() {
              return { color: 'blue' }
            }
            function Component() {
              return <div style={getStyles()}>Content</div>
            }
        "},
    );
}

#[test]
fn conditional_expression() {
    valid(
        RULE,
        indoc! {"
            function Component({ isDark }: { isDark: boolean }) {
              return <div style={{ color: isDark ? 'white' : 'black' }}>Content</div>
            }
        "},
    );
}

#[test]
fn template_literal_with_expression() {
    valid(
        RULE,
        indoc! {"
            function Component({ r, g, b }: { r: number; g: number; b: number }) {
              return <div style={{ color: `rgb(${r}, ${g}, ${b})` }}>Content</div>
            }
        "},
    );
}

#[test]
fn computed_property() {
    valid(
        RULE,
        indoc! {"
            function Component({ prop }: { prop: string }) {
              return <div style={{ [prop]: 'value' }}>Content</div>
            }
        "},
    );
}

#[test]
fn spread_operator() {
    valid(
        RULE,
        indoc! {"
            const baseStyles = { color: 'red' }
            function Component() {
              return <div style={{ ...baseStyles, fontSize: 16 }}>Content</div>
            }
        "},
    );
}

#[test]
fn function_call_in_property_value() {
    valid(
        RULE,
        indoc! {"
            function getColor() {
              return 'red'
            }
            function Component() {
              return <div style={{ color: getColor() }}>Content</div>
            }
        "},
    );
}

#[test]
fn variable_in_property_value() {
    valid(
        RULE,
        indoc! {"
            const color = 'red'
            function Component() {
              return <div style={{ color }}>Content</div>
            }
        "},
    );
}

#[test]
fn mixed_dynamic_and_static_properties() {
    valid(
        RULE,
        indoc! {"
            const color = 'red'
            function Component() {
              return <div style={{ color, fontSize: 16 }}>Content</div>
            }
        "},
    );
}

#[test]
fn no_style_prop() {
    valid(
        RULE,
        indoc! {r#"
            function Component() {
              return <div className="container">Content</div>
            }
        "#},
    );
}

#[test]
fn static_object_literal() {
    let result = invalid(
        RULE,
        indoc! {"
            function Component() {
              return <div style={{ color: 'red', fontSize: 16 }}>Content</div>
            }
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noStaticStyleProp'
      msg: 'Static style props are not allowed use css instead.'
      line: 2
    ");
}

#[test]
fn static_string_literal() {
    let result = invalid(
        RULE,
        indoc! {r#"
            function Component() {
              return <div style="color: red; font-size: 16px;">Content</div>
            }
        "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noStaticStyleProp', line: 2 }");
}

#[test]
fn empty_object_literal() {
    let result = invalid(
        RULE,
        indoc! {"
            function Component() {
              return <div style={{}}>Content</div>
            }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noStaticStyleProp', line: 2 }");
}

#[test]
fn nested_static_object() {
    let result = invalid(
        RULE,
        indoc! {"
            function Component() {
              return (
                <div>
                  <span style={{ color: 'blue' }}>Text</span>
                </div>
              )
            }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noStaticStyleProp', line: 4 }");
}

#[test]
fn custom_error_message() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                function Component() {
                  return <div style={{ color: 'red' }}>Content</div>
                }
            "},
            options: Some(json!({ "customMessage": "Use CSS classes instead" })),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noStaticStyleProp'
      msg: 'Static style props are not allowed use css instead. Use CSS classes instead'
      line: 2
    ");
}

#[test]
fn empty_custom_message() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                function Component() {
                  return <div style={{ color: 'red' }}>Content</div>
                }
            "},
            options: Some(json!({ "customMessage": "" })),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noStaticStyleProp'
      msg: 'Static style props are not allowed use css instead.'
      line: 2
    ");
}

#[test]
fn static_arrays_and_templates() {
    let result = invalid(
        RULE,
        indoc! {"
            function Component() {
              return (
                <>
                  <div style={{ margin: [1, , 2], font: `x` }} />
                  <div style={({ color: 'red' })} />
                  <div style={{ margin: [...sizes] }} />
                  <div style={{ m() { return 1 } }} />
                </>
              )
            }
        "},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noStaticStyleProp', line: 4 }
    - { messageId: 'noStaticStyleProp', line: 5 }
    ");
}
