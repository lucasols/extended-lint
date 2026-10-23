import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
  getErrorsWithMsgFromResult,
  getSuggestionOutput,
} from '../../tests/utils/createTester'
import { strictCasting } from './strict-casting'

const { valid, invalid } = createNewTester(strictCasting)

test('valid casts between different types', async () => {
  await valid(
    dedent`
      declare const num: number
      declare const bool: boolean
      declare const str: string

      const a = String(num)
      const b = String(bool)
      const c = Number(str)
      const d = Number(bool)
    `,
  )
})

test('Number on object with numeric valueOf is valid', async () => {
  await valid(
    dedent`
      declare const date: Date

      const timestamp = Number(date)
    `,
  )
})

test('any and unknown casts', async () => {
  const { result } = await invalid(dedent`
    declare const unknownValue: unknown
    declare function getAny(): any

    const a = String(unknownValue)
    const b = Number(getAny())
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unknownCasting', line: 4 }
    - { messageId: 'anyCasting', line: 5 }
    "
  `)
})

test('any and unknown casts are allowed with options', async () => {
  await valid({
    code: dedent`
      declare const unknownValue: unknown
      declare function getAny(): any

      const a = String(unknownValue)
      const b = Number(getAny())
    `,
    options: [{ allowAnyCasting: true, allowUnknownCasting: true }],
  })
})

test('any and unknown checks do not apply to custom cast functions', async () => {
  await valid({
    code: dedent`
      declare function toNumber(val: string | number): number

      declare const anyValue: any

      const result = toNumber(anyValue)
    `,
    options: [
      {
        additionalCastFunctions: [
          { name: 'toNumber', expectedType: 'number' },
        ],
      },
    ],
  })
})

test('calls with no args or multiple args are skipped', async () => {
  await valid(
    dedent`
      const a = String()
      const b = Number()
    `,
  )
})

test('nullish casting is allowed with allowNullishCasting', async () => {
  await valid({
    code: dedent`
      declare const value: string | null

      const a = String(value)
      const b = Number(value)
    `,
    options: [{ allowNullishCasting: true }],
  })
})

test('invalid casting is allowed with allowInvalidCasting', async () => {
  await valid({
    code: dedent`
      declare const obj: { a: string }
      declare const sym: symbol

      const a = Number(obj)
      const b = Number(sym)
    `,
    options: [{ allowInvalidCasting: true }],
  })
})

test('custom cast function with different type is valid', async () => {
  await valid({
    code: dedent`
      declare function toNumber(val: string | number): number

      declare const str: string

      const result = toNumber(str)
    `,
    options: [
      {
        additionalCastFunctions: [
          { name: 'toNumber', expectedType: 'number' },
        ],
      },
    ],
  })
})

test('unnecessary String cast on string', async () => {
  const { result } = await invalid(dedent`
    declare const str: string

    const a = String(str)
    const b = String('literal')
    const c = String(\`template \${str}\`)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 3 }
    - { messageId: 'unnecessaryCasting', line: 4 }
    - { messageId: 'unnecessaryCasting', line: 5 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const str: string

    const a = str
    const b = 'literal'
    const c = \`template \${str}\`"
  `)
})

test('unnecessary Number cast on number expression', async () => {
  const { result } = await invalid(dedent`
    declare const x: number
    declare const y: number

    const num = Number(x + y)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 4 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const x: number
    declare const y: number

    const num = x + y"
  `)
})

test('unnecessary cast on generic with string constraint', async () => {
  const { result } = await invalid(dedent`
    function toStr<T extends string>(value: T): string {
      return String(value)
    }
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 2 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "function toStr<T extends string>(value: T): string {
      return value
    }"
  `)
})

test('unnecessary cast fix adds parens when needed', async () => {
  const { result } = await invalid(dedent`
    declare const cond: boolean
    declare const a: string
    declare const b: string

    const len = String(cond ? a : b).length
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 5 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const cond: boolean
    declare const a: string
    declare const b: string

    const len = (cond ? a : b).length"
  `)
})

test('nullish String cast', async () => {
  const { result } = await invalid(dedent`
    declare const value: string | null

    const str = String(value)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishStringCasting', line: 3 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "declare const value: string | null

    const str = String(value ?? '')"
  `)
})

test('nullish String cast on optional value', async () => {
  const { result } = await invalid(dedent`
    function format(value?: number) {
      return String(value)
    }
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishStringCasting', line: 2 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "function format(value?: number) {
      return String(value ?? '')
    }"
  `)
})

test('nullish String cast suggestion adds parens when needed', async () => {
  const { result } = await invalid(dedent`
    declare const a: string | null
    declare const b: string | null

    const str = String(a || b)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishStringCasting', line: 4 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "declare const a: string | null
    declare const b: string | null

    const str = String((a || b) ?? '')"
  `)
})

test('cast of pure null literal', async () => {
  const { result } = await invalid(dedent`
    const str = String(null)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishStringCasting', line: 1 }
    "
  `)
})

test('nullish Number cast', async () => {
  const { result } = await invalid(dedent`
    declare const value: number | undefined

    const num = Number(value)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishNumberCasting', line: 3 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "declare const value: number | undefined

    const num = Number(value ?? 0)"
  `)
})

test('Number cast of symbol', async () => {
  const { result } = await invalid(dedent`
    declare const sym: symbol

    const num = Number(sym)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'symbolNumberCasting', line: 3 }
    "
  `)
})

test('Number cast of objects without numeric valueOf', async () => {
  const { result } = await invalid(dedent`
    declare const obj: { a: string }
    declare const arr: number[]
    declare const fn: () => void

    const a = Number(obj)
    const b = Number(arr)
    const c = Number(fn)
  `)

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'nanNumberCasting'
      msg: 'Number() cast of a value of type "{ a: string; }" will result in NaN'
      line: 5
    - messageId: 'nanNumberCasting'
      msg: 'Number() cast of a value of type "number[]" will result in NaN'
      line: 6
    - messageId: 'nanNumberCasting'
      msg: 'Number() cast of a value of type "() => void" will result in NaN'
      line: 7
    "
  `)
})

test('unnecessary custom cast function', async () => {
  const { result } = await invalid({
    code: dedent`
      declare function toNumber(val: string | number): number

      declare const num: number

      const result = toNumber(num)
    `,
    options: [
      {
        additionalCastFunctions: [
          { name: 'toNumber', expectedType: 'number' },
        ],
      },
    ],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'unnecessaryCasting'
      msg: 'Unnecessary toNumber() cast of a value that is already of type number'
      line: 5
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare function toNumber(val: string | number): number

    declare const num: number

    const result = num"
  `)
})

test('nullish check does not apply to custom cast functions', async () => {
  await valid({
    code: dedent`
      declare function toNumber(val: string | number | null): number

      declare const value: string | null

      const result = toNumber(value)
    `,
    options: [
      {
        additionalCastFunctions: [
          { name: 'toNumber', expectedType: 'number' },
        ],
      },
    ],
  })
})

test('nullish String cast with object in union', async () => {
  const { result } = await invalid(dedent`
    declare const value: { a: string } | undefined

    const str = String(value)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishStringCasting', line: 3 }
    "
  `)
})
