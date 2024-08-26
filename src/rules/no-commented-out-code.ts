import { ESLintUtils, TSESTree } from '@typescript-eslint/utils'

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
    },
    messages: {
      commentedOutCode:
        'Commented code is not allowed. Detected pattern: `{{ wrongPattern }}` Use a comment starting with `INFO:` if you want to keep this code commented out.',
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

      if (
        comment.startsWith('*') ||
        commentWithTrimmedStart.startsWith('INFO:') ||
        commentWithTrimmedStart.startsWith('TODO:') ||
        commentWithTrimmedStart.startsWith('FIX:') ||
        commentWithTrimmedStart.startsWith('eslint-disable') ||
        comment.includes('@deprecated') ||
        comment.includes('@example')
      ) {
        return false
      }

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
        const sourceCode = context.sourceCode
        const comments = sourceCode.getAllComments()

        for (const comment of comments) {
          if (
            comment.type === TSESTree.AST_TOKEN_TYPES.Line ||
            comment.type === TSESTree.AST_TOKEN_TYPES.Block
          ) {
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
