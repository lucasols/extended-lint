import { createTester } from '../../tests/utils/createTester'
import { wrapSingleLineProp } from './wrap-single-line-prop'

const tests = createTester(wrapSingleLineProp)

tests.addValid(
  'no single prop objects',
  `
    const foo = { a: 1, b: 2 }
    type Bar = { a: string; b: string }
    interface Baz { a: number; b: number }
  `,
)

tests.addValid(
  'empty objects',
  `
    const foo = {}
    type Bar = {}
    interface Baz {}
  `,
)

tests.addValid(
  'objects with comments',
  `
    const foo = {
      // comment
      a: 1
    }
    type Bar = {
      // comment
      a: string
    }
    interface Baz {
      // comment
      a: number
    }
  `,
)

tests.addValid(
  'multiline prop',
  `
    const foo = {
      a: {
        b: 1,
        c: 2
      }
    }
    type Bar = {
      a: {
        b: string
        c: string
      }
    }
    interface Baz {
      a: {
        b: number
        c: number
      }
    }
  `,
)

tests.addInvalid(
  'single prop object',
  `
    const foo = {
      a: 1
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    const foo = { a: 1 }
  `,
  },
)

tests.addInvalid(
  'single prop object with trailing comma',
  `
    const foo = {
      a: 1,
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    const foo = { a: 1 }
  `,
  },
)

tests.addInvalid(
  'single spread prop object',
  `
    const foo = {
      ...bar
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    const foo = { ...bar }
  `,
  },
)

tests.addInvalid(
  'single prop type',
  `
    type Bar = {
      a: string
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    type Bar = { a: string }
  `,
  },
)

tests.addInvalid(
  'single prop type with semicolon',
  `
    type Bar = {
      a: string;
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
      type Bar = { a: string }
    `,
  },
)

tests.addValid(
  'single prop type comment',
  `
    type Bar = {
      a: string; // comment
    }
  `,
)

tests.addValid(
  'multiple props',
  `
    type Bar = {
      a: string;
      b: number;
    }
  `,
)

tests.addInvalid(
  'Interface with single prop',
  `
    interface Bar {
      a: string;
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    interface Bar { a: string }
  `,
  },
)

tests.addValid(
  'interface with single prop with comment',
  `
    interface Bar {
      a: string; // comment
    }
  `,
)

tests.addInvalid(
  'single extends type',
  `
    type Bar = Foo & {
      prop: Foo
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    type Bar = Foo & { prop: Foo }
  `,
  },
)

tests.addValid(
  'not format mapped type',
  `
    type Bar = {
      [K in keyof Foo]: Foo[K]
    }
  `,
)

tests.addInvalid(
  'single prop type',
  `
    type Bar = {
      a: string
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    type Bar = { a: string }
  `,
  },
)

tests.addInvalid(
  'single prop interface',
  `
    interface Baz {
      a: number
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    interface Baz { a: number }
  `,
  },
)

tests.addInvalid(
  'nested single prop object',
  `
    const foo = {
      a: {
        b: 1
      }
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    const foo = {
      a: { b: 1 }
    }
  `,
  },
)

tests.addInvalid(
  'nested single spread prop object',
  `
    const foo = {
      a: {
        ...bar
      }
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    const foo = {
      a: { ...bar }
    }
  `,
  },
)

tests.addInvalid(
  'nested single prop type',
  `
    type Bar = {
      a: {
        b: string
      }
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    type Bar = {
      a: { b: string }
    }
  `,
  },
)

tests.addInvalid(
  'nested single prop interface',
  `
    interface Baz {
      a: {
        b: number
      }
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    interface Baz {
      a: { b: number }
    }
  `,
  },
)

tests.addValid(
  'very long prop',
  `
    const foo = {
      veryLongPropertyNameThatShouldNotFitInOneLine: 'very long value that should not fit in one line'
    }
  `,
  { maxLineLength: 50 },
)

tests.addValid(
  'long indented line',
  `
    {
      {
        {
          {
            {
              {
                const foo = {
                  a: 1
                }
              }
            }
          }
        }
      }
    }
  `,
  { maxLineLength: 25 },
)

tests.addValid(
  'do not format when result is longer than maxLineLength',
  `
    export const UpdatesPage: FC = () => {
      const activeNotification = useNewSearchParams({
        activeNotification: 'string',
      }).activeNotification;
    }
  `,
  { maxLineLength: 80 },
)

tests.addInvalid(
  'nested prop with trailing comma',
  `
    const foo = {
      a: {
        b: 1,
      },
      c: 2,
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    const foo = {
      a: { b: 1 },
      c: 2,
    }
  `,
  },
)

tests.addInvalid(
  'nested type prop with trailing semicolon',
  `
    type Bar = {
      a: {
        b: 1;
      };
      c: 2;
    }
  `,
  [{ messageId: 'singleLineProp' }],
  {
    output: `
    type Bar = {
      a: { b: 1 };
      c: 2;
    }
  `,
  },
)

tests.run()
