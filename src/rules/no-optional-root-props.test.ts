import { createTester } from '../../tests/utils/createTester'
import { noOptionalRootProps } from './no-optional-root-props'

const tests = createTester(noOptionalRootProps)

tests.addValid(
  'no optional properties',
  `
    type Foo = { a: number; b: string };
    const x: Foo = { a: 42, b: "hello" };
  `,
)

tests.addValid(
  'exported type alias with optional properties',
  `
    export type Bar = { a?: number; b: string };
    const y: Bar = { b: "world" };
  `,
)

tests.addValid(
  'referenced multiple times',
  `
    type Baz = { a?: number; b: string };
    const x: Baz = { b: "one" };
    const y: Baz = { b: "two" };
  `,
)

tests.addValid(
  'local type alias referenced once with optional prop',
  `
    type Qux = { a?: number; b: string };
    const x: Qux = { b: "error" };
  `,
)

tests.addValid(
  'local type alias with two optional props',
  `
    type Local = { a?: number; b?: string; c: boolean };
    const x: Local = { c: true };
  `,
)

tests.addValid(
  'interface with no optional properties',
  `
    interface Foo {
      a: number;
      b: string;
    }
    const x: Foo = { a: 42, b: "hello" };
  `,
)

tests.addValid(
  'exported interface with optional properties',
  `
    export interface Bar {
      a?: number;
      b: string;
    }
    const y: Bar = { b: "world" };
  `,
)

tests.addValid(
  'interface referenced multiple times',
  `
    interface Baz {
      a?: number;
      b: string;
    }
    const x: Baz = { b: "one" };
    const y: Baz = { b: "two" };
  `,
)

tests.addValid(
  'local interface referenced once with optional prop',
  `
    interface Qux {
      a?: number;
      b: string;
    }
    const x: Qux = { b: "error" };
  `,
)

tests.addInvalid(
  'local interface arg with two optional props',
  `
    interface Local {
      a?: number;
      b?: string;
      c: boolean;
    }

    function test(arg: Local) {
    }
  `,
  [
    { messageId: 'optionalNotAllowed', data: { propertyName: 'a' } },
    { messageId: 'optionalNotAllowed', data: { propertyName: 'b' } },
  ],
  {
    output: `
      interface Local {
        a: undefined | number;
        b: undefined | string;
        c: boolean;
      }

      function test(arg: Local) {
      }
    `,
  },
)

tests.addValid(
  'nested optional properties',
  `
    type Nested = { a: { b?: string }; c: number };
    const x: Nested = { a: {}, c: 42 };
  `,
)

tests.addValid(
  'inline type in function parameter',
  `
    function test(arg: { a?: number; b: string }) {}
    test({ b: "test" });
  `,
)

tests.addValid(
  'ignore types used in exported functions',
  `
    type Foo = { a?: number; b: string };
    export function test(arg: Foo) {}
  `,
)

tests.addInvalid(
  'used once in function parameter',
  `
    type Foo = { a?: number; b: string };
    function test(arg: Foo) {}
  `,
  [{ messageId: 'optionalNotAllowed', data: { propertyName: 'a' } }],
  {
    output: `
      type Foo = { a: undefined | number; b: string };
      function test(arg: Foo) {}
    `,
  },
)

tests.addInvalid(
  'used once in arrow function parameter',
  `
    type Foo = { a?: number; b: string };
    const test = (arg: Foo) => {};
  `,
  [{ messageId: 'optionalNotAllowed', data: { propertyName: 'a' } }],
  {
    output: `
      type Foo = { a: undefined | number; b: string };
      const test = (arg: Foo) => {};
    `,
  },
)

tests.addValid(
  'ignore types used in exported variables',
  `
    type Foo = { a?: number; b: string };
    export const test: Foo = { a: 1, b: "test" };
  `,
)

tests.addValid(
  'ignore interface used in exported function parameter',
  `
    interface Foo {
      a?: number;
      b: string;
    }
    export function test<T extends string>(t: T, arg?: Foo) {}
  `,
)

tests.addValid(
  'ignore on return type',
  `
    type Foo = { a?: number; b: string };
    function test(): Foo {}
  `,
)

tests.addValid(
  'not check all parent type nodes for export',
  `
    export type Test = Intermediate;

    type Intermediate = {
      b: WithOptional;
    }

    type WithOptional = {
      a?: number;
      b: string;
    }
  `,
)

tests.addValid(
  'not check parent variable exports',
  `
    export const Test: Intermediate = {};

    type Intermediate = {
      b: WithOptional;
    }

    type WithOptional = {
      a?: number;
      b: string;
    }
  `,
)

tests.addValid(
  'not throw error on circular references',
  `
    type Circular = { a: Test };
    type Test = Circular;
  `,
)

tests.addValid(
  'ignore nested references',
  `
    type Test = { a?: string }

    const x: { a?: Test } = { a: {} }
  `,
)

tests.addInvalid(
  'check nested functions',
  `
    export function test(arg: { a?: string }) {
      type Test = { a?: string }

      function inner(arg: Test) {}

      inner({})
    }
  `,
  [{ messageId: 'optionalNotAllowed', data: { propertyName: 'a' } }],
  {
    output: `
      export function test(arg: { a?: string }) {
        type Test = { a: undefined | string }

        function inner(arg: Test) {}

        inner({})
      }
    `,
  },
)

tests.addValid(
  'check nested variables',
  `
    export function test(arg: { a?: string }) {
      type Test = { a?: string }

      const x: Test = {}
    }
  `,
)

tests.addValid(
  'mutable object',
  `
    type Mutable = { a?: number }

    const x: Mutable = {}

    x.a = 2
  `,
)

tests.addInvalid(
  'FC component',
  `
    type Props = { a?: number }

    const Component: FC<Props> = () => {}
  `,
  [{ messageId: 'optionalNotAllowed', data: { propertyName: 'a' } }],
  {
    output: `
      type Props = { a: undefined | number }

      const Component: FC<Props> = () => {}
    `,
  },
)

tests.addValid(
  'exported FC component',
  `
    type Props = { a?: number }

    export const Component: FC<Props> = () => {}
  `,
)

tests.addValid(
  'optional arg prop',
  `
    type Props = { a?: number }

    function test({ a }: Props = {}) {}
  `,
)

tests.addValid(
  'optional arg prop in arrow function',
  `
    type CurrencyMaskConfig = {
      currencySymbol?: string;
      locale?: string;
    };

    export const currencyMask = (
      value: string,
      { currencySymbol = '', locale = 'en' }: CurrencyMaskConfig = {},
    ) => {}
  `,
)

tests.addValid(
  'destructured',
  `
    type StoryDocProps = {
      markdown: MarkdownText;
      className?: string;
    };

    export const StoryDoc = ({ markdown, className }: StoryDocProps) => {
      return null;
    };
  `,
)

tests.addValid(
  'ignore exported function parameter',
  `

      type Props = {
        inline?: boolean;
        rotate?: number;
      };

      export const FieldTypeIcon = memo(
        ({ inline, rotate }: Props) => {
          return null;
        },
      );
  `,
)

tests.run()
