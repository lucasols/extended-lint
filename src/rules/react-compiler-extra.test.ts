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
