import type { ESLintUtils, TSESLint } from '@typescript-eslint/utils'
import type { JSONSchema4 } from '@typescript-eslint/utils/json-schema'
import * as z from 'zod/v4'

const docsUrlPrefix = 'https://github.com/lucasols/extended-lint#'

function toObjectEntries(value: unknown): Map<string, unknown> | null {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    return null
  }

  const entries: [string, unknown][] = Object.entries(value)
  return new Map(entries)
}

function deepMerge(
  first: Map<string, unknown>,
  second: Map<string, unknown>,
): Record<string, unknown> {
  const result: Record<string, unknown> = {}

  for (const key of new Set([...first.keys(), ...second.keys()])) {
    const firstValue = first.get(key)
    const secondValue = second.get(key)

    if (first.has(key) && second.has(key)) {
      const firstEntries = toObjectEntries(firstValue)
      const secondEntries = toObjectEntries(secondValue)
      result[key] =
        firstEntries && secondEntries
          ? deepMerge(firstEntries, secondEntries)
          : secondValue
    } else {
      result[key] = first.has(key) ? firstValue : secondValue
    }
  }

  return result
}

export function applyDefaultOptions<O extends readonly unknown[]>(
  defaultOptions: O,
  userOptions: readonly unknown[] | null | undefined,
): O {
  const options = structuredClone(defaultOptions)
  if (!userOptions) return options

  const merged = options.map((defaultOption, index) => {
    const userOption = userOptions[index]
    if (userOption === undefined) return defaultOption

    const defaultEntries = toObjectEntries(defaultOption)
    const userEntries = toObjectEntries(userOption)

    return defaultEntries && userEntries
      ? deepMerge(defaultEntries, userEntries)
      : userOption
  })

  return Object.assign(options, merged)
}

export function createRule<
  O extends readonly unknown[],
  ErrIds extends string,
>({
  name,
  meta,
  create,
  defaultOptions,
}: ESLintUtils.RuleWithMetaAndName<O, ErrIds, unknown>): TSESLint.RuleModule<
  ErrIds,
  O
> {
  return {
    meta: { ...meta, docs: { ...meta.docs, url: `${docsUrlPrefix}${name}` } },
    defaultOptions,
    create(context) {
      return create(context, applyDefaultOptions(defaultOptions, context.options))
    },
  }
}

export function createExtendedLintRule<
  O extends readonly unknown[],
  ErrIds extends string,
>(
  rule: ESLintUtils.RuleWithMetaAndName<O, ErrIds, unknown>,
): {
  name: string
  rule: TSESLint.RuleModule<ErrIds, O>
} {
  return { name: rule.name, rule: createRule(rule) }
}

export function getJsonSchemaFromZod(zodSchema: z.ZodTypeAny): JSONSchema4 {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/consistent-type-assertions -- it is fine to use type assertions here
  return z.toJSONSchema(zodSchema) as any
}
