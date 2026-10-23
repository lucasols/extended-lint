import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import ts from 'typescript'
import { z } from 'zod/v4'
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

      const thenSymbol = type.getProperty('then')
      if (!thenSymbol) return false

      const thenType = typeChecker.getTypeOfSymbol(thenSymbol)
      return thenType.getCallSignatures().length > 0
    }

    return {
      UnaryExpression(node: TSESTree.UnaryExpression) {
        if (node.operator !== 'void') return

        if (node.argument.type !== AST_NODE_TYPES.CallExpression) return

        const tsNode = parserServices.esTreeNodeToTSNodeMap.get(node.argument)
        const type = checker.getTypeAtLocation(tsNode)

        if (!isPromiseLike(type, checker)) {
          const sourceCode = context.sourceCode
          const argumentText = sourceCode.getText(node.argument)

          context.report({
            node,
            messageId: 'unnecessaryVoid',
            suggest: [
              {
                messageId: 'removeVoid',
                fix(fixer) {
                  return fixer.replaceText(node, argumentText)
                },
              },
            ],
          })
        }
      },
    }
  },
})
