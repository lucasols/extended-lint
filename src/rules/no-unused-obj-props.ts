import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-unused-obj-props'

export const noUnusedObjProps = {
  name,
  rule: createRule<[], 'unusedObjectProperty'>({
    name,
    meta: {
      type: 'suggestion',
      docs: {
        description:
          'Disallow unused properties in objects that are not explicitly typed',
      },
      messages: {
        unusedObjectProperty:
          'The object property "{{name}}" is not being used',
      },
      schema: [],
    },
    defaultOptions: [],
    create(context) {
      return {
        VariableDeclarator(node) {
          // Skip if it's a destructuring pattern
          if (node.id.type === AST_NODE_TYPES.ObjectPattern) {
            return
          }

          // Only check variable declarations with object expressions
          if (
            !node.init ||
            node.init.type !== AST_NODE_TYPES.ObjectExpression
          ) {
            return
          }

          // Skip if the variable has an explicit type annotation
          if (
            node.id.type === AST_NODE_TYPES.Identifier &&
            node.id.typeAnnotation
          ) {
            return
          }

          // Get the properties from the object
          const objectProperties = new Map<string, TSESTree.Property>()

          for (const property of node.init.properties) {
            if (property.type !== AST_NODE_TYPES.Property) continue
            if (property.key.type !== AST_NODE_TYPES.Identifier) continue

            objectProperties.set(property.key.name, property)
          }

          if (objectProperties.size === 0) return

          // Get references to the variable
          if (node.id.type !== AST_NODE_TYPES.Identifier) return

          const references = getIdentifierReferences(node, context.sourceCode)

          if (references.length === 0) return

          // Check if the object has methods that might use 'this'
          const hasMethodsUsingThis = node.init.properties.some((prop) => {
            if (prop.type !== AST_NODE_TYPES.Property) return false

            // Only function expressions can use 'this', arrow functions cannot
            return prop.value.type === AST_NODE_TYPES.FunctionExpression
          })

          // Track used properties
          const usedProperties = new Set<string>()

          for (const ref of references) {
            // If the reference is not a member expression parent, we need to check other cases
            if (
              ref.identifier.parent.type !== AST_NODE_TYPES.MemberExpression
            ) {
              // If it's a spread element, consider all properties used
              if (ref.identifier.parent.type === AST_NODE_TYPES.SpreadElement) {
                return
              }

              // If it's a JSX spread attribute, consider all properties used
              if (
                ref.identifier.parent.type === AST_NODE_TYPES.JSXSpreadAttribute
              ) {
                return
              }

              // If it's used in a way we can't track (e.g., passed to a function),
              // conservatively assume all properties are used
              if (
                ref.identifier.parent.type === AST_NODE_TYPES.CallExpression ||
                ref.identifier.parent.type ===
                  AST_NODE_TYPES.BinaryExpression ||
                ref.identifier.parent.type ===
                  AST_NODE_TYPES.VariableDeclarator ||
                ref.identifier.parent.type ===
                  AST_NODE_TYPES.JSXExpressionContainer
              ) {
                return
              }

              continue
            }

            const memberExpr = ref.identifier
              .parent as TSESTree.MemberExpression

            // Skip if this is not the object of the member expression
            if (memberExpr.object !== ref.identifier) continue

            // Handle computed property access (obj[prop])
            if (memberExpr.computed) {
              // If the property is a literal string, we can track it
              if (
                memberExpr.property.type === AST_NODE_TYPES.Literal &&
                typeof memberExpr.property.value === 'string'
              ) {
                usedProperties.add(memberExpr.property.value)
              } else {
                // For dynamic computed properties, conservatively assume all properties are used
                return
              }
            } else {
              // Direct property access (obj.prop)
              if (memberExpr.property.type !== AST_NODE_TYPES.Identifier)
                continue

              usedProperties.add(memberExpr.property.name)
            }

            // If this member expression is part of a method call and the object has methods that might use 'this'
            if (
              memberExpr.parent.type === AST_NODE_TYPES.CallExpression &&
              hasMethodsUsingThis
            ) {
              // If we find a method that might use 'this', conservatively assume all properties are used
              return
            }

            // If this member expression is used in JSX, check if it's in a spread attribute
            if (
              memberExpr.parent.type ===
                AST_NODE_TYPES.JSXExpressionContainer &&
              memberExpr.parent.parent.type ===
                AST_NODE_TYPES.JSXSpreadAttribute
            ) {
              return
            }
          }

          // Report unused properties
          for (const [property, propertyNode] of objectProperties) {
            if (!usedProperties.has(property)) {
              context.report({
                node: propertyNode,
                messageId: 'unusedObjectProperty',
                data: { name: property },
              })
            }
          }
        },
      }
    },
  }),
}

function getIdentifierReferences(
  node: TSESTree.VariableDeclarator,
  sourceCode: TSESLint.SourceCode,
): TSESLint.Scope.Reference[] {
  const variables = sourceCode.getDeclaredVariables(node)

  if (variables.length !== 1) return []

  const variable = variables[0]

  if (!variable) return []

  return variable.references.filter((ref) => ref.identifier !== node.id)
}
