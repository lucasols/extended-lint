import { createTester } from '../../tests/utils/createTester'
import { noUnnecessaryTyping } from './no-unnecessary-typing'

const tests = createTester(noUnnecessaryTyping, {
  defaultErrorId: 'unnecessaryTypeAnnotation',
})

// Default behavior
tests.addValid(
  'callback without type annotation',
  `
    const items = [1, 2, 3];
    items.find(item => item > 2);
  `,
)

tests.addValid(
  'callback with destructured parameters',
  `
    const users = [{name: 'John', age: 30}];
    users.find(({name}) => name === 'John');
  `,
)

tests.addValid(
  'function expression without annotation',
  `
    const items = [1, 2, 3];
    items.find(function(item) {
      return item > 2;
    });
  `,
)

tests.addValid(
  'explicit type needed for generics',
  `
    function customMap<T, U>(arr: T[], fn: (item: T) => U): U[] {
      return arr.map(fn);
    }
    const result = customMap<string, number>(['a', 'b'], item => item.length);
  `,
)

tests.addInvalid(
  'unnecessary type annotation in array.find callback',
  `
    type User = { id: number; name: string };
    const users: User[] = [{id: 1, name: 'John'}];
    const user = users.find((user: User) => user.id === 1);
  `,
  [{ messageId: 'unnecessaryTypeAnnotation' }],
  {
    output: `
      type User = { id: number; name: string };
      const users: User[] = [{id: 1, name: 'John'}];
      const user = users.find((user) => user.id === 1);
    `,
  },
)

tests.addInvalid(
  'unnecessary type annotation in array.map callback',
  `
    const numbers = [1, 2, 3];
    const doubled = numbers.map((num: number) => num * 2);
  `,
  [{ messageId: 'unnecessaryTypeAnnotation' }],
  {
    output: `
      const numbers = [1, 2, 3];
      const doubled = numbers.map((num) => num * 2);
    `,
  },
)

tests.addInvalid(
  'unnecessary type annotation in array.filter callback',
  `
    const strings = ['a', 'bb', 'ccc'];
    const longStrings = strings.filter((str: string) => str.length > 1);
  `,
  [{ messageId: 'unnecessaryTypeAnnotation' }],
  {
    output: `
      const strings = ['a', 'bb', 'ccc'];
      const longStrings = strings.filter((str) => str.length > 1);
    `,
  },
)

tests.addInvalid(
  'unnecessary type annotation with complex type annotation',
  `
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
  `,
  [{ messageId: 'unnecessaryTypeAnnotation' }],
  {
    output: `
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
    `,
  },
)

// Tests for array mode
tests.addValid(
  'array mode ignores custom methods',
  `
    const obj = {
      process: (callback: (x: string) => void) => callback('test')
    };
    obj.process((data: string) => console.log(data));
  `,
  { methods: 'array' },
)

tests.addInvalidWithOptions(
  'array mode still flags array methods',
  `
    const numbers = [1, 2, 3];
    const doubled = numbers.map((num: number) => num * 2);
  `,
  { methods: 'array' },
  [{ messageId: 'unnecessaryTypeAnnotation' }],
  {
    output: `
      const numbers = [1, 2, 3];
      const doubled = numbers.map((num) => num * 2);
    `,
  },
)

// Tests for specific methods array
tests.addValid(
  'specific methods ignores array methods not in list',
  `
    const numbers = [1, 2, 3];
    const doubled = numbers.map((num: number) => num * 2);
  `,
  { methods: ['find'] },
)

tests.addInvalidWithOptions(
  'specific methods flags only specified methods',
  `
    const items = [1, 2, 3];
    const found = items.find((item: number) => item > 2);
  `,
  { methods: ['find'] },
  [{ messageId: 'unnecessaryTypeAnnotation' }],
  {
    output: `
      const items = [1, 2, 3];
      const found = items.find((item) => item > 2);
    `,
  },
)

tests.addInvalidWithOptions(
  'specific methods works with custom method',
  `
    const obj = {
      customProcess: (callback: (x: string) => void) => callback('test')
    };
    obj.customProcess((data: string) => console.log(data));
  `,
  { methods: ['customProcess'] },
  [{ messageId: 'unnecessaryTypeAnnotation' }],
  {
    output: `
      const obj = {
        customProcess: (callback: (x: string) => void) => callback('test')
      };
      obj.customProcess((data) => console.log(data));
    `,
  },
)

tests.run()
