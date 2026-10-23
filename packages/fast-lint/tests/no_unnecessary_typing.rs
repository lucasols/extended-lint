mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-unnecessary-typing";

fn with_options(code: &str, options: serde_json::Value) -> Case<'_> {
    Case {
        code,
        options: Some(options),
        filename: None,
    }
}

#[test]
fn callback_without_type_annotation() {
    valid(
        RULE,
        indoc! {"
            const items = [1, 2, 3];
            items.find(item => item > 2);
        "},
    );
}

#[test]
fn callback_with_destructured_parameters() {
    valid(
        RULE,
        indoc! {"
            const users = [{name: 'John', age: 30}];
            users.find(({name}) => name === 'John');
        "},
    );
}

#[test]
fn function_expression_without_annotation() {
    valid(
        RULE,
        indoc! {"
            const items = [1, 2, 3];
            items.find(function(item) {
              return item > 2;
            });
        "},
    );
}

#[test]
fn explicit_type_needed_for_generics() {
    valid(
        RULE,
        indoc! {"
            function customMap<T, U>(arr: T[], fn: (item: T) => U): U[] {
              return arr.map(fn);
            }
            const result = customMap<string, number>(['a', 'b'], item => item.length);
        "},
    );
}

#[test]
fn unnecessary_type_annotation_in_array_find_callback() {
    let result = invalid(
        RULE,
        indoc! {"
            type User = { id: number; name: string };
            const users: User[] = [{id: 1, name: 'John'}];
            const user = users.find((user: User) => user.id === 1);
        "},
    );

    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'unnecessaryTypeAnnotation', line: 3, column: 30, endLine: 3, endColumn: 36 }");
    assert_snapshot!(result.output, @r"
    type User = { id: number; name: string };
    const users: User[] = [{id: 1, name: 'John'}];
    const user = users.find((user) => user.id === 1);
    ");
}

#[test]
fn unnecessary_type_annotation_in_array_map_callback() {
    let result = invalid(
        RULE,
        indoc! {"
            const numbers = [1, 2, 3];
            const doubled = numbers.map((num: number) => num * 2);
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeAnnotation', line: 2 }");
    assert_snapshot!(result.output, @r"
    const numbers = [1, 2, 3];
    const doubled = numbers.map((num) => num * 2);
    ");
}

#[test]
fn unnecessary_type_annotation_in_array_filter_callback() {
    let result = invalid(
        RULE,
        indoc! {"
            const strings = ['a', 'bb', 'ccc'];
            const longStrings = strings.filter((str: string) => str.length > 1);
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeAnnotation', line: 2 }");
    assert_snapshot!(result.output, @r"
    const strings = ['a', 'bb', 'ccc'];
    const longStrings = strings.filter((str) => str.length > 1);
    ");
}

#[test]
fn unnecessary_type_annotation_with_complex_type_annotation() {
    let result = invalid(
        RULE,
        indoc! {"
            interface AppConfig {
              components: Array<{ id: string; name: string }>;
            }
            const appConfig: { value: AppConfig } = {
              value: { components: [{ id: 'block1', name: 'Block 1' }] }
            };
            const blockId = 'block1';
            const blockComponent = appConfig.value.components.find(
              (component: AppConfig['components'][0]) => component.id === blockId,
            );
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeAnnotation', line: 9 }");
    assert_snapshot!(result.output, @r"
    interface AppConfig {
      components: Array<{ id: string; name: string }>;
    }
    const appConfig: { value: AppConfig } = {
      value: { components: [{ id: 'block1', name: 'Block 1' }] }
    };
    const blockId = 'block1';
    const blockComponent = appConfig.value.components.find(
      (component) => component.id === blockId,
    );
    ");
}

#[test]
fn array_mode_ignores_custom_methods() {
    valid(
        RULE,
        with_options(
            indoc! {"
                const obj = {
                  process: (callback: (x: string) => void) => callback('test')
                };
                obj.process((data: string) => console.log(data));
            "},
            json!({ "methods": "array" }),
        ),
    );
}

#[test]
fn array_mode_still_flags_array_methods() {
    let result = invalid(
        RULE,
        with_options(
            indoc! {"
                const numbers = [1, 2, 3];
                const doubled = numbers.map((num: number) => num * 2);
            "},
            json!({ "methods": "array" }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeAnnotation', line: 2 }");
    assert_snapshot!(result.output, @r"
    const numbers = [1, 2, 3];
    const doubled = numbers.map((num) => num * 2);
    ");
}

#[test]
fn specific_methods_ignores_array_methods_not_in_list() {
    valid(
        RULE,
        with_options(
            indoc! {"
                const numbers = [1, 2, 3];
                const doubled = numbers.map((num: number) => num * 2);
            "},
            json!({ "methods": ["find"] }),
        ),
    );
}

#[test]
fn specific_methods_flags_only_specified_methods() {
    let result = invalid(
        RULE,
        with_options(
            indoc! {"
                const items = [1, 2, 3];
                const found = items.find((item: number) => item > 2);
            "},
            json!({ "methods": ["find"] }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeAnnotation', line: 2 }");
    assert_snapshot!(result.output, @r"
    const items = [1, 2, 3];
    const found = items.find((item) => item > 2);
    ");
}

#[test]
fn specific_methods_works_with_custom_method() {
    let result = invalid(
        RULE,
        with_options(
            indoc! {"
                const obj = {
                  customProcess: (callback: (x: string) => void) => callback('test')
                };
                obj.customProcess((data: string) => console.log(data));
            "},
            json!({ "methods": ["customProcess"] }),
        ),
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeAnnotation', line: 4 }");
    assert_snapshot!(result.output, @r"
    const obj = {
      customProcess: (callback: (x: string) => void) => callback('test')
    };
    obj.customProcess((data) => console.log(data));
    ");
}

#[test]
fn only_the_first_argument_and_plain_parameters_are_checked() {
    valid(
        RULE,
        indoc! {"
            items.reduce((acc, item) => acc, (x: number) => x);
            items.map((...rest: number[]) => rest);
            items.map((item: number = 1) => item);
            items.map(({ a }: Item) => a);
        "},
    );
}

#[test]
fn computed_member_callee_and_function_expressions_are_checked() {
    let result = invalid(
        RULE,
        indoc! {"
            items[map]((item: number) => item);
            items.forEach(function (item: number) {});
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unnecessaryTypeAnnotation', line: 1 }
    - { messageId: 'unnecessaryTypeAnnotation', line: 2 }
    ");
    assert_snapshot!(result.output, @r"
    items[map]((item) => item);
    items.forEach(function (item) {});
    ");
}
