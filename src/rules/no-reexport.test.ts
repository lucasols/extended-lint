import { createTester } from '../../tests/utils/createTester'
import { noReexport } from './no-reexport'

const tests = createTester(noReexport, {
  defaultErrorId: 'noReexport',
})

tests.addValid(
  'regular named export',
  `
    export const something = 42
  `,
)

tests.addValid(
  'regular default export',
  `
    export default function something() {}
  `,
)

tests.addValid(
  'regular import',
  `
    import { something } from './other-module'
  `,
)

tests.addInvalid(
  'import and export (indirect reexport)',
  `
    import { something } from './other-module'
    export const myThing = something
  `,
)

tests.addInvalid(
  'named reexport',
  `
    export { something } from './other-module'
  `,
)

tests.addInvalid(
  'multiple named reexports',
  `
    export { something, other } from './other-module'
  `,
)

tests.addInvalid(
  'wildcard reexport',
  `
    export * from './other-module'
  `,
)

tests.addInvalid(
  'namespace reexport',
  `
    export * as utils from './utils'
  `,
)

tests.addInvalid(
  'aliased reexport',
  `
    export { something as renamed } from './other-module'
  `,
)

tests.addInvalid(
  'default reexport',
  `
    export { default } from './other-module'
  `,
)

tests.addInvalid(
  'default reexport with alias',
  `
    export { default as something } from './other-module'
  `,
)


tests.addInvalid(
  'import and export specifier (indirect reexport)',
  `
    import { test } from './other-module'
    export { test }
  `,
)

tests.addInvalid(
  'export specifier with alias (indirect reexport)',
  `
    import { test } from './other-module'
    export { test as renamed }
  `,
)

tests.addInvalid(
  'import default and export default',
  `
    import something from './other-module'
    export default something
  `,
)

tests.addInvalid(
  'import named and export default',
  `
    import { something } from './other-module'
    export default something
  `,
)

tests.addInvalid(
  'export locally defined variable (should use direct export)',
  `
    const test = 42
    export { test }
  `,
)

tests.addInvalid(
  'namespace import and re-export',
  `
    import * as utils from './module'
    export const helper = utils
  `,
)

tests.addInvalid(
  'destructuring re-export',
  `
    import { obj } from './module'
    export const { prop } = obj
  `,
)

tests.addInvalid(
  'member access re-export',
  `
    import obj from './module'
    export const prop = obj.prop
  `,
)

tests.addInvalid(
  'type re-export',
  `
    export type { MyType } from './module'
  `,
)

tests.addInvalid(
  'type import and re-export',
  `
    import type { MyType } from './module'
    export type { MyType }
  `,
)

tests.addValid(
  'destructuring locally defined object',
  `
    const obj = { prop: 42 }
    export const { prop } = obj
  `,
)

tests.run()
