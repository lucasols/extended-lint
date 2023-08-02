import { describe, test } from 'vitest'
import { noUnusedObjectTypeProperties } from '../src/rules/no-unused-type-props-in-args'
import { createTester } from './utils/createTester'

const { valid, invalid } = createTester(noUnusedObjectTypeProperties, {
  defaultErrorId: 'unusedObjectTypeProperty',
  ignoreError: {
    code: `function test({ usedType }: { unusedType?: string, usedType?: string }) {
        console.log(usedType);
      }`,
    errors: [
      {
        data: { propertyName: 'unusedType' },
        messageId: 'unusedObjectTypeProperty',
      },
    ],
  },
})

describe('no type annotation', () => {
  valid(`
      function test({ usedType }: { [k: string]: string }) {
        console.log(usedType);
      }

      function test(test: 'k') {
        console.log(usedType);
      }
    `)
})

describe('no unused properties with object type literal', () => {
  valid(`
      function test({ usedType }: { usedType?: string }) {
        console.log(usedType);
      }
    `)
})

describe('no unused properties with object type reference', () => {
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

describe('unused properties with object type literal', () => {
  invalid(
    `
      function test({ usedType }: { unusedType?: string, usedType?: string }) {
        console.log(usedType);
      }
    `,
    [{ data: { propertyName: 'unusedType' } }],
  )
})

describe('unused properties with object type literal', () => {
  invalid(
    `
      const test = ({ usedType }: { unusedType?: string, usedType?: string }) => {
        console.log(usedType);
      }
    `,
    [{ data: { propertyName: 'unusedType' } }],
  )
})

describe('unused properties with object type reference', () => {
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

describe('unused properties with object interface reference', () => {
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

describe('ignore types with unions', () => {
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

describe('ignore imported types', () => {
  valid(
    `
      import { Test } from './test';

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `,
  )
})

describe('ignored shared types', () => {
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

describe('unused properties with FC object type reference', () => {
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

describe('unused properties with FC object type literal', () => {
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

describe('ignore rest parameters', () => {
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

describe('ignore exported refs rest parameters 2', () => {
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

describe('dont ignore types with intersections, referenced', () => {
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

describe('dont ignore types with intersections', () => {
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

describe('false positive', () => {
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

describe('dont ignore exported refs in FC components', () => {
  invalid(
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
    [{ data: { propertyName: 'onClose' } }],
  )
})

describe('test bug', () => {
  invalid(
    `
      type FormItemsInput = {
  className?: string;
  selected: FormItem[];
  hint?: string;
  label: string;
  optional?: boolean;
  errors: string[];
  handleChange: (setter: (current: FormItem[]) => FormItem[]) => void;
};

export const FormItemsInput: FC<FormItemsInput> = ({
  label,
  hint,
  errors,
  selected,
  handleChange,
}) => {
  return null
};
    `,
    [
      {
        data: {
          propertyName: 'className',
        },
      },
      {
        data: {
          propertyName: 'optional',
        },
      },
    ],
  )
})
