import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import * as t from 'tschema'
import ts from 'typescript'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'improved-no-unnecessary-condition'

const optionsSchema = t.object({})

type Options = t.Infer<typeof optionsSchema>

const typeofValues = [
  'string',
  'number',
  'bigint',
  'boolean',
  'symbol',
  'undefined',
  'object',
  'function',
] as const

type TypeofValue = (typeof typeofValues)[number]

const validTypeofValues = new Set(typeofValues)

export const improvedNoUnnecessaryCondition = {
  name,
  rule: createRule<
    [Options],
    'unnecessaryTypeofCondition' | 'alwaysFalseTypeofCondition'
  >({
    name,
    meta: {
      type: 'suggestion',
      docs: {
        description: 'Extends checks of `no-unnecessary-condition` rule',
      },
      messages: {
        unnecessaryTypeofCondition:
          'This condition is unnecessary. The type of "{{name}}" is always "{{type}}".',
        alwaysFalseTypeofCondition:
          'This condition will always be false. The type of "{{name}}" is "{{actualType}}" so the condition has no overlap with "{{conditionType}}".',
      },
      schema: [optionsSchema as any],
    },
    defaultOptions: [{}],
    create(context) {
      const parserServices = ESLintUtils.getParserServices(context, true)
      const checker = parserServices.program?.getTypeChecker()

      // If the TypeScript services or program aren't available, return an empty object
      if (!checker || !parserServices.program) {
        throw new Error('TypeScript services or program not available')
      }

      function isTypeofExpression(
        node: TSESTree.Node,
      ): node is TSESTree.UnaryExpression {
        return (
          node.type === AST_NODE_TYPES.UnaryExpression &&
          node.operator === 'typeof'
        )
      }

      function getTypeOfFromType(type: ts.Type): TypeofValue | null {
        if (
          type.flags & ts.TypeFlags.Any ||
          type.flags & ts.TypeFlags.Unknown
        ) {
          return null
        }

        // For string types
        if (type.flags & ts.TypeFlags.StringLike) {
          return 'string'
        }

        // For number types
        if (type.flags & ts.TypeFlags.NumberLike) {
          return 'number'
        }

        // For bigint types
        if (type.flags & ts.TypeFlags.BigIntLike) {
          return 'bigint'
        }

        // For boolean types
        if (type.flags & ts.TypeFlags.BooleanLike) {
          return 'boolean'
        }

        // For symbol types - check different possible symbol flags
        if (type.flags & ts.TypeFlags.ESSymbolLike) {
          return 'symbol'
        }

        // For undefined types
        if (
          type.flags & ts.TypeFlags.Undefined ||
          type.flags & ts.TypeFlags.Void
        ) {
          return 'undefined'
        }

        // For null types
        if (type.flags & ts.TypeFlags.Null) {
          return 'object'
        }

        // For function types - check if it has call signatures
        if (type.getCallSignatures().length > 0) {
          return 'function'
        }

        // For object types (including arrays, etc.)
        if (type.flags & ts.TypeFlags.Object) {
          return 'object'
        }

        // For all other non-primitive types
        if (type.flags & ts.TypeFlags.NonPrimitive) {
          return 'object'
        }

        // For other types not explicitly handled
        return null
      }

      function getTypeofOperandType(
        node: TSESTree.Expression,
      ): Set<TypeofValue> | null {
        if (!checker) return null

        const tsNode = parserServices.esTreeNodeToTSNodeMap.get(
          node,
        ) as ts.TypeOfExpression

        const type = checker.getTypeAtLocation(tsNode.expression)

        // Skip any/unknown types
        if (
          type.flags & ts.TypeFlags.Any ||
          type.flags & ts.TypeFlags.Unknown
        ) {
          return null
        }

        // For all types, collect possible typeof values
        const possibleTypeofValues = new Set<TypeofValue>()

        // Handle union types
        if (type.isUnion()) {
          for (const unionType of type.types) {
            const primitiveType = getTypeOfFromType(unionType)

            if (primitiveType) {
              possibleTypeofValues.add(primitiveType)
            } else {
              return null
            }
          }

          return possibleTypeofValues
        }

        // For non-union types
        const primitiveType = getTypeOfFromType(type)

        if (primitiveType) {
          possibleTypeofValues.add(primitiveType)
        }

        return possibleTypeofValues
      }

      function checkBinaryExpression(node: TSESTree.BinaryExpression) {
        const isValidOperator =
          node.operator === '===' || node.operator === '!=='

        if (!isValidOperator) {
          return
        }

        // Special case for "non-nullable typeof check" pattern like "value && typeof value === 'string'"
        // This is a valid pattern, so we shouldn't report an error
        if (
          node.parent.type === AST_NODE_TYPES.LogicalExpression &&
          node.parent.operator === '&&' &&
          node.parent.left.type === AST_NODE_TYPES.Identifier
        ) {
          // Check if the left operand of && is the same as the operand of typeof
          const leftIdentifier = node.parent.left
          let typeofOperand = null

          if (isTypeofExpression(node.left)) {
            typeofOperand = node.left.argument
          } else if (isTypeofExpression(node.right)) {
            typeofOperand = node.right.argument
          }

          if (
            typeofOperand?.type === AST_NODE_TYPES.Identifier &&
            typeofOperand.name === leftIdentifier.name
          ) {
            return // Don't report errors for this pattern
          }
        }

        let typeOfNode: TSESTree.UnaryExpression | null = null
        let conditionType: string | null = null

        if (isTypeofExpression(node.left)) {
          typeOfNode = node.left
          conditionType =
            node.right.type === AST_NODE_TYPES.Literal &&
            typeof node.right.value === 'string'
              ? node.right.value
              : null
        } else if (isTypeofExpression(node.right)) {
          typeOfNode = node.right
          conditionType =
            node.left.type === AST_NODE_TYPES.Literal &&
            typeof node.left.value === 'string'
              ? node.left.value
              : null
        }

        if (!typeOfNode || !conditionType) return

        if (!narrowStringToUnion(conditionType, validTypeofValues)) {
          return
        }

        // Since we've validated conditionType, we know it's one of the valid TypeofValues
        const possibleTypeofValues = getTypeofOperandType(typeOfNode)

        if (!possibleTypeofValues) return

        const isNegated = node.operator === '!=='

        // Check if the condition is always true or always false
        const conditionTypeIncluded = possibleTypeofValues.has(conditionType)

        // For === operator: condition is unnecessary if it's always true
        // For !== operator: condition is unnecessary if it's always false
        if (possibleTypeofValues.size === 1) {
          // Only one possible type, so either always true or always false
          if (conditionTypeIncluded && !isNegated) {
            // typeof x === 'string' where x is always string
            context.report({
              node,
              messageId: 'unnecessaryTypeofCondition',
              data: {
                name: getNodeText(typeOfNode, context),
                type: conditionType,
              },
            })
          } else if (!conditionTypeIncluded && isNegated) {
            // typeof x !== 'number' where x is always string
            context.report({
              node,
              messageId: 'unnecessaryTypeofCondition',
              data: {
                name: getNodeText(typeOfNode, context),
                type: Array.from(possibleTypeofValues)[0],
              },
            })
          } else if (!conditionTypeIncluded && !isNegated) {
            // typeof x === 'number' where x is always string
            context.report({
              node,
              messageId: 'alwaysFalseTypeofCondition',
              data: {
                name: getNodeText(typeOfNode, context),
                actualType:
                  Array.from(possibleTypeofValues).join(' | ') || 'never',
                conditionType,
              },
            })
          } else if (conditionTypeIncluded && isNegated) {
            // typeof x !== 'string' where x is always string
            context.report({
              node,
              messageId: 'alwaysFalseTypeofCondition',
              data: {
                name: getNodeText(typeOfNode, context),
                actualType:
                  Array.from(possibleTypeofValues).join(' | ') || 'never',
                conditionType,
              },
            })
          }
        } else if (!conditionTypeIncluded && !isNegated) {
          // Multiple possible types, but none match the condition
          // e.g., typeof x === 'boolean' where x is string | number
          context.report({
            node,
            messageId: 'alwaysFalseTypeofCondition',
            data: {
              name: getNodeText(typeOfNode, context),
              actualType:
                Array.from(possibleTypeofValues).join(' | ') || 'never',
              conditionType,
            },
          })
        } else if (!conditionTypeIncluded && isNegated) {
          // Multiple possible types, and we're checking for a type that isn't included
          // e.g., typeof x !== 'object' where x is string | number | boolean
          context.report({
            node,
            messageId: 'unnecessaryTypeofCondition',
            data: {
              name: getNodeText(typeOfNode, context),
              type: Array.from(possibleTypeofValues).join(' | '),
            },
          })
        }
      }

      return {
        BinaryExpression: checkBinaryExpression,
      }
    },
  }),
}

function getNodeText(
  typeOfNode: TSESTree.UnaryExpression,
  context: TSESLint.RuleContext<string, unknown[]>,
) {
  const operand = typeOfNode.argument
  const variableName =
    operand.type === AST_NODE_TYPES.Identifier
      ? operand.name
      : context.sourceCode.getText(operand)

  return variableName
}

function narrowStringToUnion<T extends string>(
  value: string,
  validValues: Set<T>,
): value is T {
  return validValues.has(value as T)
}
