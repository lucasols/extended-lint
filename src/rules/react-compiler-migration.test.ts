import { createTester } from '../../tests/utils/createTester'
import { reactCompilerMigration } from './react-compiler-migration'

const tests = createTester(reactCompilerMigration)

tests.addValid(
  'Regular code with no enable directive',
  `
    function Component() {
      useUnsafeHook()
      return null
    }
  `,
  {
    disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }],
    runOnlyWithEnableCompilerDirective: true,
  },
)

tests.addValid(
  'Disabled hooks with enable directive but not matching the hooks',
  `
    // eslint react-compiler/react-compiler: ["error"]
    function Component() {
      useSafeHook()
      return null
    }
  `,
  {
    disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }],
    runOnlyWithEnableCompilerDirective: true,
  },
)

tests.addInvalidWithOptions(
  'Using disallowed hook without enable directive when runOnlyWithEnableCompilerDirective is false',
  `
    function Component() {
      useUnsafeHook()
      return null
    }
  `,
  {
    disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }],
    runOnlyWithEnableCompilerDirective: false,
  },
  [
    {
      messageId: 'disallowedFunctionOrMethod',
      data: {
        functionOrMethod: 'useUnsafeHook',
        replacement: 'useSafeHook',
      },
      suggestions: [
        {
          messageId: 'replace',
          data: {
            replacement: 'useSafeHook',
          },
          output: `
            function Component() {
              useSafeHook()
              return null
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalidWithOptions(
  'Using disallowed hook with enable directive',
  `
    // eslint react-compiler/react-compiler: ["error"]

    function Component() {
      useUnsafeHook()
      return null
    }
  `,
  {
    disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }],
    runOnlyWithEnableCompilerDirective: true,
  },
  [
    {
      messageId: 'disallowedFunctionOrMethod',
      data: {
        functionOrMethod: 'useUnsafeHook',
        replacement: 'useSafeHook',
      },
      suggestions: [
        {
          messageId: 'replace',
          data: {
            replacement: 'useSafeHook',
          },
          output: `
            // eslint react-compiler/react-compiler: ["error"]

            function Component() {
              useSafeHook()
              return null
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalidWithOptions(
  'Using disallowed hook via namespace with enable directive',
  `
    // eslint react-compiler/react-compiler: ["error"]

    function Component() {
      namespace.useUnsafeHook()
      return null
    }
  `,
  {
    disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }],
    runOnlyWithEnableCompilerDirective: true,
  },
  [
    {
      messageId: 'disallowedFunctionOrMethod',
      data: {
        functionOrMethod: 'useUnsafeHook',
        replacement: 'useSafeHook',
      },
      suggestions: [
        {
          messageId: 'replace',
          data: {
            replacement: 'useSafeHook',
          },
          output: `
            // eslint react-compiler/react-compiler: ["error"]

            function Component() {
              namespace.useSafeHook()
              return null
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalidWithOptions(
  'Using disallowed method in a hook with enable directive 2',
  `
    // eslint react-compiler/react-compiler: ["error"]

    function Component() {
      useHook({
        unsafeMethod() {
          // implementation
        }
      })
      return null
    }
  `,
  {
    disallowMethods: [{ name: 'unsafeMethod', replacement: 'safeMethod' }],
    runOnlyWithEnableCompilerDirective: true,
  },
  [
    {
      messageId: 'disallowedFunctionOrMethod',
      data: {
        functionOrMethod: 'unsafeMethod',
        replacement: 'safeMethod',
      },
      suggestions: [
        {
          messageId: 'replace',
          data: {
            replacement: 'safeMethod',
          },
          output: `
            // eslint react-compiler/react-compiler: ["error"]

            function Component() {
              useHook({
                safeMethod() {
                  // implementation
                }
              })
              return null
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalidWithOptions(
  'Using disallowed method in a hook inside namespace with enable directive',
  `
    // eslint react-compiler/react-compiler: ["error"]

    function Component() {
      namespace.useHook({
        unsafeMethod: () => {
          // implementation
        }
      })
      return null
    }
  `,
  {
    disallowMethods: [{ name: 'unsafeMethod', replacement: 'safeMethod' }],
    runOnlyWithEnableCompilerDirective: true,
  },
  [
    {
      messageId: 'disallowedFunctionOrMethod',
      data: {
        functionOrMethod: 'unsafeMethod',
        replacement: 'safeMethod',
      },
      suggestions: [
        {
          messageId: 'replace',
          data: {
            replacement: 'safeMethod',
          },
          output: `
            // eslint react-compiler/react-compiler: ["error"]

            function Component() {
              namespace.useHook({
                safeMethod: () => {
                  // implementation
                }
              })
              return null
            }
          `,
        },
      ],
    },
  ],
)

tests.addInvalidWithOptions(
  'Using disallowed method in a hook inside namespace with enable directive 2',
  `
    // eslint react-compiler/react-compiler: ["error"]

    function Component() {
      namespace.useHook({
        unsafeMethod() {
          // implementation
        }
      })
      return null
    }
  `,
  {
    disallowMethods: [{ name: 'unsafeMethod', replacement: 'safeMethod' }],
    runOnlyWithEnableCompilerDirective: true,
  },
  [
    {
      messageId: 'disallowedFunctionOrMethod',
      data: {
        functionOrMethod: 'unsafeMethod',
        replacement: 'safeMethod',
      },
      suggestions: [
        {
          messageId: 'replace',
          data: {
            replacement: 'safeMethod',
          },
          output: `
            // eslint react-compiler/react-compiler: ["error"]

            function Component() {
              namespace.useHook({
                safeMethod() {
                  // implementation
                }
              })
              return null
            }
          `,
        },
      ],
    },
  ],
)

tests.describe('disallowMethod with requireTrueProp', () => {
  tests.addValid(
    'Using disallowed method with missing requireTrueProp',
    `
      // eslint react-compiler/react-compiler: ["error"]

      function Component() {
        useHook({
          unsafeMethod: () => {},
          isSafe: true,
        })
        return null
      }
    `,
    {
      disallowMethods: [
        {
          name: 'unsafeMethod',
          requireTrueProp: 'isSafe',
        },
      ],
      runOnlyWithEnableCompilerDirective: true,
    },
  )

  tests.addInvalidWithOptions(
    'requireTrueProp is set to false',
    `
      // eslint react-compiler/react-compiler: ["error"]

      function Component() {
        useHook({
          unsafeMethod: () => {},
          isSafe: false,
        })
        return null
      }
    `,
    {
      disallowMethods: [
        {
          name: 'unsafeMethod',
          requireTrueProp: 'isSafe',
        },
      ],
      runOnlyWithEnableCompilerDirective: true,
    },
    [
      {
        messageId: 'disallowedMethodWithMissingRequireTrueProp',
        data: {
          method: 'unsafeMethod',
          requireTrueProp: 'isSafe',
        },
      },
    ],
  )

  tests.addInvalidWithOptions(
    'requireTrueProp is missing',
    `
      // eslint react-compiler/react-compiler: ["error"]

      function Component() {
        useHook({
          unsafeMethod: () => {},
        })
        return null
      }
    `,
    {
      disallowMethods: [
        {
          name: 'unsafeMethod',
          requireTrueProp: 'isSafe',
        },
      ],
      runOnlyWithEnableCompilerDirective: true,
    },
    [
      {
        messageId: 'disallowedMethodWithMissingRequireTrueProp',
        data: {
          method: 'unsafeMethod',
          requireTrueProp: 'isSafe',
        },
      },
    ],
  )

  tests.addInvalidWithOptions(
    'requireTrueProp is missing 2',
    `
      // eslint react-compiler/react-compiler: ["error"]


      const chatMessages = chatMessagesList.useListQuery(
        {
          chatId: isRecordChat ? chatId : Number(chatId),
          chatType,
          parentMessageId: mainMessageId,
        },
        {
          returnRefetchingStatus: true,
          itemSelector(data, id) {
            return { data, id };
          },
        },
      );
    `,
    {
      disallowMethods: [
        {
          name: 'itemSelector',
          requireTrueProp: 'isSafe',
        },
      ],
      runOnlyWithEnableCompilerDirective: true,
    },
    [
      {
        messageId: 'disallowedMethodWithMissingRequireTrueProp',
        data: {
          method: 'itemSelector',
          requireTrueProp: 'isSafe',
        },
      },
    ],
  )
})

// Run the tests
tests.run()
