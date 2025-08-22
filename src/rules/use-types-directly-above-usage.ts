import type { TSESLint } from '@typescript-eslint/utils'
import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  checkOnly: z
    .array(z.enum(['function-args', 'FC', 'generic-args-at-fn-calls']))
    .optional(),
  checkTypesFromSelectors: z.array(z.string()).optional(),
})

type Options = z.infer<typeof optionsSchema>

type TypeDefinition = {
  node: TSESTree.TSTypeAliasDeclaration | TSESTree.TSInterfaceDeclaration
  name: string
  statement: TSESTree.Statement
}

export const useTypesDirectlyAboveUsage = createExtendedLintRule<
  [Options],
  'moveTypeAboveUsage'
>({
  name: 'use-types-directly-above-usage',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Require type definitions to be placed directly above their first usage for better readability',
    },
    fixable: 'code',
    schema: [getJsonSchemaFromZod(optionsSchema)],
    messages: {
      moveTypeAboveUsage:
        'Type definition should be placed directly above its first usage.',
    },
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const sourceCode = context.sourceCode
    const typeDefinitions = new Map<string, TypeDefinition>()

    // Store all program-level statements for easy lookup
    const programStatements: TSESTree.Statement[] = []

    function findStatementContaining(
      node: TSESTree.Node,
    ): TSESTree.Statement | null {
      for (const statement of programStatements) {
        if (
          node.range[0] >= statement.range[0] &&
          node.range[1] <= statement.range[1]
        ) {
          return statement
        }
      }
      return null
    }

    function isInFunctionArgument(node: TSESTree.Node): boolean {
      let current = node.parent
      while (current) {
        if (
          current.type === AST_NODE_TYPES.FunctionDeclaration ||
          current.type === AST_NODE_TYPES.FunctionExpression ||
          current.type === AST_NODE_TYPES.ArrowFunctionExpression
        ) {
          // Check if the node is within the params array
          const params = current.params
          for (const param of params) {
            if (
              node.range[0] >= param.range[0] &&
              node.range[1] <= param.range[1]
            ) {
              return true
            }
          }
          break
        }
        current = current.parent
      }
      return false
    }

    function isInFCProps(node: TSESTree.Node): boolean {
      let current = node.parent
      while (current) {
        // Check if we're inside a TSTypeParameterInstantiation that belongs to FC<Props>
        if (current.type === AST_NODE_TYPES.TSTypeParameterInstantiation) {
          const parent = current.parent
          if (
            parent.type === AST_NODE_TYPES.TSTypeReference &&
            parent.typeName.type === AST_NODE_TYPES.Identifier &&
            (parent.typeName.name === 'FC' ||
              parent.typeName.name === 'React.FC')
          ) {
            return true
          }
          // Check for qualified name like React.FC
          if (
            parent.type === AST_NODE_TYPES.TSTypeReference &&
            parent.typeName.type === AST_NODE_TYPES.TSQualifiedName &&
            parent.typeName.right.name === 'FC'
          ) {
            return true
          }
        }
        current = current.parent
      }
      return false
    }

    function isInGenericArgAtFunctionCall(node: TSESTree.Node): boolean {
      let current = node.parent
      while (current) {
        // Check if we're inside a TSTypeParameterInstantiation
        if (current.type === AST_NODE_TYPES.TSTypeParameterInstantiation) {
          const parent = current.parent
          // Check if the parent is a CallExpression or NewExpression
          if (
            parent.type === AST_NODE_TYPES.CallExpression ||
            parent.type === AST_NODE_TYPES.NewExpression
          ) {
            return true
          }
        }
        current = current.parent
      }
      return false
    }

    const allTypeReferences: Array<{
      typeName: string
      node: TSESTree.TSTypeReference
      inFunctionArgs: boolean
      inFCProps: boolean
      inGenericArgAtFunctionCall: boolean
      usagePosition: number
    }> = []

    function checkAndReportAllTypes(
      typesToProcess: Array<{
        typeName: string
        firstUsage: TSESTree.Statement
        firstUsagePosition: number
      }>,
    ) {
      // Collect all types that actually need to be moved
      const typesToMove: Array<{
        typeName: string
        typeDef: TypeDefinition
        targetStatement: TSESTree.Statement
        firstUsagePosition: number
      }> = []

      // Group types by target statement to handle multiple types going to the same location
      const typesByTarget = new Map<TSESTree.Statement, string[]>()
      for (const { typeName, firstUsage } of typesToProcess) {
        const existing = typesByTarget.get(firstUsage) || []
        existing.push(typeName)
        typesByTarget.set(firstUsage, existing)
      }

      for (const {
        typeName,
        firstUsage,
        firstUsagePosition,
      } of typesToProcess) {
        const typeDef = typeDefinitions.get(typeName)
        if (!typeDef) continue
        if (typeDef.statement === firstUsage) continue

        const typeDefIndex = programStatements.indexOf(typeDef.statement)
        const usageIndex = programStatements.indexOf(firstUsage)

        if (typeDefIndex === -1 || usageIndex === -1) continue

        // Check if type definition comes after usage or not directly before
        // Be more lenient only for specific checkOnly contexts that need it
        const needsLenientPositioning =
          options.checkOnly &&
          options.checkOnly.length > 0 &&
          options.checkOnly.includes('function-args')

        // If multiple types are targeting the same statement, use lenient positioning
        const typesTargetingSameStatement = typesByTarget.get(firstUsage) || []
        const hasMultipleTypesForSameTarget = typesTargetingSameStatement.length > 1

        const shouldMove = needsLenientPositioning || hasMultipleTypesForSameTarget
          ? typeDefIndex > usageIndex // Just needs to be above
          : typeDefIndex > usageIndex || typeDefIndex !== usageIndex - 1 // Directly above otherwise

        if (shouldMove) {
          typesToMove.push({
            typeName,
            typeDef,
            targetStatement: firstUsage,
            firstUsagePosition,
          })
        }
      }

      if (typesToMove.length === 0) return

      // Sort by first usage position to maintain order as they appear in the code
      typesToMove.sort((a, b) => {
        // Primary sort: by first usage position
        const positionDiff = a.firstUsagePosition - b.firstUsagePosition
        if (positionDiff !== 0) {
          return positionDiff
        }
        // Secondary sort: by original definition position for stability
        const aIndex = programStatements.indexOf(a.typeDef.statement)
        const bIndex = programStatements.indexOf(b.typeDef.statement)
        return aIndex - bIndex
      })

      // Report with combined fix for all types
      const firstType = typesToMove[0]
      if (!firstType) return

      context.report({
        node: firstType.typeDef.node,
        messageId: 'moveTypeAboveUsage',
        fix: createCombinedFixer(typesToMove),
      })
    }

    function createCombinedFixer(
      typesToMove: Array<{
        typeName: string
        typeDef: TypeDefinition
        targetStatement: TSESTree.Statement
        firstUsagePosition: number
      }>,
    ) {
      return function* (fixer: TSESLint.RuleFixer) {
        // Group types by target statement
        const typesByTarget = new Map<
          TSESTree.Statement,
          Array<(typeof typesToMove)[0]>
        >()
        for (const typeToMove of typesToMove) {
          const existing = typesByTarget.get(typeToMove.targetStatement) || []
          existing.push(typeToMove)
          typesByTarget.set(typeToMove.targetStatement, existing)
        }

        // Sort each group by usage position to maintain order they appear in the code
        for (const [, groupedTypes] of typesByTarget) {
          groupedTypes.sort((a, b) => {
            // Primary sort: by first usage position
            const positionDiff = a.firstUsagePosition - b.firstUsagePosition
            if (positionDiff !== 0) {
              return positionDiff
            }
            // Secondary sort: by original definition position for absolute stability
            const aIndex = programStatements.indexOf(a.typeDef.statement)
            const bIndex = programStatements.indexOf(b.typeDef.statement)
            return aIndex - bIndex
          })
        }

        for (const typeToMove of typesToMove) {
          const { typeDef } = typeToMove
          const typeDefStatement = typeDef.statement
          const typeDefComments = sourceCode.getCommentsBefore(typeDefStatement)

          let rangeStart = typeDefStatement.range[0]
          const rangeEnd = typeDefStatement.range[1]

          if (typeDefComments.length > 0 && typeDefComments[0]) {
            rangeStart = typeDefComments[0].range[0]
          }

          yield fixer.removeRange([rangeStart, rangeEnd])
        }

        // Process insertions (group by target)
        for (const [targetStatement, groupedTypes] of typesByTarget) {
          const targetComments = sourceCode.getCommentsBefore(targetStatement)
          const insertPosition =
            targetComments.length > 0 && targetComments[0]
              ? targetComments[0].range[0]
              : targetStatement.range[0]

          // Collect text for all types going to this location
          const textsToInsert: string[] = []
          for (const typeToMove of groupedTypes) {
            const typeDefStatement = typeToMove.typeDef.statement
            const typeDefText = sourceCode.getText(typeDefStatement)
            const typeDefComments =
              sourceCode.getCommentsBefore(typeDefStatement)

            let fullTypeDefText = typeDefText
            if (typeDefComments.length > 0) {
              const commentsText = typeDefComments
                .map((comment) => sourceCode.getText(comment))
                .join('\n')
              fullTypeDefText = `${commentsText}\n${typeDefText}`
            }
            textsToInsert.push(fullTypeDefText)
          }

          yield fixer.insertTextBeforeRange(
            [insertPosition, insertPosition],
            `${textsToInsert.join('\n\n')}\n\n`,
          )
        }
      }
    }

    return {
      Program(node) {
        programStatements.push(...node.body)
      },

      TSTypeAliasDeclaration(node) {
        // Only process top-level type declarations (not nested inside functions)
        // Check if this type is a direct child of a program statement
        const isTopLevel = programStatements.some((stmt) => {
          if (
            stmt.type === AST_NODE_TYPES.ExportNamedDeclaration &&
            stmt.declaration === node
          ) {
            return true
          }
          return stmt === node
        })

        if (isTopLevel) {
          const statement = findStatementContaining(node)
          if (statement) {
            typeDefinitions.set(node.id.name, {
              node,
              name: node.id.name,
              statement,
            })
          }
        }
      },

      TSInterfaceDeclaration(node) {
        // Only process top-level type declarations (not nested inside functions)
        // Check if this type is a direct child of a program statement
        const isTopLevel = programStatements.some((stmt) => {
          if (
            stmt.type === AST_NODE_TYPES.ExportNamedDeclaration &&
            stmt.declaration === node
          ) {
            return true
          }
          return stmt === node
        })

        if (isTopLevel) {
          const statement = findStatementContaining(node)
          if (statement) {
            typeDefinitions.set(node.id.name, {
              node,
              name: node.id.name,
              statement,
            })
          }
        }
      },

      TSTypeReference(node) {
        if (node.typeName.type === AST_NODE_TYPES.Identifier) {
          const inFunctionArgs = isInFunctionArgument(node)
          const inFCProps = isInFCProps(node)
          const inGenericArgAtFunctionCall = isInGenericArgAtFunctionCall(node)

          allTypeReferences.push({
            typeName: node.typeName.name,
            node,
            inFunctionArgs,
            inFCProps,
            inGenericArgAtFunctionCall,
            usagePosition: node.range[0],
          })
        }
      },

      'Program:exit'() {
        const typeUsagesMap = new Map<string, TSESTree.Statement[]>()

        // Filter type references based on checkOnly option
        const filteredReferences = allTypeReferences.filter((typeRef) => {
          // If no checkOnly option, include all references
          if (!options.checkOnly || options.checkOnly.length === 0) {
            return true
          }

          // Check if this reference is in the specified contexts
          for (const checkContext of options.checkOnly) {
            if (checkContext === 'function-args' && typeRef.inFunctionArgs) {
              return true
            }
            if (checkContext === 'FC' && typeRef.inFCProps) {
              return true
            }
            if (
              checkContext === 'generic-args-at-fn-calls' &&
              typeRef.inGenericArgAtFunctionCall
            ) {
              return true
            }
          }

          return false
        })

        // Process filtered type references
        for (const typeRef of filteredReferences) {
          const { typeName, node } = typeRef

          if (typeDefinitions.has(typeName)) {
            const statement = findStatementContaining(node)
            const def = typeDefinitions.get(typeName)

            if (statement && def && statement !== def.statement) {
              if (!typeUsagesMap.has(typeName)) {
                typeUsagesMap.set(typeName, [])
              }

              const usages = typeUsagesMap.get(typeName)
              if (!usages) return
              if (!usages.includes(statement)) {
                usages.push(statement)
              }
            }
          }
        }

        // Check each type that has usages - process in order of first appearance
        const typesToProcess: Array<{
          typeName: string
          firstUsage: TSESTree.Statement
          firstUsagePosition: number
        }> = []

        for (const [typeName, usageStatements] of typeUsagesMap) {
          if (usageStatements.length > 0) {
            let firstUsage = usageStatements[0]
            if (!firstUsage) continue
            let firstUsagePosition = Number.MAX_SAFE_INTEGER

            // Find the first usage statement and the earliest position within it
            for (const current of usageStatements) {
              if (current.range[0] < firstUsage.range[0]) {
                firstUsage = current
              }
            }

            // Find the earliest usage position for this type within the first usage statement
            for (const typeRef of filteredReferences) {
              if (typeRef.typeName === typeName) {
                const statement = findStatementContaining(typeRef.node)
                if (
                  statement === firstUsage &&
                  typeRef.usagePosition < firstUsagePosition
                ) {
                  firstUsagePosition = typeRef.usagePosition
                }
              }
            }

            typesToProcess.push({ typeName, firstUsage, firstUsagePosition })
          }
        }

        // Process all types that need to be moved in a single fix
        if (typesToProcess.length > 0) {
          checkAndReportAllTypes(typesToProcess)
        }
      },
    }
  },
})
