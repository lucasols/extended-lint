import { dedent } from '@ls-stack/utils/dedent'
import { describe, expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { reactCompilerExtra } from './react-compiler-extra'

const { valid, invalid } = createNewTester(reactCompilerExtra)

test('non-hook call with object method', async () => {
  await valid(
    dedent`
      const result = someFunction({
        method() {
          return 42
        }
      });
    `,
  )
})

test('hook call without object method', async () => {
  await valid(
    dedent`
      const result = useState({
        method: () => {
          return 42
        }
      });
    `,
  )
})

test('hook call with function expression', async () => {
  await valid(
    dedent`
      const result = useCallback({
        method: () => {
          return 42
        }
      }, []);
    `,
  )
})

test('hook call with object method', async () => {
  const { result } = await invalid(dedent`
    const result = useState({
      method() {
        return 42
      }
    });
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 2
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "const result = useState({
      method: () => {
        return 42
      }
    });"
  `)
})

test('hook call with object method using this', async () => {
  const { result } = await invalid(dedent`
    const result = useState({
      method() {
        this.value = 42;
        return this.value;
      }
    });
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'thisKeywordInMethod'
      data: 'Object method uses \`this\` keyword which would have different behavior if converted to an arrow function. Fix this manually.'
      line: 2
    "
  `)
})

test('hook call with nested this usage', async () => {
  const { result } = await invalid(dedent`
    const result = useState({
      method() {
        if (true) {
          console.log(this.value);
        }
        return 42;
      }
    });
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'thisKeywordInMethod'
      data: 'Object method uses \`this\` keyword which would have different behavior if converted to an arrow function. Fix this manually.'
      line: 2
    "
  `)
})

test('useCallback with object method', async () => {
  const { result } = await invalid(dedent`
    const result = useCallback({
      method() {
        return 42
      }
    }, []);
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 2
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "const result = useCallback({
      method: () => {
        return 42
      }
    }, []);"
  `)
})

test('useMemo with object method', async () => {
  const { result } = await invalid(dedent`
    const result = useMemo(() => ({
      method() {
        return 42
      }
    }), []);
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 2
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "const result = useMemo(() => ({
      method: () => {
        return 42
      }
    }), []);"
  `)
})

test('multiple object methods in hook call', async () => {
  const { result } = await invalid(dedent`
    const result = useCallback({
      method1() {
        return 42
      },
      method2() {
        console.log('hello')
      }
    }, []);
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 2
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "const result = useCallback({
      method1: () => {
        return 42
      },
      method2: () => {
        console.log('hello')
      }
    }, []);"
  `)
})

test('mixed methods with and without this', async () => {
  const { result } = await invalid({
    code: dedent`
      const result = useCallback({
        method1() {
          return 42;
        },
        method2() {
          return this.value;
        }
      }, []);
    `,
    verifyAfterFix: false,
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 2
    - messageId: 'thisKeywordInMethod'
      data: 'Object method uses \`this\` keyword which would have different behavior if converted to an arrow function. Fix this manually.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "const result = useCallback({
      method1: () => {
        return 42;
      },
      method2() {
        return this.value;
      }
    }, []);"
  `)
})

test('hook call with generator method', async () => {
  const { result } = await invalid(dedent`
    const result = useMemo(() => ({
      *generator() {
        yield 42
      }
    }), []);
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 2
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "const result = useMemo(() => ({
      generator: function* () {
        yield 42
      }
    }), []);"
  `)
})

test('hook call with object method with parameters', async () => {
  const { result } = await invalid(dedent`
    const result = useState({
      method(a, b, c) {
        return a + b + c
      }
    });
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'objectMethodIsNotSupported'
      data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
      line: 2
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "const result = useState({
      method: (a, b, c) => {
        return a + b + c
      }
    });"
  `)
})

test('nested hook call with object method is ok', async () => {
  await valid(
    dedent`
      const result = useState({
        outer: {
          inner() {
            return 42
          }
        }
      });
    `,
  )
})

test('nested hook call with object method using this', async () => {
  const { result } = await invalid(dedent`
    const result = useState({
      outer: {
        inner() {
          return this.value
        }
      }
    });
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'thisKeywordInMethod'
      data: 'Object method uses \`this\` keyword which would have different behavior if converted to an arrow function. Fix this manually.'
      line: 3
    "
  `)
})

describe('with runOnlyWithEnableCompilerDirective option', () => {
  test('hook call with object method without directive', async () => {
    await valid({
      code: dedent`
        const result = useState({
          method() {
            return 42
          }
        });
      `,
      options: [{ runOnlyWithEnableCompilerDirective: true }],
    })
  })

  test('hook call with object method with directive', async () => {
    const { result } = await invalid({
      code: dedent`
        // eslint react-compiler/react-compiler: ["error"]
        const result = useState({
          method() {
            return 42
          }
        });
      `,
      options: [{ runOnlyWithEnableCompilerDirective: true }],
    })
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'objectMethodIsNotSupported'
        data: 'Object methods such as \`{ method() {} }\` have limited support to optimizations in the React compiler, use a function expression \`{ method: () => {} }\` instead.'
        line: 3
      "
    `)
    expect(result.output).toMatchInlineSnapshot(`
      "// eslint react-compiler/react-compiler: ["error"]
      const result = useState({
        method: () => {
          return 42
        }
      });"
    `)
  })

  test('hook call with object method using this with directive', async () => {
    const { result } = await invalid({
      code: dedent`
        // eslint react-compiler/react-compiler: ["error"]
        const result = useState({
          method() {
            return this.value
          }
        });
      `,
      options: [{ runOnlyWithEnableCompilerDirective: true }],
    })
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'thisKeywordInMethod'
        data: 'Object method uses \`this\` keyword which would have different behavior if converted to an arrow function. Fix this manually.'
        line: 3
      "
    `)
  })
})

describe('React component and hook behavior checking', () => {
  test('FC component creating JSX is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const Component: FC = () => {
          return <div>Hello</div>
        }
      `,
    )
  })

  test('React.FC component creating JSX is valid', async () => {
    await valid(
      dedent`
        import React from 'react'
        
        const Component: React.FC = () => {
          return <div>Hello</div>
        }
      `,
    )
  })

  test('FC component calling hooks is valid', async () => {
    await valid(
      dedent`
        import { FC, useState } from 'react'
        
        const Component: FC = () => {
          const [count, setCount] = useState(0)
          return count.toString()
        }
      `,
    )
  })

  test('FC component calling hooks inside namespace is not valid', async () => {
    const { result } = await invalid(
      dedent`
        import { FC, useState } from 'react'
        
        const Component: FC = () => {
          const [count, setCount] = namespace.useTest(0)
          return count.toString()
        }
      `,
    )

    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React components and hooks should create JSX elements, call other hooks or use the "use memo" directive for optimal React compiler detection.'
        line: 3
      "
    `)
  })

  test('FC component creating JSX in variable assignment is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const Component: FC = () => {
          const element = <div>Hello</div>
          return element
        }
      `,
    )
  })

  test('hook function creating JSX is valid', async () => {
    await valid(
      dedent`
        function useCustomHook() {
          return <div>Hook JSX</div>
        }
      `,
    )
  })

  test('hook function calling other hooks is valid', async () => {
    await valid(
      dedent`
        import { useState } from 'react'
        
        function useCounter() {
          const [count, setCount] = useState(0)
          return { count, setCount }
        }
      `,
    )
  })

  test('FC component with neither JSX nor hooks should show error', async () => {
    const { result } = await invalid(dedent`
      import { FC } from 'react'
      
      const Component: FC = () => {
        return 'Hello World'
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React components and hooks should create JSX elements, call other hooks or use the "use memo" directive for optimal React compiler detection.'
        line: 3
      "
    `)
  })

  test('hook function with neither JSX nor hooks should show error', async () => {
    const { result } = await invalid(dedent`
      function useData() {
        return { data: 'some data' }
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React components and hooks should create JSX elements, call other hooks or use the "use memo" directive for optimal React compiler detection.'
        line: 1
      "
    `)
  })

  test('PascalCase function without FC type should not be checked', async () => {
    await valid(
      dedent`
        const RegularFunction = () => {
          return 'This should not be checked'
        }
      `,
    )
  })

  test('regular function should not be checked', async () => {
    await valid(
      dedent`
        const regularFunction = () => {
          return 'This is fine'
        }
      `,
    )
  })

  test('component with different type annotation should not be checked', async () => {
    await valid(
      dedent`
        const Component: () => string = () => {
          return 'This is fine'
        }
      `,
    )
  })

  test('FC component with mixed JSX and non-JSX returns is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const isLoading = true
        
        const Component: FC = () => {
          return isLoading ? 'Loading...' : <div>Content</div>
        }
      `,
    )
  })

  test('hook function with React.createElement is valid', async () => {
    await valid(
      dedent`
        import React from 'react'
        
        function useElement() {
          return React.createElement('div', null, 'Hello')
        }
      `,
    )
  })

  test('FC component with createPortal containing JSX is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        import { createPortal } from 'react-dom'
        
        const TableActionsMenuPlugin: FC<{anchorElem: HTMLElement}> = ({ anchorElem }) => {
          return createPortal(
            <div>Menu content</div>,
            anchorElem,
          )
        }
      `,
    )
  })

  test('FC component with nested function calls containing JSX is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const Component: FC = () => {
          return someFunction(
            anotherFunction(<span>Nested JSX</span>)
          )
        }
      `,
    )
  })

  describe('use memo directive', () => {
    test('FC component with "use memo" directive is valid', async () => {
      await valid(
        dedent`
          import { FC } from 'react'
          
          const Component: FC = () => {
            "use memo"
            return calculateExpensiveValue()
          }
        `,
      )
    })

    test('hook function with "use memo" directive is valid', async () => {
      await valid(
        dedent`
          function useExpensiveValue() {
            "use memo"
            return calculateExpensiveValue()
          }
        `,
      )
    })

    test('FC component without JSX/hooks should suggest "use memo"', async () => {
      const { result } = await invalid(dedent`
        import { FC } from 'react'
        
        const Component: FC = () => {
          return 'Hello World'
        }
      `)
      expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
        "
        - messageId: 'fcComponentShouldReturnJsx'
          data: 'React components and hooks should create JSX elements, call other hooks or use the "use memo" directive for optimal React compiler detection.'
          line: 3
        "
      `)
      // Check that suggestions are provided
      expect(result.messages[0]?.suggestions).toBeDefined()
      expect(result.messages[0]?.suggestions?.[0]?.messageId).toBe(
        'addUseMemoDirective',
      )
    })

    test('hook function without JSX/hooks should suggest "use memo"', async () => {
      const { result } = await invalid(dedent`
        function useData() {
          return { data: 'some data' }
        }
      `)
      expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
        "
        - messageId: 'fcComponentShouldReturnJsx'
          data: 'React components and hooks should create JSX elements, call other hooks or use the "use memo" directive for optimal React compiler detection.'
          line: 1
        "
      `)
      // Check that suggestions are provided
      expect(result.messages[0]?.suggestions).toBeDefined()
      expect(result.messages[0]?.suggestions?.[0]?.messageId).toBe(
        'addUseMemoDirective',
      )
    })

    test('FC component with comment directive should still show error', async () => {
      const { result } = await invalid(dedent`
        import { FC } from 'react'
        
        // "use memo" 
        const Component: FC = () => {
          return 'Hello World'
        }
      `)
      expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
        "
        - messageId: 'fcComponentShouldReturnJsx'
          data: 'React components and hooks should create JSX elements, call other hooks or use the "use memo" directive for optimal React compiler detection.'
          line: 4
        "
      `)
      // Comments are ignored, so suggestions should still be provided
      expect(result.messages[0]?.suggestions).toBeDefined()
      expect(result.messages[0]?.suggestions?.[0]?.messageId).toBe(
        'addUseMemoDirective',
      )
    })
  })
})

describe('Functions calling hooks validation', () => {
  test('React component calling hooks is valid', async () => {
    await valid(
      dedent`
        import { FC, useState } from 'react'
        
        const Component: FC = () => {
          const [count] = useState(0)
          return <div>{count}</div>
        }
      `,
    )
  })

  test('Hook function calling hooks is valid', async () => {
    await valid(
      dedent`
        import { useState } from 'react'
        
        function useCounter() {
          const [count, setCount] = useState(0)
          return { count, setCount }
        }
      `,
    )
  })

  test('Regular function not calling hooks should be ignored', async () => {
    await valid(
      dedent`
        function regularFunction() {
          return 'regular function'
        }
      `,
    )
  })

  test('Regular function calling hooks should show error', async () => {
    const { result } = await invalid(dedent`
      import { useState } from 'react'
      
      function regularFunction() {
        const [count] = useState(0)
        return count
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'functionCallingHooksMustBeComponent'
        data: 'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").'
        line: 3
      "
    `)
  })

  test('Arrow function calling hooks should show error', async () => {
    const { result } = await invalid(dedent`
      import { useState } from 'react'
      
      const regularFunction = () => {
        const [count] = useState(0)
        return count
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'functionCallingHooksMustBeComponent'
        data: 'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").'
        line: 3
      "
    `)
  })

  test('Function calling namespaced hooks should show error', async () => {
    const { result } = await invalid(dedent`
      function regularFunction() {
        const data = test.useState(0)
        return data
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'functionCallingHooksMustBeComponent'
        data: 'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").'
        line: 1
      "
    `)
  })

  test('PascalCase function without FC type calling hooks should show error', async () => {
    const { result } = await invalid(dedent`
      import { useState } from 'react'
      
      const Component = () => {
        const [count] = useState(0)
        return count
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'functionCallingHooksMustBeComponent'
        data: 'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").'
        line: 3
      "
    `)
  })

  test('Function expression calling hooks should show error', async () => {
    const { result } = await invalid(dedent`
      import { useState } from 'react'
      
      const obj = {
        method: function() {
          const [count] = useState(0)
          return count
        }
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'functionCallingHooksMustBeComponent'
        data: 'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").'
        line: 4
      "
    `)
  })

  test('Nested function calling hooks should show error', async () => {
    const { result } = await invalid(dedent`
      import { useState } from 'react'
      
      function parentFunction() {
        function nestedFunction() {
          const [count] = useState(0)
          return count
        }
        return nestedFunction()
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'functionCallingHooksMustBeComponent'
        data: 'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").'
        line: 4
      "
    `)
  })

  test('Valid hook function with useCallback', async () => {
    await valid(
      dedent`
        import { useCallback } from 'react'
        
        function useCustomHook() {
          const callback = useCallback(() => {}, [])
          return callback
        }
      `,
    )
  })
})
