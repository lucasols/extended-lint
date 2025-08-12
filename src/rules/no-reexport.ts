import { AST_NODE_TYPES } from '@typescript-eslint/utils'
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
      noReexport: 'Re-exports are not allowed. Use direct exports only.',
    },
  },
  defaultOptions: [],
  create(context) {
    const importedIdentifiers = new Set<string>()

    return {
      ImportDeclaration(node) {
        for (const specifier of node.specifiers) {
          if (
            specifier.type === AST_NODE_TYPES.ImportSpecifier ||
            specifier.type === AST_NODE_TYPES.ImportDefaultSpecifier
          ) {
            importedIdentifiers.add(specifier.local.name)
          }
        }
      },
      ExportNamedDeclaration(node) {
        if (node.source) {
          context.report({ node, messageId: 'noReexport' })
          return
        }

        if (node.declaration?.type === AST_NODE_TYPES.VariableDeclaration) {
          for (const declarator of node.declaration.declarations) {
            if (
              declarator.id.type === AST_NODE_TYPES.Identifier &&
              declarator.init?.type === AST_NODE_TYPES.Identifier &&
              importedIdentifiers.has(declarator.init.name)
            ) {
              context.report({ node, messageId: 'noReexport' })
            }
          }
        }

        if (node.specifiers.length > 0) {
          context.report({ node, messageId: 'noReexport' })
        }
      },
      ExportDefaultDeclaration(node) {
        if (
          node.declaration.type === AST_NODE_TYPES.Identifier &&
          importedIdentifiers.has(node.declaration.name)
        ) {
          context.report({ node, messageId: 'noReexport' })
        }
      },
      ExportAllDeclaration(node) {
        context.report({ node, messageId: 'noReexport' })
      },
    }
  },
})
