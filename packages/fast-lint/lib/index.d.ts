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

export type RulesConfig = {
  'improved-no-unnecessary-condition'?: RuleEntry<ImprovedNoUnnecessaryConditionOptions>
  'strict-casting'?: RuleEntry<StrictCastingOptions>
  'no-unnecessary-void-on-promise'?: RuleEntry<Record<string, never>>
  'no-leaked-conditional-rendering'?: RuleEntry<NoLeakedConditionalRenderingOptions>
}

export type Override = {
  files: string[]
  ignores?: string[]
  rules: RulesConfig
}

export type FastLintConfig = {
  projects?: string[]
  ignores?: string[]
  rules?: RulesConfig
  overrides?: Override[]
}

export function defineConfig(config: FastLintConfig): FastLintConfig
