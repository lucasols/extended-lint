import { AST_NODE_TYPES, TSESLint, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod'
import { traverseAST } from '../astUtils'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  runOnlyWithEnableCompilerDirective: z.boolean().optional(),
})

const hasEnableCompilerDirectiveRegex =
  /eslint +react-compiler\/react-compiler: +\["error/

const NUMERIC_PATTERN = /^\d+$/
const SPLIT_PATTERN = /[\s_-]+/
const CAMEL_CASE_PATTERN = /[a-z][A-Z]/
const COMPOUND_WORD_PATTERN = /^(.+?)([A-Z][a-z]+s?)$/
const ALREADY_CAMEL_CASE_PATTERN = /^[a-z][a-zA-Z]*$/

function hasUnstableValues(
  node: TSESTree.JSXElement | TSESTree.JSXFragment,
  sourceCode: TSESLint.SourceCode,
): { hasUnstable: boolean; problematicProps: string[] } {
  const problematicProps: string[] = []
  
  if (node.type === AST_NODE_TYPES.JSXElement) {
    for (const attr of node.openingElement.attributes) {
      if (attr.type === AST_NODE_TYPES.JSXAttribute && 
          attr.name.type === AST_NODE_TYPES.JSXIdentifier &&
          attr.value?.type === AST_NODE_TYPES.JSXExpressionContainer &&
          attr.value.expression.type !== AST_NODE_TYPES.JSXEmptyExpression) {
        
        const propName = attr.name.name
        const expression = attr.value.expression
        
        // Check if this prop contains unstable values
        let hasUnstableInProp = false
        
        traverseAST(
          expression,
          (current) => {
            switch (current.type) {
              case AST_NODE_TYPES.ObjectExpression:
              case AST_NODE_TYPES.ArrowFunctionExpression:
              case AST_NODE_TYPES.FunctionExpression:
              case AST_NODE_TYPES.ArrayExpression:
                hasUnstableInProp = true
                return true // Stop traversing
            }
            return false
          },
          sourceCode,
        )
        
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- hasUnstableInProp is modified by the traverseAST callback
        if (hasUnstableInProp) {
          problematicProps.push(propName)
        }
      }
    }
  }

  return { hasUnstable: problematicProps.length > 0, problematicProps }
}

function returnsJSX(
  node: TSESTree.ArrowFunctionExpression | TSESTree.FunctionExpression,
): boolean {
  if (
    node.body.type === AST_NODE_TYPES.JSXElement ||
    node.body.type === AST_NODE_TYPES.JSXFragment
  ) {
    return true
  }

  if (node.body.type === AST_NODE_TYPES.BlockStatement) {
    for (const stmt of node.body.body) {
      if (stmt.type === AST_NODE_TYPES.ReturnStatement && stmt.argument) {
        if (
          stmt.argument.type === AST_NODE_TYPES.JSXElement ||
          stmt.argument.type === AST_NODE_TYPES.JSXFragment
        ) {
          return true
        }
      }
    }
  }

  return false
}

function inferComponentName(
  callNode: TSESTree.CallExpression,
  params: TSESTree.Parameter[],
): string {
  const callee = callNode.callee

  if (callee.type === AST_NODE_TYPES.MemberExpression) {
    const object = callee.object

    if (object.type === AST_NODE_TYPES.Identifier) {
      const arrayName = object.name

      const singularized = singularize(arrayName)
      if (singularized !== arrayName) {
        return pascalCase(singularized)
      }
    }
  }

  if (params.length > 0) {
    const firstParam = params[0]
    if (firstParam && firstParam.type === AST_NODE_TYPES.Identifier) {
      return pascalCase(firstParam.name)
    }
  }

  return 'ListItem'
}

function singularize(word: string): string {
  const irregulars: Record<string, string> = {
    people: 'person',
    children: 'child',
    feet: 'foot',
    teeth: 'tooth',
    mice: 'mouse',
    men: 'man',
    women: 'woman',
  }

  const irregular = irregulars[word.toLowerCase()]
  if (irregular) return irregular

  // Handle compound words like userAccounts, productCategories, etc.
  // Look for known pluralization patterns at the end
  if (word.match(CAMEL_CASE_PATTERN)) { // camelCase detected
    // Try to find the last capitalized word
    const match = word.match(COMPOUND_WORD_PATTERN);
    if (match) {
      const prefix = match[1];
      const suffix = match[2];
      
      if (!prefix || !suffix) return singularizeSimple(word);
      
      const singularSuffix = singularizeSimple(suffix);
      
      if (singularSuffix !== suffix) {
        return prefix + singularSuffix;
      }
    }
  }

  return singularizeSimple(word)
}

function singularizeSimple(word: string): string {
  if (word.endsWith('ies')) return `${word.slice(0, -3)}y`
  if (word.endsWith('es') && word.length > 3) {
    return word.slice(0, -2)
  }
  if (word.endsWith('s') && word.length > 1) {
    return word.slice(0, -1)
  }
  
  return word
}

function pascalCase(str: string): string {
  // If already in camelCase, just capitalize the first letter
  if (str.match(ALREADY_CAMEL_CASE_PATTERN)) {
    return str.charAt(0).toUpperCase() + str.slice(1)
  }
  
  // Otherwise split and recombine
  return str
    .split(SPLIT_PATTERN)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join('')
}

function extractClosureVariables(
  jsxNode: TSESTree.JSXElement | TSESTree.JSXFragment,
  mapParams: string[],
  sourceCode: TSESLint.SourceCode,
): { name: string; type: string }[] {
  const closureVars: { name: string; type: string }[] = []
  const usedVars = new Set<string>()

  // Known built-in identifiers that should not be considered closure variables
  const builtIns = new Set([
    'console',
    'document',
    'window',
    'Math',
    'Object',
    'Array',
    'String',
    'Number',
    'Boolean',
    'undefined',
    'null',
  ])

  traverseAST(
    jsxNode,
    (node) => {
      if (
        node.type === AST_NODE_TYPES.Identifier &&
        !mapParams.includes(node.name) &&
        !builtIns.has(node.name) &&
        !NUMERIC_PATTERN.test(node.name)
      ) {
        const parent = node.parent

        // Skip object property keys (like 'padding' in {padding: 10})
        if (parent.type === AST_NODE_TYPES.Property && parent.key === node) {
          return false
        }

        // Skip object property names in member expressions (like 'id' in todo.id)
        if (
          parent.type === AST_NODE_TYPES.MemberExpression &&
          parent.property === node &&
          !parent.computed
        ) {
          return false
        }

        // Skip JSX tag names
        if (
          parent.type === AST_NODE_TYPES.JSXOpeningElement ||
          parent.type === AST_NODE_TYPES.JSXClosingElement
        ) {
          return false
        }

        // Skip JSX attribute names (like 'key' in key={...})
        if (
          parent.type === AST_NODE_TYPES.JSXAttribute &&
          parent.name.type === AST_NODE_TYPES.JSXIdentifier &&
          parent.name.name === node.name
        ) {
          return false
        }

        usedVars.add(node.name)
      }
      return false
    },
    sourceCode,
  )

  for (const varName of usedVars) {
    if (!mapParams.includes(varName)) {
      closureVars.push({ name: varName, type: 'unknown' })
    }
  }

  return closureVars
}

function generateComponent(
  componentName: string,
  jsxNode: TSESTree.JSXElement | TSESTree.JSXFragment,
  mapParams: TSESTree.Parameter[],
  closureVars: { name: string; type: string }[],
  sourceCode: TSESLint.SourceCode,
): string {
  const paramNames = mapParams.map((param) => {
    if (param.type === AST_NODE_TYPES.Identifier) {
      return param.name
    }
    return 'item'
  })

  const props = [
    ...paramNames.map((name, index) => ({
      name,
      type:
        index === 0
          ? `${componentName}Type`
          : index === 1
          ? 'number'
          : 'unknown',
    })),
    ...closureVars,
  ]

  const propsInterface = props
    .map((prop) => {
      let propType = prop.type
      if (prop.name.includes('on') && propType === 'any') {
        propType = 'unknown'
      }
      return `  ${prop.name}: ${propType};`
    })
    .join('\n')

  const propsDestructured = props.map((prop) => prop.name).join(', ')
  const jsxText = sourceCode.getText(jsxNode)
  const propsTypeName = `${componentName}Props`

  return `type ${propsTypeName} = {
${propsInterface}
};

const ${componentName}: FC<${propsTypeName}> = ({ ${propsDestructured} }) => {
  return (
    ${jsxText}
  );
};`
}

function generateMapReplacement(
  componentName: string,
  mapParams: TSESTree.Parameter[],
  closureVars: { name: string; type: string }[],
  keyProp: string | null,
): string {
  const paramNames = mapParams.map((param) => {
    if (param.type === AST_NODE_TYPES.Identifier) {
      return param.name
    }
    return 'item'
  })

  const props = [
    keyProp ? `key={${keyProp}}` : `key={${paramNames[0]}.id}`,
    ...paramNames.map((name) => `${name}={${name}}`),
    ...closureVars.map((v) => `${v.name}={${v.name}}`),
  ].filter(Boolean)

  return `<${componentName} ${props.join(' ')} />`
}

function inferComponentNameFromPush(
  jsxNode: TSESTree.JSXElement | TSESTree.JSXFragment,
): string {
  if (
    jsxNode.type === AST_NODE_TYPES.JSXElement &&
    jsxNode.openingElement.name.type === AST_NODE_TYPES.JSXIdentifier
  ) {
    const tagName = jsxNode.openingElement.name.name
    if (tagName !== 'div' && tagName !== 'span') {
      return pascalCase(tagName)
    }
  }

  return 'ListItem'
}

function generatePushReplacement(
  componentName: string,
  closureVars: { name: string; type: string }[],
  keyProp: string | null,
): string {
  const props = [
    keyProp ? `key={${keyProp}}` : '',
    ...closureVars.map((v) => `${v.name}={${v.name}}`),
  ].filter(Boolean)

  return `<${componentName}${props.length > 0 ? ` ${props.join(' ')}` : ''} />`
}

function findKeyProp(
  jsxNode: TSESTree.JSXElement,
  sourceCode: TSESLint.SourceCode,
): string | null {
  for (const attr of jsxNode.openingElement.attributes) {
    if (
      attr.type === AST_NODE_TYPES.JSXAttribute &&
      attr.name.type === AST_NODE_TYPES.JSXIdentifier &&
      attr.name.name === 'key' &&
      attr.value &&
      attr.value.type === AST_NODE_TYPES.JSXExpressionContainer &&
      attr.value.expression.type !== AST_NODE_TYPES.JSXEmptyExpression
    ) {
      return sourceCode.getText(attr.value.expression)
    }
  }
  return null
}

type Options = z.infer<typeof optionsSchema>

export const reactOptimizedCollections = createExtendedLintRule<
  [Options],
  'unstableValueInMap' | 'extractComponent'
>({
  name: 'react-optimized-collections',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Detect unstable values in map renders that prevent React Compiler optimization',
    },
    messages: {
      unstableValueInMap:
        'Unstable values in {{problematicProps}} prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.',
      extractComponent: 'Extract to {{componentName}} component',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
    hasSuggestions: true,
  },
  defaultOptions: [{ runOnlyWithEnableCompilerDirective: false }],
  create(context, [options]) {
    const { sourceCode } = context

    if (options.runOnlyWithEnableCompilerDirective) {
      let isEnabled = false
      for (const comment of sourceCode.getAllComments()) {
        if (hasEnableCompilerDirectiveRegex.test(comment.value)) {
          isEnabled = true
          break
        }
      }
      if (!isEnabled) return {}
    }

    return {
      CallExpression(node) {
        if (
          node.callee.type === AST_NODE_TYPES.MemberExpression &&
          node.callee.property.type === AST_NODE_TYPES.Identifier &&
          (node.callee.property.name === 'map' ||
            node.callee.property.name === 'push')
        ) {
          const methodName = node.callee.property.name

          if (methodName === 'map') {
            handleMapCall(node, sourceCode, context)
          } else {
            handlePushCall(node, sourceCode, context)
          }
        }
      },
    }
  },
})

function handleMapCall(
  node: TSESTree.CallExpression,
  sourceCode: TSESLint.SourceCode,
  context: TSESLint.RuleContext<'unstableValueInMap' | 'extractComponent', [Options]>,
) {
  const callback = node.arguments[0]
  if (
    !callback ||
    (callback.type !== AST_NODE_TYPES.ArrowFunctionExpression &&
      callback.type !== AST_NODE_TYPES.FunctionExpression)
  ) {
    return
  }

  if (!returnsJSX(callback)) return

  let jsxNode: TSESTree.JSXElement | TSESTree.JSXFragment | null = null

  if (
    callback.body.type === AST_NODE_TYPES.JSXElement ||
    callback.body.type === AST_NODE_TYPES.JSXFragment
  ) {
    jsxNode = callback.body
  } else if (
    callback.body.type === AST_NODE_TYPES.BlockStatement &&
    callback.body.body[0]?.type === AST_NODE_TYPES.ReturnStatement &&
    callback.body.body[0].argument &&
    (callback.body.body[0].argument.type === AST_NODE_TYPES.JSXElement ||
      callback.body.body[0].argument.type === AST_NODE_TYPES.JSXFragment)
  ) {
    jsxNode = callback.body.body[0].argument
  }

  if (!jsxNode) return

  const unstableCheck = hasUnstableValues(jsxNode, sourceCode)
  if (!unstableCheck.hasUnstable) return

  const componentName = inferComponentName(node, callback.params)
  const paramNames = callback.params.map((param) => {
    if (param.type === AST_NODE_TYPES.Identifier) {
      return param.name
    }
    return 'item'
  })

  const closureVars = extractClosureVariables(jsxNode, paramNames, sourceCode)
  const keyProp =
    jsxNode.type === AST_NODE_TYPES.JSXElement
      ? findKeyProp(jsxNode, sourceCode)
      : null

  const problematicPropsText = unstableCheck.problematicProps.length === 1 
    ? `prop "${unstableCheck.problematicProps[0]}"`
    : `props "${unstableCheck.problematicProps.join('", "')}"` 

  context.report({
    node: jsxNode,
    messageId: 'unstableValueInMap',
    data: { problematicProps: problematicPropsText },
    suggest: [
      {
        messageId: 'extractComponent',
        data: { componentName },
        fix: (fixer: TSESLint.RuleFixer) => {
          const componentCode = generateComponent(
            componentName,
            jsxNode,
            callback.params,
            closureVars,
            sourceCode,
          )

          const mapReplacement = generateMapReplacement(
            componentName,
            callback.params,
            closureVars,
            keyProp,
          )

          // Find the containing function to insert the component below it
          let containingFunction: TSESTree.Node = node

          // Traverse up to find the containing function declaration
          while (containingFunction.parent.type !== AST_NODE_TYPES.Program) {
            if (
              containingFunction.type === AST_NODE_TYPES.FunctionDeclaration ||
              (containingFunction.type === AST_NODE_TYPES.VariableDeclarator &&
                containingFunction.init &&
                (containingFunction.init.type ===
                  AST_NODE_TYPES.ArrowFunctionExpression ||
                  containingFunction.init.type ===
                    AST_NODE_TYPES.FunctionExpression))
            ) {
              break
            }
            containingFunction = containingFunction.parent
          }

          const fixes: TSESLint.RuleFix[] = []

          fixes.push(
            fixer.insertTextAfter(containingFunction, `\n\n${componentCode}`),
          )

          fixes.push(fixer.replaceText(jsxNode, mapReplacement))

          return fixes
        },
      },
    ],
  })
}

function handlePushCall(
  node: TSESTree.CallExpression,
  sourceCode: TSESLint.SourceCode,
  context: TSESLint.RuleContext<'unstableValueInMap' | 'extractComponent', [Options]>,
) {
  const firstArg = node.arguments[0]
  if (
    !firstArg ||
    (firstArg.type !== AST_NODE_TYPES.JSXElement &&
      firstArg.type !== AST_NODE_TYPES.JSXFragment)
  ) {
    return
  }

  const jsxNode = firstArg

  const unstableCheck = hasUnstableValues(jsxNode, sourceCode)
  if (!unstableCheck.hasUnstable) return

  const componentName = inferComponentNameFromPush(jsxNode)
  const closureVars = extractClosureVariables(jsxNode, [], sourceCode)
  const keyProp =
    jsxNode.type === AST_NODE_TYPES.JSXElement
      ? findKeyProp(jsxNode, sourceCode)
      : null

  const problematicPropsText = unstableCheck.problematicProps.length === 1 
    ? `prop "${unstableCheck.problematicProps[0]}"`
    : `props "${unstableCheck.problematicProps.join('", "')}"` 

  context.report({
    node: jsxNode,
    messageId: 'unstableValueInMap',
    data: { problematicProps: problematicPropsText },
    suggest: [
      {
        messageId: 'extractComponent',
        data: { componentName },
        fix: (fixer: TSESLint.RuleFixer) => {
          const componentCode = generateComponent(
            componentName,
            jsxNode,
            [],
            closureVars,
            sourceCode,
          )

          const pushReplacement = generatePushReplacement(
            componentName,
            closureVars,
            keyProp,
          )

          // Find the containing function to insert the component below it
          let containingFunction: TSESTree.Node = node

          // Traverse up to find the containing function declaration
          while (containingFunction.parent.type !== AST_NODE_TYPES.Program) {
            if (
              containingFunction.type === AST_NODE_TYPES.FunctionDeclaration ||
              (containingFunction.type === AST_NODE_TYPES.VariableDeclarator &&
                containingFunction.init &&
                (containingFunction.init.type ===
                  AST_NODE_TYPES.ArrowFunctionExpression ||
                  containingFunction.init.type ===
                    AST_NODE_TYPES.FunctionExpression))
            ) {
              break
            }
            containingFunction = containingFunction.parent
          }

          const fixes: TSESLint.RuleFix[] = []

          fixes.push(
            fixer.insertTextAfter(containingFunction, `\n\n${componentCode}`),
          )

          fixes.push(fixer.replaceText(jsxNode, pushReplacement))

          return fixes
        },
      },
    ],
  })
}
