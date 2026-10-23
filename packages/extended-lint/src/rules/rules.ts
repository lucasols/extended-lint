import { noDataAttributesOnComponents } from './no-data-attributes-on-components'
import { noRestrictedComments } from './no-restricted-comments'
import { noUnnecessaryAsyncOnJsxProps } from './no-unnecessary-async-on-jsx-props'
import { noUnnecessaryDescribe } from './no-unnecessary-describe'
import { noUnusedObjProps } from './no-unused-obj-props'
import { reactCompilerMigration } from './react-compiler-migration'
import { requireUsageExplanation } from './require-usage-explanation'
import { templateIndent } from './template-indent'
import { useTopLevelRegex } from './use-top-level-regex'

type ExtendedLintRuleListener = {
  visit(node: unknown): void
}['visit']

export type ExtendedLintRule = {
  meta?: object
  create(context: never): Record<string, ExtendedLintRuleListener | undefined>
}

export const rules: Record<string, ExtendedLintRule> = {
  [noDataAttributesOnComponents.name]: noDataAttributesOnComponents.rule,
  [noRestrictedComments.name]: noRestrictedComments.rule,
  [noUnnecessaryAsyncOnJsxProps.name]: noUnnecessaryAsyncOnJsxProps.rule,
  [noUnnecessaryDescribe.name]: noUnnecessaryDescribe.rule,
  [noUnusedObjProps.name]: noUnusedObjProps.rule,
  [reactCompilerMigration.name]: reactCompilerMigration.rule,
  [requireUsageExplanation.name]: requireUsageExplanation.rule,
  [templateIndent.name]: templateIndent.rule,
  [useTopLevelRegex.name]: useTopLevelRegex.rule,
}
