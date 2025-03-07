import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import * as t from 'tschema'
import * as ts from 'typescript'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'improved-no-unnecessary-condition'

const optionsSchema = t.object({})

type Options = t.Infer<typeof optionsSchema>

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
        description: 'Disallow unnecessary typeof conditions with known types',
      },
      messages: {
        unnecessaryTypeofCondition:
          'Unnecessary typeof condition. The type of "{{name}}" is always "{{type}}".',
        alwaysFalseTypeofCondition:
          'This condition will always be false. The type of "{{name}}" is "{{actualType}}" so the condition has no overlap with "{{expectedType}}".',
      },
      schema: [optionsSchema as any],
    },
    defaultOptions: [{}],
    create(context) {
      // Use try-catch to ensure we don't crash if TypeScript services aren't available
      try {
        const parserServices = ESLintUtils.getParserServices(context, true)
        const checker = parserServices.program?.getTypeChecker()

        // If the TypeScript services or program aren't available, return an empty object
        if (!checker || !parserServices.program) {
          return {}
        }

        function isTypeofExpression(
          node: TSESTree.Node,
        ): node is TSESTree.UnaryExpression {
          return (
            node.type === AST_NODE_TYPES.UnaryExpression &&
            node.operator === 'typeof' &&
            node.argument.type === AST_NODE_TYPES.Identifier
          )
        }

        function getPrimitiveTypeName(type: ts.Type): string | null {
          if (
            type.flags & ts.TypeFlags.String ||
            type.flags & ts.TypeFlags.StringLiteral
          ) {
            return 'string'
          }
          if (
            type.flags & ts.TypeFlags.Number ||
            type.flags & ts.TypeFlags.NumberLiteral
          ) {
            return 'number'
          }
          if (
            type.flags & ts.TypeFlags.Boolean ||
            type.flags & ts.TypeFlags.BooleanLiteral
          ) {
            return 'boolean'
          }
          if (type.flags & ts.TypeFlags.Null) {
            return 'object' // typeof null === 'object'
          }
          if (
            type.flags & ts.TypeFlags.Undefined ||
            type.flags & ts.TypeFlags.Void
          ) {
            return 'undefined'
          }
          if (type.flags & ts.TypeFlags.Object) {
            // Check if it's a function
            if (type.getCallSignatures().length > 0) {
              return 'function'
            }
            return 'object'
          }
          if (type.flags & ts.TypeFlags.ESSymbol) {
            return 'symbol'
          }
          if (type.flags & ts.TypeFlags.BigInt) {
            return 'bigint'
          }

          return null
        }

        function getTypeofOperandType(
          node: TSESTree.Expression,
        ): string | null {
          if (!checker) return null

          const tsNode = parserServices.esTreeNodeToTSNodeMap.get(node)
          const type = checker.getTypeAtLocation(tsNode)

          // Skip if type has union types (unless all union types map to the same typeof value)
          if (type.isUnion()) {
            const unionTypes: Array<string | null> = []

            for (const unionType of type.types) {
              unionTypes.push(getPrimitiveTypeName(unionType))
            }

            // Filter out null values
            const validTypes = unionTypes.filter((t): t is string => t !== null)

            // If all union types have the same typeof value, we can use that
            if (validTypes.length > 0) {
              const firstType = validTypes[0]
              const allSame = validTypes.every((t) => t === firstType)

              if (allSame) {
                return firstType
              }
            }

            return null
          }

          // Skip any/unknown types
          if (
            type.flags & ts.TypeFlags.Any ||
            type.flags & ts.TypeFlags.Unknown
          ) {
            return null
          }

          return getPrimitiveTypeName(type)
        }

        function checkBinaryExpression(node: TSESTree.BinaryExpression) {
          // Check if this is a typeof comparison (typeof x === 'string')
          if (
            !['===', '!==', '==', '!='].includes(node.operator) ||
            node.right.type !== AST_NODE_TYPES.Literal ||
            typeof node.right.value !== 'string'
          ) {
            return
          }

          let typeofNode: TSESTree.UnaryExpression | null = null
          const expectedType: string = node.right.value

          // Check if left side is typeof expression
          if (isTypeofExpression(node.left)) {
            typeofNode = node.left
          }

          if (!typeofNode) {
            return
          }

          const identifierNode = typeofNode.argument as TSESTree.Identifier
          const identifierName = identifierNode.name
          const actualType = getTypeofOperandType(identifierNode)

          if (actualType === null) {
            return // Cannot determine type statically
          }

          // For !== and != operators, invert the condition logic
          const isNegated = node.operator === '!==' || node.operator === '!='
          const conditionIsTrue = actualType === expectedType

          if (
            (!isNegated && conditionIsTrue) ||
            (isNegated && !conditionIsTrue)
          ) {
            context.report({
              node,
              messageId: 'unnecessaryTypeofCondition',
              data: {
                name: identifierName,
                type: actualType,
              },
            })
          } else if (
            (!isNegated && !conditionIsTrue) ||
            (isNegated && conditionIsTrue)
          ) {
            context.report({
              node,
              messageId: 'alwaysFalseTypeofCondition',
              data: {
                name: identifierName,
                actualType,
                expectedType,
              },
            })
          }
        }

        return {
          BinaryExpression: checkBinaryExpression,
        }
      } catch (error) {
        // If there's an error getting the parser services, return an empty object
        return {}
      }
    },
  }),
}
