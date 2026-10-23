mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-optional-root-props";

#[test]
fn no_optional_properties() {
    valid(
        RULE,
        indoc! {r#"
            type Foo = { a: number; b: string };
            const x: Foo = { a: 42, b: "hello" };
        "#},
    );
}

#[test]
fn exported_type_alias_with_optional_properties() {
    valid(
        RULE,
        indoc! {r#"
            export type Bar = { a?: number; b: string };
            const y: Bar = { b: "world" };
        "#},
    );
}

#[test]
fn referenced_multiple_times() {
    valid(
        RULE,
        indoc! {r#"
            type Baz = { a?: number; b: string };
            const x: Baz = { b: "one" };
            const y: Baz = { b: "two" };
        "#},
    );
}

#[test]
fn local_type_alias_referenced_once_with_optional_prop() {
    valid(
        RULE,
        indoc! {r#"
            type Qux = { a?: number; b: string };
            const x: Qux = { b: "error" };
        "#},
    );
}

#[test]
fn local_type_alias_with_two_optional_props() {
    valid(
        RULE,
        indoc! {r"
            type Local = { a?: number; b?: string; c: boolean };
            const x: Local = { c: true };
        "},
    );
}

#[test]
fn interface_with_no_optional_properties() {
    valid(
        RULE,
        indoc! {r#"
            interface Foo {
              a: number;
              b: string;
            }
            const x: Foo = { a: 42, b: "hello" };
        "#},
    );
}

#[test]
fn exported_interface_with_optional_properties() {
    valid(
        RULE,
        indoc! {r#"
            export interface Bar {
              a?: number;
              b: string;
            }
            const y: Bar = { b: "world" };
        "#},
    );
}

#[test]
fn interface_referenced_multiple_times() {
    valid(
        RULE,
        indoc! {r#"
            interface Baz {
              a?: number;
              b: string;
            }
            const x: Baz = { b: "one" };
            const y: Baz = { b: "two" };
        "#},
    );
}

#[test]
fn local_interface_referenced_once_with_optional_prop() {
    valid(
        RULE,
        indoc! {r#"
            interface Qux {
              a?: number;
              b: string;
            }
            const x: Qux = { b: "error" };
        "#},
    );
}

#[test]
fn nested_optional_properties() {
    valid(
        RULE,
        indoc! {r"
            type Nested = { a: { b?: string }; c: number };
            const x: Nested = { a: {}, c: 42 };
        "},
    );
}

#[test]
fn inline_type_in_function_parameter() {
    valid(
        RULE,
        indoc! {r#"
            function test(arg: { a?: number; b: string }) {}
            test({ b: "test" });
        "#},
    );
}

#[test]
fn ignore_types_used_in_exported_functions() {
    valid(
        RULE,
        indoc! {r"
            type Foo = { a?: number; b: string };
            export function test(arg: Foo) {}
        "},
    );
}

#[test]
fn declarations_exported_indirectly_should_not_be_checked() {
    valid(
        RULE,
        indoc! {r"
            type Foo = { a?: number; };
            const test = (arg: Foo) => {};

            export const Test2 = test
        "},
    );
}

#[test]
fn declarations_exported_indirectly_in_function_should_not_be_checked() {
    valid(
        RULE,
        indoc! {r"
            type Foo = { a?: number; };
            function test(arg: Foo) {}

            export const Test2 = test
        "},
    );
}

#[test]
fn ignore_types_used_in_exported_variables() {
    valid(
        RULE,
        indoc! {r#"
            type Foo = { a?: number; b: string };
            export const test: Foo = { a: 1, b: "test" };
        "#},
    );
}

#[test]
fn ignore_interface_used_in_exported_function_parameter() {
    valid(
        RULE,
        indoc! {r"
            interface Foo {
              a?: number;
              b: string;
            }
            export function test<T extends string>(t: T, arg?: Foo) {}
        "},
    );
}

#[test]
fn ignore_on_return_type() {
    valid(
        RULE,
        indoc! {r"
            type Foo = { a?: number; b: string };
            function test(): Foo {}
        "},
    );
}

#[test]
fn not_check_all_parent_type_nodes_for_export() {
    valid(
        RULE,
        indoc! {r"
            export type Test = Intermediate;

            type Intermediate = {
              b: WithOptional;
            }

            type WithOptional = {
              a?: number;
              b: string;
            }
        "},
    );
}

#[test]
fn not_check_parent_variable_exports() {
    valid(
        RULE,
        indoc! {r"
            export const Test: Intermediate = {};

            type Intermediate = {
              b: WithOptional;
            }

            type WithOptional = {
              a?: number;
              b: string;
            }
        "},
    );
}

#[test]
fn not_throw_error_on_circular_references() {
    valid(
        RULE,
        indoc! {r"
            type Circular = { a: Test };
            type Test = Circular;
        "},
    );
}

#[test]
fn ignore_nested_references() {
    valid(
        RULE,
        indoc! {r"
            type Test = { a?: string }

            const x: { a?: Test } = { a: {} }
        "},
    );
}

#[test]
fn check_nested_variables() {
    valid(
        RULE,
        indoc! {r"
            export function test(arg: { a?: string }) {
              type Test = { a?: string }

              const x: Test = {}
            }
        "},
    );
}

#[test]
fn mutable_object() {
    valid(
        RULE,
        indoc! {r"
            type Mutable = { a?: number }

            const x: Mutable = {}

            x.a = 2
        "},
    );
}

#[test]
fn exported_fc_component() {
    valid(
        RULE,
        indoc! {r"
            type Props = { a?: number }

            export const Component: FC<Props> = () => {}
        "},
    );
}

#[test]
fn optional_arg_prop() {
    valid(
        RULE,
        indoc! {r"
            type Props = { a?: number }

            function test({ a }: Props = {}) {}
        "},
    );
}

#[test]
fn optional_arg_prop_in_arrow_function() {
    valid(
        RULE,
        indoc! {r"
            type CurrencyMaskConfig = {
              currencySymbol?: string;
              locale?: string;
            };

            export const currencyMask = (
              value: string,
              { currencySymbol = '', locale = 'en' }: CurrencyMaskConfig = {},
            ) => {}
        "},
    );
}

#[test]
fn destructured() {
    valid(
        RULE,
        indoc! {r"
            type StoryDocProps = {
              markdown: MarkdownText;
              className?: string;
            };

            export const StoryDoc = ({ markdown, className }: StoryDocProps) => {
              return null;
            };
        "},
    );
}

#[test]
fn ignore_exported_function_parameter() {
    valid(
        RULE,
        indoc! {r"
            type Props = {
              inline?: boolean;
              rotate?: number;
            };

            export const FieldTypeIcon = memo(
              ({ inline, rotate }: Props) => {
                return null;
              },
            );
        "},
    );
}

#[test]
fn exported_fc_component_with_inline_type_should_not_be_checked() {
    valid(
        RULE,
        r"export const Component: FC<{ prop?: boolean }> = () => {}",
    );
}

#[test]
fn fc_component_with_generic_type_parameter() {
    valid(
        RULE,
        indoc! {r"
            type Props<T> = { data?: T }
            export const Component: FC<Props<string>> = () => {}
        "},
    );
}

#[test]
fn fc_component_with_complex_type() {
    valid(
        RULE,
        r"const Component: FC<{ nested: { optional?: string } }> = () => {}",
    );
}

#[test]
fn fc_component_used_multiple_times_should_not_show_error() {
    valid(
        RULE,
        indoc! {r"
            const Test = () => {
              return <Component />
            }

            const Test2 = () => {
              return <Component />
            }

            const Component: FC<{ prop?: string }> = () => {}
        "},
    );
}

#[test]
fn fc_component_exported_indirectly_should_not_be_checked() {
    valid(
        RULE,
        indoc! {r"
            const Component: FC<{ prop?: string }> = () => {}

            export const Test = Component
        "},
    );
}

#[test]
fn fc_component_exported_indirectly_in_memo_should_not_be_checked() {
    valid(
        RULE,
        indoc! {r"
            const Component: FC<{ prop?: string }> = () => {}

            export const Test = memo(Component)
        "},
    );
}

#[test]
fn fc_component_exported_indirectly_in_memo_should_not_be_checked_2() {
    valid(
        RULE,
        indoc! {r"
            type Props = { prop?: string }

            const Component: FC<Props> = () => {}

            export const Test = memo(Component)
        "},
    );
}

#[test]
fn local_interface_arg_with_two_optional_props() {
    let result = invalid(
        RULE,
        indoc! {r"
            interface Local {
              a?: number;
              b?: string;
              c: boolean;
            }

            function test(arg: Local) {
            }

            console.log(test)
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 2
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "b" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 3
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    interface Local {
      a: undefined | number;
      b?: string;
      c: boolean;
    }

    function test(arg: Local) {
    }

    console.log(test)
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[1], 0), @r"
    interface Local {
      a?: number;
      b: undefined | string;
      c: boolean;
    }

    function test(arg: Local) {
    }

    console.log(test)
    ");
}

#[test]
fn used_once_in_function_parameter() {
    let result = invalid(
        RULE,
        indoc! {r"
            type Foo = { a?: number; b: string };
            function test(arg: Foo) {}

            console.log(test)
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type Foo = { a: undefined | number; b: string };
    function test(arg: Foo) {}

    console.log(test)
    ");
}

#[test]
fn used_once_in_arrow_function_parameter() {
    let result = invalid(
        RULE,
        indoc! {r"
            type Foo = { a?: number; b: string };
            const test = (arg: Foo) => {};

            console.log(test)
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type Foo = { a: undefined | number; b: string };
    const test = (arg: Foo) => {};

    console.log(test)
    ");
}

#[test]
fn check_nested_functions() {
    let result = invalid(
        RULE,
        indoc! {r"
            export function test(arg: { a?: string }) {
              type Test = { a?: string }

              function inner(arg: Test) {}

              inner({})
            }
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 2
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    export function test(arg: { a?: string }) {
      type Test = { a: undefined | string }

      function inner(arg: Test) {}

      inner({})
    }
    ");
}

#[test]
fn fc_component() {
    let result = invalid(
        RULE,
        indoc! {r"
            type Props = { a?: number }

            const Component: FC<Props> = () => {}

            const Parent = () => {
              return <Component />
            }
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type Props = { a: undefined | number }

    const Component: FC<Props> = () => {}

    const Parent = () => {
      return <Component />
    }
    ");
}

#[test]
fn fc_component_with_inline_type() {
    let result = invalid(
        RULE,
        indoc! {r"
            export const Parent = () => {
              return <Component />
            }

            const Component: FC<{ prop?: boolean }> = () => {}
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "prop" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 5
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    export const Parent = () => {
      return <Component />
    }

    const Component: FC<{ prop: undefined | boolean }> = () => {}
    ");
}

#[test]
fn fc_component_with_multiple_inline_optional_props() {
    let result = invalid(
        RULE,
        indoc! {r"
            export const Parent = () => {
              return <Component />
            }

            const Component: FC<{ prop1?: string; prop2?: number; required: boolean }> = () => {}
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "prop1" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 5
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "prop2" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 5
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    export const Parent = () => {
      return <Component />
    }

    const Component: FC<{ prop1: undefined | string; prop2?: number; required: boolean }> = () => {}
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[1], 0), @r"
    export const Parent = () => {
      return <Component />
    }

    const Component: FC<{ prop1?: string; prop2: undefined | number; required: boolean }> = () => {}
    ");
}

#[test]
fn fc_component_used_only_once_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r"
            const Test = () => {
              return <Component />
            }

            const Component: FC<{ prop?: string }> = () => {}
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "prop" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 5
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    const Test = () => {
      return <Component />
    }

    const Component: FC<{ prop: undefined | string }> = () => {}
    ");
}

#[test]
fn extra_computed_key() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const k = "a"
            type P = { [k]?: number }
            function f(a: P) {}
            f({})
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "k" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 2
    "#);
    assert_snapshot!(result.suggestion_output(0), @r#"
    const k = "a"
    type P = { [k]: undefined | number }
    function f(a: P) {}
    f({})
    "#);
}

#[test]
fn extra_rest_parameter() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            function f(...a: P[]) {}
            f()
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    function f(...a: P[]) {}
    f()
    ");
}

#[test]
fn extra_class_method_parameter() {
    valid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            class A { m(a: P) {} }
            new A()
        "},
    );
}

#[test]
fn extra_object_method_parameter() {
    valid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            const o = { m(a: P) {} }
            o.m({})
        "},
    );
}

#[test]
fn extra_fc_exported_as_default() {
    let result = invalid(
        RULE,
        indoc! {r"
            const C: FC<{ a?: string }> = () => null
            export default C
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    const C: FC<{ a: undefined | string }> = () => null
    export default C
    ");
}

#[test]
fn extra_function_exported_as_default() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            function f(x: P) {}
            export default f
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    function f(x: P) {}
    export default f
    ");
}

#[test]
fn extra_parenthesized_type_literal() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = ({ a?: number })
            function f(x: P) {}
            f({})
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = ({ a: undefined | number })
    function f(x: P) {}
    f({})
    ");
}

#[test]
fn extra_interface_method_and_string_keys() {
    let result = invalid(
        RULE,
        indoc! {r#"
            interface P { m?(): void; "s"?: number; b?: string }
            function f(x: P) {}
            f({})
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "b" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r#"
    interface P { m?(): void; "s"?: number; b: undefined | string }
    function f(x: P) {}
    f({})
    "#);
}

#[test]
fn extra_qualified_fc() {
    valid(
        RULE,
        indoc! {r"
            const C: React.FC<{ a?: string }> = () => null
            const D = () => <C />
        "},
    );
}

#[test]
fn extra_fc_used_by_local_component() {
    let result = invalid(
        RULE,
        indoc! {r"
            const C: FC<{ a?: string }> = () => null
            const D = () => <C />
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    const C: FC<{ a: undefined | string }> = () => null
    const D = () => <C />
    ");
}

#[test]
fn extra_arrow_inside_call() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            const x = useCallback((a: P) => {}, [])
            x({})
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    const x = useCallback((a: P) => {}, [])
    x({})
    ");
}

#[test]
fn extra_interface_heritage() {
    valid(
        RULE,
        indoc! {r#"
            interface A { a?: number }
            interface B extends A { b: string }
            function f(x: B) {}
            f({ b: "" })
        "#},
    );
}

#[test]
fn extra_type_argument_in_function_body() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            function f() { return useState<P>() }
            f()
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    function f() { return useState<P>() }
    f()
    ");
}

#[test]
fn extra_var_function_in_block() {
    valid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            { var f = (x: P) => {} }
            f({})
        "},
    );
}

#[test]
fn extra_overload_signature() {
    valid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            function f(x: P): void
            function f(x: unknown) {}
            f({})
        "},
    );
}

#[test]
fn extra_destructured_default_param() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            const f = ({ a }: P) => a
            f({})
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    const f = ({ a }: P) => a
    f({})
    ");
}

#[test]
fn extra_optional_with_spaces() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a ?: number; b?:string }
            function f(x: P) {}
            f({})
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "b" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a : undefined | number; b?:string }
    function f(x: P) {}
    f({})
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[1], 0), @r"
    type P = { a ?: number; b: undefined |string }
    function f(x: P) {}
    f({})
    ");
}

#[test]
fn extra_fc_inside_union() {
    valid(
        RULE,
        indoc! {r"
            const C: FC<{ a?: string }> | undefined = () => null
            const D = () => <C />
        "},
    );
}

#[test]
fn extra_fc_memo_wrapped() {
    valid(
        RULE,
        indoc! {r"
            const C: FC<{ a?: string }> = () => null
            const M = memo(C)
        "},
    );
}

#[test]
fn extra_type_used_in_return_type() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            const f = (): P => ({})
            f()
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    const f = (): P => ({})
    f()
    ");
}

#[test]
fn extra_type_in_generic_constraint() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            function f<T extends P>(x: T) {}
            f({})
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    function f<T extends P>(x: T) {}
    f({})
    ");
}

#[test]
fn extra_type_in_nested_arrow() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            const f = () => (x: P) => x
            f()
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    const f = () => (x: P) => x
    f()
    ");
}

#[test]
fn extra_type_used_in_typeof_query_of_fn() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            function f(x: P) {}
            type F = typeof f
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    function f(x: P) {}
    type F = typeof f
    ");
}

#[test]
fn extra_namespace_export() {
    let result = invalid(
        RULE,
        indoc! {r"
            type P = { a?: number }
            function f(x: P) {}
            export { f }
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'optionalNotAllowed'
      msg: 'Optional property "a" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.'
      line: 1
    "#);
    assert_snapshot!(result.suggestion_output(0), @r"
    type P = { a: undefined | number }
    function f(x: P) {}
    export { f }
    ");
}
