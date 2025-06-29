import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-non-camel-case-functions'

const camelCaseRegex = /^(\$?[a-z][a-zA-Z0-9]*)$/

function isJsxElementType(typeAnnotation: TSESTree.TSTypeReference): boolean {
  if (typeAnnotation.typeName.type !== AST_NODE_TYPES.TSQualifiedName) {
    return false
  }

  const { left, right } = typeAnnotation.typeName

  return (
    left.type === AST_NODE_TYPES.Identifier &&
    left.name === 'JSX' &&
    right.name === 'Element'
  )
}

const rule = createRule({
  name,
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Enforce camelCase naming convention for function declarations',
    },
    schema: [],
    messages: {
      nonCamelCaseFunction:
        'Function name "{{functionName}}" should be in camelCase format. If this fn is a React component and can\'t use the `FC` type, add a explicit "JSX.Element" return type to it',
    },
  },
  defaultOptions: [],
  create(context) {
    return {
      FunctionDeclaration(node) {
        if (node.id && !camelCaseRegex.test(node.id.name)) {
          const returnTypeAnnotation = node.returnType?.typeAnnotation

          if (returnTypeAnnotation) {
            const hasJsxReturnType =
              returnTypeAnnotation.type === AST_NODE_TYPES.TSTypeReference
                ? isJsxElementType(returnTypeAnnotation)
                : returnTypeAnnotation.type === AST_NODE_TYPES.TSUnionType &&
                  returnTypeAnnotation.types.some(
                    (type) =>
                      type.type === AST_NODE_TYPES.TSTypeReference &&
                      isJsxElementType(type),
                  )

            if (hasJsxReturnType) return
          }

          context.report({
            node: node.id,
            messageId: 'nonCamelCaseFunction',
            data: {
              functionName: node.id.name,
            },
          })
        }
      },
    }
  },
})

export const noNonCamelCaseFunctions = {
  name,
  rule,
}
