import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-commented-out-code'

const startsWithPatterns = [
  'function ',
  'class ',
  'interface ',
  'type ',
  'enum ',
  'namespace ',
  'import ',
  'export ',
  'const ',
  'let ',
  'var ',
  'return ',
]

const codePatterns: (string | RegExp)[] = [
  ') {',
  'return;',
  'if (',
  'else {',
  'for (',
  'switch (',
  '/>',
  '</',
  '},',
  ': {',
  ' } = ',
  '={',
  /\w=("|')/,
  ');',
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
        'Commented code is not allowed. Detected pattern: `{{ wrongPattern }}` Use a block comment `\\* *\\` if you want to keep this code commented out.',
    },
    schema: [],
  },
  defaultOptions: [],
  create: function (context) {
    function isCommentedCode(
      comment: string,
    ): false | { wrongPattern: string } {
      if (comment.startsWith('/')) {
        return false
      }

      const commentWithTrimmedStart = comment.trimStart()

      for (const pattern of startsWithPatterns) {
        if (commentWithTrimmedStart.startsWith(pattern)) {
          return { wrongPattern: pattern }
        }
      }

      for (const pattern of codePatterns) {
        if (typeof pattern === 'string') {
          if (comment.includes(pattern)) {
            return { wrongPattern: pattern }
          }
        } else {
          if (pattern.test(comment)) {
            return { wrongPattern: `regex(${pattern.toString()})` }
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
          if (comment.type === TSESTree.AST_TOKEN_TYPES.Line) {
            const commentedCode = isCommentedCode(comment.value)

            if (commentedCode) {
              context.report({
                node: comment,
                messageId: 'commentedOutCode',
                data: {
                  wrongPattern: commentedCode.wrongPattern,
                },
              })
            }
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
