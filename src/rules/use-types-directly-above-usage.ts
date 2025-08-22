import type { TSESLint } from '@typescript-eslint/utils'
import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  checkOnly: z.array(z.enum(['function-args', 'FC'])).optional(),
  checkSelectedTypes: z.array(z.string()).optional(),
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

    function checkAndReportType(
      typeName: string,
      usageStatement: TSESTree.Statement,
    ) {
      if (!typeDefinitions.has(typeName)) {
        return
      }

      const typeDef = typeDefinitions.get(typeName)
      if (!typeDef) return
      if (typeDef.statement === usageStatement) return
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

        // Include trailing whitespace and newlines to avoid leaving blank lines
        let searchEnd = rangeEnd
        while (searchEnd < sourceCode.text.length) {
          const char = sourceCode.text[searchEnd]
          if (char === '\n') {
            searchEnd++
            // If we find a newline, check if the next line is blank
            let nextLineStart = searchEnd
            while (
              nextLineStart < sourceCode.text.length &&
              (sourceCode.text[nextLineStart] === ' ' ||
                sourceCode.text[nextLineStart] === '\t')
            ) {
              nextLineStart++
            }
            // If next line is blank (only whitespace followed by newline), include it
            if (
              nextLineStart < sourceCode.text.length &&
              sourceCode.text[nextLineStart] === '\n'
            ) {
              searchEnd = nextLineStart + 1
            }
            break
          } else if (char === ' ' || char === '\t') {
            searchEnd++
          } else {
            break
          }
        }

        rangeEnd = searchEnd

        // Remove the type definition, comments, and trailing newline
        yield fixer.removeRange([rangeStart, rangeEnd])

        // Insert the type definition before the usage statement (including before any comments)
        const usageComments = sourceCode.getCommentsBefore(usageStatement)
        const insertPosition =
          usageComments.length > 0 && usageComments[0]
            ? usageComments[0].range[0]
            : usageStatement.range[0]

        yield fixer.insertTextBeforeRange(
          [insertPosition, insertPosition],
          `${fullTypeDefText}\n\n`,
        )
      }
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

    const allTypeReferences: Array<{
      typeName: string
      node: TSESTree.TSTypeReference
      inFunctionArgs: boolean
      inFCProps: boolean
    }> = []

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
          allTypeReferences.push({
            typeName: node.typeName.name,
            node,
            inFunctionArgs: isInFunctionArgument(node),
            inFCProps: isInFCProps(node),
          })
        }
      },

      'Program:exit'() {
        const typeUsagesMap = new Map<string, TSESTree.Statement[]>()

        // Filter type references based on checkOnly option
        const filteredReferences = allTypeReferences.filter((typeRef) => {
          const { typeName } = typeRef

          // If no checkOnly option, include all references
          if (!options.checkOnly || options.checkOnly.length === 0) {
            return true
          }

          // Count total references for this type
          const allRefsForType = allTypeReferences.filter(
            (ref) => ref.typeName === typeName,
          )

          // If type is used multiple times, ignore when checkOnly is set
          if (allRefsForType.length > 1) {
            return false
          }

          // Check if this single reference is in the specified contexts
          for (const checkContext of options.checkOnly) {
            if (checkContext === 'function-args' && typeRef.inFunctionArgs) {
              return true
            }
            if (checkContext === 'FC' && typeRef.inFCProps) {
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
        }> = []

        for (const [typeName, usageStatements] of typeUsagesMap) {
          if (usageStatements.length > 0) {
            let firstUsage = usageStatements[0]
            if (!firstUsage) continue
            for (const current of usageStatements) {
              if (current.range[0] < firstUsage.range[0]) {
                firstUsage = current
              }
            }
            typesToProcess.push({ typeName, firstUsage })
          }
        }

        // Process types in order of their first usage position
        // Only report one violation at a time to avoid fix conflicts
        const sortedTypes = typesToProcess.sort(
          (a, b) => a.firstUsage.range[0] - b.firstUsage.range[0],
        )

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
