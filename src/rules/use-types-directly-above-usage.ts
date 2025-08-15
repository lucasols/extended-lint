import type { TSESLint } from '@typescript-eslint/utils'
import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

type TypeDefinition = {
  node: TSESTree.TSTypeAliasDeclaration | TSESTree.TSInterfaceDeclaration
  name: string
  statement: TSESTree.Statement
}

export const useTypesDirectlyAboveUsage = createExtendedLintRule<
  [],
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
    schema: [],
    messages: {
      moveTypeAboveUsage:
        'Type definition should be placed directly above its first usage.',
    },
  },
  defaultOptions: [],
  create(context) {
    const sourceCode = context.sourceCode
    const typeDefinitions = new Map<string, TypeDefinition>()

    // Store all program-level statements for easy lookup
    const programStatements: TSESTree.Statement[] = []

    function checkAndReportType(typeName: string, usageStatement: TSESTree.Statement) {
      if (!typeDefinitions.has(typeName)) {
        return
      }

      const typeDef = typeDefinitions.get(typeName)!
      const typeDefIndex = programStatements.indexOf(typeDef.statement)
      const usageIndex = programStatements.indexOf(usageStatement)

      if (typeDefIndex === -1 || usageIndex === -1) return

      // Check if type definition comes after usage or not directly before
      if (typeDefIndex > usageIndex || typeDefIndex !== usageIndex - 1) {
        context.report({
          node: typeDef.node,
          messageId: 'moveTypeAboveUsage',
          fix: createFixer(typeDef.statement, usageStatement),
        })
      }
    }

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

    function createFixer(
      typeDefStatement: TSESTree.Statement,
      usageStatement: TSESTree.Statement,
    ) {
      return function* (fixer: TSESLint.RuleFixer) {
        const typeDefText = sourceCode.getText(typeDefStatement)
        const typeDefComments = sourceCode.getCommentsBefore(typeDefStatement)

        let fullTypeDefText = typeDefText
        if (typeDefComments.length > 0) {
          const commentsText = typeDefComments
            .map((comment) => sourceCode.getText(comment))
            .join('\n')
          fullTypeDefText = `${commentsText}\n${typeDefText}`
        }

        // Calculate removal range including comments and trailing newline
        let rangeStart = typeDefStatement.range[0]
        let rangeEnd = typeDefStatement.range[1]

        // Include comments before the type definition
        if (typeDefComments.length > 0 && typeDefComments[0]) {
          rangeStart = typeDefComments[0].range[0]
        }

        // Include trailing newline if present
        const nextChar = sourceCode.text[rangeEnd]
        if (nextChar === '\n') {
          rangeEnd += 1
        }

        // Remove the type definition, comments, and trailing newline
        yield fixer.removeRange([rangeStart, rangeEnd])

        // Replace the usage statement with: typeDefinition + newlines + usageStatement
        const usageText = sourceCode.getText(usageStatement)
        yield fixer.replaceText(
          usageStatement,
          `${fullTypeDefText}\n\n${usageText}`,
        )
      }
    }

    const allTypeReferences: Array<{
      typeName: string
      node: TSESTree.TSTypeReference
    }> = []

    return {
      Program(node) {
        programStatements.push(...node.body)
      },

      TSTypeAliasDeclaration(node) {
        // Find the top-level statement that contains this type declaration
        const statement = findStatementContaining(node) || node as TSESTree.Statement
        typeDefinitions.set(node.id.name, { node, name: node.id.name, statement })
      },

      TSInterfaceDeclaration(node) {
        // Find the top-level statement that contains this type declaration
        const statement = findStatementContaining(node) || node as TSESTree.Statement
        typeDefinitions.set(node.id.name, { node, name: node.id.name, statement })
      },

      TSTypeReference(node) {
        if (node.typeName.type === AST_NODE_TYPES.Identifier) {
          allTypeReferences.push({ typeName: node.typeName.name, node })
        }
      },

      'Program:exit'() {
        const typeUsagesMap = new Map<string, TSESTree.Statement[]>()

        // Process all type references after type definitions are collected
        for (const typeRef of allTypeReferences) {
          const { typeName, node } = typeRef

          if (typeDefinitions.has(typeName)) {
            const statement = findStatementContaining(node)

            if (statement) {
              if (!typeUsagesMap.has(typeName)) {
                typeUsagesMap.set(typeName, [])
              }

              const usages = typeUsagesMap.get(typeName)!
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
        }> = []

        for (const [typeName, usageStatements] of typeUsagesMap) {
          if (usageStatements.length > 0) {
            // Find the first usage (earliest in the file)
            const firstUsage = usageStatements.reduce((earliest, current) =>
              current.range[0] < earliest.range[0] ? current : earliest,
            )
            typesToProcess.push({ typeName, firstUsage })
          }
        }

        // Process types in order of their first usage position
        // Only report one violation at a time to avoid fix conflicts
        const sortedTypes = typesToProcess
          .sort((a, b) => a.firstUsage.range[0] - b.firstUsage.range[0])
        
        if (sortedTypes.length > 0) {
          const firstType = sortedTypes[0]
          if (firstType) {
            checkAndReportType(firstType.typeName, firstType.firstUsage)
          }
        }
      },
    }
  },
})
