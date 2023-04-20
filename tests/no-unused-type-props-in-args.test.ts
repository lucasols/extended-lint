import { describe, test } from 'vitest'
import { noUnusedObjectTypeProperties } from '../src/rules/no-unused-type-props-in-args'
import { createTester } from './utils/createTester'

const { valid, invalid } = createTester(
  noUnusedObjectTypeProperties,
  'unusedObjectTypeProperty',
)

test('no type annotation', () => {
  valid(`
      function test({ usedType }: { [k: string]: string }) {
        console.log(usedType);
      }

      function test(test: 'k') {
        console.log(usedType);
      }
    `)
})

test('no unused properties with object type literal', () => {
  valid(`
      function test({ usedType }: { usedType?: string }) {
        console.log(usedType);
      }
    `)
})

test('no unused properties with object type reference', () => {
  valid(`
      type Test = {
        usedType?: string;
      };

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `)
})

test('ignore param type unions', () => {
  valid(`
      type Test = {
        usedType?: string;
      };

      function test({ usedType }: Test | { otherType?: string }) {
        console.log(usedType);
      }
    `)
})

test('unused properties with object type literal', () => {
  invalid(
    `
      function test({ usedType }: { unusedType?: string, usedType?: string }) {
        console.log(usedType);
      }
    `,
    [{ data: { propertyName: 'unusedType' } }],
  )
})

test('unused properties with object type literal', () => {
  invalid(
    `
      const test = ({ usedType }: { unusedType?: string, usedType?: string }) => {
        console.log(usedType);
      }
    `,
    [{ data: { propertyName: 'unusedType' } }],
  )
})

test('unused properties with object type reference', () => {
  invalid(
    `
      type Test = {
        unusedType?: string;
        usedType?: string;
      };

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `,
    [{ data: { propertyName: 'unusedType' } }],
  )
})

test('unused properties with object interface reference', () => {
  invalid(
    `
      interface Test {
        unusedType?: string;
        usedType?: string;
      };

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `,
    [{ data: { propertyName: 'unusedType' } }],
  )
})

test('ignore types with unions', () => {
  valid(
    `
      type Test = {
        unusedType?: string;
        usedType?: string;
      } | { otherType?: string };

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `,
  )
})

test('ignore imported types', () => {
  valid(
    `
      import { Test } from './test';

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `,
  )
})

test('ignored shared types', () => {
  valid(
    `
      type Test = {
        unusedType?: string;
        usedType?: string;
      };

      function test({ usedType }: Test) {
        console.log(usedType);
      }

      function test2({ usedType }: Test) {
        console.log(usedType);
      }
    `,
  )
})

test('unused properties with FC object type reference', () => {
  invalid(
    `
      type Props = {
        title: ReactNode;
        onClose: () => void;
      };

      export const Component: FC<Props> = ({
        title,
      }) => {
        return null;
      };
    `,
    [{ data: { propertyName: 'onClose' } }],
  )
})

test('unused properties with FC object type literal', () => {
  invalid(
    `
      type Props = {
        title: ReactNode;
        onClose: () => void;
      };

      export const Component: FC<{
        title: ReactNode;
        onClose: () => void;
      }> = ({
        title,
      }) => {
        return null;
      };
    `,
    [{ data: { propertyName: 'onClose' } }],
  )
})

test('ignore rest parameters', () => {
  valid(
    `
      type Props = {
        title: ReactNode;
        onClose: () => void;
      };

      export const Component: FC<Props> = ({
        title,
        ...rest
      }) => {
        return null;
      };
    `,
  )
})

test('ignore exported refs rest parameters', () => {
  valid(
    `
      export type Props = {
        title: ReactNode;
        onClose: () => void;
      };

      export const Component: FC<Props> = ({
        title,
      }) => {
        return null;
      };
    `,
  )
})

test('ignore exported refs rest parameters 2', () => {
  valid(
    `
       type Props = {
        title: ReactNode;
        onClose: () => void;
      };

      export type Props2 = Props

      export const Component: FC<Props> = ({
        title,
      }) => {
        return null;
      };
    `,
  )
})

test('dont ignore types with intersections, referenced', () => {
  invalid(
    `
      type Test = {
        unusedType?: string;
        usedType?: string;
      } & { otherType?: string };

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `,
    [
      { data: { propertyName: 'unusedType' } },
      { data: { propertyName: 'otherType' } },
    ],
  )
})

test('dont ignore types with intersections', () => {
  invalid(
    `
      function test({ usedType }: {
        unusedType?: string;
        usedType?: string;
      } & { otherType?: string }) {
        console.log(usedType);
      }
    `,
    [
      { data: { propertyName: 'unusedType' } },
      { data: { propertyName: 'otherType' } },
    ],
  )
})

test.only('false positive', () => {
  invalid(
    `
import { sleep } from '@utils/sleep';

export async function retryOnError<T>(
  fn: () => Promise<T>,
  maxRetries: number,
  {
    delayBetweenRetriesMs,
    // retryCondition
  }: {
    delayBetweenRetriesMs?: number;
    retryCondition?: (error: unknown) => boolean;
  } = {},
): Promise<T> {
  try {
    return await fn();
  } catch (error) {
    if (maxRetries > 0) {
      if (delayBetweenRetriesMs) {
        await sleep(delayBetweenRetriesMs);
      }

      return retryOnError(fn, maxRetries - 1, { delayBetweenRetriesMs });
    } else {
      throw error;
    }
  }
}
  `,
    [{ data: { propertyName: 'retryCondition' } }],
  )
})
