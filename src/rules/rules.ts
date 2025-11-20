import { LooseRuleDefinition } from '@typescript-eslint/utils/ts-eslint'
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
import { noReexport } from './no-reexport'
import { noRelativeImports } from './no-relative-imports'
import { noStaticStyleProp } from './no-static-style-prop'
import { noTypeGuards } from './no-type-guards'
import { noUnnecessaryAsyncOnJsxProps } from './no-unnecessary-async-on-jsx-props'
import { noUnnecessaryCasting } from './no-unnecessary-casting'
import { noUnnecessaryDescribe } from './no-unnecessary-describe'
import { noUnnecessaryTyping } from './no-unnecessary-typing'
import { noUnusedObjProps } from './no-unused-obj-props'
import { noUnusedOptionalArgs } from './no-unused-optional-args'
import { noUnusedSelectedValues } from './no-unused-selected-values'
import { noUnusedTStateField } from './no-unused-t-state-field'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'
import { noWriteOnlyRef } from './no-write-only-ref'
import { preferNamedFunction } from './prefer-named-functions'
import { preferReactHookAlternative } from './prefer-react-hook-alternative'
import { preferSingleLineIf } from './prefer-single-line-if'
import { reactCompilerExtra } from './react-compiler-extra'
import { reactCompilerMigration } from './react-compiler-migration'
import { reactOptimizedCollections } from './react-optimized-collections'
import { requireDescription } from './require-description'
import { requireReadsToVarProp } from './require-reads-to-var-prop'
import { rulesOfHooksESLintRule } from './rules-of-hooks'
import { templateIndent } from './template-indent'
import { useTopLevelRegex } from './use-top-level-regex'
import { useTypesDirectlyAboveUsage } from './use-types-directly-above-usage'
import { reactSingleExport } from './react-single-export'
import { useStateSetterNaming } from './use-state-setter-naming'
import { noRedundantFunctionParams } from './no-redundant-function-params'
import { noSyncDynamicImport } from './no-sync-dynamic-import'
import { sortReactComponentsAndStyles } from './sort-react-components-and-styles'
import { noUnnecessaryVoidOnPromise } from './no-unnecessary-void-on-promise'
import { requireLazyExecution } from './require-lazy-execution'

export const rules: Record<string, LooseRuleDefinition> = {
  [noUnusedObjectTypeProperties.name]: noUnusedObjectTypeProperties.rule,
  [noCommentedOutCode.name]: noCommentedOutCode.rule,
  [noCallWithInferredGenerics.name]: noCallWithInferredGenerics.rule,
  ['rules-of-hooks']: rulesOfHooksESLintRule,
  ['exhaustive-deps']: exhaustiveDepsESLintRule,
  ['require-description']: requireDescription.rule,
  [noDefaultExport.name]: noDefaultExport.rule,
  [noNonCamelCaseFunctions.name]: noNonCamelCaseFunctions.rule,
  [preferNamedFunction.name]: preferNamedFunction.rule,
  [advancedNoRestrictedSyntax.name]: advancedNoRestrictedSyntax.rule,
  [noCallWithExplicitGenerics.name]: noCallWithExplicitGenerics.rule,
  [noRelativeImports.name]: noRelativeImports.rule,
  [noReexport.name]: noReexport.rule,
  [preferSingleLineIf.name]: preferSingleLineIf.rule,
  [noOptionalRootProps.name]: noOptionalRootProps.rule,
  [noLeakedTextInJSX.name]: noLeakedTextInJSX.rule,
  [noStaticStyleProp.name]: noStaticStyleProp.rule,
  [collapseSimpleObjsInOneLine.name]: collapseSimpleObjsInOneLine.rule,
  [reactCompilerMigration.name]: reactCompilerMigration.rule,
  [reactCompilerExtra.name]: reactCompilerExtra.rule,
  [reactOptimizedCollections.name]: reactOptimizedCollections.rule,
  [improvedNoUnnecessaryCondition.name]: improvedNoUnnecessaryCondition.rule,
  [noUnnecessaryCasting.name]: noUnnecessaryCasting.rule,
  [noUnnecessaryDescribe.name]: noUnnecessaryDescribe.rule,
  [noUnnecessaryTyping.name]: noUnnecessaryTyping.rule,
  [noUnusedSelectedValues.name]: noUnusedSelectedValues.rule,
  [noUnusedTStateField.name]: noUnusedTStateField.rule,
  [noUnusedObjProps.name]: noUnusedObjProps.rule,
  [noUnusedOptionalArgs.name]: noUnusedOptionalArgs.rule,
  [preferReactHookAlternative.name]: preferReactHookAlternative.rule,
  [noUnnecessaryAsyncOnJsxProps.name]: noUnnecessaryAsyncOnJsxProps.rule,
  [noTypeGuards.name]: noTypeGuards.rule,
  [useTopLevelRegex.name]: useTopLevelRegex.rule,
  [noWriteOnlyRef.name]: noWriteOnlyRef.rule,
  [requireReadsToVarProp.name]: requireReadsToVarProp.rule,
  [templateIndent.name]: templateIndent.rule,
  [useTypesDirectlyAboveUsage.name]: useTypesDirectlyAboveUsage.rule,
  [reactSingleExport.name]: reactSingleExport.rule,
  [useStateSetterNaming.name]: useStateSetterNaming.rule,
  [noRedundantFunctionParams.name]: noRedundantFunctionParams.rule,
  [noSyncDynamicImport.name]: noSyncDynamicImport.rule,
  [sortReactComponentsAndStyles.name]: sortReactComponentsAndStyles.rule,
  [noUnnecessaryVoidOnPromise.name]: noUnnecessaryVoidOnPromise.rule,
  [requireLazyExecution.name]: requireLazyExecution.rule,
  [noReexport.name]: noReexport.rule,
}
