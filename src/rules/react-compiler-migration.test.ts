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
  { disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }] },
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
  { disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }] },
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
  { disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }] },
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
  { disallowHooks: [{ name: 'useUnsafeHook', replacement: 'useSafeHook' }] },
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
  'Using disallowed method in a hook with enable directive',
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
  { disallowMethods: [{ name: 'unsafeMethod', replacement: 'safeMethod' }] },
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
  { disallowMethods: [{ name: 'unsafeMethod', replacement: 'safeMethod' }] },
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
  'Using disallowed method in a hook inside namespace with enable directive',
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
  { disallowMethods: [{ name: 'unsafeMethod', replacement: 'safeMethod' }] },
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

// Run the tests
tests.run()
