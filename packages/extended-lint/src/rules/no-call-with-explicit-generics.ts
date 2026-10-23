import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  functions: z.array(
    z.string().or(z.object({ name: z.string(), message: z.string() })),
  ),
})

function getCalleeName(
  callee: TSESTree.Expression,
): string | undefined {
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name
  }

  if (
    callee.type === AST_NODE_TYPES.MemberExpression &&
    callee.property.type === AST_NODE_TYPES.Identifier
  ) {
    if (callee.object.type === AST_NODE_TYPES.Identifier) {
      return `${callee.object.name}.${callee.property.name}`
    }
  }

  return undefined
}

type Options = z.infer<typeof optionsSchema>

export const noCallWithExplicitGenerics = createExtendedLintRule<
  [Options],
  'noExplicitGenerics' | 'noExplicitGenericsCustom'
>({
  name: 'no-call-with-explicit-generics',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Enforce calling configured functions with inferred generics only',
    },
    messages: {
      noExplicitGenerics: `Function '{{ functionName }}' should be called with inferred generics (remove the explicit type parameters)`,
      noExplicitGenericsCustom: `{{ message }}`,
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ functions: [] }],
  create(context, [options]) {
    const exactMatches = new Map<string, string | undefined>()
    const wildcardMatches = new Map<string, string | undefined>()

    for (const fn of options.functions) {
      const name = typeof fn === 'string' ? fn : fn.name
      const message = typeof fn === 'string' ? undefined : fn.message

      if (name.startsWith('*.')) {
        wildcardMatches.set(name.slice(2), message)
      } else {
        exactMatches.set(name, message)
      }
    }

    function findMatch(
      functionName: string,
    ): { message: string | undefined } | undefined {
      if (exactMatches.has(functionName)) {
        return { message: exactMatches.get(functionName) }
      }

      const dotIndex = functionName.indexOf('.')

      if (dotIndex !== -1) {
        const methodName = functionName.slice(dotIndex + 1)

        if (wildcardMatches.has(methodName)) {
          return { message: wildcardMatches.get(methodName) }
        }
      }

      return undefined
    }

    return {
      CallExpression(node) {
        const functionName = getCalleeName(node.callee)

        if (!functionName) return

        if (!node.typeArguments) return

        const match = findMatch(functionName)

        if (!match) return

        if (match.message) {
          context.report({
            node,
            messageId: 'noExplicitGenericsCustom',
            data: { message: match.message },
          })
        } else {
          context.report({
            node,
            messageId: 'noExplicitGenerics',
            data: { functionName },
          })
        }
      },
    }
  },
})
