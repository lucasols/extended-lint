import { Reference } from '@typescript-eslint/scope-manager'
import { AST_NODE_TYPES, TSESLint, TSESTree } from '@typescript-eslint/utils'

export function findParentNode<T extends TSESTree.AST_NODE_TYPES>(
  node: TSESTree.Node,
  type: T,
  maxDepth = Infinity,
): (TSESTree.Node & { type: T }) | undefined {
  if (maxDepth === 0) {
    return undefined
  }

  if (!node.parent) {
    return undefined
  }

  if (node.type === type) {
    // eslint-disable-next-line @typescript-eslint/consistent-type-assertions
    return node as TSESTree.Node & { type: T }
  }

  return findParentNode(
    node.parent,
    type,
    maxDepth === Infinity ? maxDepth : maxDepth - 1,
  )
}

export function* walkUp(node: TSESTree.Node): Generator<TSESTree.Node> {
  yield node

  if (node.parent) {
    yield* walkUp(node.parent)
  }
}

export function typedFind<T, R>(
  array: T[],
  predicate: (item: T) => R | false | undefined | null,
): R | undefined {
  for (const item of array) {
    const result = predicate(item)

    if (result !== undefined && result !== null && result !== false) {
      return result
    }
  }

  return undefined
}

export function getVarReferences(
  scopeNode: TSESTree.Node,
  varName: string,
  sourceCode: TSESLint.SourceCode,
): Reference[] {
  const variables = sourceCode.getDeclaredVariables(scopeNode)

  const variable = variables.find(
    (v) =>
      v.name === varName ||
      (v.identifiers[0]?.parent.type === AST_NODE_TYPES.Property &&
        v.identifiers[0]?.parent.key.type === AST_NODE_TYPES.Identifier &&
        v.identifiers[0]?.parent.key.name === varName),
  )

  if (!variable) return []

  return variable.references.filter((ref) => !ref.init)
}
