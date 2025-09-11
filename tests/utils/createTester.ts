 
import { dedent } from '@ls-stack/utils/dedent'
import { compactSnapshot } from '@ls-stack/utils/testUtils'
import typescriptParser from '@typescript-eslint/parser'
import {
  InvalidTestCase,
  RuleTester,
  SuggestionOutput,
  TestCaseError,
  ValidTestCase,
} from '@typescript-eslint/rule-tester'
import { TSESLint } from '@typescript-eslint/utils'
import {
  createRuleTester,
  type TestExecutionResult,
} from 'eslint-vitest-rule-tester'
import { fileURLToPath } from 'node:url'
import tseslint from 'typescript-eslint'

const newlineOnlyRegex = /^\n+$/

const ruleTester = new RuleTester({
  linterOptions: {
    reportUnusedDisableDirectives: 'off',
  },
  languageOptions: {
    parser: typescriptParser,
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

export function getErrorsFromResult(
  result: TestExecutionResult,
  include: {
    msg?: boolean
    column?: boolean
    endLine?: boolean
    endColumn?: boolean
  } = {},
) {
  return compactSnapshot(
    result.messages.map((m) => ({
      messageId: m.messageId,
      data: include.msg ? m.message : undefined,
      line: m.line,
      column: include.column ? m.column : undefined,
      endLine: include.endLine ? m.endLine : undefined,
      endColumn: include.endColumn ? m.endColumn : undefined,
    })),
  )
}

export function getErrorsWithMsgFromResult(result: TestExecutionResult) {
  return compactSnapshot(
    result.messages.map((m) => ({
      messageId: m.messageId,
      msg: m.message,
      line: m.line,
    })),
  )
}

export function createNewTester<
  M extends string,
  O extends readonly unknown[],
>(rule: { name: string; rule: TSESLint.RuleModule<M, O> }) {
  const parserOptions: TSESLint.ParserOptions = {
    tsconfigRootDir: fileURLToPath(new URL('../fixture', import.meta.url)),
    ecmaFeatures: {
      jsx: true,
    },
    ecmaVersion: 2020,
    sourceType: 'module',
    projectService: {
      allowDefaultProject: ['*.ts*'],
      defaultProject: './tsconfig.json',
      maximumDefaultProjectFileMatchCount_THIS_WILL_SLOW_DOWN_LINTING: 10000,
    },
  }
  return createRuleTester<O, M>({
    name: rule.name,
    rule: rule.rule,

    linterOptions: {
      reportUnusedDisableDirectives: 'off',
    },
    languageOptions: {
      parser: tseslint.parser,
      parserOptions,
    },
  })
}

/** @deprecated Use `createNewTester` instead */
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
      prependToOutput,
      appendToOutput,
      options,
      skip = false,
      only = false,
    }: {
      skip?: boolean
      only?: boolean
      output?: string
      appendToOutput?: string
      prependToOutput?: string
      options?: Options extends [infer O] ? O | Options : Options
    } = {},
  ) {
    if ((only || skip) && process.env.VITEST_MODE !== 'WATCH') {
      throw new Error('Only tests are not allowed in production')
    }

    if (
      prependToOutput &&
      prependToOutput !== '\n' &&
      !prependToOutput.match(newlineOnlyRegex)
    ) {
      throw new Error('prependToOutput must only contain line breaks')
    }

    if (
      appendToOutput &&
      appendToOutput !== '\n' &&
      !appendToOutput.match(newlineOnlyRegex)
    ) {
      throw new Error('appendToOutput must only contain line breaks')
    }

    let o = output ? (disableDedent ? output : dedent(output)) : undefined

    if (prependToOutput) {
      o = prependToOutput + o
    }

    if (appendToOutput) {
      o = o + appendToOutput
    }

    invalid.push({
      name: testName,
      only,
      skip,
      code: disableDedent ? code : dedent(code),
      output: o,
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
      prependToOutput,
      appendToOutput,
      skip = false,
      only = false,
    }: {
      output?: string
      skip?: boolean
      only?: boolean
      prependToOutput?: string
      appendToOutput?: string
    } = {},
  ) {
    if ((only || skip) && process.env.VITEST_MODE !== 'WATCH') {
      throw new Error('Only tests are not allowed in production')
    }

    if (
      appendToOutput &&
      appendToOutput !== '\n' &&
      !appendToOutput.match(newlineOnlyRegex)
    ) {
      throw new Error('appendToOutput must only contain line breaks')
    }

    let o = output ? (disableDedent ? output : dedent(output)) : undefined

    if (prependToOutput) {
      o = prependToOutput + o
    }

    if (appendToOutput) {
      o = o + appendToOutput
    }

    invalid.push({
      name: testName,
      only,
      skip,
      code: disableDedent ? code : dedent(code),
      output: o,
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

export function getSuggestionOutput(
  result: TestExecutionResult,
  index?: number,
) {
  const suggestion = result.messages[0]?.suggestions?.[index ?? 0]

  if (!suggestion) {
    throw new Error('No suggestion found')
  }

  const originalSource =
    result.steps.length > 0
      ? result.steps[0]?.output ?? result.output
      : result.output
  const fix = suggestion.fix

  const before = originalSource.slice(0, fix.range[0])
  const after = originalSource.slice(fix.range[1])

  return before + fix.text + after
}
