import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { noUnnecessaryIife } from './no-unnecessary-iife'

const { valid, invalid } = createNewTester(noUnnecessaryIife)

test('valid: IIFE with multiple statements', async () => {
  await valid(dedent`
    const x = (() => {
      const y = 1;
      return y + 1;
    })();
  `)
})

test('valid: IIFE with variable declaration and return', async () => {
  await valid(dedent`
    const x = (() => {
      const temp = getValue();
      return temp.map(t => t.id);
    })();
  `)
})

test('valid: IIFE without return (side effects only)', async () => {
  await valid(dedent`
    (() => {
      console.log('init');
    })();
  `)
})

test('valid: regular function (not IIFE)', async () => {
  await valid(dedent`
    const fn = () => { return 1; };
  `)
})

test('valid: IIFE with arguments', async () => {
  await valid(dedent`
    const x = ((a) => a + 1)(5);
  `)
})

test('valid: IIFE with multiple arguments', async () => {
  await valid(dedent`
    const x = ((a, b) => a + b)(1, 2);
  `)
})

test('valid: async IIFE with await statements', async () => {
  await valid(dedent`
    const x = (async () => {
      const data = await fetch('/api');
      return data.json();
    })();
  `)
})

test('valid: IIFE with try-catch', async () => {
  await valid(dedent`
    const x = (() => {
      try {
        return getValue();
      } catch {
        return null;
      }
    })();
  `)
})

test('invalid: arrow IIFE with single return statement', async () => {
  const { result } = await invalid(dedent`
    const x = (() => {
      return members.map(m => m.id);
    })();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = members.map(m => m.id);"
  `)
})

test('invalid: arrow IIFE with expression body', async () => {
  const { result } = await invalid(dedent`
    const x = (() => members.map(m => m.id))();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = members.map(m => m.id);"
  `)
})

test('invalid: regular function IIFE with single return', async () => {
  const { result } = await invalid(dedent`
    const x = (function() {
      return getValue();
    })();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = getValue();"
  `)
})

test('invalid: named function IIFE with single return', async () => {
  const { result } = await invalid(dedent`
    const x = (function myFunc() {
      return getValue();
    })();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = getValue();"
  `)
})

test('invalid: IIFE returning object literal', async () => {
  const { result } = await invalid(dedent`
    const x = (() => {
      return { a: 1, b: 2 };
    })();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = { a: 1, b: 2 };"
  `)
})

test('invalid: IIFE returning simple value', async () => {
  const { result } = await invalid(dedent`
    const x = (() => {
      return 42;
    })();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = 42;"
  `)
})

test('invalid: IIFE returning void', async () => {
  const { result } = await invalid(dedent`
    const x = (() => {
      return;
    })();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = undefined;"
  `)
})

test('invalid: arrow IIFE with expression body returning object', async () => {
  const { result } = await invalid(dedent`
    const x = (() => ({ a: 1 }))();
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'unnecessaryIife', line: 1 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const x = { a: 1 };"
  `)
})
