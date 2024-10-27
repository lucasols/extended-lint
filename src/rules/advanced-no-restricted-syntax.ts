import { ESLintUtils } from '@typescript-eslint/utils'
import { TSESTree } from '../../node_modules/.pnpm/@typescript-eslint+utils@8.2.0_eslint@9.9.1_typescript@5.5.4/node_modules/@typescript-eslint/utils/dist/ts-estree'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

export type Options = [
  {
    disallow?: {
      selector: string
      message: string
    }[]
    mustMatch?: {
      selector: string
      message: string
    }[]
  },
]

const name = 'advanced-no-restricted-syntax'

const rule = createRule<Options, 'restrictedSyntax'>({
  name,
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Disallow specific syntax patterns',
    },
    schema: [
      {
        type: 'object',
        properties: {
          disallow: {
            type: 'array',
            items: {
              type: 'object',
              properties: {
                selector: { type: 'string' },
                message: { type: 'string' },
              },
              required: ['selector'],
            },
          },
          mustMatch: {
            type: 'array',
            items: {
              type: 'object',
              properties: {
                selector: { type: 'string' },
                message: { type: 'string' },
              },
              required: ['selector'],
            },
          },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      restrictedSyntax: '{{message}}',
    },
  },
  defaultOptions: [{ disallow: [], mustMatch: [] }],
  create(context) {
    const result: Record<
      string,
      (node: TSESTree.Node | TSESTree.Token) => void
    > = {}

    for (const { selector, message } of context.options[0].disallow ?? []) {
      result[selector] = (node) => {
        context.report({
          node,
          messageId: 'restrictedSyntax',
          data: { message },
        })
      }
    }

    return result
  },
})

export const advancedNoRestrictedSyntax = {
  name,
  rule,
}
