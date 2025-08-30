import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
  getSuggestionOutput,
} from '../../tests/utils/createTester'
import { noUnnecessaryDescribe } from './no-unnecessary-describe'

const { valid, invalid } = createNewTester(noUnnecessaryDescribe)

test('valid: test file without describe blocks', async () => {
  await valid({
    code: dedent`
      test('should work correctly', () => {
        expect(true).toBe(true)
      })

      test('should handle edge case', () => {
        expect(false).toBe(false)
      })
    `,
    filename: 'example.tests.ts',
  })
})

test('valid: .test.ts file without describe blocks', async () => {
  await valid({
    code: dedent`
      test('should work correctly', () => {
        expect(true).toBe(true)
      })

      test('should handle edge case', () => {
        expect(false).toBe(false)
      })
    `,
    filename: 'example.test.ts',
  })
})

test('valid: test file with multiple describe blocks', async () => {
  await valid({
    code: dedent`
      describe('Component A', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })
      })

      describe('Component B', () => {
        test('should also work', () => {
          expect(true).toBe(true)
        })
      })
    `,
    filename: 'components.tests.ts',
  })
})

test('valid: describe with matching ignoreWithDescription pattern', async () => {
  await valid({
    code: dedent`
      describe('Integration Tests', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })

        test('should handle errors', () => {
          expect(false).toBe(false)
        })
      })
    `,
    filename: 'integration.tests.ts',
    options: [{ ignoreWithDescription: '^Integration' }],
  })
})

test('valid: describe with regex matching ignoreWithDescription pattern', async () => {
  await valid({
    code: dedent`
      describe('E2E Test Suite', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })
      })
    `,
    filename: 'e2e.tests.ts',
    options: [{ ignoreWithDescription: 'E2E|Integration' }],
  })
})

test('invalid: single describe wrapping all tests', async () => {
  const { result } = await invalid({
    code: dedent`
      describe('All tests', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })

        test('should handle errors', () => {
          expect(false).toBe(false)
        })
      })
    `,
    filename: 'example.tests.ts',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryDescribe', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "test('should work', () => {
        expect(true).toBe(true)
      })

    test('should handle errors', () => {
        expect(false).toBe(false)
      })"
  `)
})

test('invalid: single describe with nested structure', async () => {
  const { result } = await invalid({
    code: dedent`
      describe('Main suite', () => {
        beforeEach(() => {
          setup()
        })

        test('first test', () => {
          expect(1).toBe(1)
        })

        test('second test', () => {
          expect(2).toBe(2)
        })

        afterEach(() => {
          cleanup()
        })
      })
    `,
    filename: 'complex.tests.ts',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryDescribe', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "beforeEach(() => {
        setup()
      })

    test('first test', () => {
        expect(1).toBe(1)
      })

    test('second test', () => {
        expect(2).toBe(2)
      })

    afterEach(() => {
        cleanup()
      })"
  `)
})

test('invalid: should not ignore when description does not match pattern', async () => {
  const { result } = await invalid({
    code: dedent`
      describe('Unit Tests', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })
      })
    `,
    filename: 'unit.tests.ts',
    options: [{ ignoreWithDescription: '^Integration' }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryDescribe', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "test('should work', () => {
        expect(true).toBe(true)
      })"
  `)
})

test('invalid: mixed content with single describe', async () => {
  const { result } = await invalid({
    code: dedent`
      const helper = () => 'helper'

      describe('Tests', () => {
        test('should work', () => {
          expect(helper()).toBe('helper')
        })
      })

      const anotherHelper = () => 'another'
    `,
    filename: 'mixed.tests.ts',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryDescribe', line: 3 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const helper = () => 'helper'

    test('should work', () => {
        expect(helper()).toBe('helper')
      })

    const anotherHelper = () => 'another'"
  `)
})

test('valid: single describe with top-level test', async () => {
  await valid({
    code: dedent`
      test('top level test', () => {
        expect(true).toBe(true)
      })

      describe('Group A', () => {
        test('grouped test', () => {
          expect(true).toBe(true)
        })
      })
    `,
    filename: 'example.tests.ts',
  })
})

test('valid: describe with timeout option should be ignored', async () => {
  await valid({
    code: dedent`
      describe('Tests with timeout', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })
      }, 10000)
    `,
    filename: 'example.tests.ts',
  })
})

test('valid: describe.concurrent should be ignored', async () => {
  await valid({
    code: dedent`
      describe.concurrent('Concurrent tests', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })

        test('should also work', () => {
          expect(false).toBe(false)
        })
      })
    `,
    filename: 'example.tests.ts',
  })
})

test('valid: describe.skip should be ignored', async () => {
  await valid({
    code: dedent`
      describe.skip('Skipped tests', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })
      })
    `,
    filename: 'example.tests.ts',
  })
})

test('valid: describe.only should be ignored', async () => {
  await valid({
    code: dedent`
      describe.only('Only tests', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })
      })
    `,
    filename: 'example.tests.ts',
  })
})

test('valid: describe with additional options should be ignored', async () => {
  await valid({
    code: dedent`
      describe('Tests', () => {
        test('should work', () => {
          expect(true).toBe(true)
        })
      }, { retry: 3 })
    `,
    filename: 'example.tests.ts',
  })
})

test('invalid: single describe with helper functions inside', async () => {
  const { result } = await invalid({
    code: dedent`
      describe('conditionals', () => {
        function matches(value: number[], filter: any) {
          return value.includes(filter.value)
        }

        test('logged_user', () => {
          expect(matches([1, 2], { value: 1 })).toBe(true)
        })

        test('list / contains-any', () => {
          expect(matches([1, 2], { value: 3 })).toBe(false)
        })
      })
    `,
    filename: 'example.tests.ts',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryDescribe', line: 1 }
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "function matches(value: number[], filter: any) {
        return value.includes(filter.value)
      }

    test('logged_user', () => {
        expect(matches([1, 2], { value: 1 })).toBe(true)
      })

    test('list / contains-any', () => {
        expect(matches([1, 2], { value: 3 })).toBe(false)
      })"
  `)
})

