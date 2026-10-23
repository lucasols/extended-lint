import { createTester } from '../../tests/utils/createTester'
import { noDefaultExport } from './no-default-export'

const tests = createTester(noDefaultExport, {
  defaultErrorId: 'noDefaultExport',
})

tests.addValid(
  'named exports',
  `
    export const foo = 'bar';
    export function bar() {}
    export class Baz {}
    const qux = 'quux';
    export { qux };
  `,
)

tests.addInvalid(
  'default export function',
  `
    export default function foo() {}
  `,
)

tests.addInvalid(
  'default export class',
  `
    export default class Foo {}
  `,
)

tests.addInvalid(
  'default export variable',
  `
    const foo = 'bar';
    export default foo;
  `,
)

tests.addInvalid(
  'default export object',
  `
    export default { foo: 'bar' };
  `,
)

tests.addInvalid(
  'alias default export',
  `
    let foo; export { foo as default }
  `,
)

tests.run()
