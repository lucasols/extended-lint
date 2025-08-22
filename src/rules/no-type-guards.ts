import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import * as z from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  alternativeMsgs: z
    .object({
      inArrayFind: z.string().optional(),
      inArrayFilter: z.string().optional(),
    })
    .optional(),
  __dev_simulateFileName: z.string().optional(),
})

function isTypeGuard(node: TSESTree.TSTypeAnnotation): boolean {
  if (node.typeAnnotation.type !== AST_NODE_TYPES.TSTypePredicate) {
    return false
  }

  return node.typeAnnotation.asserts === false
}

const typeGuardsFileRegex = /\.(typeGuards|type-guards)\.(ts|tsx)$/

function isInTypeGuardsFile(filename: string): boolean {
  return typeGuardsFileRegex.test(filename)
}

function isInFilterOrFind(
  node: TSESTree.Node,
): { method: 'filter' | 'find' } | null {
  let current = node.parent

  while (current) {
    if (
      current.type === AST_NODE_TYPES.CallExpression &&
      current.callee.type === AST_NODE_TYPES.MemberExpression &&
      current.callee.property.type === AST_NODE_TYPES.Identifier
    ) {
      const methodName = current.callee.property.name
      if (methodName === 'filter' || methodName === 'find') {
        return { method: methodName }
      }
    }
    current = current.parent
  }

  return null
}

type Options = z.infer<typeof optionsSchema>

export const noTypeGuards = createExtendedLintRule<
  [Options],
  'typeGuardNotAllowed' | 'useFilterWithTypeCheck' | 'useFindWithTypeCheck'
>({
  name: 'no-type-guards',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow type guards unless in *.typeGuards.(ts|tsx) or *.type-guards.(ts|tsx) files',
    },
    messages: {
      typeGuardNotAllowed:
        'Check if the type guard can be inferred by typescript, in most cases it can, e.g. `.filter((nullable) => nullable !== null)`. If not, type guards are only allowed in *.typeGuards.(ts|tsx) or *.type-guards.(ts|tsx) files',
      useFilterWithTypeCheck: '{{message}}',
      useFindWithTypeCheck: '{{message}}',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
    hasSuggestions: true,
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const filename = options.__dev_simulateFileName ?? context.filename

    if (isInTypeGuardsFile(filename)) return {}

    function handleTypeGuard(node: TSESTree.TSTypeAnnotation) {
      if (!isTypeGuard(node)) return

      const filterOrFind = isInFilterOrFind(node)

      if (options.alternativeMsgs && filterOrFind) {
        const shouldSuggest =
          filterOrFind.method === 'filter'
            ? options.alternativeMsgs.inArrayFilter
            : options.alternativeMsgs.inArrayFind

        if (shouldSuggest) {
          const messageId =
            filterOrFind.method === 'filter'
              ? ('useFilterWithTypeCheck' as const)
              : ('useFindWithTypeCheck' as const)

          context.report({
            node,
            messageId,
            data: { message: shouldSuggest },
          })
          return
        }
      }

      context.report({
        node,
        messageId: 'typeGuardNotAllowed',
      })
    }

    return {
      'FunctionDeclaration > :matches(TSTypeAnnotation)': handleTypeGuard,
      'ArrowFunctionExpression > :matches(TSTypeAnnotation)': handleTypeGuard,
      'MethodDefinition > FunctionExpression > :matches(TSTypeAnnotation)':
        handleTypeGuard,
    }
  },
})
