import { LooseRuleDefinition } from '@typescript-eslint/utils/ts-eslint'
import { exhaustiveDepsESLintRule } from './exhaustive-deps'
import { noCallWithInferedGenerics } from './no-call-with-infered-generics'
import { noCommentedOutCode } from './no-commented-out-code'
import { noDefaultExport } from './no-default-export'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'
import { requireDescription } from './require-description'
import { rulesOfHooksESLintRule } from './rules-of-hooks'

export const rules: Record<string, LooseRuleDefinition> | undefined = {
  [noUnusedObjectTypeProperties.name]: noUnusedObjectTypeProperties.rule,
  [noCommentedOutCode.name]: noCommentedOutCode.rule,
  [noCallWithInferedGenerics.name]: noCallWithInferedGenerics.rule,
  ['rules-of-hooks']: rulesOfHooksESLintRule as any,
  ['exhaustive-deps']: exhaustiveDepsESLintRule as any,
  ['require-description']: requireDescription.rule,
  [noDefaultExport.name]: noDefaultExport,
}
