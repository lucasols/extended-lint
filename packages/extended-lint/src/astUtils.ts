import { Reference } from '@typescript-eslint/scope-manager'
import { AST_NODE_TYPES, TSESLint, TSESTree } from '@typescript-eslint/utils'

export function findParentNode<T extends TSESTree.AST_NODE_TYPES>(
  node: TSESTree.Node,
  type: T,
  maxDepth = Infinity,
): (TSESTree.Node & { type: T }) | undefined {
  if (maxDepth === 0) return undefined

  if (!node.parent) return undefined

  if (node.type === type) {
    // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- it is fine to use type assertions here
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
        v.identifiers[0].parent.key.type === AST_NODE_TYPES.Identifier &&
        v.identifiers[0].parent.key.name === varName),
  )

  if (!variable) return []

  return variable.references.filter((ref) => !ref.init)
}

/**
 * Type guard to check if a value is a TSESTree Node
 */
// eslint-disable-next-line @ls-stack/no-type-guards -- Required for generic AST traversal
function isASTNode(value: unknown): value is TSESTree.Node {
  if (value === null || typeof value !== 'object') {
    return false
  }
  
  if (!('type' in value)) return false
  
  // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- Required for AST traversal
  const potentialNode = value as Record<string, unknown>
  return typeof potentialNode.type === 'string'
}

/**
 * Get child property safely from AST node
 */
function getChildProperty(node: TSESTree.Node, key: string): unknown {
  // eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- Required for AST traversal
  const nodeRecord = node as unknown as Record<string, unknown>
  return nodeRecord[key]
}

/**
 * Generic AST traversal function using visitorKeys
 */
export function traverseAST(
  node: TSESTree.Node,
  visitor: (node: TSESTree.Node) => boolean | void,
  sourceCode: TSESLint.SourceCode,
): void {
  const visitorKeys = sourceCode.visitorKeys
  const visited = new Set<TSESTree.Node>()

  function traverse(currentNode: TSESTree.Node): boolean {
    if (visited.has(currentNode)) return false
    visited.add(currentNode)

    // Call visitor function - if it returns true, stop traversing
    const shouldStop = visitor(currentNode)
    if (shouldStop === true) return true

    // Traverse child nodes using visitorKeys
    const keys = visitorKeys[currentNode.type]
    if (keys) {
      for (const key of keys) {
        const child = getChildProperty(currentNode, key)
        if (child) {
          if (Array.isArray(child)) {
            for (const item of child) {
              if (isASTNode(item)) {
                if (traverse(item)) return true
              }
            }
          } else if (isASTNode(child)) {
            if (traverse(child)) return true
          }
        }
      }
    }

    return false
  }

  traverse(node)
}

