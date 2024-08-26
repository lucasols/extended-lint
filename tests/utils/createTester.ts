import { dedent } from '@ls-stack/utils/dedent'
import {
  InvalidTestCase,
  RuleTester,
  TestCaseError,
  ValidTestCase,
} from '@typescript-eslint/rule-tester'
import { TSESLint } from '@typescript-eslint/utils'
import { fileURLToPath } from 'node:url'

const ruleTester = new RuleTester({
  linterOptions: {
    reportUnusedDisableDirectives: 'off',
  },
  languageOptions: {
    parserOptions: {
      tsconfigRootDir: fileURLToPath(new URL('../fixture', import.meta.url)),
      project: './tsconfig.json',
      ecmaFeatures: {
        jsx: true,
      },
      ecmaVersion: 2020,
      sourceType: 'module',
    },
  },
})

export function createTester<T extends TSESLint.RuleModule<string, any[]>>(
  rule: {
    name: string
    rule: T
  },
  {
    defaultErrorId,
  }: {
    defaultErrorId?: string
  } = {},
) {
  const valid: ValidTestCase<any[]>[] = []
  const invalid: InvalidTestCase<string, any[]>[] = []

  function run() {
    ruleTester.run(rule.name, rule.rule, {
      valid: valid,
      invalid: invalid,
    })
  }

  type Options = T extends TSESLint.RuleModule<string, infer O> ? O : never

  function addValid(testName: string, code: string, options?: Options) {
    valid.push({
      name: testName,
      code,
      options: options || [],
      only: testName.startsWith('only:'),
    })
  }

  function addInvalid(
    testName: string,
    code: string,
    errors:
      | {
          messageId?: string
          data?: Record<string, string>
        }[]
      | number
      | 'default-error',
    {
      output,
      options,
    }: {
      output?: string
      options?: Options
    } = {},
  ) {
    const only = testName.startsWith('only:')

    if (only && process.env.VITEST_MODE !== 'WATCH') {
      throw new Error('Only tests are not allowed in production')
    }

    invalid.push({
      name: testName,
      only: only,
      code: dedent(code),
      output: output ? dedent(output) : undefined,
      options: options || [],
      errors:
        errors === 'default-error'
          ? [{ messageId: defaultErrorId || '?' }]
          : typeof errors === 'number'
          ? Array.from({ length: errors }, () => ({
              messageId: defaultErrorId || '?',
            }))
          : errors.map(
              (error): TestCaseError<string> => ({
                messageId: error.messageId || defaultErrorId || '?',
                data: error.data,
              }),
            ) || [],
    })
  }

  return {
    run,
    addValid,
    addInvalid,
  }
}
