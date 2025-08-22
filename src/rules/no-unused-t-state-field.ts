import { AST_NODE_TYPES, TSESLint, TSESTree } from '@typescript-eslint/utils'
import { getVarReferences, typedFind } from '../astUtils'
import { createExtendedLintRule } from '../createRule'

export const noUnusedTStateField = createExtendedLintRule<[], 'unusedField'>({
  name: 'no-unused-t-state-field',
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Prevent declaring unused t-state fields',
    },
    messages: {
      unusedField:
        'Field "{{name}}" is not being used, you can safely remove it',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    const isJsxFile =
      context.filename.endsWith('.jsx') || context.filename.endsWith('.tsx')

    if (!isJsxFile) return {}

    const hasTStateFormImport = context.sourceCode.ast.body.some(
      (node) =>
        node.type === AST_NODE_TYPES.ImportDeclaration &&
        node.source.value === 't-state-form',
    )

    if (!hasTStateFormImport) return {}

    let declaredFields: Map<string, TSESTree.Property> | null = null
    let ruleExecuted = false

    return {
      CallExpression(node) {
        if (ruleExecuted) return

        if (!declaredFields) {
          const fields = getDeclaredFields(node)

          if (fields) {
            declaredFields = fields
          }

          return
        }

        const usedFields = getUsedFields(node, context.sourceCode)

        if (!usedFields) return

        ruleExecuted = true

        for (const field of usedFields) {
          declaredFields.delete(field)
        }

        if (declaredFields.size === 0) return

        for (const [propName, property] of declaredFields) {
          context.report({
            node: property,
            messageId: 'unusedField',
            data: { name: propName },
          })
        }
      },
    }
  },
})

function getDeclaredFields(
  node: TSESTree.CallExpression,
): Map<string, TSESTree.Property> | null {
  const isUseForm =
    node.callee.type === AST_NODE_TYPES.Identifier &&
    node.callee.name === 'useForm'

  if (!isUseForm) return null

  const options = node.arguments[0]

  if (!options) return null

  if (options.type !== AST_NODE_TYPES.ObjectExpression) return null

  const initialConfig = typedFind(options.properties, (property) => {
    if (
      property.type !== AST_NODE_TYPES.Property ||
      property.key.type !== AST_NODE_TYPES.Identifier ||
      property.key.name !== 'initialConfig'
    ) {
      return null
    }

    if (property.value.type === AST_NODE_TYPES.ObjectExpression) {
      return property.value
    }

    if (
      property.value.type === AST_NODE_TYPES.ArrowFunctionExpression ||
      property.value.type === AST_NODE_TYPES.FunctionExpression
    ) {
      return getReturnObjectFromArrowFunction(property.value)
    }

    return null
  })

  if (!initialConfig) return null

  const declaredFields = new Map<string, TSESTree.Property>()

  for (const property of initialConfig.properties) {
    if (property.type !== AST_NODE_TYPES.Property) continue

    if (property.key.type !== AST_NODE_TYPES.Identifier) continue

    declaredFields.set(property.key.name, property)
  }

  return declaredFields
}

function getUsedFields(
  node: TSESTree.CallExpression,
  sourceCode: TSESLint.SourceCode,
) {
  const isUseFormState =
    node.callee.type === AST_NODE_TYPES.Identifier &&
    node.callee.name === 'useFormState'

  if (!isUseFormState) return null

  if (node.parent.type !== AST_NODE_TYPES.VariableDeclarator) return null

  if (node.parent.id.type !== AST_NODE_TYPES.ObjectPattern) return null

  const formFieldsRef = typedFind(node.parent.id.properties, (property) => {
    return (
      property.type === AST_NODE_TYPES.Property &&
      property.key.type === AST_NODE_TYPES.Identifier &&
      property.key.name === 'formFields' &&
      property
    )
  })

  if (!formFieldsRef) return null

  const references = getVarReferences(node.parent, 'formFields', sourceCode)

  const usedFields = new Set<string>()

  for (const { identifier } of references) {
    if (identifier.type !== AST_NODE_TYPES.Identifier) return null

    // is in object return
    if (
      identifier.parent.type === AST_NODE_TYPES.Property &&
      identifier.parent.parent.type === AST_NODE_TYPES.ObjectExpression &&
      identifier.parent.parent.parent.type === AST_NODE_TYPES.ReturnStatement
    ) {
      return null
    }

    // is directly returned
    if (identifier.parent.type === AST_NODE_TYPES.ReturnStatement) {
      return null
    }

    // is not property access
    if (identifier.parent.type !== AST_NODE_TYPES.MemberExpression) {
      continue
    }

    if (identifier.parent.object.type !== AST_NODE_TYPES.Identifier) return null

    if (identifier.parent.property.type !== AST_NODE_TYPES.Identifier)
      return null

    usedFields.add(identifier.parent.property.name)
  }

  return usedFields
}

function getReturnObjectFromArrowFunction(
  node: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression,
): TSESTree.ObjectExpression | null {
  if (node.body.type === AST_NODE_TYPES.ObjectExpression) {
    return node.body
  }

  if (node.body.type === AST_NODE_TYPES.BlockStatement) {
    const returnStatement = node.body.body.filter(
      (statement) => statement.type === AST_NODE_TYPES.ReturnStatement,
    )

    if (returnStatement.length !== 1 || !returnStatement[0]) return null

    if (returnStatement[0].argument?.type !== AST_NODE_TYPES.ObjectExpression)
      return null

    return returnStatement[0].argument
  }

  return null
}
