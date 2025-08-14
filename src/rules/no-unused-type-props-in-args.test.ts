import { createTester } from '../../tests/utils/createTester'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'

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
  'unused properties with object type literal 2',
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
  'shared types still ignored by default (multiple references)',
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

tests.addInvalid(
  'FC typed with props but not used',
  `
  type Props = {
    title: ReactNode;
    onClose: () => void;
  };

  export const Component: FC<Props> = () => {
    return null;
  };
  `,
  [{ messageId: 'missingComponentParam' }],
)

tests.addInvalidWithOptions(
  'props used in FC should still be checked even if referenced more than once',
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

  const usedProps: Props = {
    title,
    onClose,
  };
  `,
  { forceCheckOnFCPropTypesWithName: ['Props$'] },
  [
    {
      messageId: 'unusedObjectTypeProperty',
      data: { propertyName: 'onClose' },
    },
  ],
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

      const usedProps: Props = {
        title,
        onClose,
      };
    `,
  },
)

tests.addValid(
  'shared types in regular functions are still ignored (multiple references)',
  `
  type Props = {
    title: ReactNode;
    onClose: () => void;
  };

  export const Component = ({
    title,
  }: Props) => {
    return null;
  };

  const usedProps: Props = {
    title,
    onClose,
  };
  `,
)

tests.addInvalid(
  'always check function option types by default - exported type',
  `
  export type AppCreationContext = {
    creationRequest: string;
    contextTables: string[];
    contextApps: string[];
    language: string;
    companyColor: string | null;
    tenantId: string;
    agentId: string;
    attachments: { file: string; fileType: string }[];
  };

  function generateAppStructure({
    creationRequest,
  }: AppCreationContext): string {
    return creationRequest; // only using creationRequest
  }
  `,
  [
    { data: { propertyName: 'contextTables' } },
    { data: { propertyName: 'contextApps' } },
    { data: { propertyName: 'language' } },
    { data: { propertyName: 'companyColor' } },
    { data: { propertyName: 'tenantId' } },
    { data: { propertyName: 'agentId' } },
    { data: { propertyName: 'attachments' } },
  ],
  {
    output: `
      export type AppCreationContext = {
        creationRequest: string;
        contextTables: string[];
        contextApps: string[];
        language: string;
        companyColor: string | null;
        tenantId: string;
        agentId: string;
        attachments: { file: string; fileType: string }[];
      };

      function generateAppStructure({
        creationRequest, contextTables, contextApps, language, companyColor, tenantId, agentId, attachments,
      }: AppCreationContext): string {
        return creationRequest; // only using creationRequest
      }
    `,
  },
)

tests.addValid(
  'shared types should still be ignored even with alwaysCheckFunctionOptionTypes',
  `
  type RequestOptions = {
    url: string;
    method: string;
    headers?: Record<string, string>;
    body?: string;
    timeout?: number;
  };

  function makeRequest({ url, method }: RequestOptions) {
    return fetch(url, { method });
  }

  function anotherRequest({ url }: RequestOptions) {
    return url;
  }
  `,
)

tests.addValid(
  'disable always check function option types - shared type should be ignored',
  `
  type RequestOptions = {
    url: string;
    method: string;
    headers?: Record<string, string>;
    body?: string;
    timeout?: number;
  };

  function makeRequest({ url, method }: RequestOptions) {
    return fetch(url, { method });
  }

  function anotherRequest({ url }: RequestOptions) {
    return url;
  }
  `,
  { alwaysCheckFunctionOptionTypes: false },
)

tests.addValid(
  'disable always check function option types - exported type should be ignored',
  `
  export type AppCreationContext = {
    creationRequest: string;
    contextTables: string[];
  };

  export async function generateAppStructure({
    creationRequest,
  }: AppCreationContext): Promise<string> {
    return creationRequest;
  }
  `,
  { alwaysCheckFunctionOptionTypes: false },
)

tests.addInvalid(
  'always check function option types - single use type should still work',
  `
  function processData({ 
    input, 
  }: { 
    input: string; 
    output?: string;
    config?: Record<string, any>;
  }) {
    return input;
  }
  `,
  [
    { data: { propertyName: 'output' } },
    { data: { propertyName: 'config' } },
  ],
  {
    output: `
      function processData({ 
        input, output, config, 
      }: { 
        input: string; 
        output?: string;
        config?: Record<string, any>;
      }) {
        return input;
      }
    `,
  },
)

tests.addInvalid(
  'debug - simple exported type test',
  `
  export type SimpleType = {
    used: string;
    unused: string;
  };

  function test({ used }: SimpleType) {
    return used;
  }
  `,
  [
    { data: { propertyName: 'unused' } },
  ],
  {
    output: `
      export type SimpleType = {
        used: string;
        unused: string;
      };

      function test({ used, unused }: SimpleType) {
        return used;
      }
    `,
  },
)

tests.run()
