import { createTester } from '../../tests/utils/createTester'
import { reactCompilerMigration } from '../react-compiler-extra'

const tests = createTester(reactCompilerMigration)

tests.addValid(
  'non-hook call with object method',
  `
    const result = someFunction({
      method() {
        return 42
      }
    });
  `,
)

tests.addValid(
  'hook call without object method',
  `
    const result = useState({
      method: () => {
        return 42
      }
    });
  `,
)

tests.addValid(
  'hook call with function expression',
  `
    const result = useCallback({
      method: () => {
        return 42
      }
    }, []);
  `,
)

tests.addInvalid(
  'hook call with object method',
  `
    const result = useState({
      method() {
        return 42
      }
    });
  `,
  [{ messageId: 'objectMethodIsNotSupported' }],
  {
    output: `
    const result = useState({
      method: () => {
        return 42
      }
    });
  `,
  },
)

tests.addInvalid(
  'hook call with object method using this',
  `
    const result = useState({
      method() {
        this.value = 42;
        return this.value;
      }
    });
  `,
  [{ messageId: 'thisKeywordInMethod' }],
)

tests.addInvalid(
  'hook call with nested this usage',
  `
    const result = useState({
      method() {
        if (true) {
          console.log(this.value);
        }
        return 42;
      }
    });
  `,
  [{ messageId: 'thisKeywordInMethod' }],
)

tests.addInvalid(
  'useCallback with object method',
  `
    const result = useCallback({
      method() {
        return 42
      }
    }, []);
  `,
  [{ messageId: 'objectMethodIsNotSupported' }],
  {
    output: `
    const result = useCallback({
      method: () => {
        return 42
      }
    }, []);
  `,
  },
)

tests.addInvalid(
  'useMemo with object method',
  `
    const result = useMemo(() => ({
      method() {
        return 42
      }
    }), []);
  `,
  [{ messageId: 'objectMethodIsNotSupported' }],
  {
    output: `
    const result = useMemo(() => ({
      method: () => {
        return 42
      }
    }), []);
  `,
  },
)

tests.addInvalid(
  'multiple object methods in hook call',
  `
    const result = useCallback({
      method1() {
        return 42
      },
      method2() {
        console.log('hello')
      }
    }, []);
  `,
  [
    { messageId: 'objectMethodIsNotSupported' },
    { messageId: 'objectMethodIsNotSupported' },
  ],
  {
    output: `
    const result = useCallback({
      method1: () => {
        return 42
      },
      method2: () => {
        console.log('hello')
      }
    }, []);
  `,
  },
)

tests.addInvalid(
  'mixed methods with and without this',
  `
    const result = useCallback({
      method1() {
        return 42;
      },
      method2() {
        return this.value;
      }
    }, []);
  `,
  [
    { messageId: 'objectMethodIsNotSupported' },
    { messageId: 'thisKeywordInMethod' },
  ],
  {
    output: `
    const result = useCallback({
      method1: () => {
        return 42;
      },
      method2() {
        return this.value;
      }
    }, []);
  `,
  },
)

tests.addInvalid(
  'hook call with generator method',
  `
    const result = useMemo(() => ({
      *generator() {
        yield 42
      }
    }), []);
  `,
  [{ messageId: 'objectMethodIsNotSupported' }],
  {
    output: `
    const result = useMemo(() => ({
      generator: function* () {
        yield 42
      }
    }), []);
  `,
  },
)

tests.addInvalid(
  'hook call with object method with parameters',
  `
    const result = useState({
      method(a, b, c) {
        return a + b + c
      }
    });
  `,
  [{ messageId: 'objectMethodIsNotSupported' }],
  {
    output: `
    const result = useState({
      method: (a, b, c) => {
        return a + b + c
      }
    });
  `,
  },
)

tests.addValid(
  'nested hook call with object method is ok',
  `
    const result = useState({
      outer: {
        inner() {
          return 42
        }
      }
    });
  `,
)

tests.addInvalid(
  'nested hook call with object method using this',
  `
    const result = useState({
      outer: {
        inner() {
          return this.value
        }
      }
    });
  `,
  [{ messageId: 'thisKeywordInMethod' }],
)

tests.describe('with runOnlyWithEnableCompilerDirective option', () => {
  tests.addValid(
    'hook call with object method without directive',
    `
      const result = useState({
        method() {
          return 42
        }
      });
    `,
    { runOnlyWithEnableCompilerDirective: true },
  )

  tests.addInvalidWithOptions(
    'hook call with object method with directive',
    `
      // eslint react-compiler/react-compiler: ["error"]
      const result = useState({
        method() {
          return 42
        }
      });
    `,
    { runOnlyWithEnableCompilerDirective: true },
    [{ messageId: 'objectMethodIsNotSupported' }],
    {
      output: `
      // eslint react-compiler/react-compiler: ["error"]
      const result = useState({
        method: () => {
          return 42
        }
      });
    `,
    },
  )

  tests.addInvalidWithOptions(
    'hook call with object method using this with directive',
    `
      // eslint react-compiler/react-compiler: ["error"]
      const result = useState({
        method() {
          return this.value
        }
      });
    `,
    { runOnlyWithEnableCompilerDirective: true },
    [{ messageId: 'thisKeywordInMethod' }],
  )
})

// Run the tests
tests.run()
