import { TSESTree } from '@typescript-eslint/utils'

export function findParentNode<T extends TSESTree.AST_NODE_TYPES>(
  node: TSESTree.Node,
  type: T,
): TSESTree.Node | undefined {
  if (!node.parent) {
    return undefined
  }

  if (node.type === type) {
    return node
  }

  return findParentNode(node.parent, type)
}

export function* walkUp(node: TSESTree.Node): Generator<TSESTree.Node> {
  yield node

  if (node.parent) {
    yield* walkUp(node.parent)
  }
}
