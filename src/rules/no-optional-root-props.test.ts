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

tests.addInvalid(
  'local type alias referenced once with optional prop',
  `
    type Qux = { a?: number; b: string };
    const x: Qux = { b: "error" };
  `,
  [{ messageId: 'optionalNotAllowed', data: { propertyName: 'a' } }],
  {
    output: `
      type Qux = { a: undefined | number; b: string };
      const x: Qux = { b: "error" };
    `,
  },
)

tests.addInvalid(
  'local type alias with two optional props',
  `
    type Local = { a?: number; b?: string; c: boolean };
    const x: Local = { c: true };
  `,
  [
    { messageId: 'optionalNotAllowed', data: { propertyName: 'a' } },
    { messageId: 'optionalNotAllowed', data: { propertyName: 'b' } },
  ],
  {
    output: `
      type Local = { a: undefined | number; b: undefined | string; c: boolean };
      const x: Local = { c: true };
    `,
  },
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

tests.addInvalid(
  'local interface referenced once with optional prop',
  `
    interface Qux {
      a?: number;
      b: string;
    }
    const x: Qux = { b: "error" };
  `,
  [{ messageId: 'optionalNotAllowed', data: { propertyName: 'a' } }],
  {
    output: `
    interface Qux {
      a: undefined | number;
      b: string;
    }
    const x: Qux = { b: "error" };
  `,
  },
)

tests.addInvalid(
  'local interface with two optional props',
  `
    interface Local {
      a?: number;
      b?: string;
      c: boolean;
    }
    const x: Local = { c: true };
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
    const x: Local = { c: true };
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
  'check all parent nodes for export',
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
  'not throw error on circular references',
  `
    type Circular = { a: Test };
    type Test = Circular;
  `,
)

tests.run()
