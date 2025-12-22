import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const name = 'no-restricted-comments'

const patternSchema = z.object({
  regex: z.string().optional(),
  includes: z.string().optional(),
  message: z.string().optional(),
  autoFix: z.boolean().optional(),
})

const optionsSchema = z.object({
  patterns: z.array(patternSchema),
})

type Options = z.infer<typeof optionsSchema>

export const noRestrictedComments = createExtendedLintRule<
  [Options],
  'restrictedComment' | 'restrictedCommentWithMessage' | 'removeComment'
>({
  name,
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow comments matching specific patterns',
    },
    fixable: 'code',
    hasSuggestions: true,
    schema: [getJsonSchemaFromZod(optionsSchema)],
    messages: {
      restrictedComment: "Comment matches restricted pattern '{{pattern}}'",
      restrictedCommentWithMessage: '{{message}}',
      removeComment: 'Remove this comment',
    },
  },
  defaultOptions: [{ patterns: [] }],
  create(context, [options]) {
    const sourceCode = context.sourceCode
    const regexCache = new Map<string, RegExp>()

    function getCompiledRegex(pattern: string): RegExp {
      let compiled = regexCache.get(pattern)
      if (!compiled) {
        compiled = new RegExp(pattern)
        regexCache.set(pattern, compiled)
      }
      return compiled
    }

    function matchesPattern(
      commentValue: string,
      pattern: z.infer<typeof patternSchema>,
    ): boolean {
      if (pattern.regex) {
        const regex = getCompiledRegex(pattern.regex)
        return regex.test(commentValue)
      }
      if (pattern.includes) {
        return commentValue.includes(pattern.includes)
      }
      return false
    }

    return {
      Program() {
        const comments = sourceCode.getAllComments()

        for (const comment of comments) {
          for (const pattern of options.patterns) {
            if (matchesPattern(comment.value, pattern)) {
              const patternDisplay = pattern.regex ?? pattern.includes ?? ''

              function removeFix(fixer: Parameters<Parameters<typeof context.report>[0]['fix'] & {}>[0]) {
                return fixer.remove(comment)
              }

              if (pattern.message) {
                context.report({
                  loc: comment.loc,
                  messageId: 'restrictedCommentWithMessage',
                  data: {
                    message: pattern.message,
                  },
                  fix: pattern.autoFix ? removeFix : undefined,
                  suggest: pattern.autoFix
                    ? undefined
                    : [
                        {
                          messageId: 'removeComment',
                          fix: removeFix,
                        },
                      ],
                })
              } else {
                context.report({
                  loc: comment.loc,
                  messageId: 'restrictedComment',
                  data: {
                    pattern: patternDisplay,
                  },
                  fix: pattern.autoFix ? removeFix : undefined,
                  suggest: pattern.autoFix
                    ? undefined
                    : [
                        {
                          messageId: 'removeComment',
                          fix: removeFix,
                        },
                      ],
                })
              }
              break
            }
          }
        }
      },
    }
  },
})
