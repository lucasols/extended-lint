import { narrowStringToUnion } from '@ls-stack/utils/typingFnUtils'
import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import * as t from 'tschema'
import * as ts from 'typescript'
import * as TSESLint from '../../node_modules/.pnpm/@typescript-eslint+utils@8.22.0_eslint@9.19.0_typescript@5.7.3/node_modules/@typescript-eslint/utils/dist/ts-eslint'

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
        description: 'Disallow unnecessary typeof conditions with known types',
      },
      messages: {
        unnecessaryTypeofCondition:
          'Types has no overlap. The type of "{{name}}" is always "{{type}}".',
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

      function getPrimitiveTypeName(type: ts.Type): TypeofValue | null {
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
      ): Set<TypeofValue> | null {
        if (!checker) return null

        const tsNode = parserServices.esTreeNodeToTSNodeMap.get(node)

        const type = checker.getTypeAtLocation(tsNode)
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
            const primitiveType = getPrimitiveTypeName(unionType)

            if (primitiveType !== null) {
              possibleTypeofValues.add(primitiveType)
            } else {
              return null
            }
          }

          return possibleTypeofValues
        }

        // For non-union types
        const primitiveType = getPrimitiveTypeName(type)
        if (primitiveType !== null) {
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
                actualType: Array.from(possibleTypeofValues).join(' | '),
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
                actualType: Array.from(possibleTypeofValues).join(' | '),
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
              actualType: Array.from(possibleTypeofValues).join(' | '),
              conditionType,
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
