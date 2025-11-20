import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  functions: z.array(z.string()).optional(),
  selectors: z.array(z.string()).optional(),
})

function isInsideFunctionContext(node: TSESTree.Node): boolean {
  let current = node.parent

  while (current) {
    if (
      current.type === AST_NODE_TYPES.FunctionDeclaration ||
      current.type === AST_NODE_TYPES.FunctionExpression ||
      current.type === AST_NODE_TYPES.ArrowFunctionExpression
    ) {
      return true
    }

    current = current.parent
  }

  return false
}

function getFunctionName(
  node: TSESTree.CallExpression | TSESTree.TaggedTemplateExpression,
): string | null {
  if (node.type === AST_NODE_TYPES.TaggedTemplateExpression) {
    const { tag } = node
    if (tag.type === AST_NODE_TYPES.Identifier) {
      return tag.name
    }
    if (
      tag.type === AST_NODE_TYPES.MemberExpression &&
      tag.property.type === AST_NODE_TYPES.Identifier
    ) {
      return tag.property.name
    }
    return null
  }

  const { callee } = node
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name
  }
  if (
    callee.type === AST_NODE_TYPES.MemberExpression &&
    callee.property.type === AST_NODE_TYPES.Identifier
  ) {
    return callee.property.name
  }

  return null
}

type Options = z.infer<typeof optionsSchema>

export const requireLazyExecution = createExtendedLintRule<
  [Options],
  'moduleLevel'
>({
  name: 'require-lazy-execution',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Require certain function calls to be executed lazily (inside functions, not at module level)',
    },
    messages: {
      moduleLevel:
        'Function "{{ functionName }}" must only be called inside functions or getters, not at module level',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ functions: [], selectors: [] }],
  create(context) {
    const [options] = context.options
    const functionNames = new Set(options.functions ?? [])
    const selectors = options.selectors ?? []

    function checkNode(
      node: TSESTree.CallExpression | TSESTree.TaggedTemplateExpression,
    ): void {
      if (isInsideFunctionContext(node)) return

      const functionName = getFunctionName(node)
      if (functionName && functionNames.has(functionName)) {
        context.report({
          node,
          messageId: 'moduleLevel',
          data: {
            functionName,
          },
        })
      }
    }

    const listeners: Record<string, (node: TSESTree.Node) => void> = {
      CallExpression(node: TSESTree.Node) {
        if (node.type !== AST_NODE_TYPES.CallExpression) return
        checkNode(node)
      },
      TaggedTemplateExpression(node: TSESTree.Node) {
        if (node.type !== AST_NODE_TYPES.TaggedTemplateExpression) return
        checkNode(node)
      },
    }

    for (const selector of selectors) {
      listeners[selector] = (node: TSESTree.Node) => {
        if (
          node.type !== AST_NODE_TYPES.CallExpression &&
          node.type !== AST_NODE_TYPES.TaggedTemplateExpression
        ) {
          return
        }

        if (isInsideFunctionContext(node)) return

        const functionName = getFunctionName(node)
        if (functionName) {
          context.report({
            node,
            messageId: 'moduleLevel',
            data: {
              functionName,
            },
          })
        }
      }
    }

    return listeners
  },
})
