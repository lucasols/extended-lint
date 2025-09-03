import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

function capitalize(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1)
}

function isUseStateCall(node: TSESTree.CallExpression): boolean {
  if (node.callee.type === AST_NODE_TYPES.Identifier) {
    return node.callee.name === 'useState'
  }

  if (
    node.callee.type === AST_NODE_TYPES.MemberExpression &&
    node.callee.object.type === AST_NODE_TYPES.Identifier &&
    node.callee.property.type === AST_NODE_TYPES.Identifier
  ) {
    return (
      node.callee.object.name === 'React' &&
      node.callee.property.name === 'useState'
    )
  }

  return false
}

const optionsSchema = z.object({
  ignorePrefixes: z.array(z.string()).optional(),
})

function stripPrefixes(name: string, prefixes: string[]): string {
  for (const prefix of prefixes) {
    if (name.startsWith(prefix)) {
      return name.slice(prefix.length)
    }
  }
  return name
}

function getExpectedSetterName(valueName: string): string {
  return `set${capitalize(valueName)}`
}

type Options = z.infer<typeof optionsSchema>

export const useStateSetterNaming = createExtendedLintRule<
  [Options],
  'incorrectSetterName' | 'useSuggestedName'
>({
  name: 'use-state-setter-naming',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Enforce consistent naming convention for useState destructuring where setter should be set{Value}',
    },
    messages: {
      incorrectSetterName:
        'useState setter should follow the pattern "set{{expectedName}}" with a "set" prefix but got "{{actualName}}"',
      useSuggestedName: 'Use "{{expectedSetterName}}" instead',
    },
    hasSuggestions: true,
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{ ignorePrefixes: [] }],
  create(context, [options]) {
    return {
      VariableDeclarator(node) {
        if (
          !node.init ||
          node.init.type !== AST_NODE_TYPES.CallExpression ||
          !isUseStateCall(node.init) ||
          node.id.type !== AST_NODE_TYPES.ArrayPattern ||
          node.id.elements.length !== 2
        ) {
          return
        }

        const [valueElement, setterElement] = node.id.elements

        if (
          !valueElement ||
          !setterElement ||
          valueElement.type !== AST_NODE_TYPES.Identifier ||
          setterElement.type !== AST_NODE_TYPES.Identifier
        ) {
          return
        }

        const valueName = valueElement.name
        const setterName = setterElement.name
        const ignorePrefixes = options.ignorePrefixes || []

        // Strip configured prefixes from both names
        const strippedValueName = stripPrefixes(valueName, ignorePrefixes)
        const strippedSetterName = stripPrefixes(setterName, ignorePrefixes)

        // Skip validation if value name becomes empty after stripping prefixes
        if (strippedValueName === '') {
          // For empty value names, just check if setter starts with 'set'
          if (!strippedSetterName.startsWith('set')) {
            const expectedSetterName = `set${capitalize(valueName)}`
            context.report({
              node: setterElement,
              messageId: 'incorrectSetterName',
              data: {
                expectedName: capitalize(valueName),
                actualName: setterName,
              },
              suggest: [
                {
                  messageId: 'useSuggestedName',
                  data: {
                    expectedSetterName,
                  },
                  fix: (fixer) => {
                    return fixer.replaceText(setterElement, expectedSetterName)
                  },
                },
              ],
            })
          }
          return
        }

        const expectedSetterName = getExpectedSetterName(strippedValueName)

        if (strippedSetterName !== expectedSetterName) {
          context.report({
            node: setterElement,
            messageId: 'incorrectSetterName',
            data: {
              expectedName: capitalize(strippedValueName),
              actualName: setterName,
            },
            suggest: [
              {
                messageId: 'useSuggestedName',
                data: {
                  expectedSetterName,
                },
                fix: (fixer) => {
                  return fixer.replaceText(setterElement, expectedSetterName)
                },
              },
            ],
          })
        }
      },
    }
  },
})
