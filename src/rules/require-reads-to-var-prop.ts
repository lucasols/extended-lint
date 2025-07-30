import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  varsToCheck: z.array(
    z.object({
      selector: z.string().optional(),
      fromFnCall: z.string().optional(),
      prop: z.string(),
      errorMsg: z.string().optional(),
    }),
  ),
})

type Options = z.infer<typeof optionsSchema>

export const requireReadsToVarProp = createExtendedLintRule<
  [Options],
  'propNotRead'
>({
  name: 'require-reads-to-var-prop',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Require variables from specific function calls to be used (any usage counts as potentially accessing required properties)',
    },
    messages: {
      propNotRead: 'Variable "{{varName}}" is never used. {{customMsg}}',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ varsToCheck: [] }],
  create(context, [options]) {
    const varsToTrack = new Map<
      string,
      { prop: string; errorMsg?: string; node: TSESTree.Node }
    >()

    // Helper function to match function call patterns
    function matchesFnCall(
      callExpression: TSESTree.CallExpression,
      pattern: string,
    ): boolean {
      if (callExpression.callee.type === AST_NODE_TYPES.Identifier) {
        // Simple function call: fnName()
        return callExpression.callee.name === pattern
      }

      if (
        callExpression.callee.type === AST_NODE_TYPES.MemberExpression &&
        callExpression.callee.property.type === AST_NODE_TYPES.Identifier
      ) {
        // Member call: obj.method()
        const methodName = callExpression.callee.property.name

        if (pattern.startsWith('*.')) {
          // Wildcard pattern: *.useElement matches any.useElement
          const expectedMethod = pattern.slice(2)
          return methodName === expectedMethod
        }

        if (pattern.includes('.')) {
          // Specific pattern: test.useElement
          const [expectedObj, expectedMethod] = pattern.split('.')
          if (callExpression.callee.object.type === AST_NODE_TYPES.Identifier) {
            return (
              callExpression.callee.object.name === expectedObj &&
              methodName === expectedMethod
            )
          }
        }
      }

      return false
    }

    // Helper function to check if a variable declarator should be tracked
    function shouldTrackVariable(
      node: TSESTree.VariableDeclarator,
      check: (typeof options.varsToCheck)[0],
    ): boolean {
      if (
        node.id.type !== AST_NODE_TYPES.Identifier ||
        !node.init ||
        node.init.type !== AST_NODE_TYPES.CallExpression
      ) {
        return false
      }

      if (check.fromFnCall) {
        return matchesFnCall(node.init, check.fromFnCall)
      }

      return false // selector-based matching handled separately
    }

    // Create AST selectors for each configured check that uses selectors
    const selectors: Record<string, (node: TSESTree.Node) => void> = {}

    for (const check of options.varsToCheck) {
      if (check.selector) {
        selectors[check.selector] = (node: TSESTree.Node) => {
          // For variable declarations, we need to find the identifier
          if (
            node.type === AST_NODE_TYPES.VariableDeclarator &&
            node.id.type === AST_NODE_TYPES.Identifier
          ) {
            varsToTrack.set(node.id.name, {
              prop: check.prop,
              errorMsg: check.errorMsg,
              node: node.id,
            })
          }
        }
      }
    }

    return {
      ...selectors,

      VariableDeclarator(node) {
        // Check fromFnCall patterns
        for (const check of options.varsToCheck) {
          if (check.fromFnCall && shouldTrackVariable(node, check)) {
            if (node.id.type === AST_NODE_TYPES.Identifier) {
              varsToTrack.set(node.id.name, {
                prop: check.prop,
                errorMsg: check.errorMsg,
                node: node.id,
              })
            }
          }
        }
      },

      'Program:exit'() {
        // Check each tracked variable to see if it has any meaningful usage
        for (const [varName, { prop, errorMsg, node }] of varsToTrack) {
          const scope = context.sourceCode.getScope(node)
          const variable = scope.set.get(varName)

          if (variable) {
            let hasUsage = false

            // Check all references to this variable
            for (const reference of variable.references) {
              const refNode = reference.identifier
              const parent = refNode.parent

              // Skip the declaration itself
              if (
                parent.type === AST_NODE_TYPES.VariableDeclarator &&
                parent.id === refNode
              ) {
                continue
              }

              // Any other usage of the variable counts as potentially accessing the property
              // Functions, JSX props, expressions, etc. could all access the required property
              hasUsage = true
              break
            }

            if (!hasUsage) {
              context.report({
                node,
                messageId: 'propNotRead',
                data: {
                  prop,
                  varName,
                  customMsg: errorMsg || '',
                },
              })
            }
          }
        }
      },
    }
  },
})
