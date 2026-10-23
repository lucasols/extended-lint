mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-unused-obj-props";

#[test]
fn no_object_usage() {
    valid(
        RULE,
        indoc! {r#"
        const x = 5;
    "#},
    );
}

#[test]
fn all_object_properties_are_used() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        console.log(obj.a, obj.b);
    "#},
    );
}

#[test]
fn object_with_explicit_type_annotation() {
    valid(
        RULE,
        indoc! {r#"
        const obj: { a: number; b: number } = { a: 1, b: 2 };
        console.log(obj.a);
    "#},
    );
}

#[test]
fn used_object() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        console.log(obj);
    "#},
    );
}

#[test]
fn exported_object() {
    valid(
        RULE,
        indoc! {r#"
        export const obj = { a: 1, b: 2 };
        console.log(obj.a);
    "#},
    );
}

#[test]
fn exported_object_2() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };

        export const test = obj;
    "#},
    );
}

#[test]
fn destructuring_assignment() {
    valid(
        RULE,
        indoc! {r#"
        const { a, b } = { a: 1, b: 2 };
    "#},
    );
}

#[test]
fn spread_operator_usage() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const newObj = { ...obj, c: 3 };
    "#},
    );
}

#[test]
fn non_identifier_property_keys() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { 'a-b': 1, ['dynamic']: 2 };
        console.log(obj['a-b'], obj['dynamic']);
    "#},
    );
}

#[test]
fn computed_property_access() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const prop = 'a';
        console.log(obj[prop]);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 1
    "#);
}

#[test]
fn computed_property_access_with_string_key() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        let prop = 'a';
        prop = 'b';
        console.log(obj[prop]);
    "#},
    );
}

#[test]
fn object_with_method_that_might_use_this() {
    valid(
        RULE,
        indoc! {r#"
        const obj = {
          a: 1,
          b: 2,
          method() { return this.a; }
        };
        obj.method();
    "#},
    );
}

#[test]
fn object_passed_to_function() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        someFunction(obj);
    "#},
    );
}

#[test]
fn object_used_in_assignment() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const newObj = obj;
    "#},
    );
}

#[test]
fn object_with_nested_object() {
    valid(
        RULE,
        indoc! {r#"
        const obj = {
          a: 1,
          nested: {
            b: 2,
            c: 3
          }
        };
        console.log(obj.a, obj.nested);
    "#},
    );
}

#[test]
fn object_with_computed_property_access_using_literal() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        console.log(obj['a'], obj['b']);
    "#},
    );
}

#[test]
fn object_with_property_access_in_loop() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const keys = ['a', 'b'];
        for (const key of keys) {
          console.log(obj[key]);
        }
    "#},
    );
}

#[test]
fn object_with_getter_setter() {
    valid(
        RULE,
        indoc! {r#"
        const obj = {
          _value: 0,
          get value() { return this._value; },
          set value(v) { this._value = v; }
        };
        console.log(obj.value);
        obj.value = 42;
    "#},
    );
}

#[test]
fn object_with_property_access_in_try_catch() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        try {
          console.log(obj.a, obj.b);
        } catch (e) {
          console.error(e);
        }
    "#},
    );
}

#[test]
fn object_spread_in_jsx() {
    valid(
        RULE,
        indoc! {r#"
        const props = { id: 'test', className: 'container' };
        const element = <div {...props}>Content</div>;
    "#},
    );
}

#[test]
fn object_passed_as_jsx_props() {
    valid(
        RULE,
        indoc! {r#"
        const style = { color: 'red', fontSize: '16px' };
        const element = <div style={style}>Styled content</div>;
    "#},
    );
}

#[test]
fn object_used_in_react_component() {
    valid(
        RULE,
        indoc! {r#"
        function Component() {
          const data = { title: 'Hello', description: 'World' };
          return (
            <div>
              <h1>{data.title}</h1>
              <p>{data.description}</p>
            </div>
          );
        }
    "#},
    );
}

#[test]
fn object_with_explicit_type_in_react_component() {
    valid(
        RULE,
        indoc! {r#"
        interface Props {
          user: {
            name: string;
            email: string;
          };
        }

        function UserProfile({ user }: Props) {
          const details = { name: user.name, email: user.email };
          return (
            <div>
              <h2>{details.name}</h2>
              <p>{details.email}</p>
            </div>
          );
        }
    "#},
    );
}

#[test]
fn object_returned_directly() {
    valid(
        RULE,
        indoc! {r#"
        function getData() {
          const data = { id: 1, name: 'Test', value: 42 };
          return data;
        }
    "#},
    );
}

#[test]
fn object_returned_in_arrow_function() {
    valid(
        RULE,
        indoc! {r#"
        const getData = () => {
          const data = { id: 1, name: 'Test', value: 42 };
          return data;
        }
    "#},
    );
}

#[test]
fn object_returned_conditionally() {
    valid(
        RULE,
        indoc! {r#"
        function getData(condition) {
          const data = { id: 1, name: 'Test', value: 42 };
          if (condition) {
            return data;
          }
          return null;
        }
    "#},
    );
}

#[test]
fn object_used_in_multiple_ways() {
    valid(
        RULE,
        indoc! {r#"
        function process() {
          const data = { id: 1, name: 'Test', value: 42 };
          console.log(data.id);
          return data;
        }
    "#},
    );
}

#[test]
fn object_passed_to_another_function() {
    valid(
        RULE,
        indoc! {r#"
        function process() {
          const data = { id: 1, name: 'Test', value: 42 };
          processData(data);
        }
    "#},
    );
}

#[test]
fn object_used_in_template_literal() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        console.log(`${obj}`);
    "#},
    );
}

#[test]
fn object_used_in_array() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const arr = [obj];
    "#},
    );
}

#[test]
fn object_used_in_object_property() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const container = { inner: obj };
    "#},
    );
}

#[test]
fn object_used_in_conditional() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        if (obj) {
          console.log('has object');
        }
    "#},
    );
}

#[test]
fn unused_object_property() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        console.log(obj.a);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 1
    "#);
}

#[test]
fn multiple_unused_object_properties() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2, c: 3, d: 4 };
        console.log(obj.a, obj.d);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 1
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "c" is not being used'
      line: 1
    "#);
}

#[test]
fn unused_property_in_nested_scope() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function test() {
          const obj = { a: 1, b: 2 };
          console.log(obj.a);
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 2
    "#);
}

#[test]
fn unused_properties_with_arrow_function_that_doesn_t_use_this() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = {
          a: 1,
          b: 2,
          method: () => 42
        };
        obj.method();
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "a" is not being used'
      line: 2
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 3
    "#);
}

#[test]
fn unused_property_with_shorthand() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const a = 1, b = 2;
        const obj = { a, b };
        console.log(obj.a);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 2
    "#);
}

#[test]
fn unused_property_with_computed_access_to_other_property() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2, c: 3 };
        console.log(obj.a, obj['b']);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "c" is not being used'
      line: 1
    "#);
}

#[test]
fn unused_property_in_object_with_nested_objects() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = {
          a: 1,
          b: 2,
          nested: {
            c: 3,
            d: 4
          }
        };
        console.log(obj.a, obj.nested);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 3
    "#);
}

#[test]
fn object_with_mixed_property_access_and_unused_property() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2, c: 3 };
        console.log(obj.a, obj['b']);
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "c" is not being used'
      line: 1
    "#);
}

#[test]
fn unused_property_in_object_used_in_jsx() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function Component() {
          const styles = { color: 'red', fontSize: '16px', margin: '10px' };
          return <div style={{ color: styles.color, fontSize: styles.fontSize }}>Content</div>;
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "margin" is not being used'
      line: 2
    "#);
}

#[test]
fn partial_property_access_in_object() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function getData() {
          const data = { id: 1, name: 'Test', value: 42 };
          console.log(data.id, data.name);
          // Only using id and name, not value
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "value" is not being used'
      line: 2
    "#);
}

fn invalid_errors(code: &str) -> String {
    invalid(RULE, code).errors_with_msg()
}

#[test]
fn destructured_keys_are_used() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { a: 1, b: 2, c: 3, d: 4 };
        const { a, b: renamed, c = 5 } = obj;
        console.log(a, renamed, c);
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "d" is not being used'
      line: 1
    "#);
}

#[test]
fn nested_destructuring_marks_the_outer_key_used() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { a: { x: 1 }, b: 2 };
        const { a: { x } } = obj;
        console.log(x);
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 1
    "#);
}

#[test]
fn destructuring_assignment_marks_keys_used() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { a: 1, b: 2 };
        let a = 0;
        ({ a } = obj);
        console.log(a);
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 1
    "#);
}

#[test]
fn destructuring_with_rest_is_an_escape() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const { a, ...rest } = obj;
        console.log(a, rest);
    "#},
    );
}

#[test]
fn destructuring_with_computed_key_is_an_escape() {
    valid(
        RULE,
        indoc! {r#"
        declare const key: string;
        const obj = { a: 1, b: 2 };
        const { [key]: value } = obj;
        console.log(value);
    "#},
    );
}

#[test]
fn as_const_and_parenthesized_initializers_are_checked() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { a: 1, b: 2 } as const;
        const other = ({ c: 1, d: 2 });
        console.log(obj.a, other.c);
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 1
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "d" is not being used'
      line: 2
    "#);
}

#[test]
fn satisfies_and_other_assertions_are_skipped() {
    valid(
        RULE,
        indoc! {r#"
        type Config = { a: number; b: number };
        const satisfied = { a: 1, b: 2 } satisfies Config;
        const asserted = { a: 1, b: 2 } as Config;
        console.log(satisfied.a, asserted.a);
    "#},
    );
}

#[test]
fn string_and_numeric_keys_are_checked() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { 'a-b': 1, 'c-d': 2, 'e-f': 3, 1: 'x', 2: 'y', ['g']: 4 };
        console.log(obj['a-b'], obj[`c-d`], obj[1]);
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "e-f" is not being used'
      line: 1
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "2" is not being used'
      line: 1
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "g" is not being used'
      line: 1
    "#);
}

#[test]
fn computed_identifier_keys_are_not_tracked() {
    valid(
        RULE,
        indoc! {r#"
        const KEY = 'a';
        const obj = { [KEY]: 1, b: 2 };
        console.log(obj.b);
    "#},
    );
}

#[test]
fn proto_key_is_not_a_property() {
    valid(
        RULE,
        indoc! {r#"
        const proto = { x: 1 };
        const obj = { __proto__: proto, b: 2 };
        console.log(obj.b);
    "#},
    );
}

#[test]
fn literal_typed_dynamic_keys_are_used() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { a: 1, b: 2, c: 3 };
        const keys = ['a', 'b'] as const;
        for (const key of keys) {
          console.log(obj[key]);
        }
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "c" is not being used'
      line: 1
    "#);
}

#[test]
fn write_only_properties_are_reported() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { a: 1, b: 2, c: 3 };
        obj.a = 3;
        obj['c'] = 4;
        console.log(obj.b);
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "a" is not being used'
      line: 1
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "c" is not being used'
      line: 1
    "#);
}

#[test]
fn compound_assignments_updates_and_deletes_read_the_property() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2, c: { d: 1 } as { d?: number }, e: { f: 1 } };
        obj.a += 1;
        obj.b++;
        delete obj.c.d;
        obj.e.f = 2;
    "#},
    );
}

#[test]
fn deleting_a_property_reads_it() {
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        delete obj.a;
        console.log(obj.b);
    "#},
    );
}

#[test]
fn jsx_member_tag_reads_the_property() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const ui = { Header: () => <h1 />, Body: () => <main />, Footer: () => <p /> };
        const page = (
          <div>
            <ui.Header />
            <ui.Body></ui.Body>
          </div>
        );
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "Footer" is not being used'
      line: 1
    "#);
}

#[test]
fn passing_a_property_to_a_call_is_a_read() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        declare function use(value: unknown): void;
        const obj = { a: 1, b: 2, run() { return 1 } };
        use(obj.a);
        obj.run();
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 2
    "#);
}

#[test]
fn methods_using_this_are_an_escape() {
    valid(
        RULE,
        indoc! {r#"
        const obj = {
          a: 1,
          b: 2,
          sum() {
            const add = () => this.a + this.b;
            return add();
          },
        };
        console.log(obj.sum);
    "#},
    );
}

#[test]
fn function_property_using_this_is_an_escape() {
    valid(
        RULE,
        indoc! {r#"
        const obj = {
          a: 1,
          read: function () {
            return this.a;
          },
        };
        console.log(obj.read());
    "#},
    );
}

#[test]
fn referenced_function_using_this_is_an_escape() {
    for code in [
        "function readA(this: { a: number }) {\n  return this.a;\n}\nconst obj = { a: 1, b: 2, readA };\nconsole.log(obj.readA(), obj.b);",
        "const readA = function (this: { a: number }) {\n  return this.a;\n};\nconst obj = { a: 1, b: 2, read: readA };\nconsole.log(obj.read(), obj.b);",
    ] {
        valid(RULE, code);
    }
}

#[test]
fn referenced_function_without_this_is_checked() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        function readA() {
          return 1;
        }
        const double = (n: number) => n * 2;
        const obj = { a: 1, b: 2, readA, double };
        console.log(obj.readA(), obj.double(obj.b));
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "a" is not being used'
      line: 5
    "#);
}

#[test]
fn accessors_are_not_reported() {
    valid(
        RULE,
        indoc! {r#"
        let stored = 0;
        const obj = {
          get value() { return stored; },
          set value(v: number) { stored = v; },
          b: 2,
        };
        console.log(obj.b);
    "#},
    );
}

#[test]
fn escaping_uses_bail_out() {
    for code in [
        "const obj = { a: 1, b: 2 };\ntype T = typeof obj;\nconsole.log(obj.a);",
        "const obj = { a: 1, b: 2 };\ntype K = keyof typeof obj;\nconsole.log(obj.a);",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a);\nexport { obj };",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a);\nexport { obj as renamed };",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a);\nexport default obj;",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a, 'b' in obj);",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a, Object.keys(obj));",
        "const obj = { a: 1, b: 2 };\nfor (const key in obj) console.log(key);\nconsole.log(obj.a);",
        "const obj = { a: 1, b: 2 };\nlet other;\nother = obj;\nconsole.log(obj.a, other);",
        "const obj = { a: 1, b: 2 };\nconst get = () => obj;\nconsole.log(obj.a, get().b);",
        "const obj = { a: 1, b: 2 };\nconst list = [obj];\nconsole.log(obj.a, list);",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a, obj!.b);",
        "let obj = { a: 1, b: 2 };\nobj = { a: 2, b: 3 };\nconsole.log(obj.a);",
        "let obj = { a: 1, b: 2 };\nconsole.log(obj.a);\nfunction reset() {\n  obj = { a: 0, b: 0 };\n}\nreset();",
        "const obj = { a: 1, b: 2 };\ndeclare const key: string;\nconsole.log(obj.a, obj[key]);",
        "const obj = { a: 1, b: 2 };\nconsole.log(`${obj.a}`, obj);",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a, { ...obj });",
        "const obj = { a: 1, b: 2 };\nconsole.log(obj.a, <div {...obj} />);",
    ] {
        valid(RULE, code);
    }
}

#[test]
fn optional_chain_reads_the_property() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const obj = { a: 1, b: 2 };
        console.log(obj?.a);
    "#}), @r#"
    - messageId: 'unusedObjectProperty'
      msg: 'The object property "b" is not being used'
      line: 1
    "#);
}

#[test]
fn returned_object_properties_unused_by_every_caller_are_reported() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        function useThing() {
          return { a: 1, b: 2, c: 3 };
        }

        function First() {
          const { a } = useThing();
          return a;
        }

        function Second() {
          const thing = useThing();
          return thing.c;
        }
    "#}), @r#"
    - messageId: 'unusedReturnObjectProperty'
      msg: 'The object property "b" returned by this function is not being used'
      line: 2
    "#);
}

#[test]
fn returned_object_of_an_arrow_function_is_checked() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        const getData = () => ({ a: 1, b: 2 });
        const getOther = function () {
          return { c: 1, d: 2 } as const;
        };
        const data = getData();
        const { c } = getOther();
        console.log(data.a, c);
    "#}), @r#"
    - messageId: 'unusedReturnObjectProperty'
      msg: 'The object property "b" returned by this function is not being used'
      line: 1
    - messageId: 'unusedReturnObjectProperty'
      msg: 'The object property "d" returned by this function is not being used'
      line: 3
    "#);
}

#[test]
fn every_returned_object_is_checked() {
    assert_snapshot!(invalid_errors(indoc! {r#"
        function pick(flag: boolean) {
          if (flag) {
            return { a: 1, b: 2 };
          }
          const inner = () => 1;
          return { a: inner(), b: 3 };
        }
        const { a } = pick(true);
        console.log(a);
    "#}), @r#"
    - messageId: 'unusedReturnObjectProperty'
      msg: 'The object property "b" returned by this function is not being used'
      line: 3
    - messageId: 'unusedReturnObjectProperty'
      msg: 'The object property "b" returned by this function is not being used'
      line: 6
    "#);
}

#[test]
fn returned_object_escapes_bail_out() {
    for code in [
        "export function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconsole.log(a);",
        "export const f = () => ({ a: 1, b: 2 });\nconst { a } = f();\nconsole.log(a);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconsole.log(a);\nexport { f };",
        "function f() {\n  return { a: 1, b: 2 };\n}\nexport default f;\nconst { a } = f();\nconsole.log(a);",
        "export default function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconsole.log(a);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconsole.log(a, [1].map(f));",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconsole.log(a, f());",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconsole.log(a, f().b);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconst g = () => f();\nconsole.log(a, g);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst r = f();\nconsole.log(r.a, r);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst { a, ...rest } = f();\nconsole.log(a, rest);",
        "type R = { a: number; b: number };\nfunction f(): R {\n  return { a: 1, b: 2 };\n}\nconst { a } = f();\nconsole.log(a);",
        "type R = { a: number; b: number };\nconst f = (): R => ({ a: 1, b: 2 });\nconst { a } = f();\nconsole.log(a);",
        "type R = { a: number; b: number };\nconst f: () => R = () => ({ a: 1, b: 2 });\nconst { a } = f();\nconsole.log(a);",
        "type R = { a: number; b: number };\nfunction f() {\n  return { a: 1, b: 2 };\n}\nconst r: R = f();\nconsole.log(r.a);",
        "async function f() {\n  return { a: 1, b: 2 };\n}\nasync function g() {\n  const { a } = await f();\n  return a;\n}\nconsole.log(g);",
        "function* f() {\n  yield 1;\n  return { a: 1, b: 2 };\n}\nconst r = f();\nconsole.log(r);",
        "function f(flag: boolean) {\n  if (flag) return { a: 1, b: 2 };\n  return null;\n}\nconst r = f(true);\nconsole.log(r?.a);",
        "function f() {\n  const result = { a: 1, b: 2 };\n  return result;\n}\nconst { a } = f();\nconsole.log(a);",
        "function f() {\n  return {\n    a: 1,\n    b: 2,\n    sum() {\n      return this.a;\n    },\n  };\n}\nconst r = f();\nconsole.log(r.sum());",
        "function f(n: number): { a: number; b: number } {\n  return n > 0 ? f(n - 1) : { a: 1, b: 2 };\n}\nconst { a } = f(1);\nconsole.log(a);",
        "function f(n: number) {\n  if (n > 0) {\n    const { a } = f(n - 1);\n    return { a, b: 1 };\n  }\n  return { a: 1, b: 2 };\n}\nconsole.log(f);",
        "function f(a: string): { a: string };\nfunction f(a: number): { a: number };\nfunction f(a: unknown) {\n  return { a, b: 1 };\n}\nconst { a } = f(1);\nconsole.log(a);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconsole.log(f);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nlet r = f();\nr = { a: 3, b: 4 };\nconsole.log(r.a);",
        "function f() {\n  return { a: 1, b: 2 };\n}\nconst { a } = f.call(null);\nconsole.log(a);",
        "function f() {\n  return { a: 1, b: 2 };\n}\ntype R = ReturnType<typeof f>;\nconst { a } = f();\nconsole.log(a);",
        "let f = () => ({ a: 1, b: 2 });\nf = () => ({ a: 3, b: 4 });\nconst { a } = f();\nconsole.log(a);",
        "const f = () => ({ a: 1, b: 2 });\nconst { a } = f();\nconst [first] = [f()];\nconsole.log(a, first);",
    ] {
        valid(RULE, code);
    }
}

#[test]
fn never_called_function_is_not_reported() {
    valid(
        RULE,
        indoc! {r#"
        function f() {
          return { a: 1, b: 2 };
        }
    "#},
    );
}
