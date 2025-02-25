import { ESLintUtils, TSESTree } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-leaked-text-in-jsx'

type Options = []

const disallowedTexts = [',', ';', '[', ']', '(', ')']

const rule = createRule<Options, 'leakedTextInJSX'>({
  name,
  meta: {
    type: 'problem',
    docs: {
      description:
        'Prevents leaking of text in JSX that should be wrapped in an expression container',
    },
    messages: {
      leakedTextInJSX:
        'Text "{{ text }}" should be wrapped in a JSX expression container.',
    },
    schema: [],
    fixable: 'code',
  },
  defaultOptions: [],
  create(context) {
    return {
      JSXText(node: TSESTree.JSXText) {
        const text = node.value.trim()

        if (!text) return

        let invalidText: string = ''

        if (disallowedTexts.includes(text)) {
          invalidText = text
        } else {
          if (text.includes('&&')) {
            invalidText = '&&'
          } else if (text.includes('||')) {
            invalidText = '||'
          } else if (text.endsWith('? (')) {
            invalidText = '? ('
          }
        }

        if (invalidText) {
          context.report({
            node,
            messageId: 'leakedTextInJSX',
            data: { text: invalidText },
          })
        }
      },
    }
  },
})

export const noLeakedTextInJSX = {
  name,
  rule,
}
