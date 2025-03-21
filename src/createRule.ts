import { ESLintUtils, TSESLint } from '@typescript-eslint/utils'

export function createExtendedLintRule<
  O extends readonly unknown[],
  ErrIds extends string,
>(
  rule: ESLintUtils.RuleWithMetaAndName<O, ErrIds, unknown>,
): {
  name: string
  rule: TSESLint.RuleModule<string, O>
} {
  const createRule = ESLintUtils.RuleCreator(
    (name) => `https://github.com/lucasols/extended-lint#${name}`,
  )

  const eslintRule = createRule(rule)

  return {
    name: rule.name,
    rule: eslintRule as unknown as TSESLint.RuleModule<string, O>,
  }
}
