import { createTester } from '../../tests/utils/createTester'
import { preferSingleLineIf } from './prefer-single-line-if'

const tests = createTester(preferSingleLineIf)

tests.addValid(
  'no curly ifs',
  `
    const foo = 42;
  `,
)

tests.addValid(
  'no curly ifs with comments',
  `
    const foo = 42; // 42
  `,
)

tests.addValid(
  'multiline curly ifs',
  `
    const foo = 42
    if (foo) {
      return null;
      return null;
    }
  `,
)

tests.addInvalid(
  'early void return',
  `
    const foo = 42
    if (foo) {
      return;
    }
  `,
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      const foo = 42
      if (foo) return;
    `,
  },
)

tests.addValid(
  'comment inside if',
  `
    const foo = 42
    if (foo) {
      // 42
      return;
    }
  `,
)

tests.addInvalid(
  'non void return',
  `
    const foo = 42
    if (foo) {
      return foo;
    }
  `,
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      const foo = 42
      if (foo) return foo;
    `,
  },
)

tests.addInvalid(
  'literal return',
  `
    const foo = 42
    if (foo) {
      return 42;
    }

    if (foo) {
      return '42';
    }
  `,
  [{ messageId: 'noSingleLineCurly' }, { messageId: 'noSingleLineCurly' }],
  {
    output: `
      const foo = 42
      if (foo) return 42;

      if (foo) return '42';
    `,
  },
)

tests.addValid(
  'very long return',
  `
    if (foo) {
      return veryLongFunctionCallNameShouldThatNotFitInOneLine;
    }
  `,
  { maxLineLength: 50 },
)

tests.addValid(
  'very long condition',
  `
    if (veryLongFunctionCallNameShouldThatNotFitInOneLine()) {
      return foo;
    }
  `,
  { maxLineLength: 50 },
)

tests.addValid(
  'multi line condition',
  `
    if (
      test &&
      test2 &&
      test3
    ) {
      return;
    }
  `,
  { maxLineLength: 50 },
)

tests.addInvalidWithOptions(
  'simple call expression',
  `
    if (foo) {
      foo();
    }
  `,
  { maxLineLength: 50 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      if (foo) foo();
    `,
  },
)

tests.addValid(
  'ignored conditions',
  `
    if (foo && bar) {
      return foo;
    }

    if (foo && bar !== 42) {
      return foo;
    }

    // with ternary
    if (bar ? true : false) {
      return foo;
    }
  `,
)

tests.addValid(
  'Muti line expression',
  `
    if (test) {
      return foo(
        a,
        b
      );
    }
  `,
)

tests.addValid(
  'Long indented line',
  `
{
  {
    {
      {
        {
          {
            {
              {
                if (test) {
                  return;
}}}}}}}}}
  `,
  { maxLineLength: 25 },
)

tests.addValid(
  'If with code starting with } else in the next line',
  `
    if (test) {
      if (test) {
        return;
      }
    } else {
      console.log('test');
    }
  `,
)

tests.addValid(
  'If with code starting with } catch in the next line',
  `
    try {
      if (test) {
        return;
      }
    } catch (error) {
      console.log('test');
    }
  `,
)

tests.addValid(
  'If else',
  `
    if (test) {
      return;
    } else {
      console.log('test');
    }
  `,
)

tests.describe('maxCallConditionLength', () => {
  tests.addInvalidWithOptions(
    'Max call condition length',
    `
    if (testShortCall()) {
      return null;
    }
  `,
    { maxNonSimpleConditionLength: 100 },
    [{ messageId: 'noSingleLineCurly' }],
    {
      output: `
        if (testShortCall()) return null;
      `,
    },
  )

  tests.addValid(
    'Max call condition length',
    `
    if (testVeryLongCall(veryLongArgument)) {
      return null;
    }
  `,
    { maxNonSimpleConditionLength: 20 },
  )

  tests.addValid(
    'Long call with void return should be ignored',
    `
    if (testVeryLongCall(veryLongArgument)) {
      return;
    }
  `,
    { maxNonSimpleConditionLength: 20 },
  )
})

tests.addInvalidWithOptions(
  'Break and continue',
  `
    if (test) {
      break;
    }

    if (test2) {
      continue;
    }
  `,
  { maxLineLength: 50 },
  [{ messageId: 'noSingleLineCurly' }, { messageId: 'noSingleLineCurly' }],
  {
    output: `
      if (test) break;

      if (test2) continue;
    `,
  },
)

tests.addInvalidWithOptions(
  'Should not ignore single condition',
  `
    export function getFieldMutateAfterOptimisticUpdateFn(type: FieldsTypeIds) {
      const fieldConfig = getFieldTypeConfig(type);

      if (!fieldConfig.recordValueConfig.mutateAfterOptimisticUpdate) {
        return null;
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      export function getFieldMutateAfterOptimisticUpdateFn(type: FieldsTypeIds) {
        const fieldConfig = getFieldTypeConfig(type);

        if (!fieldConfig.recordValueConfig.mutateAfterOptimisticUpdate) return null;
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Should not ignore positive condition',
  `
    export function getFieldMutateAfterOptimisticUpdateFn(type: FieldsTypeIds) {
      const fieldConfig = getFieldTypeConfig(type);

      if (!fieldConfig.recordValueConfig.mutateAfterOptimisticUpdate) {
        return null;
      }

      if (fieldConfig.recordValueConfig.mutateAfterOptimisticUpdate) {
        return null;
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }, { messageId: 'noSingleLineCurly' }],
  {
    output: `
      export function getFieldMutateAfterOptimisticUpdateFn(type: FieldsTypeIds) {
        const fieldConfig = getFieldTypeConfig(type);

        if (!fieldConfig.recordValueConfig.mutateAfterOptimisticUpdate) return null;

        if (fieldConfig.recordValueConfig.mutateAfterOptimisticUpdate) return null;
      }
    `,
  },
)

tests.addValid(
  'Ignore complex member expression',
  `
    export function getFieldMutateAfterOptimisticUpdateFn(type: FieldsTypeIds) {
      const fieldConfig = getFieldTypeConfig(type);

      if (!fieldConfig.recordValueConfig('ok').mutateAfterOptimisticUpdate) {
        return null;
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
)

tests.addInvalidWithOptions(
  'Should not ignore single condition',
  `
    function foo() {
      if (!bar) {
        return forceFormValidation();
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return forceFormValidation();
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Should not ignore single condition',
  `
    function foo() {
      if (!bar) {
        return !forceFormValidation();
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return !forceFormValidation();
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Should not ignore single condition',
  `
    function foo() {
      if (!bar) {
        return !variable;
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return !variable;
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Should not ignore single condition',
  `
    function foo() {
      if (!bar) {
        return !!variable;
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return !!variable;
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Return empty array',
  `
    function foo() {
      if (!bar) {
        return [];
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return [];
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Return empty   ',
  `
    function foo() {
      if (!bar) {
        return {};
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return {};
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Return template literal',
  `
    function foo() {
      if (!bar) {
        return \`foo\`;
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return \`foo\`;
      }
    `,
  },
)

tests.addInvalidWithOptions(
  'Return tagged template literal',
  `
    function foo() {
      if (!bar) {
        return __\`foo\`;
      }
    }
  `,
  { maxLineLength: 80, maxNonSimpleConditionLength: 40 },
  [{ messageId: 'noSingleLineCurly' }],
  {
    output: `
      function foo() {
        if (!bar) return __\`foo\`;
      }
    `,
  },
)

tests.run()
