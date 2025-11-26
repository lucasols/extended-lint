import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

type FunctionNode =
  | TSESTree.FunctionExpression
  | TSESTree.ArrowFunctionExpression

function getReturnedExpression(
  fn: FunctionNode,
  sourceCode: { getText: (node: TSESTree.Node) => string },
): string {
  const { body } = fn

  if (body.type !== AST_NODE_TYPES.BlockStatement) {
    return sourceCode.getText(body)
  }

  const statement = body.body[0]

  if (!statement || statement.type !== AST_NODE_TYPES.ReturnStatement) {
    return ''
  }

  if (!statement.argument) return 'undefined'

  return sourceCode.getText(statement.argument)
}

export const noUnnecessaryIife = createExtendedLintRule<[], 'unnecessaryIife'>({
  name: 'no-unnecessary-iife',
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description:
        'Disallow unnecessary immediately invoked function expressions',
    },
    messages: {
      unnecessaryIife:
        'Unnecessary IIFE. Can be simplified to the returned expression.',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    const sourceCode = context.sourceCode

    return {
      CallExpression(node) {
        const { callee, arguments: callArgs } = node

        if (callArgs.length > 0) return

        if (
          callee.type !== AST_NODE_TYPES.FunctionExpression &&
          callee.type !== AST_NODE_TYPES.ArrowFunctionExpression
        ) {
          return
        }

        if (callee.async) return

        if (callee.params.length > 0) return

        const { body } = callee

        if (body.type === AST_NODE_TYPES.BlockStatement) {
          if (body.body.length !== 1) return

          const statement = body.body[0]

          if (!statement || statement.type !== AST_NODE_TYPES.ReturnStatement) {
            return
          }
        }

        const hasComments = sourceCode.getCommentsInside(callee).length > 0

        if (hasComments) return

        const returnedExpression = getReturnedExpression(callee, sourceCode)

        context.report({
          node,
          messageId: 'unnecessaryIife',
          fix: (fixer) => fixer.replaceText(node, returnedExpression),
        })
      },
    }
  },
})
