import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
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

    function containsImportedIdentifier(node: TSESTree.Node | null | undefined): boolean {
      if (!node) {
        return false
      }
      
      if (node.type === AST_NODE_TYPES.Identifier) {
        return importedIdentifiers.has(node.name)
      }
      
      if (node.type === AST_NODE_TYPES.MemberExpression) {
        return containsImportedIdentifier(node.object)
      }
      
      return false
    }

    return {
      ImportDeclaration(node) {
        for (const specifier of node.specifiers) {
          importedIdentifiers.add(specifier.local.name)
        }
      },
      ExportNamedDeclaration(node) {
        if (node.source) {
          context.report({ node, messageId: 'noReexport' })
          return
        }

        if (node.declaration?.type === AST_NODE_TYPES.VariableDeclaration) {
          for (const declarator of node.declaration.declarations) {
            if (containsImportedIdentifier(declarator.init)) {
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
