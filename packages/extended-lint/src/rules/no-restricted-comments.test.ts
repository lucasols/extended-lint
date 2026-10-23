import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
  getSuggestionOutput,
} from '../../tests/utils/createTester'
import { noRestrictedComments } from './no-restricted-comments'

const { valid, invalid } = createNewTester(noRestrictedComments)

test('valid: no patterns configured', async () => {
  await valid({
    code: dedent`
      // This is a comment
      const x = 1
    `,
    options: [{ patterns: [] }],
  })
})

test('valid: comment does not match any pattern', async () => {
  await valid({
    code: dedent`
      // This is a regular comment
      const x = 1
    `,
    options: [{ patterns: [{ includes: 'TODO:' }] }],
  })
})

test('invalid: line comment matches includes pattern', async () => {
  const { result } = await invalid({
    code: dedent`
      // TODO: fix this later
      const x = 1
    `,
    options: [{ patterns: [{ includes: 'TODO:' }] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedComment', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "
    const x = 1"
  `)
})

test('invalid: line comment matches includes pattern with custom message', async () => {
  const { result } = await invalid({
    code: dedent`
      // FIXME: broken code
      const x = 1
    `,
    options: [
      {
        patterns: [
          { includes: 'FIXME:', message: 'FIXMEs must be resolved before merge' },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedCommentWithMessage', line: 1 }
    "
  `)
})

test('invalid: line comment matches regex pattern', async () => {
  const { result } = await invalid({
    code: dedent`
      // eslint-disable-next-line
      const x = 1
    `,
    options: [{ patterns: [{ regex: '^\\s*eslint-disable' }] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedComment', line: 1 }
    "
  `)
})

test('invalid: block comment matches includes pattern', async () => {
  const { result } = await invalid({
    code: dedent`
      /* TODO: implement this */
      const x = 1
    `,
    options: [{ patterns: [{ includes: 'TODO:' }] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedComment', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "
    const x = 1"
  `)
})

test('invalid: multiple patterns, only one matches', async () => {
  const { result } = await invalid({
    code: dedent`
      // HACK: workaround for bug
      const x = 1
    `,
    options: [
      {
        patterns: [
          { includes: 'TODO:' },
          { includes: 'FIXME:' },
          { includes: 'HACK:' },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedComment', line: 1 }
    "
  `)
})

test('invalid: multiple comments match patterns', async () => {
  const { result } = await invalid({
    code: dedent`
      // TODO: first task
      const x = 1
      // FIXME: second issue
      const y = 2
    `,
    options: [
      {
        patterns: [{ includes: 'TODO:' }, { includes: 'FIXME:' }],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedComment', line: 1 }
    - { messageId: 'restrictedComment', line: 3 }
    "
  `)
})

test('valid: regex pattern does not match', async () => {
  await valid({
    code: dedent`
      // eslint is great
      const x = 1
    `,
    options: [{ patterns: [{ regex: '^\\s*eslint-disable' }] }],
  })
})

test('invalid: autoFix removes comment automatically', async () => {
  const { result } = await invalid({
    code: dedent`
      // TODO: fix this
      const x = 1
    `,
    options: [{ patterns: [{ includes: 'TODO:', autoFix: true }] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedComment', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "
    const x = 1"
  `)
})

test('invalid: autoFix with custom message', async () => {
  const { result } = await invalid({
    code: dedent`
      // FIXME: broken
      const x = 1
    `,
    options: [
      {
        patterns: [
          { includes: 'FIXME:', message: 'Fix this before merge', autoFix: true },
        ],
      },
    ],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'restrictedCommentWithMessage', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "
    const x = 1"
  `)
})
