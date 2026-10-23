import { dedent } from '@ls-stack/utils/dedent'
import { createTester } from '../../tests/utils/createTester'
import { preferNamedFunction } from './prefer-named-functions'

const tests = createTester(preferNamedFunction, {
  defaultErrorId: 'default',
})

tests.addValid(
  'named function exports',
  `
    export function test() {
    }
    export function anotherTest() {
    }
  `,
)

tests.addValid(
  'exported function expressions',
  `
    export const test = function() {
    }
    export const anotherTest = function namedFn() {
    }
  `,
)

tests.addValid(
  'iife',
  `
    (() => {})();

    (function () {})();

    (async () => {})();
  `,
)

tests.addValid(
  'allow typed arrow functions',
  `
    export const test: VoidFunction = () => {
    }
  `,
)

tests.addValid(
  'allow arrow functions matching ignoreRegex',
  `
    export const testFn = (a: T) => {
    }
  `,
  [
    {
      ignoreRegex: 'Fn$',
    },
  ],
)

tests.addValid(
  'one line arrow function with implicit return',
  `
    const test = () => true
  `,
)

tests.addValid(
  'arrow function with implicit return in multiple lines',
  `
    const test = (a: number, b: string) =>
      a + b;
  `,
)

// invalid

tests.addInvalid(
  'exported arrow function',
  `
    export const test = (arg1: string) => {
      console.log('test');
      return null;
    }
  `,
  [
    {
      data: { functionName: 'test' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: `
            export function test(arg1: string) {
              console.log('test');
              return null;
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalid(
  'multiple exported arrow functions',
  `
    export const test = () => {
    }
    export const anotherTest = () => {
    }
  `,
  [
    {
      data: { functionName: 'test' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            export function test() {
            }
            export const anotherTest = () => {
            }
          `,
        },
      ],
    },
    {
      data: { functionName: 'anotherTest' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            export const test = () => {
            }
            export function anotherTest() {
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalid(
  'exported arrow function with parameters',
  `
    export const test = (a: number, b: string) => {
    }
  `,
  [
    {
      data: { functionName: 'test' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            export function test(a: number, b: string) {
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalid(
  'exported async arrow function',
  `
    export const test = async () => {
    }
  `,
  [
    {
      data: { functionName: 'test' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            export async function test() {
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalid(
  'allow arrow functions matching ignoreRegex 2',
  `
    export const test = (a: T) => {
    }
  `,
  [
    {
      messageId: 'withIgnoreRegex',
      data: {
        functionName: 'test',
        ignoreRegex: 'Fn$',
      },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            export function test(a: T) {
            }
          `,
        },
      ],
    },
  ],
  {
    options: [
      {
        ignoreRegex: 'Fn$',
      },
    ],
  },
)

tests.addInvalid(
  'arrow function',
  `
    const test = () => {
    }
    const anotherTest = () => {
    }
  `,
  [
    {
      data: { functionName: 'test' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            function test() {
            }
            const anotherTest = () => {
            }
          `,
        },
      ],
    },
    {
      data: { functionName: 'anotherTest' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            const test = () => {
            }
            function anotherTest() {
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalid(
  'async arrow function',
  `
    const test = async () => {
    }
  `,
  [
    {
      data: { functionName: 'test' },
      suggestions: [
        {
          messageId: 'suggestion',
          output: dedent`
            async function test() {
            }
          `,
        },
      ],
    },
  ],
)

tests.run()
