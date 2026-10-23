import { rules, type ExtendedLintRule } from './rules/rules'

export type ExtendedLintPlugin = {
  rules: Record<string, ExtendedLintRule>
}

export const extendedLintPlugin: ExtendedLintPlugin = {
  rules,
}

// eslint-disable-next-line @ls-stack/no-default-export -- needed for oxlint
export default extendedLintPlugin
