import { createTester } from '../../tests/utils/createTester'
import { improvedNoUnnecessaryCondition } from './improved-no-unnecessary-condition'

const tests = createTester(improvedNoUnnecessaryCondition)

tests.addValid(
  'valid code with no typeof usage',
  `
    const str = 'string'
    if (str.length > 0) {
      console.log(str)
    }
  `,
)

tests.addValid(
  'valid typeof check on variable with unknown type',
  `
    function test(value: any) {
      if (typeof value === 'string') {
        console.log(value.toUpperCase())
      }
    }
  `,
)

tests.addValid(
  'valid typeof check on variable with union type',
  `
    function test(value: string | number) {
      if (typeof value === 'string') {
        console.log(value.toUpperCase())
      }
    }
  `,
)

tests.addValid(
  'valid typeof check in ternary with union type',
  `
    function test(value: string | number) {
      const result = typeof value === 'string' ? value.toUpperCase() : value.toString();
      console.log(result);
    }
  `,
)

tests.addInvalid(
  'invalid typeof check in ternary with literal types',
  `
    function test(value: 'str' | 1) {
      const result = typeof value === 'boolean' ? value.toUpperCase() : value.toString();
      console.log(result);
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'value',
        actualType: 'string | number',
        conditionType: 'boolean',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on string variable',
  `
    const str = 'hello'
    if (typeof str === 'string') {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'str',
        type: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on string variable, typeof on right side',
  `
    const str = 'hello'
    if ('string' === typeof str) {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'str',
        type: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on string variable',
  `
    const str = 'hello'
    if (typeof str === 'number') {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'str',
        actualType: 'string',
        conditionType: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on number variable',
  `
    const num = 42
    if (typeof num === 'number') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'num',
        type: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on number variable',
  `
    const num = 42
    if (typeof num === 'string') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'num',
        actualType: 'number',
        conditionType: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on boolean variable',
  `
    const bool = true
    if (typeof bool === 'boolean') {
      console.log(bool)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'bool',
        type: 'boolean',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on boolean variable',
  `
    const bool = true
    if (typeof bool === 'object') {
      console.log(bool)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'bool',
        actualType: 'boolean',
        conditionType: 'object',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on object variable',
  `
    const obj = { a: 1 }
    if (typeof obj === 'object') {
      console.log(obj)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'obj',
        type: 'object',
      },
    },
  ],
)

tests.addValid(
  'ignore `==` operator',
  `
    const str = 'hello'
    if (typeof str == 'string') {
      console.log(str)
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check with !== operator',
  `
    const num = 42
    if (typeof num !== 'string') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'num',
        type: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check with !== operator',
  `
    const num = 42
    if (typeof num !== 'number') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'num',
        actualType: 'number',
        conditionType: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check in ternary expression',
  `
    const str = 'hello'
    const result = typeof str === 'string' ? str.toUpperCase() : str
    console.log(result)
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'str',
        type: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check in ternary expression',
  `
    const str = 'hello'
    const result = typeof str === 'number' ? str.toFixed(2) : str.toLowerCase()
    console.log(result)
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'str',
        actualType: 'string',
        conditionType: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check with not operator in ternary expression',
  `
    const num = 42
    const result = typeof num !== 'string' ? num.toFixed(2) : 'Not a number'
    console.log(result)
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'num',
        type: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check with not operator in ternary expression',
  `
    const num = 42
    const result = typeof num !== 'number' ? 'Not a number' : num.toFixed(2)
    console.log(result)
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'num',
        actualType: 'number',
        conditionType: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check in variable with union type',
  `
    const num = 42 as string | number

    if (typeof num === 'boolean') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'num',
        actualType: 'string | number',
        conditionType: 'boolean',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check in variable with union type in ternary expression',
  `
    const num = 42 as string | number

    const result = typeof num === 'boolean' ? num.toFixed(2) : num.toString()
    console.log(result)
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'num',
        actualType: 'string | number',
        conditionType: 'boolean',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check in variable with union type with !== operator',
  `
    const num = 42 as string | number

    if (typeof num !== 'boolean') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'num',
        type: 'string | number',
      },
    },
  ],
)

// Function Type Checks
tests.addValid(
  'valid typeof check for function parameter that could be a function',
  `
    function test(callback: any) {
      if (typeof callback === 'function') {
        callback()
      }
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check on function variable',
  `
    const myFunction = () => { console.log('hello') }
    if (typeof myFunction === 'function') {
      myFunction()
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'myFunction',
        type: 'function',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on function variable',
  `
    const myFunction = () => { console.log('hello') }
    if (typeof myFunction === 'string') {
      console.log(myFunction)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'myFunction',
        actualType: 'function',
        conditionType: 'string',
      },
    },
  ],
)

// Symbol Type Checks
tests.addValid(
  'valid typeof check for symbol parameter that could be a different type',
  `
    function test(value: any) {
      if (typeof value === 'symbol') {
        console.log(value.description)
      }
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check on symbol variable',
  `
    const sym = Symbol('description')
    if (typeof sym === 'symbol') {
      console.log(sym.description)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'sym',
        type: 'symbol',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on symbol variable',
  `
    const sym = Symbol('description')
    if (typeof sym === 'number') {
      console.log(sym)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'sym',
        actualType: 'symbol',
        conditionType: 'number',
      },
    },
  ],
)

// BigInt Type Checks
tests.addValid(
  'valid typeof check for bigint parameter that could be a different type',
  `
    function test(value: any) {
      if (typeof value === 'bigint') {
        console.log(value.toString())
      }
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check on bigint variable',
  `
    const big = 42n
    if (typeof big === 'bigint') {
      console.log(big.toString())
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'big',
        type: 'bigint',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on bigint variable',
  `
    const big = 42n
    if (typeof big === 'number') {
      console.log(big)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'big',
        actualType: 'bigint',
        conditionType: 'number',
      },
    },
  ],
)

// Null Type Checks
tests.addValid(
  'valid typeof check for null parameter that could be a different type',
  `
    function test(value: any) {
      if (typeof value === 'object' && value === null) {
        console.log('value is null')
      }
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check on null variable',
  `
    const nullVar = null
    if (typeof nullVar === 'object') {
      console.log('null is an object in JavaScript')
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'nullVar',
        type: 'object',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on null variable',
  `
    const nullVar = null
    if (typeof nullVar === 'string') {
      console.log(nullVar)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'nullVar',
        actualType: 'object',
        conditionType: 'string',
      },
    },
  ],
)

// Undefined Type Checks
tests.addValid(
  'valid typeof check for undefined parameter that could be a different type',
  `
    function test(value: any) {
      if (typeof value === 'undefined') {
        console.log('value is undefined')
      }
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check on undefined variable',
  `
    const undefinedVar: undefined = undefined
    if (typeof undefinedVar === 'undefined') {
      console.log('variable is undefined')
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'undefinedVar',
        type: 'undefined',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on undefined variable',
  `
    const undefinedVar: undefined = undefined
    if (typeof undefinedVar === 'number') {
      console.log(undefinedVar)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'undefinedVar',
        actualType: 'undefined',
        conditionType: 'number',
      },
    },
  ],
)

// Complex Union Types (more than two types)
tests.addValid(
  'valid typeof check for complex union type that includes the checked type',
  `
    function test(value: string | number | boolean) {
      if (typeof value === 'boolean') {
        console.log(value ? 'true' : 'false')
      }
    }
  `,
)

tests.addInvalid(
  'always false typeof check on complex union type',
  `
    const complexUnion = 'test' as string | number | boolean
    if (typeof complexUnion === 'object') {
      console.log(complexUnion)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'complexUnion',
        actualType: 'string | number | boolean',
        conditionType: 'object',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof negation check on complex union type',
  `
    const complexUnion = 'test' as string | number | boolean
    if (typeof complexUnion !== 'object') {
      console.log(complexUnion)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'complexUnion',
        type: 'string | number | boolean',
      },
    },
  ],
)

// Optional Chaining With Typeof
tests.addValid(
  'valid typeof check with optional chaining on parameter that could be null/undefined',
  `
    function test(obj: any) {
      if (typeof obj?.name === 'string') {
        console.log(obj.name.toUpperCase())
      }
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check with optional chaining',
  `
    const obj = { name: 'test' }
    if (typeof obj?.name === 'string') {
      console.log(obj.name)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'obj?.name',
        type: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check with optional chaining',
  `
    const obj = { name: 'test' }
    if (typeof obj?.name === 'number') {
      console.log(obj.name)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'obj?.name',
        actualType: 'string',
        conditionType: 'number',
      },
    },
  ],
)

// Logical Operators with Typeof
tests.addInvalid(
  'valid logical OR typeof checks on union type',
  `
    function test(value: string | number) {
      if (typeof value === 'string' || typeof value === 'number') {
        console.log(value)
      }
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'value',
        type: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary logical OR typeof checks on string variable',
  `
    const str = 'hello'
    if (typeof str === 'string' || typeof str === 'number') {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'str',
        type: 'string',
      },
    },
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'str',
        actualType: 'never',
        conditionType: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'always false logical AND typeof checks on string variable',
  `
    const str = 'hello'
    if (typeof str === 'string' && typeof str === 'number') {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'str',
        type: 'string',
      },
    },
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'str',
        actualType: 'string',
        conditionType: 'number',
      },
    },
  ],
)

tests.addValid(
  'non nullable typeof check',
  `
    function test(value: unknown) {
      if (value && typeof value === 'string') {
        console.log(value)
      }
    }
  `,
)

tests.addValid(
  'non nullable typeof check',
  `
    function test(value: {}) {
      if (typeof value === 'string') {
        console.log(value)
      }
    }
  `,
)

tests.addInvalid(
  'non nullable typeof check',
  `
    function test(value: {}) {
      if (typeof value === 'undefined') {
        console.log(value)
      }
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'value',
        actualType:
          'string | number | bigint | boolean | symbol | object | function',
        conditionType: 'undefined',
      },
    },
  ],
)

tests.addValid(
  'argument with generic type',
  `
    function test<T>(value: T) {
      if (typeof value === 'string') {
        console.log(value)
      }
    }
  `,
)

tests.run()
