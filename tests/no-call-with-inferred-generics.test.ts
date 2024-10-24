import { noCallWithInferredGenerics } from '../src/rules/no-call-with-inferred-generics'
import { createTester } from './utils/createTester'

const tests = createTester(noCallWithInferredGenerics)

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
  {
    options: [
      {
        functions: [{ name: 'test' }],
      },
    ],
  },
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
  {
    options: [
      {
        functions: [{ name: 'test', minGenerics: 2 }],
      },
    ],
  },
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
  {
    options: [
      {
        functions: [{ name: 'test' }],
      },
    ],
  },
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
  {
    options: [
      {
        functions: [{ name: 'test' }],
        disallowTypes: ['__ANY__'],
      },
    ],
  },
)

tests.run()
