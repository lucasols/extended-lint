import {
  AST_NODE_TYPES,
  AST_TOKEN_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { getJsonSchemaFromZod } from '../createRule'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'collapse-simple-objs-in-one-line'

const optionsSchema = z.object({
  maxLineLength: z.number().optional(),
  maxProperties: z.number().optional(),
  nestedObjMaxLineLength: z.number().optional(),
  nestedObjMaxProperties: z.number().optional(),
  ignoreTypesWithSuffix: z.array(z.string()).optional(),
})

type Options = z.infer<typeof optionsSchema>

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
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const sourceCode = context.sourceCode

    const ignoreTypesWithSuffix = options.ignoreTypesWithSuffix ?? []

    const maxProperties = options.maxProperties ?? 2
    const nestedObjMaxLineLength =
      options.nestedObjMaxLineLength ?? options.maxLineLength
    const nestedObjMaxProperties = options.nestedObjMaxProperties ?? 3

    function getMatchedProperties(
      node: TSESTree.ObjectExpression | TSESTree.TSTypeLiteral,
    ): { text: string; isNested: boolean; propsSize: number } | false {
      if (node.type === AST_NODE_TYPES.ObjectExpression) {
        const isNested = node.parent.type === AST_NODE_TYPES.Property

        const maxPropertiesToUse = isNested
          ? nestedObjMaxProperties
          : maxProperties

        const propsSize = node.properties.length
        if (propsSize > maxPropertiesToUse) return false

        if (propsSize === 1) {
          const property = node.properties[0]
          if (!property) return false

          if (
            property.type === AST_NODE_TYPES.Property &&
            property.value.type === AST_NODE_TYPES.ObjectExpression
          ) {
            return false
          }

          if (
            property.type === AST_NODE_TYPES.Property &&
            property.value.type === AST_NODE_TYPES.ArrayExpression
          ) {
            const hasSimpleValues = property.value.elements.every(
              (element) => element && isSimpleObjectPropValue(element),
            )

            if (!hasSimpleValues) return false
          }

          return { text: sourceCode.getText(property), isNested, propsSize }
        }

        const propertyTexts: string[] = []

        for (const property of node.properties) {
          if (property.type === AST_NODE_TYPES.Property) {
            const isSimpleValue = isSimpleObjectPropValue(property.value)

            if (!isSimpleValue) return false
          }

          propertyTexts.push(sourceCode.getText(property))
        }

        return { text: propertyTexts.join(', '), isNested, propsSize }
      } else {
        const isNested =
          node.parent.parent?.type === AST_NODE_TYPES.TSPropertySignature

        const maxPropertiesToUse = isNested
          ? nestedObjMaxProperties
          : maxProperties

        const propsSize = node.members.length

        if (propsSize > maxPropertiesToUse) return false

        if (
          ignoreTypesWithSuffix.length > 0 &&
          node.parent.type === AST_NODE_TYPES.TSTypeAliasDeclaration
        ) {
          const typeName = node.parent.id.name

          if (
            ignoreTypesWithSuffix.some((suffix) => typeName.endsWith(suffix))
          ) {
            return false
          }
        }

        if (propsSize === 1) {
          const property = node.members[0]
          if (!property) return false

          if (
            property.type === AST_NODE_TYPES.TSPropertySignature &&
            property.typeAnnotation?.typeAnnotation.type ===
              AST_NODE_TYPES.TSTypeLiteral
          ) {
            return false
          }

          return { text: sourceCode.getText(property), isNested, propsSize: 1 }
        }

        if (node.parent.type === AST_NODE_TYPES.TSIntersectionType) {
          return false
        }

        if (
          node.parent.type === AST_NODE_TYPES.TSUnionType &&
          node.parent.types[0] !== node
        ) {
          return false
        }

        const propertyTexts: string[] = []
        for (const property of node.members) {
          if (property.type !== AST_NODE_TYPES.TSPropertySignature) {
            return false
          }

          const typeAnn = property.typeAnnotation?.typeAnnotation

          if (!typeAnn) return false

          if (typeAnn.type === AST_NODE_TYPES.TSTypeLiteral) return false

          if (!isSimpleTypePropValue(typeAnn)) return false

          if (
            typeAnn.type === AST_NODE_TYPES.TSTypeReference &&
            typeAnn.typeArguments
          ) {
            if (typeAnn.typeArguments.params.length > 1) return false

            const typeArgument = typeAnn.typeArguments.params[0]
            if (!typeArgument) return false

            if (!isSimpleTypePropValue(typeArgument)) return false
          }

          let text = sourceCode.getText(property).trim()

          if (text.endsWith(';') || text.endsWith(',')) {
            text = text.slice(0, -1)
          }

          propertyTexts.push(text)
        }

        return { text: propertyTexts.join('; '), isNested, propsSize }
      }
    }

    function checkNode(
      node: TSESTree.ObjectExpression | TSESTree.TSTypeLiteral,
    ) {
      if (node.loc.start.line === node.loc.end.line) return

      if (node.parent.type !== AST_NODE_TYPES.JSXExpressionContainer) {
        let skippedTokens = 0
        let stopCounting = false

        const tokenAfterNode = sourceCode.getTokenAfter(node, {
          filter: (token) => {
            if (token.type !== AST_TOKEN_TYPES.Punctuator) return true

            if (stopCounting) return true

            if (token.value === ',') {
              skippedTokens++
              return false
            }

            if (token.value === ';') {
              skippedTokens++
              stopCounting = true
              return false
            }

            if (token.value === ')') {
              skippedTokens++
              return false
            }

            if (token.value === '}') {
              skippedTokens++
              return false
            }

            return true
          },
        })

        if (skippedTokens > 4) return

        const nextTokenIsTemplate =
          tokenAfterNode?.type === AST_TOKEN_TYPES.Template &&
          tokenAfterNode.value.startsWith('}\n')

        if (
          tokenAfterNode?.loc.start.line === node.loc.end.line &&
          !nextTokenIsTemplate
        ) {
          return
        }
      }

      const matchedObject = getMatchedProperties(node)

      if (!matchedObject) return

      let propertyText = matchedObject.text

      if (propertyText.includes('\n')) return

      if (sourceCode.getCommentsInside(node).length > 0) return

      if (propertyText.endsWith(';')) {
        propertyText = propertyText.slice(0, -1)
      }

      const singleLine = `{ ${propertyText} }`

      const nodeIndent = getTokenIndent(sourceCode, node)

      const maxLineLength =
        matchedObject.isNested && matchedObject.propsSize > 2
          ? nestedObjMaxLineLength
          : options.maxLineLength

      if (
        maxLineLength &&
        singleLine.length +
          nodeIndent.length +
          getExtraCharsAfterNode(node, sourceCode) >
          maxLineLength
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

function isSimpleTypePropValue(node: TSESTree.TypeNode) {
  if (node.type === AST_NODE_TYPES.TSLiteralType) return true
  if (node.type === AST_NODE_TYPES.TSTypeReference) return true
  if (node.type === AST_NODE_TYPES.TSNumberKeyword) return true
  if (node.type === AST_NODE_TYPES.TSStringKeyword) return true
  if (node.type === AST_NODE_TYPES.TSBooleanKeyword) return true
  if (node.type === AST_NODE_TYPES.TSNullKeyword) return true
  if (node.type === AST_NODE_TYPES.TSUndefinedKeyword) return true

  return false
}

function isSimpleObjectPropValue(node: TSESTree.Node, skipArray?: boolean) {
  if (node.type === AST_NODE_TYPES.Literal) return true
  if (node.type === AST_NODE_TYPES.Identifier) return true
  if (node.type === AST_NODE_TYPES.TemplateLiteral) return true
  if (node.type === AST_NODE_TYPES.TaggedTemplateExpression) return true

  if (!skipArray && node.type === AST_NODE_TYPES.ArrayExpression) {
    if (
      node.elements.every(
        (element) => element && isSimpleObjectPropValue(element, true),
      )
    ) {
      return true
    }
  }

  return false
}

function getExtraCharsAfterNode(
  node: TSESTree.Node,
  sourceCode: TSESLint.SourceCode,
): number {
  const closingToken = sourceCode.getLastToken(node)
  if (!closingToken) return 0

  const lineEndIndex = sourceCode.text.indexOf('\n', closingToken.range[1])
  const endOfLine = lineEndIndex !== -1 ? lineEndIndex : sourceCode.text.length

  return endOfLine - closingToken.range[1]
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
