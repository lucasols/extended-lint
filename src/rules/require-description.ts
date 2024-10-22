import { ESLintUtils, TSESTree } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'require-description'

const DIRECTIVE_PATTERN =
  /^(eslint(?:-env|-enable|-disable(?:(?:-next)?-line)?)?|exported|globals?)(?:\s|$)/u
const LINE_COMMENT_PATTERN = /^eslint-disable-(next-)?line$/u

type Options = [
  {
    ignore?: string[]
  },
]

const rule = createRule<Options, 'missingDescription'>({
  name,
  meta: {
    type: 'problem',
    docs: {
      description: 'Require descriptions for eslint directives.',
    },
    messages: {
      missingDescription:
        'Unexpected undescribed directive comment. Include descriptions to explain why the comment is necessary.',
    },
    schema: [
      {
        type: 'object',
        properties: {
          ignore: {
            type: 'array',
            items: {
              type: 'string',
              enum: [
                'eslint',
                'eslint-disable',
                'eslint-disable-line',
                'eslint-disable-next-line',
                'eslint-enable',
                'eslint-env',
                'exported',
                'global',
                'globals',
              ],
            },
            additionalItems: false,
            uniqueItems: true,
          },
        },
        additionalProperties: false,
      },
    ],
  },
  defaultOptions: [{ ignore: [] }],
  create(context) {
    const sourceCode = context.sourceCode
    const ignores = new Set(
      (context.options[0] && context.options[0].ignore) || [],
    )

    return {
      Program() {
        for (const comment of sourceCode.getAllComments()) {
          const directiveComment = parseDirectiveComment(comment)
          if (directiveComment == null) {
            continue
          }
          if (ignores.has(directiveComment.kind)) {
            continue
          }
          if (!directiveComment.description) {
            context.report({
              loc: toForceLocation(comment.loc),
              messageId: 'missingDescription',
            })
          }
        }
      },
    }
  },
})

export const requireDescription = {
  name,
  rule,
}

function parseDirectiveComment(comment: TSESTree.Comment) {
  const { text, description } = divideDirectiveComment(comment.value)

  if (!text) {
    return null
  }

  const match = DIRECTIVE_PATTERN.exec(text)

  if (!match) {
    return null
  }

  const directiveText = match[1]

  if (!directiveText) {
    return null
  }

  const lineCommentSupported = LINE_COMMENT_PATTERN.test(directiveText)

  if (comment.type === 'Line' && !lineCommentSupported) {
    return null
  }

  if (lineCommentSupported && comment.loc.start.line !== comment.loc.end.line) {
    // disable-line comment should not span multiple lines.
    return null
  }

  const directiveValue = text.slice(match.index + directiveText.length)

  return {
    kind: directiveText,
    value: directiveValue.trim(),
    description,
  }
}

function divideDirectiveComment(value: string) {
  const divided = value.split(/\s-{2,}\s/u)
  const text = divided[0]?.trim()
  return {
    text,
    description: divided.length > 1 ? divided[1]?.trim() ?? null : null,
  }
}

function toForceLocation(location: TSESTree.SourceLocation) {
  return {
    start: {
      line: location.start.line,
      column: -1,
    },
    end: location.end,
  }
}
