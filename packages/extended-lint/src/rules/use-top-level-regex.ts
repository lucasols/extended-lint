import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

function hasStatefulFlags(node: TSESTree.Literal): boolean {
  if (!('regex' in node)) return false

  const flags = node.regex.flags
  return flags.includes('g') || flags.includes('y')
}


function isInRegexConstructor(node: TSESTree.Node): boolean {
  const parent = node.parent
  return (
    parent?.type === AST_NODE_TYPES.NewExpression &&
    parent.callee.type === AST_NODE_TYPES.Identifier &&
    parent.callee.name === 'RegExp'
  )
}

export const useTopLevelRegex = createExtendedLintRule<
  [],
  'regexShouldBeTopLevel'
>({
  name: 'use-top-level-regex',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Require regex literals to be declared at the top level to avoid performance issues',
    },
    messages: {
      regexShouldBeTopLevel:
        'Move this regex literal to the top level to avoid creating new RegExp objects on each function call.',
    },
    schema: [],
    hasSuggestions: false,
  },
  defaultOptions: [],
  create(context) {
    function checkRegexLiteral(node: TSESTree.Literal) {
      if (hasStatefulFlags(node)) return

      if (isInRegexConstructor(node)) return

      context.report({
        node,
        messageId: 'regexShouldBeTopLevel',
      })
    }

    return {
      'FunctionDeclaration Literal[regex], FunctionExpression Literal[regex], ArrowFunctionExpression Literal[regex], MethodDefinition Literal[regex]': checkRegexLiteral,
    }
  },
})
