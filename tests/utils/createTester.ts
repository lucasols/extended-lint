import { ESLintUtils } from '@typescript-eslint/utils'
import { RuleModule } from '@typescript-eslint/utils/dist/ts-eslint'

const ruleTester = new ESLintUtils.RuleTester({
  parser: '@typescript-eslint/parser',
  parserOptions: {
    createDefaultProgram: true,
  },
})
export function createTester<T extends RuleModule<string, any[]>>(
  rule: {
    name: string
    rule: T
  },
  defaultErrorId?: string,
) {
  return {
    valid(code: string) {
      ruleTester.run(rule.name, rule.rule, {
        valid: [
          {
            code,
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
    ) {
      ruleTester.run(rule.name, rule.rule, {
        valid: [],
        invalid: [
          {
            code,
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
