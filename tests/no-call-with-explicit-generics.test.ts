import { noCallWithExplicitGenerics } from '../src/rules/no-call-with-explicit-generics'
import { createTester } from './utils/createTester'

const tests = createTester(noCallWithExplicitGenerics)

tests.addValid(
  'inferred generics',
  `
    test('user/update', { name });
  `,
  [
    {
      functions: ['test'],
    },
  ],
)

tests.addValid(
  'non-configured function',
  `
    otherFunction<Type>('user/update', { name });
  `,
  [
    {
      functions: ['test'],
    },
  ],
)

tests.addInvalid(
  'explicit generics',
  `
    test<Type>('user/update', { name });
  `,
  [
    {
      messageId: 'noExplicitGenerics',
      data: { functionName: 'test' },
    },
  ],
  {
    options: [
      {
        functions: ['test'],
      },
    ],
  },
)

tests.addInvalid(
  'multiple explicit generics',
  `
    test<Type1, Type2>('user/update', { name });
  `,
  [
    {
      messageId: 'noExplicitGenerics',
      data: { functionName: 'test' },
    },
  ],
  {
    options: [
      {
        functions: ['test'],
      },
    ],
  },
)

tests.run()