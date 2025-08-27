import { AST_NODE_TYPES, ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { getJsonSchemaFromZod } from '../createRule'

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

/**
 * Checks if a type annotation represents React.FC or FC
 */
function isFCType(typeAnnotation: TSESTree.TSTypeAnnotation): boolean {
  const typeNode = typeAnnotation.typeAnnotation

  if (typeNode.type === AST_NODE_TYPES.TSTypeReference) {
    if (typeNode.typeName.type === AST_NODE_TYPES.Identifier) {
      return typeNode.typeName.name === 'FC'
    }

    if (
      typeNode.typeName.type === AST_NODE_TYPES.TSQualifiedName &&
       
      typeNode.typeName.left.type === AST_NODE_TYPES.Identifier &&
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- type guard needed for TS
      typeNode.typeName.right.type === AST_NODE_TYPES.Identifier
    ) {
      return (
        typeNode.typeName.left.name === 'React' &&
        typeNode.typeName.right.name === 'FC'
      )
    }
  }

  return false
}

/**
 * Checks if an expression ultimately returns JSX
 */
function returnsJSX(node: TSESTree.Expression | null | undefined): boolean {
  if (!node) return false

  switch (node.type) {
    case AST_NODE_TYPES.JSXElement:
    case AST_NODE_TYPES.JSXFragment:
      return true

    case AST_NODE_TYPES.ConditionalExpression:
      return returnsJSX(node.consequent) || returnsJSX(node.alternate)

    case AST_NODE_TYPES.LogicalExpression:
      if (node.operator === '&&') {
        return returnsJSX(node.right)
      }
      if (node.operator === '||') {
        return returnsJSX(node.left) || returnsJSX(node.right)
      }
      return false

    case AST_NODE_TYPES.CallExpression:
      // Check for React.createElement or jsx() calls
      if (
        node.callee.type === AST_NODE_TYPES.MemberExpression &&
        node.callee.object.type === AST_NODE_TYPES.Identifier &&
        node.callee.object.name === 'React' &&
        node.callee.property.type === AST_NODE_TYPES.Identifier &&
        node.callee.property.name === 'createElement'
      ) {
        return true
      }
      return false

    default:
      return false
  }
}

/**
 * Recursively find all return statements in a block
 */
function findAllReturnStatements(
  node: TSESTree.Node,
  returns: TSESTree.ReturnStatement[] = [],
): TSESTree.ReturnStatement[] {
  if (node.type === AST_NODE_TYPES.ReturnStatement) {
    returns.push(node)
    return returns
  }

  // Handle different statement types that can contain returns
  if (node.type === AST_NODE_TYPES.BlockStatement) {
    for (const statement of node.body) {
      findAllReturnStatements(statement, returns)
    }
  } else if (node.type === AST_NODE_TYPES.IfStatement) {
    findAllReturnStatements(node.consequent, returns)
    if (node.alternate) {
      findAllReturnStatements(node.alternate, returns)
    }
  } else if (node.type === AST_NODE_TYPES.SwitchStatement) {
    for (const switchCase of node.cases) {
      for (const statement of switchCase.consequent) {
        findAllReturnStatements(statement, returns)
      }
    }
  } else if (node.type === AST_NODE_TYPES.TryStatement) {
    findAllReturnStatements(node.block, returns)
    if (node.handler) {
      findAllReturnStatements(node.handler.body, returns)
    }
    if (node.finalizer) {
      findAllReturnStatements(node.finalizer, returns)
    }
  }

  return returns
}

/**
 * Checks if a function body returns JSX in at least one return statement
 */
function functionReturnsJSX(
  body: TSESTree.BlockStatement | TSESTree.Expression,
): boolean {
  if (body.type !== AST_NODE_TYPES.BlockStatement) {
    // Arrow function with expression body
    return returnsJSX(body)
  }

  // Find all return statements recursively
  const allReturns = findAllReturnStatements(body)
  
  if (allReturns.length === 0) {
    return false // No return statements
  }

  // Check if at least one return statement returns JSX
  for (const returnStmt of allReturns) {
    if (returnStmt.argument && returnsJSX(returnStmt.argument)) {
      return true
    }
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
  | 'fcComponentShouldReturnJsx'
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
      fcComponentShouldReturnJsx:
        'React.FC components should return JSX elements for optimal React compiler detection. Consider wrapping the return value in a fragment.',
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
     * but doesn't report or adjust regular nested methods
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

      VariableDeclarator(node) {
        // Check for FC component declarations like: const Component: React.FC = ...
        if (
          node.id.type === AST_NODE_TYPES.Identifier &&
          node.id.typeAnnotation &&
          isFCType(node.id.typeAnnotation) &&
          node.init
        ) {
          let functionNode: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression | null = null

          if (
            node.init.type === AST_NODE_TYPES.ArrowFunctionExpression ||
            node.init.type === AST_NODE_TYPES.FunctionExpression
          ) {
            functionNode = node.init
          }

          if (functionNode && !functionReturnsJSX(functionNode.body)) {
            context.report({
              node: functionNode,
              messageId: 'fcComponentShouldReturnJsx',
              suggest: [
                {
                  messageId: 'replaceWithFunctionExpression',
                  fix(fixer) {
                    const sourceCode = context.sourceCode

                    if (functionNode.body.type === AST_NODE_TYPES.BlockStatement) {
                      // Find all return statements and wrap their arguments
                      const fixes = []

                      for (const statement of functionNode.body.body) {
                        if (
                          statement.type === AST_NODE_TYPES.ReturnStatement &&
                          statement.argument
                        ) {
                          const returnText = sourceCode.getText(statement.argument)
                          fixes.push(
                            fixer.replaceText(
                              statement.argument,
                              `<>{${returnText}}</>`
                            )
                          )
                        }
                      }

                      return fixes
                    } else {
                      // Arrow function with expression body
                      const bodyText = sourceCode.getText(functionNode.body)
                      return fixer.replaceText(
                        functionNode.body,
                        `<>{${bodyText}}</>`
                      )
                    }
                  },
                },
              ],
            })
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
