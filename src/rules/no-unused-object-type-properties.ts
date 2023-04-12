import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://example.com/rule/${name}`,
)

function getTypeLiteralMembers(
  typeAnnotation: TSESTree.TSTypeLiteral,
): [string, TSESTree.Node][] {
  const members: [string, TSESTree.Node][] = []

  for (const member of typeAnnotation.members) {
    if (
      member.type === AST_NODE_TYPES.TSPropertySignature &&
      member.key.type === AST_NODE_TYPES.Identifier
    ) {
      members.push([member.key.name, member])
    }
  }

  return members
}

function extendMap<K, V>(map: Map<K, V>, ...entries: [K, V][]): Map<K, V> {
  for (const entry of entries) {
    map.set(...entry)
  }

  return map
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
    function extendFromTypeReference(
      reference: TSESTree.Identifier,
      declaredProperties: Map<string, TSESTree.Node>,
    ) {
      const typeName = reference.name

      const resolved = context
        .getScope()
        .references.find(
          (reference) => reference.identifier.name === typeName,
        )?.resolved

      if (!resolved || resolved.references.length > 1) {
        return
      }

      const type = resolved?.defs[0]?.node

      if (
        type?.type === AST_NODE_TYPES.TSTypeAliasDeclaration &&
        type.typeAnnotation.type === AST_NODE_TYPES.TSTypeLiteral
      ) {
        extendMap(
          declaredProperties,
          ...getTypeLiteralMembers(type.typeAnnotation),
        )
      }
    }

    function checkParamsOfInferedDeclarations(params: TSESTree.Parameter[]) {
      for (const param of params) {
        if (param.type === 'ObjectPattern' && param.typeAnnotation) {
          const declaredProperties = new Map<string, TSESTree.Node>()

          if (
            param.typeAnnotation.typeAnnotation.type ===
            AST_NODE_TYPES.TSTypeLiteral
          ) {
            extendMap(
              declaredProperties,
              ...getTypeLiteralMembers(param.typeAnnotation.typeAnnotation),
            )
          } else {
            const typeAnnotation = param.typeAnnotation.typeAnnotation

            if (
              typeAnnotation.type === AST_NODE_TYPES.TSTypeReference &&
              typeAnnotation.typeName.type === AST_NODE_TYPES.Identifier
            ) {
              extendFromTypeReference(
                typeAnnotation.typeName,
                declaredProperties,
              )
            }
          }

          if (declaredProperties.size === 0) {
            continue
          }

          checkProperties(param, declaredProperties)
        }
      }
    }

    function checkProperties(
      param: TSESTree.ObjectPattern,
      declaredProperties: Map<string, TSESTree.Node>,
    ) {
      const destructuredProperties: string[] = []

      for (const property of param.properties) {
        if (
          property.type === AST_NODE_TYPES.Property &&
          property.key.type === AST_NODE_TYPES.Identifier
        ) {
          destructuredProperties.push(property.key.name)
        }
      }

      for (const [declaredProperty, node] of declaredProperties) {
        if (!destructuredProperties.includes(declaredProperty)) {
          context.report({
            node: node,
            messageId: 'unusedObjectTypeProperty',
            data: {
              propertyName: declaredProperty,
            },
          })
        }
      }
    }

    return {
      VariableDeclaration: function (node) {
        const declaration = node.declarations[0]

        if (!declaration) return

        const declaredProperties = new Map<string, TSESTree.Node>()

        const fcPropsParam =
          declaration.id.type === AST_NODE_TYPES.Identifier &&
          declaration.id.typeAnnotation?.typeAnnotation.type ===
            AST_NODE_TYPES.TSTypeReference &&
          declaration.id.typeAnnotation.typeAnnotation.typeName.type ===
            AST_NODE_TYPES.Identifier &&
          declaration.id.typeAnnotation.typeAnnotation.typeName.name === 'FC' &&
          declaration.id.typeAnnotation.typeAnnotation.typeParameters?.params[0]

        if (!fcPropsParam) return

        if (
          fcPropsParam.type === AST_NODE_TYPES.TSTypeReference &&
          fcPropsParam.typeName.type === AST_NODE_TYPES.Identifier
        ) {
          extendFromTypeReference(fcPropsParam.typeName, declaredProperties)
        } else if (fcPropsParam.type === AST_NODE_TYPES.TSTypeLiteral) {
          extendMap(declaredProperties, ...getTypeLiteralMembers(fcPropsParam))
        }

        if (declaredProperties.size === 0) return

        if (declaration.init?.type === AST_NODE_TYPES.ArrowFunctionExpression) {
          const params = declaration.init.params[0]

          if (params && params.type === AST_NODE_TYPES.ObjectPattern) {
            checkProperties(params, declaredProperties)
          }
        }
      },
      FunctionDeclaration: function (node) {
        checkParamsOfInferedDeclarations(node.params)
      },
      ArrowFunctionExpression: function (node) {
        checkParamsOfInferedDeclarations(node.params)
      },
    }
  },
})

export const noUnusedObjectTypeProperties = {
  name: 'no-unused-object-type-properties',
  rule: rule,
}
