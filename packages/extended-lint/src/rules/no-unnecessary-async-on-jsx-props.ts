import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

const name = 'no-unnecessary-async-on-jsx-props'

function isUnnecessaryAsyncPattern(
  node: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression,
): boolean {
  if (node.body.type !== AST_NODE_TYPES.BlockStatement) {
    return false
  }

  const statements = node.body.body

  if (statements.length !== 1) return false

  const statement = statements[0]

  if (!statement || statement.type !== AST_NODE_TYPES.ExpressionStatement) {
    return false
  }

  return statement.expression.type === AST_NODE_TYPES.AwaitExpression
}

export const noUnnecessaryAsyncOnJsxProps = createExtendedLintRule<
  [],
  'unnecessaryAsyncInJsxProp'
>({
  name,
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description:
        'Disallow unnecessary async/await in JSX props where a single await provides no benefit',
    },
    messages: {
      unnecessaryAsyncInJsxProp:
        'Unnecessary async/await in JSX prop. The single await expression provides no benefit here.',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    function checkAsyncFunction(
      node: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression,
    ) {
      if (isUnnecessaryAsyncPattern(node)) {
        context.report({
          node,
          messageId: 'unnecessaryAsyncInJsxProp',
          fix(fixer) {
            const sourceCode = context.sourceCode
            const fixes = []

            const asyncToken = sourceCode.getFirstToken(
              node,
              (token) => token.value === 'async',
            )
            if (asyncToken) {
              const nextToken = sourceCode.getTokenAfter(asyncToken)
              if (nextToken) {
                fixes.push(
                  fixer.replaceTextRange(
                    [asyncToken.range[0], nextToken.range[0]],
                    '',
                  ),
                )
              } else {
                fixes.push(fixer.remove(asyncToken))
              }
            }

            function removeAwaitTokens(current: TSESTree.Node): void {
              if (current.type === AST_NODE_TYPES.AwaitExpression) {
                const awaitToken = sourceCode.getFirstToken(current)
                if (awaitToken && awaitToken.value === 'await') {
                  const nextToken = sourceCode.getTokenAfter(awaitToken)
                  if (nextToken) {
                    fixes.push(
                      fixer.replaceTextRange(
                        [awaitToken.range[0], nextToken.range[0]],
                        '',
                      ),
                    )
                  } else {
                    fixes.push(fixer.remove(awaitToken))
                  }
                }
              }

              if (current.type === AST_NODE_TYPES.BlockStatement) {
                for (const stmt of current.body) {
                  removeAwaitTokens(stmt)
                }
              } else if (current.type === AST_NODE_TYPES.ExpressionStatement) {
                removeAwaitTokens(current.expression)
              }
            }

            removeAwaitTokens(node.body)

            return fixes
          },
        })
      }
    }

    return {
      // ESQuery selector: async arrow functions and function expressions inside JSX attributes
      'JSXAttribute ArrowFunctionExpression[async=true]': checkAsyncFunction,
      'JSXAttribute FunctionExpression[async=true]': checkAsyncFunction,
    }
  },
})
