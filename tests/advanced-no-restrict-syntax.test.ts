import { advancedNoRestrictedSyntax } from '../src/rules/advanced-no-restricted-syntax'
import { createTester } from './utils/createTester'

const tests = createTester(advancedNoRestrictedSyntax)

tests.describe('disallow', () => {
  // valid
  tests.addValid('basic valid code', 'doSomething();', [
    {
      disallow: [{ selector: 'ConditionalExpression', message: 'noop' }],
    },
  ])

  tests.addValid('conditional expression', 'var foo = 42;', [
    {
      disallow: [
        {
          selector: 'ConditionalExpression',
          message: 'Conditional expressions are not allowed',
        },
      ],
    },
  ])

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
        messageId: 'restrictedSyntax',
        data: { message: "Using 'VariableDeclaration' is not allowed." },
      },
    ],
    {
      options: [
        {
          disallow: [
            {
              selector: 'VariableDeclaration',
              message: "Using 'VariableDeclaration' is not allowed.",
            },
          ],
        },
      ],
    },
  )

  tests.addInvalid(
    'empty statement',
    ';function lol(a) { return 42; }',
    [
      {
        messageId: 'restrictedSyntax',
        data: { message: "Using 'EmptyStatement' is not allowed." },
      },
    ],
    {
      options: [
        {
          disallow: [
            {
              selector: 'EmptyStatement',
              message: "Using 'EmptyStatement' is not allowed.",
            },
          ],
        },
      ],
    },
  )

  tests.addInvalid(
    'multiple violations',
    'try { voila(); } catch (e) { oops(); }',
    [
      {
        messageId: 'restrictedSyntax',
        data: { message: "Using 'TryStatement' is not allowed." },
      },
      {
        messageId: 'restrictedSyntax',
        data: { message: "Using 'CallExpression' is not allowed." },
      },
      {
        messageId: 'restrictedSyntax',
        data: { message: "Using 'CatchClause' is not allowed." },
      },
      {
        messageId: 'restrictedSyntax',
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
        messageId: 'restrictedSyntax',
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
        messageId: 'restrictedSyntax',
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
        messageId: 'restrictedSyntax',
        data: { message: "Using '[optional=true]' is not allowed." },
      },
      {
        messageId: 'restrictedSyntax',
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

tests.describe('mustMatch', () => {
  tests.addValid(
    'should have a if statement',
    `
    if (foo) {
      bar();
    }
  `,
    {
      mustMatch: [
        { selector: 'IfStatement', message: 'If statements should be used' },
      ],
    },
  )
})

tests.run()
