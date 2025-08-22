import {
  AST_NODE_TYPES,
  ESLintUtils,
  TSESLint,
  TSESTree,
} from '@typescript-eslint/utils'
import * as ts from 'typescript'
import z from 'zod/v4'
import { getJsonSchemaFromZod } from '../createRule'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-unnecessary-casting'

// Define options schema
const optionsSchema = z.object({
  additionalCastFunctions: z
    .array(
      z.object({
        name: z.string(),
        expectedType: z.enum(['string', 'number']),
      }),
    )
    .optional(),
})

type CastFunction = {
  name: string
  expectedType: 'string' | 'number'
}

type Options = z.infer<typeof optionsSchema>

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
      schema: [getJsonSchemaFromZod(optionsSchema)],
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

      // Helper to check if a type can be inferred from the AST without using the type checker
      function canInferTypeFromAST(
        node: TSESTree.Expression,
        expectedType: 'number' | 'string',
      ): boolean {
        switch (node.type) {
          case AST_NODE_TYPES.Literal:
            if (expectedType === 'number') {
              return typeof node.value === 'number'
            } else {
              return typeof node.value === 'string'
            }
          case AST_NODE_TYPES.TemplateLiteral:
            return expectedType === 'string'

          case AST_NODE_TYPES.UnaryExpression:
            // Unary numeric operators
            if (expectedType === 'number') {
              return (
                node.operator === '+' ||
                node.operator === '-' ||
                node.operator === '~'
              )
            }
            return false
          default:
            return false
        }
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

        // Fast path: Try to infer type from AST first
        const isTypeInferrable = canInferTypeFromAST(
          arg,
          castFunction.expectedType,
        )

        // If we can infer the type from AST or the type checker confirms it
        if (
          isTypeInferrable ||
          typeMatchesExpected(
            checker.getTypeAtLocation(
              parserServices.esTreeNodeToTSNodeMap.get(arg),
            ),
            castFunction.expectedType,
          )
        ) {
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
