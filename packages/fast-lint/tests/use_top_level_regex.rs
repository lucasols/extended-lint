mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "use-top-level-regex";

#[test]
fn regex_at_top_level() {
    valid(
        RULE,
        indoc! {r#"
        const regex = /test/

        function foo() {
          return regex.test('hello')
        }
    "#},
    );
}

#[test]
fn regex_with_global_flag_in_function() {
    valid(
        RULE,
        indoc! {r#"
        function foo() {
          return /test/g.exec('hello')
        }
    "#},
    );
}

#[test]
fn regex_with_sticky_flag_in_function() {
    valid(
        RULE,
        indoc! {r#"
        function foo() {
          return /test/y.exec('hello')
        }
    "#},
    );
}

#[test]
fn regex_with_both_global_and_sticky_flags() {
    valid(
        RULE,
        indoc! {r#"
        function foo() {
          return /test/gy.exec('hello')
        }
    "#},
    );
}

#[test]
fn regex_in_regexp_constructor() {
    valid(
        RULE,
        indoc! {r#"
        function foo() {
          return new RegExp(/test/)
        }
    "#},
    );
}

#[test]
fn regex_in_module_level_variable_declaration() {
    valid(
        RULE,
        indoc! {r#"
        const pattern = /[a-z]+/i
    "#},
    );
}

#[test]
fn regex_in_export_statement() {
    valid(
        RULE,
        indoc! {r#"
        export const EMAIL_REGEX = /^[^@]+@[^@]+$/
    "#},
    );
}

#[test]
fn regex_in_class_property() {
    valid(
        RULE,
        indoc! {r#"
        class Validator {
          pattern = /test/
        }
    "#},
    );
}

#[test]
fn regex_in_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function foo(someString) {
          return /[a-z]*/.test(someString)
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_arrow_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const foo = (someString) => {
          return /[a-z]*/.test(someString)
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        class Test {
          validate(input) {
            return /test/.test(input)
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 3 }");
}

#[test]
fn regex_in_function_expression() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const validator = function(input) {
          return /^[a-z]+$/.test(input)
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_nested_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function outer() {
          function inner() {
            return /test/.test('hello')
          }
          return inner
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 3 }");
}

#[test]
fn regex_in_callback_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function processItems(items) {
          return items.filter(item => /valid/.test(item))
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn multiple_regex_in_same_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function validate(input) {
          const emailPattern = /^[^@]+@[^@]+$/
          const phonePattern = /\d{10}/
          return emailPattern.test(input) || phonePattern.test(input)
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'regexShouldBeTopLevel', line: 2 }
    - { messageId: 'regexShouldBeTopLevel', line: 3 }
    "#);
}

#[test]
fn regex_in_if_statement_inside_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function isValid(input) {
          if (/test/.test(input)) {
            return true
          }
          return false
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_return_statement() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function getPattern() {
          return /pattern/
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_object_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const config = {
          validate: function(input) {
            return /pattern/.test(input)
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 3 }");
}

#[test]
fn regex_in_object_arrow_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const handlers = {
          check: (str) => /test/.test(str)
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_top_level_object_literal() {
    valid(
        RULE,
        indoc! {r#"
        const patterns = {
          email: /^[^@]+@[^@]+$/,
          phone: /\d{10}/
        }
    "#},
    );
}

#[test]
fn regex_in_top_level_array() {
    valid(
        RULE,
        indoc! {r#"
        const testPatterns = [/test/]
    "#},
    );
}

#[test]
fn regex_in_nested_top_level_object() {
    valid(
        RULE,
        indoc! {r#"
        const config = [/test/, { regex: /test/ }]
    "#},
    );
}

#[test]
fn regex_in_function_inside_another_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function outer() {
          const config = {
            validate: function(input) {
              return /pattern/.test(input)
            }
          }
          return config
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 4 }");
}

#[test]
fn regex_in_shorthand_object_method_and_accessors() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const config = {
          validate(input) {
            return /pattern/.test(input)
          },
          get pattern() {
            return /getter/
          },
        }

        class Test {
          constructor() {
            this.pattern = /ctor/
          }
          static get pattern() {
            return /static/
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'regexShouldBeTopLevel', line: 3 }
    - { messageId: 'regexShouldBeTopLevel', line: 6 }
    - { messageId: 'regexShouldBeTopLevel', line: 12 }
    - { messageId: 'regexShouldBeTopLevel', line: 15 }
    "#);
}

#[test]
fn regex_in_default_parameter() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function match(input, pattern = /default/) {
          return pattern.test(input)
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 1 }");
}

#[test]
fn regex_in_parenthesized_regexp_constructor() {
    valid(
        RULE,
        indoc! {r#"
        function foo() {
          return new RegExp((/test/), 'i')
        }
    "#},
    );
}

#[test]
fn regex_in_regexp_call_without_new_is_reported() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function foo() {
          return RegExp(/test/)
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_object_method_computed_key_is_top_level() {
    valid(
        RULE,
        indoc! {r#"
        const config = {
          [/key/.source]() {
            return 1
          },
        }
    "#},
    );
}

#[test]
fn regex_in_class_method_computed_key_is_reported() {
    let result = invalid(
        RULE,
        indoc! {r#"
        class Test {
          [/key/.source]() {
            return 1
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'regexShouldBeTopLevel', line: 2 }");
}

#[test]
fn regex_in_class_static_block_and_property_initializer() {
    valid(
        RULE,
        indoc! {r#"
        class Test {
          static pattern = /static/
          static {
            const local = /block/
          }
        }
    "#},
    );
}
