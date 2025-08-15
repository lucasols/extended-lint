import { createExtendedLintRule } from '../createRule'

export const useTypesDirectlyAboveUsage = createExtendedLintRule<
  [],
  'moveTypeAboveUsage'
>({
  name: 'use-types-directly-above-usage',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Require type definitions to be placed directly above their first usage for better readability',
    },
    fixable: 'code',
    schema: [],
    messages: {
      moveTypeAboveUsage:
        'Type definition should be placed directly above its first usage.',
    },
  },
  defaultOptions: [],
  create(_context) {
    return {}
  },
})
