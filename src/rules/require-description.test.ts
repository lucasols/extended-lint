import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { requireDescription } from './require-description'

const { valid, invalid } = createNewTester(requireDescription)

test('eslint and oxlint directives with descriptions', async () => {
  await valid(
    dedent`
      // eslint-disable-next-line no-console -- logs the dev tools output
      console.log(1)
      // oxlint-disable-next-line no-console -- logs the dev tools output
      console.log(2)
      console.log(3) // oxlint-disable-line no-console -- logs the dev tools output
      /* oxlint-disable no-console -- dev script */
      /* oxlint-enable no-console -- dev script */
    `,
  )
})

test('comments that are not directives', async () => {
  await valid(
    dedent`
      // oxlint is used to lint the project
      // the oxlint-disable comments are checked
      const value = 1
    `,
  )
})

test('eslint and oxlint directives without descriptions', async () => {
  const { result } = await invalid(
    dedent`
      // eslint-disable-next-line no-console
      console.log(1)
      // oxlint-disable-next-line no-console
      console.log(2)
      console.log(3) // oxlint-disable-line no-console
      /* oxlint-disable no-console */
      /* oxlint-enable no-console */
      /* oxlint-disable */
    `,
  )

  expect(getErrorsFromResult(result, { column: true })).toMatchInlineSnapshot(`
    "
    - { messageId: 'missingDescription', line: 1, column: 1 }
    - { messageId: 'missingDescription', line: 3, column: 1 }
    - { messageId: 'missingDescription', line: 5, column: 1 }
    - { messageId: 'missingDescription', line: 6, column: 1 }
    - { messageId: 'missingDescription', line: 7, column: 1 }
    - { messageId: 'missingDescription', line: 8, column: 1 }
    "
  `)
})

test('ignored oxlint directive kinds', async () => {
  await valid({
    code: dedent`
      // oxlint-disable-next-line no-console
      console.log(1)
      /* oxlint-disable no-console */
    `,
    options: [{ ignore: ['oxlint-disable-next-line', 'oxlint-disable'] }],
  })
})
