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

tests.addInvalid(
  'object used in conditional with unused properties',
  `
    const obj = { a: 1, b: 2 };
    if (obj) {
      console.log('has object');
    }
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'a' } },
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
  ],
)

tests.addInvalid(
  'object used in template literal with unused properties',
  `
    const obj = { a: 1, b: 2 };
    console.log(\`\${obj}\`);
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'a' } },
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
  ],
)

tests.addInvalid(
  'object used in array with unused properties',
  `
    const obj = { a: 1, b: 2 };
    const arr = [obj];
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'a' } },
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
  ],
)

tests.addInvalid(
  'object used in object property with unused properties',
  `
    const obj = { a: 1, b: 2 };
    const container = { inner: obj };
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'a' } },
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
  ],
)

tests.addInvalid(
  'object used in return statement with unused properties',
  `
    function test() {
      const obj = { a: 1, b: 2 };
      return obj;
    }
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'a' } },
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
  ],
)

tests.addInvalid(
  'multiple objects with unused properties',
  `
    const obj1 = { a: 1, b: 2 };
    const obj2 = { c: 3, d: 4 };
    console.log(obj1.a, obj2.c);
  `,
  [
    { messageId: 'unusedObjectProperty', data: { name: 'b' } },
    { messageId: 'unusedObjectProperty', data: { name: 'd' } },
  ],
)

tests.addInvalid(
  'unused property in object with property access in different scope',
  `
    const obj = { a: 1, b: 2, c: 3 };
    function test() {
      console.log(obj.a, obj.b);
    }
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

tests.run()
