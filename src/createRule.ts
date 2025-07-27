import { ESLintUtils, TSESLint } from '@typescript-eslint/utils'
import type { JSONSchema4 } from '@typescript-eslint/utils/json-schema'
import * as z from 'zod/v4'

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

export function getJsonSchemaFromZod(zodSchema: z.ZodTypeAny): JSONSchema4 {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  return z.toJSONSchema(zodSchema) as any
}
