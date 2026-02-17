import { TSESLint } from '@typescript-eslint/utils'
import { rules } from './rules/rules'

export const extendedLintPlugin: TSESLint.FlatConfig.Plugin = {
  rules,
}

// eslint-disable-next-line @ls-stack/no-default-export -- needed for oxlint
export default extendedLintPlugin
