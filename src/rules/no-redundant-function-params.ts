import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const functionConfigSchema = z.object({
  name: z.string(),
  defaults: z.array(z.unknown()),
})

const optionsSchema = z.object({
  functions: z.array(functionConfigSchema),
})

function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true
  
  if (a === null || b === null) return a === b
  if (a === undefined || b === undefined) return a === b
  
  if (typeof a !== typeof b) return false
  
  if (typeof a !== 'object') return false
  
  if (Array.isArray(a) !== Array.isArray(b)) return false
  
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false
    return a.every((item, index) => deepEqual(item, b[index]))
  }
  
  const aKeys = Object.keys(a)
  const bKeys = Object.keys(b)
  
  if (aKeys.length !== bKeys.length) return false
  
  for (const key of aKeys) {
    if (!bKeys.includes(key)) return false
    
    const aDescriptor = Object.getOwnPropertyDescriptor(a, key)
    const bDescriptor = Object.getOwnPropertyDescriptor(b, key)
    
    if (!aDescriptor || !bDescriptor) return false
    
    const aValue = aDescriptor.value
    const bValue = bDescriptor.value
    
    if (!deepEqual(aValue, bValue)) return false
  }
  
  return true
}


function getRedundantProperties(argValue: unknown, defaultValue: unknown): Set<string> {
  const redundantProps = new Set<string>()
  
  if (typeof argValue !== 'object' || argValue === null || 
      typeof defaultValue !== 'object' || defaultValue === null) {
    return redundantProps
  }
  
  for (const key in argValue) {
    if (!(key in defaultValue)) continue
    
    const argDescriptor = Object.getOwnPropertyDescriptor(argValue, key)
    const defaultDescriptor = Object.getOwnPropertyDescriptor(defaultValue, key)
    
    if (argDescriptor && defaultDescriptor && 
        deepEqual(argDescriptor.value, defaultDescriptor.value)) {
      redundantProps.add(key)
    }
  }
  
  return redundantProps
}


function getArgumentValue(arg: TSESTree.CallExpressionArgument): unknown {
  if (arg.type === AST_NODE_TYPES.SpreadElement) {
    return Symbol('COMPLEX_EXPRESSION')
  }

  switch (arg.type) {
    case AST_NODE_TYPES.Literal:
      return arg.value
    
    case AST_NODE_TYPES.ObjectExpression: {
      const obj: Record<string, unknown> = {}
      for (const prop of arg.properties) {
        if (
          prop.type === AST_NODE_TYPES.Property &&
          prop.key.type === AST_NODE_TYPES.Identifier &&
          !prop.computed
        ) {
          if (prop.value.type === AST_NODE_TYPES.Literal) {
            obj[prop.key.name] = prop.value.value
          } else if (prop.value.type === AST_NODE_TYPES.ObjectExpression) {
            obj[prop.key.name] = getArgumentValue(prop.value)
          } else {
            obj[prop.key.name] = Symbol('COMPLEX_EXPRESSION')
          }
        }
      }
      return obj
    }
    
    case AST_NODE_TYPES.ArrayExpression:
      return arg.elements.map(el => {
        if (!el || el.type === AST_NODE_TYPES.SpreadElement) {
          return undefined
        }
        if (el.type === AST_NODE_TYPES.Literal) {
          return el.value
        }
        return Symbol('COMPLEX_EXPRESSION')
      })
    
    default:
      return Symbol('COMPLEX_EXPRESSION')
  }
}

function getFunctionName(callee: TSESTree.CallExpression['callee']): string | null {
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name
  }
  
  if (
    callee.type === AST_NODE_TYPES.MemberExpression &&
    callee.property.type === AST_NODE_TYPES.Identifier &&
    !callee.computed
  ) {
    return callee.property.name
  }
  
  return null
}

type Options = z.infer<typeof optionsSchema>

export const noRedundantFunctionParams = createExtendedLintRule<
  [Options],
  'redundantParam'
>({
  name: 'no-redundant-function-params',
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Disallow calling functions with redundant default parameters',
    },
    messages: {
      redundantParam: 'Function call has redundant default parameter(s) that can be omitted',
    },
    fixable: 'code',
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ functions: [] }],
  create(context, [options]) {
    const functionConfigs = new Map<string, unknown[]>()
    
    for (const config of options.functions) {
      functionConfigs.set(config.name, config.defaults)
    }
    
    return {
      CallExpression(node) {
        const functionName = getFunctionName(node.callee)
        if (!functionName) return
        
        const defaults = functionConfigs.get(functionName)
        if (!defaults) return
        
        const args = node.arguments
        if (args.length === 0) return
        
        let lastNonRedundantIndex = -1
        let hasObjectWithRedundantProps = false
        const objectRedundantProps = new Map<number, Set<string>>()
        
        for (let i = 0; i < args.length; i++) {
          if (i >= defaults.length) {
            lastNonRedundantIndex = i
            continue
          }
          
          const defaultValue = defaults[i]
          const arg = args[i]
          if (!arg || arg.type === AST_NODE_TYPES.SpreadElement) {
            lastNonRedundantIndex = i
            continue
          }
          
          const argValue = getArgumentValue(arg)
          
          if (defaultValue === undefined) {
            lastNonRedundantIndex = i
            continue
          }
          
          if (deepEqual(argValue, defaultValue)) continue
          
          const redundantProps = getRedundantProperties(argValue, defaultValue)
          if (redundantProps.size > 0) {
            hasObjectWithRedundantProps = true
            objectRedundantProps.set(i, redundantProps)
            lastNonRedundantIndex = i
          } else {
            lastNonRedundantIndex = i
          }
        }
        
        if (lastNonRedundantIndex < args.length - 1 || hasObjectWithRedundantProps) {
          context.report({
            node,
            messageId: 'redundantParam',
            fix(fixer) {
              const fixes = []
              
              if (hasObjectWithRedundantProps) {
                for (const [argIndex, redundantProps] of objectRedundantProps) {
                  const arg = args[argIndex]
                  if (arg && arg.type === AST_NODE_TYPES.ObjectExpression) {
                    const keepProperties = []
                    
                    for (const prop of arg.properties) {
                      if (prop.type === AST_NODE_TYPES.Property) {
                        if (
                          prop.key.type === AST_NODE_TYPES.Identifier &&
                          !prop.computed &&
                          redundantProps.has(prop.key.name)
                        ) {
                          // Skip redundant property
                          continue
                        }
                      }
                      keepProperties.push(prop)
                    }
                    
                    if (keepProperties.length === 0) {
                      fixes.push(fixer.replaceText(arg, '{}'))
                    } else {
                      const propertyTexts = keepProperties.map(prop => context.sourceCode.getText(prop))
                      fixes.push(fixer.replaceText(arg, `{ ${propertyTexts.join(', ')} }`))
                    }
                  }
                }
              }
              
              if (lastNonRedundantIndex < args.length - 1) {
                if (lastNonRedundantIndex === -1) {
                  const openParen = context.sourceCode.getTokenAfter(node.callee)
                  const closeParen = context.sourceCode.getLastToken(node)
                  
                  if (openParen && closeParen) {
                    fixes.push(fixer.replaceTextRange(
                      [openParen.range[1], closeParen.range[0]],
                      ''
                    ))
                  }
                } else {
                  const lastNonRedundantArg = args[lastNonRedundantIndex]
                  const closeParen = context.sourceCode.getLastToken(node)
                  
                  if (closeParen && lastNonRedundantArg) {
                    fixes.push(fixer.replaceTextRange(
                      [lastNonRedundantArg.range[1], closeParen.range[0]],
                      ''
                    ))
                  }
                }
              }
              
              return fixes.length > 0 ? fixes : null
            },
          })
        }
      },
    }
  },
})