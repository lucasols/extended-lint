import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import ts from 'typescript'
import { z } from 'zod/v4'
import { getReplacementAt } from '../astUtils'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({})

type Options = z.infer<typeof optionsSchema>

export const noUnnecessaryVoidOnPromise = createExtendedLintRule<
  [Options],
  'unnecessaryVoid' | 'removeVoid'
>({
  name: 'no-unnecessary-void-on-promise',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow void operator on non-Promise function calls to avoid unnecessary void keywords added by no-floating-promises',
    },
    messages: {
      unnecessaryVoid:
        'Unnecessary void operator on non-Promise function call',
      removeVoid: 'Remove void operator',
    },
    hasSuggestions: true,
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context) {
    const parserServices = ESLintUtils.getParserServices(context, true)
    const checker = parserServices.program?.getTypeChecker()

    if (!checker || !parserServices.program) {
      throw new Error('TypeScript services or program not available')
    }

    function isPromiseLike(type: ts.Type, typeChecker: ts.TypeChecker): boolean {
      if (type.flags & ts.TypeFlags.Any || type.flags & ts.TypeFlags.Unknown) {
        return false
      }

      if (type.isUnion()) {
        return type.types.some((part) => isPromiseLike(part, typeChecker))
      }

      if (type.flags & ts.TypeFlags.Instantiable) {
        const constraint = typeChecker.getBaseConstraintOfType(type)
        return (
          constraint !== undefined &&
          constraint !== type &&
          isPromiseLike(constraint, typeChecker)
        )
      }

      const thenSymbol = type.getProperty('then')
      if (!thenSymbol) return false

      const thenType = typeChecker.getTypeOfSymbol(thenSymbol)
      return thenType.getCallSignatures().length > 0
    }

    function isResultDiscarded(node: TSESTree.Node): boolean {
      const parent = node.parent
      if (!parent) return false

      if (parent.type === AST_NODE_TYPES.ExpressionStatement) return true

      if (parent.type === AST_NODE_TYPES.ForStatement) {
        return parent.init === node || parent.update === node
      }

      if (parent.type === AST_NODE_TYPES.SequenceExpression) {
        return parent.expressions.at(-1) !== node || isResultDiscarded(parent)
      }

      return false
    }

    return {
      UnaryExpression(node: TSESTree.UnaryExpression) {
        if (node.operator !== 'void') return

        if (!isResultDiscarded(node)) return

        if (node.argument.type !== AST_NODE_TYPES.CallExpression) return

        const tsNode = parserServices.esTreeNodeToTSNodeMap.get(node.argument)
        const type = checker.getTypeAtLocation(tsNode)

        if (!isPromiseLike(type, checker)) {
          const sourceCode = context.sourceCode
          const argumentText = sourceCode.getText(node.argument)

          if (
            sourceCode.getCommentsInside(node).length >
            sourceCode.getCommentsInside(node.argument).length
          ) {
            context.report({ node, messageId: 'unnecessaryVoid' })
            return
          }

          context.report({
            node,
            messageId: 'unnecessaryVoid',
            suggest: [
              {
                messageId: 'removeVoid',
                fix(fixer) {
                  return fixer.replaceText(
                    node,
                    getReplacementAt(node, argumentText, sourceCode),
                  )
                },
              },
            ],
          })
        }
      },
    }
  },
})
