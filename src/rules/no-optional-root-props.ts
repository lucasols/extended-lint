import { ESLintUtils, TSESLint, TSESTree } from '@typescript-eslint/utils'
import { findParentNode, walkUp } from '../astUtils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-optional-root-props'

type Options = []

type Declaration =
  | TSESTree.TSTypeAliasDeclaration
  | TSESTree.TSInterfaceDeclaration

const rule = createRule<Options, 'optionalNotAllowed' | 'suggestion'>({
  name,
  meta: {
    type: 'problem',
    docs: {
      description:
        'Prevents usage of optional properties at the root of a type if that type is a local type alias or interface referenced only once',
    },
    messages: {
      optionalNotAllowed:
        'Optional property "{{ propertyName }}" is not allowed on a local type used only once. Use `prop: undefined | ...` instead.',
      suggestion: 'Use `prop: undefined | ...` instead.',
    },
    hasSuggestions: true,
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    function isReferencedOnlyOnce(decl: Declaration): boolean {
      const variables = context.sourceCode.getDeclaredVariables(decl)
      const variable = variables[0]

      if (!variable) return false

      if (variable.references.length !== 1) return false

      const reference = variable.references[0]

      if (!reference) return false

      // checks if the reference is a function argument
      for (const node of walkUp(reference.identifier)) {
        if ('returnType' in node) {
          return false
        }

        if (node.type === TSESTree.AST_NODE_TYPES.ExportNamedDeclaration) {
          return false
        }

        const parent = node.parent

        if (!parent) return false

        if (
          parent.type ===
            TSESTree.AST_NODE_TYPES.TSTypeParameterInstantiation &&
          parent.parent.type === TSESTree.AST_NODE_TYPES.TSTypeReference
        ) {
          const parentIsFcType =
            parent.parent.typeName.type ===
              TSESTree.AST_NODE_TYPES.Identifier &&
            parent.parent.typeName.name === 'FC'

          if (parentIsFcType) {
            const varDecl = findParentNode(
              parent.parent.parent,
              TSESTree.AST_NODE_TYPES.VariableDeclaration,
              4,
            )

            if (!varDecl) return false

            const singleRef = getValidSingleRef(varDecl, context.sourceCode)

            if (!singleRef) return false

            const isMemo =
              singleRef.parent.type ===
                TSESTree.AST_NODE_TYPES.CallExpression &&
              singleRef.parent.callee.type ===
                TSESTree.AST_NODE_TYPES.Identifier &&
              singleRef.parent.callee.name === 'memo'

            return !isMemo
          }
        }

        if (node.type === TSESTree.AST_NODE_TYPES.AssignmentPattern) {
          return false
        }

        if (parent.type === TSESTree.AST_NODE_TYPES.ArrowFunctionExpression) {
          const fnDeclaration = findParentNode(
            parent,
            TSESTree.AST_NODE_TYPES.VariableDeclaration,
          )

          if (!fnDeclaration) return false

          return !!getValidSingleRef(fnDeclaration, context.sourceCode)
        }

        if (parent.type === TSESTree.AST_NODE_TYPES.FunctionDeclaration) {
          return !!getValidSingleRef(parent, context.sourceCode)
        }
      }

      return false
    }

    function reportOptionalProperty(
      member: TSESTree.TSPropertySignature,
    ): void {
      if (
        member.key.type !== TSESTree.AST_NODE_TYPES.Identifier ||
        !member.optional
      ) {
        return
      }

      context.report({
        node: member.key,
        messageId: 'optionalNotAllowed',
        data: { propertyName: member.key.name },
        suggest: [
          {
            messageId: 'suggestion',
            fix: (fixer) => {
              const parentProp = findParentNode(
                member,
                TSESTree.AST_NODE_TYPES.TSPropertySignature,
              )

              if (!parentProp) return null

              const parentPropText = context.sourceCode.getText(parentProp)

              return fixer.replaceText(
                parentProp,
                parentPropText.replace('?:', ': undefined |'),
              )
            },
          },
        ],
      })
    }

    return {
      TSTypeAliasDeclaration(node) {
        if (node.typeAnnotation.type !== TSESTree.AST_NODE_TYPES.TSTypeLiteral)
          return

        if (isExported(node) || !isReferencedOnlyOnce(node)) return

        for (const member of node.typeAnnotation.members) {
          if (member.type === TSESTree.AST_NODE_TYPES.TSPropertySignature) {
            reportOptionalProperty(member)
          }
        }
      },
      TSInterfaceDeclaration(node) {
        if (isExported(node) || !isReferencedOnlyOnce(node)) return

        for (const member of node.body.body) {
          if (member.type === TSESTree.AST_NODE_TYPES.TSPropertySignature) {
            reportOptionalProperty(member)
          }
        }
      },
      TSTypeReference(node) {
        if (
          node.typeName.type !== TSESTree.AST_NODE_TYPES.Identifier ||
          node.typeName.name !== 'FC' ||
          !node.typeArguments?.params[0]
        ) {
          return
        }

        const propsType = node.typeArguments.params[0]

        if (propsType.type !== TSESTree.AST_NODE_TYPES.TSTypeLiteral) {
          return
        }

        const varDecl = findParentNode(
          node.parent,
          TSESTree.AST_NODE_TYPES.VariableDeclaration,
          4,
        )

        if (!varDecl) return

        const singleRef = getValidSingleRef(varDecl, context.sourceCode)

        if (!singleRef) return

        const isMemo =
          singleRef.parent.type === TSESTree.AST_NODE_TYPES.CallExpression &&
          singleRef.parent.callee.type === TSESTree.AST_NODE_TYPES.Identifier &&
          singleRef.parent.callee.name === 'memo'

        if (isMemo) return

        for (const member of propsType.members) {
          if (member.type === TSESTree.AST_NODE_TYPES.TSPropertySignature) {
            reportOptionalProperty(member)
          }
        }
      },
    }
  },
})

function isExported(node: TSESTree.Node | undefined): boolean {
  if (!node) return false

  return (
    node.parent?.type === TSESTree.AST_NODE_TYPES.ExportNamedDeclaration ||
    node.parent?.type === TSESTree.AST_NODE_TYPES.ExportDefaultDeclaration
  )
}

function getValidSingleRef(
  varDecl: TSESTree.VariableDeclaration | TSESTree.FunctionDeclaration,
  sourceCode: TSESLint.SourceCode,
): TSESTree.Identifier | TSESTree.JSXIdentifier | undefined {
  if (isExported(varDecl)) return undefined

  let declarationIdentifier: TSESTree.Identifier | undefined

  if (varDecl.type === TSESTree.AST_NODE_TYPES.VariableDeclaration) {
    if (varDecl.declarations.length !== 1) return undefined

    if (
      varDecl.declarations[0].id.type === TSESTree.AST_NODE_TYPES.Identifier
    ) {
      declarationIdentifier = varDecl.declarations[0].id
    }
  } else {
    if (!varDecl.id) return undefined

    declarationIdentifier = varDecl.id
  }

  if (!declarationIdentifier) return undefined

  let scope = sourceCode.getScope(varDecl)

  if (
    varDecl.type === TSESTree.AST_NODE_TYPES.FunctionDeclaration &&
    scope.upper
  ) {
    scope = scope.upper
  }

  const scopeVar = scope.variables.find((v) =>
    v.identifiers.includes(declarationIdentifier),
  )

  if (!scopeVar) return undefined

  const varRefs = scopeVar.references.filter(
    (ref) => ref.identifier !== declarationIdentifier,
  )

  if (varRefs.length !== 1 || !varRefs[0]) return undefined

  if (isExported(varRefs[0].identifier.parent.parent)) return undefined

  return varRefs[0].identifier
}

export const noOptionalRootProps = {
  name,
  rule,
}
