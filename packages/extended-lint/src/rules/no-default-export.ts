import { ESLintUtils, TSESTree } from '@typescript-eslint/utils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-default-export'

const rule = createRule({
  name,
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow default exports',
    },
    schema: [],
    messages: {
      noDefaultExport:
        'Default exports are not allowed, use named exports instead.',
    },
  },
  defaultOptions: [],
  create(context) {
    return {
      ExportDefaultDeclaration(node) {
        context.report({ node, messageId: 'noDefaultExport' })
      },
      ExportNamedDeclaration(node) {
        for (const specifier of node.specifiers) {
          if (
            specifier.exported.type === TSESTree.AST_NODE_TYPES.Identifier &&
            specifier.exported.name === 'default'
          ) {
            context.report({ node, messageId: 'noDefaultExport' })
          }
        }
      },
    }
  },
})

export const noDefaultExport = {
  name,
  rule,
}
