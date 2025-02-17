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

const name = 'collapse-simple-objs-in-one-line'

const optionsSchema = t.object({
  maxLineLength: t.optional(t.number()),
  maxProperties: t.optional(t.number()),
})

type Options = t.Infer<typeof optionsSchema>

const rule = createRule<[Options], 'singleLineProp'>({
  name,
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description: 'Format simple objects and types in one line',
    },
    messages: {
      singleLineProp: 'Object/type should be written in a single line',
    },
    schema: [optionsSchema as any],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const sourceCode = context.sourceCode

    const maxProperties = options.maxProperties ?? 2

    function getPropertyText(
      node: TSESTree.ObjectExpression | TSESTree.TSTypeLiteral,
    ): string | false {
      if (node.type === AST_NODE_TYPES.ObjectExpression) {
        if (node.properties.length > maxProperties) return false

        if (node.properties.length === 1) {
          const property = node.properties[0]!

          if (
            property.type === AST_NODE_TYPES.Property &&
            property.value.type === AST_NODE_TYPES.ObjectExpression
          ) {
            return false
          }

          return sourceCode.getText(property)
        }

        const propertyTexts: string[] = []

        for (const property of node.properties) {
          if (property.type === AST_NODE_TYPES.Property) {
            const valueType = property.value.type

            const isSimpleValue =
              valueType === AST_NODE_TYPES.Literal ||
              valueType === AST_NODE_TYPES.Identifier

            if (!isSimpleValue) {
              return false
            }
          }

          propertyTexts.push(sourceCode.getText(property))
        }

        return propertyTexts.join(', ')
      } else {
        if (node.members.length > maxProperties) return false

        if (node.members.length === 1) {
          const property = node.members[0]!

          if (
            property.type === AST_NODE_TYPES.TSPropertySignature &&
            property.typeAnnotation?.typeAnnotation.type ===
              AST_NODE_TYPES.TSTypeLiteral
          ) {
            return false
          }

          return sourceCode.getText(property)
        }

        const propertyTexts: string[] = []
        for (const property of node.members) {
          if (property.type !== AST_NODE_TYPES.TSPropertySignature) {
            return false
          }

          const typeAnn = property.typeAnnotation?.typeAnnotation

          if (!typeAnn) return false

          if (typeAnn.type === AST_NODE_TYPES.TSTypeLiteral) return false

          if (!isSimplePropValueType(typeAnn.type)) {
            return false
          }

          if (
            typeAnn.type === AST_NODE_TYPES.TSTypeReference &&
            typeAnn.typeArguments
          ) {
            if (typeAnn.typeArguments.params.length > 1) return false

            const typeArgument = typeAnn.typeArguments.params[0]!

            if (!isSimplePropValueType(typeArgument.type)) {
              return false
            }
          }

          let text = sourceCode.getText(property).trim()

          if (text.endsWith(';') || text.endsWith(',')) {
            text = text.slice(0, -1)
          }

          propertyTexts.push(text)
        }

        return propertyTexts.join('; ')
      }
    }

    function checkNode(
      node: TSESTree.ObjectExpression | TSESTree.TSTypeLiteral,
    ) {
      if (node.loc.start.line === node.loc.end.line) return

      if (node.parent.type !== AST_NODE_TYPES.JSXExpressionContainer) {
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
          (hasTrailingToken(node, sourceCode) ? 1 : 0) >
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

function isSimplePropValueType(type: TSESTree.AST_NODE_TYPES) {
  if (type === AST_NODE_TYPES.TSLiteralType) return true
  if (type === AST_NODE_TYPES.TSTypeReference) return true
  if (type === AST_NODE_TYPES.TSNumberKeyword) return true
  if (type === AST_NODE_TYPES.TSStringKeyword) return true
  if (type === AST_NODE_TYPES.TSBooleanKeyword) return true
  if (type === AST_NODE_TYPES.TSNullKeyword) return true
  if (type === AST_NODE_TYPES.TSUndefinedKeyword) return true

  return false
}

function hasTrailingToken(
  node: TSESTree.Node,
  sourceCode: TSESLint.SourceCode,
) {
  const tokenAfterNode = sourceCode.getTokenAfter(node)

  return (
    tokenAfterNode?.type === AST_TOKEN_TYPES.Punctuator &&
    (tokenAfterNode.value === ',' ||
      tokenAfterNode.value === ';' ||
      tokenAfterNode.value === '}') &&
    tokenAfterNode.loc.start.line === node.loc.end.line
  )
}

function getTokenIndent(sourceCode: TSESLint.SourceCode, token: TSESTree.Node) {
  return sourceCode.text.slice(
    token.range[0] - token.loc.start.column,
    token.range[0],
  )
}

export const collapseSimpleObjsInOneLine = {
  name,
  rule,
}
