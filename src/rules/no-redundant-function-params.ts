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

function getRedundantProperties(
  argValue: unknown,
  defaultValue: unknown,
): Set<string> {
  const redundantProps = new Set<string>()

  if (
    typeof argValue !== 'object' ||
    argValue === null ||
    typeof defaultValue !== 'object' ||
    defaultValue === null
  ) {
    return redundantProps
  }

  for (const key in argValue) {
    if (!(key in defaultValue)) continue

    const argDescriptor = Object.getOwnPropertyDescriptor(argValue, key)
    const defaultDescriptor = Object.getOwnPropertyDescriptor(defaultValue, key)

    if (
      argDescriptor &&
      defaultDescriptor &&
      deepEqual(argDescriptor.value, defaultDescriptor.value)
    ) {
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
      return arg.elements.map((el) => {
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

function getFunctionName(
  callee: TSESTree.CallExpression['callee'],
): string | null {
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
  'redundantParams'
>({
  name: 'no-redundant-function-params',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow calling functions with redundant default parameters',
    },
    messages: {
      redundantParams:
        'Function call has redundant parameters matching its default values and can be removed: {{details}}',
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
        const fullyRedundantObjects = new Set<number>()
        const simpleRedundantParams = new Map<number, unknown>()

        type RedundancyInfo = {
          type: 'simple' | 'object' | 'partialObject'
          position: number
          value?: unknown
          redundantProperties?: string[]
        }
        const redundancyDetails: RedundancyInfo[] = []

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

          if (deepEqual(argValue, defaultValue)) {
            // Check if this is an object parameter
            if (arg.type === AST_NODE_TYPES.ObjectExpression) {
              fullyRedundantObjects.add(i)
              redundancyDetails.push({
                type: 'object',
                position: i + 1,
              })
            } else {
              simpleRedundantParams.set(i, argValue)
              redundancyDetails.push({
                type: 'simple',
                position: i + 1,
                value: argValue,
              })
            }
            continue
          }

          // Check if object is empty and default has properties
          if (
            arg.type === AST_NODE_TYPES.ObjectExpression &&
            arg.properties.length === 0 &&
            typeof defaultValue === 'object' &&
            defaultValue !== null &&
            Object.keys(defaultValue).length > 0
          ) {
            fullyRedundantObjects.add(i)
            redundancyDetails.push({
              type: 'object',
              position: i + 1,
            })
            continue
          }

          const redundantProps = getRedundantProperties(argValue, defaultValue)
          if (redundantProps.size > 0) {
            // Check if ALL properties are redundant AND object has same number of properties as default
            const defaultPropertyCount =
              typeof defaultValue === 'object' && defaultValue !== null
                ? Object.keys(defaultValue).length
                : 0
            if (
              arg.type === AST_NODE_TYPES.ObjectExpression &&
              arg.properties.length === redundantProps.size &&
              arg.properties.length === defaultPropertyCount
            ) {
              fullyRedundantObjects.add(i)
              redundancyDetails.push({
                type: 'object',
                position: i + 1,
              })
              continue
            } else {
              hasObjectWithRedundantProps = true
              objectRedundantProps.set(i, redundantProps)
              redundancyDetails.push({
                type: 'partialObject',
                position: i + 1,
                redundantProperties: Array.from(redundantProps),
              })
              lastNonRedundantIndex = i
            }
          } else {
            lastNonRedundantIndex = i
          }
        }

        if (
          lastNonRedundantIndex < args.length - 1 ||
          hasObjectWithRedundantProps ||
          fullyRedundantObjects.size > 0
        ) {
          function generateMessageData() {
            const detailParts: string[] = []
            
            for (const detail of redundancyDetails) {
              if (detail.type === 'simple') {
                detailParts.push(`Param ${detail.position} with value ${JSON.stringify(detail.value)}`)
              } else if (detail.type === 'object') {
                detailParts.push(`Param ${detail.position} (object)`)
              } else {
                const props = detail.redundantProperties || []
                const defaultValues = defaults ? defaults[detail.position - 1] : undefined
                
                if (typeof defaultValues === 'object' && defaultValues !== null) {
                  const defaultObj = defaultValues
                  for (const prop of props) {
                    if (prop in defaultObj) {
                      const propertyDescriptor = Object.getOwnPropertyDescriptor(defaultObj, prop)
                      if (propertyDescriptor) {
                        detailParts.push(`Param ${detail.position} property "${prop}" with value ${JSON.stringify(propertyDescriptor.value)}`)
                      }
                    }
                  }
                }
              }
            }
            
            return {
              messageId: 'redundantParams' as const,
              data: {
                details: detailParts.join(', ')
              }
            }
          }

          const messageData = generateMessageData()

          context.report({
            node,
            messageId: messageData.messageId,
            data: messageData.data,
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

                    const propertyTexts = keepProperties.map((prop) =>
                      context.sourceCode.getText(prop),
                    )
                    fixes.push(
                      fixer.replaceText(arg, `{ ${propertyTexts.join(', ')} }`),
                    )
                  }
                }
              }

              // Find the rightmost non-redundant argument
              let rightmostNonRedundant = -1
              for (let i = args.length - 1; i >= 0; i--) {
                if (
                  !fullyRedundantObjects.has(i) &&
                  i <= lastNonRedundantIndex
                ) {
                  rightmostNonRedundant = i
                  break
                }
              }

              // Remove trailing redundant arguments (including fully redundant objects)
              if (rightmostNonRedundant < args.length - 1) {
                if (rightmostNonRedundant === -1) {
                  const openParen = context.sourceCode.getTokenAfter(
                    node.callee,
                  )
                  const closeParen = context.sourceCode.getLastToken(node)

                  if (openParen && closeParen) {
                    fixes.push(
                      fixer.replaceTextRange(
                        [openParen.range[1], closeParen.range[0]],
                        '',
                      ),
                    )
                  }
                } else {
                  const lastNonRedundantArg = args[rightmostNonRedundant]
                  const closeParen = context.sourceCode.getLastToken(node)

                  if (closeParen && lastNonRedundantArg) {
                    fixes.push(
                      fixer.replaceTextRange(
                        [lastNonRedundantArg.range[1], closeParen.range[0]],
                        '',
                      ),
                    )
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
