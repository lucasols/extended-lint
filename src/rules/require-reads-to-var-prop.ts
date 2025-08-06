import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  varsToCheck: z.array(
    z.object({
      selector: z.string().optional(),
      fromFnCall: z.string().optional(),
      prop: z.string().or(z.array(z.string())),
      errorMsg: z.string().optional(),
    }),
  ),
})

type Options = z.infer<typeof optionsSchema>

export const requireReadsToVarProp = createExtendedLintRule<
  [Options],
  'propNotRead' | 'propsNotRead'
>({
  name: 'require-reads-to-var-prop',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Require specific properties from variables to be read or ensure the variable is passed to functions/components',
    },
    messages: {
      propNotRead:
        '"{{fnName}}" requires reads to the prop "{{prop}}". {{customMsg}}',
      propsNotRead:
        '"{{fnName}}" requires reads to the props {{props}}. {{customMsg}}',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ varsToCheck: [] }],
  create(context, [options]) {
    const varsToTrack = new Map<
      string,
      {
        props: string[]
        errorMsg?: string
        node: TSESTree.Node
        fnName: string
      }
    >()

    // Helper function to check destructuring patterns
    const checkDestructuringPattern = (
      destructuringNode: TSESTree.ObjectPattern,
      requiredProps: string[],
      errorMsg: string | undefined,
      fnName: string,
    ) => {
      const destructuredProps = new Set<string>()

      for (const property of destructuringNode.properties) {
        if (
          property.type === AST_NODE_TYPES.Property &&
          property.key.type === AST_NODE_TYPES.Identifier
        ) {
          destructuredProps.add(property.key.name)
        }
      }

      // Check which required props are missing from destructuring
      const missingProps = requiredProps.filter(
        (prop) => !destructuredProps.has(prop),
      )

      if (missingProps.length > 0) {
        if (missingProps.length === 1) {
          context.report({
            node: destructuringNode,
            messageId: 'propNotRead',
            data: {
              prop: missingProps[0],
              fnName,
              customMsg: errorMsg || '',
            },
          })
        } else {
          const propsString = missingProps.map((prop) => `"${prop}"`).join(', ')
          context.report({
            node: destructuringNode,
            messageId: 'propsNotRead',
            data: {
              props: propsString,
              fnName,
              customMsg: errorMsg || '',
            },
          })
        }
      }
    }

    // Helper function to extract function name from call expression
    const getFunctionName = (node: TSESTree.VariableDeclarator): string => {
      if (node.init?.type === AST_NODE_TYPES.CallExpression) {
        if (node.init.callee.type === AST_NODE_TYPES.Identifier) {
          return node.init.callee.name
        }
        if (node.init.callee.type === AST_NODE_TYPES.MemberExpression) {
          if (node.init.callee.property.type === AST_NODE_TYPES.Identifier) {
            if (node.init.callee.object.type === AST_NODE_TYPES.Identifier) {
              return `${node.init.callee.object.name}.${node.init.callee.property.name}`
            }
            return node.init.callee.property.name
          }
        }
      }
      return 'unknown function'
    }

    // Create AST selectors for each configured check
    const selectors: Record<string, (node: TSESTree.Node) => void> = {}

    // Handle selector-based checks
    for (const check of options.varsToCheck) {
      if (check.selector) {
        selectors[check.selector] = (node: TSESTree.Node) => {
          // For variable declarations, we need to find the identifier
          if (
            node.type === AST_NODE_TYPES.VariableDeclarator &&
            node.id.type === AST_NODE_TYPES.Identifier
          ) {
            varsToTrack.set(node.id.name, {
              props: Array.isArray(check.prop) ? check.prop : [check.prop],
              errorMsg: check.errorMsg,
              node: node.id,
              fnName: getFunctionName(node),
            })
          }
          // Handle direct destructuring patterns
          if (
            node.type === AST_NODE_TYPES.VariableDeclarator &&
            node.id.type === AST_NODE_TYPES.ObjectPattern
          ) {
            const requiredProps = Array.isArray(check.prop)
              ? check.prop
              : [check.prop]
            checkDestructuringPattern(
              node.id,
              requiredProps,
              check.errorMsg,
              getFunctionName(node),
            )
          }
        }
      }
    }

    // Handle fromFnCall-based checks with specific selectors
    for (const check of options.varsToCheck) {
      if (check.fromFnCall) {
        const pattern = check.fromFnCall

        if (pattern.startsWith('*.')) {
          // Wildcard pattern: *.useElement -> VariableDeclarator > CallExpression > MemberExpression[property.name="useElement"]
          const methodName = pattern.slice(2)
          const selectorKey = `VariableDeclarator > CallExpression > MemberExpression[property.name="${methodName}"]`

          selectors[selectorKey] = (node: TSESTree.Node) => {
            if (node.type !== AST_NODE_TYPES.MemberExpression) return
            if (node.parent.type !== AST_NODE_TYPES.CallExpression) return
            const callExpr = node.parent
            if (callExpr.parent.type !== AST_NODE_TYPES.VariableDeclarator)
              return
            const varDeclarator = callExpr.parent

            if (varDeclarator.id.type === AST_NODE_TYPES.Identifier) {
              varsToTrack.set(varDeclarator.id.name, {
                props: Array.isArray(check.prop) ? check.prop : [check.prop],
                errorMsg: check.errorMsg,
                node: varDeclarator.id,
                fnName: check.fromFnCall || 'unknown function',
              })
            } else if (varDeclarator.id.type === AST_NODE_TYPES.ObjectPattern) {
              const requiredProps = Array.isArray(check.prop)
                ? check.prop
                : [check.prop]
              checkDestructuringPattern(
                varDeclarator.id,
                requiredProps,
                check.errorMsg,
                check.fromFnCall || 'unknown function',
              )
            }
          }
        } else if (pattern.includes('.')) {
          // Specific pattern: test.useElement
          const [objName, methodName] = pattern.split('.')
          const selectorKey = `VariableDeclarator > CallExpression > MemberExpression[object.name="${objName}"][property.name="${methodName}"]`

          selectors[selectorKey] = (node: TSESTree.Node) => {
            if (node.type !== AST_NODE_TYPES.MemberExpression) return
            if (node.parent.type !== AST_NODE_TYPES.CallExpression) return
            const callExpr = node.parent
            if (callExpr.parent.type !== AST_NODE_TYPES.VariableDeclarator)
              return
            const varDeclarator = callExpr.parent

            if (varDeclarator.id.type === AST_NODE_TYPES.Identifier) {
              varsToTrack.set(varDeclarator.id.name, {
                props: Array.isArray(check.prop) ? check.prop : [check.prop],
                errorMsg: check.errorMsg,
                node: varDeclarator.id,
                fnName: check.fromFnCall || 'unknown function',
              })
            } else if (varDeclarator.id.type === AST_NODE_TYPES.ObjectPattern) {
              const requiredProps = Array.isArray(check.prop)
                ? check.prop
                : [check.prop]
              checkDestructuringPattern(
                varDeclarator.id,
                requiredProps,
                check.errorMsg,
                check.fromFnCall || 'unknown function',
              )
            }
          }
        } else {
          // Simple function call: fnName
          const selectorKey = `VariableDeclarator > CallExpression > Identifier[name="${pattern}"]`

          selectors[selectorKey] = (node: TSESTree.Node) => {
            if (node.type !== AST_NODE_TYPES.Identifier) return
            if (node.parent.type !== AST_NODE_TYPES.CallExpression) return
            const callExpr = node.parent
            if (callExpr.parent.type !== AST_NODE_TYPES.VariableDeclarator)
              return
            const varDeclarator = callExpr.parent

            if (varDeclarator.id.type === AST_NODE_TYPES.Identifier) {
              varsToTrack.set(varDeclarator.id.name, {
                props: Array.isArray(check.prop) ? check.prop : [check.prop],
                errorMsg: check.errorMsg,
                node: varDeclarator.id,
                fnName: check.fromFnCall || 'unknown function',
              })
            } else if (varDeclarator.id.type === AST_NODE_TYPES.ObjectPattern) {
              const requiredProps = Array.isArray(check.prop)
                ? check.prop
                : [check.prop]
              checkDestructuringPattern(
                varDeclarator.id,
                requiredProps,
                check.errorMsg,
                check.fromFnCall || 'unknown function',
              )
            }
          }
        }
      }
    }

    return {
      ...selectors,

      'Program:exit'() {
        // Check each tracked variable to see if its required properties were accessed
        for (const [
          varName,
          { props, errorMsg, node, fnName },
        ] of varsToTrack) {
          const scope = context.sourceCode.getScope(node)
          const variable = scope.set.get(varName)

          if (variable) {
            const unreadProps = new Set(props)

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

              // Check if this reference accesses any of the required properties
              if (
                parent.type === AST_NODE_TYPES.MemberExpression &&
                parent.object === refNode &&
                parent.property.type === AST_NODE_TYPES.Identifier &&
                unreadProps.has(parent.property.name)
              ) {
                unreadProps.delete(parent.property.name)
                if (unreadProps.size === 0) break
                continue
              }

              // Check for destructuring that accesses any of the properties
              if (
                parent.type === AST_NODE_TYPES.VariableDeclarator &&
                parent.init === refNode &&
                parent.id.type === AST_NODE_TYPES.ObjectPattern
              ) {
                for (const property of parent.id.properties) {
                  if (
                    property.type === AST_NODE_TYPES.Property &&
                    property.key.type === AST_NODE_TYPES.Identifier &&
                    unreadProps.has(property.key.name)
                  ) {
                    unreadProps.delete(property.key.name)
                  }
                }
                if (unreadProps.size === 0) break
                continue
              }

              // If this is accessing a property that's not in our required list, skip this reference
              if (
                parent.type === AST_NODE_TYPES.MemberExpression &&
                parent.object === refNode &&
                parent.property.type === AST_NODE_TYPES.Identifier &&
                !props.includes(parent.property.name)
              ) {
                continue
              }

              // Any other reference (variable without member access) is considered valid
              // This means the entire variable was used, so all properties are considered read
              unreadProps.clear()
              break
            }

            // Report errors for any unread properties
            if (unreadProps.size > 0) {
              const unreadPropsArray = Array.from(unreadProps)
              if (unreadPropsArray.length === 1) {
                context.report({
                  node,
                  messageId: 'propNotRead',
                  data: {
                    prop: unreadPropsArray[0],
                    fnName,
                    customMsg: errorMsg || '',
                  },
                })
              } else {
                const propsString = unreadPropsArray
                  .map((prop) => `"${prop}"`)
                  .join(', ')
                context.report({
                  node,
                  messageId: 'propsNotRead',
                  data: {
                    props: propsString,
                    fnName,
                    customMsg: errorMsg || '',
                  },
                })
              }
            }
          }
        }
      },
    }
  },
})
