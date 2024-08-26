import { FlatConfig } from '@typescript-eslint/utils/ts-eslint'
import { exhaustiveDepsESLintRule } from './exhaustive-deps'
import { noCallWithInferedGenerics } from './no-call-with-infered-generics'
import { noCommentedOutCode } from './no-commented-out-code'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'
import { rulesOfHooksESLintRule } from './rules-of-hooks'

export const rules: FlatConfig.Rules = {
  [noUnusedObjectTypeProperties.name]: noUnusedObjectTypeProperties.rule,
  [noCommentedOutCode.name]: noCommentedOutCode.rule,
  [noCallWithInferedGenerics.name]: noCallWithInferedGenerics.rule,
  ['rules-of-hooks']: rulesOfHooksESLintRule as any,
  ['exhaustive-deps']: exhaustiveDepsESLintRule as any,
}
