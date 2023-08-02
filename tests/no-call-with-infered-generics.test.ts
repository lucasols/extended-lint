import { test, describe } from 'vitest'
import {
  Options,
  noCallWithInferedGenerics,
} from '../src/rules/no-call-with-infered-generics'
import { createTester } from './utils/createTester'

const { valid, invalid } = createTester(noCallWithInferedGenerics, {
  optionsType: {} as Options,
  ignoreError: {
    code: `test('user/update', { name });`,
    options: [
      {
        functions: [{ name: 'test' }],
      },
    ],
    errors: [
      {
        data: { functionName: 'test', minGenerics: '1' },
        messageId: 'missingGenericDeclaration',
      },
    ],
  },
})

describe('valid code', () => {
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

describe('invalid', () => {
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

describe('invalid, need more than one generic defined', () => {
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

describe('invalid usage of any', () => {
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

describe('invalid usage of any alias', () => {
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
