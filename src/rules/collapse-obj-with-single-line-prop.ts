import {
  AST_NODE_TYPES,
  AST_TOKEN_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import * as t from 'tschema'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'collapse-obj-with-single-line-prop'

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

    function getPropertyText(
      node: TSESTree.ObjectExpression | TSESTree.TSTypeLiteral,
    ): string | false {
      let property

      if (node.type === AST_NODE_TYPES.ObjectExpression) {
        if (node.properties.length !== 1) return false

        property = node.properties[0]!

        if (
          property.type === AST_NODE_TYPES.Property &&
          property.value.type === AST_NODE_TYPES.ObjectExpression
        ) {
          return false
        }
      } else {
        if (node.members.length !== 1) return false

        property = node.members[0]!

        if (
          property.type === AST_NODE_TYPES.TSPropertySignature &&
          property.typeAnnotation?.typeAnnotation.type ===
            AST_NODE_TYPES.TSTypeLiteral
        ) {
          return false
        }
      }

      return sourceCode.getText(property)
    }

    function checkNode(
      node: TSESTree.ObjectExpression | TSESTree.TSTypeLiteral,
    ) {
      if (node.loc.start.line === node.loc.end.line) return

      const tokenAfterNode = sourceCode.getTokenAfter(node, {
        filter: (token) => {
          const isTrailingComma =
            token.type === AST_TOKEN_TYPES.Punctuator && token.value === ','

          const isTrailingSemicolon =
            token.type === AST_TOKEN_TYPES.Punctuator && token.value === ';'

          return !isTrailingComma && !isTrailingSemicolon
        },
      })

      if (tokenAfterNode?.loc.start.line === node.loc.end.line) {
        return
      }

      let propertyText = getPropertyText(node)

      if (!propertyText) return

      if (propertyText.includes('\n')) return

      if (sourceCode.getCommentsInside(node).length > 0) return

      if (propertyText.endsWith(';')) {
        propertyText = propertyText.slice(0, -1)
      }

      const singleLine = `{ ${propertyText} }`

      const nodeIndent = getTokenIndent(sourceCode, node)

      if (
        options.maxLineLength &&
        singleLine.length +
          nodeIndent.length +
          (hasTrailingCommaOrSemicolon(node, sourceCode) ? 1 : 0) >
          options.maxLineLength
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
      TSTypeLiteral: checkNode,
      ObjectExpression: checkNode,
    }
  },
})

function hasTrailingCommaOrSemicolon(
  node: TSESTree.Node,
  sourceCode: TSESLint.SourceCode,
) {
  const tokenAfterNode = sourceCode.getTokenAfter(node)

  return (
    tokenAfterNode?.type === AST_TOKEN_TYPES.Punctuator &&
    (tokenAfterNode.value === ',' || tokenAfterNode.value === ';') &&
    tokenAfterNode.loc.start.line === node.loc.end.line
  )
}

function getTokenIndent(sourceCode: TSESLint.SourceCode, token: TSESTree.Node) {
  return sourceCode.text.slice(
    token.range[0] - token.loc.start.column,
    token.range[0],
  )
}

export const collapseObjWithSingleLineProp = {
  name,
  rule,
}
