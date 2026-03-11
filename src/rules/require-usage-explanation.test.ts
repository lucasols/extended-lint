import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { requireUsageExplanation } from './require-usage-explanation'

const { valid, invalid } = createNewTester(requireUsageExplanation)

test('valid: fn call with line comment prefix above', async () => {
  await valid({
    code: dedent`
      // WORKAROUND: legacy API requires this call
      dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })
})

test('valid: fn call with block comment prefix above', async () => {
  await valid({
    code: dedent`
      /* WORKAROUND: legacy API requires this call */
      dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })
})

test('valid: fn call inside variable declaration with comment above', async () => {
  await valid({
    code: dedent`
      // WORKAROUND: needed for backwards compatibility
      const result = dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })
})

test('valid: no matching function call', async () => {
  await valid({
    code: dedent`
      otherFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })
})

test('valid: selector match with correct comment above', async () => {
  await valid({
    code: dedent`
      // CAST: needed for external library compatibility
      const x = value as string
    `,
    options: [
      {
        matches: [{ selector: 'TSAsExpression', commentPrefix: 'CAST:' }],
      },
    ],
  })
})

test('valid: member expression fn call with comment above', async () => {
  await valid({
    code: dedent`
      // WORKAROUND: required by the SDK
      obj.dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })
})

test('invalid: fn call with no comment', async () => {
  const { result } = await invalid({
    code: dedent`
      dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 1 }
    "
  `)
})

test('invalid: fn call with wrong prefix', async () => {
  const { result } = await invalid({
    code: dedent`
      // NOTE: some note
      dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 2 }
    "
  `)
})

test('invalid: fn call with prefix only, no explanation text', async () => {
  const { result } = await invalid({
    code: dedent`
      // WORKAROUND:
      dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 2 }
    "
  `)
})

test('invalid: selector match with no comment', async () => {
  const { result } = await invalid({
    code: dedent`
      const x = value as string
    `,
    options: [
      {
        matches: [{ selector: 'TSAsExpression', commentPrefix: 'CAST:' }],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 1 }
    "
  `)
})

test('invalid: multiple configs, one missing comment', async () => {
  const { result } = await invalid({
    code: dedent`
      // WORKAROUND: needed
      dangerousFn()
      unsafeFn()
    `,
    options: [
      {
        matches: [
          { fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' },
          { fn: 'unsafeFn', commentPrefix: 'HACK:' },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 3 }
    "
  `)
})

test('invalid: member expression fn call with no comment', async () => {
  const { result } = await invalid({
    code: dedent`
      obj.dangerousFn()
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 1 }
    "
  `)
})

test('invalid: fn call with custom message appended', async () => {
  const { result } = await invalid({
    code: dedent`
      dangerousFn()
    `,
    options: [
      {
        matches: [
          {
            fn: 'dangerousFn',
            commentPrefix: 'WORKAROUND:',
            message: 'This function has security implications.',
          },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 1 }
    "
  `)
})

test('valid: comment above return statement containing fn call', async () => {
  await valid({
    code: dedent`
      function foo() {
        // WORKAROUND: needed for edge case
        return dangerousFn()
      }
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })
})

test('invalid: fn call inside return with no comment', async () => {
  const { result } = await invalid({
    code: dedent`
      function foo() {
        return dangerousFn()
      }
    `,
    options: [
      {
        matches: [{ fn: 'dangerousFn', commentPrefix: 'WORKAROUND:' }],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'default', line: 2 }
    "
  `)
})
