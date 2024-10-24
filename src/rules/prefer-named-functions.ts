import { ESLintUtils } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'prefer-named-functions'

type Options = [
  {
    ignoreRegex?: string
  },
]

const regexpsCache = new Map<string, RegExp>()

const rule = createRule<Options, 'default' | 'withIgnoreRegex'>({
  name,
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Prevent using arrow functions when a named function can be used instead',
    },
    schema: [
      {
        type: 'object',
        properties: {
          ignoreRegex: {
            type: 'string',
          },
        },
      },
    ],
    messages: {
      default:
        'Function {{functionName}} should be defined as a named function "function {{functionName}} () {}" instead of an arrow function',
      withIgnoreRegex:
        'Function {{functionName}} should be defined as a named function "function {{functionName}} () {}" or have a name that matches the regex "{{ignoreRegex}}"',
    },
  },
  defaultOptions: [{ ignoreRegex: undefined }],
  create(context) {
    const options = context.options[0] || {}

    let ignoreRegex: RegExp | null = null

    if (options.ignoreRegex) {
      const cachedRegex = regexpsCache.get(options.ignoreRegex)
      if (cachedRegex) {
        ignoreRegex = cachedRegex
      } else {
        ignoreRegex = new RegExp(options.ignoreRegex)
        regexpsCache.set(options.ignoreRegex, ignoreRegex)
      }
    }

    return {
      VariableDeclarator(node) {
        if (
          node.init &&
          node.init.type === 'ArrowFunctionExpression' &&
          node.id.type === 'Identifier'
        ) {
          const functionName = node.id.name

          // Skip if the variable has a type annotation
          if (node.id.typeAnnotation) {
            return
          }

          // Check if the function name matches the ignore regex
          if (ignoreRegex && ignoreRegex.test(functionName)) {
            return
          }

          context.report({
            node,
            messageId: ignoreRegex ? 'withIgnoreRegex' : 'default',
            data: {
              functionName,
              ignoreRegex: options.ignoreRegex,
            },
          })
        }
      },
    }
  },
})

export const preferNamedFunction = {
  name,
  rule,
}
