import { AST_NODE_TYPES, ESLintUtils } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-call-with-infered-generics'

export type Options = [
  {
    functions: {
      name: string
      minGenerics?: number
      allowAny?: boolean
      disallowTypes?: string[]
    }[]
    disallowTypes?: string[]
  },
]

const rule = createRule<
  Options,
  'missingGenericDeclaration' | 'anyUsedInGenerics'
>({
  name,
  meta: {
    type: 'problem',
    docs: {
      description: 'Disable calling configured functions with infered generics',
    },
    messages: {
      missingGenericDeclaration: `Function '{{ functionName }}' should be called with at least {{ minGenerics }} generic(s) (ex: \`fn<Generic>()\`) defined`,
      anyUsedInGenerics: `Function '{{ functionName }}' should not be called with 'any' in generics`,
    },
    schema: [
      {
        type: 'object',
        properties: {
          functions: {
            type: 'array',
            items: {
              type: 'object',
              properties: {
                name: { type: 'string' },
                minGenerics: { type: 'number' },
                allowAny: { type: 'boolean' },
                disallowTypes: { type: 'array', items: { type: 'string' } },
              },
              required: ['name'],
            },
          },
          anyAliases: { type: 'array', items: { type: 'string' } },
        },
        required: ['functions'],
      },
    ],
  },
  defaultOptions: [{ functions: [] }],
  create(context) {
    const options = context.options[0]

    const functionConfigMap = new Map<string, Options[0]['functions'][0]>(
      options.functions.map((config) => [config.name, config]),
    )

    return {
      CallExpression(node) {
        const { callee } = node

        if (callee.type !== AST_NODE_TYPES.Identifier) return

        const functionConfig = functionConfigMap.get(callee.name)

        if (!functionConfig) return

        const {
          minGenerics = 1,
          allowAny,
          disallowTypes = options.disallowTypes,
        } = functionConfig

        const generics = node.typeArguments?.params.length || 0

        if (generics < (minGenerics || 0)) {
          context.report({
            node,
            messageId: 'missingGenericDeclaration',
            data: {
              functionName: callee.name,
              minGenerics: minGenerics || 0,
            },
          })
        }

        if (allowAny && !disallowTypes) return

        if (
          node.typeArguments?.params?.some(
            (type) =>
              (!allowAny && type.type === AST_NODE_TYPES.TSAnyKeyword) ||
              (disallowTypes &&
                type.type === AST_NODE_TYPES.TSTypeReference &&
                type.typeName.type === AST_NODE_TYPES.Identifier &&
                disallowTypes.includes(type.typeName.name)),
          )
        ) {
          context.report({
            node,
            messageId: 'anyUsedInGenerics',
            data: {
              functionName: callee.name,
            },
          })
        }
      },
    }
  },
})

export const noCallWithInferedGenerics = {
  name,
  rule,
}
