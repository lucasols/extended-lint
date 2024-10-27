import { dedent } from '@ls-stack/utils/dedent'
import {
  InvalidTestCase,
  RuleTester,
  SuggestionOutput,
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

  function addValid(
    testName: string,
    code: string,
    ruleOptions?: Options extends [infer O] ? O | Options : Options,
    {
      fileName,
    }: {
      fileName?: string
    } = {},
  ) {
    valid.push({
      name: testName,
      code,
      filename: fileName,
      options: ruleOptions
        ? Array.isArray(ruleOptions)
          ? ruleOptions
          : [ruleOptions]
        : [],
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
      | 'default-error' = 'default-error',
    {
      output,
      options,
    }: {
      output?: string
      options?: Options extends [infer O] ? O | Options : Options
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
      options: options ? (Array.isArray(options) ? options : [options]) : [],
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

  function addInvalidWithOptions(
    testName: string,
    code: string,
    options: Options extends [infer O] ? O | Options : Options,
    errors:
      | {
          messageId?: string
          data?: Record<string, string>
          suggestions?: SuggestionOutput<string>[]
        }[]
      | number
      | 'default-error' = 'default-error',
    {
      output,
    }: {
      output?: string
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
      options: options ? (Array.isArray(options) ? options : [options]) : [],
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
                suggestions: error.suggestions,
              }),
            ) || [],
    })
  }

  function describe(name: string, fn: () => void) {
    fn()
  }

  return {
    run,
    addValid,
    addInvalid,
    addInvalidWithOptions,
    describe,
  }
}
