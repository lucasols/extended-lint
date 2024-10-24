import { preferNamedFunction } from '../src/rules/prefer-named-functions'
import { createTester } from './utils/createTester'

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
  'one line arrow function',
  `
    const test = () => {}
  `,
)

// invalid

tests.addInvalid(
  'exported arrow function',
  `
    export const test = () => {
    }
  `,
  [{ data: { functionName: 'test' } }],
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
    { data: { functionName: 'test' } },
    { data: { functionName: 'anotherTest' } },
  ],
)

tests.addInvalid(
  'exported arrow function with parameters',
  `
    export const test = (a: number, b: string) => {
    }
  `,
  [{ data: { functionName: 'test' } }],
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
    },
  ],
)

tests.addInvalid(
  'allow arrow functions matching ignoreRegex',
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
    { data: { functionName: 'test' } },
    { data: { functionName: 'anotherTest' } },
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
    },
  ],
)

tests.run()
