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

tests.addValid(
  'import and use separately',
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

tests.run()