import { AST_NODE_TYPES } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  allow: z.record(z.string(), z.array(z.string())).optional(),
})

type Options = z.infer<typeof optionsSchema>

export const noDataAttributesOnComponents = createExtendedLintRule<
  [Options],
  'noDataAttributesOnComponents'
>({
  name: 'no-data-attributes-on-components',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow data-* attributes on custom React components. Use regular props instead.',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
    messages: {
      noDataAttributesOnComponents:
        'Avoid using data-* attributes on custom components. Use regular props instead or apply data attributes to the underlying DOM element.',
    },
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const allowMap = options.allow ?? {}

    function isCustomComponent(name: string): boolean {
      const firstChar = name.charAt(0)
      return firstChar === firstChar.toUpperCase() && firstChar !== firstChar.toLowerCase()
    }

    function getComponentName(
      elementName:
        | { type: AST_NODE_TYPES.JSXIdentifier; name: string }
        | { type: AST_NODE_TYPES.JSXMemberExpression }
        | { type: AST_NODE_TYPES.JSXNamespacedName; namespace: { name: string }; name: { name: string } },
    ): { name: string; isCustom: boolean } | null {
      if (elementName.type === AST_NODE_TYPES.JSXIdentifier) {
        return { name: elementName.name, isCustom: isCustomComponent(elementName.name) }
      }

      if (elementName.type === AST_NODE_TYPES.JSXMemberExpression) {
        return { name: '', isCustom: true }
      }

      return { name: `${elementName.namespace.name}:${elementName.name.name}`, isCustom: false }
    }

    function isAttributeAllowed(componentName: string, attributeName: string): boolean {
      const allowedAttributes = allowMap[componentName]
      if (!allowedAttributes) return false
      return allowedAttributes.includes(attributeName)
    }

    return {
      JSXAttribute(node) {
        if (node.name.type !== AST_NODE_TYPES.JSXIdentifier) return

        const attributeName = node.name.name
        if (!attributeName.startsWith('data-')) return

        const componentInfo = getComponentName(node.parent.name)
        if (!componentInfo || !componentInfo.isCustom) return

        if (componentInfo.name && isAttributeAllowed(componentInfo.name, attributeName)) return

        context.report({
          node,
          messageId: 'noDataAttributesOnComponents',
        })
      },
    }
  },
})
