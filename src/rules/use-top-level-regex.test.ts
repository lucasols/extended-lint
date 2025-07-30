import { createTester } from '../../tests/utils/createTester'
import { useTopLevelRegex } from './use-top-level-regex'

const { addValid, addInvalid, run } = createTester(useTopLevelRegex, {
  defaultErrorId: 'regexShouldBeTopLevel',
})

addValid(
  'regex at top level',
  `
  const regex = /test/
  
  function foo() {
    return regex.test('hello')
  }
`,
)

addValid(
  'regex with global flag in function',
  `
  function foo() {
    return /test/g.exec('hello')
  }
`,
)

addValid(
  'regex with sticky flag in function',
  `
  function foo() {
    return /test/y.exec('hello')
  }
`,
)

addValid(
  'regex with both global and sticky flags',
  `
  function foo() {
    return /test/gy.exec('hello')
  }
`,
)

addValid(
  'regex in RegExp constructor',
  `
  function foo() {
    return new RegExp(/test/)
  }
`,
)

addValid(
  'regex in module level variable declaration',
  `
  const pattern = /[a-z]+/i
`,
)

addValid(
  'regex in export statement',
  `
  export const EMAIL_REGEX = /^[^@]+@[^@]+$/
`,
)

addValid(
  'regex in class property',
  `
  class Validator {
    pattern = /test/
  }
`,
)

addInvalid(
  'regex in function',
  `
  function foo(someString) {
    return /[a-Z]*/.test(someString)
  }
`,
)

addInvalid(
  'regex in arrow function',
  `
  const foo = (someString) => {
    return /[a-Z]*/.test(someString)
  }
`,
)

addInvalid(
  'regex in method',
  `
  class Test {
    validate(input) {
      return /test/.test(input)
    }
  }
`,
)

addInvalid(
  'regex in function expression',
  `
  const validator = function(input) {
    return /^[a-z]+$/.test(input)
  }
`,
)

addInvalid(
  'regex in nested function',
  `
  function outer() {
    function inner() {
      return /test/.test('hello')
    }
    return inner
  }
`,
)

addInvalid(
  'regex in callback function',
  `
  function processItems(items) {
    return items.filter(item => /valid/.test(item))
  }
`,
)

addInvalid(
  'multiple regex in same function',
  `
  function validate(input) {
    const emailPattern = /^[^@]+@[^@]+$/
    const phonePattern = /\\d{10}/
    return emailPattern.test(input) || phonePattern.test(input)
  }
`,
  2,
)

addInvalid(
  'regex in if statement inside function',
  `
  function isValid(input) {
    if (/test/.test(input)) {
      return true
    }
    return false
  }
`,
)

addInvalid(
  'regex in return statement',
  `
  function getPattern() {
    return /pattern/
  }
`,
)

addInvalid(
  'regex in object method',
  `
  const config = {
    validate: function(input) {
      return /pattern/.test(input)
    }
  }
`,
)

addInvalid(
  'regex in object arrow function',
  `
  const handlers = {
    check: (str) => /test/.test(str)
  }
`,
)

addValid(
  'regex in top-level object literal',
  `
  const patterns = {
    email: /^[^@]+@[^@]+$/,
    phone: /\\d{10}/
  }
`,
)

addValid(
  'regex in top-level array',
  `
  const testPatterns = [/test/]
`,
)

addValid(
  'regex in nested top-level object',
  `
  const config = [/test/, { regex: /test/ }]
`,
)

addInvalid(
  'regex in function inside another function',
  `
  function outer() {
    const config = {
      validate: function(input) {
        return /pattern/.test(input)
      }
    }
    return config
  }
`,
)

run()
