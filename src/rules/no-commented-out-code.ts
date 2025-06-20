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
  /^\s*return\s+/,
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
  /^\s*(['"`]).+?\1\s*:/,
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

      // Check if comment contains URLs - treat as descriptive text
      if (comment.includes('https://')) {
        return false
      }

      // Generic structural detection without relying on specific words
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

    function stripInlineCodeIfTextOutside(
      original: string,
      commentType: 'Block' | 'Line',
    ): string {
      let cleaned = original

      // 0. If comment is just a back-ticked string (optionally ending with ',' or ';')
      const trimmed = cleaned.trim()
      if (trimmed.startsWith('`')) {
        if (
          trimmed.endsWith('`,') ||
          trimmed.endsWith('`;') ||
          trimmed.endsWith('`')
        ) {
          return cleaned // treat as pure code – do not strip
        }
      }

      // 1. Handle fenced blocks ``` ... ``` only for block comments.
      if (commentType === 'Block' && cleaned.includes('```')) {
        const parts = cleaned.split(/```[\s\S]*?```/g)
        const hasText = parts.some((p) => /[a-zA-Z0-9]/.test(p))
        if (hasText) cleaned = parts.join('')
      }

      // 2. Handle single-backtick inline code
      if (!cleaned.includes('`')) return cleaned

      const segments = cleaned.split(/`[^`]*`/g)
      const hasOuterText = segments.some((seg) => /[a-zA-Z0-9]/.test(seg))

      return hasOuterText ? segments.join('') : cleaned
    }

    return {
      Program() {
        const sourceCode = context.sourceCode
        const comments = sourceCode.getAllComments()

        for (const comment of comments) {
          const processed = stripInlineCodeIfTextOutside(
            comment.value,
            comment.type,
          )
          const commentedCode = isCommentedCode(processed, comment.type)

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
