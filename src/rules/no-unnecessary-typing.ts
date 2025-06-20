import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import * as z from 'zod/v4'
import { createExtendedLintRule } from '../createRule'

const optionsSchema = z.object({
  methods: z.union([z.array(z.string()), z.literal('array')]),
})

type Options = z.infer<typeof optionsSchema>

export const noUnnecessaryTyping = createExtendedLintRule<
  [Options],
  'unnecessaryTypeAnnotation'
>({
  name: 'no-unnecessary-typing',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Prevents unnecessary explicit type annotations in callback parameters where TypeScript can infer the type',
    },
    messages: {
      unnecessaryTypeAnnotation:
        'Unnecessary type annotation. TypeScript can infer this type from context.',
    },
    fixable: 'code',
    schema: [z.toJSONSchema(optionsSchema) as any],
  },
  defaultOptions: [{ methods: 'array' }],
  create(context, [options]) {
    const methods = options.methods

    const defaultArrayMethods = new Set([
      'find',
      'filter',
      'map',
      'forEach',
      'some',
      'every',
      'reduce',
      'findIndex',
      'sort',
    ])

    function shouldCheckMethod(methodName: string): boolean {
      if (Array.isArray(methods)) {
        // Specific methods mode - check if method is in the array
        return methods.includes(methodName)
      } else {
        // Array methods mode - check built-in array methods
        return defaultArrayMethods.has(methodName)
      }
    }

    function getMethodName(
      callExpression: TSESTree.CallExpression,
    ): string | null {
      const { callee } = callExpression

      if (callee.type === AST_NODE_TYPES.MemberExpression) {
        const property = callee.property
        if (property.type === AST_NODE_TYPES.Identifier) {
          return property.name
        }
      }

      return null
    }

    function checkFunctionParameter(
      param: TSESTree.Parameter,
      functionNode:
        | TSESTree.ArrowFunctionExpression
        | TSESTree.FunctionExpression,
    ) {
      if (param.type !== AST_NODE_TYPES.Identifier || !param.typeAnnotation) {
        return
      }

      const parent = functionNode.parent

      if (parent.type === AST_NODE_TYPES.CallExpression) {
        const methodName = getMethodName(parent)
        if (methodName && shouldCheckMethod(methodName)) {
          const argumentIndex = parent.arguments.indexOf(functionNode)

          // For most methods, the first argument is the callback
          if (argumentIndex === 0) {
            context.report({
              node: param.typeAnnotation,
              messageId: 'unnecessaryTypeAnnotation',
              fix(fixer) {
                return fixer.remove(param.typeAnnotation!)
              },
            })
          }
        }
      }
    }

    return {
      ArrowFunctionExpression(node) {
        for (const param of node.params) {
          checkFunctionParameter(param, node)
        }
      },
      FunctionExpression(node) {
        for (const param of node.params) {
          checkFunctionParameter(param, node)
        }
      },
    }
  },
})
