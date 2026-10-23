import { createTester } from '../../tests/utils/createTester'
import { noUnnecessaryCasting } from './no-unnecessary-casting'

const tests = createTester(noUnnecessaryCasting)

// Valid cases - no unnecessary casting
tests.addValid(
  'valid string',
  `
    const str: string = 'hello'
  `,
)

tests.addValid(
  'valid number',
  `
    const num: number = 42
  `,
)

tests.addValid(
  'valid Number casting of non-number types',
  `
    const num1 = Number('42')
    const num2 = Number(true)
    const num3 = Number(undefined)
    const num4 = Number(null)
    const num5 = Number({ valueOf() { return 42 } })
  `,
)

tests.addValid(
  'valid String casting of non-string types',
  `
    const str1 = String(42)
    const str2 = String(true)
    const str3 = String(undefined)
    const str4 = String(null)
    const str5 = String({ toString() { return 'hello' } })
  `,
)

tests.addValid(
  'custom Number and String functions in namespace',
  `
    // Define custom functions in a module
    export namespace CustomFunctions {
      export function Number(value: any): number {
        console.log('Custom Number function');
        return value + 1;
      }

      export function String(value: any): string {
        console.log('Custom String function');
        return value + 'str';
      }
    }

    // Use the custom functions
    const a = CustomFunctions.Number(42);
    const b = CustomFunctions.String('hello');
  `,
)

tests.addInvalid(
  'locally defined Number and String functions',
  `
    // These will now be caught since we don't check if they're built-in
    function Number(value: number): number {
      return value + 1;
    }

    function String(value: string): string {
      return value + '!';
    }

    const num = 42;
    const str = 'hello';

    const a = Number(num);
    const b = String(str);
  `,
  [
    { messageId: 'unnecessaryNumberCasting' },
    { messageId: 'unnecessaryStringCasting' },
  ],
  {
    output: `
      // These will now be caught since we don't check if they're built-in
      function Number(value: number): number {
        return value + 1;
      }

      function String(value: string): string {
        return value + '!';
      }

      const num = 42;
      const str = 'hello';

      const a = num;
      const b = str;
    `,
  },
)

// Invalid cases - unnecessary Number() casting
tests.addInvalid(
  'unnecessary Number() on number literal',
  `
    const num = Number(42)
  `,
  [{ messageId: 'unnecessaryNumberCasting' }],
  {
    output: `
      const num = 42
    `,
  },
)

tests.addInvalid(
  'unnecessary Number() on number variable',
  `
    const x = 42
    const num = Number(x)
  `,
  [{ messageId: 'unnecessaryNumberCasting' }],
  {
    output: `
      const x = 42
      const num = x
    `,
  },
)

tests.addInvalid(
  'unnecessary Number() on expression with number type',
  `
    const x = 42
    const y = 10
    const num = Number(x + y)
  `,
  [{ messageId: 'unnecessaryNumberCasting' }],
  {
    output: `
      const x = 42
      const y = 10
      const num = x + y
    `,
  },
)

// Invalid cases - unnecessary String() casting
tests.addInvalid(
  'unnecessary String() on string literal',
  `
    const str = String('hello')
  `,
  [{ messageId: 'unnecessaryStringCasting' }],
  {
    output: `
      const str = 'hello'
    `,
  },
)

tests.addInvalid(
  'unnecessary String() on string variable',
  `
    const x = 'hello'
    const str = String(x)
  `,
  [{ messageId: 'unnecessaryStringCasting' }],
  {
    output: `
      const x = 'hello'
      const str = x
    `,
  },
)

tests.addInvalid(
  'unnecessary String() on expression with string type',
  `
    const x = 'hello'
    const y = 'world'
    const str = String(x + y)
  `,
  [{ messageId: 'unnecessaryStringCasting' }],
  {
    output: `
      const x = 'hello'
      const y = 'world'
      const str = x + y
    `,
  },
)

tests.addInvalid(
  'unnecessary String() on template literal',
  `
    const x = 'hello'
    const str = String(\`\${x} world\`)
  `,
  [{ messageId: 'unnecessaryStringCasting' }],
  {
    output: `
      const x = 'hello'
      const str = \`\${x} world\`
    `,
  },
)

// Function arguments
tests.addInvalid(
  'unnecessary castings in function arguments',
  `
    function log(x: number, y: string) {
      console.log(x, y)
    }
    const num = 42
    const str = 'hello'
    log(Number(num), String(str))
  `,
  [
    { messageId: 'unnecessaryNumberCasting' },
    { messageId: 'unnecessaryStringCasting' },
  ],
  {
    output: `
      function log(x: number, y: string) {
        console.log(x, y)
      }
      const num = 42
      const str = 'hello'
      log(num, str)
    `,
  },
)

// Custom cast functions tests
tests.addInvalidWithOptions(
  'custom number cast function',
  `
    function toNumber(val: number | string): number {
      return typeof val === 'string' ? parseInt(val, 10) : val;
    }

    const num = 42;
    const result = toNumber(num);
  `,
  {
    additionalCastFunctions: [{ name: 'toNumber', expectedType: 'number' }],
  },
  [
    {
      messageId: 'unnecessaryCustomCasting',
      data: { name: 'toNumber', type: 'number' },
    },
  ],
  {
    output: `
      function toNumber(val: number | string): number {
        return typeof val === 'string' ? parseInt(val, 10) : val;
      }

      const num = 42;
      const result = num;
    `,
  },
)

tests.addInvalidWithOptions(
  'custom string cast function',
  `
    function formatString(val: string): string {
      return val.trim();
    }

    const text = 'hello world';
    const result = formatString(text);
  `,
  {
    additionalCastFunctions: [{ name: 'formatString', expectedType: 'string' }],
  },
  [
    {
      messageId: 'unnecessaryCustomCasting',
      data: { name: 'formatString', type: 'string' },
    },
  ],
  {
    output: `
      function formatString(val: string): string {
        return val.trim();
      }

      const text = 'hello world';
      const result = text;
    `,
  },
)

tests.addValid(
  'appropriate custom cast functions',
  `
    function toNumber(val: string): number {
      return parseInt(val, 10);
    }

    function toString(val: number): string {
      return val.toString();
    }

    const strNum = '42';
    const num = 42;

    // These are valid because the types don't match
    const result1 = toNumber(strNum);
    const result2 = toString(num);
  `,
  {
    additionalCastFunctions: [
      { name: 'toNumber', expectedType: 'number' },
      { name: 'toString', expectedType: 'string' },
    ],
  },
)

// Run the tests
tests.run()
