import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
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
    },
    messages: {
      unusedObjectTypeProperty: `Object type property '{{ propertyName }}' is defined but never used`,
    },
    schema: [],
    fixable: 'code',
  },
  defaultOptions: [],
  create: function (context) {
    function extendFromTypeReference(
      scope: TSESTree.Node,
      reference: TSESTree.Identifier,
      declaredProperties: Map<string, TSESTree.Node>,
      ignoreExported = true,
    ) {
      const typeName = reference.name

      const resolved = context.sourceCode
        .getScope(scope)
        .references.find(
          (reference) => reference.identifier.name === typeName,
        )?.resolved

      if (
        !resolved ||
        resolved.references.filter((reference) => reference.isTypeReference)
          .length > 1
      ) {
        return
      }

      const type = resolved?.defs[0]?.node

      if (
        ignoreExported &&
        type?.parent?.type === AST_NODE_TYPES.ExportNamedDeclaration
      ) {
        return
      }

      if (type?.type === AST_NODE_TYPES.TSTypeAliasDeclaration) {
        extendDeclaredTypeParams(
          scope,
          declaredProperties,
          type.typeAnnotation,
          true,
        )
        return
      }

      if (type?.type === AST_NODE_TYPES.TSInterfaceDeclaration) {
        extendDeclaredTypeParams(scope, declaredProperties, type.body, true)
        return
      }
    }

    function extendDeclaredTypeParams(
      scope: TSESTree.Node,
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
          extendDeclaredTypeParams(scope, declaredProperties, type, true)
        }
        return
      }

      if (!ignoreReferences) {
        if (
          typeNode.type === AST_NODE_TYPES.TSTypeReference &&
          typeNode.typeName.type === AST_NODE_TYPES.Identifier
        ) {
          extendFromTypeReference(scope, typeNode.typeName, declaredProperties)
        }
      }
    }

    function checkParamsOfInferredDeclarations(
      scope: TSESTree.Node,
      params: TSESTree.Parameter[],
    ) {
      for (const param of params) {
        if (param.type === 'ObjectPattern' && param.typeAnnotation) {
          const declaredProperties = new Map<string, TSESTree.Node>()

          extendDeclaredTypeParams(
            scope,
            declaredProperties,
            param.typeAnnotation.typeAnnotation,
            false,
          )

          if (declaredProperties.size === 0) {
            continue
          }

          checkProperties(param, declaredProperties)
        } else if (
          param.type === AST_NODE_TYPES.AssignmentPattern &&
          param.left.type === AST_NODE_TYPES.ObjectPattern
        ) {
          checkParamsOfInferredDeclarations(scope, [param.left])
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

      const reports: TSESLint.ReportDescriptor<'unusedObjectTypeProperty'>[] =
        []
      const propertiesToAddInFix: string[] = []

      for (const [declaredProperty, node] of declaredProperties) {
        if (!destructuredProperties.includes(declaredProperty)) {
          propertiesToAddInFix.push(declaredProperty)
          reports.push({
            node: node,
            messageId: 'unusedObjectTypeProperty',
            data: {
              propertyName: declaredProperty,
            },
          })
        }
      }

      for (const [i, report] of reports.entries()) {
        context.report({
          ...report,
          fix:
            i === reports.length - 1
              ? (fixer) => {
                  const lastProperty = param.properties.at(-1)

                  const propertiesText = propertiesToAddInFix.join(', ')

                  if (!lastProperty) {
                    return fixer.insertTextBeforeRange(
                      [param.range[0] + 1, param.range[1]],
                      `${propertiesText}`,
                    )
                  }

                  if (lastProperty?.type === AST_NODE_TYPES.RestElement) {
                    return null
                  }

                  return fixer.insertTextAfter(
                    lastProperty,
                    `, ${propertiesText}`,
                  )
                }
              : undefined,
        })
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
          declaration.id.typeAnnotation.typeAnnotation.typeArguments?.params[0]

        if (!fcPropsParam) return

        if (
          fcPropsParam.type === AST_NODE_TYPES.TSTypeReference &&
          fcPropsParam.typeName.type === AST_NODE_TYPES.Identifier
        ) {
          extendFromTypeReference(
            node,
            fcPropsParam.typeName,
            declaredProperties,
            false,
          )
        } else if (fcPropsParam.type === AST_NODE_TYPES.TSTypeLiteral) {
          extendMap(declaredProperties, ...getTypeLiteralMembers(fcPropsParam))
        } else if (fcPropsParam.type === AST_NODE_TYPES.TSIntersectionType) {
          for (const type of fcPropsParam.types) {
            if (
              type.type === AST_NODE_TYPES.TSTypeReference &&
              type.typeName.type === AST_NODE_TYPES.Identifier
            ) {
              extendFromTypeReference(
                node,
                type.typeName,
                declaredProperties,
                false,
              )
            } else {
              extendDeclaredTypeParams(node, declaredProperties, type, true)
            }
          }
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
        checkParamsOfInferredDeclarations(node, node.params)
      },
      ArrowFunctionExpression: function (node) {
        checkParamsOfInferredDeclarations(node, node.params)
      },
    }
  },
})

export const noUnusedObjectTypeProperties = {
  name,
  rule,
}
