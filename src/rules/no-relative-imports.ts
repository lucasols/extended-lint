import { ESLintUtils } from '@typescript-eslint/utils'
import path from 'node:path'
import * as t from 'tschema'

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://github.com/lucasols/extended-lint#${name}`,
)

const name = 'no-relative-imports'

const aliasPatternSchema = t.object({
  find: t.string(),
  replacement: t.string(),
})

const optionsSchema = t.object({
  aliases: t.array(aliasPatternSchema),
  rootDir: t.optional(t.string()),
  allowNotFoundAliases: t.optional(t.boolean()),
  _dev_simulateFileName: t.optional(t.string()),
})

type Options = t.Infer<typeof optionsSchema>

const rule = createRule<
  [Options],
  'noRelativeImports' | 'noRelativeImportsWithAlias'
>({
  name,
  meta: {
    type: 'problem',
    fixable: 'code',
    docs: {
      description:
        'Prevent relative imports and auto-fix them using configured aliases',
    },
    messages: {
      noRelativeImports:
        'Relative imports are not allowed. Use one of the valid aliases in tsconfig.json instead.',
      noRelativeImportsWithAlias:
        'Relative imports are not allowed. Use the "{{ alias }}" alias instead.',
    },
    schema: [optionsSchema as any],
  },
  defaultOptions: [{ aliases: [], rootDir: undefined }],
  create(context, [options]) {
    const sourceFilePath = options._dev_simulateFileName ?? context.filename

    function isRelativePath(importPath: string): boolean {
      return importPath.startsWith('.') || importPath.startsWith('..')
    }

    function resolveRelativeToAbsolute(
      relativePath: string,
      currentFilePath: string,
    ): string {
      const currentDir = path.dirname(currentFilePath)
      return path.resolve(currentDir, relativePath)
    }

    function findMatchingAlias(
      absolutePath: string,
    ): { alias: string; newPath: string } | null {
      const rootDir = options.rootDir ?? process.cwd()

      let pathRelativeToRoot = path.relative(rootDir, absolutePath)

      if (pathRelativeToRoot.startsWith('.')) {
        return null
      }

      if (!pathRelativeToRoot.startsWith('/')) {
        pathRelativeToRoot = `/${pathRelativeToRoot}`
      }

      for (const { find: alias, replacement } of options.aliases) {
        if (pathRelativeToRoot.startsWith(replacement)) {
          const newPath = pathRelativeToRoot.replace(replacement, alias)
          return { alias, newPath }
        }
      }
      return null
    }

    return {
      ImportDeclaration(node) {
        const importPath = node.source.value

        if (!isRelativePath(importPath)) {
          return
        }

        const absolutePath = resolveRelativeToAbsolute(
          importPath,
          sourceFilePath,
        )
        const aliasMatch = findMatchingAlias(absolutePath)

        if (!aliasMatch && options.allowNotFoundAliases) return

        context.report({
          node,
          messageId: aliasMatch
            ? 'noRelativeImportsWithAlias'
            : 'noRelativeImports',
          data: {
            alias: aliasMatch?.alias,
          },
          fix: aliasMatch
            ? (fixer) => {
                return fixer.replaceText(node.source, `'${aliasMatch.newPath}'`)
              }
            : undefined,
        })
      },
    }
  },
})

export const noRelativeImports = {
  name,
  rule,
}
