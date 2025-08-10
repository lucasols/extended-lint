import {
  AST_NODE_TYPES,
  AST_TOKEN_TYPES,
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

    function getIndentString(): string {
      if (typeof indent === 'string') {
        return indent
      }
      if (typeof indent === 'number') {
        return ' '.repeat(indent)
      }
      return '  '
    }

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

    function getFunctionName(callee: TSESTree.Expression): string | null {
      if (callee.type === AST_NODE_TYPES.Identifier) {
        return callee.name
      }
      if (callee.type === AST_NODE_TYPES.MemberExpression) {
        const object =
          callee.object.type === AST_NODE_TYPES.Identifier
            ? callee.object.name
            : null
        const property =
          callee.property.type === AST_NODE_TYPES.Identifier
            ? callee.property.name
            : null
        return object && property ? `${object}.${property}` : property
      }
      return null
    }

    function isJestInlineSnapshot(node: TSESTree.TemplateLiteral): boolean {
      const parent = node.parent
      if (
        parent.type === AST_NODE_TYPES.CallExpression &&
        parent.arguments[0] === node &&
        parent.callee.type === AST_NODE_TYPES.MemberExpression &&
        parent.callee.property.type === AST_NODE_TYPES.Identifier &&
        parent.callee.property.name === 'toMatchInlineSnapshot' &&
        parent.arguments.length === 1 &&
        parent.callee.object.type === AST_NODE_TYPES.CallExpression &&
        parent.callee.object.callee.type === AST_NODE_TYPES.Identifier &&
        parent.callee.object.callee.name === 'expect' &&
        parent.callee.object.arguments.length === 1
      ) {
        return true
      }
      return false
    }

    function shouldIndent(node: TSESTree.TemplateLiteral): boolean {
      const previousToken = sourceCode.getTokenBefore(node, {
        includeComments: true,
      })
      if (
        previousToken?.type === AST_TOKEN_TYPES.Block &&
        normalizedComments.includes(previousToken.value.trim().toLowerCase())
      ) {
        return true
      }

      if (isJestInlineSnapshot(node)) {
        return true
      }

      const parent = node.parent
      if (parent.type === AST_NODE_TYPES.TaggedTemplateExpression) {
        const tagName = getTagName(parent.tag)
        if (tagName && tags.includes(tagName)) {
          return true
        }
      }

      if (
        parent.type === AST_NODE_TYPES.CallExpression &&
        parent.arguments.includes(node)
      ) {
        const functionName = getFunctionName(parent.callee)
        if (functionName && functions.includes(functionName)) {
          return true
        }
      }

      return false
    }

    return {
      TemplateLiteral(node) {
        if (!shouldIndent(node)) {
          return
        }

        const delimiter = `__PLACEHOLDER__${Math.random()}`
        const templateContent = node.quasis
          .map((quasi) => {
            const text = sourceCode.getText(quasi)
            return text.slice(1, quasi.tail ? -1 : -2)
          })
          .join(delimiter)

        const eolMatch = templateContent.match(/\r?\n/)
        if (!eolMatch) {
          return
        }

        const eol = eolMatch[0]
        const loc = sourceCode.getLocFromIndex(node.range[0])
        const startLine = sourceCode.lines[loc.line - 1]
        if (!startLine) return

        const marginMatch = startLine.match(/^(\s*)\S/)
        const parentMargin = marginMatch?.[1] ?? ''
        const indentString = getIndentString()

        const dedented = stripIndent(templateContent)
        const trimmed = dedented.replace(
          new RegExp(`^${eol}|${eol}[ \t]*$`, 'g'),
          '',
        )

        const lines = trimmed.split(/\r?\n/)
        const indentedLines = lines.map((line, index) => {
          if (index === 0 || line.trim() === '') return line
          return parentMargin + indentString + line
        })

        const fixed = eol + indentedLines.join(eol) + eol + parentMargin

        if (fixed === templateContent) {
          return
        }

        return context.report({
          node,
          messageId: 'improperlyIndented',
          fix(fixer) {
            const parts = fixed.split(delimiter)
            const fixes = []
            for (let i = 0; i < node.quasis.length; i++) {
              const quasi = node.quasis[i]
              if (!quasi) continue
              const replacement = parts[i]
              if (replacement !== undefined) {
                fixes.push(
                  fixer.replaceTextRange(
                    [quasi.range[0] + 1, quasi.range[1] - (quasi.tail ? 1 : 2)],
                    replacement,
                  ),
                )
              }
            }
            return fixes
          },
        })
      },
    }
  },
})
