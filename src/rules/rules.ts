import { exhaustiveDepsESLintRule } from './exhaustive-deps'
import { noCallWithInferedGenerics } from './no-call-with-infered-generics'
import { noCommentedOutCode } from './no-commented-out-code'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'
import { rulesOfHooksESLintRule } from './rules-of-hooks'

export const rules = {
  [noUnusedObjectTypeProperties.name]: noUnusedObjectTypeProperties.rule,
  [noCommentedOutCode.name]: noCommentedOutCode.rule,
  [noCallWithInferedGenerics.name]: noCallWithInferedGenerics.rule,
  ['rules-of-hooks']: rulesOfHooksESLintRule,
  ['exhaustive-deps']: exhaustiveDepsESLintRule,
}
