import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  checkVariables: z.boolean().optional(),
})

type Options = z.infer<typeof optionsSchema>

export const useTypesAboveUsage = createExtendedLintRule<
  [Options],
  'moveTypeAboveUsage'
>({
  name: 'use-types-above-usage',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Require type definitions to be placed directly above their usage for better readability',
    },
    fixable: 'code',
    schema: getJsonSchemaFromZod(optionsSchema),
    messages: {
      moveTypeAboveUsage:
        'Type definition should be placed directly above its first usage.',
    },
  },
  defaultOptions: [{ checkVariables: false }],
  create(context, [options]) {
    return {}
  },
})
