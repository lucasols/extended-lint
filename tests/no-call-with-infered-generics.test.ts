import { test, describe } from 'vitest'
import {
  Options,
  noCallWithInferedGenerics,
} from '../src/rules/no-call-with-infered-generics'
import { createOldTester, createTester } from './utils/createTester'

const tests = createTester(noCallWithInferedGenerics, {
  optionsType: {} as Options,
})

tests.addValid(
  'valid code',
  `
      otherFunction('user/update', { name });
    `,
  [
    {
      functions: [{ name: 'test', minGenerics: 1 }],
    },
  ],
)

tests.addInvalid(
  'invalid',
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

tests.addInvalid(
  'invalid, need more than one generic defined',
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

tests.addInvalid(
  'invalid usage of any',
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

tests.addInvalid(
  'invalid usage of any alias',
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

tests.run()
