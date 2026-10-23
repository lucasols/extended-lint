mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-default-export";

#[test]
fn named_exports() {
    valid(
        RULE,
        indoc! {"
            export const foo = 'bar';
            export function bar() {}
            export class Baz {}
            const qux = 'quux';
            export { qux };
        "},
    );
}

#[test]
fn default_export_function() {
    let result = invalid(RULE, "export default function foo() {}");
    assert_snapshot!(result.errors(), @"- { messageId: 'noDefaultExport', line: 1 }");
}

#[test]
fn default_export_class() {
    let result = invalid(RULE, "export default class Foo {}");
    assert_snapshot!(result.errors(), @"- { messageId: 'noDefaultExport', line: 1 }");
}

#[test]
fn default_export_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const foo = 'bar';
            export default foo;
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noDefaultExport', line: 2 }");
}

#[test]
fn default_export_object() {
    let result = invalid(RULE, "export default { foo: 'bar' };");
    assert_snapshot!(result.errors(), @"- { messageId: 'noDefaultExport', line: 1 }");
}

#[test]
fn alias_default_export() {
    let result = invalid(RULE, "let foo; export { foo as default }");
    assert_snapshot!(result.errors(), @"- { messageId: 'noDefaultExport', line: 1 }");
}

#[test]
fn default_export_interface() {
    let result = invalid(RULE, "export default interface Foo {}");
    assert_snapshot!(result.errors(), @"- { messageId: 'noDefaultExport', line: 1 }");
}

#[test]
fn default_reexport_is_a_default_export() {
    let result = invalid(RULE, "export { default } from './other-module'");
    assert_snapshot!(result.errors(), @"- { messageId: 'noDefaultExport', line: 1 }");
}

#[test]
fn export_equals_is_not_a_default_export() {
    valid(RULE, "const foo = 1; export = foo");
}

#[test]
fn message() {
    let result = invalid(RULE, "export default 1");
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noDefaultExport'
      msg: 'Default exports are not allowed, use named exports instead.'
      line: 1
    ");
}
