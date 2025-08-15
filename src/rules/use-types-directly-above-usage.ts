import type { TSESLint } from '@typescript-eslint/utils'
import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

type TypeDefinition = {
  node: TSESTree.TSTypeAliasDeclaration | TSESTree.TSInterfaceDeclaration
  name: string
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

    function addTypeUsage(typeName: string, statement: TSESTree.Statement) {
      if (!typeDefinitions.has(typeName)) {
        return
      }

      const typeDef = typeDefinitions.get(typeName)!
      const typeDefIndex = programStatements.indexOf(
        typeDef.node as TSESTree.Statement,
      )
      const usageIndex = programStatements.indexOf(statement)

      if (typeDefIndex === -1 || usageIndex === -1) return

      // Check if type definition comes after usage or not directly before
      if (typeDefIndex > usageIndex || typeDefIndex !== usageIndex - 1) {
        context.report({
          node: typeDef.node,
          messageId: 'moveTypeAboveUsage',
          fix: createFixer(typeDef.node, statement),
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
      typeDefNode: TSESTree.Node,
      usageStatement: TSESTree.Statement,
    ) {
      return function* (fixer: TSESLint.RuleFixer) {
        const typeDefText = sourceCode.getText(typeDefNode)
        const typeDefComments = sourceCode.getCommentsBefore(typeDefNode)

        let fullTypeDefText = typeDefText
        if (typeDefComments.length > 0) {
          const commentsText = typeDefComments
            .map((comment) => sourceCode.getText(comment))
            .join('\n')
          fullTypeDefText = `${commentsText}\n${typeDefText}`
        }

        // Remove the type definition and any associated comments
        yield fixer.remove(typeDefNode)
        if (typeDefComments.length > 0) {
          for (const comment of typeDefComments) {
            yield fixer.remove(comment)
          }
        }

        // Replace the usage statement with type definition + usage statement
        const usageText = sourceCode.getText(usageStatement).trimEnd()
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
        typeDefinitions.set(node.id.name, { node, name: node.id.name })
      },

      TSInterfaceDeclaration(node) {
        typeDefinitions.set(node.id.name, { node, name: node.id.name })
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

        // Check each type that has usages
        for (const [typeName, usageStatements] of typeUsagesMap) {
          if (usageStatements.length > 0) {
            // Find the first usage (earliest in the file)
            const firstUsage = usageStatements.reduce((earliest, current) =>
              current.range[0] < earliest.range[0] ? current : earliest,
            )

            addTypeUsage(typeName, firstUsage)
          }
        }
      },
    }
  },
})
