mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "require-const-binding";

fn case(code: &str) -> Case<'_> {
    Case {
        code,
        options: Some(json!([{ "functions": ["rc_obj_builder"] }])),
        filename: None,
    }
}

#[test]
fn allows_const_binding() {
    valid(
        RULE,
        case(indoc! {r#"
        const userSchema = rc_obj_builder<User>()({ name: rc_string })
    "#}),
    );
}

#[test]
fn allows_exported_const_binding() {
    valid(
        RULE,
        case(indoc! {r#"
        export const userSchema = rc_obj_builder<User>()({ name: rc_string })
    "#}),
    );
}

#[test]
fn allows_chained_calls_in_const_binding() {
    valid(
        RULE,
        case(indoc! {r#"
        const userSchema = rc_obj_builder<User>()({ name: rc_string })
          .orNull()
          .withFallback(null)
          .optional()
          .withFallback(undefined)
    "#}),
    );
}

#[test]
fn allows_parens_and_type_wrappers_in_const_binding() {
    valid(
        RULE,
        case(indoc! {r#"
        const a = (rc_obj_builder<User>()({ name: rc_string }))
        const b = rc_obj_builder<User>()({ name: rc_string }) satisfies Schema
        const c = rc_obj_builder<User>()({ name: rc_string }) as const
        const d = rc_obj_builder<User>()({ name: rc_string })!
    "#}),
    );
}

#[test]
fn allows_const_binding_inside_function() {
    valid(
        RULE,
        case(indoc! {r#"
        function parse(value: unknown) {
          const userSchema = rc_obj_builder<User>()({ name: rc_string })
          return userSchema.parse(value)
        }
    "#}),
    );
}

#[test]
fn allows_referencing_the_binding_in_other_schemas() {
    valid(
        RULE,
        case(indoc! {r#"
        const userSchema = rc_obj_builder<User>()({ name: rc_string })
        const usersSchema = rc_array(userSchema)
        const teamSchema = rc_obj_builder<Team>()({ users: rc_array(userSchema.orNull()) })
    "#}),
    );
}

#[test]
fn ignores_functions_not_listed() {
    valid(
        RULE,
        case(indoc! {r#"
        const teamSchema = rc_object({ users: rc_array(rc_object({ name: rc_string })) })
    "#}),
    );
}

#[test]
fn ignores_member_calls_with_the_same_name() {
    valid(
        RULE,
        case(indoc! {r#"
        const teamSchema = rc_array(rc.rc_obj_builder<User>()({ name: rc_string }))
    "#}),
    );
}

#[test]
fn reports_call_nested_in_another_call() {
    let result = invalid(
        RULE,
        case(indoc! {r#"
        const usersSchema = rc_array(
          rc_obj_builder<User>()({ name: rc_string }),
        )
    "#}),
    );
    assert_snapshot!(result.errors_with_msg(), @"
    - messageId: 'requireConstBinding'
      msg: '`rc_obj_builder` must be the value of its own `const` binding, declare it as a `const` and reference it'
      line: 2
    ");
}

#[test]
fn reports_call_nested_in_another_call_with_chained_calls() {
    let result = invalid(
        RULE,
        case(indoc! {r#"
        const usersSchema = rc_array(
          rc_obj_builder<User>()({ name: rc_string }).orNull(),
        )
    "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'requireConstBinding', line: 2 }");
}

#[test]
fn reports_call_nested_in_an_object_property() {
    let result = invalid(
        RULE,
        case(indoc! {r#"
        const teamSchema = rc_obj_builder<Team>()({
          owner: rc_obj_builder<User>()({ name: rc_string }),
          admin: rc_obj_builder<User>()({ name: rc_string }).orNull(),
        })

        const blockCfg = {
          apiPropsSchema: rc_obj_builder<Cfg>()({ title: rc_string }),
        }
    "#}),
    );
    assert_snapshot!(result.errors(), @"
    - { messageId: 'requireConstBinding', line: 2 }
    - { messageId: 'requireConstBinding', line: 3 }
    - { messageId: 'requireConstBinding', line: 7 }
    ");
}

#[test]
fn reports_call_passed_to_a_chained_method() {
    let result = invalid(
        RULE,
        case(indoc! {r#"
        const userSchema = rc_obj_builder<User>()({ name: rc_string }).extend(
          rc_obj_builder<Extra>()({ id: rc_number }),
        )
    "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'requireConstBinding', line: 2 }");
}

#[test]
fn reports_non_const_bindings() {
    let result = invalid(
        RULE,
        case(indoc! {r#"
        let a = rc_obj_builder<User>()({ name: rc_string })
        var b = rc_obj_builder<User>()({ name: rc_string })
    "#}),
    );
    assert_snapshot!(result.errors(), @"
    - { messageId: 'requireConstBinding', line: 1 }
    - { messageId: 'requireConstBinding', line: 2 }
    ");
}

#[test]
fn reports_calls_outside_variable_declarations() {
    let result = invalid(
        RULE,
        case(indoc! {r#"
        function getSchema() {
          return rc_obj_builder<User>()({ name: rc_string })
        }

        export default rc_obj_builder<User>()({ name: rc_string })

        parse(rc_obj_builder<User>()({ name: rc_string }), value)
    "#}),
    );
    assert_snapshot!(result.errors(), @"
    - { messageId: 'requireConstBinding', line: 2 }
    - { messageId: 'requireConstBinding', line: 5 }
    - { messageId: 'requireConstBinding', line: 7 }
    ");
}

#[test]
fn reports_destructured_const_bindings() {
    let result = invalid(
        RULE,
        case(indoc! {r#"
        const { shape } = rc_obj_builder<User>()({ name: rc_string })
    "#}),
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'requireConstBinding', line: 1 }");
}
