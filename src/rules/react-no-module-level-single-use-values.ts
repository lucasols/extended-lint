import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { createExtendedLintRule } from '../createRule'

const reactFileExtensionRegex = /\.(tsx|jsx)$/
const pascalCaseRegex = /^[A-Z][A-Za-z0-9]*$/
const longStringLengthMin = 80

type ComponentFunctionNode =
  | TSESTree.ArrowFunctionExpression
  | TSESTree.FunctionDeclaration
  | TSESTree.FunctionExpression

export const reactNoModuleLevelSingleUseValues = createExtendedLintRule<
  [],
  'moveValueInsideComponent'
>({
  name: 'react-no-module-level-single-use-values',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow module-level allocated data values and long strings that are only used in one React component, excluding functions, inferable aliases, and regex values',
    },
    schema: [],
    messages: {
      moveValueInsideComponent:
        'Module-level "{{valueName}}" is only used in React component "{{componentName}}". Move it inside the component to improve memory usage.',
    },
  },
  defaultOptions: [],
  create(context) {
    if (!reactFileExtensionRegex.test(context.filename)) {
      return {}
    }

    const sourceCode = context.sourceCode
    const topLevelComponents = getTopLevelComponents(sourceCode.ast.body)

    if (topLevelComponents.length === 0) return {}

    return {
      VariableDeclarator(node) {
        if (node.id.type !== AST_NODE_TYPES.Identifier) return
        if (!isModuleLevelDeclarator(node)) return
        if (!isTargetInitializer(node.init)) return

        const valueName = node.id.name
        const declaredVariables = sourceCode.getDeclaredVariables(node)
        const variable = declaredVariables.find(
          (declaredVariable) => declaredVariable.name === valueName,
        )

        if (!variable) return

        const componentUsages = new Set<string>()
        let hasUsageOutsideComponents = false
        let hasUsage = false

        for (const reference of variable.references) {
          if (reference.init) continue
          hasUsage = true

          const componentName = getContainingComponentName(
            reference.identifier,
            topLevelComponents,
          )

          if (!componentName) {
            hasUsageOutsideComponents = true
            break
          }

          componentUsages.add(componentName)

          if (componentUsages.size > 1) break
        }

        if (!hasUsage) return
        if (hasUsageOutsideComponents) return
        if (componentUsages.size !== 1) return

        const [componentName] = Array.from(componentUsages)

        if (!componentName) return

        context.report({
          node: node.id,
          messageId: 'moveValueInsideComponent',
          data: {
            valueName,
            componentName,
          },
        })
      },
    }
  },
})

type TopLevelComponent = {
  name: string
  range: TSESTree.Range
}

function getTopLevelComponents(
  programBody: TSESTree.ProgramStatement[],
): TopLevelComponent[] {
  const components: TopLevelComponent[] = []
  const registeredNames = new Set<string>()

  function addComponent(name: string, functionNode: ComponentFunctionNode) {
    if (!pascalCaseRegex.test(name)) return
    if (registeredNames.has(name)) return

    registeredNames.add(name)
    components.push({ name, range: functionNode.range })
  }

  function addFromVariableDeclaration(node: TSESTree.VariableDeclaration) {
    for (const declaration of node.declarations) {
      if (declaration.id.type !== AST_NODE_TYPES.Identifier) continue

      const functionNode = getFunctionNodeFromInitializer(declaration.init)

      if (!functionNode) continue

      addComponent(declaration.id.name, functionNode)
    }
  }

  for (const node of programBody) {
    if (node.type === AST_NODE_TYPES.VariableDeclaration) {
      addFromVariableDeclaration(node)
      continue
    }

    if (node.type === AST_NODE_TYPES.FunctionDeclaration) {
      addComponent(node.id.name, node)
      continue
    }

    if (
      node.type === AST_NODE_TYPES.ExportNamedDeclaration &&
      node.declaration
    ) {
      if (node.declaration.type === AST_NODE_TYPES.VariableDeclaration) {
        addFromVariableDeclaration(node.declaration)
      } else if (
        node.declaration.type === AST_NODE_TYPES.FunctionDeclaration &&
        node.declaration.id
      ) {
        addComponent(node.declaration.id.name, node.declaration)
      }
    }
  }

  return components
}

function isModuleLevelDeclarator(node: TSESTree.VariableDeclarator): boolean {
  return node.parent.parent.type === AST_NODE_TYPES.Program
}

function getContainingComponentName(
  referenceNode: TSESTree.Node,
  components: TopLevelComponent[],
): string | null {
  let smallestContainingComponent: TopLevelComponent | null = null

  for (const component of components) {
    const isInsideComponent =
      referenceNode.range[0] >= component.range[0] &&
      referenceNode.range[1] <= component.range[1]

    if (!isInsideComponent) continue

    if (!smallestContainingComponent) {
      smallestContainingComponent = component
      continue
    }

    const smallestRangeSize =
      smallestContainingComponent.range[1] -
      smallestContainingComponent.range[0]
    const currentRangeSize = component.range[1] - component.range[0]

    if (currentRangeSize < smallestRangeSize) {
      smallestContainingComponent = component
    }
  }

  return smallestContainingComponent?.name ?? null
}

function getFunctionNodeFromInitializer(
  initializer: TSESTree.Expression | null,
): ComponentFunctionNode | null {
  if (!initializer) return null

  const unwrappedInitializer = unwrapExpression(initializer)

  if (
    unwrappedInitializer.type === AST_NODE_TYPES.ArrowFunctionExpression ||
    unwrappedInitializer.type === AST_NODE_TYPES.FunctionExpression
  ) {
    return unwrappedInitializer
  }

  if (
    unwrappedInitializer.type !== AST_NODE_TYPES.CallExpression ||
    !isComponentWrapperCallee(unwrappedInitializer.callee)
  ) {
    return null
  }

  const firstArgument = unwrappedInitializer.arguments[0]
  if (!firstArgument || firstArgument.type === AST_NODE_TYPES.SpreadElement) {
    return null
  }

  if (firstArgument.type === AST_NODE_TYPES.CallExpression) {
    return getFunctionNodeFromInitializer(firstArgument)
  }

  const unwrappedFirstArgument = unwrapExpression(firstArgument)

  if (
    unwrappedFirstArgument.type === AST_NODE_TYPES.ArrowFunctionExpression ||
    unwrappedFirstArgument.type === AST_NODE_TYPES.FunctionExpression
  ) {
    return unwrappedFirstArgument
  }

  return null
}

function isComponentWrapperCallee(callee: TSESTree.Expression) {
  if (callee.type === AST_NODE_TYPES.Identifier) {
    return callee.name === 'memo' || callee.name === 'forwardRef'
  }

  if (callee.type === AST_NODE_TYPES.MemberExpression) {
    return (
      callee.property.type === AST_NODE_TYPES.Identifier &&
      (callee.property.name === 'memo' || callee.property.name === 'forwardRef')
    )
  }

  return false
}

function unwrapExpression(expression: TSESTree.Expression): TSESTree.Expression {
  let currentExpression = expression

  while (true) {
    if (currentExpression.type === AST_NODE_TYPES.TSAsExpression) {
      currentExpression = currentExpression.expression
      continue
    }

    if (currentExpression.type === AST_NODE_TYPES.TSSatisfiesExpression) {
      currentExpression = currentExpression.expression
      continue
    }

    if (currentExpression.type === AST_NODE_TYPES.TSNonNullExpression) {
      currentExpression = currentExpression.expression
      continue
    }

    return currentExpression
  }
}

function isTargetInitializer(initializer: TSESTree.Expression | null): boolean {
  if (!initializer) return false

  const unwrappedInitializer = unwrapExpression(initializer)

  if (isInferableReferenceInitializer(unwrappedInitializer)) return false

  if (isLongString(unwrappedInitializer)) return true

  return isNonPrimitiveInitializer(unwrappedInitializer)
}

function isInferableReferenceInitializer(expression: TSESTree.Expression) {
  return (
    expression.type === AST_NODE_TYPES.Identifier
    || expression.type === AST_NODE_TYPES.MemberExpression
  )
}

function isLongString(expression: TSESTree.Expression): boolean {
  if (expression.type === AST_NODE_TYPES.Literal) {
    return (
      typeof expression.value === 'string' &&
      expression.value.length >= longStringLengthMin
    )
  }

  if (expression.type === AST_NODE_TYPES.TemplateLiteral) {
    if (expression.expressions.length > 0) return false

    const firstQuasi = expression.quasis[0]
    if (!firstQuasi) return false

    return firstQuasi.value.cooked.length >= longStringLengthMin
  }

  return false
}

function isNonPrimitiveInitializer(expression: TSESTree.Expression): boolean {
  if (expression.type === AST_NODE_TYPES.ArrayExpression) return true
  if (expression.type === AST_NODE_TYPES.ObjectExpression) return true
  if (expression.type === AST_NODE_TYPES.ClassExpression) return true
  if (expression.type === AST_NODE_TYPES.NewExpression) return true

  return false
}
