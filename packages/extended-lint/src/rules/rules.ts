import { noRestrictedComments } from './no-restricted-comments'
import { noUnnecessaryAsyncOnJsxProps } from './no-unnecessary-async-on-jsx-props'
import { noUnusedObjProps } from './no-unused-obj-props'
import { requireUsageExplanation } from './require-usage-explanation'
import { useTopLevelRegex } from './use-top-level-regex'

type ExtendedLintRuleListener = {
  visit(node: unknown): void
}['visit']

export type ExtendedLintRule = {
  meta?: object
  create(context: never): Record<string, ExtendedLintRuleListener | undefined>
}

export const rules: Record<string, ExtendedLintRule> = {
  [noRestrictedComments.name]: noRestrictedComments.rule,
  [noUnnecessaryAsyncOnJsxProps.name]: noUnnecessaryAsyncOnJsxProps.rule,
  [noUnusedObjProps.name]: noUnusedObjProps.rule,
  [requireUsageExplanation.name]: requireUsageExplanation.rule,
  [useTopLevelRegex.name]: useTopLevelRegex.rule,
}
