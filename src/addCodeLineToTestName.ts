import { Plugin } from 'vite'

export function ViteAddCodeLineToTestNamePlugin(): Plugin {
  return {
    name: 'vite-plugin-add-code-line-to-test-name',
    enforce: 'pre',
    transform(code) {
      if (!code.includes('createTester(')) {
        return undefined
      }

      let finalCode = code

      const lines = code.split('\n')
      let addCodeLineToLine = false
      const foundTitles = new Set<string>()

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i]?.trim()

        if (!line) continue

        if (addCodeLineToLine) {
          if (line.startsWith("'")) {
            if (foundTitles.has(line)) {
              throw new Error(`Duplicate test title: ${line} in line ${i + 1}`)
            }

            foundTitles.add(line)
            finalCode = finalCode.replace(line, `':${i + 1} ' + ${line}`)
          }

          addCodeLineToLine = false
          continue
        }

        if (
          line.includes('.addValid(') ||
          line.includes('.addInvalid(') ||
          line.includes('.addInvalidWithOptions(')
        ) {
          addCodeLineToLine = true
          continue
        }
      }

      return finalCode
    },
  }
}
