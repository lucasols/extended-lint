mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-reexport";

#[test]
fn regular_named_export() {
    valid(RULE, "export const something = 42");
}

#[test]
fn regular_default_export() {
    valid(RULE, "export default function something() {}");
}

#[test]
fn regular_import() {
    valid(RULE, "import { something } from './other-module'");
}

#[test]
fn import_and_export_indirect_reexport() {
    let result = invalid(
        RULE,
        indoc! {"
            import { something } from './other-module'
            export const myThing = something
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn named_reexport() {
    let result = invalid(RULE, "export { something } from './other-module'");
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn multiple_named_reexports() {
    let result = invalid(RULE, "export { something, other } from './other-module'");
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn wildcard_reexport() {
    let result = invalid(RULE, "export * from './other-module'");
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn namespace_reexport() {
    let result = invalid(RULE, "export * as utils from './utils'");
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn aliased_reexport() {
    let result = invalid(
        RULE,
        "export { something as renamed } from './other-module'",
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn default_reexport() {
    let result = invalid(RULE, "export { default } from './other-module'");
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn default_reexport_with_alias() {
    let result = invalid(
        RULE,
        "export { default as something } from './other-module'",
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn import_and_export_specifier_indirect_reexport() {
    let result = invalid(
        RULE,
        indoc! {"
            import { test } from './other-module'
            export { test }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn export_specifier_with_alias_indirect_reexport() {
    let result = invalid(
        RULE,
        indoc! {"
            import { test } from './other-module'
            export { test as renamed }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn import_default_and_export_default() {
    let result = invalid(
        RULE,
        indoc! {"
            import something from './other-module'
            export default something
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn import_named_and_export_default() {
    let result = invalid(
        RULE,
        indoc! {"
            import { something } from './other-module'
            export default something
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn export_locally_defined_variable_should_use_direct_export() {
    let result = invalid(
        RULE,
        indoc! {"
            const test = 42
            export { test }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn namespace_import_and_re_export() {
    let result = invalid(
        RULE,
        indoc! {"
            import * as utils from './module'
            export const helper = utils
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn destructuring_re_export() {
    let result = invalid(
        RULE,
        indoc! {"
            import { obj } from './module'
            export const { prop } = obj
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn member_access_re_export() {
    let result = invalid(
        RULE,
        indoc! {"
            import obj from './module'
            export const prop = obj.prop
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn type_re_export() {
    let result = invalid(RULE, "export type { MyType } from './module'");
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 1 }");
}

#[test]
fn type_import_and_re_export() {
    let result = invalid(
        RULE,
        indoc! {"
            import type { MyType } from './module'
            export type { MyType }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn type_import_alias_and_exported_type_alias() {
    let result = invalid(
        RULE,
        indoc! {"
            import type { MyType as MyTypeType } from './module'
            export type MyType = MyTypeType
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noReexport', line: 2 }");
}

#[test]
fn destructuring_locally_defined_object() {
    valid(
        RULE,
        indoc! {"
            const obj = { prop: 42 }
            export const { prop } = obj
        "},
    );
}

#[test]
fn export_type_inferred_from_local_schema_with_imported_generic() {
    valid(
        RULE,
        indoc! {"
            import { z } from 'zod'

            const userSchema = z.object({
              id: z.number(),
            })

            export type User = z.infer<typeof userSchema>
        "},
    );
}

#[test]
fn export_type_created_from_imported_generic() {
    valid(
        RULE,
        indoc! {"
            import type { Result } from './module'

            type User = {
              name: string
            }

            export type UserResult = Result<User>
        "},
    );
}

#[test]
fn optional_chain_on_import_is_not_a_reexport() {
    valid(
        RULE,
        indoc! {"
            import { obj } from './module'
            export const prop = obj?.prop
        "},
    );
}

#[test]
fn parenthesized_imported_identifiers_are_reexports() {
    let result = invalid(
        RULE,
        indoc! {"
            import obj from './module'
            import type { T } from './module'
            export const prop = (obj).prop
            export type U = (T)
        "},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noReexport', line: 3 }
    - { messageId: 'noReexport', line: 4 }
    ");
}

#[test]
fn message() {
    let result = invalid(RULE, "export * from './other-module'");
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noReexport'
      msg: 'Re-exports are not allowed. Use direct exports only.'
      line: 1
    ");
}
