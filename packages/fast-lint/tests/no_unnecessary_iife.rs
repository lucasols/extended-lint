mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-unnecessary-iife";

#[test]
fn valid_iife_with_multiple_statements() {
    valid(
        RULE,
        indoc! {r#"
        const x = (() => {
          const y = 1;
          return y + 1;
        })();
    "#},
    );
}

#[test]
fn valid_iife_with_variable_declaration_and_return() {
    valid(
        RULE,
        indoc! {r#"
        const x = (() => {
          const temp = getValue();
          return temp.map(t => t.id);
        })();
    "#},
    );
}

#[test]
fn valid_iife_without_return_side_effects_only() {
    valid(
        RULE,
        indoc! {r#"
        (() => {
          console.log('init');
        })();
    "#},
    );
}

#[test]
fn valid_regular_function_not_iife() {
    valid(
        RULE,
        indoc! {r#"
        const fn = () => { return 1; };
    "#},
    );
}

#[test]
fn valid_iife_with_arguments() {
    valid(
        RULE,
        indoc! {r#"
        const x = ((a) => a + 1)(5);
    "#},
    );
}

#[test]
fn valid_iife_with_multiple_arguments() {
    valid(
        RULE,
        indoc! {r#"
        const x = ((a, b) => a + b)(1, 2);
    "#},
    );
}

#[test]
fn valid_async_iife_with_await_statements() {
    valid(
        RULE,
        indoc! {r#"
        const x = (async () => {
          const data = await fetch('/api');
          return data.json();
        })();
    "#},
    );
}

#[test]
fn valid_iife_with_try_catch() {
    valid(
        RULE,
        indoc! {r#"
        const x = (() => {
          try {
            return getValue();
          } catch {
            return null;
          }
        })();
    "#},
    );
}

#[test]
fn invalid_arrow_iife_with_single_return_statement() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (() => {
          return members.map(m => m.id);
        })();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = members.map(m => m.id);");
}

#[test]
fn invalid_arrow_iife_with_expression_body() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (() => members.map(m => m.id))();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = members.map(m => m.id);");
}

#[test]
fn invalid_regular_function_iife_with_single_return() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (function() {
          return getValue();
        })();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = getValue();");
}

#[test]
fn invalid_named_function_iife_with_single_return() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (function myFunc() {
          return getValue();
        })();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = getValue();");
}

#[test]
fn invalid_iife_returning_object_literal() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (() => {
          return { a: 1, b: 2 };
        })();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = { a: 1, b: 2 };");
}

#[test]
fn invalid_iife_returning_simple_value() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (() => {
          return 42;
        })();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = 42;");
}

#[test]
fn invalid_iife_returning_void() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (() => {
          return;
        })();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = undefined;");
}

#[test]
fn invalid_arrow_iife_with_expression_body_returning_object() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const x = (() => ({ a: 1 }))();
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 1 }
    "#);
    assert_snapshot!(result.output, @"const x = { a: 1 };");
}

#[test]
fn invalid_whole_function_body_is_an_iife() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function foo() {
          return (() => {
            const bar = 42;

            console.log(bar);

            return bar;
          })();
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    function foo() {
        const bar = 42;

        console.log(bar);

        return bar;
      }
    "#);
}

#[test]
fn invalid_whole_arrow_function_body_is_an_iife() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const foo = () => {
          return (() => {
            const bar = 42;

            console.log(bar);

            return bar;
          })();
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'unnecessaryIife', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const foo = () => {
        const bar = 42;

        console.log(bar);

        return bar;
      }
    "#);
}

#[test]
fn valid_arrow_function_body_returns_an_iife_but_there_are_other_statements_before_it() {
    valid(
        RULE,
        indoc! {r#"
        const foo = () => {
          console.log('test');

          return (() => {
            const bar = 42;

            console.log(bar);

            return bar;
          })();
        }

        function bar() {
          console.log('test');

          return (() => {
            const baz = 42;

            console.log(baz);

            return baz;
          })();
        }
    "#},
    );
}

#[test]
fn whole_method_body_is_an_iife() {
    let result = invalid(
        RULE,
        indoc! {r#"
        class A {
          method() {
            return (() => {
              const bar = 42;
              return bar;
            })();
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryIife', line: 3 }");
    assert_snapshot!(result.output, @r###"
    class A {
      method() {
          const bar = 42;
          return bar;
        }
    }
    "###);
}

#[test]
fn iifes_with_comments_or_async_are_kept() {
    valid(
        RULE,
        indoc! {r#"
        const a = (() => {
          // the value
          return 1;
        })();
        const b = (async () => {
          return 1;
        })();
        function c() {
          return (() => {
            /* setup */
            const value = 1;
            return value;
          })();
        }
    "#},
    );
}

#[test]
fn iife_statement_returning_an_object_stays_an_expression() {
    let result = invalid(
        RULE,
        indoc! {r#"
        (() => ({ a: 1 }))();
    "#},
    );
    assert_snapshot!(result.output, @"({ a: 1 });");
}
