import { LooseRuleDefinition } from '@typescript-eslint/utils/ts-eslint'
import { reactCompilerExtra } from '../react-compiler-extra'
import { advancedNoRestrictedSyntax } from './advanced-no-restricted-syntax'
import { collapseSimpleObjsInOneLine } from './collapse-simple-objs-in-one-line'
import { exhaustiveDepsESLintRule } from './exhaustive-deps'
import { improvedNoUnnecessaryCondition } from './improved-no-unnecessary-condition'
import { noCallWithExplicitGenerics } from './no-call-with-explicit-generics'
import { noCallWithInferredGenerics } from './no-call-with-inferred-generics'
import { noCommentedOutCode } from './no-commented-out-code'
import { noDefaultExport } from './no-default-export'
import { noLeakedTextInJSX } from './no-leaked-text-in-jsx'
import { noNonCamelCaseFunctions } from './no-non-camel-case-functions'
import { noOptionalRootProps } from './no-optional-root-props'
import { noRelativeImports } from './no-relative-imports'
import { noTypeGuards } from './no-type-guards'
import { noUnnecessaryAsyncOnJsxProps } from './no-unnecessary-async-on-jsx-props'
import { noUnnecessaryCasting } from './no-unnecessary-casting'
import { noUnnecessaryTyping } from './no-unnecessary-typing'
import { noUnusedObjProps } from './no-unused-obj-props'
import { noUnusedSelectedValues } from './no-unused-selected-values'
import { noUnusedTStateField } from './no-unused-t-state-field'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'
import { preferNamedFunction } from './prefer-named-functions'
import { preferReactHookAlternative } from './prefer-react-hook-alternative'
import { preferSingleLineIf } from './prefer-single-line-if'
import { reactCompilerMigration } from './react-compiler-migration'
import { requireDescription } from './require-description'
import { rulesOfHooksESLintRule } from './rules-of-hooks'
import { useTopLevelRegex } from './use-top-level-regex'

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
  [noCallWithExplicitGenerics.name]: noCallWithExplicitGenerics.rule,
  [noRelativeImports.name]: noRelativeImports.rule,
  [preferSingleLineIf.name]: preferSingleLineIf.rule,
  [noOptionalRootProps.name]: noOptionalRootProps.rule,
  [noLeakedTextInJSX.name]: noLeakedTextInJSX.rule,
  [collapseSimpleObjsInOneLine.name]: collapseSimpleObjsInOneLine.rule,
  [reactCompilerMigration.name]: reactCompilerMigration.rule,
  [reactCompilerExtra.name]: reactCompilerExtra.rule,
  [improvedNoUnnecessaryCondition.name]: improvedNoUnnecessaryCondition.rule,
  [noUnnecessaryCasting.name]: noUnnecessaryCasting.rule,
  [noUnnecessaryTyping.name]: noUnnecessaryTyping.rule,
  [noUnusedSelectedValues.name]: noUnusedSelectedValues.rule,
  [noUnusedTStateField.name]: noUnusedTStateField.rule,
  [noUnusedObjProps.name]: noUnusedObjProps.rule,
  [preferReactHookAlternative.name]: preferReactHookAlternative.rule,
  [noUnnecessaryAsyncOnJsxProps.name]: noUnnecessaryAsyncOnJsxProps.rule,
  [noTypeGuards.name]: noTypeGuards.rule,
  [useTopLevelRegex.name]: useTopLevelRegex.rule,
}
