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

  tests.only.addInvalidWithOptions(
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

tests.run()
