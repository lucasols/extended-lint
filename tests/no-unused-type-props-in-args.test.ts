import { noUnusedObjectTypeProperties } from '../src/rules/no-unused-type-props-in-args'
import { createTester } from './utils/createTester'

const tests = createTester(noUnusedObjectTypeProperties, {
  defaultErrorId: 'unusedObjectTypeProperty',
})

tests.addValid(
  'no type annotation',
  `
      function test({ usedType }: { [k: string]: string }) {
        console.log(usedType);
      }

      function test(test: 'k') {
        console.log(usedType);
      }
    `,
)

tests.addValid(
  'no unused properties with object type literal',
  `
  function test({ usedType }: { usedType?: string }) {
    console.log(usedType);
  }
`,
)

tests.addValid(
  'no unused properties with object type reference',
  `
  type Test = {
    usedType?: string;
  };

  function test({ usedType }: Test) {
    console.log(usedType);
  }
`,
)

tests.addValid(
  'ignore param type unions',
  `
  type Test = {
    usedType?: string;
  };

  function test({ usedType }: Test | { otherType?: string }) {
    console.log(usedType);
  }
`,
)

tests.addInvalid(
  'unused properties with object type literal',
  `
  function test({ usedType }: { unusedType?: string, usedType?: string }) {
    console.log(usedType);
  }
`,
  [
    {
      data: { propertyName: 'unusedType' },
    },
  ],
  {
    output: `
      function test({ usedType, unusedType }: { unusedType?: string, usedType?: string }) {
        console.log(usedType);
      }
    `,
  },
)

tests.addInvalid(
  'unused properties with object type literal',
  `
  const test = ({ usedType }: { unusedType?: string, usedType?: string }) => {
    console.log(usedType);
  }
`,
  [{ data: { propertyName: 'unusedType' } }],
  {
    output: `
      const test = ({ usedType, unusedType }: { unusedType?: string, usedType?: string }) => {
        console.log(usedType);
      }
    `,
  },
)

tests.addInvalid(
  'unused properties with object type reference',
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
  {
    output: `
      type Test = {
        unusedType?: string;
        usedType?: string;
      };

      function test({ usedType, unusedType }: Test) {
        console.log(usedType);
      }
    `,
  },
)

tests.addInvalid(
  'unused properties with object interface reference',
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
  {
    output: `
      interface Test {
        unusedType?: string;
        usedType?: string;
      };

      function test({ usedType, unusedType }: Test) {
        console.log(usedType);
      }
    `,
  },
)

tests.addValid(
  'ignore types with unions',
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
tests.addValid(
  'ignore imported types',
  `
  import { Test } from './test';

  function test({ usedType }: Test) {
    console.log(usedType);
  }
`,
)
tests.addValid(
  'ignored shared types',
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

tests.addInvalid(
  'unused properties with FC object type reference',
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
  {
    output: `
      type Props = {
        title: ReactNode;
        onClose: () => void;
      };

      export const Component: FC<Props> = ({
        title, onClose,
      }) => {
        return null;
      };
    `,
  },
)
tests.addInvalid(
  'unused properties with FC object type literal',
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
  {
    output: `
      type Props = {
        title: ReactNode;
        onClose: () => void;
      };

      export const Component: FC<{
        title: ReactNode;
        onClose: () => void;
      }> = ({
        title, onClose,
      }) => {
        return null;
      };
    `,
  },
)

tests.addValid(
  'ignore rest parameters',
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

tests.addValid(
  'ignore exported refs rest parameters 2',
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

tests.addInvalid(
  'dont ignore types with intersections, referenced',
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
  {
    output: `
      type Test = {
        unusedType?: string;
        usedType?: string;
      } & { otherType?: string };

      function test({ usedType, unusedType, otherType }: Test) {
        console.log(usedType);
      }
    `,
  },
)

tests.addInvalid(
  'dont ignore types with intersections',
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
  {
    output: `
      function test({ usedType, unusedType, otherType }: {
        unusedType?: string;
        usedType?: string;
      } & { otherType?: string }) {
        console.log(usedType);
      }
    `,
  },
)

tests.addInvalid(
  'false positive',
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

  {
    output: `
      import { sleep } from '@utils/sleep';

      export async function retryOnError<T>(
        fn: () => Promise<T>,
        maxRetries: number,
        {
          delayBetweenRetriesMs, retryCondition,
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
  },
)

tests.addInvalid(
  'dont ignore exported refs in FC components',
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
  {
    output: `
      export type Props = {
        title: ReactNode;
        onClose: () => void;
      };


      export const Component: FC<Props> = ({
        title, onClose,
      }) => {
        return null;
      };
    `,
  },
)

tests.addInvalid(
  'dont ignore types with intersections, referenced in FC',
  `
  export type Props = {
    onClose: () => void;
  };

  export const Component: FC<Props & {
    otherType?: string;
  }> = ({
    title,
  }) => {
    return null;
  };
`,
  [
    { data: { propertyName: 'onClose' } },
    { data: { propertyName: 'otherType' } },
  ],
  {
    output: `
      export type Props = {
        onClose: () => void;
      };

      export const Component: FC<Props & {
        otherType?: string;
      }> = ({
        title, onClose, otherType,
      }) => {
        return null;
      };
    `,
  },
)

tests.addInvalid(
  'test bug',
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
  {
    output: `
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
        handleChange, className, optional,
      }) => {
        return null
      };
    `,
  },
)

tests.addInvalid(
  'unused properties with empty destructed props',
  `
  function test({}: { unusedType?: string }) {
    console.log('ok');
  }
`,
  [{ data: { propertyName: 'unusedType' } }],
  {
    output: `
      function test({unusedType}: { unusedType?: string }) {
        console.log('ok');
      }
    `,
  },
)

tests.run()
