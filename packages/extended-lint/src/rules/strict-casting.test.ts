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
      return String(value)
    }"
  `)
})

test('unnecessary cast of enum or type parameter is not autofixed', async () => {
  const { result } = await invalid(dedent`
    enum SE { A = 'a', B = 'b' }
    enum NE { A, B }
    declare const member: SE

    let a = String(SE.A)
    const b = String(member)
    const c = Number(NE.B)
    const d = <T extends number>(x: T) => Number(x)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 5 }
    - { messageId: 'unnecessaryCasting', line: 6 }
    - { messageId: 'unnecessaryCasting', line: 7 }
    - { messageId: 'unnecessaryCasting', line: 8 }
    "
  `)
})

test('unnecessary cast of literal union is autofixed', async () => {
  const { result } = await invalid(dedent`
    declare const status: 'a' | 'b'

    const e = String(status)
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const status: 'a' | 'b'

    const e = status"
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

test('unnecessary cast fix keeps statements apart', async () => {
  const { result } = await invalid(dedent`
    declare const a: string
    declare const b: string
    declare const n: number
    declare function sideEffect(): void

    sideEffect()
    String(\`\${a}\`).split(',')
    sideEffect()
    String(a + b).length
    sideEffect()
    Number(-n).toFixed()
    sideEffect();
    String(a + b).length
    const f = () => String({} as unknown as string)
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const a: string
    declare const b: string
    declare const n: number
    declare function sideEffect(): void

    sideEffect()
    ;\`\${a}\`.split(',')
    sideEffect()
    ;(a + b).length
    sideEffect()
    ;(-n).toFixed()
    sideEffect();
    (a + b).length
    const f = () => ({} as unknown as string)"
  `)
})

test('unnecessary number literal cast fix keeps member access valid', async () => {
  const { result } = await invalid('const s = Number(1).toString()')

  expect(result.output).toMatchInlineSnapshot(`"const s = (1).toString()"`)
})

test('unnecessary cast with comments is not autofixed', async () => {
  const { result } = await invalid(dedent`
    declare const s: string

    const a = String(/* keep */ s)
    const b = String(s /* keep */)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 3 }
    - { messageId: 'unnecessaryCasting', line: 4 }
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "declare const s: string

    const a = String(/* keep */ s)
    const b = String(s /* keep */)"
  `)
})

test('shadowed builtin casts are ignored', async () => {
  await valid(dedent`
    export {}

    function Number(value: number) {
      return value + 1
    }

    declare const s: string | undefined
    const n = Number(41)
    const String = (value: string | undefined) => value ?? ''
    const t = String(s)
  `)
})

test('optional call of possibly missing cast function is ignored', async () => {
  await valid({
    code: dedent`
      declare const toNumber: ((value: unknown) => number) | undefined

      const n = toNumber?.(1)
    `,
    options: [
      {
        additionalCastFunctions: [{ name: 'toNumber', expectedType: 'number' }],
      },
    ],
  })
})

test('parenthesized arg and callee are looked through', async () => {
  const { result } = await invalid(dedent`
    declare const str: string

    const x = (String)((str))
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 3 }
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "declare const str: string

    const x = str"
  `)
})

test('parenthesized nullish arg suggestion', async () => {
  const { result } = await invalid(dedent`
    declare const a: string | null
    declare const b: string | null

    const y = String((a || b))
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishStringCasting', line: 4 }
    "
  `)
  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "declare const a: string | null
    declare const b: string | null

    const y = String(((a || b) ?? ''))"
  `)
})

test('parenthesized cast parent is looked through', async () => {
  const { result } = await invalid(dedent`
    declare const x: number
    declare const y: number

    const num = (Number(x + y))
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 4 }
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "declare const x: number
    declare const y: number

    const num = (x + y)"
  `)
})

test('optional chain arg is not a simple arg', async () => {
  const { result } = await invalid(dedent`
    declare const obj: { value: string } | undefined

    const a = String(obj?.value)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nullishStringCasting', line: 3 }
    "
  `)
  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "declare const obj: { value: string } | undefined

    const a = String((obj?.value) ?? '')"
  `)
})

test('optional call of cast function fix keeps parens', async () => {
  const { result } = await invalid(dedent`
    declare const x: number
    declare const y: number

    const num = Number?.(x + y)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryCasting', line: 4 }
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "declare const x: number
    declare const y: number

    const num = (x + y)"
  `)
})

test('unnecessary cast fix parens by parent', async () => {
  const { result } = await invalid(dedent`
    declare const x: number
    declare const y: number
    declare function fn(value: number): void
    let target = 0

    target = Number(x + y)
    fn(Number(x + y))
    const obj = { a: Number(x + y), [Number(x + y)]: 1 }
    const arr = [Number(x + y)]
    const arrow = () => Number(x + y)
    const neg = -Number(x + y)
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const x: number
    declare const y: number
    declare function fn(value: number): void
    let target = 0

    target = x + y
    fn(x + y)
    const obj = { a: x + y, [x + y]: 1 }
    const arr = [x + y]
    const arrow = () => x + y
    const neg = -(x + y)"
  `)
})

test('unnecessary cast fix in braceless bodies', async () => {
  const { result } = await invalid(dedent`
    declare const debug: boolean
    declare const a: string
    declare const b: string
    declare const n: number
    declare function log(value: string): void

    if (debug) String(a || b).split(',').forEach(log)
    else Number(-n).toFixed()
    while (debug) Number(-n)
    label: String(\`\${a}\`).trim()
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const debug: boolean
    declare const a: string
    declare const b: string
    declare const n: number
    declare function log(value: string): void

    if (debug) (a || b).split(',').forEach(log)
    else (-n).toFixed()
    while (debug) -n
    label: \`\${a}\`.trim()"
  `)
})

test('unnecessary cast fix keeps directives and export default valid', async () => {
  const { result } = await invalid({
    code: dedent`
      declare const s: unknown

      export function f(x = 1) {
        String('use strict')
        return x
      }
      const x = s
      String(<string>s)
      export default String(class {}.name)
    `,
    filename: 'assertion.ts',
  })

  expect(result.output).toMatchInlineSnapshot(`
    "declare const s: unknown

    export function f(x = 1) {
      ('use strict')
      return x
    }
    const x = s
    ;<string>s
    export default (class {}.name)"
  `)
})

test('Number cast of branded primitive or toPrimitive object', async () => {
  await valid(dedent`
    declare const id: string & { __brand: 'id' }
    declare const count: number & { __brand: 'count' }
    declare const temperature: { [Symbol.toPrimitive](hint: string): number }

    const a = Number(id)
    const b = Number(count)
    const c = Number(temperature)
  `)
})

test('comment-like text in a type argument keeps the fix', async () => {
  const { result } = await invalid({
    code: dedent`
      declare function toStr<T>(value: unknown): string
      declare const s: string

      const x = toStr<'http://x'>(s)
    `,
    options: [
      { additionalCastFunctions: [{ name: 'toStr', expectedType: 'string' }] },
    ],
  })

  expect(result.output).toMatchInlineSnapshot(`
    "declare function toStr<T>(value: unknown): string
    declare const s: string

    const x = s"
  `)
})

test('values that may hold primitives are not NaN', async () => {
  await valid(dedent`
    declare const empty: {}
    declare const obj: Object
    declare const bool: Boolean
    declare const custom: { toString(): string }
    declare const valueOfString: { valueOf(): string }

    function narrowed(value: unknown) {
      if (value != null) return Number(value)
    }
    function generic<T extends {}>(value: T) {
      return Number(value)
    }
    const a = Number(empty)
    const b = Number(obj)
    const c = Number(bool)
    const d = Number(custom)
    const e = Number(valueOfString)
  `)
})

test('toPrimitive returning a symbol is invalid', async () => {
  const { result } = await invalid(dedent`
    declare const weird: { [Symbol.toPrimitive](): symbol }
    const a = Number(weird)
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'nanNumberCasting', line: 2 }
    "
  `)
})

test('optional call of an any cast function is ignored', async () => {
  await valid({
    code: dedent`
      const toNumber: any = undefined
      let calls = 0
      const a = toNumber?.(++calls)
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

test('ASI guard ignores JSDoc tokens', async () => {
  const { result } = await invalid(dedent`
    declare const n: number
    const x = 1
    /** @type {{ */
    Number(-n)
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "declare const n: number
    const x = 1
    /** @type {{ */
    ;-n"
  `)
})
