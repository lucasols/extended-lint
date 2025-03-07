import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import * as t from 'tschema'
import * as ts from 'typescript'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-unnecessary-casting'

// Define options schema
const optionsSchema = t.object({
  additionalCastFunctions: t.optional(
    t.array(
      t.object({
        name: t.string(),
        expectedType: t.enum(['string', 'number']),
      }),
    ),
  ),
})

type Options = t.Infer<typeof optionsSchema>

type CastFunction = {
  name: string
  expectedType: 'string' | 'number'
}

export const noUnnecessaryCasting = {
  name,
  rule: createRule<
    [Options],
    | 'unnecessaryNumberCasting'
    | 'unnecessaryStringCasting'
    | 'unnecessaryCustomCasting'
  >({
    name,
    meta: {
      type: 'suggestion',
      docs: {
        description: 'Prevents unnecessary Number and String castings',
      },
      messages: {
        unnecessaryNumberCasting:
          'Unnecessary Number() casting on a value already of number type',
        unnecessaryStringCasting:
          'Unnecessary String() casting on a value already of string type',
        unnecessaryCustomCasting:
          'Unnecessary {{name}}() casting on a value already of {{type}} type',
      },
      schema: [optionsSchema as any],
      fixable: 'code',
    },
    defaultOptions: [{}],
    create(context) {
      const options = context.options[0]
      const additionalCastFunctions = options.additionalCastFunctions || []

      // Create a unified list of all cast functions to check
      const castFunctions: CastFunction[] = [
        { name: 'Number', expectedType: 'number' },
        { name: 'String', expectedType: 'string' },
        ...additionalCastFunctions,
      ]

      const parserServices = ESLintUtils.getParserServices(context, true)
      const checker = parserServices.program?.getTypeChecker()

      if (!checker || !parserServices.program) {
        throw new Error('TypeScript services or program not available')
      }

      // Helper to check if a type matches the expected type
      function typeMatchesExpected(
        type: ts.Type,
        expectedType: 'number' | 'string',
      ): boolean {
        if (expectedType === 'number') {
          return !!(type.flags & ts.TypeFlags.NumberLike)
        } else {
          return !!(type.flags & ts.TypeFlags.StringLike)
        }
      }

      // Create a fix that replaces the function call with its argument
      function createFix(
        node: TSESTree.CallExpression,
        arg: TSESTree.Expression,
      ): TSESLint.ReportFixFunction {
        return (fixer: TSESLint.RuleFixer): TSESLint.RuleFix => {
          return fixer.replaceText(node, context.sourceCode.getText(arg))
        }
      }

      function checkCastingCall(node: TSESTree.CallExpression) {
        if (!checker) return

        // Only check if there's exactly one argument
        if (node.arguments.length !== 1) return

        const arg = node.arguments[0]
        if (!arg || arg.type === AST_NODE_TYPES.SpreadElement) return

        const { callee } = node
        if (callee.type !== AST_NODE_TYPES.Identifier) return

        const functionName = callee.name

        // Find the cast function definition if it exists
        const castFunction = castFunctions.find(
          (fn) => fn.name === functionName,
        )
        if (!castFunction) return

        // Get the argument type
        const argTsNode = parserServices.esTreeNodeToTSNodeMap.get(arg)
        const argType = checker.getTypeAtLocation(argTsNode)

        // Check if the cast is unnecessary
        if (typeMatchesExpected(argType, castFunction.expectedType)) {
          // Use appropriate message ID based on function name
          let messageId:
            | 'unnecessaryNumberCasting'
            | 'unnecessaryStringCasting'
            | 'unnecessaryCustomCasting'
          let data: Record<string, string> | undefined

          if (functionName === 'Number') {
            messageId = 'unnecessaryNumberCasting'
          } else if (functionName === 'String') {
            messageId = 'unnecessaryStringCasting'
          } else {
            messageId = 'unnecessaryCustomCasting'
            data = {
              name: castFunction.name,
              type: castFunction.expectedType,
            }
          }

          context.report({
            node,
            messageId,
            ...(data ? { data } : {}),
            fix: createFix(node, arg),
          })
        }
      }

      return {
        CallExpression: checkCastingCall,
      }
    },
  }),
}
