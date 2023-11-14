import { test, expect } from 'vitest'
// @ts-ignore
import { getCodeLine } from 'virtual:get-code-line'

test('getCodeLine', () => {
  expect(getCodeLine()).toBe(6)
  expect(getCodeLine()).toBe(7)
})
