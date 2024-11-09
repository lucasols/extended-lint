import { advancedNoRestrictedSyntax } from '../src/rules/advanced-no-restricted-syntax'
import { createTester } from './utils/createTester'

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
      mustCallFn: {
        shouldCallFn: {
          args: [{ pos: 0, literal: 'foo' }],
          message: 'shouldCallFn should be called with "foo"',
        },
      },
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
      mustCallFn: {
        shouldCallFn: {
          args: [{ pos: 0, literal: 'foo' }],
          message: 'shouldCallFn should be called with "foo"',
        },
      },
    },
    [
      {
        data: {
          message:
            'Invalid argument value: shouldCallFn should be called with "foo"',
        },
      },
    ],
  )

  tests.addInvalidWithOptions(
    'missing argument',
    `
      function foo(bar) {
        shouldCallFn();
      }
    `,
    {
      mustCallFn: {
        shouldCallFn: {
          args: [{ pos: 0, literal: 'foo' }],
          message:
            "Missing required argument at position 0: shouldCallFn should be called with 'foo'",
        },
      },
    },
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
      mustCallFn: {
        shouldCallFn: {
          getFileNameVarsRegex: '(.+)File',
          args: [{ pos: 0, literal: '$1' }],
          message: 'shouldCallFn should be called with "$1"',
        },
      },
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
      mustCallFn: {
        shouldCallFn: {
          getFileNameVarsRegex: '(.+)File',
          args: [{ pos: 0, literal: '$1_lowercase' }],
          message: 'shouldCallFn should be called with "$1_lowercase"',
        },
      },
    },
  )

  tests.addInvalidWithOptions(
    'with fileNameVars',
    `
      function foo(bar) {
        shouldCallFn('number');
      }
    `,
    {
      __dev_simulateFileName: 'stringFile.ts',
      mustCallFn: {
        shouldCallFn: {
          getFileNameVarsRegex: '(.+)File.ts',
          args: [{ pos: 0, literal: '$1' }],
          message: 'shouldCallFn should be called with "$1"',
        },
      },
    },
    [
      {
        data: {
          message:
            'Invalid argument value: shouldCallFn should be called with "string"',
        },
      },
    ],
  )
})

tests.run()
