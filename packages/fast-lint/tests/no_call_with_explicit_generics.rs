mod common;

use common::{invalid, valid, Case};
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-call-with-explicit-generics";

fn case(code: &str, options: serde_json::Value) -> Case<'_> {
    Case {
        code,
        options: Some(options),
        filename: None,
    }
}

#[test]
fn inferred_generics_are_valid() {
    valid(
        RULE,
        case(
            "test('user/update', { name });",
            json!({ "functions": ["test"] }),
        ),
    );
}

#[test]
fn non_configured_function_is_valid() {
    valid(
        RULE,
        case(
            "otherFunction<Type>('user/update', { name });",
            json!({ "functions": ["test"] }),
        ),
    );
}

#[test]
fn method_call_with_inferred_generics_is_valid() {
    valid(
        RULE,
        case(
            "namespace.method('user/update', { name });",
            json!({ "functions": ["namespace.method"] }),
        ),
    );
}

#[test]
fn non_configured_method_call_is_valid() {
    valid(
        RULE,
        case(
            "namespace.other<Type>('user/update', { name });",
            json!({ "functions": ["namespace.method"] }),
        ),
    );
}

#[test]
fn explicit_generics_on_function() {
    let result = invalid(
        RULE,
        case(
            "test<Type>('user/update', { name });",
            json!({ "functions": ["test"] }),
        ),
    );

    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'noExplicitGenerics', line: 1, column: 1, endLine: 1, endColumn: 36 }");
}

#[test]
fn multiple_explicit_generics_on_function() {
    let result = invalid(
        RULE,
        case(
            "test<Type1, Type2>('user/update', { name });",
            json!({ "functions": ["test"] }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noExplicitGenerics', line: 1 }");
}

#[test]
fn explicit_generics_on_method_call() {
    let result = invalid(
        RULE,
        case(
            "namespace.method<Type>('user/update', { name });",
            json!({ "functions": ["namespace.method"] }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noExplicitGenerics', line: 1 }");
}

#[test]
fn custom_message_for_function() {
    let result = invalid(
        RULE,
        case(
            "test<Type>('user/update', { name });",
            json!({ "functions": [{ "name": "test", "message": "Use inferred types for test()" }] }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noExplicitGenericsCustom'
      msg: 'Use inferred types for test()'
      line: 1
    ");
}

#[test]
fn custom_message_for_method_call() {
    let result = invalid(
        RULE,
        case(
            "api.fetch<Response>('/users');",
            json!({
                "functions": [{
                    "name": "api.fetch",
                    "message": "Let api.fetch infer the response type from the endpoint",
                }],
            }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noExplicitGenericsCustom'
      msg: 'Let api.fetch infer the response type from the endpoint'
      line: 1
    ");
}

#[test]
fn wildcard_matches_any_namespace() {
    let result = invalid(
        RULE,
        case(
            "foo.method<Type>('a');\nbar.method<Type>('b');",
            json!({ "functions": ["*.method"] }),
        ),
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noExplicitGenerics', line: 1 }
    - { messageId: 'noExplicitGenerics', line: 2 }
    ");
}

#[test]
fn wildcard_does_not_match_plain_function_call() {
    valid(
        RULE,
        case("method<Type>('a');", json!({ "functions": ["*.method"] })),
    );
}

#[test]
fn wildcard_with_inferred_generics_is_valid() {
    valid(
        RULE,
        case("foo.method('a');", json!({ "functions": ["*.method"] })),
    );
}

#[test]
fn wildcard_with_custom_message() {
    let result = invalid(
        RULE,
        case(
            "any.fetch<Response>('/users');",
            json!({
                "functions": [{ "name": "*.fetch", "message": "Do not pass explicit generics to fetch" }],
            }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noExplicitGenericsCustom'
      msg: 'Do not pass explicit generics to fetch'
      line: 1
    ");
}

#[test]
fn mix_of_string_and_object_function_configs() {
    let result = invalid(
        RULE,
        case(
            "test<Type>('a');\ncustom<Type>('b');",
            json!({ "functions": ["test", { "name": "custom", "message": "Custom message here" }] }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noExplicitGenerics'
      msg: 'Function 'test' should be called with inferred generics (remove the explicit type parameters)'
      line: 1
    - messageId: 'noExplicitGenericsCustom'
      msg: 'Custom message here'
      line: 2
    ");
}

#[test]
fn parenthesized_and_computed_callees_are_matched_like_estree() {
    let result = invalid(
        RULE,
        case(
            "(test)<Type>('a');\nnamespace[method]<Type>('b');",
            json!({ "functions": ["test", "namespace.method"] }),
        ),
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noExplicitGenerics', line: 1 }
    - { messageId: 'noExplicitGenerics', line: 2 }
    ");
}
