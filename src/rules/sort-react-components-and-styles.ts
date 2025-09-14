import type { TSESLint } from '@typescript-eslint/utils'
import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'
import { traverseAST } from '../astUtils'

const optionsSchema = z.object({
  mainComponentRegex: z.string().optional(),
})

const PASCAL_CASE_RE = /^[A-Z][A-Za-z0-9]*$/

type ComponentEntry = {
  node: TSESTree.Statement
  name: string
  isMainComponent: boolean
  usedStyles: Set<string>
}

type StyleEntry = {
  node: TSESTree.Statement
  name: string
  firstUsagePosition?: number
}

type Options = z.infer<typeof optionsSchema>

export const sortReactComponentsAndStyles = createExtendedLintRule<
  [Options],
  'stylesShouldBeAboveUsage'
>({
  name: 'sort-react-components-and-styles',
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Sort React components and styles with styles placed above their first usage',
    },
    fixable: 'code',
    schema: [getJsonSchemaFromZod(optionsSchema)],
    messages: {
      stylesShouldBeAboveUsage:
        'Style definitions should be placed above the first component that uses them',
    },
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const sourceCode = context.sourceCode
    const mainComponentRegex = options.mainComponentRegex
      ? new RegExp(options.mainComponentRegex)
      : undefined

    const componentNames = new Set<string>()
    const aliasOf = new Map<string, string>()
    const components: ComponentEntry[] = []
    const styles: StyleEntry[] = []

    function isPascalCase(name: string): boolean {
      return PASCAL_CASE_RE.test(name)
    }

    function isWrapperCall(node: TSESTree.Node): boolean {
      if (node.type !== AST_NODE_TYPES.CallExpression) return false
      const callee = node.callee
      if (callee.type === AST_NODE_TYPES.Identifier) {
        return callee.name === 'memo' || callee.name === 'forwardRef'
      }
      if (callee.type === AST_NODE_TYPES.MemberExpression) {
        if (callee.property.type === AST_NODE_TYPES.Identifier) {
          return (
            callee.property.name === 'memo' ||
            callee.property.name === 'forwardRef'
          )
        }
      }
      return false
    }

    function returnsJSX(node: TSESTree.Expression | TSESTree.Node): boolean {
      if (
        node.type === AST_NODE_TYPES.JSXElement ||
        node.type === AST_NODE_TYPES.JSXFragment
      ) {
        return true
      }

      if (node.type === AST_NODE_TYPES.BlockStatement) {
        for (const stmt of node.body) {
          if (stmt.type === AST_NODE_TYPES.ReturnStatement && stmt.argument) {
            return returnsJSX(stmt.argument)
          }
        }
      }

      if (node.type === AST_NODE_TYPES.ArrayExpression) {
        for (const el of node.elements) {
          if (!el) continue
          if (
            el.type === AST_NODE_TYPES.JSXElement ||
            el.type === AST_NODE_TYPES.JSXFragment
          ) {
            return true
          }
        }
      }
      return false
    }

    function isHOCInitializer(node: TSESTree.Expression): boolean {
      if (node.type !== AST_NODE_TYPES.ArrowFunctionExpression) return false
      const body = node.body
      if (body.type === AST_NODE_TYPES.ArrowFunctionExpression) {
        const inner = body.body
        if (returnsJSX(inner)) return true
      }
      return false
    }

    function isStyledComponent(node: TSESTree.Expression): boolean {
      if (node.type === AST_NODE_TYPES.TaggedTemplateExpression) {
        const tag = node.tag
        if (tag.type === AST_NODE_TYPES.MemberExpression) {
          if (
            tag.object.type === AST_NODE_TYPES.Identifier &&
            tag.object.name === 'styled'
          ) {
            return true
          }
        }
        if (tag.type === AST_NODE_TYPES.CallExpression) {
          const callee = tag.callee
          if (
            callee.type === AST_NODE_TYPES.Identifier &&
            callee.name === 'styled'
          ) {
            return true
          }
        }
      }
      return false
    }

    function isCSSTemplate(node: TSESTree.Expression): boolean {
      if (node.type === AST_NODE_TYPES.TaggedTemplateExpression) {
        const tag = node.tag
        if (tag.type === AST_NODE_TYPES.Identifier && tag.name === 'css') {
          return true
        }
      }
      return false
    }

    function collectFromVariableDeclarator(
      decl: TSESTree.VariableDeclarator,
      statement: TSESTree.Statement,
    ) {
      if (decl.id.type !== AST_NODE_TYPES.Identifier) return
      const name = decl.id.name
      const init = decl.init
      if (!init) return

      if (isStyledComponent(init) || isCSSTemplate(init)) {
        styles.push({ node: statement, name })
        return
      }

      if (
        (isPascalCase(name) || init.type === AST_NODE_TYPES.ArrowFunctionExpression) &&
        (init.type === AST_NODE_TYPES.ArrowFunctionExpression ||
          init.type === AST_NODE_TYPES.FunctionExpression ||
          isWrapperCall(init))
      ) {
        if (init.type === AST_NODE_TYPES.ArrowFunctionExpression ||
            init.type === AST_NODE_TYPES.FunctionExpression) {
          if (returnsJSX(init.body)) {
            componentNames.add(name)
            return
          }
        } else if (isWrapperCall(init)) {
          componentNames.add(name)
          return
        }
      }

      if (init.type === AST_NODE_TYPES.Identifier) {
        aliasOf.set(name, init.name)
      }

      if (!isPascalCase(name)) {
        if (isHOCInitializer(init)) {
          componentNames.add(name)
        }
      }
    }

    function isExportedComponent(statement: TSESTree.Statement): boolean {
      if (statement.type === AST_NODE_TYPES.ExportDefaultDeclaration) {
        return true
      }
      if (statement.type === AST_NODE_TYPES.ExportNamedDeclaration) {
        return true
      }
      return false
    }

    function getComponentName(statement: TSESTree.Statement): string | null {
      if (statement.type === AST_NODE_TYPES.FunctionDeclaration) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- Required for AST traversal
        return statement.id ? statement.id.name : null
      }
      if (statement.type === AST_NODE_TYPES.VariableDeclaration) {
        const decl = statement.declarations[0]
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- Required for AST traversal
        if (decl && decl.id.type === AST_NODE_TYPES.Identifier) {
          return decl.id.name
        }
      }
      if (statement.type === AST_NODE_TYPES.ExportDefaultDeclaration) {
        const decl = statement.declaration
        if (decl.type === AST_NODE_TYPES.FunctionDeclaration) {
          return decl.id ? decl.id.name : 'default'
        }
        if (decl.type === AST_NODE_TYPES.Identifier) {
          return decl.name
        }
      }
      if (statement.type === AST_NODE_TYPES.ExportNamedDeclaration) {
        const decl = statement.declaration
        if (decl && decl.type === AST_NODE_TYPES.FunctionDeclaration) {
          return decl.id ? decl.id.name : null
        }
        if (decl && decl.type === AST_NODE_TYPES.VariableDeclaration) {
          const varDecl = decl.declarations[0]
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- Required for AST traversal
          if (varDecl && varDecl.id.type === AST_NODE_TYPES.Identifier) {
            return varDecl.id.name
          }
        }
      }
      return null
    }

    function isMainComponent(statement: TSESTree.Statement, name: string): boolean {
      if (mainComponentRegex) {
        return mainComponentRegex.test(name)
      }
      return isExportedComponent(statement)
    }

    function collectStyleUsages(component: ComponentEntry) {
      const usedStyles = new Set<string>()

      traverseAST(component.node, (node) => {
        if (node.type === AST_NODE_TYPES.Identifier) {
          for (const style of styles) {
            if (style.name === node.name) {
              usedStyles.add(style.name)

              if (style.firstUsagePosition === undefined ||
                  node.range[0] < style.firstUsagePosition) {
                style.firstUsagePosition = node.range[0]
              }
            }
          }
        }

        if (node.type === AST_NODE_TYPES.JSXElement) {
          const opening = node.openingElement
          if (opening.name.type === AST_NODE_TYPES.JSXIdentifier) {
            for (const style of styles) {
              if (style.name === opening.name.name) {
                usedStyles.add(style.name)

                if (style.firstUsagePosition === undefined ||
                    opening.name.range[0] < style.firstUsagePosition) {
                  style.firstUsagePosition = opening.name.range[0]
                }
              }
            }
          }
        }

        return false
      }, sourceCode)

      component.usedStyles = usedStyles
    }

    function collectTopLevel(statement: TSESTree.Statement) {
      if (statement.type === AST_NODE_TYPES.VariableDeclaration) {
        for (const d of statement.declarations) {
          collectFromVariableDeclarator(d, statement)
        }
        return
      }
      if (statement.type === AST_NODE_TYPES.FunctionDeclaration) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- Required for AST traversal
        if (statement.id && isPascalCase(statement.id.name) && returnsJSX(statement.body)) {
          componentNames.add(statement.id.name)
        }
        return
      }
      if (
        statement.type === AST_NODE_TYPES.ExportNamedDeclaration &&
        statement.declaration
      ) {
        const decl = statement.declaration
        if (decl.type === AST_NODE_TYPES.VariableDeclaration) {
          for (const d of decl.declarations) {
            collectFromVariableDeclarator(d, statement)
          }
          return
        }
        if (decl.type === AST_NODE_TYPES.FunctionDeclaration && decl.id) {
          if (isPascalCase(decl.id.name) && returnsJSX(decl.body)) {
            componentNames.add(decl.id.name)
          }
          return
        }
      }
      if (statement.type === AST_NODE_TYPES.ExportDefaultDeclaration) {
        const d = statement.declaration
        if (d.type === AST_NODE_TYPES.Identifier) {
          componentNames.add(d.name)
        }
        if (
          d.type === AST_NODE_TYPES.FunctionDeclaration ||
          d.type === AST_NODE_TYPES.ArrowFunctionExpression ||
          d.type === AST_NODE_TYPES.FunctionExpression
        ) {
          if (d.type === AST_NODE_TYPES.FunctionDeclaration) {
            if (d.id) {
              componentNames.add(d.id.name)
            } else {
              componentNames.add('default')
            }
          } else if (returnsJSX(d.body)) {
            componentNames.add('default')
          }
        }
      }
    }

    function resolvesToComponent(
      name: string,
      seen = new Set<string>(),
    ): boolean {
      if (componentNames.has(name)) return true
      if (seen.has(name)) return false
      seen.add(name)
      const aliased = aliasOf.get(name)
      if (!aliased) return false
      return resolvesToComponent(aliased, seen)
    }

    return {
      Program(node) {
        for (const statement of node.body) {
          collectTopLevel(statement)
        }

        for (const statement of node.body) {
          const name = getComponentName(statement)
          if (name && resolvesToComponent(name)) {
            const isMainComp = isMainComponent(statement, name)
            components.push({
              node: statement,
              name,
              isMainComponent: isMainComp,
              usedStyles: new Set(),
            })
          }
        }

        for (const component of components) {
          collectStyleUsages(component)
        }

        const stylesToMove: Array<{
          style: StyleEntry
          targetPosition: number
        }> = []

        for (const style of styles) {
          if (style.firstUsagePosition === undefined) continue

          const styledStatementIndex = node.body.indexOf(style.node)
          let firstUsingComponentIndex = -1

          for (const component of components) {
            if (!component.usedStyles.has(style.name)) continue
            const componentIndex = node.body.indexOf(component.node)
            if (
              firstUsingComponentIndex === -1 ||
              componentIndex < firstUsingComponentIndex
            ) {
              firstUsingComponentIndex = componentIndex
            }
          }

          if (firstUsingComponentIndex === -1) continue

          let previousComponentIndex = -1
          for (const component of components) {
            const idx = node.body.indexOf(component.node)
            if (idx < firstUsingComponentIndex && idx > previousComponentIndex) {
              previousComponentIndex = idx
            }
          }

          const isBeforeFirstUsing = styledStatementIndex < firstUsingComponentIndex
          const isAfterPreviousComponent = styledStatementIndex > previousComponentIndex

          const correctlyPlaced = isBeforeFirstUsing && isAfterPreviousComponent

          if (!correctlyPlaced) {
            stylesToMove.push({
              style,
              targetPosition: firstUsingComponentIndex,
            })
          }
        }

        if (stylesToMove.length > 0) {
          const firstStyle = stylesToMove[0]
          if (firstStyle) {
            context.report({
              node: firstStyle.style.node,
              messageId: 'stylesShouldBeAboveUsage',
              fix: createCombinedFixer(stylesToMove, node.body),
            })
          }
        }
      },
    }

    function createCombinedFixer(
      stylesToMove: Array<{
        style: StyleEntry
        targetPosition: number
      }>,
      programBody: TSESTree.Statement[],
    ) {
      return function* (fixer: TSESLint.RuleFixer) {
        stylesToMove.sort((a, b) => a.targetPosition - b.targetPosition)

        for (const { style } of stylesToMove) {
          const styleComments = sourceCode.getCommentsBefore(style.node)

          let rangeStart = style.node.range[0]
          const rangeEnd = style.node.range[1]

          if (styleComments.length > 0 && styleComments[0]) {
            rangeStart = styleComments[0].range[0]
          }

          if (rangeStart === style.node.range[0]) {
            const leadingText = sourceCode.text.slice(0, rangeStart)
            if (leadingText.trim().length === 0) {
              rangeStart = 0
            }
          }

          yield fixer.removeRange([rangeStart, rangeEnd])
        }

        const groupedByTarget = new Map<number, Array<typeof stylesToMove[0]>>()
        for (const item of stylesToMove) {
          const existing = groupedByTarget.get(item.targetPosition) || []
          existing.push(item)
          groupedByTarget.set(item.targetPosition, existing)
        }

        for (const [targetPos, stylesGroup] of groupedByTarget) {
          const targetStatement = programBody[targetPos]
          if (!targetStatement) continue

          const targetComments = sourceCode.getCommentsBefore(targetStatement)
          const insertPosition =
            targetComments.length > 0 && targetComments[0]
              ? targetComments[0].range[0]
              : targetStatement.range[0]

          const textsToInsert: string[] = []
          for (const { style } of stylesGroup) {
            const styleComments = sourceCode.getCommentsBefore(style.node)
            let fullStyleText = sourceCode.getText(style.node)

            if (styleComments.length > 0) {
              const commentsText = styleComments
                .map((comment) => sourceCode.getText(comment))
                .join('\n')
              fullStyleText = `${commentsText}\n${fullStyleText}`
            }
            textsToInsert.push(fullStyleText)
          }

          yield fixer.insertTextBeforeRange(
            [insertPosition, insertPosition],
            `${textsToInsert.join('\n\n')}\n\n`,
          )
        }
      }
    }
  },
})
