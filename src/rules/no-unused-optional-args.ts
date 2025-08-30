import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  ignoreArgsMatching: z.string().optional(),
})

type Options = z.infer<typeof optionsSchema>

export const noUnusedOptionalArgs = createExtendedLintRule<
  [Options],
  'unusedOptionalArg' | 'unusedOptionalProp'
>({
  name: 'no-unused-optional-args',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Detect unused optional function arguments in non-exported functions',
    },
    messages: {
      unusedOptionalArg: "Optional parameter '{{name}}' is never used",
      unusedOptionalProp: "Optional prop '{{name}}' is never provided",
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {

    type FunctionInfo = {
      node:
        | TSESTree.FunctionDeclaration
        | TSESTree.ArrowFunctionExpression
        | TSESTree.FunctionExpression
      name: string
      isExported: boolean
      optionalParams: Array<{
        param: TSESTree.Parameter
        index: number
        name: string
        isObjectParam: boolean
        objectProps?: Array<{ name: string; optional: boolean }>
      }>
      calls: Array<{
        node: TSESTree.CallExpression | TSESTree.JSXOpeningElement
        argCount: number
        hasSpread: boolean
        objectArgs?: Array<{ [key: string]: boolean }> // maps prop name to whether it was provided
      }>
      isPassedAsArgument: boolean
      isUsedInJSX: boolean
    }

    const functions = new Map<TSESTree.Node, FunctionInfo>()
    const functionsByName = new Map<string, TSESTree.Node[]>()
    
    function shouldIgnoreParam(paramName: string): boolean {
      if (!options.ignoreArgsMatching) return false
      const regex = new RegExp(options.ignoreArgsMatching)
      return regex.test(paramName)
    }

    function isExportedFunction(node: TSESTree.Node): boolean {
      if (node.parent?.type === AST_NODE_TYPES.ExportNamedDeclaration || node.parent?.type === AST_NODE_TYPES.ExportDefaultDeclaration) {
        return true
      }
      return false
    }

    function getFunctionName(
      node:
        | TSESTree.FunctionDeclaration
        | TSESTree.ArrowFunctionExpression  
        | TSESTree.FunctionExpression
    ): string | null {
      if (node.type === AST_NODE_TYPES.FunctionDeclaration) {
        return node.id?.name || null
      }
      
      // For arrow functions and function expressions, check if they're assigned to a variable
      if (node.parent.type === AST_NODE_TYPES.VariableDeclarator && node.parent.id.type === AST_NODE_TYPES.Identifier) {
        return node.parent.id.name
      }
      
      return null
    }

    function getOptionalParams(params: TSESTree.Parameter[]): FunctionInfo['optionalParams'] {
      const result: FunctionInfo['optionalParams'] = []
      
      for (let i = 0; i < params.length; i++) {
        const param = params[i]
        
        if (!param || param.type !== AST_NODE_TYPES.Identifier) continue
        
        const paramName = param.name
        if (shouldIgnoreParam(paramName)) continue
        
        // Case 1: Optional parameter with optional type literal (options?: { a?: string })
        if (param.optional && param.typeAnnotation && param.typeAnnotation.typeAnnotation.type === AST_NODE_TYPES.TSTypeLiteral) {
          const typeAnnotation = param.typeAnnotation.typeAnnotation
          const objectProps: Array<{ name: string; optional: boolean }> = []
          
          for (const member of typeAnnotation.members) {
            if (member.type === AST_NODE_TYPES.TSPropertySignature && member.key.type === AST_NODE_TYPES.Identifier) {
              objectProps.push({
                name: member.key.name,
                optional: member.optional === true,
              })
            }
          }
          
          if (objectProps.some(prop => prop.optional)) {
            result.push({
              param,
              index: i,
              name: paramName,
              isObjectParam: true,
              objectProps,
            })
          } else {
            result.push({
              param,
              index: i,
              name: paramName,
              isObjectParam: false,
            })
          }
        }
        // Case 2: Simple optional parameter (arg?: string)
        else if (param.optional) {
          result.push({
            param,
            index: i,
            name: paramName,
            isObjectParam: false,
          })
        }
        // Case 3: Required parameter with optional properties (props: { a?: string })
        else if (param.typeAnnotation && param.typeAnnotation.typeAnnotation.type === AST_NODE_TYPES.TSTypeLiteral) {
          const typeAnnotation = param.typeAnnotation.typeAnnotation
          const objectProps: Array<{ name: string; optional: boolean }> = []
          
          for (const member of typeAnnotation.members) {
            if (member.type === AST_NODE_TYPES.TSPropertySignature && member.key.type === AST_NODE_TYPES.Identifier) {
              objectProps.push({
                name: member.key.name,
                optional: member.optional === true,
              })
            }
          }
          
          if (objectProps.some(prop => prop.optional)) {
            result.push({
              param,
              index: i,
              name: paramName,
              isObjectParam: true,
              objectProps,
            })
          }
        }
      }
      
      return result
    }

    function analyzeCallExpression(call: TSESTree.CallExpression, functionInfo: FunctionInfo) {
      const argCount = call.arguments.length
      const hasSpread = call.arguments.some(arg => arg.type === AST_NODE_TYPES.SpreadElement)
      
      const callInfo: FunctionInfo['calls'][0] = {
        node: call,
        argCount,
        hasSpread,
      }
      
      // Analyze object arguments for object parameters
      for (const optionalParam of functionInfo.optionalParams) {
        if (optionalParam.isObjectParam && optionalParam.index < argCount) {
          const arg = call.arguments[optionalParam.index]
          if (arg?.type === AST_NODE_TYPES.ObjectExpression) {
            const objectArg: { [key: string]: boolean } = {}
            for (const prop of arg.properties) {
              if (prop.type === AST_NODE_TYPES.Property && prop.key.type === AST_NODE_TYPES.Identifier) {
                objectArg[prop.key.name] = true
              }
            }
            if (!callInfo.objectArgs) callInfo.objectArgs = []
            callInfo.objectArgs[optionalParam.index] = objectArg
          } else if (arg) {
            // Non-object argument passed to object parameter - can't analyze
            functionInfo.isPassedAsArgument = true
            return
          }
        }
      }
      
      functionInfo.calls.push(callInfo)
    }

    function analyzeJSXElement(jsx: TSESTree.JSXOpeningElement, functionInfo: FunctionInfo) {
      const hasSpread = jsx.attributes.some(attr => attr.type === AST_NODE_TYPES.JSXSpreadAttribute)
      
      const callInfo: FunctionInfo['calls'][0] = {
        node: jsx,
        argCount: 1, // JSX always passes props as first argument
        hasSpread,
      }
      
      // Analyze JSX attributes for object parameters
      for (const optionalParam of functionInfo.optionalParams) {
        if (optionalParam.isObjectParam && optionalParam.index === 0) {
          const objectArg: { [key: string]: boolean } = {}
          for (const attr of jsx.attributes) {
            if (attr.type === AST_NODE_TYPES.JSXAttribute && attr.name.type === AST_NODE_TYPES.JSXIdentifier) {
              objectArg[attr.name.name] = true
            }
          }
          if (!callInfo.objectArgs) callInfo.objectArgs = []
          callInfo.objectArgs[0] = objectArg
        }
      }
      
      functionInfo.calls.push(callInfo)
    }

    return {
      // Track function definitions
      FunctionDeclaration(node) {
        const name = getFunctionName(node)
        if (!name) return
        
        const isExported = isExportedFunction(node)
        if (isExported) return
        
        const optionalParams = getOptionalParams(node.params)
        if (optionalParams.length === 0) return
        
        const functionInfo: FunctionInfo = {
          node,
          name,
          isExported,
          optionalParams,
          calls: [],
          isPassedAsArgument: false,
          isUsedInJSX: false,
        }
        
        functions.set(node, functionInfo)
        
        // Track functions by name for call resolution
        const existingFunctions = functionsByName.get(name) || []
        existingFunctions.push(node)
        functionsByName.set(name, existingFunctions)
      },
      
      VariableDeclarator(node) {
        if (node.id.type !== AST_NODE_TYPES.Identifier) return
        if (!node.init) return
        if (node.init.type !== AST_NODE_TYPES.ArrowFunctionExpression && node.init.type !== AST_NODE_TYPES.FunctionExpression) return
        
        const name = node.id.name
        const functionNode = node.init
        
        // Check if this is exported
        const isExported = isExportedFunction(node.parent.parent)
        if (isExported) return
        
        const baseOptionalParams = getOptionalParams(functionNode.params)
        const optionalParams = [...baseOptionalParams]
        
        // Check if this has FC type annotation for React components
        if (node.id.typeAnnotation && node.id.typeAnnotation.typeAnnotation.type === AST_NODE_TYPES.TSTypeReference) {
          const typeRef = node.id.typeAnnotation.typeAnnotation
          if (typeRef.typeName.type === AST_NODE_TYPES.Identifier && typeRef.typeName.name === 'FC') {
            // Extract props from FC<Props> type  
            const typeArgs = typeRef.typeArguments
            if (typeArgs && typeArgs.params.length === 1) {
              const propsType = typeArgs.params[0]
              if (propsType && propsType.type === AST_NODE_TYPES.TSTypeLiteral) {
                const objectProps: Array<{ name: string; optional: boolean }> = []
                
                for (const member of propsType.members) {
                  if (member.type === AST_NODE_TYPES.TSPropertySignature && member.key.type === AST_NODE_TYPES.Identifier) {
                    objectProps.push({
                      name: member.key.name,
                      optional: member.optional === true,
                    })
                  }
                }
                
                if (objectProps.some(prop => prop.optional)) {
                  // Add a synthetic parameter for FC props
                  optionalParams.push({
                    param: node.id,
                    index: 0,
                    name: 'props',
                    isObjectParam: true,
                    objectProps,
                  })
                }
              }
            }
          }
        }
        
        if (optionalParams.length === 0) return
        
        const functionInfo: FunctionInfo = {
          node: functionNode,
          name,
          isExported,
          optionalParams,
          calls: [],
          isPassedAsArgument: false,
          isUsedInJSX: false,
        }
        
        functions.set(functionNode, functionInfo)
        
        // Track functions by name for call resolution
        const existingFunctions = functionsByName.get(name) || []
        existingFunctions.push(functionNode)
        functionsByName.set(name, existingFunctions)
      },
      
      // Track function calls
      CallExpression(node) {
        if (node.callee.type === AST_NODE_TYPES.Identifier) {
          const functionNodes = functionsByName.get(node.callee.name)
          if (functionNodes) {
            // If there are multiple functions with the same name, skip analysis to be safe
            if (functionNodes.length > 1) {
              for (const functionNode of functionNodes) {
                const functionInfo = functions.get(functionNode)
                if (functionInfo) {
                  functionInfo.isPassedAsArgument = true
                }
              }
            } else if (functionNodes[0]) {
              const functionInfo = functions.get(functionNodes[0])
              if (functionInfo) {
                analyzeCallExpression(node, functionInfo)
              }
            }
          }
        }
      },
      
      // Track JSX element usage
      JSXOpeningElement(node) {
        if (node.name.type === AST_NODE_TYPES.JSXIdentifier) {
          const functionNodes = functionsByName.get(node.name.name)
          if (functionNodes) {
            // If there are multiple functions with the same name, skip analysis to be safe
            if (functionNodes.length > 1) {
              for (const functionNode of functionNodes) {
                const functionInfo = functions.get(functionNode)
                if (functionInfo) {
                  functionInfo.isPassedAsArgument = true
                }
              }
            } else if (functionNodes[0]) {
              const functionInfo = functions.get(functionNodes[0])
              if (functionInfo) {
                functionInfo.isUsedInJSX = true
                analyzeJSXElement(node, functionInfo)
              }
            }
          }
        }
      },
      
      // Track when function is passed as argument
      'CallExpression > Identifier'(node: TSESTree.Identifier) {
        const functionNodes = functionsByName.get(node.name)
        if (functionNodes) {
          const parent = node.parent
          if (parent.type === AST_NODE_TYPES.CallExpression && parent.arguments.includes(node)) {
            // Mark all functions with this name as passed as argument
            for (const functionNode of functionNodes) {
              const functionInfo = functions.get(functionNode)
              if (functionInfo) {
                functionInfo.isPassedAsArgument = true
              }
            }
          }
        }
      },
      
      // Track when function is used as JSX prop
      'JSXExpressionContainer > Identifier'(node: TSESTree.Identifier) {
        const functionNodes = functionsByName.get(node.name)
        if (functionNodes) {
          for (const functionNode of functionNodes) {
            const functionInfo = functions.get(functionNode)
            if (functionInfo) {
              functionInfo.isPassedAsArgument = true
            }
          }
        }
      },
      
      // Track when function is returned
      'ReturnStatement > Identifier'(node: TSESTree.Identifier) {
        const functionNodes = functionsByName.get(node.name)
        if (functionNodes) {
          for (const functionNode of functionNodes) {
            const functionInfo = functions.get(functionNode)
            if (functionInfo) {
              functionInfo.isPassedAsArgument = true
            }
          }
        }
      },
      
      // Track when function is returned as object property
      'ReturnStatement > ObjectExpression > Property > Identifier'(node: TSESTree.Identifier) {
        const functionNodes = functionsByName.get(node.name)
        if (functionNodes) {
          for (const functionNode of functionNodes) {
            const functionInfo = functions.get(functionNode)
            if (functionInfo) {
              functionInfo.isPassedAsArgument = true
            }
          }
        }
      },
      
      'Program:exit'() {
        for (const functionInfo of functions.values()) {
          // Skip if function is passed as argument or used in complex ways
          if (functionInfo.isPassedAsArgument) continue
          
          // Skip if any call uses spread args
          if (functionInfo.calls.some(call => call.hasSpread)) continue
          
          // Skip if no calls found
          if (functionInfo.calls.length === 0) continue
          
          // Check each optional parameter
          for (const optionalParam of functionInfo.optionalParams) {
            if (optionalParam.isObjectParam && optionalParam.objectProps) {
              // Check object properties
              for (const objectProp of optionalParam.objectProps) {
                if (!objectProp.optional) continue
                
                let isPropertyUsed = false
                
                for (const call of functionInfo.calls) {
                  if (call.objectArgs && call.objectArgs[optionalParam.index]) {
                    const objectArg = call.objectArgs[optionalParam.index]
                    if (objectArg && objectArg[objectProp.name]) {
                      isPropertyUsed = true
                      break
                    }
                  }
                }
                
                if (!isPropertyUsed) {
                  context.report({
                    node: optionalParam.param,
                    messageId: 'unusedOptionalProp',
                    data: { name: objectProp.name },
                  })
                }
              }
            } else {
              // Check regular optional parameter
              let isParameterUsed = false
              
              for (const call of functionInfo.calls) {
                if (call.argCount > optionalParam.index) {
                  isParameterUsed = true
                  break
                }
              }
              
              if (!isParameterUsed) {
                context.report({
                  node: optionalParam.param,
                  messageId: 'unusedOptionalArg',
                  data: { name: optionalParam.name },
                })
              }
            }
          }
        }
      },
    }
  },
})
