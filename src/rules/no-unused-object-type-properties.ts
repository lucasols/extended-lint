import type { Rule } from 'eslint'

import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://example.com/rule/${name}`,
)

function getTypeLiteralMembers(
  typeAnnotation: TSESTree.TSTypeLiteral,
): string[] {
  const members: string[] = []

  for (const member of typeAnnotation.members) {
    if (
      member.type === AST_NODE_TYPES.TSPropertySignature &&
      member.key.type === AST_NODE_TYPES.Identifier
    ) {
      members.push(member.key.name)
    }
  }

  return members
}

const rule = createRule({
  name: 'no-unused-object-type-properties',
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow unused undestructured object type properties',
      recommended: 'error',
    },
    messages: {
      unusedObjectTypeProperty: `Unused object type property '{{ propertyName }}'`,
    },
    schema: [],
  },
  defaultOptions: [],
  create: function (context) {
    return {
      FunctionDeclaration: function (node) {
        for (const param of node.params) {
          if (param.type === 'ObjectPattern' && param.typeAnnotation) {
            const declaredProperties: string[] = []

            if (
              param.typeAnnotation.typeAnnotation.type ===
              AST_NODE_TYPES.TSTypeLiteral
            ) {
              declaredProperties.push(
                ...getTypeLiteralMembers(param.typeAnnotation.typeAnnotation),
              )
            } else {
              const typeAnnotation = param.typeAnnotation.typeAnnotation

              if (
                typeAnnotation.type === AST_NODE_TYPES.TSTypeReference &&
                typeAnnotation.typeName.type === AST_NODE_TYPES.Identifier
              ) {
                const typeName = typeAnnotation.typeName.name

                const type = context
                  .getScope()
                  .references.find(
                    (reference) => reference.identifier.name === typeName,
                  )?.resolved?.defs[0]?.node

                if (
                  type?.type === AST_NODE_TYPES.TSTypeAliasDeclaration &&
                  type.typeAnnotation.type === AST_NODE_TYPES.TSTypeLiteral
                ) {
                  declaredProperties.push(
                    ...getTypeLiteralMembers(type.typeAnnotation),
                  )
                }
              }
            }

            if (declaredProperties.length === 0) {
              continue
            }

            const destructuredProperties: string[] = []

            for (const property of param.properties) {
              if (
                property.type === AST_NODE_TYPES.Property &&
                property.key.type === AST_NODE_TYPES.Identifier
              ) {
                destructuredProperties.push(property.key.name)
              }
            }

            for (const declaredProperty of declaredProperties) {
              if (!destructuredProperties.includes(declaredProperty)) {
                context.report({
                  node: param,
                  messageId: 'unusedObjectTypeProperty',
                  data: {
                    propertyName: declaredProperty,
                  },
                })
              }
            }
          }
        }
      },
    }
  },
})

export const noUnusedObjectTypeProperties = {
  name: 'no-unused-object-type-properties',
  rule: rule,
}
