import { AST_NODE_TYPES, ESLintUtils } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { getJsonSchemaFromZod } from '../createRule'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-call-with-explicit-generics'

const optionsSchema = z.object({
  functions: z.array(z.string()),
})

type Options = z.infer<typeof optionsSchema>

const rule = createRule<[Options], 'noExplicitGenerics'>({
  name,
  meta: {
    type: 'problem',
    docs: {
      description:
        'Enforce calling configured functions with inferred generics only',
    },
    messages: {
      noExplicitGenerics: `Function '{{ functionName }}' should be called with inferred generics (remove the explicit type parameters)`,
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ functions: [] }],
  create(context, [options]) {
    const functionNames = new Set(options.functions)

    return {
      CallExpression(node) {
        const { callee } = node

        if (callee.type !== AST_NODE_TYPES.Identifier) return

        if (!functionNames.has(callee.name)) return

        if (node.typeArguments) {
          context.report({
            node,
            messageId: 'noExplicitGenerics',
            data: {
              functionName: callee.name,
            },
          })
        }
      },
    }
  },
})

export const noCallWithExplicitGenerics = {
  name,
  rule,
}
