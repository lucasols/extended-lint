import { expect, test } from 'vitest'
// @ts-expect-error - getCodeLine is a virtual module
import { getCodeLine } from 'virtual:get-code-line'

test('getCodeLine', () => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call -- getCodeLine is untyped virtual module
  expect(getCodeLine()).toBe(7)
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call -- getCodeLine is untyped virtual module
  expect(getCodeLine()).toBe(9)
})
