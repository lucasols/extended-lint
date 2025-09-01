import { AST_NODE_TYPES, TSESTree } from '@typescript-eslint/utils'
import * as z from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const defaultReactExtensions = ['tsx']
const PASCAL_CASE_RE = /^[A-Z][A-Za-z0-9]*$/

const optionsSchema = z.object({
  extensions: z.array(z.string()).optional(),
})

type Options = z.infer<typeof optionsSchema>

export const reactSingleExport = createExtendedLintRule<
  [Options],
  'multipleExports'
>({
  name: 'react-single-export',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Enforces only one export per React component file to support fast refresh',
    },
    messages: {
      multipleExports:
        'React component files should have only one export to support fast refresh, if you want to export multiple components, use a separate file for each component. Type-only exports are allowed.',
    },
    schema: [getJsonSchemaFromZod(optionsSchema)],
  },
  defaultOptions: [{}],
  create(context, [options]) {
    const filename = context.getFilename()
    const extensions = options.extensions ?? defaultReactExtensions

    let matches = false
    for (const ext of extensions) {
      if (filename.endsWith(ext)) {
        matches = true
        break
      }
    }

    if (!matches) return {}

    const sourceCode = context.sourceCode

    const componentNames = new Set<string>()
    const aliasOf = new Map<string, string>()

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

    function collectFromVariableDeclarator(decl: TSESTree.VariableDeclarator) {
      if (decl.id.type !== AST_NODE_TYPES.Identifier) return
      const name = decl.id.name
      const init = decl.init
      if (!init) return

      if (
        isPascalCase(name) &&
        (init.type === AST_NODE_TYPES.ArrowFunctionExpression ||
          init.type === AST_NODE_TYPES.FunctionExpression ||
          isWrapperCall(init))
      ) {
        componentNames.add(name)
        return
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

    function collectTopLevel(node: TSESTree.Node) {
      if (node.type === AST_NODE_TYPES.VariableDeclaration) {
        for (const d of node.declarations) collectFromVariableDeclarator(d)
        return
      }
      if (node.type === AST_NODE_TYPES.FunctionDeclaration && node.id) {
        if (isPascalCase(node.id.name)) componentNames.add(node.id.name)
        return
      }
      if (
        node.type === AST_NODE_TYPES.ExportNamedDeclaration &&
        node.declaration
      ) {
        const decl = node.declaration
        if (decl.type === AST_NODE_TYPES.VariableDeclaration) {
          for (const d of decl.declarations) collectFromVariableDeclarator(d)
          return
        }
        if (decl.type === AST_NODE_TYPES.FunctionDeclaration && decl.id) {
          if (isPascalCase(decl.id.name)) componentNames.add(decl.id.name)
          return
        }
      }
    }

    for (const node of sourceCode.ast.body) collectTopLevel(node)

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

    type ExportEntry = {
      node: TSESTree.Node
      isType: boolean
      isComponent: boolean
    }
    const exports: ExportEntry[] = []

    function pushExport(
      node: TSESTree.Node,
      isType: boolean,
      isComponent: boolean,
    ) {
      exports.push({ node, isType, isComponent })
    }

    for (const node of sourceCode.ast.body) {
      if (node.type === AST_NODE_TYPES.ExportNamedDeclaration) {
        const isType = node.exportKind === 'type'
        if (node.declaration) {
          const decl = node.declaration
          if (
            decl.type === AST_NODE_TYPES.TSTypeAliasDeclaration ||
            decl.type === AST_NODE_TYPES.TSInterfaceDeclaration
          ) {
            pushExport(decl, true, false)
            continue
          }
          if (decl.type === AST_NODE_TYPES.VariableDeclaration) {
            for (const d of decl.declarations) {
              if (d.id.type !== AST_NODE_TYPES.Identifier) continue
              const name = d.id.name
              const isComp = resolvesToComponent(name)
              pushExport(d, isType, isComp)
            }
            continue
          }
          if (decl.type === AST_NODE_TYPES.FunctionDeclaration) {
            const name = decl.id?.name
            const isComp = name
              ? resolvesToComponent(name) || isPascalCase(name)
              : false
            pushExport(decl, isType, isComp)
            continue
          }
          pushExport(decl, isType, false)
          continue
        }
        if (node.specifiers.length > 0) {
          for (const s of node.specifiers) {
            if (s.local.type !== AST_NODE_TYPES.Identifier) continue
            const localName = s.local.name
            const isComp = resolvesToComponent(localName)
            pushExport(s, isType, isComp)
          }
        }
        continue
      }
      if (node.type === AST_NODE_TYPES.ExportDefaultDeclaration) {
        const d = node.declaration
        if (d.type === AST_NODE_TYPES.Identifier) {
          const isComp = resolvesToComponent(d.name) || isPascalCase(d.name)
          pushExport(node, false, isComp)
          continue
        }
        if (
          d.type === AST_NODE_TYPES.FunctionDeclaration ||
          d.type === AST_NODE_TYPES.ArrowFunctionExpression ||
          d.type === AST_NODE_TYPES.FunctionExpression ||
          d.type === AST_NODE_TYPES.CallExpression
        ) {
          pushExport(node, false, true)
          continue
        }
        pushExport(node, false, false)
        continue
      }
    }

    let hasComponent = false
    for (const e of exports) {
      if (!e.isType && e.isComponent) {
        hasComponent = true
        break
      }
    }

    if (!hasComponent) return {}

    const valueExports: ExportEntry[] = []
    for (const e of exports) {
      if (!e.isType) valueExports.push(e)
    }

    let firstComponentIndex = -1
    for (const [idx, entry] of valueExports.entries()) {
      if (entry.isComponent) {
        firstComponentIndex = idx
        break
      }
    }

    if (firstComponentIndex === -1) return {}

    for (const [idx, entry] of valueExports.entries()) {
      if (idx === firstComponentIndex) continue
      context.report({ node: entry.node, messageId: 'multipleExports' })
      break
    }

    return {}
  },
})
