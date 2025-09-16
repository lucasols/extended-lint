import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import ts from 'typescript'
import { z } from 'zod/v4'
import { traverseAST } from '../astUtils'
import { getJsonSchemaFromZod } from '../createRule'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'improved-no-unnecessary-condition'

const optionsSchema = z.object({})

const typeofValues = [
  'string',
  'number',
  'bigint',
  'boolean',
  'symbol',
  'undefined',
  'object',
  'function',
  'never',
] as const

type TypeofValue = (typeof typeofValues)[number]

const validTypeofValues = new Set(typeofValues)

type Options = z.infer<typeof optionsSchema>

export const improvedNoUnnecessaryCondition = {
  name,
  rule: createRule<
    [Options],
    | 'unnecessaryTypeofCondition'
    | 'alwaysFalseTypeofCondition'
    | 'unnecessaryStartsWithCondition'
    | 'alwaysFalseStartsWithCondition'
    | 'unnecessaryEndsWithCondition'
    | 'alwaysFalseEndsWithCondition'
    | 'unnecessaryIncludesCondition'
    | 'alwaysFalseIncludesCondition'
    | 'unnecessaryLengthCondition'
    | 'alwaysFalseLengthCondition'
    | 'unnecessaryInCondition'
    | 'alwaysFalseInCondition'
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
        unnecessaryStartsWithCondition:
          'This startsWith check is unnecessary as it always evaluates to true.',
        alwaysFalseStartsWithCondition:
          'This startsWith check will always be false.',
        unnecessaryEndsWithCondition:
          'This endsWith check is unnecessary as it always evaluates to true.',
        alwaysFalseEndsWithCondition:
          'This endsWith check will always be false.',
        unnecessaryIncludesCondition:
          'This includes check is unnecessary as it always evaluates to true.',
        alwaysFalseIncludesCondition:
          'This includes check will always be false.',
        unnecessaryLengthCondition:
          'This length comparison is unnecessary as it always evaluates to true.',
        alwaysFalseLengthCondition:
          'This length comparison will always be false.',
        unnecessaryInCondition:
          'This in check is unnecessary. Property "{{property}}" always exists on type "{{type}}".',
        alwaysFalseInCondition:
          'This in check will always be false. Property "{{property}}" does not exist on type "{{type}}".',
      },
      schema: [getJsonSchemaFromZod(optionsSchema)],
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
        // eslint-disable-next-line @ls-stack/no-type-guards -- necessary type guard for AST node type checking
      ): node is TSESTree.UnaryExpression {
        return (
          node.type === AST_NODE_TYPES.UnaryExpression &&
          node.operator === 'typeof'
        )
      }

      function getTypeOfFromType(type: ts.Type): TypeofValue[] | null {
        if (
          type.flags & ts.TypeFlags.Any ||
          type.flags & ts.TypeFlags.Unknown
        ) {
          return null
        }

        // For string types
        if (type.flags & ts.TypeFlags.StringLike) {
          return ['string']
        }

        // For number types
        if (type.flags & ts.TypeFlags.NumberLike) {
          return ['number']
        }

        // For bigint types
        if (type.flags & ts.TypeFlags.BigIntLike) {
          return ['bigint']
        }

        // For boolean types
        if (type.flags & ts.TypeFlags.BooleanLike) {
          return ['boolean']
        }

        // For symbol types - check different possible symbol flags
        if (type.flags & ts.TypeFlags.ESSymbolLike) {
          return ['symbol']
        }

        // For undefined types
        if (
          type.flags & ts.TypeFlags.Undefined ||
          type.flags & ts.TypeFlags.Void
        ) {
          return ['undefined']
        }

        // For null types
        if (type.flags & ts.TypeFlags.Null) {
          return ['object']
        }

        // For function types - check if it has call signatures
        if (type.getCallSignatures().length > 0) {
          return ['function']
        }

        // For object types (including arrays, etc.)
        if (type.flags & ts.TypeFlags.Object) {
          // Non-nullable type - {}
          if (type.getProperties().length === 0) {
            return [
              'string',
              'number',
              'bigint',
              'boolean',
              'symbol',
              'object',
              'function',
            ]
          }

          return ['object']
        }

        // For all other non-primitive types
        if (type.flags & ts.TypeFlags.NonPrimitive) {
          return ['object']
        }

        if (type.flags & ts.TypeFlags.Never) {
          return ['never']
        }

        // For other types not explicitly handled
        return null
      }

      function getTypeofOperandType(
        node: TSESTree.Expression,
      ): TypeofValue[] | null {
        if (!checker) return null

        // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- safe conversion from ESTree to TS node
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
        const possibleTypeofValues: TypeofValue[] = []

        // Handle union types
        if (type.isUnion()) {
          for (const unionType of type.types) {
            const primitiveType = getTypeOfFromType(unionType)

            if (primitiveType) {
              possibleTypeofValues.push(...primitiveType)
            } else {
              return null
            }
          }

          return possibleTypeofValues
        }

        // For non-union types
        const primitiveType = getTypeOfFromType(type)

        if (primitiveType) {
          possibleTypeofValues.push(...primitiveType)
          return possibleTypeofValues
        } else {
          return null
        }
      }

      function checkBinaryExpression(node: TSESTree.BinaryExpression) {
        const isValidOperator =
          node.operator === '===' || node.operator === '!=='

        if (!isValidOperator) return

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
        const conditionTypeIncluded =
          possibleTypeofValues.includes(conditionType)

        // For === operator: condition is unnecessary if it's always true
        // For !== operator: condition is unnecessary if it's always false
        if (possibleTypeofValues.length === 1) {
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
                actualType: getActualType(possibleTypeofValues),
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
                actualType: getActualType(possibleTypeofValues),
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
              actualType: getActualType(possibleTypeofValues),
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
              type: getActualType(possibleTypeofValues),
            },
          })
        }
      }

      function getStringLiteralValuesFromType(type: ts.Type): string[] | null {
        if (
          type.flags & ts.TypeFlags.Any ||
          type.flags & ts.TypeFlags.Unknown
        ) {
          return null
        }

        if (type.isUnion()) {
          const values: string[] = []
          for (const t of type.types) {
            if (t.flags & ts.TypeFlags.StringLiteral) {
              // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- safe access to StringLiteralType value
              const literal = (t as ts.StringLiteralType).value
              values.push(literal)
            } else {
              return null
            }
          }
          return values
        }

        if (type.flags & ts.TypeFlags.StringLiteral) {
          // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- safe access to StringLiteralType value
          const literal = (type as ts.StringLiteralType).value
          return [literal]
        }

        return null
      }

      function getStringLiteralValuesFromExpression(
        node: TSESTree.Expression,
      ): string[] | null {
        if (!checker) return null
        // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- safe conversion from ESTree to TS node
        const tsNode = parserServices.esTreeNodeToTSNodeMap.get(
          node,
        ) as ts.Expression
        const typeAtLoc = checker.getTypeAtLocation(tsNode)
        const fromLoc = getStringLiteralValuesFromType(typeAtLoc)
        if (fromLoc) return fromLoc

        const symbol = checker.getSymbolAtLocation(tsNode)
        if (symbol) {
          const typeOfSym = checker.getTypeOfSymbolAtLocation(symbol, tsNode)
          const fromSym = getStringLiteralValuesFromType(typeOfSym)
          if (fromSym) return fromSym
        }

        return null
      }

      function getStringLiteralValuesFromIdentifierAnnotation(
        id: TSESTree.Identifier,
      ): string[] | null {
        const programNode = context.sourceCode.ast
        let found: string[] | null = null

        traverseAST(
          programNode,
          (n) => {
            if (
              n.type === AST_NODE_TYPES.VariableDeclarator &&
              n.id.type === AST_NODE_TYPES.Identifier &&
              n.id.name === id.name &&
              n.id.typeAnnotation
            ) {
              const t = n.id.typeAnnotation.typeAnnotation
              if (t.type === AST_NODE_TYPES.TSUnionType) {
                const values: string[] = []
                for (const tt of t.types) {
                  if (
                    tt.type === AST_NODE_TYPES.TSLiteralType &&
                    tt.literal.type === AST_NODE_TYPES.Literal &&
                    typeof tt.literal.value === 'string'
                  ) {
                    values.push(tt.literal.value)
                  } else {
                    return true
                  }
                }
                found = values
                return true
              }
              if (
                t.type === AST_NODE_TYPES.TSLiteralType &&
                t.literal.type === AST_NODE_TYPES.Literal &&
                typeof t.literal.value === 'string'
              ) {
                found = [t.literal.value]
                return true
              }
            }
            return false
          },
          context.sourceCode,
        )

        return found
      }

      function checkStringAssertions(node: TSESTree.CallExpression) {
        if (
          node.callee.type !== AST_NODE_TYPES.MemberExpression ||
          node.callee.property.type !== AST_NODE_TYPES.Identifier ||
          node.callee.computed
        )
          return

        const method = node.callee.property.name
        if (
          method !== 'startsWith' &&
          method !== 'endsWith' &&
          method !== 'includes'
        )
          return

        if (node.arguments.length !== 1) return
        const [firstArg] = node.arguments
        if (
          !firstArg ||
          firstArg.type !== AST_NODE_TYPES.Literal ||
          typeof firstArg.value !== 'string'
        )
          return

        const objectExpr = node.callee.object
        let possibleValues = getStringLiteralValuesFromExpression(objectExpr)
        if (!possibleValues && objectExpr.type === AST_NODE_TYPES.Identifier) {
          possibleValues =
            getStringLiteralValuesFromIdentifierAnnotation(objectExpr)
        }
        if (!possibleValues || possibleValues.length === 0) return

        const search = firstArg.value
        if (method === 'includes' && possibleValues.length > 1) return
        let trues = 0
        let falses = 0
        for (const v of possibleValues) {
          const result =
            method === 'startsWith'
              ? v.startsWith(search)
              : method === 'endsWith'
              ? v.endsWith(search)
              : v.includes(search)
          if (result) trues++
          else falses++
          if (trues > 0 && falses > 0) return
        }

        if (trues > 0 && falses === 0) {
          const id =
            method === 'startsWith'
              ? 'unnecessaryStartsWithCondition'
              : method === 'endsWith'
              ? 'unnecessaryEndsWithCondition'
              : 'unnecessaryIncludesCondition'
          context.report({ node, messageId: id })
        } else if (falses > 0 && trues === 0) {
          const id =
            method === 'startsWith'
              ? 'alwaysFalseStartsWithCondition'
              : method === 'endsWith'
              ? 'alwaysFalseEndsWithCondition'
              : 'alwaysFalseIncludesCondition'
          context.report({ node, messageId: id })
        }
      }

      function checkStringLengthComparison(node: TSESTree.BinaryExpression) {
        const numericOperators = new Set(['===', '!==', '>', '>=', '<', '<='])
        if (!numericOperators.has(node.operator)) return

        function getLengthInfo(
          expr: TSESTree.Node,
        ): { values: number[] } | null {
          if (
            expr.type !== AST_NODE_TYPES.MemberExpression ||
            expr.computed ||
            expr.property.type !== AST_NODE_TYPES.Identifier ||
            expr.property.name !== 'length'
          )
            return null
          const obj = expr.object
          const strings = getStringLiteralValuesFromExpression(
            // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- acceptable cast for AST union
            obj as unknown as TSESTree.Expression,
          )
          if (!strings || strings.length === 0) return null
          const lengths: number[] = []
          for (const s of strings) lengths.push(s.length)
          return { values: lengths }
        }

        const leftLen = getLengthInfo(node.left)
        const rightLen = getLengthInfo(node.right)

        let lens: number[] | null = null
        let literal: number | null = null

        if (leftLen) {
          if (
            node.right.type === AST_NODE_TYPES.Literal &&
            typeof node.right.value === 'number'
          ) {
            lens = leftLen.values
            literal = node.right.value
          }
        } else if (rightLen) {
          if (
            node.left.type === AST_NODE_TYPES.Literal &&
            typeof node.left.value === 'number'
          ) {
            lens = rightLen.values
            literal = node.left.value
          }
        }

        if (!lens || literal === null) return

        let trues = 0
        let falses = 0
        for (const l of lens) {
          let result = false
          if (node.operator === '===') result = l === literal
          else if (node.operator === '!==') result = l !== literal
          else if (node.operator === '>') result = l > literal
          else if (node.operator === '>=') result = l >= literal
          else if (node.operator === '<') result = l < literal
          else if (node.operator === '<=') result = l <= literal
          if (result) trues++
          else falses++
          if (trues > 0 && falses > 0) return
        }

        if (trues > 0 && falses === 0) {
          context.report({ node, messageId: 'unnecessaryLengthCondition' })
        } else if (falses > 0 && trues === 0) {
          context.report({ node, messageId: 'alwaysFalseLengthCondition' })
        }
      }

      function getLiteralKeyFromExpression(
        expression: TSESTree.Expression,
      ): string | null {
        if (
          expression.type === AST_NODE_TYPES.Literal &&
          typeof expression.value === 'string'
        ) {
          return expression.value
        }

        const values = getStringLiteralValuesFromExpression(expression)
        if (!values) return null
        if (values.length !== 1) return null

        const [value] = values
        return value ?? null
      }

      type PropertyPresence = 'required' | 'optional' | 'absent' | 'unknown'

      function isIntersectionType(type: ts.Type) {
        return (type.flags & ts.TypeFlags.Intersection) !== 0
      }

      function isObjectType(type: ts.Type) {
        return (type.flags & ts.TypeFlags.Object) !== 0
      }

      function hasIndexSignatures(type: ts.ObjectType) {
        if (!checker) return true
        const stringIndex = checker.getIndexTypeOfType(type, ts.IndexKind.String)
        if (stringIndex) return true
        const numberIndex = checker.getIndexTypeOfType(type, ts.IndexKind.Number)
        return Boolean(numberIndex)
      }

      function getPropertyPresenceForObject(
        type: ts.ObjectType,
        propertyName: string,
      ): PropertyPresence {
        if (!checker) return 'unknown'
        if (hasIndexSignatures(type)) return 'unknown'

        if (type.getProperties().length === 0) return 'unknown'

        const property = checker.getPropertyOfType(type, propertyName)
        if (!property) return 'absent'

        if (property.flags & ts.SymbolFlags.Optional) return 'optional'

        return 'required'
      }

      function getPropertyPresence(
        type: ts.Type,
        propertyName: string,
      ): PropertyPresence {
        if (
          type.flags & ts.TypeFlags.Any ||
          type.flags & ts.TypeFlags.Unknown ||
          type.flags & ts.TypeFlags.Never ||
          type.flags & ts.TypeFlags.TypeParameter ||
          type.flags & ts.TypeFlags.IndexedAccess ||
          type.flags & ts.TypeFlags.StringLike ||
          type.flags & ts.TypeFlags.NumberLike ||
          type.flags & ts.TypeFlags.BigIntLike ||
          type.flags & ts.TypeFlags.BooleanLike ||
          type.flags & ts.TypeFlags.EnumLike ||
          type.flags & ts.TypeFlags.ESSymbolLike ||
          type.flags & ts.TypeFlags.NonPrimitive
        ) {
          return 'unknown'
        }

        if (type.flags & ts.TypeFlags.Null) return 'unknown'
        if (type.flags & ts.TypeFlags.Undefined) return 'unknown'
        if (type.flags & ts.TypeFlags.Void) return 'unknown'

        if (type.isUnion()) {
          let allRequired = true
          let allAbsent = true

          for (const unionType of type.types) {
            const presence = getPropertyPresence(unionType, propertyName)
            if (presence === 'unknown') return 'unknown'
            if (presence !== 'required') allRequired = false
            if (presence !== 'absent') allAbsent = false
            if (presence === 'optional') return 'unknown'
          }

          if (allRequired) return 'required'
          if (allAbsent) return 'absent'

          return 'unknown'
        }

        if (isIntersectionType(type)) {
          // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- safe narrowing for intersection types
          const intersectionType = type as ts.IntersectionType
          let hasRequired = false
          let hasOptional = false

          for (const intersectionPart of intersectionType.types) {
            const presence = getPropertyPresence(intersectionPart, propertyName)
            if (presence === 'unknown') return 'unknown'
            if (presence === 'required') {
              hasRequired = true
            } else if (presence === 'optional') {
              hasOptional = true
            }
          }

          if (hasRequired) return 'required'
          if (hasOptional) return 'optional'

          return 'absent'
        }

        if (!isObjectType(type)) return 'unknown'

        // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- object flag ensures safe cast
        return getPropertyPresenceForObject(type as ts.ObjectType, propertyName)
      }

      function checkInCondition(node: TSESTree.BinaryExpression) {
        if (node.operator !== 'in') return

        if (node.left.type === AST_NODE_TYPES.PrivateIdentifier) return

        const propertyName = getLiteralKeyFromExpression(node.left)
        if (!propertyName) return

        const right = node.right

        // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- conversion between ESTree and TS nodes
        const tsRight = parserServices.esTreeNodeToTSNodeMap.get(
          right,
        ) as ts.Expression

        if (!checker) return
        const type = checker.getTypeAtLocation(tsRight)
        const presence = getPropertyPresence(type, propertyName)

        if (presence === 'required') {
          context.report({
            node,
            messageId: 'unnecessaryInCondition',
            data: {
              property: propertyName,
              type: checker.typeToString(type),
            },
          })
        } else if (presence === 'absent') {
          context.report({
            node,
            messageId: 'alwaysFalseInCondition',
            data: {
              property: propertyName,
              type: checker.typeToString(type),
            },
          })
        }
      }

      return {
        BinaryExpression(node) {
          checkBinaryExpression(node)
          checkStringLengthComparison(node)
          checkInCondition(node)
        },
        CallExpression: checkStringAssertions,
      }
    },
  }),
}

function getActualType(possibleTypeofValues: TypeofValue[]): unknown {
  return Array.from(new Set(possibleTypeofValues)).join(' | ') || 'never'
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
  // eslint-disable-next-line @ls-stack/no-type-guards -- necessary type guard for type narrowing check
): value is T {
  // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- safe cast for type narrowing check
  return validValues.has(value as T)
}
