import { TSESTree } from '@typescript-eslint/utils'

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
