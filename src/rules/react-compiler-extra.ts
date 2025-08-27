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

const PASCAL_CASE_REGEX = /^[A-Z][a-zA-Z0-9]*$/

/**
 * Checks if a function name suggests it's a React component (PascalCase)
 */
function isPascalCase(functionName: string): boolean {
  return PASCAL_CASE_REGEX.test(functionName)
}

/**
 * Checks if a function name suggests it's a React hook (starts with "use")
 */
function isHookName(functionName: string): boolean {
  return functionName.startsWith('use') && functionName.length > 3
}

/**
 * Checks if a function is potentially a React component or hook based on name or type
 */
function isReactComponentOrHook(
  functionNode: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression | TSESTree.FunctionDeclaration,
  identifier?: TSESTree.Identifier,
  typeAnnotation?: TSESTree.TSTypeAnnotation,
): boolean {
  // Check FC type annotation - only check PascalCase functions if they have FC type
  if (typeAnnotation && isFCType(typeAnnotation)) {
    // For FC typed functions, also require PascalCase name
    if (identifier) {
      return isPascalCase(identifier.name)
    }
    // For function declarations with FC type, check the function's own id
    if (functionNode.type === AST_NODE_TYPES.FunctionDeclaration && functionNode.id) {
      return isPascalCase(functionNode.id.name)
    }
    return true // FC type without name restriction
  }

  // Check function name for hooks (always check hook functions regardless of type)
  if (identifier && isHookName(identifier.name)) {
    return true
  }

  // For function declarations, check if it's a hook
  if (functionNode.type === AST_NODE_TYPES.FunctionDeclaration && functionNode.id && isHookName(functionNode.id.name)) {
    return true
  }

  // Don't check PascalCase functions without FC type annotation
  return false
}

/**
 * Checks if an expression creates JSX
 */
function createsJSX(node: TSESTree.Expression | null | undefined): boolean {
  if (!node) return false

  switch (node.type) {
    case AST_NODE_TYPES.JSXElement:
    case AST_NODE_TYPES.JSXFragment:
      return true

    case AST_NODE_TYPES.ConditionalExpression:
      return createsJSX(node.consequent) || createsJSX(node.alternate)

    case AST_NODE_TYPES.LogicalExpression:
      if (node.operator === '&&') {
        return createsJSX(node.right)
      }
      if (node.operator === '||') {
        return createsJSX(node.left) || createsJSX(node.right)
      }
      return false

    case AST_NODE_TYPES.CallExpression:
      // Check for React.createElement calls
      if (
        node.callee.type === AST_NODE_TYPES.MemberExpression &&
        node.callee.object.type === AST_NODE_TYPES.Identifier &&
        node.callee.object.name === 'React' &&
        node.callee.property.type === AST_NODE_TYPES.Identifier &&
        node.callee.property.name === 'createElement'
      ) {
        return true
      }
      
      // Check arguments of function calls for JSX content
      for (const arg of node.arguments) {
        if (arg.type !== AST_NODE_TYPES.SpreadElement && createsJSX(arg)) {
          return true
        }
      }
      
      return false

    default:
      return false
  }
}

/**
 * Checks if a function body contains JSX creation anywhere
 */
function containsJSXCreation(node: TSESTree.Node, visited = new Set<TSESTree.Node>()): boolean {
  if (visited.has(node)) return false
  visited.add(node)

  // Check if current node is JSX
  if (node.type === AST_NODE_TYPES.JSXElement || node.type === AST_NODE_TYPES.JSXFragment) {
    return true
  }

  // Check return statements
  if (node.type === AST_NODE_TYPES.ReturnStatement && node.argument) {
    if (createsJSX(node.argument)) return true
  }

  // Check variable declarations for JSX assignments
  if (node.type === AST_NODE_TYPES.VariableDeclaration) {
    for (const declarator of node.declarations) {
      if (declarator.init && createsJSX(declarator.init)) return true
    }
  }

  // Check assignment expressions
  if (node.type === AST_NODE_TYPES.AssignmentExpression) {
    if (createsJSX(node.right)) return true
  }

  // Check call expressions for React.createElement
  if (node.type === AST_NODE_TYPES.CallExpression) {
    if (createsJSX(node)) return true
  }

  // Check block statements
  if (node.type === AST_NODE_TYPES.BlockStatement) {
    for (const statement of node.body) {
      if (containsJSXCreation(statement, visited)) return true
    }
  }

  // Check if statements
  if (node.type === AST_NODE_TYPES.IfStatement) {
    if (containsJSXCreation(node.consequent, visited)) return true
    if (node.alternate && containsJSXCreation(node.alternate, visited)) return true
  }

  // Check expression statements
  if (node.type === AST_NODE_TYPES.ExpressionStatement) {
    if (createsJSX(node.expression)) return true
  }

  return false
}

/**
 * Checks if a function calls any React hooks
 */
function callsHooks(functionBody: TSESTree.BlockStatement | TSESTree.Expression): boolean {
  if (functionBody.type !== AST_NODE_TYPES.BlockStatement) {
    // Arrow function with expression body - check if it's a hook call
    if (functionBody.type === AST_NODE_TYPES.CallExpression) {
      return isHook(functionBody.callee)
    }
    return false
  }

  return containsHookCalls(functionBody)
}

/**
 * Checks if a function body contains hook calls
 */
function containsHookCalls(node: TSESTree.Node, visited = new Set<TSESTree.Node>()): boolean {
  if (visited.has(node)) return false
  visited.add(node)

  // Check if current node is a hook call
  if (node.type === AST_NODE_TYPES.CallExpression) {
    if (isHook(node.callee)) return true
  }

  // Check block statements
  if (node.type === AST_NODE_TYPES.BlockStatement) {
    for (const statement of node.body) {
      if (containsHookCalls(statement, visited)) return true
    }
  }

  // Check variable declarations for hook calls in initializers
  if (node.type === AST_NODE_TYPES.VariableDeclaration) {
    for (const declarator of node.declarations) {
      if (declarator.init && declarator.init.type === AST_NODE_TYPES.CallExpression) {
        if (isHook(declarator.init.callee)) return true
      }
    }
  }

  // Check expression statements
  if (node.type === AST_NODE_TYPES.ExpressionStatement) {
    if (node.expression.type === AST_NODE_TYPES.CallExpression) {
      if (isHook(node.expression.callee)) return true
    }
  }

  // Check if statements
  if (node.type === AST_NODE_TYPES.IfStatement) {
    if (containsHookCalls(node.consequent, visited)) return true
    if (node.alternate && containsHookCalls(node.alternate, visited)) return true
  }

  // Check return statements
  if (node.type === AST_NODE_TYPES.ReturnStatement && node.argument) {
    if (node.argument.type === AST_NODE_TYPES.CallExpression && isHook(node.argument.callee)) {
      return true
    }
  }

  return false
}


/**
 * Checks if a function behaves like a React component or hook according to compiler heuristics
 */
function behavesLikeReactComponentOrHook(
  functionNode: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression | TSESTree.FunctionDeclaration,
): boolean {
  const body = functionNode.body

  // For expression body (arrow functions), check directly
  if (body.type !== AST_NODE_TYPES.BlockStatement) {
    // Check if expression body creates JSX or is a hook call
    return createsJSX(body) || (body.type === AST_NODE_TYPES.CallExpression && isHook(body.callee))
  }

  // Check if function creates JSX anywhere in the body
  const createsJSXAnywhere = containsJSXCreation(body)

  // Check if function calls any hooks
  const callsAnyHooks = callsHooks(body)

  // According to React Compiler heuristics: function is component/hook if it creates JSX and/or calls hooks
  return createsJSXAnywhere || callsAnyHooks
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
        'React components and hooks should create JSX elements or call other hooks for optimal React compiler detection.',
    },
    hasSuggestions: false,
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
        // Check for potential React component/hook variable declarations
        if (
          node.id.type === AST_NODE_TYPES.Identifier &&
          node.init &&
          (
            node.init.type === AST_NODE_TYPES.ArrowFunctionExpression ||
            node.init.type === AST_NODE_TYPES.FunctionExpression
          )
        ) {
          const functionNode = node.init
          const identifier = node.id
          const typeAnnotation = node.id.typeAnnotation

          // Check if this looks like a React component or hook
          if (isReactComponentOrHook(functionNode, identifier, typeAnnotation)) {
            // Check if it behaves like a component/hook (creates JSX or calls hooks)
            if (!behavesLikeReactComponentOrHook(functionNode)) {
              context.report({
                node: functionNode,
                messageId: 'fcComponentShouldReturnJsx',
              })
            }
          }
        }
      },

      FunctionDeclaration(node) {
        // Check function declarations that might be React components or hooks
        if (isReactComponentOrHook(node)) {
          // Check if it behaves like a component/hook (creates JSX or calls hooks)
          if (!behavesLikeReactComponentOrHook(node)) {
            context.report({
              node,
              messageId: 'fcComponentShouldReturnJsx',
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
