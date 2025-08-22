import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  customMessage: z.string().optional(),
})

type Options = z.infer<typeof optionsSchema>

export const noStaticStyleProp = createExtendedLintRule<
  [Options],
  'noStaticStyleProp'
>({
  name: 'no-static-style-prop',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Prevent using static style props in JSX, only dynamic values should be allowed',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
    messages: {
      noStaticStyleProp:
        'Static style props are not allowed use css instead.{{customMessage}}',
    },
  },
  defaultOptions: [{}],
  create(context, [options]) {
    function isStaticValue(node: TSESTree.Node): boolean {
      switch (node.type) {
        case AST_NODE_TYPES.Literal:
          return true
        case AST_NODE_TYPES.ObjectExpression:
          return node.properties.every((prop) => {
            if (prop.type === AST_NODE_TYPES.Property) {
              if (prop.computed) return false
              if (
                prop.value.type === AST_NODE_TYPES.Identifier &&
                prop.key.type === AST_NODE_TYPES.Identifier &&
                prop.key.name === prop.value.name
              ) {
                return false
              }
              return isStaticValue(prop.value)
            }
            return false
          })
        case AST_NODE_TYPES.ArrayExpression:
          return node.elements.every((element) => {
            if (!element) return true
            if (element.type === AST_NODE_TYPES.SpreadElement) {
              return false
            }
            return isStaticValue(element)
          })
        case AST_NODE_TYPES.TemplateLiteral:
          return node.expressions.length === 0
        case AST_NODE_TYPES.ConditionalExpression:
        case AST_NODE_TYPES.LogicalExpression:
        case AST_NODE_TYPES.BinaryExpression:
        case AST_NODE_TYPES.UnaryExpression:
        case AST_NODE_TYPES.CallExpression:
        case AST_NODE_TYPES.MemberExpression:
        case AST_NODE_TYPES.Identifier:
        case AST_NODE_TYPES.ArrowFunctionExpression:
        case AST_NODE_TYPES.FunctionExpression:
          return false
        default:
          return false
      }
    }

    return {
      JSXAttribute(node) {
        if (
          node.name.type === AST_NODE_TYPES.JSXIdentifier &&
          node.name.name === 'style' &&
          node.value
        ) {
          let valueToCheck: TSESTree.Node | null = null

          if (node.value.type === AST_NODE_TYPES.JSXExpressionContainer) {
            if (
              node.value.expression.type !== AST_NODE_TYPES.JSXEmptyExpression
            ) {
              valueToCheck = node.value.expression
            }
          } else if (node.value.type === AST_NODE_TYPES.Literal) {
            valueToCheck = node.value
          }

          if (valueToCheck && isStaticValue(valueToCheck)) {
            context.report({
              node,
              messageId: 'noStaticStyleProp',
              data: {
                customMessage: options.customMessage
                  ? ` ${options.customMessage}`
                  : '',
              },
            })
          }
        }
      },
    }
  },
})
