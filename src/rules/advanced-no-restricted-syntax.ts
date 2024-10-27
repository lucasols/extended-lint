import { ESLintUtils } from '@typescript-eslint/utils'
import { ReportFixFunction } from '../../node_modules/.pnpm/@typescript-eslint+utils@8.2.0_eslint@9.9.1_typescript@5.5.4/node_modules/@typescript-eslint/utils/dist/ts-eslint/Rule.d.ts'
import { TSESTree } from '../../node_modules/.pnpm/@typescript-eslint+utils@8.2.0_eslint@9.9.1_typescript@5.5.4/node_modules/@typescript-eslint/utils/dist/ts-estree'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

export type Options = [
  {
    disallow: {
      selector: string
      message: string
      replace?:
        | string
        | {
            regex: string
            with: string
          }
      replaceType?: 'suggestion' | 'autofix'
    }[]
  },
]

const name = 'advanced-no-restricted-syntax'

const rule = createRule<Options, 'default'>({
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
                replace: {
                  oneOf: [
                    { type: 'string' },
                    {
                      type: 'object',
                      properties: {
                        regex: { type: 'string' },
                        with: { type: 'string' },
                      },
                      required: ['regex', 'with'],
                      additionalProperties: false,
                    },
                  ],
                },
                replaceType: {
                  type: 'string',
                  enum: ['suggestion', 'autofix'],
                },
              },
              required: ['selector', 'message'],
              additionalProperties: false,
            },
          },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      default: '{{message}}',
    },
    fixable: 'code',
    hasSuggestions: true,
  },
  defaultOptions: [{ disallow: [] }],
  create(context) {
    const result: Record<
      string,
      (node: TSESTree.Node | TSESTree.Token) => void
    > = {}

    for (const {
      selector,
      message,
      replace,
      replaceType = 'suggestion',
    } of context.options[0].disallow ?? []) {
      result[selector] = (node) => {
        const fixFn: ReportFixFunction = (fixer) => {
          if (!replace) return null

          if (typeof replace === 'string') {
            return fixer.replaceText(node, replace)
          } else {
            const replaceRegex = new RegExp(replace.regex)

            const nodeText = context.sourceCode.getText(node)

            return fixer.replaceText(
              node,
              nodeText.replace(replaceRegex, replace.with),
            )
          }
        }

        context.report({
          node,
          messageId: 'default',
          data: { message },
          fix: replace && replaceType === 'autofix' ? fixFn : undefined,
          suggest:
            replace && replaceType === 'suggestion'
              ? [
                  {
                    messageId: 'default',
                    data: {
                      message: `Replace with "${
                        typeof replace === 'string' ? replace : replace.with
                      }"`,
                    },
                    fix: fixFn,
                  },
                ]
              : undefined,
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
