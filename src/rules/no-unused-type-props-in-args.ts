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

const name = 'no-unused-type-props-in-args'

const rule = createRule({
  name,
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow unused undestructured object type properties',
      recommended: 'error',
    },
    messages: {
      unusedObjectTypeProperty: `Object type property '{{ propertyName }}' is defined but never used`,
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

      if (type?.parent?.type === AST_NODE_TYPES.ExportNamedDeclaration) return

      if (type?.type === AST_NODE_TYPES.TSTypeAliasDeclaration) {
        extendDeclaredTypeParams(declaredProperties, type.typeAnnotation, true)
        return
      }

      if (type?.type === AST_NODE_TYPES.TSInterfaceDeclaration) {
        extendDeclaredTypeParams(declaredProperties, type.body, true)
        return
      }
    }

    function extendDeclaredTypeParams(
      declaredProperties: Map<string, TSESTree.Node>,
      typeNode: TSESTree.TypeNode | TSESTree.TSInterfaceBody,
      ignoreReferences: boolean,
    ) {
      if (typeNode.type === AST_NODE_TYPES.TSInterfaceBody) {
        for (const member of typeNode.body) {
          if (
            member.type === AST_NODE_TYPES.TSPropertySignature &&
            member.key.type === AST_NODE_TYPES.Identifier
          ) {
            declaredProperties.set(member.key.name, member)
          }
        }

        return
      }

      if (typeNode.type === AST_NODE_TYPES.TSTypeLiteral) {
        extendMap(declaredProperties, ...getTypeLiteralMembers(typeNode))
        return
      }

      if (typeNode.type === AST_NODE_TYPES.TSIntersectionType) {
        for (const type of typeNode.types) {
          extendDeclaredTypeParams(declaredProperties, type, true)
        }
        return
      }

      if (!ignoreReferences) {
        if (
          typeNode.type === AST_NODE_TYPES.TSTypeReference &&
          typeNode.typeName.type === AST_NODE_TYPES.Identifier
        ) {
          extendFromTypeReference(typeNode.typeName, declaredProperties)
        }
      }
    }

    function checkParamsOfInferedDeclarations(params: TSESTree.Parameter[]) {
      for (const param of params) {
        if (param.type === 'ObjectPattern' && param.typeAnnotation) {
          const declaredProperties = new Map<string, TSESTree.Node>()

          extendDeclaredTypeParams(
            declaredProperties,
            param.typeAnnotation.typeAnnotation,
            false,
          )

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

      const lastIsRest =
        param.properties.at(-1)?.type === AST_NODE_TYPES.RestElement

      if (lastIsRest) {
        return
      }

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
  name,
  rule,
}
