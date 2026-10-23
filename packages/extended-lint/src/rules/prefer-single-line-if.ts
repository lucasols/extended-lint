import {
  AST_NODE_TYPES,
  AST_TOKEN_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import z from 'zod/v4'
import { getJsonSchemaFromZod } from '../createRule'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'prefer-single-line-if'

const optionsSchema = z.object({
  maxLineLength: z.number().optional(),
  maxNonSimpleConditionLength: z.number().optional(),
})

type Options = z.infer<typeof optionsSchema>

const rule = createRule<[Options], 'noSingleLineCurly'>({
  name,
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description: 'Enforce single-line if in simple if statements',
    },
    messages: {
      noSingleLineCurly:
        'If return statement with single statement body should be written in a single line',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const sourceCode = context.sourceCode

    return {
      IfStatement(node) {
        if (node.consequent.type !== AST_NODE_TYPES.BlockStatement) return

        // Skip if the condition is a single line
        if (node.loc.start.line === node.loc.end.line) return

        // Skip if there are comments inside the block
        const comments = sourceCode.getCommentsInside(node.consequent)

        if (comments.length > 0) return

        if (node.alternate) return

        // Only transform if there's exactly one statement
        if (node.consequent.body.length !== 1) return

        const statement = node.consequent.body[0]
        if (!statement) return

        if (statement.type === AST_NODE_TYPES.ReturnStatement) {
          const statementArgCanBeInlined = isValidReturnStatement(statement)

          if (!statementArgCanBeInlined) return
        } else {
          const isValidStatement =
            statement.type === AST_NODE_TYPES.ContinueStatement ||
            statement.type === AST_NODE_TYPES.BreakStatement

          if (!isValidStatement) return
        }

        if (
          node.test.type === AST_NODE_TYPES.LogicalExpression ||
          node.test.type === AST_NODE_TYPES.ConditionalExpression
        ) {
          return
        }

        let ifCondition: string | undefined

        if (options.maxNonSimpleConditionLength) {
          let isComplexCondition = isComplexNodeCondition(node.test)

          if (!isComplexCondition) {
            if (
              node.test.type === AST_NODE_TYPES.UnaryExpression &&
              node.test.operator === '!'
            ) {
              const innerCondition = node.test.argument

              if (isComplexNodeCondition(innerCondition)) {
                isComplexCondition = true
              }
            }
          }

          if (isComplexCondition) {
            ifCondition = sourceCode.getText(node.test)

            if (ifCondition.length > options.maxNonSimpleConditionLength) {
              return
            }
          }
        }

        if (!ifCondition) {
          ifCondition = sourceCode.getText(node.test)
        }

        const statementText = sourceCode.getText(statement)

        if (ifCondition.includes('\n')) return

        const nextToken = sourceCode.getTokenAfter(node)

        if (
          nextToken &&
          nextToken.type === AST_TOKEN_TYPES.Punctuator &&
          nextToken.value === '}'
        ) {
          const nextToken2 = sourceCode.getTokenAfter(nextToken)

          if (
            nextToken2 &&
            nextToken2.type === AST_TOKEN_TYPES.Keyword &&
            (nextToken2.value === 'else' || nextToken2.value === 'catch')
          ) {
            return
          }
        }

        const ifIndent = getTokenIndent(sourceCode, node)

        // Calculate the length of the single-line version
        const singleLine = `if (${ifCondition}) ${statementText}`

        // Skip if the line would be too long
        if (
          options.maxLineLength &&
          singleLine.length + ifIndent.length > options.maxLineLength
        ) {
          return
        }

        context.report({
          node,
          messageId: 'noSingleLineCurly',
          fix: (fixer) => {
            return fixer.replaceText(node, singleLine)
          },
        })
      },
    }
  },
})

function isComplexNodeCondition(node: TSESTree.Node) {
  return (
    node.type === AST_NODE_TYPES.CallExpression ||
    node.type === AST_NODE_TYPES.BinaryExpression ||
    (node.type === AST_NODE_TYPES.MemberExpression &&
      isComplexMemberExpression(node))
  )
}

function isComplexMemberExpression(node: TSESTree.MemberExpression) {
  if (node.object.type === AST_NODE_TYPES.MemberExpression) {
    return isComplexMemberExpression(node.object)
  }
  return node.object.type !== AST_NODE_TYPES.Identifier
}

function getTokenIndent(sourceCode: TSESLint.SourceCode, token: TSESTree.Node) {
  return sourceCode.text.slice(
    token.range[0] - token.loc.start.column,
    token.range[0],
  )
}

function isValidReturnStatement(
  statement: TSESTree.ReturnStatement | TSESTree.UnaryExpression,
) {
  if (!statement.argument) return true

  const argument = statement.argument

  if (
    argument.type === AST_NODE_TYPES.ArrayExpression &&
    argument.elements.length === 0
  ) {
    return true
  }

  if (
    argument.type === AST_NODE_TYPES.ObjectExpression &&
    argument.properties.length === 0
  ) {
    return true
  }

  if (
    argument.type === AST_NODE_TYPES.Literal ||
    argument.type === AST_NODE_TYPES.Identifier ||
    argument.type === AST_NODE_TYPES.TemplateLiteral ||
    argument.type === AST_NODE_TYPES.TaggedTemplateExpression
  ) {
    return true
  }

  if (argument.type === AST_NODE_TYPES.CallExpression) {
    return argument.arguments.length === 0
  }

  if (argument.type === AST_NODE_TYPES.UnaryExpression) {
    return isValidReturnStatement(argument)
  }

  return false
}

export const preferSingleLineIf = {
  name,
  rule,
}
