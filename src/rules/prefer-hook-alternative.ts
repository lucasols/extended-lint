import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import * as t from 'tschema'
import { createExtendedLintRule } from '../createRule'

const optionsSchema = t.object({
  disallowedFunctions: t.array(
    t.object({
      name: t.string(),
      hookAlternative: t.string(),
    }),
  ),
})

type Options = t.Infer<typeof optionsSchema>

function isHookName(name: string): boolean {
  return /^use[A-Z]/.test(name)
}

function isComponentName(name: string): boolean {
  return /^[A-Z]/.test(name)
}

function isHook(node: TSESTree.CallExpression['callee']): boolean {
  if (node.type === AST_NODE_TYPES.Identifier) {
    return isHookName(node.name)
  }

  if (
    node.type === AST_NODE_TYPES.MemberExpression &&
    node.property.type === AST_NODE_TYPES.Identifier
  ) {
    return isHookName(node.property.name)
  }

  return false
}

function isReactFunction(node: TSESTree.Node, functionName: string): boolean {
  if (node.type === AST_NODE_TYPES.Identifier) {
    return node.name === functionName
  }

  if (
    node.type === AST_NODE_TYPES.MemberExpression &&
    node.object.type === AST_NODE_TYPES.Identifier &&
    node.property.type === AST_NODE_TYPES.Identifier
  ) {
    return node.object.name === 'React' && node.property.name === functionName
  }

  return false
}

function isForwardRefCallback(node: TSESTree.Node): boolean {
  return !!(
    node.parent &&
    node.parent.type === AST_NODE_TYPES.CallExpression &&
    isReactFunction(node.parent.callee, 'forwardRef')
  )
}

function isMemoCallback(node: TSESTree.Node): boolean {
  return !!(
    node.parent &&
    node.parent.type === AST_NODE_TYPES.CallExpression &&
    isReactFunction(node.parent.callee, 'memo')
  )
}

function getFunctionName(node: TSESTree.Node): TSESTree.Node | undefined {
  if (
    node.type === AST_NODE_TYPES.FunctionDeclaration ||
    (node.type === AST_NODE_TYPES.FunctionExpression && node.id)
  ) {
    return node.id || undefined
  } else if (
    node.type === AST_NODE_TYPES.FunctionExpression ||
    node.type === AST_NODE_TYPES.ArrowFunctionExpression
  ) {
    if (
      node.parent.type === AST_NODE_TYPES.VariableDeclarator &&
      node.parent.init === node
    ) {
      return node.parent.id
    } else if (
      node.parent.type === AST_NODE_TYPES.AssignmentExpression &&
      node.parent.right === node &&
      node.parent.operator === '='
    ) {
      return node.parent.left
    } else if (
      node.parent.type === AST_NODE_TYPES.Property &&
      node.parent.value === node &&
      !node.parent.computed
    ) {
      return node.parent.key
    } else if (
      node.parent.type === AST_NODE_TYPES.AssignmentPattern &&
      node.parent.right === node
    ) {
      return node.parent.left
    }
  }
  return undefined
}

function isDirectlyInsideComponentOrHook(node: TSESTree.Node): boolean {
  let current = node.parent
  let foundNestedNonHookFunction = false

  while (current) {
    if (
      current.type === AST_NODE_TYPES.FunctionDeclaration ||
      current.type === AST_NODE_TYPES.FunctionExpression ||
      current.type === AST_NODE_TYPES.ArrowFunctionExpression
    ) {
      const functionName = getFunctionName(current)
      if (functionName && functionName.type === AST_NODE_TYPES.Identifier) {
        if (
          isComponentName(functionName.name) ||
          isHookName(functionName.name)
        ) {
          return !foundNestedNonHookFunction
        } else {
          foundNestedNonHookFunction = true
        }
      } else if (isForwardRefCallback(current) || isMemoCallback(current)) {
        return !foundNestedNonHookFunction
      } else {
        // Check if this is inside a hook call (like useEffect callback)
        if (current.parent.type === AST_NODE_TYPES.CallExpression) {
          const callExpression = current.parent
          if (isHook(callExpression.callee)) {
            // This is a callback inside a hook call, don't count it as a nested function
            // Continue looking up the tree
          } else {
            foundNestedNonHookFunction = true
          }
        } else if (
          current.parent.type === AST_NODE_TYPES.JSXExpressionContainer
        ) {
          // This is a callback inside JSX (like onClick={() => ...})
          // Don't count it as a nested function, continue looking up
        } else {
          foundNestedNonHookFunction = true
        }
      }
    }
    current = current.parent
  }
  return false
}

function getFunctionCallName(node: TSESTree.CallExpression): string | null {
  if (node.callee.type === AST_NODE_TYPES.Identifier) {
    return node.callee.name
  }

  if (
    node.callee.type === AST_NODE_TYPES.MemberExpression &&
    node.callee.property.type === AST_NODE_TYPES.Identifier
  ) {
    return node.callee.property.name
  }

  return null
}

export const preferHookAlternative = createExtendedLintRule<
  [Options],
  'preferHookAlternative'
>({
  name: 'prefer-hook-alternative',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Prefer hook alternatives for certain functions in React components and hooks',
    },
    messages: {
      preferHookAlternative:
        'This function should not be used in react use the hook {{hookAlternative}} instead.',
    },
    schema: [optionsSchema as any],
    hasSuggestions: true,
  },
  defaultOptions: [{ disallowedFunctions: [] }],
  create(context, [options]) {
    const { disallowedFunctions } = options

    const disallowedMap = new Map(
      disallowedFunctions.map((fn) => [fn.name, fn]),
    )

    return {
      CallExpression(node) {
        if (!isDirectlyInsideComponentOrHook(node)) {
          return
        }

        const functionName = getFunctionCallName(node)
        if (!functionName) {
          return
        }

        const disallowedFn = disallowedMap.get(functionName)
        if (disallowedFn) {
          context.report({
            node,
            messageId: 'preferHookAlternative',
            data: {
              hookAlternative: disallowedFn.hookAlternative,
            },
            suggest: [
              {
                messageId: 'preferHookAlternative',
                data: {
                  message: `Replace with ${disallowedFn.hookAlternative}`,
                  hookAlternative: disallowedFn.hookAlternative,
                },
                fix: (fixer) => {
                  if (node.callee.type === AST_NODE_TYPES.Identifier) {
                    return fixer.replaceText(
                      node.callee,
                      disallowedFn.hookAlternative,
                    )
                  } else if (
                    node.callee.type === AST_NODE_TYPES.MemberExpression &&
                    node.callee.property.type === AST_NODE_TYPES.Identifier
                  ) {
                    return fixer.replaceText(
                      node.callee.property,
                      disallowedFn.hookAlternative,
                    )
                  }
                  return null
                },
              },
            ],
          })
        }
      },
    }
  },
})
