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
      declarationNode: TSESTree.Node  // The node that declares the function (FunctionDeclaration or VariableDeclarator)
      optionalParams: Array<{
        param: TSESTree.Parameter
        index: number
        name: string
        isObjectParam: boolean
        objectProps?: Array<{ name: string; optional: boolean }>
      }>
    }

    const trackedFunctions: FunctionInfo[] = []
    
    function shouldIgnoreParam(paramName: string): boolean {
      if (!options.ignoreArgsMatching) return false
      const regex = new RegExp(options.ignoreArgsMatching)
      return regex.test(paramName)
    }

    function isExportedFunction(declarationNode: TSESTree.Node): boolean {
      return declarationNode.type === AST_NODE_TYPES.ExportNamedDeclaration ||
             declarationNode.type === AST_NODE_TYPES.ExportDefaultDeclaration ||
             declarationNode.parent?.type === AST_NODE_TYPES.ExportNamedDeclaration || 
             declarationNode.parent?.type === AST_NODE_TYPES.ExportDefaultDeclaration
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


    return {
      FunctionDeclaration(node) {
        if (isExportedFunction(node)) return
        
        const optionalParams = getOptionalParams(node.params)
        if (optionalParams.length === 0) return
        
        trackedFunctions.push({
          node,
          declarationNode: node,
          optionalParams,
        })
      },
      
      VariableDeclarator(node) {
        if (node.id.type !== AST_NODE_TYPES.Identifier) return
        if (!node.init) return
        if (node.init.type !== AST_NODE_TYPES.ArrowFunctionExpression && 
            node.init.type !== AST_NODE_TYPES.FunctionExpression) return
        
        // Check if this is exported
        if (isExportedFunction(node.parent.parent)) return
        
        const baseOptionalParams = getOptionalParams(node.init.params)
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
        
        trackedFunctions.push({
          node: node.init,
          declarationNode: node,
          optionalParams,
        })
      },
      
      'Program:exit'() {
        for (const functionInfo of trackedFunctions) {
          const variables = context.sourceCode.getDeclaredVariables(functionInfo.declarationNode)
          const functionVariable = variables[0]
          if (!functionVariable) continue
          
          const references = functionVariable.references.filter(ref => ref.identifier !== functionVariable.identifiers[0])
          
          if (references.length === 0) continue
          
          // Check if function is passed as argument, returned, or used in complex ways
          let skipAnalysis = false
          for (const ref of references) {
            const parent = ref.identifier.parent
            
            // Function passed as argument
            if (parent.type === AST_NODE_TYPES.CallExpression && 
                ref.identifier.type === AST_NODE_TYPES.Identifier && 
                parent.arguments.includes(ref.identifier)) {
              skipAnalysis = true
              break
            }
            
            // Function returned
            if (parent.type === AST_NODE_TYPES.ReturnStatement) {
              skipAnalysis = true
              break
            }
            
            // Function returned as object property
            if (parent.type === AST_NODE_TYPES.Property && 
                parent.parent.type === AST_NODE_TYPES.ObjectExpression &&
                parent.parent.parent.type === AST_NODE_TYPES.ReturnStatement) {
              skipAnalysis = true
              break
            }
            
            // Function used as JSX prop
            if (parent.type === AST_NODE_TYPES.JSXExpressionContainer) {
              skipAnalysis = true
              break
            }
          }
          
          if (skipAnalysis) continue
          
          // Analyze calls and JSX usage
          const calls: Array<{ argCount: number; hasSpread: boolean; objectArgs?: Array<{ [key: string]: boolean }> }> = []
          
          for (const ref of references) {
            const parent = ref.identifier.parent
            
            // Regular function call
            if (parent.type === AST_NODE_TYPES.CallExpression && parent.callee === ref.identifier) {
              const argCount = parent.arguments.length
              const hasSpread = parent.arguments.some(arg => arg.type === AST_NODE_TYPES.SpreadElement)
              
              if (hasSpread) {
                skipAnalysis = true
                break
              }
              
              const callInfo: { argCount: number; hasSpread: boolean; objectArgs?: Array<{ [key: string]: boolean }> } = {
                argCount,
                hasSpread,
              }
              
              // Analyze object arguments
              for (const optionalParam of functionInfo.optionalParams) {
                if (optionalParam.isObjectParam && optionalParam.index < argCount) {
                  const arg = parent.arguments[optionalParam.index]
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
                    // Non-object argument - can't analyze
                    skipAnalysis = true
                    break
                  }
                }
              }
              
              calls.push(callInfo)
            }
            
            // JSX usage
            else if (parent.type === AST_NODE_TYPES.JSXOpeningElement && parent.name === ref.identifier) {
              const hasSpread = parent.attributes.some(attr => attr.type === AST_NODE_TYPES.JSXSpreadAttribute)
              
              if (hasSpread) {
                skipAnalysis = true
                break
              }
              
              const callInfo: { argCount: number; hasSpread: boolean; objectArgs?: Array<{ [key: string]: boolean }> } = {
                argCount: 1,
                hasSpread,
              }
              
              // Analyze JSX attributes for object parameters
              for (const optionalParam of functionInfo.optionalParams) {
                if (optionalParam.isObjectParam && optionalParam.index === 0) {
                  const objectArg: { [key: string]: boolean } = {}
                  for (const attr of parent.attributes) {
                    if (attr.type === AST_NODE_TYPES.JSXAttribute && attr.name.type === AST_NODE_TYPES.JSXIdentifier) {
                      objectArg[attr.name.name] = true
                    }
                  }
                  if (!callInfo.objectArgs) callInfo.objectArgs = []
                  callInfo.objectArgs[0] = objectArg
                }
              }
              
              calls.push(callInfo)
            }
          }
          
          if (skipAnalysis || calls.length === 0) continue
          
          // Check each optional parameter
          for (const optionalParam of functionInfo.optionalParams) {
            if (optionalParam.isObjectParam && optionalParam.objectProps) {
              // Check object properties
              for (const objectProp of optionalParam.objectProps) {
                if (!objectProp.optional) continue
                
                let isPropertyUsed = false
                
                for (const call of calls) {
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
              
              for (const call of calls) {
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
