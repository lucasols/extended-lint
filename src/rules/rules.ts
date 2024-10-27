import { LooseRuleDefinition } from '@typescript-eslint/utils/ts-eslint'
import { advancedNoRestrictedSyntax } from './advanced-no-restricted-syntax'
import { exhaustiveDepsESLintRule } from './exhaustive-deps'
import { noCallWithInferredGenerics } from './no-call-with-inferred-generics'
import { noCommentedOutCode } from './no-commented-out-code'
import { noDefaultExport } from './no-default-export'
import { noNonCamelCaseFunctions } from './no-non-camel-case-functions'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'
import { preferNamedFunction } from './prefer-named-functions'
import { requireDescription } from './require-description'
import { rulesOfHooksESLintRule } from './rules-of-hooks'

export const rules: Record<string, LooseRuleDefinition> = {
  [noUnusedObjectTypeProperties.name]: noUnusedObjectTypeProperties.rule,
  [noCommentedOutCode.name]: noCommentedOutCode.rule,
  [noCallWithInferredGenerics.name]: noCallWithInferredGenerics.rule,
  ['rules-of-hooks']: rulesOfHooksESLintRule as any,
  ['exhaustive-deps']: exhaustiveDepsESLintRule as any,
  ['require-description']: requireDescription.rule,
  [noDefaultExport.name]: noDefaultExport.rule,
  [noNonCamelCaseFunctions.name]: noNonCamelCaseFunctions.rule,
  [preferNamedFunction.name]: preferNamedFunction.rule,
  [advancedNoRestrictedSyntax.name]: advancedNoRestrictedSyntax.rule,
}
