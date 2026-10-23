import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESTree,
} from '@typescript-eslint/utils'
import * as ts from 'typescript'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  additionalCastFunctions: z
    .array(
      z.object({
        name: z.string(),
        expectedType: z.enum(['string', 'number']),
      }),
    )
    .optional(),
  allowNullishCasting: z.boolean().optional(),
  allowInvalidCasting: z.boolean().optional(),
  allowAnyCasting: z.boolean().optional(),
  allowUnknownCasting: z.boolean().optional(),
})

type CastFunction = {
  expectedType: 'string' | 'number'
  builtin: boolean
}

const simpleArgTypes = new Set<AST_NODE_TYPES>([
  AST_NODE_TYPES.Identifier,
  AST_NODE_TYPES.Literal,
  AST_NODE_TYPES.TemplateLiteral,
  AST_NODE_TYPES.MemberExpression,
  AST_NODE_TYPES.CallExpression,
])

const nullishTypeFlags =
  ts.TypeFlags.Null | ts.TypeFlags.Undefined | ts.TypeFlags.Void

function unnecessaryFixNeedsParens(
  arg: TSESTree.Expression,
  node: TSESTree.CallExpression,
): boolean {
  if (simpleArgTypes.has(arg.type)) return false

  if (arg.type === AST_NODE_TYPES.SequenceExpression) return true

  const parent = node.parent

  switch (parent.type) {
    case AST_NODE_TYPES.VariableDeclarator:
    case AST_NODE_TYPES.ExpressionStatement:
    case AST_NODE_TYPES.ReturnStatement:
    case AST_NODE_TYPES.ArrowFunctionExpression:
    case AST_NODE_TYPES.Property:
    case AST_NODE_TYPES.ArrayExpression:
    case AST_NODE_TYPES.AssignmentExpression:
    case AST_NODE_TYPES.JSXExpressionContainer:
      return false
    case AST_NODE_TYPES.CallExpression:
    case AST_NODE_TYPES.NewExpression:
      return parent.callee === node
    default:
      return true
  }
}

type Options = z.infer<typeof optionsSchema>

export const strictCasting = createExtendedLintRule<
  [Options],
  | 'unnecessaryCasting'
  | 'nullishStringCasting'
  | 'nullishNumberCasting'
  | 'symbolNumberCasting'
  | 'nanNumberCasting'
  | 'anyCasting'
  | 'unknownCasting'
  | 'addNullishFallback'
>({
  name: 'strict-casting',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow unnecessary, nullish and invalid String()/Number() casts',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
    fixable: 'code',
    hasSuggestions: true,
    messages: {
      unnecessaryCasting:
        'Unnecessary {{name}}() cast of a value that is already of type {{type}}',
      nullishStringCasting:
        "String() cast of a possibly nullish value results in the strings 'null' or 'undefined', handle the nullish case explicitly",
      nullishNumberCasting:
        'Number() cast of a possibly nullish value results in 0 for null and NaN for undefined, handle the nullish case explicitly',
      symbolNumberCasting: 'Number() cast of a symbol throws at runtime',
      nanNumberCasting:
        'Number() cast of a value of type "{{type}}" will result in NaN',
      anyCasting:
        '{{name}}() cast of an `any` value is unsafe, type the value before casting',
      unknownCasting:
        '{{name}}() cast of an `unknown` value is unsafe, narrow the value before casting',
      addNullishFallback:
        'Add a `?? {{fallback}}` fallback to handle the nullish case',
    },
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const allowNullishCasting = options.allowNullishCasting ?? false
    const allowInvalidCasting = options.allowInvalidCasting ?? false
    const allowAnyCasting = options.allowAnyCasting ?? false
    const allowUnknownCasting = options.allowUnknownCasting ?? false

    const castFunctions = new Map<string, CastFunction>([
      ['String', { expectedType: 'string', builtin: true }],
      ['Number', { expectedType: 'number', builtin: true }],
    ])

    for (const fn of options.additionalCastFunctions ?? []) {
      castFunctions.set(fn.name, {
        expectedType: fn.expectedType,
        builtin: false,
      })
    }

    const services = ESLintUtils.getParserServices(context)
    const checker = services.program.getTypeChecker()

    function resolveUnionParts(type: ts.Type): ts.Type[] | undefined {
      const rawParts = type.isUnion() ? type.types : [type]

      const parts: ts.Type[] = []

      for (const rawPart of rawParts) {
        if (rawPart.isTypeParameter()) {
          const constraint = checker.getBaseConstraintOfType(rawPart)

          if (!constraint) return undefined

          if (constraint.isUnion()) {
            parts.push(...constraint.types)
          } else {
            parts.push(constraint)
          }
        } else {
          parts.push(rawPart)
        }
      }

      return parts
    }

    function hasNumericValueOf(type: ts.Type, tsNode: ts.Node): boolean {
      const apparentType = checker.getApparentType(type)
      const valueOfSymbol = apparentType.getProperty('valueOf')

      if (!valueOfSymbol) return false

      const valueOfType = checker.getTypeOfSymbolAtLocation(
        valueOfSymbol,
        tsNode,
      )

      for (const signature of valueOfType.getCallSignatures()) {
        if (signature.getReturnType().flags & ts.TypeFlags.NumberLike) {
          return true
        }
      }

      return false
    }

    function reportUnnecessary(
      node: TSESTree.CallExpression,
      arg: TSESTree.Expression,
      name: string,
      type: 'string' | 'number',
    ) {
      context.report({
        node,
        messageId: 'unnecessaryCasting',
        data: { name, type },
        fix(fixer) {
          const argText = context.sourceCode.getText(arg)

          return fixer.replaceText(
            node,
            unnecessaryFixNeedsParens(arg, node) ? `(${argText})` : argText,
          )
        },
      })
    }

    return {
      CallExpression(node) {
        if (node.callee.type !== AST_NODE_TYPES.Identifier) return

        const castFunction = castFunctions.get(node.callee.name)

        if (!castFunction) return

        if (node.arguments.length !== 1) return

        const arg = node.arguments[0]

        if (!arg || arg.type === AST_NODE_TYPES.SpreadElement) return

        const functionName = node.callee.name
        const { expectedType, builtin } = castFunction

        if (arg.type === AST_NODE_TYPES.Literal) {
          if (typeof arg.value === expectedType) {
            reportUnnecessary(node, arg, functionName, expectedType)
            return
          }
        } else if (
          arg.type === AST_NODE_TYPES.TemplateLiteral &&
          expectedType === 'string'
        ) {
          reportUnnecessary(node, arg, functionName, expectedType)
          return
        }

        const argType = checker.getTypeAtLocation(
          services.esTreeNodeToTSNodeMap.get(arg),
        )

        const parts = resolveUnionParts(argType)

        if (!parts) return

        let hasAnyPart = false
        let hasUnknownPart = false

        for (const part of parts) {
          if (part.flags & ts.TypeFlags.Any) {
            hasAnyPart = true
          } else if (part.flags & ts.TypeFlags.Unknown) {
            hasUnknownPart = true
          }
        }

        if (hasAnyPart || hasUnknownPart) {
          if (builtin) {
            if (hasAnyPart && !allowAnyCasting) {
              context.report({
                node,
                messageId: 'anyCasting',
                data: { name: functionName },
              })
            } else if (hasUnknownPart && !allowUnknownCasting) {
              context.report({
                node,
                messageId: 'unknownCasting',
                data: { name: functionName },
              })
            }
          }

          return
        }

        if (builtin && !allowNullishCasting) {
          const hasNullishPart = parts.some(
            (part) => part.flags & nullishTypeFlags,
          )

          if (hasNullishPart) {
            const fallback = expectedType === 'string' ? "''" : '0'

            context.report({
              node,
              messageId:
                expectedType === 'string'
                  ? 'nullishStringCasting'
                  : 'nullishNumberCasting',
              suggest: [
                {
                  messageId: 'addNullishFallback',
                  data: { fallback },
                  fix(fixer) {
                    const argText = context.sourceCode.getText(arg)
                    const wrappedArg = simpleArgTypes.has(arg.type)
                      ? argText
                      : `(${argText})`

                    return fixer.replaceText(
                      arg,
                      `${wrappedArg} ?? ${fallback}`,
                    )
                  },
                },
              ],
            })
            return
          }
        }

        if (builtin && expectedType === 'number' && !allowInvalidCasting) {
          if (parts.some((part) => part.flags & ts.TypeFlags.ESSymbolLike)) {
            context.report({ node, messageId: 'symbolNumberCasting' })
            return
          }

          const tsNode = services.esTreeNodeToTSNodeMap.get(arg)

          const hasNaNPart = parts.some(
            (part) =>
              part.flags & (ts.TypeFlags.Object | ts.TypeFlags.Intersection) &&
              !hasNumericValueOf(part, tsNode),
          )

          if (hasNaNPart) {
            context.report({
              node,
              messageId: 'nanNumberCasting',
              data: { type: checker.typeToString(argType) },
            })
            return
          }
        }

        const targetTypeFlags =
          expectedType === 'string'
            ? ts.TypeFlags.StringLike
            : ts.TypeFlags.NumberLike

        if (parts.every((part) => part.flags & targetTypeFlags)) {
          reportUnnecessary(node, arg, functionName, expectedType)
        }
      },
    }
  },
})
