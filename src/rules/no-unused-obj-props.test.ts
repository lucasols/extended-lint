import { createTester } from '../../tests/utils/createTester'
import { noUnusedObjProps } from './no-unused-obj-props'

const tests = createTester(noUnusedObjProps, {
  defaultErrorId: 'unusedObjectProperty',
})

// Valid test cases
tests.addValid(
  'no object usage',
  `
    const x = 5;
  `,
)

tests.addValid(
  'all object properties are used',
  `
    const obj = { a: 1, b: 2 };
    console.log(obj.a, obj.b);
  `,
)

tests.addValid(
  'object with explicit type annotation',
  `
    const obj: { a: number; b: number } = { a: 1, b: 2 };
    console.log(obj.a);
  `,
)

tests.addValid(
  'used object',
  `
    const obj = { a: 1, b: 2 };
    console.log(obj);
  `,
)

tests.addValid(
  'exported object',
  `
    export const obj = { a: 1, b: 2 };
    console.log(obj.a);
  `,
)

tests.addValid(
  'exported object 2',
  `
    const obj = { a: 1, b: 2 };

    export const test = obj;
  `,
)

tests.addValid(
  'destructuring assignment',
  `
    const { a, b } = { a: 1, b: 2 };
  `,
)

tests.addValid(
  'spread operator usage',
  `
    const obj = { a: 1, b: 2 };
    const newObj = { ...obj, c: 3 };
  `,
)

tests.addValid(
  'non-identifier property keys',
  `
    const obj = { 'a-b': 1, ['dynamic']: 2 };
    console.log(obj['a-b'], obj['dynamic']);
  `,
)

tests.addValid(
  'computed property access',
  `
    const obj = { a: 1, b: 2 };
    const prop = 'a';
    console.log(obj[prop]);
  `,
)

tests.addValid(
  'object with method that might use this',
  `
    const obj = {
      a: 1,
      b: 2,
      method() { return this.a; }
    };
    obj.method();
  `,
)

tests.addValid(
  'object passed to function',
  `
    const obj = { a: 1, b: 2 };
    someFunction(obj);
  `,
)

tests.addValid(
  'object used in assignment',
  `
    const obj = { a: 1, b: 2 };
    const newObj = obj;
  `,
)

tests.addValid(
  'object with nested object',
  `
    const obj = {
      a: 1,
      nested: {
        b: 2,
        c: 3
      }
    };
    console.log(obj.a, obj.nested);
  `,
)

tests.addValid(
  'object with computed property access using literal',
  `
    const obj = { a: 1, b: 2 };
    console.log(obj['a'], obj['b']);
  `,
)

tests.addValid(
  'object with property access in loop',
  `
    const obj = { a: 1, b: 2 };
    const keys = ['a', 'b'];
    for (const key of keys) {
      console.log(obj[key]);
    }
  `,
)

tests.addValid(
  'object with getter/setter',
  `
    const obj = {
      _value: 0,
      get value() { return this._value; },
      set value(v) { this._value = v; }
    };
    console.log(obj.value);
    obj.value = 42;
  `,
)

tests.addValid(
  'object with property access in try/catch',
  `
    const obj = { a: 1, b: 2 };
    try {
      console.log(obj.a, obj.b);
    } catch (e) {
      console.error(e);
    }
  `,
)

// JSX test cases
tests.addValid(
  'object spread in JSX',
  `
    const props = { id: 'test', className: 'container' };
    const element = <div {...props}>Content</div>;
  `,
)

tests.addValid(
  'object passed as JSX props',
  `
    const style = { color: 'red', fontSize: '16px' };
    const element = <div style={style}>Styled content</div>;
  `,
)

tests.addValid(
  'object used in React component',
  `
    function Component() {
      const data = { title: 'Hello', description: 'World' };
      return (
        <div>
          <h1>{data.title}</h1>
          <p>{data.description}</p>
        </div>
      );
    }
  `,
)

tests.addValid(
  'object with explicit type in React component',
  `
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
  `,
)

// New tests for objects being returned
tests.addValid(
  'object returned directly',
  `
    function getData() {
      const data = { id: 1, name: 'Test', value: 42 };
      return data;
    }
  `,
)

tests.addValid(
  'object returned in arrow function',
  `
    const getData = () => {
      const data = { id: 1, name: 'Test', value: 42 };
      return data;
    }
  `,
)

tests.addValid(
  'object returned conditionally',
  `
    function getData(condition) {
      const data = { id: 1, name: 'Test', value: 42 };
      if (condition) {
        return data;
      }
      return null;
    }
  `,
)

tests.addValid(
  'object used in multiple ways',
  `
    function process() {
      const data = { id: 1, name: 'Test', value: 42 };
      console.log(data.id);
      return data;
    }
  `,
)

tests.addValid(
  'object passed to another function',
  `
    function process() {
      const data = { id: 1, name: 'Test', value: 42 };
      processData(data);
    }
  `,
)

tests.addValid(
  'object used in template literal',
  `
    const obj = { a: 1, b: 2 };
    console.log(\`\${obj}\`);
  `,
)

tests.addValid(
  'object used in array',
  `
    const obj = { a: 1, b: 2 };
    const arr = [obj];
  `,
)

tests.addValid(
  'object used in object property',
  `
    const obj = { a: 1, b: 2 };
    const container = { inner: obj };
  `,
)

tests.addValid(
  'object used in conditional',
  `
    const obj = { a: 1, b: 2 };
    if (obj) {
      console.log('has object');
    }
  `,
)

// Invalid test cases
tests.addInvalid(
  'unused object property',
  `
    const obj = { a: 1, b: 2 };
    console.log(obj.a);
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'b' } }],
)

tests.addInvalid(
  'multiple unused object properties',
  `
    const obj = { a: 1, b: 2, c: 3, d: 4 };
    console.log(obj.a, obj.d);
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
    { messageId: 'unusedObjectProperty', data: { name: 'c' } },
  ],
)

tests.addInvalid(
  'unused property in nested scope',
  `
    function test() {
      const obj = { a: 1, b: 2 };
      console.log(obj.a);
    }
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'b' } }],
)

tests.addInvalid(
  "unused properties with arrow function that doesn't use this",
  `
    const obj = {
      a: 1,
      b: 2,
      method: () => 42
    };
    obj.method();
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'a' } },
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
  ],
)

tests.addInvalid(
  'unused property with shorthand',
  `
    const a = 1, b = 2;
    const obj = { a, b };
    console.log(obj.a);
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'b' } }],
)

tests.addInvalid(
  'unused property with computed access to other property',
  `
    const obj = { a: 1, b: 2, c: 3 };
    console.log(obj.a, obj['b']);
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'c' } }],
)

tests.addInvalid(
  'unused property in object with nested objects',
  `
    const obj = {
      a: 1,
      b: 2,
      nested: {
        c: 3,
        d: 4
      }
    };
    console.log(obj.a, obj.nested);
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'b' } }],
)

tests.addInvalid(
  'object with mixed property access and unused property',
  `
    const obj = { a: 1, b: 2, c: 3 };
    console.log(obj.a, obj['b']);
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'c' } }],
)

// JSX invalid test cases
tests.addInvalid(
  'unused property in object used in JSX',
  `
    function Component() {
      const styles = { color: 'red', fontSize: '16px', margin: '10px' };
      return <div style={{ color: styles.color, fontSize: styles.fontSize }}>Content</div>;
    }
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'margin' } }],
)

tests.addInvalid(
  'partial property access in object',
  `
    function getData() {
      const data = { id: 1, name: 'Test', value: 42 };
      console.log(data.id, data.name);
      // Only using id and name, not value
    }
  `,
  [{ messageId: 'unusedObjectProperty', data: { name: 'value' } }],
)

tests.run()
