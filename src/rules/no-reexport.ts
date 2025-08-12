import { createExtendedLintRule } from '../createRule'

export const noReexport = createExtendedLintRule<[], 'noReexport'>({
  name: 'no-reexport',
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow re-exports to prevent indirection',
    },
    schema: [],
    messages: {
      noReexport: 'Re-exports are not allowed. Import and export separately instead.',
    },
  },
  defaultOptions: [],
  create(context) {
    return {
      ExportNamedDeclaration(node) {
        if (node.source) {
          context.report({ node, messageId: 'noReexport' })
        }
      },
      ExportAllDeclaration(node) {
        context.report({ node, messageId: 'noReexport' })
      },
    }
  },
})