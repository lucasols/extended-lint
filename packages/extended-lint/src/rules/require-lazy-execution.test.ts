import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { requireLazyExecution } from './require-lazy-execution'

const { valid, invalid } = createNewTester(requireLazyExecution)

test('allows function call inside function declaration', async () => {
  await valid({
    code: dedent`
      function foo() {
        dangerousFunction()
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside arrow function', async () => {
  await valid({
    code: dedent`
      const foo = () => {
        dangerousFunction()
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside method', async () => {
  await valid({
    code: dedent`
      class Foo {
        bar() {
          dangerousFunction()
        }
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside getter', async () => {
  await valid({
    code: dedent`
      class Foo {
        get bar() {
          return dangerousFunction()
        }
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside class property arrow function', async () => {
  await valid({
    code: dedent`
      class Foo {
        bar = () => {
          dangerousFunction()
        }
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside callback', async () => {
  await valid({
    code: dedent`
      setTimeout(() => {
        dangerousFunction()
      }, 1000)
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside promise callback', async () => {
  await valid({
    code: dedent`
      promise.then(() => {
        dangerousFunction()
      })
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside nested functions', async () => {
  await valid({
    code: dedent`
      function outer() {
        function inner() {
          dangerousFunction()
        }
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows tagged template inside function', async () => {
  await valid({
    code: dedent`
      function query() {
        return sql\`SELECT * FROM users\`
      }
    `,
    options: [{ functions: ['sql'] }],
  })
})

test('allows tagged template inside arrow function', async () => {
  await valid({
    code: dedent`
      const query = () => sql\`SELECT * FROM users\`
    `,
    options: [{ functions: ['sql'] }],
  })
})

test('allows tagged template inside method', async () => {
  await valid({
    code: dedent`
      class DB {
        query() {
          return sql\`SELECT * FROM users\`
        }
      }
    `,
    options: [{ functions: ['sql'] }],
  })
})

test('allows tagged template inside callback', async () => {
  await valid({
    code: dedent`
      const queries = items.map(item => sql\`SELECT * FROM \${item}\`)
    `,
    options: [{ functions: ['sql'] }],
  })
})

test('allows member expression tagged template inside function', async () => {
  await valid({
    code: dedent`
      function query() {
        return db.sql\`SELECT * FROM users\`
      }
    `,
    options: [{ functions: ['sql'] }],
  })
})

test('allows function call in function expression', async () => {
  await valid({
    code: dedent`
      const foo = function() {
        dangerousFunction()
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows member expression call inside function', async () => {
  await valid({
    code: dedent`
      function foo() {
        fs.readFileSync('file.txt')
      }
    `,
    options: [{ functions: ['readFileSync'] }],
  })
})

test('disallows function call at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows member expression call at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      fs.readFileSync('file.txt')
    `,
    options: [{ functions: ['readFileSync'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows tagged template at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const query = sql\`SELECT * FROM users\`
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('allows multiple function names in config', async () => {
  await valid({
    code: dedent`
      function foo() {
        dangerousFunction()
        anotherDangerous()
      }
    `,
    options: [{ functions: ['dangerousFunction', 'anotherDangerous'] }],
  })
})

test('disallows multiple function names at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      dangerousFunction()
      anotherDangerous()
    `,
    options: [{ functions: ['dangerousFunction', 'anotherDangerous'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 2 }
    "
  `)
})

test('allows selector matching inside function', async () => {
  await valid({
    code: dedent`
      function foo() {
        readFileSync('file.txt')
      }
    `,
    options: [{ selectors: ['CallExpression[callee.name="readFileSync"]'] }],
  })
})

test('disallows selector matching at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      readFileSync('file.txt')
    `,
    options: [{ selectors: ['CallExpression[callee.name="readFileSync"]'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('ignores non-configured function calls at module level', async () => {
  await valid({
    code: dedent`
      someOtherFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside object method', async () => {
  await valid({
    code: dedent`
      const obj = {
        method() {
          dangerousFunction()
        }
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows function call inside object method shorthand', async () => {
  await valid({
    code: dedent`
      const obj = {
        method: () => {
          dangerousFunction()
        }
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('works with no options provided', async () => {
  await valid({
    code: dedent`
      dangerousFunction()
    `,
    options: [{}],
  })
})

test('allows call in IIFE', async () => {
  await valid({
    code: dedent`
      (function() {
        dangerousFunction()
      })()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('allows call in arrow IIFE', async () => {
  await valid({
    code: dedent`
      (() => {
        dangerousFunction()
      })()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('disallows call in variable initializer at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const result = dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('allows call in arrow function variable initializer', async () => {
  await valid({
    code: dedent`
      const fn = () => dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })
})

test('disallows call in class property initializer', async () => {
  const { result } = await invalid({
    code: dedent`
      class Foo {
        prop = dangerousFunction()
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 2 }
    "
  `)
})

test('disallows call in static class property', async () => {
  const { result } = await invalid({
    code: dedent`
      class Foo {
        static prop = dangerousFunction()
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 2 }
    "
  `)
})

test('disallows call in array literal at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const arr = [dangerousFunction(), 2, 3]
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in object literal at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const obj = {
        key: dangerousFunction()
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 2 }
    "
  `)
})

test('disallows call in ternary expression at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const value = condition ? dangerousFunction() : null
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in logical expression at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const value = someThing && dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in template literal at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const str = \`Result: \${dangerousFunction()}\`
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows multiple calls on same line', async () => {
  const { result } = await invalid({
    code: dedent`
      const a = dangerousFunction(), b = dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows chained calls at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const result = dangerousFunction().then(x => x)
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in export statement', async () => {
  const { result } = await invalid({
    code: dedent`
      export const value = dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in default export', async () => {
  const { result } = await invalid({
    code: dedent`
      export default dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call as function argument at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      console.log(dangerousFunction())
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in destructuring pattern at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const { data } = dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in array destructuring at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const [first] = dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call with spread operator at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const arr = [...dangerousFunction()]
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in await expression at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const value = await dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows nested member expression call at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      obj.nested.readFileSync('file.txt')
    `,
    options: [{ functions: ['readFileSync'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in switch statement at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      switch (dangerousFunction()) {
        case 1: break
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in if statement condition at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      if (dangerousFunction()) {
        console.log('test')
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in for loop at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      for (const item of dangerousFunction()) {
        console.log(item)
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in while loop condition at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      while (dangerousFunction()) {
        break
      }
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in throw statement at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      throw dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows call in return statement at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      return dangerousFunction()
    `,
    options: [{ functions: ['dangerousFunction'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows member expression tagged template at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const query = db.sql\`SELECT * FROM users\`
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows tagged template in array at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const queries = [sql\`SELECT * FROM users\`, sql\`SELECT * FROM posts\`]
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows tagged template in object at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const obj = {
        query: sql\`SELECT * FROM users\`
      }
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 2 }
    "
  `)
})

test('disallows tagged template as function argument at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      execute(sql\`SELECT * FROM users\`)
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows tagged template in export at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      export const query = sql\`SELECT * FROM users\`
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows tagged template in ternary at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const query = condition ? sql\`SELECT * FROM users\` : null
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows tagged template with complex expression at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const query = sql\`
        SELECT * FROM users
        WHERE id = \${userId}
        AND name = \${userName}
      \`
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows nested tagged template at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const html = html\`<div>\${css\`color: red;\`}</div>\`
    `,
    options: [{ functions: ['html', 'css'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})

test('disallows tagged template in class property at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      class Query {
        query = sql\`SELECT * FROM users\`
      }
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 2 }
    "
  `)
})

test('disallows tagged template chained with method at module level', async () => {
  const { result } = await invalid({
    code: dedent`
      const result = sql\`SELECT * FROM users\`.execute()
    `,
    options: [{ functions: ['sql'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moduleLevel', line: 1 }
    "
  `)
})
