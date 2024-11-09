import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import { ReportFixFunction } from '@typescript-eslint/utils/ts-eslint'
import * as t from 'tschema'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const optionSchema = t.object({
  disallow: t.optional(
    t.array(
      t.object({
        selector: t.string(),
        message: t.string(),
        replace: t.optional(
          t.any(t.string(), t.object({ regex: t.string(), with: t.string() })),
        ),
        replaceType: t.optional(t.enum(['suggestion', 'autofix'])),
      }),
    ),
  ),
  __dev_simulateFileName: t.optional(t.string()),
  mustCallFn: t.optional(
    t.dict(
      t.object({
        getFileNameVarsRegex: t.optional(t.string()),
        args: t.array(
          t.object({
            pos: t.integer(),
            literal: t.any(t.string(), t.number(), t.boolean()),
          }),
        ),
        message: t.string(),
      }),
    ),
  ),
})

export type Options = t.Infer<typeof optionSchema>

const name = 'advanced-no-restricted-syntax'

const rule = createRule<[Options], 'default'>({
  name,
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Disallow specific syntax patterns',
    },
    schema: [optionSchema as any],
    messages: {
      default: '{{message}}',
    },
    fixable: 'code',
    hasSuggestions: true,
  },
  defaultOptions: [{ disallow: [] }],
  create(context) {
    const result: Record<
      string,
      (node: TSESTree.Node | TSESTree.Token) => void
    > = {}

    const options = context.options[0]

    const mustCallFn = options.mustCallFn

    if (mustCallFn) {
      result['CallExpression'] = (node) => {
        if (node.type !== AST_NODE_TYPES.CallExpression) return

        const { callee } = node

        if (callee.type !== AST_NODE_TYPES.Identifier) return

        const calleeName = callee.name

        const callFn = mustCallFn[calleeName]

        if (!callFn) return

        const fileName = options.__dev_simulateFileName ?? context.filename

        let replaceVars: { name: string; value: string }[] | null = null

        if (callFn.getFileNameVarsRegex) {
          const fileNameVars = getFileNameVarsRegex(
            fileName,
            new RegExp(callFn.getFileNameVarsRegex),
          )

          if (!fileNameVars) return

          replaceVars = fileNameVars
        }

        function replaceStringWithVars(str: string) {
          if (!replaceVars) return str

          let newStr = str
          for (const { name, value } of replaceVars) {
            newStr = newStr.replaceAll(name, value)
          }
          return newStr
        }

        for (const arg of callFn.args) {
          const calledArg = node.arguments[arg.pos]

          if (!calledArg) {
            context.report({
              node,
              messageId: 'default',
              data: {
                message: `Missing required argument at position ${
                  arg.pos
                }: ${replaceStringWithVars(callFn.message)}`,
              },
            })
            continue
          }

          if (calledArg.type !== AST_NODE_TYPES.Literal) {
            context.report({
              node: calledArg,
              messageId: 'default',
              data: {
                message: `Argument at position ${
                  arg.pos
                } should be a literal: ${replaceStringWithVars(
                  callFn.message,
                )}`,
              },
            })
            continue
          }

          const normalizedValue =
            typeof arg.literal === 'string'
              ? replaceStringWithVars(arg.literal)
              : arg.literal

          if (calledArg.value !== normalizedValue) {
            context.report({
              node: calledArg,
              messageId: 'default',
              data: {
                message: `Invalid argument value: ${replaceStringWithVars(
                  callFn.message,
                )}`,
              },
            })
          }
        }
      }
    }

    for (const {
      selector,
      message,
      replace,
      replaceType = 'suggestion',
    } of context.options[0].disallow ?? []) {
      result[selector] = (node) => {
        const fixFn: ReportFixFunction = (fixer) => {
          if (!replace) return null

          if (typeof replace === 'string') {
            return fixer.replaceText(node, replace)
          } else {
            const replaceRegex = new RegExp(replace.regex)

            const nodeText = context.sourceCode.getText(node)

            return fixer.replaceText(
              node,
              nodeText.replace(replaceRegex, replace.with),
            )
          }
        }

        context.report({
          node,
          messageId: 'default',
          data: { message },
          fix: replace && replaceType === 'autofix' ? fixFn : undefined,
          suggest:
            replace && replaceType === 'suggestion'
              ? [
                  {
                    messageId: 'default',
                    data: {
                      message: `Replace with "${
                        typeof replace === 'string' ? replace : replace.with
                      }"`,
                    },
                    fix: fixFn,
                  },
                ]
              : undefined,
        })
      }
    }

    return result
  },
})

function getFileNameVarsRegex(
  fileName: string,
  fileNameRegex: RegExp,
): { name: string; value: string }[] | null {
  const vars: { name: string; value: string }[] = []

  const match = fileNameRegex.exec(fileName)

  if (!match) return null

  const [fullMatch, ...groups] = match

  vars.push({ name: '$0_lowercase', value: fullMatch.toLowerCase() })
  vars.push({ name: '$0_capitalize', value: capitalize(fullMatch) })
  vars.push({ name: '$0_uncapitalize', value: uncapitalize(fullMatch) })
  vars.push({ name: '$0', value: fullMatch })

  for (let i = 0; i < groups.length; i++) {
    const name = `$${i + 1}`
    const value = groups[i]!
    vars.push({ name: `${name}_lowercase`, value: value.toLowerCase() })
    vars.push({ name: `${name}_capitalize`, value: capitalize(value) })
    vars.push({ name: `${name}_uncapitalize`, value: uncapitalize(value) })
    vars.push({ name: name, value: value })
  }

  return vars
}

function capitalize(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1)
}

function uncapitalize(str: string) {
  return str.charAt(0).toLowerCase() + str.slice(1)
}

export const advancedNoRestrictedSyntax = {
  name,
  rule,
}
