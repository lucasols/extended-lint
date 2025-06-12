import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { ReportFixFunction } from '@typescript-eslint/utils/ts-eslint'
import * as t from 'tschema'
import { createExtendedLintRule } from '../createRule'

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
  disallowFnCalls: t.optional(
    t.array(
      t.object({
        fn: t.string(),
        withArgs: t.optional(
          t.array(
            t.object({
              atIndex: t.integer(),
              value: t.any(t.string(), t.number(), t.boolean()),
            }),
          ),
        ),
        message: t.string(),
        replaceWith: t.optional(t.string()),
        ignoreRegex: t.optional(t.string()),
      }),
    ),
  ),
  __dev_simulateFileName: t.optional(t.string()),
  mustMatchSyntax: t.optional(
    t.array(
      t.object({
        includeRegex: t.string(),
        mustCallFn: t.optional(
          t.array(
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
        ),
        mustMatchSelector: t.optional(
          t.array(
            t.object({
              selector: t.string(),
              message: t.string(),
            }),
          ),
        ),
      }),
    ),
  ),
})

export type Options = t.Infer<typeof optionSchema>

export const advancedNoRestrictedSyntax = createExtendedLintRule<
  [Options],
  'default'
>({
  name: 'advanced-no-restricted-syntax',
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

    const {
      mustMatchSyntax,
      __dev_simulateFileName,
      disallow,
      disallowFnCalls,
    } = options

    const fileName = __dev_simulateFileName ?? context.filename

    const callExpressionSelectors: ((node: TSESTree.CallExpression) => void)[] =
      []
    const mustMatchSelectorSelectors: Map<string, () => void> = new Map()

    const mustMatchSomeCallRemaining = new Set<string>()
    const mustMatchSomeSelectorRemaining = new Map<string, string>()

    const functionAliases = new Map<string, string>()

    function trackImportAliases(node: TSESTree.ImportDeclaration) {
      for (const specifier of node.specifiers) {
        if (specifier.type === AST_NODE_TYPES.ImportSpecifier) {
          if (
            specifier.imported.type === AST_NODE_TYPES.Identifier &&
            specifier.local.type === AST_NODE_TYPES.Identifier &&
            specifier.imported.name !== specifier.local.name
          ) {
            functionAliases.set(specifier.local.name, specifier.imported.name)
          }
        }
      }
    }

    function trackVariableAliases(node: TSESTree.VariableDeclarator) {
      if (
        node.id.type === AST_NODE_TYPES.Identifier &&
        node.init?.type === AST_NODE_TYPES.Identifier
      ) {
        functionAliases.set(node.id.name, node.init.name)
      }
    }

    function getOriginalFunctionName(calleeName: string): string {
      return functionAliases.get(calleeName) ?? calleeName
    }

    function matchesFunctionName(
      calleeName: string,
      targetFn: string,
    ): boolean {
      const originalName = getOriginalFunctionName(calleeName)
      return originalName === targetFn || calleeName === targetFn
    }

    for (const {
      includeRegex,
      mustCallFn,
      mustMatchSelector,
    } of mustMatchSyntax ?? []) {
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

      for (const { anyCall, message } of mustCallFn ?? []) {
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

      for (const { selector, message } of mustMatchSelector ?? []) {
        mustMatchSomeSelectorRemaining.set(
          selector,
          replaceStringWithVars(message),
        )

        mustMatchSelectorSelectors.set(replaceStringWithVars(selector), () => {
          mustMatchSomeSelectorRemaining.delete(selector)
        })
      }
    }

    for (const {
      fn,
      withArgs,
      message,
      replaceWith,
      ignoreRegex,
    } of disallowFnCalls ?? []) {
      if (ignoreRegex && new RegExp(ignoreRegex).test(fileName)) {
        continue
      }

      callExpressionSelectors.push((node) => {
        const { callee } = node

        if (callee.type !== AST_NODE_TYPES.Identifier) return

        if (!matchesFunctionName(callee.name, fn)) return

        if (withArgs) {
          for (const arg of withArgs) {
            const calledArg = node.arguments[arg.atIndex]

            if (!calledArg) {
              context.report({
                node,
                messageId: 'default',
                data: {
                  message: `Missing argument with value "${arg.value}" at index ${arg.atIndex}: ${message}`,
                },
              })
              return
            }

            if (calledArg.type !== AST_NODE_TYPES.Literal) {
              return
            }

            if (calledArg.value !== arg.value) {
              return
            }
          }
        }

        const fixFn: ReportFixFunction = (fixer) => {
          if (!replaceWith) return null
          return fixer.replaceText(node, replaceWith)
        }

        context.report({
          node,
          messageId: 'default',
          data: { message },
          suggest: replaceWith
            ? [
                {
                  messageId: 'default',
                  data: {
                    message: `Replace with "${replaceWith}"`,
                  },
                  fix: fixFn,
                },
              ]
            : undefined,
        })
      })
    }

    function addSelector(
      selector: string,
      checkSelectorFn: (selectorNode: TSESTree.Node | TSESTree.Token) => void,
    ) {
      const existingSelector = result[selector]

      if (existingSelector) {
        result[selector] = (node) => {
          existingSelector(node)
          checkSelectorFn(node)
        }
      } else {
        result[selector] = checkSelectorFn
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

    if (mustMatchSelectorSelectors.size > 0) {
      for (const [selector, checkSelectorFn] of mustMatchSelectorSelectors) {
        addSelector(selector, checkSelectorFn)
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

    result['ImportDeclaration'] = (node) => {
      if (node.type !== AST_NODE_TYPES.ImportDeclaration) return
      trackImportAliases(node)
    }

    result['VariableDeclarator'] = (node) => {
      if (node.type !== AST_NODE_TYPES.VariableDeclarator) return
      trackVariableAliases(node)
    }

    result['Program:exit'] = (node) => {
      for (const message of mustMatchSomeCallRemaining) {
        context.report({
          node,
          messageId: 'default',
          data: { message },
        })
      }

      for (const [, message] of mustMatchSomeSelectorRemaining) {
        context.report({
          node,
          messageId: 'default',
          data: { message },
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
    vars.push({ name, value })
  }

  return vars
}

function capitalize(str: string) {
  return str.charAt(0).toUpperCase() + str.slice(1)
}

function uncapitalize(str: string) {
  return str.charAt(0).toLowerCase() + str.slice(1)
}
