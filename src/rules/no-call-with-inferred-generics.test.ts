import { createTester } from '../../tests/utils/createTester'
import { noCallWithInferredGenerics } from './no-call-with-inferred-generics'

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

tests.addInvalid(
  'invalid usage of typeof in generics',
  `
      const user = { name: 'test' };
      test<typeof user>('user/update', { name });
    `,
  [
    {
      messageId: 'typeOfUsedInGenerics',
      data: { functionName: 'test' },
    },
  ],
  {
    options: [
      {
        functions: [{ name: 'test', disallowTypeOf: true }],
      },
    ],
  },
)

tests.addInvalid(
  'invalid usage of local type alias that resolves to typeof',
  `
      const user = { name: 'test' };
      type User = typeof user;
      test<User>('user/update', { name });
    `,
  [
    {
      messageId: 'typeOfUsedInGenerics',
      data: { functionName: 'test' },
    },
  ],
  {
    options: [
      {
        functions: [{ name: 'test', disallowTypeOf: true }],
      },
    ],
  },
)

tests.addValid(
  'valid: disallowTypeOf with non-typeof type reference',
  `
      type User = { name: string };
      test<User>('user/update', { name });
    `,
  [
    {
      functions: [{ name: 'test', disallowTypeOf: true }],
    },
  ],
)

tests.addValid(
  'valid: disallowTypeOf with external/non-local type reference',
  `
      test<SomeImportedType>('user/update', { name });
    `,
  [
    {
      functions: [{ name: 'test', disallowTypeOf: true }],
    },
  ],
)

tests.addValid(
  'valid: typeof used but disallowTypeOf not enabled',
  `
      const user = { name: 'test' };
      test<typeof user>('user/update', { name });
    `,
  [
    {
      functions: [{ name: 'test' }],
    },
  ],
)

tests.run()
