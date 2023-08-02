import { RuleTester } from '@typescript-eslint/rule-tester'
import { TSESLint } from '@typescript-eslint/utils'
import * as vitest from 'vitest'

RuleTester.afterAll = vitest.afterAll
RuleTester.it = vitest.it
RuleTester.itOnly = vitest.it.only
RuleTester.describe = vitest.describe

const ruleTester = new RuleTester({
  parser: '@typescript-eslint/parser',
  parserOptions: {
    tsconfigRootDir: './tests/fixture',
    project: './tsconfig.json',
  },
})
export function createTester<
  T extends TSESLint.RuleModule<string, any[]>,
  O extends any[],
>(
  rule: {
    name: string
    rule: T
  },
  { defaultErrorId }: { defaultErrorId?: string; optionsType?: O } = {},
) {
  return {
    valid(code: string, options?: O) {
      ruleTester.run(rule.name, rule.rule, {
        valid: [
          {
            code,
            options: options || [],
          },
        ],
        invalid: [],
      })
    },
    invalid(
      code: string,
      errors?:
        | {
            messageId?: string
            data?: Record<string, string>
          }[]
        | number,
      options?: O,
    ) {
      ruleTester.run(rule.name, rule.rule, {
        valid: [],
        invalid: [
          {
            code,
            options: options || [],
            errors:
              typeof errors === 'number'
                ? Array.from({ length: errors }, () => ({
                    messageId: defaultErrorId || '?',
                  }))
                : errors
                ? errors.map((error) => ({
                    messageId: error.messageId || defaultErrorId || '?',
                    data: error.data,
                  })) || []
                : [{ messageId: defaultErrorId || '?' }],
          },
        ],
      })
    },
  }
}
