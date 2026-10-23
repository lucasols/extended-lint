import { Plugin } from 'vite'

export function ViteGetCodeLinePlugin(): Plugin {
  const virtualModuleId = 'virtual:get-code-line'

  const resolvedModuleId = `\0${virtualModuleId}`

  return {
    name: 'vite-plugin-get-code-line',
    enforce: 'pre',
    resolveId(id) {
      return id === virtualModuleId ? resolvedModuleId : undefined
    },
    load(id) {
      if (id === resolvedModuleId) {
        return `export function getCodeLine() { throw new Error('This function should not be called') }`
      }

      return undefined
    },
    transform(code) {
      if (code.includes(virtualModuleId) && code.includes(`getCodeLine()`)) {
        return replacegetCodeLineCalls(code)
      }

      return undefined
    },
  }
}

export function replacegetCodeLineCalls(code: string) {
  const codeLinesMatched: number[] = []

  const codeLines = code.split('\n')

  for (let i = 0; i < codeLines.length; i++) {
    const line = codeLines[i]

    if (!line) continue

    const toReplace = 'getCodeLine()'

    if (line.includes(toReplace)) {
      codeLinesMatched.push(i)
    }
  }

  let matchIndex = 0

  const newCode = code.replaceAll('getCodeLine()', () => {
    const line = codeLinesMatched[matchIndex]

    if (line === undefined) {
      throw new Error('Failed to replace all getCodeLine() calls')
    }

    matchIndex++

    return String(line + 1)
  })

  if (newCode.includes('getCodeLine()')) {
    throw new Error('Failed to replace all getCodeLine() calls')
  }

  return newCode
}
