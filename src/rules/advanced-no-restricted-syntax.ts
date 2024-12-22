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
  mustMatchSyntax: t.optional(
    t.array(
      t.object({
        includeRegex: t.string(),
        mustCallFn: t.array(
          t.object({
            anyCall: t.array(
              t.object({
                fn: t.string(),
                withArgs: t.array(
                  t.object({
                    atIndex: t.integer(),
                    literal: t.any(t.string(), t.number(), t.boolean()),
                  }),
                ),
              }),
            ),
            message: t.optional(t.string()),
          }),
        ),
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
  create(context, [options]) {
    const result: Record<
      string,
      (node: TSESTree.Node | TSESTree.Token) => void
    > = {}

    const { mustMatchSyntax, __dev_simulateFileName, disallow } = options

    const fileName = __dev_simulateFileName ?? context.filename

    const callExpressionSelectors: ((node: TSESTree.CallExpression) => void)[] =
      []

    const mustMatchSomeCallRemaining = new Set<string>()

    for (const { includeRegex, mustCallFn } of mustMatchSyntax ?? []) {
      const fileNameVars = getFileNameVarsRegex(
        fileName,
        new RegExp(includeRegex),
      )

      if (!fileNameVars) continue

      const replaceStringWithVars = (str: string) => {
        let newStr = str
        for (const { name, value } of fileNameVars) {
          newStr = newStr.replaceAll(name, value)
        }
        return newStr
      }

      for (const { anyCall, message } of mustCallFn) {
        const mustCallMessage = `Expected file to call the function: ${anyCall
          .map(({ fn }) => fn)
          .join(' or ')}`
        mustMatchSomeCallRemaining.add(mustCallMessage)

        callExpressionSelectors.push((node) => {
          const { callee } = node

          if (callee.type !== AST_NODE_TYPES.Identifier) return

          for (const { fn, withArgs } of anyCall) {
            if (callee.name !== fn) continue

            mustMatchSomeCallRemaining.delete(mustCallMessage)

            for (const arg of withArgs) {
              const calledArg = node.arguments[arg.atIndex]

              const normalizedValue =
                typeof arg.literal === 'string'
                  ? replaceStringWithVars(arg.literal)
                  : arg.literal

              if (!calledArg) {
                context.report({
                  node,
                  messageId: 'default',
                  data: {
                    message: `Missing argument with value "${normalizedValue}" at index ${
                      arg.atIndex
                    }${message ? `: ${replaceStringWithVars(message)}` : ''}`,
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
                      arg.atIndex
                    } should the literal "${normalizedValue}"${
                      message ? `: ${replaceStringWithVars(message)}` : ''
                    }`,
                  },
                })
                continue
              }

              if (calledArg.value !== normalizedValue) {
                context.report({
                  node: calledArg,
                  messageId: 'default',
                  data: {
                    message: `Argument should have the value "${normalizedValue}"${
                      message ? `: ${replaceStringWithVars(message)}` : ''
                    }`,
                  },
                  fix: (fixer) => {
                    return fixer.replaceText(
                      calledArg,
                      typeof normalizedValue === 'string'
                        ? `'${normalizedValue}'`
                        : String(normalizedValue),
                    )
                  },
                })
              }
            }
            break
          }
        })
      }
    }

    for (const {
      selector,
      message,
      replace,
      replaceType = 'suggestion',
    } of disallow ?? []) {
      if (selector === 'CallExpression') {
        callExpressionSelectors.push((node) => {
          reportInvalidNode(replace, node, message, replaceType)
        })

        continue
      }

      result[selector] = (node) => {
        reportInvalidNode(replace, node, message, replaceType)
      }
    }

    if (callExpressionSelectors.length > 0) {
      result['CallExpression'] = (node) => {
        if (node.type !== AST_NODE_TYPES.CallExpression) return

        for (const checkCallFn of callExpressionSelectors) {
          checkCallFn(node)
        }
      }
    }

    result['Program:exit'] = (node) => {
      for (const message of mustMatchSomeCallRemaining) {
        context.report({
          node,
          messageId: 'default',
          data: { message: `${message}` },
        })
      }
    }

    return result

    function reportInvalidNode(
      replace: string | { regex: string; with: string } | undefined,
      node: TSESTree.Node | TSESTree.Token,
      message: string,
      replaceType: 'suggestion' | 'autofix',
    ) {
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
