import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import * as z from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const defaultReactExtensions = ['tsx']

const optionsSchema = z.object({
  extensions: z.array(z.string()).optional(),
})

type Options = z.infer<typeof optionsSchema>

export const reactSingleExport = createExtendedLintRule<
  [Options],
  'multipleExports'
>({
  name: 'react-single-export',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Enforces only one export per React component file to support fast refresh',
    },
    messages: {
      multipleExports:
        'React component files should have only one export to support fast refresh. Type-only exports are allowed.',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const filename = context.filename
    const sourceCode = context.sourceCode
    const extensions = options.extensions || defaultReactExtensions
    
    function isReactComponentFile(): boolean {
      // Check if file has React-related extension
      const hasReactExtension = extensions.some(ext => filename.endsWith(`.${ext}`))
      if (!hasReactExtension) return false

      // Check if file contains JSX or React imports
      const text = sourceCode.text
      return (
        text.includes('jsx') ||
        text.includes('<') ||
        text.includes('React') ||
        text.includes('from \'react\'') ||
        text.includes('from "react"')
      )
    }

    if (!isReactComponentFile()) return {}

    const valueExports: TSESTree.Node[] = []

    return {
      ExportDefaultDeclaration(node) {
        valueExports.push(node)
        if (valueExports.length > 1) {
          context.report({
            node,
            messageId: 'multipleExports',
          })
        }
      },
      ExportNamedDeclaration(node) {
        // Skip type-only exports
        if (node.exportKind === 'type') return

        // Skip if all specifiers are type exports
        if (node.specifiers.length > 0) {
          const hasValueExport = node.specifiers.some(
            specifier => 
              specifier.exportKind !== 'type'
          )
          if (!hasValueExport) return
        }

        // Skip if declaration is type-only
        if (
          node.declaration &&
          (node.declaration.type === AST_NODE_TYPES.TSTypeAliasDeclaration ||
           node.declaration.type === AST_NODE_TYPES.TSInterfaceDeclaration)
        ) return

        valueExports.push(node)
        if (valueExports.length > 1) {
          context.report({
            node,
            messageId: 'multipleExports',
          })
        }
      },
    }
  },
})