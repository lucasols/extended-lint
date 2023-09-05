import { noCallWithInferedGenerics } from './no-call-with-infered-generics'
import { noCommentedOutCode } from './no-commented-code'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'
import ReactHooksESLintRule from './rules-of-hooks'

export const rules = {
  [noUnusedObjectTypeProperties.name]: noUnusedObjectTypeProperties.rule,
  [noCommentedOutCode.name]: noCommentedOutCode.rule,
  [noCallWithInferedGenerics.name]: noCallWithInferedGenerics.rule,
  ['rules-of-hooks']: ReactHooksESLintRule,
}
