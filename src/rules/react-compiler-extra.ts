import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { traverseAST } from '../astUtils'
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

function hasUseNoMemoDirective(sourceCode: TSESLint.SourceCode): boolean {
  const firstStatement = sourceCode.ast.body[0]
  if (
    firstStatement?.type === AST_NODE_TYPES.ExpressionStatement &&
    firstStatement.expression.type === AST_NODE_TYPES.Literal &&
    firstStatement.expression.value === 'use no memo'
  ) {
    return true
  }
  return false
}

/**
 * Checks if a callee is a React hook (starts with "use")
 */
function isHook(callee: TSESTree.CallExpression['callee']): boolean {
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name.startsWith('use')
  }

  return false
}

/**
 * Checks if a callee is a React hook (starts with "use"), including member expressions
 */
function isHookIncludingMemberExpressions(callee: TSESTree.CallExpression['callee']): boolean {
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name.startsWith('use')
  }

  // Check for member expressions like namespace.useSomething
  if (callee.type === AST_NODE_TYPES.MemberExpression) {
    if (callee.property.type === AST_NODE_TYPES.Identifier) {
      return callee.property.name.startsWith('use')
    }
  }

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
  functionNode:
    | TSESTree.ArrowFunctionExpression
    | TSESTree.FunctionExpression
    | TSESTree.FunctionDeclaration,
  identifier?: TSESTree.Identifier,
  typeAnnotation?: TSESTree.TSTypeAnnotation,
  sourceCode?: TSESLint.SourceCode,
): boolean {
  // Check FC type annotation - only check PascalCase functions if they have FC type
  if (typeAnnotation && isFCType(typeAnnotation)) {
    // For FC typed functions, also require PascalCase name
    if (identifier) {
      return isPascalCase(identifier.name)
    }
    // For function declarations with FC type, check the function's own id
    if (
      functionNode.type === AST_NODE_TYPES.FunctionDeclaration &&
      functionNode.id
    ) {
      return isPascalCase(functionNode.id.name)
    }
    return true // FC type without name restriction
  }

  // Check function name for hooks (always check hook functions regardless of type)
  if (identifier && isHookName(identifier.name)) {
    return true
  }

  // For function declarations, check if it's a hook
  if (
    functionNode.type === AST_NODE_TYPES.FunctionDeclaration &&
    functionNode.id &&
    isHookName(functionNode.id.name)
  ) {
    return true
  }

  // Check PascalCase functions that contain JSX (without FC type annotation)
  if (sourceCode) {
    const functionName = identifier?.name || 
      (functionNode.type === AST_NODE_TYPES.FunctionDeclaration && functionNode.id?.name)
    
    if (functionName && isPascalCase(functionName)) {
      const analysis = analyzeReactBehavior(functionNode.body, sourceCode)
      if (analysis.containsJSX) return true
    }
  }

  return false
}

/**
 * Checks if a function has the 'use memo' string directive (makes it valid)
 */
function hasUseMemoStringDirective(
  functionNode:
    | TSESTree.ArrowFunctionExpression
    | TSESTree.FunctionExpression
    | TSESTree.FunctionDeclaration,
): boolean {
  // Check for 'use memo' directive in the function body
  if (functionNode.body.type === AST_NODE_TYPES.BlockStatement) {
    for (const statement of functionNode.body.body) {
      if (
        statement.type === AST_NODE_TYPES.ExpressionStatement &&
        statement.expression.type === AST_NODE_TYPES.Literal &&
        typeof statement.expression.value === 'string' &&
        statement.expression.value === 'use memo'
      ) {
        return true
      }
    }
  }

  return false
}

type ReactAnalysisResult = {
  containsJSX: boolean
  containsHookCalls: boolean
}

/**
 * Analyze React component behavior using the generic AST traversal
 * Combines JSX detection and hook call detection in a single pass for performance
 */
function analyzeReactBehavior(
  node: TSESTree.Node,
  sourceCode: TSESLint.SourceCode,
): ReactAnalysisResult {
  const result: ReactAnalysisResult = {
    containsJSX: false,
    containsHookCalls: false,
  }

  traverseAST(
    node,
    (currentNode): boolean | void => {
      // Early exit if we found both
      if (result.containsJSX && result.containsHookCalls) return true

      // Check current node
      switch (currentNode.type) {
        case AST_NODE_TYPES.JSXElement:
        case AST_NODE_TYPES.JSXFragment:
          result.containsJSX = true
          break

        case AST_NODE_TYPES.CallExpression:
          // Check for hook calls
          if (isHook(currentNode.callee)) {
            result.containsHookCalls = true
          }

          // Check if this call expression creates JSX
          if (createsJSX(currentNode)) {
            result.containsJSX = true
          }
          break

        case AST_NODE_TYPES.ReturnStatement:
          if (currentNode.argument && createsJSX(currentNode.argument)) {
            result.containsJSX = true
          }
          break

        case AST_NODE_TYPES.VariableDeclarator:
          if (currentNode.init && createsJSX(currentNode.init)) {
            result.containsJSX = true
          }
          break

        case AST_NODE_TYPES.AssignmentExpression:
          if (createsJSX(currentNode.right)) {
            result.containsJSX = true
          }
          break
      }
    },
    sourceCode,
  )

  return result
}

/**
 * Checks if a function behaves like a React component or hook according to compiler heuristics
 */
function behavesLikeReactComponentOrHook(
  functionNode:
    | TSESTree.ArrowFunctionExpression
    | TSESTree.FunctionExpression
    | TSESTree.FunctionDeclaration,
  sourceCode: TSESLint.SourceCode,
): boolean {
  const body = functionNode.body

  // Use the optimized combined analysis
  const analysis = analyzeReactBehavior(body, sourceCode)

  // According to React Compiler heuristics: function is component/hook if it creates JSX and/or calls hooks
  return analysis.containsJSX || analysis.containsHookCalls
}

const hasThisRegex = /\bthis[.[]/

/**
 * Checks if a function body uses the 'this' keyword using regex
 */
function containsThisKeyword(sourceCode: string): boolean {
  // Simple regex to match 'this' keyword
  return hasThisRegex.test(sourceCode)
}

/**
 * Analyze React component behavior for hook validation (includes member expressions)
 * Only checks direct calls, not nested functions
 */
function analyzeReactBehaviorForHookValidation(
  node: TSESTree.Node,
  sourceCode: TSESLint.SourceCode,
): ReactAnalysisResult {
  const result: ReactAnalysisResult = {
    containsJSX: false,
    containsHookCalls: false,
  }

  traverseAST(
    node,
    (currentNode): boolean | void => {
      // Early exit if we found hook calls (JSX not needed for this check)
      if (result.containsHookCalls) return true

      // Skip nested functions - we don't want to analyze hook calls inside nested functions
      if (currentNode.type === AST_NODE_TYPES.FunctionDeclaration ||
          currentNode.type === AST_NODE_TYPES.FunctionExpression ||
          currentNode.type === AST_NODE_TYPES.ArrowFunctionExpression) {
        // Skip traversing into nested functions
        if (currentNode !== node) return true
      }

      // Check current node
      if (currentNode.type === AST_NODE_TYPES.CallExpression) {
        // Check for hook calls (including member expressions)
        if (isHookIncludingMemberExpressions(currentNode.callee)) {
          result.containsHookCalls = true
        }
      }
    },
    sourceCode,
  )

  return result
}

/**
 * Checks if a function name follows React naming conventions (PascalCase or starts with "use")
 */
function isValidReactFunctionName(functionName: string): boolean {
  return isPascalCase(functionName) || isHookName(functionName)
}

/**
 * Checks if a function calls hooks but is not a valid React component or hook
 */
function callsHooksButNotValidComponent(
  functionNode:
    | TSESTree.ArrowFunctionExpression
    | TSESTree.FunctionExpression
    | TSESTree.FunctionDeclaration,
  sourceCode: TSESLint.SourceCode,
  identifier?: TSESTree.Identifier,
  typeAnnotation?: TSESTree.TSTypeAnnotation,
): boolean {
  // Check if function calls hooks (including member expressions)
  const analysis = analyzeReactBehaviorForHookValidation(functionNode.body, sourceCode)
  if (!analysis.containsHookCalls) {
    return false // Doesn't call hooks, so no violation
  }

  // Check if this is a valid React component or hook
  let isValidReactComponentOrHook = isReactComponentOrHook(
    functionNode,
    identifier,
    typeAnnotation,
    sourceCode,
  )

  // Special case: PascalCase functions with any type annotation that call hooks are valid
  if (!isValidReactComponentOrHook && typeAnnotation) {
    const functionName = identifier?.name || 
      (functionNode.type === AST_NODE_TYPES.FunctionDeclaration && functionNode.id?.name)
    
    if (functionName && isPascalCase(functionName)) {
      isValidReactComponentOrHook = true
    }
  }

  // If it calls hooks but isn't a valid React component or hook, it's a violation
  return !isValidReactComponentOrHook
}

type Options = z.infer<typeof optionsSchema>

const rule = createRule<
  [Options],
  | 'objectMethodIsNotSupported'
  | 'replaceWithFunctionExpression'
  | 'thisKeywordInMethod'
  | 'fcComponentShouldReturnJsx'
  | 'addUseMemoDirective'
  | 'functionCallingHooksMustBeComponent'
  | 'useMemoDirectiveNaming'
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
        'React components and hooks should create JSX elements, call other hooks or use the "use memo" directive for optimal React compiler detection.',
      addUseMemoDirective:
        'Add "use memo" directive to opt into React compiler memoization',
      functionCallingHooksMustBeComponent:
        'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").',
      useMemoDirectiveNaming:
        'Functions using "use memo" directive must follow React naming conventions (PascalCase for components or start with "use" for hooks).',
    },
    hasSuggestions: true,
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    if (hasUseNoMemoDirective(context.sourceCode)) {
      return {}
    }

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
          (node.init.type === AST_NODE_TYPES.ArrowFunctionExpression ||
            node.init.type === AST_NODE_TYPES.FunctionExpression)
        ) {
          const functionNode = node.init
          const identifier = node.id
          const typeAnnotation = node.id.typeAnnotation

          // Check if function with "use memo" directive follows React naming conventions
          if (hasUseMemoStringDirective(functionNode)) {
            if (!isValidReactFunctionName(identifier.name)) {
              context.report({
                node: functionNode,
                messageId: 'useMemoDirectiveNaming',
              })
              return
            }
          }

          // Check if this function calls hooks but is not a valid React component or hook
          if (callsHooksButNotValidComponent(functionNode, context.sourceCode, identifier, typeAnnotation)) {
            context.report({
              node: functionNode,
              messageId: 'functionCallingHooksMustBeComponent',
            })
            return
          }

          // Check if this looks like a React component or hook
          if (
            isReactComponentOrHook(functionNode, identifier, typeAnnotation, context.sourceCode)
          ) {
            const behavesLikeComponent = behavesLikeReactComponentOrHook(
              functionNode,
              context.sourceCode,
            )
            const hasStringDirective = hasUseMemoStringDirective(functionNode)

            // Valid if it behaves like a component/hook OR has string directive
            if (!behavesLikeComponent && !hasStringDirective) {
              context.report({
                node: functionNode,
                messageId: 'fcComponentShouldReturnJsx',
                suggest: [
                  {
                    messageId: 'addUseMemoDirective',
                    fix(fixer) {
                      // Add 'use memo' directive at the beginning of function body
                      if (
                        functionNode.body.type === AST_NODE_TYPES.BlockStatement
                      ) {
                        const openBrace = functionNode.body.range[0] + 1
                        return fixer.insertTextAfterRange(
                          [openBrace, openBrace],
                          '\n  "use memo"\n',
                        )
                      }
                      // For arrow functions with expression body, we can't easily add the directive
                      return null
                    },
                  },
                ],
              })
            }
          }
        }
      },

      FunctionDeclaration(node) {
        // Check if function with "use memo" directive follows React naming conventions
        if (node.id && hasUseMemoStringDirective(node)) {
          if (!isValidReactFunctionName(node.id.name)) {
            context.report({
              node,
              messageId: 'useMemoDirectiveNaming',
            })
            return
          }
        }

        // Check if this function calls hooks but is not a valid React component or hook
        if (callsHooksButNotValidComponent(node, context.sourceCode)) {
          context.report({
            node,
            messageId: 'functionCallingHooksMustBeComponent',
          })
          return
        }

        // Check function declarations that might be React components or hooks
        if (isReactComponentOrHook(node, undefined, undefined, context.sourceCode)) {
          const behavesLikeComponent = behavesLikeReactComponentOrHook(
            node,
            context.sourceCode,
          )
          const hasStringDirective = hasUseMemoStringDirective(node)

          // Valid if it behaves like a component/hook OR has string directive
          if (!behavesLikeComponent && !hasStringDirective) {
            context.report({
              node,
              messageId: 'fcComponentShouldReturnJsx',
              suggest: [
                {
                  messageId: 'addUseMemoDirective',
                  fix(fixer) {
                    // Add 'use memo' directive at the beginning of function body
                    const openBrace = node.body.range[0] + 1
                    return fixer.insertTextAfterRange(
                      [openBrace, openBrace],
                      '\n  "use memo"\n',
                    )
                  },
                },
              ],
            })
          }
        }
      },

      FunctionExpression(node) {
        // Check if function expression with "use memo" directive follows React naming conventions
        // Function expressions can't be directly named with React conventions, so they should not use "use memo"
        if (hasUseMemoStringDirective(node)) {
          context.report({
            node,
            messageId: 'useMemoDirectiveNaming',
          })
          return
        }

        // Check if this function expression calls hooks but is not a valid React component or hook
        if (callsHooksButNotValidComponent(node, context.sourceCode)) {
          context.report({
            node,
            messageId: 'functionCallingHooksMustBeComponent',
          })
        }
      },
    }
  },
})

export const reactCompilerExtra = {
  name,
  rule,
}
