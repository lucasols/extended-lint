export type Severity = 'off' | 'warn' | 'error' | 0 | 1 | 2

type RuleEntry<Options> = Severity | [Severity] | [Severity, Options]

export type ImprovedNoUnnecessaryConditionOptions = {
  alternativeForInCondition?: string
}

export type StrictCastingOptions = {
  additionalCastFunctions?: {
    name: string
    expectedType: 'string' | 'number'
  }[]
  allowNullishCasting?: boolean
  allowInvalidCasting?: boolean
  allowAnyCasting?: boolean
  allowUnknownCasting?: boolean
}

export type NoLeakedConditionalRenderingOptions = {
  reactVersion?: string
}

export type NamingConventionCheck =
  | boolean
  | {
      disallow?: string
      message?: string
    }

export type NamingConventionOptions = {
  typeNames?: NamingConventionCheck
  values?: NamingConventionCheck
}

type LiteralOptionValue = string | number | boolean

export type NoRestrictedSyntaxOptions = {
  disallow?: {
    selector: string
    message: string
    replace?: string | { regex: string; with: string }
    replaceType?: 'suggestion' | 'autofix'
  }[]
  disallowFnCalls?: {
    fn: string
    withArgs?: { atIndex: number; value: LiteralOptionValue }[]
    message: string
    replaceWith?: string
    ignoreRegex?: string
  }[]
  mustMatchSyntax?: {
    includeRegex: string
    excludeRegex?: string
    mustCallFn?: {
      anyCall: {
        fn: string
        withArgs: { atIndex: number; literal: LiteralOptionValue }[]
      }[]
      message?: string
    }[]
    mustMatchSelector?: { selector: string; message: string }[]
    mustHaveExport?: {
      name: string
      type?: 'function' | 'variable' | 'any'
      message: string
    }[]
  }[]
  __dev_simulateFileName?: string
}

export type NoRelativeImportsOptions = {
  aliases: { find: string; replacement: string }[]
  rootDir?: string
  allowNotFoundAliases?: boolean
  _dev_simulateFileName?: string
}

export type NoStaticStylePropOptions = {
  customMessage?: string
}

export type PreferNamedFunctionsOptions = {
  ignoreRegex?: string
  disallowArrowFnWithImplicitReturns?: boolean
}

export type NoCallWithExplicitGenericsOptions = {
  functions: (string | { name: string; message: string })[]
}

export type NoCallWithInferredGenericsOptions = {
  functions: {
    name: string
    minGenerics?: number
    allowAny?: boolean
    disallowTypes?: string[]
    disallowTypeOf?: boolean
    disallowUnknown?: boolean
  }[]
  disallowTypes?: string[]
  anyAliases?: string[]
}

export type NoUnnecessaryTypingOptions = {
  methods?: string[] | 'array'
}

export type NoTypeGuardsOptions = {
  alternativeMsgs?: {
    inArrayFind?: string
    inArrayFilter?: string
  }
}

export type UseStateSetterNamingOptions = {
  ignorePrefixes?: string[]
}

export type RequireLazyExecutionOptions = {
  functions?: string[]
}

export type RequireDescriptionDirective =
  | 'eslint'
  | 'eslint-disable'
  | 'eslint-disable-line'
  | 'eslint-disable-next-line'
  | 'eslint-enable'
  | 'eslint-env'
  | 'oxlint-disable'
  | 'oxlint-disable-line'
  | 'oxlint-disable-next-line'
  | 'oxlint-enable'
  | 'fast-lint-disable'
  | 'fast-lint-disable-line'
  | 'fast-lint-disable-next-line'
  | 'fast-lint-enable'
  | 'exported'
  | 'global'
  | 'globals'

export type RequireDescriptionOptions = {
  ignore?: RequireDescriptionDirective[]
}

export type ReactSingleExportOptions = {
  extensions?: string[]
}

export type NoUnusedOptionalArgsOptions = {
  ignoreArgsMatching?: string
}

export type NoUnusedTypePropsInArgsOptions = {
  forceCheckOnFCPropTypesWithName?: string[]
  alwaysCheckFunctionOptionTypes?: boolean
}

export type NoUnusedSelectedValuesOptions = {
  selectors: {
    name: string
    selectorProp?: string
    selectorArgPos?: number
    returnProp?: string
  }[]
}

export type NoRedundantFunctionParamsOptions = {
  functions: { name: string; defaults: unknown[] }[]
}

export type ReactNoModuleLevelSingleUseValuesOptions = {
  ignoreConstCaseVarRegex?: string
}

export type RequireReadsToVarPropOptions = {
  varsToCheck: {
    selector?: string
    fromFnCall?: string
    prop: string | string[]
    errorMsg?: string
  }[]
}

export type PreferReactHookAlternativeOptions = {
  disallowedFunctions: {
    name: string
    allowUsingWithArgs?: boolean
    hookAlternative?: string
    message?: string
    allowUseInside?: string[]
  }[]
}

export type SortReactComponentsAndStylesOptions = {
  mainComponentRegex?: string
  mainComponentSelector?: string
}

export type UseTypesDirectlyAboveUsageOptions = {
  checkOnly?: ('function-args' | 'FC' | 'generic-args-at-fn-calls')[]
  checkTypesFromSelectors?: string[]
}

export type ReactCompilerExtraOptions = {
  runOnlyWithEnableCompilerDirective?: boolean
}

type RuleOptions = {
  'improved-no-unnecessary-condition': ImprovedNoUnnecessaryConditionOptions
  'strict-casting': StrictCastingOptions
  'no-unnecessary-void-on-promise': Record<string, never>
  'no-leaked-conditional-rendering': NoLeakedConditionalRenderingOptions
  'naming-convention': NamingConventionOptions
  'no-restricted-syntax': NoRestrictedSyntaxOptions
  'advanced-no-restricted-syntax': NoRestrictedSyntaxOptions
  'no-default-export': Record<string, never>
  'no-reexport': Record<string, never>
  'no-relative-imports': NoRelativeImportsOptions
  'no-sync-dynamic-import': Record<string, never>
  'no-leaked-text-in-jsx': Record<string, never>
  'no-static-style-prop': NoStaticStylePropOptions
  'no-non-camel-case-functions': Record<string, never>
  'prefer-named-functions': PreferNamedFunctionsOptions
  'no-call-with-explicit-generics': NoCallWithExplicitGenericsOptions
  'no-call-with-inferred-generics': NoCallWithInferredGenericsOptions
  'no-unnecessary-typing': NoUnnecessaryTypingOptions
  'no-type-guards': NoTypeGuardsOptions
  'use-state-setter-naming': UseStateSetterNamingOptions
  'require-lazy-execution': RequireLazyExecutionOptions
  'no-unnecessary-iife': Record<string, never>
  'require-description': RequireDescriptionOptions
  'no-write-only-ref': Record<string, never>
  'no-commented-out-code': Record<string, never>
  'react-single-export': ReactSingleExportOptions
  'no-unused-optional-args': NoUnusedOptionalArgsOptions
  'no-unused-type-props-in-args': NoUnusedTypePropsInArgsOptions
  'no-unused-t-state-field': Record<string, never>
  'no-unused-selected-values': NoUnusedSelectedValuesOptions
  'no-optional-root-props': Record<string, never>
  'no-redundant-function-params': NoRedundantFunctionParamsOptions
  'react-no-module-level-single-use-values': ReactNoModuleLevelSingleUseValuesOptions
  'require-reads-to-var-prop': RequireReadsToVarPropOptions
  'prefer-react-hook-alternative': PreferReactHookAlternativeOptions
  'sort-react-components-and-styles': SortReactComponentsAndStylesOptions
  'use-types-directly-above-usage': UseTypesDirectlyAboveUsageOptions
  'react-compiler-extra': ReactCompilerExtraOptions
}

export type RulesConfig = {
  [Name in keyof RuleOptions as Name | `${string}/${Name}`]?: RuleEntry<
    RuleOptions[Name]
  >
}

export type Override = {
  files: string[]
  ignores?: string[]
  rules?: RulesConfig
}

export type FastLintConfig = {
  projects?: string[]
  ignores?: string[]
  rules?: RulesConfig
  overrides?: Override[]
  reportUnusedDisableDirectives?: boolean
}

export function defineConfig(config: FastLintConfig): FastLintConfig
