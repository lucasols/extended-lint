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

describe('FC component return type checking', () => {
  test('FC component returning JSX is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const Component: FC = () => {
          return <div>Hello</div>
        }
      `,
    )
  })

  test('React.FC component returning JSX is valid', async () => {
    await valid(
      dedent`
        import React from 'react'
        
        const Component: React.FC = () => {
          return <div>Hello</div>
        }
      `,
    )
  })

  test('FC component with conditional JSX return is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const Component: FC = () => {
          return true ? <div>Hello</div> : <span>World</span>
        }
      `,
    )
  })

  test('FC component with logical AND JSX return is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const Component: FC = () => {
          return true && <div>Hello</div>
        }
      `,
    )
  })

  test('FC component with React.createElement is valid', async () => {
    await valid(
      dedent`
        import React, { FC } from 'react'
        
        const Component: FC = () => {
          return React.createElement('div', null, 'Hello')
        }
      `,
    )
  })

  test('FC component with early return JSX is valid', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const condition = true
        
        const Component: FC = () => {
          if (condition) {
            return <div>Early JSX return</div>
          }
          return <span>Normal JSX return</span>
        }
      `,
    )
  })

  test('FC component returning string should show error', async () => {
    const { result } = await invalid(dedent`
      import { FC } from 'react'
      
      const Component: FC = () => {
        return 'Hello World'
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React.FC components should return JSX elements for optimal React compiler detection. Consider wrapping the return value in a fragment.'
        line: 3
      "
    `)
  })

  test('React.FC component returning number should show error', async () => {
    const { result } = await invalid(dedent`
      import React from 'react'
      
      const Component: React.FC = () => {
        return 42
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React.FC components should return JSX elements for optimal React compiler detection. Consider wrapping the return value in a fragment.'
        line: 3
      "
    `)
  })

  test('FC component returning object should show error', async () => {
    const { result } = await invalid(dedent`
      import { FC } from 'react'
      
      const Component: FC = () => {
        return { message: 'Hello' }
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React.FC components should return JSX elements for optimal React compiler detection. Consider wrapping the return value in a fragment.'
        line: 3
      "
    `)
  })

  test('FC component with arrow function expression returning string should show error', async () => {
    const { result } = await invalid(dedent`
      import { FC } from 'react'
      
      const Component: FC = () => 'Hello World'
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React.FC components should return JSX elements for optimal React compiler detection. Consider wrapping the return value in a fragment.'
        line: 3
      "
    `)
  })

  test('FC component returning mixed conditional with JSX branch is valid', async () => {
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

  test('FC component with multiple return statements, some non-JSX is valid when at least one returns JSX', async () => {
    await valid(
      dedent`
        import { FC } from 'react'
        
        const condition = true
        
        const Component: FC = () => {
          if (condition) {
            return 'Early return'
          }
          return <div>Normal return</div>
        }
      `,
    )
  })

  test('FC component with multiple return statements, all non-JSX should show error', async () => {
    const { result } = await invalid(dedent`
      import { FC } from 'react'
      
      const condition = true
      
      const Component: FC = () => {
        if (condition) {
          return 'Early return'
        }
        return 'Normal return'
      }
    `)
    expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'fcComponentShouldReturnJsx'
        data: 'React.FC components should return JSX elements for optimal React compiler detection. Consider wrapping the return value in a fragment.'
        line: 5
      "
    `)
  })

  test('regular function without FC type should not be checked', async () => {
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
})
