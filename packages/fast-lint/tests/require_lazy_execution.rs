mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "require-lazy-execution";

#[test]
fn allows_function_call_inside_function_declaration() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function foo() {
              dangerousFunction()
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_arrow_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const foo = () => {
              dangerousFunction()
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_method() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            class Foo {
              bar() {
                dangerousFunction()
              }
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_getter() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            class Foo {
              get bar() {
                return dangerousFunction()
              }
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_class_property_arrow_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            class Foo {
              bar = () => {
                dangerousFunction()
              }
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_callback() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            setTimeout(() => {
              dangerousFunction()
            }, 1000)
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_promise_callback() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            promise.then(() => {
              dangerousFunction()
            })
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_nested_functions() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function outer() {
              function inner() {
                dangerousFunction()
              }
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_tagged_template_inside_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function query() {
              return sql`SELECT * FROM users`
            }
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_tagged_template_inside_arrow_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const query = () => sql`SELECT * FROM users`
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_tagged_template_inside_method() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            class DB {
              query() {
                return sql`SELECT * FROM users`
              }
            }
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_tagged_template_inside_callback() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const queries = items.map(item => sql`SELECT * FROM ${item}`)
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_member_expression_tagged_template_inside_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function query() {
              return db.sql`SELECT * FROM users`
            }
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_in_function_expression() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const foo = function() {
              dangerousFunction()
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_member_expression_call_inside_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function foo() {
              fs.readFileSync('file.txt')
            }
        "#},
            options: Some(json!([{"functions":["readFileSync"]}])),
            filename: None,
        },
    );
}

#[test]
fn disallows_function_call_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_member_expression_call_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            fs.readFileSync('file.txt')
        "#},
            options: Some(json!([{"functions":["readFileSync"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_tagged_template_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const query = sql`SELECT * FROM users`
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn allows_multiple_function_names_in_config() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function foo() {
              dangerousFunction()
              anotherDangerous()
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction","anotherDangerous"]}])),
            filename: None,
        },
    );
}

#[test]
fn disallows_multiple_function_names_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            dangerousFunction()
            anotherDangerous()
        "#},
            options: Some(json!([{"functions":["dangerousFunction","anotherDangerous"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 2 }
    "#);
}

/// The ESQuery `selectors` option is not supported (there is no selector engine): a config using it is rejected and
/// the selector's function goes in `functions`.
fn selectors_option_error() -> String {
    let options = json!([{"selectors":["CallExpression[callee.name=\"readFileSync\"]"]}]);
    match fast_lint::rules::create_rule(RULE, Some(&options)) {
        Ok(_) => "accepted".to_string(),
        Err(error) => error,
    }
}

#[test]
fn allows_selector_matching_inside_function() {
    assert_snapshot!(selectors_option_error(), @"invalid options for rule require-lazy-execution: the `selectors` option is not supported, list the function names in `functions` instead");
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function foo() {
              readFileSync('file.txt')
            }
        "#},
            options: Some(json!([{"functions":["readFileSync"]}])),
            filename: None,
        },
    );
}

#[test]
fn disallows_selector_matching_at_module_level() {
    assert_snapshot!(selectors_option_error(), @"invalid options for rule require-lazy-execution: the `selectors` option is not supported, list the function names in `functions` instead");
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            readFileSync('file.txt')
        "#},
            options: Some(json!([{"functions":["readFileSync"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn ignores_non_configured_function_calls_at_module_level() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            someOtherFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_object_method() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const obj = {
              method() {
                dangerousFunction()
              }
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_function_call_inside_object_method_shorthand() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const obj = {
              method: () => {
                dangerousFunction()
              }
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn works_with_no_options_provided() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            dangerousFunction()
        "#},
            options: Some(json!([{}])),
            filename: None,
        },
    );
}

#[test]
fn allows_call_in_iife() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            (function() {
              dangerousFunction()
            })()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn allows_call_in_arrow_iife() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            (() => {
              dangerousFunction()
            })()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn disallows_call_in_variable_initializer_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn allows_call_in_arrow_function_variable_initializer() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const fn = () => dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
}

#[test]
fn disallows_call_in_class_property_initializer() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            class Foo {
              prop = dangerousFunction()
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 2 }");
}

#[test]
fn disallows_call_in_static_class_property() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            class Foo {
              static prop = dangerousFunction()
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 2 }");
}

#[test]
fn disallows_call_in_array_literal_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const arr = [dangerousFunction(), 2, 3]
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_object_literal_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const obj = {
              key: dangerousFunction()
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 2 }");
}

#[test]
fn disallows_call_in_ternary_expression_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const value = condition ? dangerousFunction() : null
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_logical_expression_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const value = someThing && dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_template_literal_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const str = `Result: ${dangerousFunction()}`
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_multiple_calls_on_same_line() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const a = dangerousFunction(), b = dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 1 }
    "#);
}

#[test]
fn disallows_chained_calls_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = dangerousFunction().then(x => x)
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_export_statement() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            export const value = dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_default_export() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            export default dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_as_function_argument_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            console.log(dangerousFunction())
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_destructuring_pattern_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const { data } = dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_array_destructuring_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const [first] = dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_with_spread_operator_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const arr = [...dangerousFunction()]
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_await_expression_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const value = await dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_nested_member_expression_call_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            obj.nested.readFileSync('file.txt')
        "#},
            options: Some(json!([{"functions":["readFileSync"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_switch_statement_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            switch (dangerousFunction()) {
              case 1: break
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_if_statement_condition_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            if (dangerousFunction()) {
              console.log('test')
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_for_loop_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            for (const item of dangerousFunction()) {
              console.log(item)
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_while_loop_condition_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            while (dangerousFunction()) {
              break
            }
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_throw_statement_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            throw dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_call_in_return_statement_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            return dangerousFunction()
        "#},
            options: Some(json!([{"functions":["dangerousFunction"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_member_expression_tagged_template_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const query = db.sql`SELECT * FROM users`
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_tagged_template_in_array_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const queries = [sql`SELECT * FROM users`, sql`SELECT * FROM posts`]
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 1 }
    "#);
}

#[test]
fn disallows_tagged_template_in_object_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const obj = {
              query: sql`SELECT * FROM users`
            }
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 2 }");
}

#[test]
fn disallows_tagged_template_as_function_argument_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            execute(sql`SELECT * FROM users`)
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_tagged_template_in_export_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            export const query = sql`SELECT * FROM users`
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_tagged_template_in_ternary_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const query = condition ? sql`SELECT * FROM users` : null
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_tagged_template_with_complex_expression_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const query = sql`
              SELECT * FROM users
              WHERE id = ${userId}
              AND name = ${userName}
            `
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn disallows_nested_tagged_template_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const html = html`<div>${css`color: red;`}</div>`
        "#},
            options: Some(json!([{"functions":["html","css"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 1 }
    "#);
}

#[test]
fn disallows_tagged_template_in_class_property_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            class Query {
              query = sql`SELECT * FROM users`
            }
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 2 }");
}

#[test]
fn disallows_tagged_template_chained_with_method_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = sql`SELECT * FROM users`.execute()
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'moduleLevel', line: 1 }");
}

#[test]
fn methods_accessors_and_constructors_are_function_contexts() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            class A {
              constructor(value = sql`a`) {
                sql`b`
              }
              method() {
                return sql`c`
              }
              get value() {
                return sql`d`
              }
            }
            const object = {
              method() {
                return sql`e`
              },
            }
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
}

#[test]
fn computed_method_keys_run_at_module_level() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            class A {
              [sql`key`]() {}
            }
            const parenthesized = (sql)`x`
            const element = db[sql]()
        "#},
            options: Some(json!([{"functions":["sql"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'moduleLevel', line: 2 }
    - { messageId: 'moduleLevel', line: 4 }
    - { messageId: 'moduleLevel', line: 5 }
    "#);
}
