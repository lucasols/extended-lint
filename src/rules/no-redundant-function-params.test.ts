import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { noRedundantFunctionParams } from './no-redundant-function-params'

const { valid, invalid } = createNewTester(noRedundantFunctionParams)

test('allows calls without redundant default parameters', async () => {
  await valid({
    code: dedent`
      inline({ align: 'left' })
      createElement(App, 'span')
      configure({ debug: true })
    `,
    options: [
      {
        functions: [
          { name: 'inline', defaults: [{ align: 'center' }] },
          { name: 'createElement', defaults: [undefined, 'div'] },
          { name: 'configure', defaults: [{ debug: false }] },
        ],
      },
    ],
  })
})

test('allows calls with no parameters when function has defaults', async () => {
  await valid({
    code: dedent`
      inline()
      createElement(App)
      configure()
    `,
    options: [
      {
        functions: [
          { name: 'inline', defaults: [{ align: 'center' }] },
          { name: 'createElement', defaults: [undefined, 'div'] },
          { name: 'configure', defaults: [{ debug: false }] },
        ],
      },
    ],
  })
})

test('allows calls with partial parameters', async () => {
  await valid({
    code: dedent`
      complex(1, { enabled: false })
      complex(2)
    `,
    options: [
      {
        functions: [
          { name: 'complex', defaults: [undefined, { enabled: true }, 'default'] },
        ],
      },
    ],
  })
})

test('disallows calls with simple default value', async () => {
  const { result } = await invalid({
    code: dedent`
      createElement(App, 'div')
    `,
    options: [
      {
        functions: [
          { name: 'createElement', defaults: [undefined, 'div'] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"createElement(App)"`)
})

test('disallows calls with object default value', async () => {
  const { result } = await invalid({
    code: dedent`
      inline({ align: 'center' })
    `,
    options: [
      {
        functions: [
          { name: 'inline', defaults: [{ align: 'center' }] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"inline()"`)
})

test('disallows calls with complex object default value', async () => {
  const { result } = await invalid({
    code: dedent`
      setup({
        theme: 'dark',
        features: { analytics: true, auth: false }
      })
    `,
    options: [
      {
        functions: [
          {
            name: 'setup',
            defaults: [
              {
                theme: 'dark',
                features: { analytics: true, auth: false },
              },
            ],
          },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"setup()"`)
})

test('disallows multiple redundant parameters', async () => {
  const { result } = await invalid({
    code: dedent`
      configure(true, 'dev', { verbose: false })
    `,
    options: [
      {
        functions: [
          { name: 'configure', defaults: [true, 'dev', { verbose: false }] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"configure()"`)
})

test('removes only trailing redundant parameters', async () => {
  const { result } = await invalid({
    code: dedent`
      advanced(1, 'default', true)
    `,
    options: [
      {
        functions: [
          { name: 'advanced', defaults: [undefined, 'default', true] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"advanced(1)"`)
})

test('handles mixed redundant and non-redundant parameters', async () => {
  const { result } = await invalid({
    code: dedent`
      mixed(42, 'custom', 'default')
    `,
    options: [
      {
        functions: [
          { name: 'mixed', defaults: [undefined, undefined, 'default'] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"mixed(42, 'custom')"`)
})

test('ignores functions not in configuration', async () => {
  await valid({
    code: dedent`
      unknownFunction({ align: 'center' })
      anotherFunction('div')
    `,
    options: [
      {
        functions: [
          { name: 'inline', defaults: [{ align: 'center' }] },
        ],
      },
    ],
  })
})

test('works with member expressions', async () => {
  const { result } = await invalid({
    code: dedent`
      obj.method('default')
    `,
    options: [
      {
        functions: [
          { name: 'method', defaults: ['default'] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"obj.method()"`)
})

test('handles boolean default values', async () => {
  const { result } = await invalid({
    code: dedent`
      toggle(false)
    `,
    options: [
      {
        functions: [
          { name: 'toggle', defaults: [false] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"toggle()"`)
})

test('handles null and undefined default values', async () => {
  const { result } = await invalid({
    code: dedent`
      nullable(null)
    `,
    options: [
      {
        functions: [
          { name: 'nullable', defaults: [null] },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'redundantParam', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`"nullable()"`)
})