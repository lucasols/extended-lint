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

type RuleOptions = {
  'improved-no-unnecessary-condition': ImprovedNoUnnecessaryConditionOptions
  'strict-casting': StrictCastingOptions
  'no-unnecessary-void-on-promise': Record<string, never>
  'no-leaked-conditional-rendering': NoLeakedConditionalRenderingOptions
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
