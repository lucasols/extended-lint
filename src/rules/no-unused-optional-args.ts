import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  ignoreArgsMatching: z.string().optional(),
})

type Options = z.infer<typeof optionsSchema>

export const noUnusedOptionalArgs = createExtendedLintRule<
  [Options],
  'unusedOptionalArg' | 'unusedOptionalProp'
>({
  name: 'no-unused-optional-args',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Detect unused optional function arguments in non-exported functions',
    },
    messages: {
      unusedOptionalArg: "Optional parameter '{{name}}' is never used",
      unusedOptionalProp: "Optional prop '{{name}}' is never provided",
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {},
})
