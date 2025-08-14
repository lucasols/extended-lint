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

const optionsSchema = t.object({
  forceCheckOnFCPropTypesWithName: t.optional(t.array(t.string())),
  alwaysCheckFunctionOptionTypes: t.optional(t.boolean()),
})

type Options = t.Infer<typeof optionsSchema>

const name = 'no-unused-type-props-in-args'

let forceCheckOnFCPropTypesWithNameRegexps: RegExp[] | null = null

const rule = createRule<
  [Options],
  'unusedObjectTypeProperty' | 'missingComponentParam'
>({
  name,
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow unused undestructured object type properties',
    },
    messages: {
      unusedObjectTypeProperty: `Object type property '{{ propertyName }}' is defined but never used`,
      missingComponentParam:
        'Component has declared props but no props are used',
    },
    schema: [optionsSchema as any],
    fixable: 'code',
  },
  defaultOptions: [{ alwaysCheckFunctionOptionTypes: true }],
  create(context, [options]) {
    const { forceCheckOnFCPropTypesWithName, alwaysCheckFunctionOptionTypes = true } = options

    if (
      forceCheckOnFCPropTypesWithName &&
      !forceCheckOnFCPropTypesWithNameRegexps
    ) {
      forceCheckOnFCPropTypesWithNameRegexps =
        forceCheckOnFCPropTypesWithName.map((pattern) => new RegExp(pattern))
    }

    function extendFromTypeReference(
      isFC: boolean,
      scope: TSESTree.Node,
      reference: TSESTree.Identifier,
      declaredProperties: Map<string, TSESTree.Node>,
      ignoreExported = true,
      forceCheckFunctionOption = false,
    ) {
      const typeName = reference.name

      const resolved = context.sourceCode
        .getScope(scope)
        .references.find(
          (ref) => ref.identifier.name === typeName,
        )?.resolved

      const forceCheck =
        isFC &&
        forceCheckOnFCPropTypesWithNameRegexps?.some((regexp) =>
          regexp.test(typeName),
        )

      const shouldForceCheckFunctionOption =
        forceCheckFunctionOption && alwaysCheckFunctionOptionTypes

      if (
        !resolved ||
        (!forceCheck &&
          resolved.references.filter((ref) => ref.isTypeReference)
            .length > 1)
      ) {
        return
      }

      const type = resolved!.defs[0]!.node

      const isExported = type?.parent?.type === AST_NODE_TYPES.ExportNamedDeclaration

      if (
        ignoreExported &&
        isExported &&
        !shouldForceCheckFunctionOption
      ) {
        return
      }

      if (type?.type === AST_NODE_TYPES.TSTypeAliasDeclaration) {
        extendDeclaredTypeParams(
          isFC,
          scope,
          declaredProperties,
          type.typeAnnotation,
          true,
          forceCheckFunctionOption,
        )
        return
      }

      if (type?.type === AST_NODE_TYPES.TSInterfaceDeclaration) {
        extendDeclaredTypeParams(
          isFC,
          scope,
          declaredProperties,
          type.body,
          true,
          forceCheckFunctionOption,
        )
        return
      }
    }

    function extendDeclaredTypeParams(
      isFC: boolean,
      scope: TSESTree.Node,
      declaredProperties: Map<string, TSESTree.Node>,
      typeNode: TSESTree.TypeNode | TSESTree.TSInterfaceBody,
      ignoreReferences: boolean,
      forceCheckFunctionOption = false,
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
          extendDeclaredTypeParams(isFC, scope, declaredProperties, type, true, forceCheckFunctionOption)
        }
        return
      }

      if (!ignoreReferences) {
        if (
          typeNode.type === AST_NODE_TYPES.TSTypeReference &&
          typeNode.typeName.type === AST_NODE_TYPES.Identifier
        ) {
          extendFromTypeReference(
            isFC,
            scope,
            typeNode.typeName,
            declaredProperties,
            true,
            forceCheckFunctionOption,
          )
        }
      }
    }

    function checkParamsOfInferredDeclarations(
      isFC: boolean,
      scope: TSESTree.Node,
      params: TSESTree.Parameter[],
      forceCheckFunctionOption = false,
    ) {
      for (const param of params) {
        if (param.type === AST_NODE_TYPES.ObjectPattern && param.typeAnnotation) {
          const declaredProperties = new Map<string, TSESTree.Node>()

          extendDeclaredTypeParams(
            isFC,
            scope,
            declaredProperties,
            param.typeAnnotation.typeAnnotation,
            false,
            forceCheckFunctionOption,
          )

          if (declaredProperties.size === 0) {
            continue
          }

          checkProperties(param, declaredProperties)
        } else if (
          param.type === AST_NODE_TYPES.AssignmentPattern &&
          param.left.type === AST_NODE_TYPES.ObjectPattern
        ) {
          checkParamsOfInferredDeclarations(isFC, scope, [param.left], forceCheckFunctionOption)
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
            node,
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
                      propertiesText,
                    )
                  }

                  if (lastProperty.type === AST_NODE_TYPES.RestElement) {
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
      VariableDeclaration(node) {
        const declaration = node.declarations[0]

        if (!declaration) {
          return
        }

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
            true,
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
                true,
                node,
                type.typeName,
                declaredProperties,
                false,
              )
            } else {
              extendDeclaredTypeParams(
                true,
                node,
                declaredProperties,
                type,
                true,
              )
            }
          }
        }

        if (declaredProperties.size === 0) return

        if (declaration.init?.type === AST_NODE_TYPES.ArrowFunctionExpression) {
          const params = declaration.init.params[0]

          if (!params) {
            context.report({
              node: declaration.init,
              messageId: 'missingComponentParam',
            })
            return
          }

          if (params.type === AST_NODE_TYPES.ObjectPattern) {
            checkProperties(params, declaredProperties)
          }
        }
      },
      FunctionDeclaration(node) {
        checkParamsOfInferredDeclarations(false, node, node.params, alwaysCheckFunctionOptionTypes)
      },
      ArrowFunctionExpression(node) {
        checkParamsOfInferredDeclarations(false, node, node.params, alwaysCheckFunctionOptionTypes)
      },
    }
  },
})

export const noUnusedObjectTypeProperties = {
  name,
  rule,
}
