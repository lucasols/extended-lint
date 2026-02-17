import { createTester } from '../../tests/utils/createTester'
import { advancedNoRestrictedSyntax } from './advanced-no-restricted-syntax'

const tests = createTester(advancedNoRestrictedSyntax, {
  defaultErrorId: 'default',
})

tests.describe('disallow', () => {
  // valid
  tests.addValid('basic valid code', 'doSomething();', {
    disallow: [{ selector: 'ConditionalExpression', message: 'noop' }],
  })

  tests.addValid('conditional expression', 'var foo = 42;', {
    disallow: [
      {
        selector: 'ConditionalExpression',
        message: 'Conditional expressions are not allowed',
      },
    ],
  })

  tests.addValid('multiple selectors', 'foo += 42;', [
    {
      disallow: [
        {
          selector: 'VariableDeclaration',
          message: 'Variable declarations are not allowed',
        },
        {
          selector: 'FunctionExpression',
          message: 'Function expressions are not allowed',
        },
      ],
    },
  ])

  tests.addValid('identifier selector', 'foo;', [
    {
      disallow: [
        {
          selector: 'Identifier[name="bar"]',
          message: 'Identifier "bar" is not allowed',
        },
      ],
    },
  ])

  tests.addValid('arrow function', '() => 5', [
    {
      disallow: [
        {
          selector: 'ArrowFunctionExpression > BlockStatement',
          message: 'Arrow functions with block statements are not allowed',
        },
      ],
    },
  ])

  tests.addValid('property selector', '({ foo: 1, bar: 2 })', [
    {
      disallow: [
        {
          selector: 'Property > Literal.key',
          message: 'Literal property keys are not allowed',
        },
      ],
    },
  ])

  // invalid
  tests.addInvalid(
    'variable declaration',
    'var foo = 41;',
    [
      {
        data: { message: "Using 'VariableDeclaration' is not allowed." },
      },
    ],
    {
      options: {
        disallow: [
          {
            selector: 'VariableDeclaration',
            message: "Using 'VariableDeclaration' is not allowed.",
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'empty statement',
    ';function lol(a) { return 42; }',
    [
      {
        data: { message: "Using 'EmptyStatement' is not allowed." },
      },
    ],
    {
      options: {
        disallow: [
          {
            selector: 'EmptyStatement',
            message: "Using 'EmptyStatement' is not allowed.",
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'multiple violations',
    'try { voila(); } catch (e) { oops(); }',
    [
      {
        data: { message: "Using 'TryStatement' is not allowed." },
      },
      {
        data: { message: "Using 'CallExpression' is not allowed." },
      },
      {
        data: { message: "Using 'CatchClause' is not allowed." },
      },
      {
        data: { message: "Using 'CallExpression' is not allowed." },
      },
    ],
    {
      options: {
        disallow: [
          {
            selector: 'TryStatement',
            message: "Using 'TryStatement' is not allowed.",
          },
          {
            selector: 'CallExpression',
            message: "Using 'CallExpression' is not allowed.",
          },
          {
            selector: 'CatchClause',
            message: "Using 'CatchClause' is not allowed.",
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'custom error message',
    'function foo(bar, baz, qux) {}',
    [
      {
        data: { message: 'custom error message.' },
      },
    ],
    {
      options: {
        disallow: [
          {
            selector: 'FunctionDeclaration[params.length>2]',
            message: 'custom error message.',
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'optional chaining',
    'var foo = foo?.bar?.();',
    [
      {
        data: { message: "Using 'ChainExpression' is not allowed." },
      },
    ],
    {
      options: {
        disallow: [
          {
            selector: 'ChainExpression',
            message: "Using 'ChainExpression' is not allowed.",
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'multiple optional chaining violations',
    'var foo = foo?.bar?.();',
    [
      {
        data: { message: "Using '[optional=true]' is not allowed." },
      },
      {
        data: { message: "Using '[optional=true]' is not allowed." },
      },
    ],
    {
      options: {
        disallow: [
          {
            selector: '[optional=true]',
            message: "Using '[optional=true]' is not allowed.",
          },
        ],
      },
    },
  )
})

tests.describe('replaceWith fix', () => {
  tests.addInvalidWithOptions(
    'replace fn name suggestion',
    `
      invalidFnName();
    `,
    {
      disallow: [
        {
          selector: 'CallExpression[callee.name="invalidFnName"]',
          message: 'invalidFnName is not allowed',
          replace: 'validFnName()',
        },
      ],
    },
    [
      {
        data: { message: 'invalidFnName is not allowed' },
        suggestions: [
          {
            messageId: 'default',
            data: { message: 'Replace with "validFnName()"' },
            output: 'validFnName();',
          },
        ],
      },
    ],
  )

  tests.addInvalidWithOptions(
    'replace fn name with regex autofix',
    `
      invalidFnName(validArg);

      invalidFnName(invalidArg2);
    `,
    {
      disallow: [
        {
          selector: 'CallExpression[callee.name="invalidFnName"]',
          message: 'invalidFnName is not allowed',
          replace: {
            regex: 'invalidFnName(.+)',
            with: 'validFnName$1',
          },
          replaceType: 'autofix',
        },
      ],
    },
    [
      { data: { message: 'invalidFnName is not allowed' } },
      { data: { message: 'invalidFnName is not allowed' } },
    ],
    {
      output: `
        validFnName(validArg);

        validFnName(invalidArg2);
      `,
    },
  )
})

tests.describe('mustCallFn', () => {
  tests.addValid(
    'calling with correct argument',
    `
      function foo(bar) {
        shouldCallFn('foo');
      }
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: 'foo' }],
                },
              ],
            },
          ],
        },
      ],
    },
  )

  tests.addInvalidWithOptions(
    'wrong argument',
    `
      function foo(bar) {
        shouldCallFn('bar');
      }
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: 'foo' }],
                },
              ],
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'Argument should have the value "foo"',
        },
      },
    ],
    {
      output: `
        function foo(bar) {
          shouldCallFn('foo');
        }
      `,
    },
  )

  tests.addInvalidWithOptions(
    'missing argument',
    `
      function foo(bar) {
        shouldCallFn();
      }
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: 'foo' }],
                },
              ],
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'Missing argument with value "foo" at index 0',
        },
      },
    ],
  )

  tests.addInvalidWithOptions(
    'missing function call',
    `
      function foo(bar) {
        const test = 1;
      }
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: 'foo' }],
                },
              ],
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'Expected file to call the function: shouldCallFn',
        },
      },
    ],
  )

  tests.addValid(
    'with fileNameVars',
    `
      function foo(bar) {
        shouldCallFn('string');
      }
    `,
    {
      __dev_simulateFileName: 'stringFile.ts',
      mustMatchSyntax: [
        {
          includeRegex: '(.+)File',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: '$1' }],
                },
              ],
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'with fileNameVars lowercase modifier',
    `
      function foo(bar) {
        shouldCallFn('string');
      }
    `,
    {
      __dev_simulateFileName: 'StringFile.ts',
      mustMatchSyntax: [
        {
          includeRegex: '(.+)File',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: '$1_lowercase' }],
                },
              ],
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'with fileNameVars capitalize modifier on literal',
    `
      function foo(bar) {
        shouldCallFn('stringType');
      }
    `,
    {
      __dev_simulateFileName: 'StringTypeFile.ts',
      mustMatchSyntax: [
        {
          includeRegex: '(.+)File',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: '$1_uncapitalize' }],
                },
              ],
            },
          ],
        },
      ],
    },
  )

  tests.addInvalidWithOptions(
    'with fileNameVars 2',
    `
      function foo(bar) {
        shouldCallFn('number');
      }
    `,
    {
      __dev_simulateFileName: 'stringFile.ts',
      mustMatchSyntax: [
        {
          includeRegex: '(.+)File.ts',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: '$1' }],
                },
              ],
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'Argument should have the value "string"',
        },
      },
    ],
    {
      output: `
        function foo(bar) {
          shouldCallFn('string');
        }
      `,
    },
  )

  tests.addInvalidWithOptions(
    'multiple calls with some invalid',
    `
      function foo(bar) {
        shouldCallFn('string');
        shouldCallFn('number');
      }
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: 'string' }],
                },
              ],
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'Argument should have the value "string"',
        },
      },
    ],
    {
      output: `
        function foo(bar) {
          shouldCallFn('string');
          shouldCallFn('string');
        }
      `,
    },
  )

  tests.addValid(
    'multiple allowed calls',
    `
      function foo(bar) {
        shouldCallFn2(true, 'string');
      }
     `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: 'string' }],
                },
                {
                  fn: 'shouldCallFn2',
                  withArgs: [{ atIndex: 1, literal: 'string' }],
                },
              ],
            },
          ],
        },
      ],
    },
  )
})

tests.describe('mustMatchSelector', () => {
  tests.addValid(
    'declaring correctly a variable',
    `
    const foo = 'bar';
  `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustMatchSelector: [
            { selector: 'Identifier[name="foo"]', message: 'noop' },
          ],
        },
      ],
    },
  )

  tests.addInvalidWithOptions(
    'not declaring a variable',
    `
      const test = 'bar';
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustMatchSelector: [
            {
              selector: 'Identifier[name="foo"]',
              message: 'File should declare "foo"',
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'File should declare "foo"',
        },
      },
    ],
  )

  tests.addValid(
    'declaring a variable in destructuring',
    `
    const { foo } = bar;
  `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustMatchSelector: [
            { selector: 'Property[key.name="foo"]', message: 'noop' },
          ],
        },
      ],
    },
  )

  tests.addInvalidWithOptions(
    'not declaring a variable in destructuring',
    `
    const { test } = bar;
  `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustMatchSelector: [
            {
              selector: 'Property[key.name="foo"]',
              message: 'Missing declaration of "foo"',
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'Missing declaration of "foo"',
        },
      },
    ],
  )

  tests.addValid(
    'use a filename var',
    `
    const string = 'bar';
  `,
    {
      __dev_simulateFileName: 'stringFile.ts',
      mustMatchSyntax: [
        {
          includeRegex: '(.+)File.ts$',
          mustMatchSelector: [
            { selector: 'Identifier[name="$1"]', message: 'noop' },
          ],
        },
      ],
    },
  )

  tests.addInvalidWithOptions(
    'invalid with use a filename var ',
    `
    const number = 'bar';
  `,
    {
      __dev_simulateFileName: 'stringFile.ts',
      mustMatchSyntax: [
        {
          includeRegex: '(.+)File.ts$',
          mustMatchSelector: [
            {
              selector: 'Identifier[name="$1"]',
              message: 'File should declare "$1"',
            },
          ],
        },
      ],
    },
    [
      {
        data: {
          message: 'File should declare "string"',
        },
      },
    ],
  )
})

tests.describe('mustMatchSyntax excludeRegex', () => {
  tests.addValid(
    'excluded file is skipped even if includeRegex matches',
    `
      const test = 'bar';
    `,
    {
      __dev_simulateFileName: 'stringFile.test.ts',
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          excludeRegex: '\\.test\\.',
          mustMatchSelector: [
            {
              selector: 'Identifier[name="foo"]',
              message: 'File should declare "foo"',
            },
          ],
        },
      ],
    },
  )

  tests.addInvalid(
    'non-excluded file still triggers error',
    `
      const test = 'bar';
    `,
    [
      {
        data: {
          message: 'File should declare "foo"',
        },
      },
    ],
    {
      options: {
        __dev_simulateFileName: 'stringFile.ts',
        mustMatchSyntax: [
          {
            includeRegex: '.*',
            excludeRegex: '\\.test\\.',
            mustMatchSelector: [
              {
                selector: 'Identifier[name="foo"]',
                message: 'File should declare "foo"',
              },
            ],
          },
        ],
      },
    },
  )

  tests.addValid(
    'excluded file with mustCallFn is skipped',
    `
      function foo(bar) {
        const test = 1;
      }
    `,
    {
      __dev_simulateFileName: 'stringFile.spec.ts',
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          excludeRegex: '\\.(spec|test)\\.',
          mustCallFn: [
            {
              anyCall: [
                {
                  fn: 'shouldCallFn',
                  withArgs: [{ atIndex: 0, literal: 'foo' }],
                },
              ],
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'excluded file with mustHaveExport is skipped',
    `
      const testVar = 'test';
    `,
    {
      __dev_simulateFileName: 'components/Test.stories.tsx',
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          excludeRegex: '\\.stories\\.',
          mustHaveExport: [
            {
              name: 'testVar',
              type: 'variable',
              message: 'Must export testVar',
            },
          ],
        },
      ],
    },
  )
})

tests.describe('disallowFnCalls', () => {
  tests.addValid('different function call', 'someOtherFunction();', {
    disallowFnCalls: [
      {
        fn: 'openModal',
        message: 'Use openManagePlan function instead',
        replaceWith: 'openManagePlan()',
      },
    ],
  })

  tests.addValid(
    'function call with different argument',
    'openModal("otherArg");',
    {
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
        },
      ],
    },
  )

  tests.addInvalid(
    'disallow function call with specific argument',
    'openModal("managePlan");',
    [
      {
        data: { message: 'Use openManagePlan function instead' },
        suggestions: [
          {
            messageId: 'default',
            data: { message: 'Replace with "openManagePlan()"' },
            output: 'openManagePlan();',
          },
        ],
      },
    ],
    {
      options: {
        disallowFnCalls: [
          {
            fn: 'openModal',
            withArgs: [{ atIndex: 0, value: 'managePlan' }],
            message: 'Use openManagePlan function instead',
            replaceWith: 'openManagePlan()',
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'disallow function call with import alias',
    `
import { openModal as om } from 'modals';
om("managePlan");
    `,
    [
      {
        data: { message: 'Use openManagePlan function instead' },
        suggestions: [
          {
            messageId: 'default',
            data: { message: 'Replace with "openManagePlan()"' },
            output: `
import { openModal as om } from 'modals';
openManagePlan();
     `,
          },
        ],
      },
    ],
    {
      options: {
        disallowFnCalls: [
          {
            fn: 'openModal',
            withArgs: [{ atIndex: 0, value: 'managePlan' }],
            message: 'Use openManagePlan function instead',
            replaceWith: 'openManagePlan()',
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'disallow function call with variable alias',
    `
const om = openModal;
om("managePlan");
    `,
    [
      {
        data: { message: 'Use openManagePlan function instead' },
        suggestions: [
          {
            messageId: 'default',
            data: { message: 'Replace with "openManagePlan()"' },
            output: `
const om = openModal;
openManagePlan();
     `,
          },
        ],
      },
    ],
    {
      options: {
        disallowFnCalls: [
          {
            fn: 'openModal',
            withArgs: [{ atIndex: 0, value: 'managePlan' }],
            message: 'Use openManagePlan function instead',
            replaceWith: 'openManagePlan()',
          },
        ],
      },
    },
  )

  tests.addValid(
    'import alias with different argument',
    `
import { openModal as om } from 'modals';
om("otherArg");
  `,
    {
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
        },
      ],
    },
  )

  tests.addValid(
    'variable alias with different argument',
    `
const om = openModal;
om("otherArg");
  `,
    {
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
        },
      ],
    },
  )

  tests.addValid('ignore when argument is variable', 'openModal(modalId);', {
    disallowFnCalls: [
      {
        fn: 'openModal',
        withArgs: [{ atIndex: 0, value: 'managePlan' }],
        message: 'Use openManagePlan function instead',
        replaceWith: 'openManagePlan()',
      },
    ],
  })

  tests.addValid(
    'ignore when argument is expression',
    'openModal(getModalId());',
    {
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
        },
      ],
    },
  )

  tests.addValid(
    'ignore when argument is object property',
    'openModal(config.modalId);',
    {
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
        },
      ],
    },
  )

  tests.addInvalid(
    'trigger when no withArgs specified',
    'openModal(modalId);',
    [
      {
        data: { message: 'Use openManagePlan function instead' },
        suggestions: [
          {
            messageId: 'default',
            data: { message: 'Replace with "openManagePlan()"' },
            output: 'openManagePlan();',
          },
        ],
      },
    ],
    {
      options: {
        disallowFnCalls: [
          {
            fn: 'openModal',
            message: 'Use openManagePlan function instead',
            replaceWith: 'openManagePlan()',
          },
        ],
      },
    },
  )
})

tests.describe('disallowFnCalls with ignoreRegex', () => {
  tests.addValid(
    'ignored file with matching pattern',
    'openModal("managePlan");',
    {
      __dev_simulateFileName: 'test.spec.ts',
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
          ignoreRegex: '\\.spec\\.',
        },
      ],
    },
  )

  tests.addValid(
    'ignored file with wildcard pattern',
    'openModal("managePlan");',
    {
      __dev_simulateFileName: 'components/test.stories.tsx',
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
          ignoreRegex: '\\.(stories|spec|test)\\.',
        },
      ],
    },
  )

  tests.addInvalid(
    'non-ignored file still triggers error',
    'openModal("managePlan");',
    [
      {
        data: { message: 'Use openManagePlan function instead' },
        suggestions: [
          {
            messageId: 'default',
            data: { message: 'Replace with "openManagePlan()"' },
            output: 'openManagePlan();',
          },
        ],
      },
    ],
    {
      options: {
        __dev_simulateFileName: 'components/Modal.tsx',
        disallowFnCalls: [
          {
            fn: 'openModal',
            withArgs: [{ atIndex: 0, value: 'managePlan' }],
            message: 'Use openManagePlan function instead',
            replaceWith: 'openManagePlan()',
            ignoreRegex: '\\.spec\\.',
          },
        ],
      },
    },
  )

  tests.addValid(
    'alias in ignored file',
    `
import { openModal as om } from 'modals';
om("managePlan");
  `,
    {
      __dev_simulateFileName: 'Modal.test.ts',
      disallowFnCalls: [
        {
          fn: 'openModal',
          withArgs: [{ atIndex: 0, value: 'managePlan' }],
          message: 'Use openManagePlan function instead',
          replaceWith: 'openManagePlan()',
          ignoreRegex: '\\.test\\.',
        },
      ],
    },
  )
})

tests.describe('mustHaveExport', () => {
  tests.addValid(
    'export function matches requirement',
    `
export function testFunction() {
  return 'test';
}
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustHaveExport: [
            {
              name: 'testFunction',
              type: 'function',
              message: 'Must export testFunction',
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'export variable matches requirement',
    `
export const testVar = 'test';
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustHaveExport: [
            {
              name: 'testVar',
              type: 'variable',
              message: 'Must export testVar',
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'export with type "any" matches function',
    `
export function testFunction() {
  return 'test';
}
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustHaveExport: [
            {
              name: 'testFunction',
              type: 'any',
              message: 'Must export testFunction',
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'export with type "any" matches variable',
    `
export const testVar = 'test';
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustHaveExport: [
            {
              name: 'testVar',
              type: 'any',
              message: 'Must export testVar',
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'named export specifier matches requirement',
    `
const testFunction = () => 'test';
export { testFunction };
    `,
    {
      mustMatchSyntax: [
        {
          includeRegex: '.*',
          mustHaveExport: [
            {
              name: 'testFunction',
              type: 'any',
              message: 'Must export testFunction',
            },
          ],
        },
      ],
    },
  )

  tests.addValid(
    'export with filename variables',
    `
export function TestComponent() {
  return 'test';
}
    `,
    {
      __dev_simulateFileName: 'TestComponent.tsx',
      mustMatchSyntax: [
        {
          includeRegex: '(.*)\\.tsx$',
          mustHaveExport: [
            {
              name: '$1',
              type: 'function',
              message: 'Must export $1',
            },
          ],
        },
      ],
    },
  )

  tests.addInvalid(
    'missing required export function',
    `
const testVar = 'test';
    `,
    [
      {
        data: {
          message:
            'Missing required export "testFunction" of type function: Must export testFunction',
        },
      },
    ],
    {
      options: {
        mustMatchSyntax: [
          {
            includeRegex: '.*',
            mustHaveExport: [
              {
                name: 'testFunction',
                type: 'function',
                message: 'Must export testFunction',
              },
            ],
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'missing required export variable',
    `
export function testFunction() {
  return 'test';
}
    `,
    [
      {
        data: {
          message:
            'Missing required export "testVar" of type variable: Must export testVar',
        },
      },
    ],
    {
      options: {
        mustMatchSyntax: [
          {
            includeRegex: '.*',
            mustHaveExport: [
              {
                name: 'testVar',
                type: 'variable',
                message: 'Must export testVar',
              },
            ],
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'wrong export type (function when variable required)',
    `
export function testVar() {
  return 'test';
}
    `,
    [
      {
        data: {
          message:
            'Missing required export "testVar" of type variable: Must export testVar',
        },
      },
    ],
    {
      options: {
        mustMatchSyntax: [
          {
            includeRegex: '.*',
            mustHaveExport: [
              {
                name: 'testVar',
                type: 'variable',
                message: 'Must export testVar',
              },
            ],
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'wrong export type (variable when function required)',
    `
export const testFunction = () => 'test';
    `,
    [
      {
        data: {
          message:
            'Missing required export "testFunction" of type function: Must export testFunction',
        },
      },
    ],
    {
      options: {
        mustMatchSyntax: [
          {
            includeRegex: '.*',
            mustHaveExport: [
              {
                name: 'testFunction',
                type: 'function',
                message: 'Must export testFunction',
              },
            ],
          },
        ],
      },
    },
  )

  tests.addInvalid(
    'multiple missing exports',
    `
export const testVar = 'test';
    `,
    [
      {
        data: {
          message:
            'Missing required export "testFunction" of type function: Must export testFunction',
        },
      },
      {
        data: {
          message:
            'Missing required export "anotherFunction" of type any: Must export anotherFunction',
        },
      },
    ],
    {
      options: {
        mustMatchSyntax: [
          {
            includeRegex: '.*',
            mustHaveExport: [
              {
                name: 'testFunction',
                type: 'function',
                message: 'Must export testFunction',
              },
              {
                name: 'anotherFunction',
                type: 'any',
                message: 'Must export anotherFunction',
              },
            ],
          },
        ],
      },
    },
  )
})

tests.run()
