import {
  AST_NODE_TYPES,
  AST_TOKEN_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import * as ts from 'typescript'
import { z } from 'zod/v4'
import { getReplacementAt } from '../astUtils'
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

function isNumericLiteralMemberObject(
  arg: TSESTree.Expression,
  node: TSESTree.CallExpression,
): boolean {
  return (
    arg.type === AST_NODE_TYPES.Literal &&
    typeof arg.value === 'number' &&
    node.parent.type === AST_NODE_TYPES.MemberExpression &&
    !node.parent.computed &&
    node.parent.object === node
  )
}

const forInitializerBoundaries = new Set<AST_NODE_TYPES>([
  AST_NODE_TYPES.ArrowFunctionExpression,
  AST_NODE_TYPES.FunctionExpression,
  AST_NODE_TYPES.ClassBody,
])

function isInForInitializer(node: TSESTree.Node): boolean {
  let current = node

  for (;;) {
    const parent = current.parent
    if (!parent) return false

    if (parent.type === AST_NODE_TYPES.ForStatement) {
      return parent.init === current
    }

    if (
      forInitializerBoundaries.has(parent.type) ||
      parent.type.endsWith('Statement')
    ) {
      return false
    }

    current = parent
  }
}

function containsInOperator(
  arg: TSESTree.Expression,
  sourceCode: TSESLint.SourceCode,
): boolean {
  return sourceCode
    .getTokens(arg)
    .some((token) => token.type === AST_TOKEN_TYPES.Keyword && token.value === 'in')
}

function unnecessaryFixNeedsParens(
  arg: TSESTree.Expression,
  node: TSESTree.CallExpression,
  sourceCode: TSESLint.SourceCode,
): boolean {
  if (isNumericLiteralMemberObject(arg, node)) return true

  if (simpleArgTypes.has(arg.type)) return false

  if (arg.type === AST_NODE_TYPES.SequenceExpression) return true

  if (containsInOperator(arg, sourceCode) && isInForInitializer(node)) {
    return true
  }

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

const primitiveTypeFlags =
  ts.TypeFlags.StringLike |
  ts.TypeFlags.NumberLike |
  ts.TypeFlags.BigIntLike |
  ts.TypeFlags.BooleanLike |
  ts.TypeFlags.EnumLike

const numberishTypeFlags =
  primitiveTypeFlags |
  ts.TypeFlags.Null |
  ts.TypeFlags.Any |
  ts.TypeFlags.Unknown

type PrimitiveHook = {
  mayReturnNumber: boolean
  mayBeAbsent: boolean
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

    function isDefaultLibDeclaration(declaration: ts.Declaration): boolean {
      return services.program.isSourceFileDefaultLibrary(
        declaration.getSourceFile(),
      )
    }

    function isGlobalObjectType(type: ts.Type): boolean {
      const symbol = type.getSymbol()

      return (
        symbol?.getName() === 'Object' &&
        (symbol.declarations?.some(isDefaultLibDeclaration) ?? false)
      )
    }

    function isEmptyObjectType(type: ts.Type): boolean {
      return (
        type.getProperties().length === 0 &&
        checker.getIndexInfosOfType(type).length === 0 &&
        type.getCallSignatures().length === 0 &&
        type.getConstructSignatures().length === 0
      )
    }

    function mayBeNumberish(type: ts.Type): boolean {
      if (type.isUnion()) return type.types.some(mayBeNumberish)

      if (type.flags & numberishTypeFlags) return true

      if (type.isIntersection()) {
        return type.types.some((part) => (part.flags & primitiveTypeFlags) !== 0)
      }

      if (type.flags & ts.TypeFlags.Instantiable) {
        const constraint = checker.getBaseConstraintOfType(type)

        return !constraint || (constraint !== type && mayBeNumberish(constraint))
      }

      return false
    }

    function getPrimitiveHook(
      method: ts.Symbol,
      tsNode: ts.Node,
    ): PrimitiveHook | undefined {
      const methodType = checker.getTypeOfSymbolAtLocation(method, tsNode)
      const parts = methodType.isUnion() ? methodType.types : [methodType]
      const presentParts = parts.filter(
        (part) => !(part.flags & nullishTypeFlags),
      )

      if (presentParts.length === 0) return undefined

      return {
        mayBeAbsent:
          (method.flags & ts.SymbolFlags.Optional) !== 0 ||
          presentParts.length < parts.length,
        mayReturnNumber: presentParts.some(
          (part) =>
            (part.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown)) !== 0 ||
            part
              .getCallSignatures()
              .some((signature) => mayBeNumberish(signature.getReturnType())),
        ),
      }
    }

    function isWellKnownToPrimitive(property: ts.Symbol): boolean {
      if (!property.getName().startsWith('__@toPrimitive@')) return false

      return (
        property.declarations?.some((declaration) => {
          const name = ts.getNameOfDeclaration(declaration)

          if (!name || !ts.isComputedPropertyName(name)) return false

          const symbol = checker.getSymbolAtLocation(name.expression)

          return symbol?.declarations?.some(isDefaultLibDeclaration) ?? false
        }) ?? false
      )
    }

    function mayConvertToNumber(type: ts.Type, tsNode: ts.Node): boolean {
      if (
        type.isIntersection() &&
        type.types.some((part) => part.flags & primitiveTypeFlags)
      ) {
        return true
      }

      if (isGlobalObjectType(type) || isEmptyObjectType(type)) return true

      const apparentType = checker.getApparentType(type)
      const toPrimitive = apparentType
        .getProperties()
        .find(isWellKnownToPrimitive)
      const toPrimitiveHook =
        toPrimitive && getPrimitiveHook(toPrimitive, tsNode)

      if (toPrimitiveHook) {
        if (toPrimitiveHook.mayReturnNumber) return true
        if (!toPrimitiveHook.mayBeAbsent) return false
      }

      const valueOf = apparentType.getProperty('valueOf')

      if (valueOf && getPrimitiveHook(valueOf, tsNode)?.mayReturnNumber) {
        return true
      }

      const toString = apparentType.getProperty('toString')

      if (
        !toString?.declarations?.some(
          (declaration) => !isDefaultLibDeclaration(declaration),
        )
      ) {
        return false
      }

      return getPrimitiveHook(toString, tsNode)?.mayReturnNumber ?? false
    }

    function hasCommentsOutsideArg(
      node: TSESTree.CallExpression,
      arg: TSESTree.Expression,
    ): boolean {
      return (
        context.sourceCode.getCommentsInside(node).length >
        context.sourceCode.getCommentsInside(arg).length
      )
    }

    function isGlobalBuiltin(callee: TSESTree.Identifier): boolean {
      const symbol = checker.getSymbolAtLocation(
        services.esTreeNodeToTSNodeMap.get(callee),
      )

      return symbol?.declarations?.some(isDefaultLibDeclaration) ?? false
    }

    function isPossiblyNullish(callee: TSESTree.Identifier): boolean {
      const calleeType = checker.getTypeAtLocation(
        services.esTreeNodeToTSNodeMap.get(callee),
      )
      const parts = calleeType.isUnion() ? calleeType.types : [calleeType]

      return parts.some(
        (part) =>
          part.flags &
          (nullishTypeFlags | ts.TypeFlags.Any | ts.TypeFlags.Unknown),
      )
    }

    function keepsNarrowerType(type: ts.Type): boolean {
      const parts = type.isUnion() ? type.types : [type]

      return parts.some(
        (part) =>
          part.flags & (ts.TypeFlags.EnumLike | ts.TypeFlags.TypeParameter),
      )
    }

    function reportUnnecessary(
      node: TSESTree.CallExpression,
      arg: TSESTree.Expression,
      name: string,
      type: 'string' | 'number',
      argType?: ts.Type,
    ) {
      if (
        hasCommentsOutsideArg(node, arg) ||
        (argType && keepsNarrowerType(argType))
      ) {
        context.report({
          node,
          messageId: 'unnecessaryCasting',
          data: { name, type },
        })
        return
      }

      context.report({
        node,
        messageId: 'unnecessaryCasting',
        data: { name, type },
        fix(fixer) {
          const argText = context.sourceCode.getText(arg)

          return fixer.replaceText(
            node,
            getReplacementAt(
              node,
              unnecessaryFixNeedsParens(arg, node, context.sourceCode)
                ? `(${argText})`
                : argText,
              context.sourceCode,
            ),
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

        if (builtin && !isGlobalBuiltin(node.callee)) return

        if (node.optional && isPossiblyNullish(node.callee)) return

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
              !mayConvertToNumber(part, tsNode),
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
          reportUnnecessary(node, arg, functionName, expectedType, argType)
        }
      },
    }
  },
})
