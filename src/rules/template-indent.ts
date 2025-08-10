import {
  AST_NODE_TYPES,
  AST_TOKEN_TYPES,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import * as z from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  indent: z
    .union([z.string().regex(/^\s+$/), z.number().int().min(1)])
    .optional(),
  tags: z.array(z.string()).optional(),
  functions: z.array(z.string()).optional(),
  comments: z.array(z.string()).optional(),
})

type Options = z.infer<typeof optionsSchema>

export const templateIndent = createExtendedLintRule<
  [Options],
  'improperlyIndented'
>({
  name: 'template-indent',
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Fix whitespace-insensitive template indentation',
    },
    fixable: 'code',
    schema: [getJsonSchemaFromZod(optionsSchema)],
    messages: {
      improperlyIndented: 'Templates should be properly indented.',
    },
  },
  defaultOptions: [{}],
  create(context) {
    const sourceCode = context.sourceCode
    const {
      comments = ['HTML', 'indent'],
      functions = ['dedent', 'stripIndent'],
      tags = ['outdent', 'dedent', 'gql', 'sql', 'html', 'styled'],
      indent,
    } = context.options[0]

    const normalizedComments = comments.map((comment) => comment.toLowerCase())

    function stripIndent(str: string): string {
      const lines = str.split(/\r?\n/)

      const nonEmptyLines = lines.filter((line) => line.trim() !== '')
      if (nonEmptyLines.length === 0) return str

      let minIndent = Number.POSITIVE_INFINITY
      for (const line of nonEmptyLines) {
        const match = line.match(/^(\s*)/)
        if (match) {
          minIndent = Math.min(minIndent, match[1]?.length ?? 0)
        }
      }

      if (!isFinite(minIndent) || minIndent === 0) {
        return str
      }

      return lines.map((line) => line.slice(minIndent)).join('\n')
    }

    function indentString(
      str: string,
      count: number,
      indentStr: string,
    ): string {
      const lines = str.split(/\r?\n/)
      return lines
        .map((line) => {
          if (line.trim() === '') return line
          return indentStr.repeat(count) + line
        })
        .join('\n')
    }

    function getTagName(tag: TSESTree.Expression): string | null {
      if (tag.type === AST_NODE_TYPES.Identifier) {
        return tag.name
      }
      if (tag.type === AST_NODE_TYPES.MemberExpression) {
        const object =
          tag.object.type === AST_NODE_TYPES.Identifier ? tag.object.name : null
        const property =
          tag.property.type === AST_NODE_TYPES.Identifier
            ? tag.property.name
            : null
        return object && property ? `${object}.${property}` : property
      }
      return null
    }

    function isNodeMatches(
      node: TSESTree.Node | null,
      patterns: string[],
    ): boolean {
      if (!node) return false

      for (const pattern of patterns) {
        if (pattern.includes('.')) {
          const parts = pattern.split('.')
          if (
            node.type === AST_NODE_TYPES.MemberExpression &&
            node.object.type === AST_NODE_TYPES.Identifier &&
            node.property.type === AST_NODE_TYPES.Identifier &&
            node.object.name === parts[0] &&
            node.property.name === parts[1]
          ) {
            return true
          }
        } else {
          if (
            node.type === AST_NODE_TYPES.Identifier &&
            node.name === pattern
          ) {
            return true
          }
        }
      }

      return false
    }

    function isTaggedTemplateLiteral(
      node: TSESTree.TemplateLiteral,
      tagPatterns: string[],
    ): boolean {
      const parent = node.parent
      if (parent.type !== AST_NODE_TYPES.TaggedTemplateExpression) {
        return false
      }

      const tagName = getTagName(parent.tag)
      return tagName ? tagPatterns.includes(tagName) : false
    }

    function isMethodCall(
      node: TSESTree.Node | null,
      options: {
        method: string
        argumentsLength?: number
        optionalCall?: boolean
        optionalMember?: boolean
      },
    ): boolean {
      if (!node || node.type !== AST_NODE_TYPES.CallExpression) {
        return false
      }

      const {
        method,
        argumentsLength,
        optionalCall = true,
        optionalMember = true,
      } = options

      if (
        argumentsLength !== undefined &&
        node.arguments.length !== argumentsLength
      ) {
        return false
      }

      if (!optionalCall && node.optional) {
        return false
      }

      if (node.callee.type !== AST_NODE_TYPES.MemberExpression) {
        return false
      }

      if (!optionalMember && node.callee.optional) {
        return false
      }

      if (node.callee.property.type !== AST_NODE_TYPES.Identifier) {
        return false
      }

      return node.callee.property.name === method
    }

    function isCallExpression(
      node: TSESTree.Node | null,
      options: {
        name: string
        argumentsLength?: number
        optionalCall?: boolean
        optionalMember?: boolean
      },
    ): boolean {
      if (!node || node.type !== AST_NODE_TYPES.CallExpression) {
        return false
      }

      const { name, argumentsLength, optionalCall = true } = options

      if (
        argumentsLength !== undefined &&
        node.arguments.length !== argumentsLength
      ) {
        return false
      }

      if (!optionalCall && node.optional) {
        return false
      }

      if (node.callee.type !== AST_NODE_TYPES.Identifier) {
        return false
      }

      return node.callee.name === name
    }

    function isJestInlineSnapshot(node: TSESTree.TemplateLiteral): boolean {
      return (
        isMethodCall(node.parent, {
          method: 'toMatchInlineSnapshot',
          argumentsLength: 1,
          optionalCall: false,
          optionalMember: false,
        }) &&
        node.parent.type === AST_NODE_TYPES.CallExpression &&
        node.parent.arguments[0] === node &&
        isCallExpression(
          node.parent.callee.type === AST_NODE_TYPES.MemberExpression
            ? node.parent.callee.object
            : null,
          {
            name: 'expect',
            argumentsLength: 1,
            optionalCall: false,
            optionalMember: false,
          },
        )
      )
    }

    function shouldIndent(node: TSESTree.TemplateLiteral): boolean {
      if (normalizedComments.length > 0) {
        const previousToken = sourceCode.getTokenBefore(node, {
          includeComments: true,
        })
        if (
          previousToken?.type === AST_TOKEN_TYPES.Block &&
          normalizedComments.includes(previousToken.value.trim().toLowerCase())
        ) {
          return true
        }
      }

      if (isJestInlineSnapshot(node)) {
        return true
      }

      if (tags.length > 0 && isTaggedTemplateLiteral(node, tags)) {
        return true
      }

      if (
        functions.length > 0 &&
        node.parent.type === AST_NODE_TYPES.CallExpression &&
        node.parent.arguments.includes(node) &&
        isNodeMatches(node.parent.callee, functions)
      ) {
        return true
      }

      return false
    }

    function getProblem(node: TSESTree.TemplateLiteral) {
      const delimiter = `__PLACEHOLDER__${Math.random()}`
      const joined = node.quasis
        .map((quasi) => {
          const untrimmedText = sourceCode.getText(quasi)
          return untrimmedText.slice(1, quasi.tail ? -1 : -2)
        })
        .join(delimiter)

      const eolMatch = joined.match(/\r?\n/)
      if (!eolMatch) {
        return
      }

      const eol = eolMatch[0]
      const startLine =
        sourceCode.lines[sourceCode.getLocFromIndex(node.range[0]).line - 1]
      if (!startLine) return

      const marginMatch = startLine.match(/^(\s*)\S/)
      const parentMargin = marginMatch?.[1] ?? ''

      let indentStr: string
      if (typeof indent === 'string') {
        indentStr = indent
      } else if (typeof indent === 'number') {
        indentStr = ' '.repeat(indent)
      } else {
        const tabs = parentMargin.startsWith('\t')
        indentStr = tabs ? '\t' : '  '
      }

      const dedented = stripIndent(joined)
      const trimmed = dedented.replace(
        new RegExp(`^${eol}|${eol}[ \t]*$`, 'g'),
        '',
      )

      const fixed =
        eol +
        indentString(trimmed, 1, parentMargin + indentStr) +
        eol +
        parentMargin

      const joiner = joined.includes('\r\n') ? '\r\n' : '\n'

      const joinedLines = joined.split(/\r?\n/)

      const normalizedJoined = joinedLines
        .map((line, i) =>
          i !== joinedLines.length - 1 && line === indentStr + parentMargin
            ? ''
            : line,
        )
        .join(joiner)

      if (fixed === normalizedJoined) {
        return
      }

      return {
        node,
        messageId: 'improperlyIndented' as const,
        fix: (fixer: TSESLint.RuleFixer) =>
          fixed
            .split(delimiter)
            .map((replacement, index) => {
              const quasi = node.quasis[index]
              if (!quasi) return []
              return fixer.replaceTextRange(
                [quasi.range[0] + 1, quasi.range[1] - (quasi.tail ? 1 : 2)],
                replacement,
              )
            })
            .flat(),
      }
    }

    return {
      TemplateLiteral(node) {
        if (!shouldIndent(node)) {
          return
        }

        const problem = getProblem(node)
        if (problem) {
          return context.report(problem)
        }
      },
    }
  },
})
