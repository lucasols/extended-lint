import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { getJsonSchemaFromZod } from './createRule'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'react-compiler-extra'

const optionsSchema = z.object({
  runOnlyWithEnableCompilerDirective: z.boolean().optional(),
})

const hasEnableCompilerDirectiveRegex =
  /eslint +react-compiler\/react-compiler: +\["error/

/**
 * Checks if a callee is a hook (starts with "use")
 */
function isHook(callee: TSESTree.CallExpression['callee']): boolean {
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name.startsWith('use')
  }

  if (
    callee.type === AST_NODE_TYPES.MemberExpression &&
    callee.property.type === AST_NODE_TYPES.Identifier
  ) {
    return callee.property.name.startsWith('use')
  }

  return false
}

const hasThisRegex = /\bthis[.[]/

/**
 * Checks if a function body uses the 'this' keyword using regex
 */
function containsThisKeyword(sourceCode: string): boolean {
  // Simple regex to match 'this' keyword
  return hasThisRegex.test(sourceCode)
}

type Options = z.infer<typeof optionsSchema>

const rule = createRule<
  [Options],
  | 'objectMethodIsNotSupported'
  | 'replaceWithFunctionExpression'
  | 'thisKeywordInMethod'
>({
  name,
  meta: {
    type: 'suggestion',
    fixable: 'code',
    docs: {
      description:
        'Extra rules to enforce best practices when using the React compiler',
    },
    messages: {
      objectMethodIsNotSupported:
        'Object methods such as `{ method() {} }` have limited support to optimizations in the React compiler, use a function expression `{ method: () => {} }` instead.',
      replaceWithFunctionExpression: 'Replace with function expression',
      thisKeywordInMethod:
        'Object method uses `this` keyword which would have different behavior if converted to an arrow function. Fix this manually.',
    },
    hasSuggestions: true,
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    let isEnabled = false

    if (options.runOnlyWithEnableCompilerDirective) {
      for (const comment of context.sourceCode.getAllComments()) {
        if (hasEnableCompilerDirectiveRegex.test(comment.value)) {
          isEnabled = true
          break
        }
      }

      if (!isEnabled) return {}
    }

    /**
     * Checks only the root-level properties of an object expression for object methods
     */
    function checkForObjectMethods(node: TSESTree.ObjectExpression) {
      for (const property of node.properties) {
        if (property.type === AST_NODE_TYPES.Property) {
          // Check if the property is a method
          if (
            property.method &&
            property.value.type === AST_NODE_TYPES.FunctionExpression
          ) {
            const sourceCode = context.sourceCode

            // Get method name
            const methodName = sourceCode.getText(property.key)

            // Extract function expression
            const functionExpr = property.value

            // Get the source code of the function body
            const bodyText = sourceCode.getText(functionExpr.body)

            // Check if the function body contains 'this' keyword using regex
            if (containsThisKeyword(bodyText)) {
              // Report an error without autofix if 'this' is used
              context.report({
                node: property,
                messageId: 'thisKeywordInMethod',
              })
            } else {
              // Create replacement text for methods that don't use 'this'
              const paramsList = functionExpr.params
                .map((param) => sourceCode.getText(param))
                .join(', ')

              // Create replacement text
              let newText = ''
              if (functionExpr.generator) {
                newText = `${methodName}: function* (${paramsList}) ${bodyText}`
              } else {
                newText = `${methodName}: (${paramsList}) => ${bodyText}`
              }

              context.report({
                node: property,
                messageId: 'objectMethodIsNotSupported',
                fix: (fixer) => {
                  return fixer.replaceText(property, newText)
                },
              })
            }
          }
        }
      }
    }

    /**
     * Checks for 'this' usage in nested object methods,
     * but doesn't report or fix regular nested methods
     */
    function checkNestedThisUsage(node: TSESTree.ObjectExpression) {
      for (const property of node.properties) {
        if (
          property.type === AST_NODE_TYPES.Property &&
          property.value.type === AST_NODE_TYPES.ObjectExpression
        ) {
          // If nested object, check its properties for 'this' usage
          for (const nestedProp of property.value.properties) {
            if (
              nestedProp.type === AST_NODE_TYPES.Property &&
              nestedProp.method &&
              nestedProp.value.type === AST_NODE_TYPES.FunctionExpression
            ) {
              const sourceCode = context.sourceCode
              const bodyText = sourceCode.getText(nestedProp.value.body)

              // Only report 'this' usage in nested methods
              if (containsThisKeyword(bodyText)) {
                context.report({
                  node: nestedProp,
                  messageId: 'thisKeywordInMethod',
                })
              }
            }
          }
        }
      }
    }

    return {
      CallExpression(node) {
        if (!isHook(node.callee)) return

        // Check direct arguments that are object expressions
        for (const arg of node.arguments) {
          if (arg.type === AST_NODE_TYPES.ObjectExpression) {
            checkForObjectMethods(arg)
            checkNestedThisUsage(arg)
          }

          // Also check for object expressions returned from arrow functions or functions
          if (arg.type === AST_NODE_TYPES.ArrowFunctionExpression) {
            // If the body is an object expression, check it
            if (arg.body.type === AST_NODE_TYPES.ObjectExpression) {
              checkForObjectMethods(arg.body)
              checkNestedThisUsage(arg.body)
            }

            // If the body is a block statement, look for return statements with object expressions
            if (arg.body.type === AST_NODE_TYPES.BlockStatement) {
              for (const statement of arg.body.body) {
                if (
                  statement.type === AST_NODE_TYPES.ReturnStatement &&
                  statement.argument?.type === AST_NODE_TYPES.ObjectExpression
                ) {
                  checkForObjectMethods(statement.argument)
                  checkNestedThisUsage(statement.argument)
                }
              }
            }
          }

          // Check function expressions
          if (arg.type === AST_NODE_TYPES.FunctionExpression) {
            for (const statement of arg.body.body) {
              if (
                statement.type === AST_NODE_TYPES.ReturnStatement &&
                statement.argument?.type === AST_NODE_TYPES.ObjectExpression
              ) {
                checkForObjectMethods(statement.argument)
                checkNestedThisUsage(statement.argument)
              }
            }
          }
        }
      },
    }
  },
})

export const reactCompilerExtra = {
  name,
  rule,
}
