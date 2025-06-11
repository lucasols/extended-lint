import { createTester } from '../../tests/utils/createTester'
import { preferHookAlternative } from './prefer-hook-alternative'

const tests = createTester(preferHookAlternative, {
  defaultErrorId: 'preferHookAlternative',
})

const defaultOptions = {
  disallowedFunctions: [
    {
      name: 'setTimeout',
      hookAlternative: 'useTimeout',
    },
    {
      name: 'setInterval',
      hookAlternative: 'useInterval',
    },
    {
      name: 'addEventListener',
      hookAlternative: 'useEventListener',
    },
  ],
}

tests.addValid(
  'non-React function using setTimeout',
  `
    function regularFunction() {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
    }
  `,
  defaultOptions,
)

tests.addValid(
  'class method using setTimeout',
  `
    class MyClass {
      method() {
        setTimeout(() => {
          console.log('hello')
        }, 1000)
      }
    }
  `,
  defaultOptions,
)

tests.addValid(
  'function with lowercase name using setTimeout',
  `
    function normalFunction() {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
    }
  `,
  defaultOptions,
)

tests.addValid(
  'React component using allowed function',
  `
    function MyComponent() {
      console.log('allowed')
      return <div>test</div>
    }
  `,
  defaultOptions,
)

tests.addValid(
  'custom hook using allowed function',
  `
    function useMyHook() {
      console.log('allowed')
      return true
    }
  `,
  defaultOptions,
)

tests.addValid(
  'nested function in component using setTimeout is allowed',
  `
    function MyComponent() {
      function handleClick() {
        setTimeout(() => {
          console.log('hello')
        }, 1000)
      }
      return <div onClick={handleClick}>test</div>
    }
  `,
  defaultOptions,
)

tests.addValid(
  'function call inside JSX props in non-React function',
  `
    function helper() {
      return <div onClick={() => setTimeout(() => {}, 1000)}>test</div>
    }
  `,
  defaultOptions,
)

tests.addInvalid(
  'React component using setTimeout',
  `
    function MyComponent() {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message: 'setTimeout should not be used in React components.',
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function MyComponent() {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'React component using setInterval',
  `
    function MyComponent() {
      setInterval(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message: 'setInterval should not be used in React components.',
        hookAlternative: 'useInterval',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function MyComponent() {
      useInterval(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'React component using addEventListener',
  `
    function MyComponent() {
      window.addEventListener('click', handler)
      return <div>test</div>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message:
          'addEventListener should not be used directly in React components.',
        hookAlternative: 'useEventListener',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function MyComponent() {
      window.useEventListener('click', handler)
      return <div>test</div>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'custom hook using setTimeout',
  `
    function useMyHook() {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return true
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message: 'setTimeout should not be used in React components.',
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function useMyHook() {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return true
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'arrow function component using setTimeout',
  `
    const MyComponent = () => {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message: 'setTimeout should not be used in React components.',
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    const MyComponent = () => {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'arrow hook using setTimeout',
  `
    const useMyHook = () => {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return true
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message: 'setTimeout should not be used in React components.',
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    const useMyHook = () => {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return true
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'forwardRef callback using setTimeout',
  `
    const MyComponent = React.forwardRef((props, ref) => {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div ref={ref}>test</div>
    })
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    const MyComponent = React.forwardRef((props, ref) => {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div ref={ref}>test</div>
    })
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'memo callback using setTimeout',
  `
    const MyComponent = React.memo((props) => {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    })
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    const MyComponent = React.memo((props) => {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    })
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'forwardRef without React prefix using setTimeout',
  `
    const MyComponent = forwardRef((props, ref) => {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div ref={ref}>test</div>
    })
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message: 'setTimeout should not be used in React components.',
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    const MyComponent = forwardRef((props, ref) => {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div ref={ref}>test</div>
    })
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'memo without React prefix using setTimeout',
  `
    const MyComponent = memo((props) => {
      setTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    })
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    const MyComponent = memo((props) => {
      useTimeout(() => {
        console.log('hello')
      }, 1000)
      return <div>test</div>
    })
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'component with nested disallowed call in useEffect',
  `
    function MyComponent() {
      useEffect(() => {
        setTimeout(() => {
          console.log('hello')
        }, 1000)
      }, [])
      return <div>test</div>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message: 'setTimeout should not be used in React components.',
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function MyComponent() {
      useEffect(() => {
        useTimeout(() => {
          console.log('hello')
        }, 1000)
      }, [])
      return <div>test</div>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'component with disallowed function in JSX prop callback',
  `
    function MyComponent() {
      return <button onClick={() => setTimeout(() => {}, 1000)}>Click</button>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        hookAlternative: 'useTimeout',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function MyComponent() {
      return <button onClick={() => useTimeout(() => {}, 1000)}>Click</button>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'component with disallowed function in JSX prop with member expression',
  `
    function MyComponent() {
      return <div onLoad={() => window.addEventListener('resize', handler)}>Content</div>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        message:
          'addEventListener should not be used directly in React components.',
        hookAlternative: 'useEventListener',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function MyComponent() {
      return <div onLoad={() => window.useEventListener('resize', handler)}>Content</div>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.addInvalid(
  'hook with disallowed function in JSX prop callback',
  `
    function useMyHook() {
      return <div onClick={() => setInterval(() => {}, 1000)}>test</div>
    }
  `,
  [
    {
      messageId: 'preferHookAlternative',
      data: {
        hookAlternative: 'useInterval',
      },
      suggestions: [
        {
          messageId: 'preferHookAlternative',
          output: `
    function useMyHook() {
      return <div onClick={() => useInterval(() => {}, 1000)}>test</div>
    }
  `,
        },
      ],
    },
  ],
  {
    options: defaultOptions,
  },
)

tests.run()
