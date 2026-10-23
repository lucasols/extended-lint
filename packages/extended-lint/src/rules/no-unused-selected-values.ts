import type { Reference } from '@typescript-eslint/scope-manager'
import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  selectors: z.array(
    z.object({
      name: z.string(),
      selectorProp: z.string().optional(),
      selectorArgPos: z.number().optional(),
      returnProp: z.string().optional(),
    }),
  ),
})

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-unused-selected-values'

type Options = z.infer<typeof optionsSchema>

export const noUnusedSelectedValues = {
  name,
  rule: createRule<[Options], 'unusedSelectedValue'>({
    name,
    meta: {
      type: 'suggestion',
      docs: {
        description: 'Disallow unused values in selector objects',
      },
      messages: {
        unusedSelectedValue: 'The selected value "{{name}}" is not being used',
      },
      schema: [getJsonSchemaFromZod(optionsSchema)],
    },
    defaultOptions: [{ selectors: [] }],
    create(context, [options]) {
      const { selectors = [] } = options

      const selectorsMap = new Map(
        selectors.map((selector) => [selector.name, selector]),
      )

      return {
        VariableDeclarator(node) {
          let callExpression: TSESTree.CallExpression | null = null

          if (node.init?.type === AST_NODE_TYPES.CallExpression) {
            callExpression = node.init
          }

          if (!callExpression) return

          const calleeName = getCalleeNameFromCallExpression(callExpression)

          if (!calleeName) return

          const selectorCfg = selectorsMap.get(calleeName)

          if (!selectorCfg) return

          const selectorArg = getSelectorFunctionFromCallExpression(
            callExpression,
            selectorCfg.selectorArgPos ?? 0,
            selectorCfg.selectorProp,
          )

          if (!selectorArg) return

          const returnObject = getReturnObjectFromArrowFunction(selectorArg)

          if (!returnObject) return

          const declaredProperties = getPropertiesFromObject(
            returnObject.properties,
            undefined,
          )

          if (!declaredProperties) return

          if (node.id.type === AST_NODE_TYPES.ObjectPattern) {
            const usedProperties = getPropertiesFromObject(
              node.id.properties,
              selectorCfg.returnProp,
            )

            if (!usedProperties) return

            for (const [property, propertyNode] of declaredProperties) {
              if (!usedProperties.has(property)) {
                context.report({
                  node: propertyNode,
                  messageId: 'unusedSelectedValue',
                  data: { name: property },
                })
              }
            }
            return
          }

          if (node.id.type !== AST_NODE_TYPES.Identifier) return

          if (selectorCfg.returnProp) return

          const returnRefs = getIdentifierReferences(node, context.sourceCode)

          if (returnRefs.length === 0) return

          const usedProperties = new Set<string>()

          for (const ref of returnRefs) {
            if (
              ref.identifier.parent.type !== AST_NODE_TYPES.MemberExpression
            ) {
              return
            }

            const propertyAccess = ref.identifier.parent.property

            if (propertyAccess.type !== AST_NODE_TYPES.Identifier) return

            usedProperties.add(propertyAccess.name)
          }

          for (const [property, propertyNode] of declaredProperties) {
            if (!usedProperties.has(property)) {
              context.report({
                node: propertyNode,
                messageId: 'unusedSelectedValue',
                data: { name: property },
              })
            }
          }
        },
      }
    },
  }),
}

function getSelectorFunctionFromCallExpression(
  node: TSESTree.CallExpression,
  selectorArgPos: number,
  selectorProp: string | undefined,
): TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression | null {
  const selectorArgument = node.arguments[selectorArgPos]

  if (!selectorArgument) return null

  if (selectorArgument.type === AST_NODE_TYPES.ArrowFunctionExpression)
    return selectorArgument

  if (selectorArgument.type === AST_NODE_TYPES.FunctionExpression)
    return selectorArgument

  if (selectorProp) {
    if (selectorArgument.type === AST_NODE_TYPES.ObjectExpression) {
      const property = selectorArgument.properties.find(
        (property) =>
          property.type === AST_NODE_TYPES.Property &&
          property.key.type === AST_NODE_TYPES.Identifier &&
          property.key.name === selectorProp,
      )

      if (property?.type !== AST_NODE_TYPES.Property) return null

      const propertyValue = property.value

      if (propertyValue.type === AST_NODE_TYPES.ArrowFunctionExpression)
        return propertyValue

      if (propertyValue.type === AST_NODE_TYPES.FunctionExpression)
        return propertyValue
    }
  }

  return null
}

function getCalleeNameFromCallExpression(
  node: TSESTree.CallExpression,
): string | null {
  if (node.callee.type === AST_NODE_TYPES.Identifier) return node.callee.name

  if (node.callee.type === AST_NODE_TYPES.MemberExpression) {
    if (node.callee.property.type === AST_NODE_TYPES.Identifier) {
      return node.callee.property.name
    }
  }

  return null
}

function getReturnObjectFromArrowFunction(
  node: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression,
): TSESTree.ObjectExpression | null {
  if (node.body.type === AST_NODE_TYPES.ObjectExpression) {
    return node.body
  }

  if (node.body.type === AST_NODE_TYPES.BlockStatement) {
    const returnStatement = node.body.body.filter(
      (statement) => statement.type === AST_NODE_TYPES.ReturnStatement,
    )

    if (returnStatement.length !== 1 || !returnStatement[0]) return null

    if (returnStatement[0].argument?.type !== AST_NODE_TYPES.ObjectExpression)
      return null

    return returnStatement[0].argument
  }

  return null
}

function getPropertiesFromObject(
  objProperties: (TSESTree.ObjectLiteralElement | TSESTree.RestElement)[],
  nestedProp: string | undefined,
): Map<string, TSESTree.Property> | null {
  let returnProperties = objProperties

  if (nestedProp) {
    const nestedProperty = objProperties.find(
      (property) =>
        property.type === AST_NODE_TYPES.Property &&
        property.key.type === AST_NODE_TYPES.Identifier &&
        property.key.name === nestedProp,
    )

    if (nestedProperty?.type !== AST_NODE_TYPES.Property) return null

    if (nestedProperty.value.type !== AST_NODE_TYPES.ObjectPattern) return null

    returnProperties = nestedProperty.value.properties
  }

  const properties = new Map<string, TSESTree.Property>()

  for (const property of returnProperties) {
    if (property.type !== AST_NODE_TYPES.Property) return null

    if (property.key.type !== AST_NODE_TYPES.Identifier) return null

    properties.set(property.key.name, property)
  }

  return properties.size > 0 ? properties : null
}

function getIdentifierReferences(
  node: TSESTree.VariableDeclarator,
  sourceCode: TSESLint.SourceCode,
): Reference[] {
  const variables = sourceCode.getDeclaredVariables(node)

  if (variables.length !== 1) return []

  const variable = variables[0]

  if (!variable) return []

  return variable.references.filter((ref) => ref.identifier !== node.id)
}
