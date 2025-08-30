import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { noUnnecessaryDescribe } from './no-unnecessary-describe'

const { valid, invalid } = createNewTester(noUnnecessaryDescribe)

test('valid: test file without describe blocks', async () => {
  await valid(
    {
      code: dedent`
        test('should work correctly', () => {
          expect(true).toBe(true)
        })
        
        test('should handle edge case', () => {
          expect(false).toBe(false)
        })
      `,
      filename: 'example.tests.ts',
    }
  )
})

test('valid: test file with multiple describe blocks', async () => {
  await valid(
    {
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
    }
  )
})

test('valid: non-test file with single describe should be ignored', async () => {
  await valid(
    {
      code: dedent`
        describe('All tests', () => {
          test('should work', () => {
            expect(true).toBe(true)
          })
        })
      `,
      filename: 'example.ts',
    }
  )
})

test('valid: describe with matching ignoreWithDescription pattern', async () => {
  await valid(
    {
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
    }
  )
})

test('valid: describe with regex matching ignoreWithDescription pattern', async () => {
  await valid(
    {
      code: dedent`
        describe('E2E Test Suite', () => {
          test('should work', () => {
            expect(true).toBe(true)
          })
        })
      `,
      filename: 'e2e.tests.ts',
      options: [{ ignoreWithDescription: 'E2E|Integration' }],
    }
  )
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

  expect(result.output).toMatchInlineSnapshot(`
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

  expect(result.output).toMatchInlineSnapshot(`
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

  expect(result.output).toMatchInlineSnapshot(`
    "test('should work', () => {
        expect(true).toBe(true)
      })"
  `)
})

test('valid: mixed content with single describe', async () => {
  await valid(
    {
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
    }
  )
})

test('valid: single describe with top-level test', async () => {
  await valid(
    {
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
    }
  )
})

test('valid: describe with timeout option should be ignored', async () => {
  await valid(
    {
      code: dedent`
        describe('Tests with timeout', () => {
          test('should work', () => {
            expect(true).toBe(true)
          })
        }, 10000)
      `,
      filename: 'example.tests.ts',
    }
  )
})

test('valid: describe.concurrent should be ignored', async () => {
  await valid(
    {
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
    }
  )
})

test('valid: describe.skip should be ignored', async () => {
  await valid(
    {
      code: dedent`
        describe.skip('Skipped tests', () => {
          test('should work', () => {
            expect(true).toBe(true)
          })
        })
      `,
      filename: 'example.tests.ts',
    }
  )
})

test('valid: describe.only should be ignored', async () => {
  await valid(
    {
      code: dedent`
        describe.only('Only tests', () => {
          test('should work', () => {
            expect(true).toBe(true)
          })
        })
      `,
      filename: 'example.tests.ts',
    }
  )
})

test('valid: describe with additional options should be ignored', async () => {
  await valid(
    {
      code: dedent`
        describe('Tests', () => {
          test('should work', () => {
            expect(true).toBe(true)
          })
        }, { retry: 3 })
      `,
      filename: 'example.tests.ts',
    }
  )
})