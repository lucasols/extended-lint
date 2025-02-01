import { ESLintUtils, TSESTree } from '@typescript-eslint/utils'
import { findParentNode, walkUp } from '../astUtils'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-optional-root-props'

type Options = []

type Declaration =
  | TSESTree.TSTypeAliasDeclaration
  | TSESTree.TSInterfaceDeclaration

const rule = createRule<Options, 'optionalNotAllowed'>({
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
    },
    schema: [],
    fixable: 'code',
  },
  defaultOptions: [],
  create(context) {
    function isExported(decl: Declaration): boolean {
      return decl.parent.type === TSESTree.AST_NODE_TYPES.ExportNamedDeclaration
    }

    function isReferencedOnlyOnce(decl: Declaration): boolean {
      const variables = context.sourceCode.getDeclaredVariables(decl)
      const variable = variables[0]

      if (!variable) return false

      if (variable.references.length !== 1) return false

      const reference = variable.references[0]

      if (!reference) return false

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

            if (varDecl) {
              const isExportedVar =
                varDecl.parent.type ===
                TSESTree.AST_NODE_TYPES.ExportNamedDeclaration

              return !isExportedVar
            }
          }
        }

        if (node.type === TSESTree.AST_NODE_TYPES.AssignmentPattern) {
          return false
        }

        if (parent.type === TSESTree.AST_NODE_TYPES.ArrowFunctionExpression) {
          const fnIsExported = findParentNode(
            parent,
            TSESTree.AST_NODE_TYPES.VariableDeclaration,
          )

          if (
            fnIsExported?.parent &&
            fnIsExported.parent.type ===
              TSESTree.AST_NODE_TYPES.ExportNamedDeclaration
          ) {
            return false
          }

          return parent.params.some((param) => param === node)
        }

        if (parent.type === TSESTree.AST_NODE_TYPES.FunctionDeclaration) {
          const fnIsExported =
            parent.parent.type ===
            TSESTree.AST_NODE_TYPES.ExportNamedDeclaration

          if (fnIsExported) return false

          return parent.params.some((param) => param === node)
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
    }
  },
})

export const noOptionalRootProps = {
  name,
  rule,
}
