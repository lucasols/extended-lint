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
        'Require specific properties from variables to be read or ensure the variable is passed to functions/components',
    },
    messages: {
      propNotRead:
        'Property "{{prop}}" from variable "{{varName}}" is never read. {{customMsg}}',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ varsToCheck: [] }],
  create(context, [options]) {
    const varsToTrack = new Map<
      string,
      { prop: string; errorMsg?: string; node: TSESTree.Node }
    >()


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
              prop: check.prop,
              errorMsg: check.errorMsg,
              node: node.id,
            })
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
            const callExpr = node.parent as TSESTree.CallExpression
            const varDeclarator = callExpr.parent as TSESTree.VariableDeclarator
            
            if (varDeclarator.id.type === AST_NODE_TYPES.Identifier) {
              varsToTrack.set(varDeclarator.id.name, {
                prop: check.prop,
                errorMsg: check.errorMsg,
                node: varDeclarator.id,
              })
            }
          }
        } else if (pattern.includes('.')) {
          // Specific pattern: test.useElement
          const [objName, methodName] = pattern.split('.')
          const selectorKey = `VariableDeclarator > CallExpression > MemberExpression[object.name="${objName}"][property.name="${methodName}"]`
          
          selectors[selectorKey] = (node: TSESTree.Node) => {
            if (node.type !== AST_NODE_TYPES.MemberExpression) return
            const callExpr = node.parent as TSESTree.CallExpression
            const varDeclarator = callExpr.parent as TSESTree.VariableDeclarator
            
            if (varDeclarator.id.type === AST_NODE_TYPES.Identifier) {
              varsToTrack.set(varDeclarator.id.name, {
                prop: check.prop,
                errorMsg: check.errorMsg,
                node: varDeclarator.id,
              })
            }
          }
        } else {
          // Simple function call: fnName
          const selectorKey = `VariableDeclarator > CallExpression > Identifier[name="${pattern}"]`
          
          selectors[selectorKey] = (node: TSESTree.Node) => {
            if (node.type !== AST_NODE_TYPES.Identifier) return
            const callExpr = node.parent as TSESTree.CallExpression
            const varDeclarator = callExpr.parent as TSESTree.VariableDeclarator
            
            if (varDeclarator.id.type === AST_NODE_TYPES.Identifier) {
              varsToTrack.set(varDeclarator.id.name, {
                prop: check.prop,
                errorMsg: check.errorMsg,
                node: varDeclarator.id,
              })
            }
          }
        }
      }
    }

    return {
      ...selectors,

      'Program:exit'() {
        // Check each tracked variable to see if its required property was accessed
        for (const [varName, { prop, errorMsg, node }] of varsToTrack) {
          const scope = context.sourceCode.getScope(node)
          const variable = scope.set.get(varName)

          if (variable) {
            let propWasRead = false

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

              // Check if this reference accesses the required property
              if (
                parent.type === AST_NODE_TYPES.MemberExpression &&
                parent.object === refNode &&
                parent.property.type === AST_NODE_TYPES.Identifier &&
                parent.property.name === prop
              ) {
                propWasRead = true
                break
              }

              // Check for destructuring that accesses the property
              if (
                parent.type === AST_NODE_TYPES.VariableDeclarator &&
                parent.init === refNode &&
                parent.id.type === AST_NODE_TYPES.ObjectPattern
              ) {
                for (const property of parent.id.properties) {
                  if (
                    property.type === AST_NODE_TYPES.Property &&
                    property.key.type === AST_NODE_TYPES.Identifier &&
                    property.key.name === prop
                  ) {
                    propWasRead = true
                    break
                  }
                }
                if (propWasRead) break
              }

              // If this is accessing a different property, skip this reference
              if (
                parent.type === AST_NODE_TYPES.MemberExpression &&
                parent.object === refNode &&
                parent.property.type === AST_NODE_TYPES.Identifier &&
                parent.property.name !== prop
              ) {
                continue
              }

              // Any other reference (variable without member access) is considered valid
              propWasRead = true
              break
            }

            if (!propWasRead) {
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
