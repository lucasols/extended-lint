import { ESLintUtils } from '@typescript-eslint/utils'

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
  'throw ',
  'break',
  'continue',
  'try {',
  'catch (',
  'finally {',
  'async ',
  'await ',
]

const codePatterns: (string | RegExp)[] = [
  ') {',
  'return;',
  'return ',
  'if (',
  'else {',
  'else if (',
  'for (',
  'while (',
  'switch (',
  'case ',
  'default:',
  '/>',
  '</',
  '< ',
  '},',
  ': {',
  ' } = ',
  '={',
  /\w=("|'|`)/,
  ');',
  /\w+:\s*('|"|`)/,
  /\[['"][^'"]*['"]\]:\s*('|"|`|[^'"`\s])/,
  /\w+-\w+:/,
  /\w+_\w+:/,
  '&&',
  '||',
  '()',
  /('|"|`),/,
  /\?\s+\w/,
  /:\s+\w/,
  /^\s*('|"|`)[^'"]*('|"|`),?\s*$/,
  /^\s*\d+[,}]/,
  /^\s*\[[^\]]*\][,}]/,
  /^\s*\{[^}]*\}[,}]/,
  /\.\w+\(/,
  /\[\w+\]/,
  '?.(',
  '??',
  '=>',
]

const jsxPatterns: (string | RegExp)[] = [
  '/>',
  '</',
  /^<[A-Z]\w*(\s|>|\/)/,
  /^<[a-z]+(\s|>|\/)/,
  /<[A-Z]\w*(\s.*)?>/,
  /<\/[A-Z]\w*>/,
  /<[a-z]+(\s.*)?>/,
  /<\/[a-z]+>/,
]

const allowedPrefixes = [
  'INFO:',
  'TODO:',
  'DOCS:',
  'FIX:',
  'FIXME:',
  'HACK:',
  'NOTE:',
  'WARNING:',
  'WARN:',
  'BUG:',
  'ISSUE:',
  'TEMP:',
  'TEMPORARY:',
  'XXX:',
  'REVIEW:',
  'eslint',
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
        'Commented code is not allowed. Detected pattern: `{{ wrongPattern }}` Use a comment starting with one of these prefixes if you want to keep this code commented out: {{ allowedPrefixes }}',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    function isCommentedCode(
      comment: string,
      commentType: 'Block' | 'Line',
    ): false | { wrongPattern: string } {
      if (comment.startsWith('/')) {
        return false
      }

      const commentWithTrimmedStart = comment.trimStart()

      if (
        comment.startsWith('*') ||
        commentWithTrimmedStart.startsWith('eslint-disable') ||
        comment.includes('@deprecated') ||
        comment.includes('@example') ||
        comment.includes('@param') ||
        comment.includes('@returns') ||
        comment.includes('@throws') ||
        comment.includes('typescript-eslint') ||
        comment.includes('@ts-') ||
        comment.includes('prettier-ignore') ||
        /^\s*[*\s]*$/.test(comment)
      ) {
        return false
      }

      for (const prefix of allowedPrefixes) {
        if (commentWithTrimmedStart.startsWith(prefix)) {
          return false
        }
      }

      if (commentType === 'Block') {
        for (const pattern of jsxPatterns) {
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
          const commentedCode = isCommentedCode(comment.value, comment.type)

          if (commentedCode) {
            context.report({
              node: comment,
              messageId: 'commentedOutCode',
              data: {
                wrongPattern: commentedCode.wrongPattern,
                allowedPrefixes: allowedPrefixes.join(', '),
              },
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
