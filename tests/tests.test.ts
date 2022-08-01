import { Rule, RuleTester } from 'eslint'
import { expect, test } from 'vitest'
import { noWarningComment } from '../src/rules/no-warning-comment'

test('test?', () => {
  const ruleTester = new RuleTester()

  ruleTester.run(noWarningComment.name, noWarningComment.rule, {
    valid: [
      {
        code: '// @ts-ignore',
        options: [{ terms: ['TEST'] }],
      },
    ],
    invalid: [
      {
        code: '// TEST',
        options: [{ terms: ['TEST'] }],
        errors: [
          {
            message: `Unexpected 'TEST' comment: 'TEST'.`,
          },
        ],
      },
    ],
  })
})
