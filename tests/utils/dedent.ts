export function dedent(strings: TemplateStringsArray, ...values: string[]) {
  // $FlowFixMe: Flow doesn't undestand .raw
  const raw = typeof strings === 'string' ? [strings] : strings.raw

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
  lines.forEach((l) => {
    const m = l.match(/^(\s+)\S+/)
    if (m) {
      const indentMatch = m[1]
      if (!indentMatch) return
      const indent = indentMatch.length
      if (mindent === null) {
        // this is the first indented line
        mindent = indent
      } else {
        mindent = Math.min(mindent, indent)
      }
    }
  })

  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
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
