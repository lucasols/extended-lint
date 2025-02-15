import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import * as t from 'tschema'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'wrap-single-line-prop'

const optionsSchema = t.object({
  maxLineLength: t.optional(t.number()),
})

type Options = t.Infer<typeof optionsSchema>

const rule = createRule<[Options], 'singleLineProp'>({
  name,
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description: 'Format single prop objects and types in one line',
    },
    messages: {
      singleLineProp:
        'Single prop object/type should be written in a single line',
    },
    schema: [optionsSchema as any],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const sourceCode = context.sourceCode

    function checkNode(node: TSESTree.Node) {
      if (
        node.type !== AST_NODE_TYPES.TSInterfaceBody &&
        node.type !== AST_NODE_TYPES.TSTypeLiteral &&
        node.type !== AST_NODE_TYPES.ObjectExpression
      ) {
        return
      }

      let properties: TSESTree.Node[]

      if (node.type === AST_NODE_TYPES.ObjectExpression) {
        properties = node.properties
      } else if (node.type === AST_NODE_TYPES.TSInterfaceBody) {
        properties = node.body
      } else {
        properties = node.members
      }

      if (properties.length === 0) return
      if (properties.length !== 1) return

      const property = properties[0]!

      if (sourceCode.getCommentsInside(node).length > 0) return

      const openBrace = sourceCode.getFirstToken(node)!
      const closeBrace = sourceCode.getLastToken(node)!
      const propertyText = sourceCode.getText(property)

      if (propertyText.includes('\n')) return

      const singleLine = `${openBrace.value} ${propertyText} ${closeBrace.value}`

      const nodeIndent = getTokenIndent(sourceCode, node)

      if (
        options.maxLineLength &&
        singleLine.length + nodeIndent.length > options.maxLineLength
      ) {
        return
      }

      context.report({
        node,
        messageId: 'singleLineProp',
        fix: (fixer) => {
          return fixer.replaceText(node, singleLine)
        },
      })
    }

    return {
      TSInterfaceBody: checkNode,
      TSTypeLiteral: checkNode,
      ObjectExpression: checkNode,
    }
  },
})

function getTokenIndent(sourceCode: TSESLint.SourceCode, token: TSESTree.Node) {
  return sourceCode.text.slice(
    token.range[0] - token.loc.start.column,
    token.range[0],
  )
}

export const wrapSingleLineProp = {
  name,
  rule,
}
