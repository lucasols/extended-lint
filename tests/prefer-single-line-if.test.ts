import { preferSingleLineIf } from '../src/rules/prefer-single-line-if'
import { createTester } from './utils/createTester'

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

tests.addValid(
  'fn call should be ignored',
  `
    if (foo) {
      foo();
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
  'If with code starting with { in the next line',
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
  'If else',
  `
    if (test) {
      return;
    } else {
      console.log('test');
    }
  `,
)

tests.run()
