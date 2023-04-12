import { describe, test } from 'vitest'
import { noUnusedObjectTypeProperties } from '../src/rules/no-unused-object-type-properties'

import { ESLintUtils } from '@typescript-eslint/utils'
import { RuleModule } from '@typescript-eslint/utils/dist/ts-eslint'

const ruleTester = new ESLintUtils.RuleTester({
  parser: '@typescript-eslint/parser',
  parserOptions: {
    createDefaultProgram: true,
    project: true,
  },
})

function createTester<T extends RuleModule<string, any[]>>(
  rule: {
    name: string
    rule: T
  },
  defaultErrorId?: string,
) {
  return {
    valid(code: string) {
      ruleTester.run(rule.name, rule.rule, {
        valid: [
          {
            code,
          },
        ],
        invalid: [],
      })
    },
    invalid(
      code: string,
      errors: {
        messageId?: string
        data?: Record<string, string>
      }[],
    ) {
      ruleTester.run(rule.name, rule.rule, {
        valid: [],
        invalid: [
          {
            code,
            errors:
              errors.map((error) => ({
                messageId: error.messageId || defaultErrorId || '?',
                data: error.data,
              })) || [],
          },
        ],
      })
    },
  }
}

describe('no-unused-object-type-properties', () => {
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

  test('ignore types with intersections', () => {
    valid(
      `
      type Test = {
        unusedType?: string;
        usedType?: string;
      } & { otherType?: string };

      function test({ usedType }: Test) {
        console.log(usedType);
      }
    `,
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
})
