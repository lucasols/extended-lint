import { test } from 'vitest'
import {
  Options,
  noCallWithInferedGenerics,
} from '../src/rules/no-call-with-infered-generics'
import { createTester } from './utils/createTester'

const { valid, invalid } = createTester(noCallWithInferedGenerics, {
  optionsType: {} as Options,
})

test('valid code', () => {
  valid(
    `
      otherFunction('user/update', { name });
    `,
    [
      {
        functions: [{ name: 'test', minGenerics: 1 }],
      },
    ],
  )
})

test('invalid', () => {
  invalid(
    `
      test('user/update', { name });
    `,
    [
      {
        messageId: 'missingGenericDeclaration',
        data: { functionName: 'test', minGenerics: '1' },
      },
    ],
    [
      {
        functions: [{ name: 'test' }],
      },
    ],
  )
})

test('invalid, need more than one generic defined', () => {
  invalid(
    `
      test<Type>('user/update', { name });
    `,
    [
      {
        messageId: 'missingGenericDeclaration',
        data: { functionName: 'test', minGenerics: '2' },
      },
    ],
    [
      {
        functions: [{ name: 'test', minGenerics: 2 }],
      },
    ],
  )
})

test('invalid usage of any', () => {
  invalid(
    `
      test<any>('user/update', { name });
    `,
    [
      {
        messageId: 'anyUsedInGenerics',
        data: { functionName: 'test' },
      },
    ],
    [
      {
        functions: [{ name: 'test' }],
      },
    ],
  )
})

test('invalid usage of any alias', () => {
  invalid(
    `
      test<__ANY__>('user/update', { name });
    `,
    [
      {
        messageId: 'anyUsedInGenerics',
        data: { functionName: 'test' },
      },
    ],
    [
      {
        functions: [{ name: 'test' }],
        disallowTypes: ['__ANY__'],
      },
    ],
  )
})
