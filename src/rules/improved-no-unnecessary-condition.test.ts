import { dedent } from '@ls-stack/utils/dedent'
import { describe, expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { improvedNoUnnecessaryCondition } from './improved-no-unnecessary-condition'

const { valid, invalid } = createNewTester(improvedNoUnnecessaryCondition)

test('valid code with no typeof usage', async () => {
  await valid(
    dedent`
      const str = 'string'
      if (str > 0) {
        console.log(str)
      }
    `,
  )
})

test('valid typeof check on variable with unknown type', async () => {
  await valid(
    dedent`
      function test(value: any) {
        if (typeof value === 'string') {
          console.log(value.toUpperCase())
        }
      }
    `,
  )
})

test('valid typeof check on variable with union type', async () => {
  await valid(
    dedent`
      function test(value: string | number) {
        if (typeof value === 'string') {
          console.log(value.toUpperCase())
        }
      }
    `,
  )
})

test('valid typeof check in ternary with union type', async () => {
  await valid(
    dedent`
      function test(value: string | number) {
        const result = typeof value === 'string' ? value.toUpperCase() : value.toString();
        console.log(result);
      }
    `,
  )
})

test('invalid typeof check in ternary with literal types', async () => {
  const { result } = await invalid(
    dedent`
      function test(value: 'str' | 1) {
        const result = typeof value === 'boolean' ? value.toUpperCase() : value.toString();
        console.log(result);
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check on string variable', async () => {
  const { result } = await invalid(
    dedent`
      const str = 'hello'
      if (typeof str === 'string') {
        console.log(str)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check on string variable, typeof on right side', async () => {
  const { result } = await invalid(
    dedent`
      const str = 'hello'
      if ('string' === typeof str) {
        console.log(str)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on string variable', async () => {
  const { result } = await invalid(
    dedent`
      const str = 'hello'
      if (typeof str === 'number') {
        console.log(str)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check on number variable', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42
      if (typeof num === 'number') {
        console.log(num)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on number variable', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42
      if (typeof num === 'string') {
        console.log(num)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check on boolean variable', async () => {
  const { result } = await invalid(
    dedent`
      const bool = true
      if (typeof bool === 'boolean') {
        console.log(bool)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on boolean variable', async () => {
  const { result } = await invalid(
    dedent`
      const bool = true
      if (typeof bool === 'object') {
        console.log(bool)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check on object variable', async () => {
  const { result } = await invalid(
    dedent`
      const obj = { a: 1 }
      if (typeof obj === 'object') {
        console.log(obj)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('ignore `==` operator', async () => {
  await valid(
    dedent`
      const str = 'hello'
      if (typeof str == 'string') {
        console.log(str)
      }
    `,
  )
})

test('unnecessary typeof check with !== operator', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42
      if (typeof num !== 'string') {
        console.log(num)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check with !== operator', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42
      if (typeof num !== 'number') {
        console.log(num)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check in ternary expression', async () => {
  const { result } = await invalid(
    dedent`
      const str = 'hello'
      const result = typeof str === 'string' ? str.toUpperCase() : str
      console.log(result)
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check in ternary expression', async () => {
  const { result } = await invalid(
    dedent`
      const str = 'hello'
      const result = typeof str === 'number' ? str.toFixed(2) : str.toLowerCase()
      console.log(result)
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check with not operator in ternary expression', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42
      const result = typeof num !== 'string' ? num.toFixed(2) : 'Not a number'
      console.log(result)
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check with not operator in ternary expression', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42
      const result = typeof num !== 'number' ? 'Not a number' : num.toFixed(2)
      console.log(result)
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof check in variable with union type', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42 as string | number

      if (typeof num === 'boolean') {
        console.log(num)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 3 }
    "
  `)
})

test('unnecessary typeof check in variable with union type in ternary expression', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42 as string | number

      const result = typeof num === 'boolean' ? num.toFixed(2) : num.toString()
      console.log(result)
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 3 }
    "
  `)
})

test('unnecessary typeof check in variable with union type with !== operator', async () => {
  const { result } = await invalid(
    dedent`
      const num = 42 as string | number

      if (typeof num !== 'boolean') {
        console.log(num)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 3 }
    "
  `)
})

test('valid typeof check for function parameter that could be a function', async () => {
  await valid(
    dedent`
      function test(callback: any) {
        if (typeof callback === 'function') {
          callback()
        }
      }
    `,
  )
})

test('unnecessary typeof check on function variable', async () => {
  const { result } = await invalid(
    dedent`
      const myFunction = () => { console.log('hello') }
      if (typeof myFunction === 'function') {
        myFunction()
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on function variable', async () => {
  const { result } = await invalid(
    dedent`
      const myFunction = () => { console.log('hello') }
      if (typeof myFunction === 'string') {
        console.log(myFunction)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('valid typeof check for symbol parameter that could be a different type', async () => {
  await valid(
    dedent`
      function test(value: any) {
        if (typeof value === 'symbol') {
          console.log(value.description)
        }
      }
    `,
  )
})

test('unnecessary typeof check on symbol variable', async () => {
  const { result } = await invalid(
    dedent`
      const sym = Symbol('description')
      if (typeof sym === 'symbol') {
        console.log(sym.description)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on symbol variable', async () => {
  const { result } = await invalid(
    dedent`
      const sym = Symbol('description')
      if (typeof sym === 'number') {
        console.log(sym)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('valid typeof check for bigint parameter that could be a different type', async () => {
  await valid(
    dedent`
      function test(value: any) {
        if (typeof value === 'bigint') {
          console.log(value.toString())
        }
      }
    `,
  )
})

test('unnecessary typeof check on bigint variable', async () => {
  const { result } = await invalid(
    dedent`
      const big = 42n
      if (typeof big === 'bigint') {
        console.log(big.toString())
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on bigint variable', async () => {
  const { result } = await invalid(
    dedent`
      const big = 42n
      if (typeof big === 'number') {
        console.log(big)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('valid typeof check for null parameter that could be a different type', async () => {
  await valid(
    dedent`
      function test(value: any) {
        if (typeof value === 'object' && value === null) {
          console.log('value is null')
        }
      }
    `,
  )
})

test('unnecessary typeof check on null variable', async () => {
  const { result } = await invalid(
    dedent`
      const nullVar = null
      if (typeof nullVar === 'object') {
        console.log('null is an object in JavaScript')
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on null variable', async () => {
  const { result } = await invalid(
    dedent`
      const nullVar = null
      if (typeof nullVar === 'string') {
        console.log(nullVar)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('valid typeof check for undefined parameter that could be a different type', async () => {
  await valid(
    dedent`
      function test(value: any) {
        if (typeof value === 'undefined') {
          console.log('value is undefined')
        }
      }
    `,
  )
})

test('unnecessary typeof check on undefined variable', async () => {
  const { result } = await invalid(
    dedent`
      const undefinedVar: undefined = undefined
      if (typeof undefinedVar === 'undefined') {
        console.log('variable is undefined')
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check on undefined variable', async () => {
  const { result } = await invalid(
    dedent`
      const undefinedVar: undefined = undefined
      if (typeof undefinedVar === 'number') {
        console.log(undefinedVar)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

// Complex Union Types (more than two types)
test('valid typeof check for complex union type that includes the checked type', async () => {
  await valid(
    dedent`
      function test(value: string | number | boolean) {
        if (typeof value === 'boolean') {
          console.log(value ? 'true' : 'false')
        }
      }
    `,
  )
})

test('always false typeof check on complex union type', async () => {
  const { result } = await invalid(
    dedent`
      const complexUnion = 'test' as string | number | boolean
      if (typeof complexUnion === 'object') {
        console.log(complexUnion)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary typeof negation check on complex union type', async () => {
  const { result } = await invalid(
    dedent`
      const complexUnion = 'test' as string | number | boolean
      if (typeof complexUnion !== 'object') {
        console.log(complexUnion)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

// Optional Chaining With Typeof
test('valid typeof check with optional chaining on parameter that could be null/undefined', async () => {
  await valid(
    dedent`
      function test(obj: any) {
        if (typeof obj?.name === 'string') {
          console.log(obj.name.toUpperCase())
        }
      }
    `,
  )
})

test('unnecessary typeof check with optional chaining', async () => {
  const { result } = await invalid(
    dedent`
      const obj = { name: 'test' }
      if (typeof obj?.name === 'string') {
        console.log(obj.name)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('always false typeof check with optional chaining', async () => {
  const { result } = await invalid(
    dedent`
      const obj = { name: 'test' }
      if (typeof obj?.name === 'number') {
        console.log(obj.name)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('always false in check for missing property', async () => {
  const { result } = await invalid(
    dedent`
      const obj: { a: string } = { a: 's' }

      if ('b' in obj) {
        console.log(obj.a)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseInCondition', line: 3 }
    "
  `)
})

test('unnecessary in check for existing property', async () => {
  const { result } = await invalid(
    dedent`
      const obj: { a: string } = { a: 's' }

      if ('a' in obj) {
        console.log(obj.a)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryInCondition', line: 3 }
    "
  `)
})

test('always false in check for union object', async () => {
  const { result } = await invalid(
    dedent`
      const obj: { a: string } | { b: number } =
        Math.random() > 0.5 ? { a: 's' } : { b: 1 }

      if ('c' in obj) {
        console.log(obj)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseInCondition', line: 4 }
    "
  `)
})

test('valid in check for union object property', async () => {
  await valid(
    dedent`
      declare const obj: { a: string } | { b: number }

      if ('a' in obj) {
        console.log(obj)
      }
    `,
  )
})

test('unnecessary in check for intersection object', async () => {
  const { result } = await invalid(
    dedent`
      const obj: { a: string } & { b: number } = { a: 's', b: 1 }

      if ('a' in obj) {
        console.log(obj.a)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryInCondition', line: 3 }
    "
  `)
})

test('always false in check for intersection object', async () => {
  const { result } = await invalid(
    dedent`
      const obj: { a: string } & { b: number } = { a: 's', b: 1 }

      if ('c' in obj) {
        console.log(obj)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseInCondition', line: 3 }
    "
  `)
})

test('valid in check for optional property', async () => {
  await valid(
    dedent`
      const obj: { a?: string } = {}

      if ('a' in obj) {
        console.log(obj)
      }
    `,
  )
})

test('valid in check for record type', async () => {
  await valid(
    dedent`
      declare const obj: Record<string, string>

      if ('a' in obj) {
        console.log(obj)
      }
    `,
  )
})

// Logical Operators with Typeof
test('valid logical OR typeof checks on union type', async () => {
  const { result } = await invalid(
    dedent`
      function test(value: string | number) {
        if (typeof value === 'string' || typeof value === 'number') {
          console.log(value)
        }
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    "
  `)
})

test('unnecessary logical OR typeof checks on string variable', async () => {
  const { result } = await invalid(
    dedent`
      const str = 'hello'
      if (typeof str === 'string' || typeof str === 'number') {
        console.log(str)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('always false logical AND typeof checks on string variable', async () => {
  const { result } = await invalid(
    dedent`
      const str = 'hello'
      if (typeof str === 'string' && typeof str === 'number') {
        console.log(str)
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('non nullable typeof check', async () => {
  await valid(
    dedent`
      function test(value: unknown) {
        if (value && typeof value === 'string') {
          console.log(value)
        }
      }
    `,
  )
})

test('non nullable typeof check 2', async () => {
  await valid(
    dedent`
      function test(value: {}) {
        if (typeof value === 'string') {
          console.log(value)
        }
      }
    `,
  )
})

test('non nullable typeof check 3', async () => {
  const { result } = await invalid(
    dedent`
      function test(value: {}) {
        if (typeof value === 'undefined') {
          console.log(value)
        }
      }
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    "
  `)
})

test('argument with generic type', async () => {
  await valid(
    dedent`
      function test<T>(value: T) {
        if (typeof value === 'string') {
          console.log(value)
        }
      }
    `,
  )
})

describe('string assertions', () => {
  test('valid startsWith check on union type', async () => {
    await valid(
      dedent`
        function test(status: 'active' | 'inactive' | 'pending') {
          if (status.startsWith('act')) {
            console.log('starts with act')
          }
        }
      `,
    )
  })

  test('always false startsWith check on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' = 'active'
        if (status.startsWith('xyz')) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseStartsWithCondition', line: 2 }
      "
    `)
  })

  test('always true startsWith check on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' = 'active'
        if (status.startsWith('act')) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryStartsWithCondition', line: 2 }
      "
    `)
  })

  test('always false endsWith check on union type', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
        if (status.endsWith('xyz')) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseEndsWithCondition', line: 2 }
      "
    `)
  })

  test('always true endsWith check on union type', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
        if (status.endsWith('e')) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryEndsWithCondition', line: 2 }
      "
    `)
  })

  test('always false includes check on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const text: 'hello' = 'hello'
        if (text.includes('xyz')) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseIncludesCondition', line: 2 }
      "
    `)
  })

  test('always true includes check on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const text: 'hello' = 'hello'
        if (text.includes('ell')) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryIncludesCondition', line: 2 }
      "
    `)
  })

  test('valid includes check on union with mixed results', async () => {
    await valid(
      dedent`
        function test(text: 'hello' | 'world') {
          if (text.includes('o')) {
            console.log('some include o')
          }
        }
      `,
    )
  })

  test('always false length comparison on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const text: 'hello' = 'hello'
        if (text.length > 10) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseLengthCondition', line: 2 }
      "
    `)
  })

  test('always true length comparison on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const text: 'hello' = 'hello'
        if (text.length === 5) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryLengthCondition', line: 2 }
      "
    `)
  })

  test('always false length comparison with !== operator', async () => {
    const { result } = await invalid(
      dedent`
        const text: 'hello' = 'hello'
        if (text.length !== 5) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseLengthCondition', line: 2 }
      "
    `)
  })

  test('valid length comparison on union with different lengths', async () => {
    await valid(
      dedent`
        function test(text: 'hi' | 'hello') {
          if (text.length > 3) {
            console.log('some are longer than 3')
          }
        }
      `,
    )
  })

  test('always true length comparison on union with same lengths', async () => {
    const { result } = await invalid(
      dedent`
        const text: 'cat' | 'dog' = 'cat' as 'cat' | 'dog'
        if (text.length === 3) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryLengthCondition', line: 2 }
      "
    `)
  })

  test('string methods with empty string literal', async () => {
    const { result } = await invalid(
      dedent`
        const empty: '' = ''
        if (empty.startsWith('a')) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseStartsWithCondition', line: 2 }
      "
    `)
  })

  test('string methods with empty string always true case', async () => {
    const { result } = await invalid(
      dedent`
        const empty: '' = ''
        if (empty.startsWith('')) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryStartsWithCondition', line: 2 }
      "
    `)
  })

  test('valid string method on any type', async () => {
    await valid(
      dedent`
        function test(value: any) {
          if (value.startsWith && value.startsWith('test')) {
            console.log('might start with test')
          }
        }
      `,
    )
  })

  test('valid string method on unknown string type', async () => {
    await valid(
      dedent`
        function test(value: string) {
          if (value.startsWith('test')) {
            console.log('might start with test')
          }
        }
      `,
    )
  })

  test('valid optional chaining startsWith on possibly undefined', async () => {
    await valid(
      dedent`
        function test(text: string | undefined) {
          if (text?.startsWith('a')) {
            console.log('maybe starts with a')
          }
        }
      `,
    )
  })

  test('always false startsWith with optional chaining on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' = 'active'
        if (status?.startsWith('xyz')) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseStartsWithCondition', line: 2 }
      "
    `)
  })

  test('always true startsWith with optional chaining on literal string', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' = 'active'
        if (status?.startsWith('act')) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryStartsWithCondition', line: 2 }
      "
    `)
  })

  test('always false endsWith with optional chaining on union type', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
        if (status?.endsWith('xyz')) {
          console.log('never happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'alwaysFalseEndsWithCondition', line: 2 }
      "
    `)
  })

  test('always true endsWith with optional chaining on union type', async () => {
    const { result } = await invalid(
      dedent`
        const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
        if (status?.endsWith('e')) {
          console.log('always happens')
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - { messageId: 'unnecessaryEndsWithCondition', line: 2 }
      "
    `)
  })
})
