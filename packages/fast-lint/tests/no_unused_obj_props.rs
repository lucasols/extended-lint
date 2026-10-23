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
    valid(
        RULE,
        indoc! {r#"
        const obj = { a: 1, b: 2 };
        const prop = 'a';
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
