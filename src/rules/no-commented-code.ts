import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-commented-out-code'
const codePatterns: (string | RegExp)[] = [
  ') {',
  'return;',
  'if (',
  'else {',
  'for (',
  'switch (',
  '/>',
  '</',
  'const ',
  'let ',
  'var ',
  '},',
  ': {',
  ' } = ',
  '={',
  /\w=("|')/,
  ');',
  /type \w+ =/,
]

const rule = createRule({
  name,
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow commented code',
      recommended: 'error',
    },
    messages: {
      commentedOutCode:
        'Commented code is not allowed. Use a block comment `\\* *\\` if you want to keep this code commented out.',
    },
    schema: [],
  },
  defaultOptions: [],
  create: function (context) {
    function isCommentedCode(comment: string) {
      if (comment.startsWith('/')) {
        return false
      }

      for (const pattern of codePatterns) {
        if (typeof pattern === 'string') {
          if (comment.includes(pattern)) {
            return true
          }
        } else {
          if (pattern.test(comment)) {
            return true
          }
        }
      }

      return false
    }

    return {
      Program() {
        const sourceCode = context.getSourceCode()
        const comments = sourceCode.getAllComments()

        for (const comment of comments) {
          if (
            comment.type === TSESTree.AST_TOKEN_TYPES.Line &&
            isCommentedCode(comment.value)
          ) {
            context.report({
              node: comment,
              messageId: 'commentedOutCode',
            })
          }
        }
      },
    }
  },
})

export const noCommentedOutCode = {
  name,
  rule,
}
