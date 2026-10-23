import type { TSESTree } from '@typescript-eslint/types'
import { AST_NODE_TYPES } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

export const noWriteOnlyRef = createExtendedLintRule<[], 'refNotRead'>({
  name: 'no-write-only-ref',
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow creating refs that are never read',
    },
    messages: {
      refNotRead:
        'Ref "{{name}}" is never read. Consider removing it if not needed.',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    const useRefNames = new Set<string>()

    function isUseRefCall(node: TSESTree.CallExpression): boolean {
      if (node.callee.type === AST_NODE_TYPES.Identifier) {
        return useRefNames.has(node.callee.name)
      }

      if (
        node.callee.type === AST_NODE_TYPES.MemberExpression &&
        node.callee.object.type === AST_NODE_TYPES.Identifier &&
        node.callee.property.type === AST_NODE_TYPES.Identifier
      ) {
        return node.callee.property.name === 'useRef'
      }

      return false
    }

    return {
      ImportDeclaration(node) {
        if (node.source.value === 'react') {
          for (const specifier of node.specifiers) {
            if (
              specifier.type === AST_NODE_TYPES.ImportSpecifier &&
              specifier.imported.type === AST_NODE_TYPES.Identifier &&
              specifier.imported.name === 'useRef'
            ) {
              useRefNames.add(specifier.local.name)
            }
          }
        }
      },

      VariableDeclarator(node) {
        if (
          node.init &&
          node.init.type === AST_NODE_TYPES.CallExpression &&
          isUseRefCall(node.init) &&
          node.id.type === AST_NODE_TYPES.Identifier
        ) {
          // Get the variable from scope
          const scope = context.sourceCode.getScope(node)
          const variable = scope.set.get(node.id.name)

          if (variable) {
            let hasNonRefUsage = false

            // Check all references to this variable
            for (const reference of variable.references) {
              const refNode = reference.identifier
              const parent = refNode.parent

              // Skip the declaration itself
              if (parent === node) continue

              // Check if this is used as a JSX ref attribute
              if (
                parent.type === AST_NODE_TYPES.JSXExpressionContainer &&
                parent.parent.type === AST_NODE_TYPES.JSXAttribute &&
                parent.parent.name.type === AST_NODE_TYPES.JSXIdentifier &&
                parent.parent.name.name === 'ref'
              ) {
                // This is a ref attribute, don't count as read
                continue
              }

              // Any other usage counts as read
              hasNonRefUsage = true
              break
            }

            if (!hasNonRefUsage) {
              context.report({
                node: node.id,
                messageId: 'refNotRead',
                data: { name: node.id.name },
              })
            }
          }
        }
      },
    }
  },
})
