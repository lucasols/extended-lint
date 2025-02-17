import { createTester } from '../../tests/utils/createTester'
import { collapseSimpleObjsInOneLine } from './collapse-simple-objs-in-one-line'

const tests = createTester(collapseSimpleObjsInOneLine)

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
  'multiline object with more than 3 properties',
  `
    const foo = {
      a: {
        b: 1,
        c: 2,
        d: 3,
        e: 4,
      }
    }
    type Bar = {
      a: {
        b: string
        c: string
        d: string
        e: string
      }
    }
    interface Baz {
      a: {
        b: number
        c: number
        d: number
        e: number
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
      c: boolean;
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
  'nested single prop type in interface',
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

tests.addValid(
  'should consider the trailing comma/semicolon in line length',
  `
    type Bar = {
      a: {
        bc: 1;
      };
      c: 2;
    }

    const foo = {
      a: {
        bc: 1,
      },
      c: 2,
    }
  `,
  { maxLineLength: 14 },
)

tests.addValid(
  'should consider not having a trailing comma/semicolon in line length',
  `
    type Bar = {
      a: {
        bc: 1,
      }
      c: 2;
    }
  `,
  { maxLineLength: 14 },
)

tests.addValid(
  'should ignore interfaces',
  `
    interface SidePanelParagraphProps {
      children: React.ReactNode;
    }
  `,
  { maxLineLength: 80 },
)

tests.describe('collapse objects with more than one property', () => {
  tests.addInvalid(
    'object with more than one property',
    `
      const foo = {
        a: 1,
        b: 2,
      }
    `,
    [{ messageId: 'singleLineProp' }],
    {
      output: `
        const foo = { a: 1, b: 2 }
      `,
    },
  )

  tests.addInvalid(
    'type with more than one property',
    `
      type Bar = {
        a: 1,
        b: 2,
      }
    `,
    [{ messageId: 'singleLineProp' }],
    {
      output: `
        type Bar = { a: 1; b: 2 }
      `,
    },
  )

  tests.addValid(
    'by default, collapse objects up to 2 properties',
    `
      const foo = {
        a: 1,
        b: 2,
        c: 3,
      }
    `,
  )

  tests.addInvalidWithOptions(
    'maxProperties',
    `
      const foo = {
        a: 1,
        b: 2,
        c: 3,
      }
    `,
    { maxProperties: 3 },
    [{ messageId: 'singleLineProp' }],
    {
      output: `
        const foo = { a: 1, b: 2, c: 3 }
      `,
    },
  )

  tests.addInvalid(
    'multiple props in type',
    `
    type Bar = {
        a: string;
        b: number;
      }
    `,
    [{ messageId: 'singleLineProp' }],
    {
      output: `
        type Bar = { a: string; b: number }
      `,
    },
  )

  tests.addValid(
    'ignore complex objects',
    `
      const foo = {
        a: withObj.member.expression,
        b: 2,
      }

      const bar = {
        a: withFunctionCall(),
        b: 2,
      }

      const baz = {
        a: withBooleanLogic || foo,
        b: 2,
      }
    `,
    { maxProperties: 100 },
  )

  tests.addValid(
    'ignore complex types',
    `
      type Bar = {
        a: WithPropertyAccess['a'],
        b: number
      }

      type Baz = {
        a: With | Union;
        b: number
      }

      type Qux = {
        a: WithComplexTypeArgument<string | number>;
        b: number
      }

      type Quux = {
        a: WithComplexTypeArgument<string | number>;
        b: number
      }

      type EditorProps = ModalSharedProps & {
        title: string;
        edit: ApiRole;
      };

      type EditorProps = ModalSharedProps | {
        title: string;
        edit: ApiRole;
      };
    `,
    { maxProperties: 100 },
  )

  tests.addInvalidWithOptions(
    'collapse objects with simple values',
    `
      const foo = {
        a: 1,
        b: '2',
        c: true,
        d: false,
        e: null,
        f: undefined,
        g: variable,
        shortHand,
      }
    `,
    { maxProperties: 100 },
    [{ messageId: 'singleLineProp' }],
    {
      output: `
        const foo = { a: 1, b: '2', c: true, d: false, e: null, f: undefined, g: variable, shortHand }
      `,
    },
  )

  tests.addInvalidWithOptions(
    'collapse types with simple values',
    `
      type Bar = {
        a: 1,
        b: '2',
        c: true,
        d: false,
        e: null,
        f: undefined,
        g: TypeRef,
        h: WithTypeArgument<string>
      }
    `,
    { maxProperties: 100 },
    [{ messageId: 'singleLineProp' }],
    {
      output: `
        type Bar = { a: 1; b: '2'; c: true; d: false; e: null; f: undefined; g: TypeRef; h: WithTypeArgument<string> }
      `,
    },
  )
})

tests.describe('objects inside JSX attributes', () => {
  tests.addInvalid(
    'object inside JSX attribute',
    `
      const foo = (
        <div
          a={{
            a: 1,
            b: 2,
          }}
          b="test"
        />
      );
    `,
    [{ messageId: 'singleLineProp' }],
    {
      output: `
        const foo = (
          <div
            a={{ a: 1, b: 2 }}
            b="test"
          />
        );
      `,
    },
  )
})

tests.describe('ignore types with suffix', () => {
  tests.addValid(
    'ignore types with suffix',
    `
      type Props = {
        a: 1,
      }

      type TestProps = {
        a: 1,
        b: 2,
      }
    `,
    { ignoreTypesWithSuffix: ['Props'] },
  )
})

tests.describe('nestedObjMaxLineLength', () => {
  tests.addInvalidWithOptions(
    'nestedObjMaxLineLength',
    `
      type Bar = {
        a: 1;
        b: {
          a: 1,
          b: 2,
          c: 3,
        }
        d: {
          a: 1,
          b: 2,
          c: 3,
          d: 4,
        }
      }

      const foo = {
        a: 1,
        b: {
          a: 1,
          b: 2,
          c: 3,
        },
        d: {
          a: 1,
          b: 2,
          c: 3,
          d: 4,
        }
      }
    `,
    { nestedObjMaxLineLength: 26 },
    [{ messageId: 'singleLineProp' }, { messageId: 'singleLineProp' }],
    {
      output: `
        type Bar = {
          a: 1;
          b: { a: 1; b: 2; c: 3 }
          d: {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
          }
        }

        const foo = {
          a: 1,
          b: { a: 1, b: 2, c: 3 },
          d: {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
          }
        }
      `,
    },
  )
})

tests.run()
