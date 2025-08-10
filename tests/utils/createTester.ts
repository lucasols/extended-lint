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
    disableDedent,
  }: {
    defaultErrorId?: string
    disableDedent?: boolean
  } = {},
) {
  const valid: ValidTestCase<any[]>[] = []
  const invalid: InvalidTestCase<string, any[]>[] = []

  function run() {
    ruleTester.run(rule.name, rule.rule, {
      valid,
      invalid,
    })
  }

  type Options = T extends TSESLint.RuleModule<string, infer O> ? O : never

  function addValid(
    testName: string,
    code: string,
    ruleOptions?: Options extends [infer O] ? O | Options : Options,
    {
      fileName,
      skip = false,
      only = false,
    }: {
      fileName?: string
      skip?: boolean
      only?: boolean
    } = {},
  ) {
    if ((only || skip) && process.env.VITEST_MODE !== 'WATCH') {
      throw new Error('Only tests are not allowed in production')
    }

    valid.push({
      name: testName,
      code: disableDedent ? code : dedent(code),
      filename: fileName,
      options: ruleOptions
        ? Array.isArray(ruleOptions)
          ? ruleOptions
          : [ruleOptions]
        : rule.rule.defaultOptions,
      only: only || testName.startsWith('only:'),
      skip,
    })
  }

  function addInvalid(
    testName: string,
    code: string,
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
      options,
      skip = false,
      only = false,
    }: {
      skip?: boolean
      only?: boolean
      output?: string
      options?: Options extends [infer O] ? O | Options : Options
    } = {},
  ) {
    if ((only || skip) && process.env.VITEST_MODE !== 'WATCH') {
      throw new Error('Only tests are not allowed in production')
    }

    invalid.push({
      name: testName,
      only,
      skip,
      code: disableDedent ? code : dedent(code),
      output: output ? (disableDedent ? output : dedent(output)) : undefined,
      options: options
        ? Array.isArray(options)
          ? options
          : [options]
        : rule.rule.defaultOptions,
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
                suggestions: error.suggestions?.map((suggestion) => ({
                  ...suggestion,
                  output: disableDedent
                    ? suggestion.output
                    : dedent(suggestion.output),
                })),
              }),
            ),
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
      skip = false,
      only = false,
    }: {
      output?: string
      skip?: boolean
      only?: boolean
    } = {},
  ) {
    if ((only || skip) && process.env.VITEST_MODE !== 'WATCH') {
      throw new Error('Only tests are not allowed in production')
    }

    invalid.push({
      name: testName,
      only,
      skip,
      code: disableDedent ? code : dedent(code),
      output: output ? (disableDedent ? output : dedent(output)) : undefined,
      options: options
        ? Array.isArray(options)
          ? options
          : [options]
        : rule.rule.defaultOptions,
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
                suggestions: error.suggestions?.map((suggestion) => ({
                  ...suggestion,
                  output: disableDedent
                    ? suggestion.output
                    : dedent(suggestion.output),
                })),
              }),
            ),
    })
  }

  function describe(name: string, fn: () => void) {
    fn()
  }

  const skipAddInvalid: typeof addInvalid = (
    testName,
    code,
    errors,
    options,
  ) => {
    addInvalid(testName, code, errors, {
      skip: true,
      ...options,
    })
  }

  const skipAddInvalidWithOptions: typeof addInvalidWithOptions = (
    testName,
    code,
    options,
    errors,
    testOptions,
  ) => {
    addInvalidWithOptions(testName, code, options, errors, {
      skip: true,
      ...testOptions,
    })
  }

  const skipAddValid: typeof addValid = (
    testName,
    code,
    ruleOptions,
    options,
  ) => {
    addValid(testName, code, ruleOptions, {
      ...options,
      skip: true,
    })
  }

  const onlyAddValid: typeof addValid = (
    testName,
    code,
    ruleOptions,
    options,
  ) => {
    addValid(testName, code, ruleOptions, {
      ...options,
      only: true,
    })
  }

  const onlyAddInvalid: typeof addInvalid = (
    testName,
    code,
    errors,
    options,
  ) => {
    addInvalid(testName, code, errors, {
      only: true,
      ...options,
    })
  }

  const onlyAddInvalidWithOptions: typeof addInvalidWithOptions = (
    testName,
    code,
    options,
    errors,
    testOptions,
  ) => {
    addInvalidWithOptions(testName, code, options, errors, {
      only: true,
      ...testOptions,
    })
  }

  return {
    run,
    addValid,
    addInvalid,
    addInvalidWithOptions,
    describe,
    skip: {
      addValid: skipAddValid,
      addInvalid: skipAddInvalid,
      addInvalidWithOptions: skipAddInvalidWithOptions,
    },
    only: {
      addValid: onlyAddValid,
      addInvalid: onlyAddInvalid,
      addInvalidWithOptions: onlyAddInvalidWithOptions,
    },
  }
}
