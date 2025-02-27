import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import * as t from 'tschema'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'react-compiler-migration'

const optionsSchema = t.object({
  disallowHooks: t.optional(
    t.array(
      t.object({
        name: t.string(),
        replacement: t.string(),
      }),
    ),
  ),
  disallowMethods: t.optional(
    t.array(
      t.object({
        name: t.string(),
        replacement: t.optional(t.string()),
        requireTrueProp: t.optional(t.string()),
      }),
    ),
  ),
})

type Options = t.Infer<typeof optionsSchema>

const hasEnableCompilerDirectiveRegex =
  /eslint +react-compiler\/react-compiler: +\["error/

/**
 * Checks if a callee is a hook (starts with "use")
 */
function isHook(callee: TSESTree.CallExpression['callee']): boolean {
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name.startsWith('use')
  }

  if (
    callee.type === AST_NODE_TYPES.MemberExpression &&
    callee.property.type === AST_NODE_TYPES.Identifier
  ) {
    return callee.property.name.startsWith('use')
  }

  return false
}

const rule = createRule<
  [Options],
  | 'disallowedFunctionOrMethod'
  | 'replace'
  | 'disallowedMethodWithMissingRequireTrueProp'
>({
  name,
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description: 'Rules to help migrate to the new React compiler',
    },
    messages: {
      disallowedFunctionOrMethod:
        '{{functionOrMethod}} is not supported in react compiler. Use {{replacement}} instead.',
      replace: 'Replace with safe alternative {{replacement}}',
      disallowedMethodWithMissingRequireTrueProp:
        '{{method}} is should have a prop named {{requireTrueProp}} set to true when used in react compiler.',
    },
    hasSuggestions: true,
    schema: [optionsSchema as any],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    let isEnabled = false

    for (const comment of context.sourceCode.getAllComments()) {
      if (hasEnableCompilerDirectiveRegex.test(comment.value)) {
        isEnabled = true
        break
      }
    }

    if (!isEnabled) {
      return {}
    }

    return {
      CallExpression(node) {
        if (options.disallowHooks?.length) {
          let hookName: string | null = null
          let hookNode: TSESTree.Node | null = null

          if (node.callee.type === AST_NODE_TYPES.Identifier) {
            hookName = node.callee.name
            hookNode = node.callee
          } else if (node.callee.type === AST_NODE_TYPES.MemberExpression) {
            if (node.callee.property.type === AST_NODE_TYPES.Identifier) {
              hookName = node.callee.property.name
              hookNode = node.callee.property
            }
          }

          if (hookName) {
            const disallowedHook = options.disallowHooks.find(
              (hook) => hook.name === hookName,
            )

            if (disallowedHook && hookNode) {
              context.report({
                node,
                messageId: 'disallowedFunctionOrMethod',
                data: {
                  functionOrMethod: disallowedHook.name,
                  replacement: disallowedHook.replacement,
                },
                suggest: [
                  {
                    messageId: 'replace',
                    data: {
                      replacement: disallowedHook.replacement,
                    },
                    fix: (fixer) => {
                      return fixer.replaceText(
                        hookNode,
                        disallowedHook.replacement,
                      )
                    },
                  },
                ],
              })
            }
          }
        }

        // Check for disallowed methods in objects passed to hooks
        if (!options.disallowMethods?.length) return

        // Only check arguments of hooks
        if (!isHook(node.callee)) return

        // Look for object expressions in the arguments
        for (const arg of node.arguments) {
          if (arg.type === AST_NODE_TYPES.ObjectExpression) {
            // Check each property
            for (const prop of arg.properties) {
              if (
                prop.type === AST_NODE_TYPES.Property &&
                prop.key.type === AST_NODE_TYPES.Identifier
              ) {
                const methodName = prop.key.name

                // Check if the method is in the disallowed list
                const disallowedMethod = options.disallowMethods.find(
                  (method) => method.name === methodName,
                )

                if (disallowedMethod) {
                  // Check for requireTrueProp condition
                  if (disallowedMethod.requireTrueProp) {
                    const requiredPropName = disallowedMethod.requireTrueProp
                    let hasTrueProp = false

                    // Check if the object has the required prop set to true
                    for (const objProp of arg.properties) {
                      if (
                        objProp.type === AST_NODE_TYPES.Property &&
                        objProp.key.type === AST_NODE_TYPES.Identifier &&
                        objProp.key.name === requiredPropName &&
                        objProp.value.type === AST_NODE_TYPES.Literal &&
                        objProp.value.value === true
                      ) {
                        hasTrueProp = true
                        break
                      }
                    }

                    if (!hasTrueProp) {
                      context.report({
                        node: prop,
                        messageId: 'disallowedMethodWithMissingRequireTrueProp',
                        data: {
                          method: disallowedMethod.name,
                          requireTrueProp: requiredPropName,
                        },
                      })
                      continue
                    }
                  }

                  // Handle replacement if available
                  if (disallowedMethod.replacement) {
                    context.report({
                      node: prop,
                      messageId: 'disallowedFunctionOrMethod',
                      data: {
                        functionOrMethod: disallowedMethod.name,
                        replacement: disallowedMethod.replacement,
                      },
                      suggest: [
                        {
                          messageId: 'replace',
                          data: {
                            replacement: disallowedMethod.replacement,
                          },
                          fix: (fixer) => {
                            return fixer.replaceText(
                              prop.key,
                              disallowedMethod.replacement!,
                            )
                          },
                        },
                      ],
                    })
                  }
                }
              }
            }
          }
        }
      },
    }
  },
})

export const reactCompilerMigration = {
  name,
  rule,
}
