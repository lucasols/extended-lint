import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import * as z from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  ignoreWithDescription: z.string().optional(),
})

type Options = z.infer<typeof optionsSchema>

export const noUnnecessaryDescribe = createExtendedLintRule<
  [Options],
  'unnecessaryDescribe' | 'removeDescribe'
>({
  name: 'no-unnecessary-describe',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow unnecessary describe blocks that wrap all tests in a file',
    },
    messages: {
      unnecessaryDescribe:
        'Describe blocks that solely wrap all tests in the file are unnecessary, they just add unnecessary nesting to the code. The file itself should be enough to group tests. Remove it.',
      removeDescribe: 'Remove the describe block',
    },
    hasSuggestions: true,
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const sourceCode = context.getSourceCode()
    const program = sourceCode.ast

    function getDescribeDescription(
      node: TSESTree.CallExpression,
    ): string | null {
      const firstArg = node.arguments[0]
      if (
        firstArg &&
        firstArg.type === AST_NODE_TYPES.Literal &&
        typeof firstArg.value === 'string'
      ) {
        return firstArg.value
      }
      return null
    }

    function shouldIgnoreDescribe(description: string | null): boolean {
      if (!description || !options.ignoreWithDescription) {
        return false
      }
      const regex = new RegExp(options.ignoreWithDescription)
      return regex.test(description)
    }

    function hasOnlyWrappedTests(
      topLevelDescribes: TSESTree.CallExpression[],
    ): boolean {
      if (topLevelDescribes.length !== 1) return false

      // Find all top-level test-related statements (not wrapped in describe)
      const topLevelTestStatements = program.body.filter((node) => {
        if (node.type === AST_NODE_TYPES.ExpressionStatement) {
          const expr = node.expression
          if (
            expr.type === AST_NODE_TYPES.CallExpression &&
            expr.callee.type === AST_NODE_TYPES.Identifier
          ) {
            const calleeName = expr.callee.name
            // Include test functions and lifecycle hooks, but exclude describe
            return [
              'test',
              'it',
              'beforeEach',
              'afterEach',
              'beforeAll',
              'afterAll',
            ].includes(calleeName)
          }
        }
        return false
      })

      // If there are no top-level test statements, the single describe is unnecessary
      return topLevelTestStatements.length === 0
    }

    function getDescribeBlockContents(node: TSESTree.CallExpression): string {
      const secondArg = node.arguments[1]
      if (
        secondArg &&
        (secondArg.type === AST_NODE_TYPES.FunctionExpression ||
          secondArg.type === AST_NODE_TYPES.ArrowFunctionExpression) &&
        secondArg.body.type === AST_NODE_TYPES.BlockStatement
      ) {
        const statements = secondArg.body.body
        if (statements.length === 0) return ''

        // Extract each statement and normalize indentation
        const extractedStatements = statements.map((statement) => {
          const statementText = sourceCode.text.slice(
            statement.range[0],
            statement.range[1],
          )
          return statementText
        })

        return extractedStatements.join('\n\n')
      }
      return ''
    }

    return {
      Program() {
        const topLevelDescribes: TSESTree.CallExpression[] = []

        for (const statement of program.body) {
          if (
            statement.type === AST_NODE_TYPES.ExpressionStatement &&
            statement.expression.type === AST_NODE_TYPES.CallExpression &&
            statement.expression.callee.type === AST_NODE_TYPES.Identifier &&
            statement.expression.callee.name === 'describe' &&
            statement.expression.arguments.length === 2 // Only simple describe('text', callback)
          ) {
            topLevelDescribes.push(statement.expression)
          }
        }

        if (!hasOnlyWrappedTests(topLevelDescribes)) return

        const [describeNode] = topLevelDescribes
        if (!describeNode) return

        const description = getDescribeDescription(describeNode)

        if (shouldIgnoreDescribe(description)) return

        const describeStatement = program.body.find(
          (node) =>
            node.type === AST_NODE_TYPES.ExpressionStatement &&
            node.expression === describeNode,
        )

        if (
          !describeStatement ||
          describeStatement.type !== AST_NODE_TYPES.ExpressionStatement
        )
          return

        context.report({
          node: describeNode,
          messageId: 'unnecessaryDescribe',
          suggest: [
            {
              messageId: 'removeDescribe',
              fix(fixer) {
                const blockContents = getDescribeBlockContents(describeNode)
                if (!blockContents) return null

                return fixer.replaceText(describeStatement, blockContents)
              },
            },
          ],
        })
      },
    }
  },
})
