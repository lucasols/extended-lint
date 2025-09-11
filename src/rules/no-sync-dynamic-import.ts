import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

function isInAsyncContext(node: TSESTree.ImportExpression): boolean {
  let parent = node.parent

  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- we want to check all parents
  while (parent) {
    // Check if directly awaited
    if (parent.type === AST_NODE_TYPES.AwaitExpression) {
      return true
    }

    // Check if used with .then() method call
    if (
      parent.type === AST_NODE_TYPES.MemberExpression &&
      parent.object === node &&
      parent.property.type === AST_NODE_TYPES.Identifier &&
      parent.property.name === 'then'
    ) {
      return true
    }

    // Check if used as arrow function return (allows patterns like () => import())
    if (parent.type === AST_NODE_TYPES.ArrowFunctionExpression && parent.body === node) {
      return true
    }

    // Check if used in Promise.all, Promise.allSettled, etc.
    if (
      parent.type === AST_NODE_TYPES.ArrayExpression &&
      parent.parent.type === AST_NODE_TYPES.CallExpression
    ) {
      const callExpression = parent.parent
      if (
        callExpression.callee.type === AST_NODE_TYPES.MemberExpression &&
        callExpression.callee.object.type === AST_NODE_TYPES.Identifier &&
        callExpression.callee.object.name === 'Promise' &&
        callExpression.callee.property.type === AST_NODE_TYPES.Identifier &&
        ['all', 'allSettled', 'race'].includes(
          callExpression.callee.property.name,
        )
      ) {
        return true
      }
    }

    // Move up the AST
    if (parent.parent) {
      parent = parent.parent
    } else {
      break
    }
  }

  return false
}

function isRequireCall(node: TSESTree.CallExpression): boolean {
  const { callee } = node

  // Direct require() call
  if (callee.type === AST_NODE_TYPES.Identifier && callee.name === 'require') {
    return true
  }

  // require.resolve() call
  if (
    callee.type === AST_NODE_TYPES.MemberExpression &&
    callee.object.type === AST_NODE_TYPES.Identifier &&
    callee.object.name === 'require'
  ) {
    return true
  }

  return false
}

export const noSyncDynamicImport = createExtendedLintRule<
  [],
  'syncDynamicImport' | 'requireNotAllowed'
>({
  name: 'no-sync-dynamic-import',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow synchronous dynamic imports and require statements, only allow async dynamic imports with await',
    },
    messages: {
      syncDynamicImport:
        'Synchronous dynamic import() is not allowed. Use regular import statement or await import() for dynamic imports.',
      requireNotAllowed:
        'require() statements are not allowed. Use ES6 import statements instead.',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    return {
      ImportExpression(node) {
        if (!isInAsyncContext(node)) {
          context.report({
            node,
            messageId: 'syncDynamicImport',
          })
        }
      },

      TSImportType(node) {
        context.report({
          node,
          messageId: 'syncDynamicImport',
        })
      },

      CallExpression(node) {
        if (isRequireCall(node)) {
          context.report({
            node,
            messageId: 'requireNotAllowed',
          })
        }
      },
    }
  },
})
