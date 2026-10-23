import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
  getErrorsWithMsgFromResult,
} from '../../tests/utils/createTester'
import { noCallWithExplicitGenerics } from './no-call-with-explicit-generics'

const { valid, invalid } = createNewTester(noCallWithExplicitGenerics)

test('inferred generics are valid', async () => {
  await valid({
    code: dedent`
      test('user/update', { name });
    `,
    options: [{ functions: ['test'] }],
  })
})

test('non-configured function is valid', async () => {
  await valid({
    code: dedent`
      otherFunction<Type>('user/update', { name });
    `,
    options: [{ functions: ['test'] }],
  })
})

test('method call with inferred generics is valid', async () => {
  await valid({
    code: dedent`
      namespace.method('user/update', { name });
    `,
    options: [{ functions: ['namespace.method'] }],
  })
})

test('non-configured method call is valid', async () => {
  await valid({
    code: dedent`
      namespace.other<Type>('user/update', { name });
    `,
    options: [{ functions: ['namespace.method'] }],
  })
})

test('explicit generics on function', async () => {
  const { result } = await invalid({
    code: dedent`
      test<Type>('user/update', { name });
    `,
    options: [{ functions: ['test'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noExplicitGenerics', line: 1 }
    "
  `)
})

test('multiple explicit generics on function', async () => {
  const { result } = await invalid({
    code: dedent`
      test<Type1, Type2>('user/update', { name });
    `,
    options: [{ functions: ['test'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noExplicitGenerics', line: 1 }
    "
  `)
})

test('explicit generics on method call', async () => {
  const { result } = await invalid({
    code: dedent`
      namespace.method<Type>('user/update', { name });
    `,
    options: [{ functions: ['namespace.method'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noExplicitGenerics', line: 1 }
    "
  `)
})

test('custom message for function', async () => {
  const { result } = await invalid({
    code: dedent`
      test<Type>('user/update', { name });
    `,
    options: [
      {
        functions: [
          { name: 'test', message: 'Use inferred types for test()' },
        ],
      },
    ],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noExplicitGenericsCustom', msg: 'Use inferred types for test()', line: 1 }
    "
  `)
})

test('custom message for method call', async () => {
  const { result } = await invalid({
    code: dedent`
      api.fetch<Response>('/users');
    `,
    options: [
      {
        functions: [
          {
            name: 'api.fetch',
            message: 'Let api.fetch infer the response type from the endpoint',
          },
        ],
      },
    ],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'noExplicitGenericsCustom'
      msg: 'Let api.fetch infer the response type from the endpoint'
      line: 1
    "
  `)
})

test('wildcard matches any namespace', async () => {
  const { result } = await invalid({
    code: dedent`
      foo.method<Type>('a');
      bar.method<Type>('b');
    `,
    options: [{ functions: ['*.method'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noExplicitGenerics', line: 1 }
    - { messageId: 'noExplicitGenerics', line: 2 }
    "
  `)
})

test('wildcard does not match plain function call', async () => {
  await valid({
    code: dedent`
      method<Type>('a');
    `,
    options: [{ functions: ['*.method'] }],
  })
})

test('wildcard with inferred generics is valid', async () => {
  await valid({
    code: dedent`
      foo.method('a');
    `,
    options: [{ functions: ['*.method'] }],
  })
})

test('wildcard with custom message', async () => {
  const { result } = await invalid({
    code: dedent`
      any.fetch<Response>('/users');
    `,
    options: [
      {
        functions: [
          { name: '*.fetch', message: 'Do not pass explicit generics to fetch' },
        ],
      },
    ],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noExplicitGenericsCustom', msg: 'Do not pass explicit generics to fetch', line: 1 }
    "
  `)
})

test('mix of string and object function configs', async () => {
  const { result } = await invalid({
    code: dedent`
      test<Type>('a');
      custom<Type>('b');
    `,
    options: [
      {
        functions: [
          'test',
          { name: 'custom', message: 'Custom message here' },
        ],
      },
    ],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'noExplicitGenerics'
      msg: "Function 'test' should be called with inferred generics (remove the explicit type parameters)"
      line: 1
    - { messageId: 'noExplicitGenericsCustom', msg: 'Custom message here', line: 2 }
    "
  `)
})
