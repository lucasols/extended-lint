import { AST_NODE_TYPES, ESLintUtils } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'prefer-named-functions'

const regexpsCache = new Map<string, RegExp>()

type Options = [
  {
    ignoreRegex?: string
    disallowArrowFnWithImplicitReturns?: boolean
  },
]

const rule = createRule<Options, 'default' | 'withIgnoreRegex' | 'suggestion'>({
  name,
  meta: {
    hasSuggestions: true,
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
          disallowArrowFnWithImplicitReturns: {
            type: 'boolean',
          },
        },
      },
    ],
    messages: {
      suggestion: 'Convert to named function',
      default:
        'Function {{functionName}} should be defined as a named function "function {{functionName}} () {}" instead of an arrow function',
      withIgnoreRegex:
        'Function {{functionName}} should be defined as a named function "function {{functionName}} () {}", if not possible to use `function` you can also use a name that matches the rule ignore regex',
    },
  },
  defaultOptions: [{}],
  create(context, [options]) {
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
          node.init.type === AST_NODE_TYPES.ArrowFunctionExpression &&
          node.id.type === AST_NODE_TYPES.Identifier
        ) {
          const functionName = node.id.name

          // Skip if the variable has a type annotation
          if (node.id.typeAnnotation) return

          // Check if the function name matches the ignore regex
          if (ignoreRegex && ignoreRegex.test(functionName)) {
            return
          }

          if (!options.disallowArrowFnWithImplicitReturns) {
            // Skip if the arrow function has an implicit return (no curly braces)
            if (node.init.body.type !== AST_NODE_TYPES.BlockStatement) {
              return
            }
          }

          const parent = node.parent

          const params = node.init.params
          const fnBody = node.init.body
          const arrowExpression = node.init

          context.report({
            node: node.id,
            messageId: ignoreRegex ? 'withIgnoreRegex' : 'default',
            data: {
              functionName,
              ignoreRegex: options.ignoreRegex,
            },
            suggest: [
              {
                messageId: 'suggestion',
                fix: (fixer) => {
                  return fixer.replaceText(
                    parent,
                    `${
                      arrowExpression.async ? 'async ' : ''
                    }function ${functionName}(${
                      params
                        .map((param) => context.sourceCode.getText(param))
                        .join(', ') || ''
                    }) ${context.sourceCode.getText(fnBody)}`,
                  )
                },
              },
            ],
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
