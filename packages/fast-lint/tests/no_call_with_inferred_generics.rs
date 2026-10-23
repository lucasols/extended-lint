mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-call-with-inferred-generics";

fn case(code: &str, options: serde_json::Value) -> Case<'_> {
    Case {
        code,
        options: Some(options),
        filename: None,
    }
}

#[test]
fn valid_code() {
    valid(
        RULE,
        case(
            "otherFunction('user/update', { name });",
            json!({ "functions": [{ "name": "test", "minGenerics": 1 }] }),
        ),
    );
}

#[test]
fn missing_generics() {
    let result = invalid(
        RULE,
        case(
            "test('user/update', { name });",
            json!({ "functions": [{ "name": "test" }] }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'missingGenericDeclaration'
      msg: 'Function 'test' should be called with at least 1 generic(s) (ex: `fn<Generic>()`) defined'
      line: 1
    ");
}

#[test]
fn needs_more_than_one_generic_defined() {
    let result = invalid(
        RULE,
        case(
            "test<Type>('user/update', { name });",
            json!({ "functions": [{ "name": "test", "minGenerics": 2 }] }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'missingGenericDeclaration'
      msg: 'Function 'test' should be called with at least 2 generic(s) (ex: `fn<Generic>()`) defined'
      line: 1
    ");
}

#[test]
fn usage_of_any() {
    let result = invalid(
        RULE,
        case(
            "test<any>('user/update', { name });",
            json!({ "functions": [{ "name": "test" }] }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'anyUsedInGenerics'
      msg: 'Function 'test' should not be called with 'any' in generics'
      line: 1
    ");
}

#[test]
fn usage_of_any_alias() {
    let result = invalid(
        RULE,
        case(
            "test<__ANY__>('user/update', { name });",
            json!({ "functions": [{ "name": "test" }], "disallowTypes": ["__ANY__"] }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'anyUsedInGenerics', line: 1 }");
}

#[test]
fn usage_of_typeof_in_generics() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                const user = { name: 'test' };
                test<typeof user>('user/update', { name });
            "},
            json!({ "functions": [{ "name": "test", "disallowTypeOf": true }] }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'typeOfUsedInGenerics'
      msg: 'Function 'test' should not be called with 'typeof' in generics'
      line: 2
    ");
}

#[test]
fn usage_of_local_type_alias_that_resolves_to_typeof() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                const user = { name: 'test' };
                type User = typeof user;
                test<User>('user/update', { name });
            "},
            json!({ "functions": [{ "name": "test", "disallowTypeOf": true }] }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'typeOfUsedInGenerics', line: 3 }");
}

#[test]
fn disallow_type_of_with_non_typeof_type_reference() {
    valid(
        RULE,
        case(
            indoc! {"
                type User = { name: string };
                test<User>('user/update', { name });
            "},
            json!({ "functions": [{ "name": "test", "disallowTypeOf": true }] }),
        ),
    );
}

#[test]
fn disallow_type_of_with_external_type_reference() {
    valid(
        RULE,
        case(
            "test<SomeImportedType>('user/update', { name });",
            json!({ "functions": [{ "name": "test", "disallowTypeOf": true }] }),
        ),
    );
}

#[test]
fn typeof_used_but_disallow_type_of_not_enabled() {
    valid(
        RULE,
        case(
            indoc! {"
                const user = { name: 'test' };
                test<typeof user>('user/update', { name });
            "},
            json!({ "functions": [{ "name": "test" }] }),
        ),
    );
}

#[test]
fn usage_of_unknown_in_generics() {
    let result = invalid(
        RULE,
        case(
            "test<unknown>('user/update', { name });",
            json!({ "functions": [{ "name": "test", "disallowUnknown": true }] }),
        ),
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'unknownUsedInGenerics'
      msg: 'Function 'test' should not be called with 'unknown' in generics'
      line: 1
    ");
}

#[test]
fn unknown_used_but_disallow_unknown_not_enabled() {
    valid(
        RULE,
        case(
            "test<unknown>('user/update', { name });",
            json!({ "functions": [{ "name": "test" }] }),
        ),
    );
}

#[test]
fn allow_any_skips_the_type_checks_without_disallow_types() {
    valid(
        RULE,
        case(
            "test<any>('a');\ntest<unknown>('b');",
            json!({ "functions": [{ "name": "test", "allowAny": true, "disallowUnknown": true }] }),
        ),
    );
}

#[test]
fn function_disallow_types_overrides_the_global_list() {
    let result = invalid(
        RULE,
        case(
            "test<__LEGIT_ANY__>('a');\nother<__ANY__>('b');",
            json!({
                "functions": [
                    { "name": "test", "disallowTypes": ["__LEGIT_ANY__"] },
                    { "name": "other", "disallowTypes": ["__LEGIT_ANY__"] },
                ],
                "disallowTypes": ["__ANY__"],
            }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'anyUsedInGenerics', line: 1 }");
}

#[test]
fn parenthesized_type_arguments_are_looked_through() {
    let result = invalid(
        RULE,
        case(
            "test<(any)>('a');",
            json!({ "functions": [{ "name": "test" }] }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'anyUsedInGenerics', line: 1 }");
}
