import { ESLintUtils } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-non-camel-case-functions'

const camelCaseRegex = /^[a-z][a-zA-Z0-9]*$/

const rule = createRule({
  name,
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Enforce camelCase naming convention for function declarations',
    },
    schema: [],
    messages: {
      nonCamelCaseFunction:
        'Function name "{{functionName}}" should be in camelCase format',
    },
  },
  defaultOptions: [],
  create(context) {
    return {
      FunctionDeclaration(node) {
        if (node.id && !camelCaseRegex.test(node.id.name)) {
          context.report({
            node: node.id,
            messageId: 'nonCamelCaseFunction',
            data: {
              functionName: node.id.name,
            },
          })
        }
      },
    }
  },
})

export const noNonCamelCaseFunctions = {
  name,
  rule,
}
