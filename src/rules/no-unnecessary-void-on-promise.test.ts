import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
  getSuggestionOutput,
} from '../../tests/utils/createTester'
import { noUnnecessaryVoidOnPromise } from './no-unnecessary-void-on-promise'

const { valid, invalid } = createNewTester(noUnnecessaryVoidOnPromise)

test('void on async function call', async () => {
  await valid(
    dedent`
      async function fetchData() {
        return await fetch('/api/data')
      }

      void fetchData()
    `,
  )
})

test('void on Promise-returning function', async () => {
  await valid(
    dedent`
      function getPromise(): Promise<void> {
        return Promise.resolve()
      }

      void getPromise()
    `,
  )
})

test('void on fetch call', async () => {
  await valid(
    dedent`
      void fetch('/api/data')
    `,
  )
})

test('void on Promise.resolve', async () => {
  await valid(
    dedent`
      void Promise.resolve()
    `,
  )
})

test('void on method returning Promise', async () => {
  await valid(
    dedent`
      const api = {
        fetchData(): Promise<string> {
          return Promise.resolve('data')
        }
      }

      void api.fetchData()
    `,
  )
})

test('void on synchronous function call', async () => {
  const { result } = await invalid(
    dedent`
      function syncFn() {
        return 'hello'
      }

      void syncFn()
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryVoid', line: 5 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "function syncFn() {
      return 'hello'
    }

    syncFn()"
  `)
})

test('void on console.log', async () => {
  const { result } = await invalid(
    dedent`
      void console.log('test')
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryVoid', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "console.log('test')"
  `)
})

test('void on Math.max', async () => {
  const { result } = await invalid(
    dedent`
      void Math.max(1, 2, 3)
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryVoid', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "Math.max(1, 2, 3)"
  `)
})

test('void on function returning number', async () => {
  const { result } = await invalid(
    dedent`
      function getNumber(): number {
        return 42
      }

      void getNumber()
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryVoid', line: 5 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "function getNumber(): number {
      return 42
    }

    getNumber()"
  `)
})

test('void on arrow function call', async () => {
  const { result } = await invalid(
    dedent`
      const greet = () => 'hello'

      void greet()
    `,
  )

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryVoid', line: 3 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const greet = () => 'hello'

    greet()"
  `)
})

test('void on variable (not a function call) is ignored', async () => {
  await valid(
    dedent`
      const value = 123
      void value
    `,
  )
})

test('void on literal is ignored', async () => {
  await valid(
    dedent`
      void 0
    `,
  )
})

test('void on object property access is ignored', async () => {
  await valid(
    dedent`
      const obj = { prop: 123 }
      void obj.prop
    `,
  )
})

test('void with Promise.all', async () => {
  await valid(
    dedent`
      async function fn1() {}
      async function fn2() {}

      void Promise.all([fn1(), fn2()])
    `,
  )
})

test('void with chained then', async () => {
  await valid(
    dedent`
      function getPromise(): Promise<number> {
        return Promise.resolve(42)
      }

      void getPromise().then(x => x + 1)
    `,
  )
})
