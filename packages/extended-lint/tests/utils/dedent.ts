const indentRegex = /^(\s+)\S+/

export function dedent(strings: TemplateStringsArray, ...values: string[]) {
  // $FlowFixMe: Flow doesn't undestand .raw
  const raw = strings.raw

  // first, perform interpolation
  let result = ''
  for (let i = 0; i < raw.length; i++) {
    const rawValue = raw[i]
    if (rawValue === undefined) continue
    result += rawValue // join lines when there is a suppressed newline
      .replace(/\\\n[ \t]*/g, '')
      // handle escaped backticks
      .replace(/\\`/g, '`')

    if (i < values.length) {
      result += values[i]
    }
  }

  // now strip indentation
  const lines = result.split('\n')
  let mindent: number | null = null
  for (const l of lines) {
    const m = l.match(indentRegex)
    if (m) {
      const indentMatch = m[1]
      if (!indentMatch) continue
      const indent = indentMatch.length
      if (mindent === null) {
        // this is the first indented line
        mindent = indent
      } else {
        mindent = Math.min(mindent, indent)
      }
    }
  }

   
  if (mindent) {
    const m = mindent // appease Flow
    result = lines.map((l) => (l.startsWith(' ') ? l.slice(m) : l)).join('\n')
  }

  return (
    result
      // dedent eats leading and trailing whitespace too
      .trim()
      // handle escaped newlines at the end to ensure they don't get stripped too
      .replace(/\\n/g, '\n')
  )
}
