import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const matchConfigSchema = z.object({
  fn: z.string().optional(),
  selector: z.string().optional(),
  commentPrefix: z.string(),
  message: z.string().optional(),
})

const optionsSchema = z.object({
  matches: z.array(matchConfigSchema),
})

type Options = z.infer<typeof optionsSchema>

export const requireUsageExplanation = createExtendedLintRule<
  [Options],
  'default'
>({
  name: 'require-usage-explanation',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Require explanatory comments with a specific prefix above matched code patterns',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
    messages: {
      default: '{{message}}',
    },
  },
  defaultOptions: [{ matches: [] }],
  create(context, [options]) {
    const sourceCode = context.sourceCode
    const result: Record<string, (node: TSESTree.Node) => void> = {}
    const callExpressionHandlers: Array<
      (node: TSESTree.CallExpression) => void
    > = []

    function findStatementLevelAncestor(
      node: TSESTree.Node,
    ): TSESTree.Node | undefined {
      let current: TSESTree.Node = node

      while (current.parent) {
        const parentType = current.parent.type

        if (
          parentType === AST_NODE_TYPES.Program ||
          parentType === AST_NODE_TYPES.BlockStatement ||
          parentType === AST_NODE_TYPES.SwitchCase
        ) {
          return current
        }

        current = current.parent
      }

      return undefined
    }

    function hasValidComment(
      comments: TSESTree.Comment[],
      prefix: string,
    ): boolean {
      for (const comment of comments) {
        const trimmedValue = comment.value.trim()

        if (trimmedValue.startsWith(prefix)) {
          const afterPrefix = trimmedValue.slice(prefix.length).trim()
          if (afterPrefix.length > 0) return true
        }
      }

      return false
    }

    function getCalleeName(
      node: TSESTree.CallExpression,
    ): string | undefined {
      const { callee } = node

      if (callee.type === AST_NODE_TYPES.Identifier) {
        return callee.name
      }

      if (
        callee.type === AST_NODE_TYPES.MemberExpression &&
        callee.property.type === AST_NODE_TYPES.Identifier
      ) {
        return callee.property.name
      }

      return undefined
    }

    function buildMessage(
      prefix: string,
      label: string,
      customMessage: string | undefined,
    ): string {
      const base = `Add a comment with prefix "${prefix}" to explain why "${label}" is being used.`

      if (customMessage) return `${base} ${customMessage}`

      return base
    }

    function hasValidCommentAbove(
      node: TSESTree.Node,
      prefix: string,
    ): boolean {
      const statementNode = findStatementLevelAncestor(node)
      if (!statementNode) return false

      let current: TSESTree.Node = node

      while (current !== statementNode && current.parent) {
        const comments = sourceCode.getCommentsBefore(current)
        if (hasValidComment(comments, prefix)) return true

        current = current.parent
      }

      const statementComments = sourceCode.getCommentsBefore(statementNode)
      if (hasValidComment(statementComments, prefix)) return true

      return false
    }

    function checkNodeForComment(
      node: TSESTree.Node,
      config: {
        commentPrefix: string
        label: string
        message: string | undefined
      },
    ): void {
      if (hasValidCommentAbove(node, config.commentPrefix)) return

      context.report({
        node,
        messageId: 'default',
        data: {
          message: buildMessage(
            config.commentPrefix,
            config.label,
            config.message,
          ),
        },
      })
    }

    function addSelector(
      selector: string,
      handler: (node: TSESTree.Node) => void,
    ) {
      const existing = result[selector]
      if (existing) {
        result[selector] = (node) => {
          existing(node)
          handler(node)
        }
      } else {
        result[selector] = handler
      }
    }

    for (const matchConfig of options.matches) {
      if (matchConfig.fn) {
        const fnName = matchConfig.fn
        const config = {
          commentPrefix: matchConfig.commentPrefix,
          label: fnName,
          message: matchConfig.message,
        }

        callExpressionHandlers.push((node) => {
          const calleeName = getCalleeName(node)
          if (calleeName !== fnName) return
          checkNodeForComment(node, config)
        })
      }

      if (matchConfig.selector) {
        const config = {
          commentPrefix: matchConfig.commentPrefix,
          label: matchConfig.selector,
          message: matchConfig.message,
        }

        addSelector(matchConfig.selector, (node) => {
          checkNodeForComment(node, config)
        })
      }
    }

    if (callExpressionHandlers.length > 0) {
      addSelector('CallExpression', (node) => {
        if (node.type !== AST_NODE_TYPES.CallExpression) return

        for (const handler of callExpressionHandlers) {
          handler(node)
        }
      })
    }

    return result
  },
})
