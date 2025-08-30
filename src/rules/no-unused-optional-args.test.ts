import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { noUnusedOptionalArgs } from './no-unused-optional-args'

const { valid, invalid } = createNewTester(noUnusedOptionalArgs)

test('exported function is ignored', async () => {
  await valid(
    dedent`
      export function f(a?: string, b?: string) {
        return "+" + a + b
      }
      f('x')
    `,
  )
})

test('unused args inside function are ignored', async () => {
  await valid(
    dedent`
      function f(a?: string, b?: string) {
        return "+" + a
      }
      f('x', 'y')
    `,
  )
})

test('skip when usage cannot be inferred', async () => {
  await valid(
    dedent`
      function f(a?: string, b?: string) { return '' + a + b }
      const g = f
      g('x', 'y')
    `,
  )
})

test('do not flag when at least one call provides arg', async () => {
  await valid(
    dedent`
      function f(a?: string, b?: string) { return '' + a + b }
      f('x')
      f('x', 'y')
    `,
  )
})

test('ignore params by regex', async () => {
  await valid({
    code: dedent`
      function f(_a?: string, b?: string) { return '' + _a + b }
      f('x', 'y')
    `,
    options: [{ ignoreArgsMatching: '^_' }],
  })
})

test('report param never passed by any call', async () => {
  const { result } = await invalid(
    dedent`
      function f(a?: string, b?: string) {
        return "+" + a + b
      }
      f('ok')
      f('test')
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unusedOptionalArg', line: 1 }
    "
  `)
})

test('report for arrow function variable', async () => {
  const { result } = await invalid(
    dedent`
      const f = (a?: number, b?: number) => a && b ? a + b : 0
      f(1)
      f(2)
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unusedOptionalArg', line: 1 }
    "
  `)
})

test('report optional object param never passed', async () => {
  const { result } = await invalid(
    dedent`
      function f(props?: { x: number }) { return props?.x ?? 0 }
      f()
      f()
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unusedOptionalArg', line: 1 }
    "
  `)
})

test('skip if any call uses spread args', async () => {
  await valid(
    dedent`
      function f(a?: string, b?: string) { return '' + a + b }
      const args = ['x', 'y'] as const
      f(...args)
    `,
  )
})

test('JSX usage counts as providing first arg', async () => {
  await valid(
    dedent`
      const Component = (props?: { x: number }) => null

      const App = () => {
        return <Component />
      }
    `,
  )
})

test('JSX usage with FC typed variable', async () => {
  await valid(
    dedent`
      const Component: FC<{ x: number }> = (props?: { x: number }) => null

      const App = () => <Component />
    `,
  )
})

test('report unused optional prop for JSX component', async () => {
  const { result } = await invalid(
    dedent`
      const Component = (props: { a?: string; b?: number }) => null

      const App = () => {
        return <Component a="x" />
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unusedOptionalProp', line: 1 }
    "
  `)
})

test('report unused optional prop for FC typed component', async () => {
  const { result } = await invalid(
    dedent`
      const Component: FC<{ a?: string; b?: number }> = () => null

      const App = () => <Component a="x" />
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unusedOptionalProp', line: 1 }
    "
  `)
})

test('do not report when prop provided somewhere', async () => {
  await valid(
    dedent`
      const Component = (props: { a?: string; b?: number }) => null

      const App = () => {
        return (
          <>
            <Component a="x" />
            <Component b={1} />
          </>
        )
      }
    `,
  )
})

test('bail when JSX spread props present', async () => {
  await valid(
    dedent`
      const Component = (props: { a?: string; b?: number }) => null

      const App = () => {
        const p = { a: 'x' }
        return <Component {...p} />
      }
    `,
  )
})

test('report unused optional prop in normal call', async () => {
  const { result } = await invalid(
    dedent`
      function withProps(props: { x?: number; b?: string }) {
        return null
      }

      withProps({ x: 1 })
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unusedOptionalProp', line: 1 }
    "
  `)
})

test('report unused optional prop in second param object', async () => {
  const { result } = await invalid(
    dedent`
      function f(a?: string, options?: { b?: string; c?: number }) {}

      f('ok', { b: 'ok' })
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unusedOptionalProp', line: 1 }
    "
  `)
})

test('bail when non-object arg is passed', async () => {
  await valid(
    dedent`
      function f(a?: string, options?: { b?: string; c?: number }) {}
      const opts = { b: 'x' }
      f('ok', opts)
    `,
  )
})

test('do not report when some calls provide missing prop', async () => {
  await valid(
    dedent`
      function f(a?: string, options?: { b?: string; c?: number }) {}
      f('ok', { b: 'ok' })
      f('ok', { c: 1 })
    `,
  )
})

test('pass function as argument to another function (skip)', async () => {
  await valid(
    dedent`
      function f(a?: string, b?: string) { return '' + a + b }

      function run(g: (x?: string, y?: string) => void) {
        g('a')
      }

      run(f)
    `,
  )
})

test('pass function as JSX prop (skip)', async () => {
  await valid(
    dedent`
      const f = (a?: number, b?: number) => {}
      const Btn = () => <button onClick={f} />
    `,
  )
})
