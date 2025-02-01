import { createTester } from '../../tests/utils/createTester'
import { noNonCamelCaseFunctions } from './no-non-camel-case-functions'

const tests = createTester(noNonCamelCaseFunctions, {
  defaultErrorId: 'nonCamelCaseFunction',
})

tests.addValid(
  'camelCase function declarations',
  `
    function camelCase() {}
    function anotherCamelCase() {}
    function validateUserInput() {}
  `,
)

tests.addValid(
  'arrow functions (not checked)',
  `
    const PascalCase = () => {}
    const snake_case = () => {}
  `,
)

tests.addValid(
  'function expressions (not checked)',
  `
    const myFunc = function PascalCase() {}
    const myOtherFunc = function snake_case() {}
  `,
)

tests.addValid(
  'function with JSX.Element return type should be ignored',
  `
    function PascalCase(): JSX.Element {}
    function snake_case(): JSX.Element | null {}
  `,
)

tests.addInvalid(
  'PascalCase function declaration',
  `
    function PascalCase() {}
  `,
  [{ data: { functionName: 'PascalCase' } }],
)

tests.addInvalid(
  'snake_case function declaration',
  `
    function snake_case() {}
  `,
  [{ data: { functionName: 'snake_case' } }],
)

tests.addInvalid(
  'UPPERCASE function declaration',
  `
    function UPPERCASE() {}
  `,
  [{ data: { functionName: 'UPPERCASE' } }],
)

tests.run()
