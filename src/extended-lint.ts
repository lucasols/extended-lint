import { TSESLint } from '@typescript-eslint/utils'
import { rules } from './rules/rules'

export const extendedLintPlugin: TSESLint.FlatConfig.Plugin = {
  rules,
}
