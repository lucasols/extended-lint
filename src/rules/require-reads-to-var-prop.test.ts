import { createTester } from '../../tests/utils/createTester'
import { requireReadsToVarProp } from './require-reads-to-var-prop'

const tests = createTester(requireReadsToVarProp, {
  defaultErrorId: 'propNotRead',
})

// Valid cases
tests.addValid(
  'variable used - any property access',
  `
    const result = createTest()
    console.log(result.data)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addValid(
  'variable used - destructuring',
  `
    const result = createTest()
    const { data } = result
    console.log(data)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'different property accessed but not required one',
  `
    const result = createTest()
    console.log(result.otherProp)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'createTest',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
)

tests.addValid(
  'variable used - passed to function',
  `
    const result = createTest()
    doSomething(result)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addValid(
  'variable used - JSX prop',
  `
    const result = createTest()
    return <Component data={result} />
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addValid(
  'variable used - in expression',
  `
    const result = createTest()
    const items = [result, otherItem]
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addValid(
  'no tracked variables',
  `
    const someOtherVar = someOtherFunction()
    console.log(someOtherVar)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addValid(
  'accessing any property is fine when variable not tracked',
  `
    const result = someOtherFunction()
    console.log(result.otherProp)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addValid(
  'works with let declaration',
  `
    let result = createTest()
    console.log(result.data)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addValid(
  'works with var declaration',
  `
    var result = createTest()
    console.log(result.data)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'detects missing property with let',
  `
    let result = createTest()
    console.log('no data access')
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'createTest',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
)

tests.addInvalidWithOptions(
  'detects missing property with var',
  `
    var result = createTest()
    console.log('no data access')
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'createTest',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
)

// Invalid cases - only when variable is completely unused
tests.addInvalidWithOptions(
  'variable never used',
  `
    const result = createTest()
    console.log('done')
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'createTest',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
)

tests.addInvalidWithOptions(
  'multiple variables, some unused',
  `
    const result1 = createTest()
    const result2 = createTest()
    
    console.log(result1.data)
    console.log('result2 not used')
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
        errorMsg: 'The data from createTest() should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'createTest',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
)

// Test with different selector and property
tests.addInvalidWithOptions(
  'useQuery variable unused',
  `
    const queryResult = useQuery()
    console.log('not using queryResult')
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: 'isLoading',
      },
    ],
  },
  [{ data: { prop: 'isLoading', fnName: 'useQuery', customMsg: '' } }],
)

tests.addValid(
  'useQuery with isLoading check',
  `
    const queryResult = useQuery()
    if (queryResult.isLoading) {
      return <Spinner />
    }
    console.log(queryResult.data)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: 'isLoading',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'multiple selectors with mixed usage',
  `
    const testResult = createTest()
    const queryResult = useQuery()
    
    console.log(testResult.data) // used
    console.log('queryResult not used') // unused
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: 'data',
      },
      {
        fromFnCall: 'useQuery',
        prop: 'isLoading',
      },
    ],
  },
  [{ data: { prop: 'isLoading', fnName: 'useQuery', customMsg: '' } }],
)

// Test fromFnCall patterns
tests.addValid(
  'wildcard member call pattern used',
  `
    const element = api.useElement()
    console.log(element)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: '*.useElement',
        prop: 'data',
        errorMsg: 'Elements from useElement() should be used.',
      },
    ],
  },
)

tests.addValid(
  'different object same method used',
  `
    const element = store.useElement()
    doSomething(element)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: '*.useElement',
        prop: 'data',
        errorMsg: 'Elements from useElement() should be used.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'wildcard member call pattern unused',
  `
    const element = api.useElement()
    console.log('not using element')
  `,
  {
    varsToCheck: [
      {
        fromFnCall: '*.useElement',
        prop: 'data',
        errorMsg: 'Elements from useElement() should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: '*.useElement',
        customMsg: 'Elements from useElement() should be used.',
      },
    },
  ],
)

tests.addValid(
  'specific member call pattern used',
  `
    const chatMessages = chatMessagesList.useListQuery()
    return <ChatList messages={chatMessages} />
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'chatMessagesList.useListQuery',
        prop: 'data',
        errorMsg: 'Chat messages should be used.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'specific member call pattern unused',
  `
    const chatMessages = chatMessagesList.useListQuery()
    console.log('messages not used')
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'chatMessagesList.useListQuery',
        prop: 'data',
        errorMsg: 'Chat messages should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'chatMessagesList.useListQuery',
        customMsg: 'Chat messages should be used.',
      },
    },
  ],
)

tests.addValid(
  'wrong object method not tracked',
  `
    const other = differentList.useListQuery()
    console.log('not tracked, so no error')
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'chatMessagesList.useListQuery',
        prop: 'data',
        errorMsg: 'Chat messages should be used.',
      },
    ],
  },
)

tests.addValid(
  'simple function call with fromFnCall',
  `
    const query = useQuery()
    if (query.isLoading) return <Spinner />
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: 'isLoading',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'useActionFn without call property used',
  `
    type Props = {
      title: string | null;
    };

    export const PromptModalContent: FC<Props> = ({
      title,
    }) => {
      const onConfirm = useActionFn(async (newValue: string) => {
        const result = await onConfirmFn(newValue.trim());

        if (result.error) {
          setConfirmError(result.error.retryErrMsg);
        }
      });

      const hasError = !!confirmError || !!requiredError || !!validationError;

      return (
        <ActionModalContent
          description={
            <>
              {description && <RichTextVisualizer markdown={description} />}
              <NewTextField
                autoFocus
                label={null}
                value={inputValue}
                onChange={(latestValue) => {
                  setInputValue(latestValue);
                  setConfirmError(undefined);
                }}
                onPressEnter={(latestValue) => {
                  if (!hasError) {
                    const validLatestValue = latestValue.trim();

                    if (isValidFn && isValidFn(validLatestValue) !== true) {
                      return true;
                    }

                    // onConfirm.call(validLatestValue);
                  }

                  setForceShowErrors(true);

                  return true;
                }}
                onKeyDown={(e) => {
                  if (onConfirm.isInProgress) {
                    e.preventDefault();
                  }
                }}
                showErrorOnFocus={!!confirmError || forceShowErrors}
              />
            </>
          }
          disableButton={hasError || onConfirm.isInProgress}
          isConfirming={onConfirm.isInProgress}
          onConfirm={() => {
            // onConfirm.call(inputValue);
          }}
        />
      );
    };
  `,
  {
    varsToCheck: [{ fromFnCall: 'useActionFn', prop: 'call' }],
  },
  [{ data: { prop: 'call', fnName: 'useActionFn', customMsg: '' } }],
)

tests.addValid(
  'property used in variable assignment',
  `
    const tableFieldsWithArchived = test.useItem();

    const tableFields = showInTrash ? tableFieldsWithArchived : tableFields_;
  `,
  {
    varsToCheck: [{ fromFnCall: '*.useItem', prop: 'data' }],
  },
)

tests.addValid(
  'property used in object assignment',
  `
    const tableFieldsWithArchived = test.useItem();

    const tableFields = { tableFieldsWithArchived };
  `,
  {
    varsToCheck: [{ fromFnCall: '*.useItem', prop: 'data' }],
  },
)

tests.addValid(
  'property used in nested object assignment',
  `
    const tableFieldsWithArchived = test.useItem();

    const tableFields = { nested: { tableFieldsWithArchived } };
  `,
  {
    varsToCheck: [{ fromFnCall: '*.useItem', prop: 'data' }],
  },
)

tests.addValid(
  'multiple props - all accessed via member expression',
  `
    const result = createTest()
    console.log(result.data)
    console.log(result.loading)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading'],
        errorMsg: 'Both data and loading should be used.',
      },
    ],
  },
)

tests.addValid(
  'multiple props - all accessed via destructuring',
  `
    const result = createTest()
    const { data, loading } = result
    console.log(data, loading)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading'],
        errorMsg: 'Both data and loading should be used.',
      },
    ],
  },
)

tests.addValid(
  'multiple props - mixed access patterns',
  `
    const result = createTest()
    console.log(result.data)
    const { loading } = result
    console.log(loading)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading'],
        errorMsg: 'Both data and loading should be used.',
      },
    ],
  },
)

tests.addValid(
  'multiple props - variable passed entirely',
  `
    const result = createTest()
    doSomething(result)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading'],
        errorMsg: 'Both data and loading should be used.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'multiple props - only one accessed',
  `
    const result = createTest()
    console.log(result.data)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading'],
        errorMsg: 'Both data and loading should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'loading',
        fnName: 'createTest',
        customMsg: 'Both data and loading should be used.',
      },
    },
  ],
)

tests.addInvalidWithOptions(
  'multiple props - none accessed',
  `
    const result = createTest()
    console.log('no props accessed')
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading'],
        errorMsg: 'Both data and loading should be used.',
      },
    ],
  },
  [
    {
      messageId: 'propsNotRead',
      data: {
        props: '"data", "loading"',
        fnName: 'createTest',
        customMsg: 'Both data and loading should be used.',
      },
    },
  ],
)

tests.addInvalidWithOptions(
  'multiple props - partial destructuring',
  `
    const result = createTest()
    const { data } = result
    console.log(data)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading', 'error'],
        errorMsg: 'All props should be used.',
      },
    ],
  },
  [
    {
      messageId: 'propsNotRead',
      data: {
        props: '"loading", "error"',
        fnName: 'createTest',
        customMsg: 'All props should be used.',
      },
    },
  ],
)

tests.addValid(
  'multiple props - accessing different property not required',
  `
    const result = createTest()
    console.log(result.data)
    console.log(result.loading)
    console.log(result.otherProp)
  `,
  {
    varsToCheck: [
      {
        selector:
          'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
        prop: ['data', 'loading'],
        errorMsg: 'Both data and loading should be used.',
      },
    ],
  },
)

// Direct destructuring tests
tests.addValid(
  'direct destructuring - single required prop present',
  `
    const { data } = useQuery()
    console.log(data)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: 'data',
        errorMsg: 'The data should be used.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'direct destructuring - single required prop missing',
  `
    const { isLoading } = useQuery()
    console.log(isLoading)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: 'data',
        errorMsg: 'The data should be used.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'useQuery',
        customMsg: 'The data should be used.',
      },
    },
  ],
)

tests.addValid(
  'direct destructuring - multiple required props present',
  `
    const { data, isLoading } = useQuery()
    console.log(data, isLoading)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: ['data', 'isLoading'],
        errorMsg: 'Both data and isLoading should be destructured.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'direct destructuring - multiple required props partial',
  `
    const { data } = useQuery()
    console.log(data)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: ['data', 'isLoading'],
        errorMsg: 'Both data and isLoading should be destructured.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'isLoading',
        fnName: 'useQuery',
        customMsg: 'Both data and isLoading should be destructured.',
      },
    },
  ],
)

tests.addInvalidWithOptions(
  'direct destructuring - multiple required props none present',
  `
    const { error } = useQuery()
    console.log(error)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: ['data', 'isLoading'],
        errorMsg: 'Both data and isLoading should be destructured.',
      },
    ],
  },
  [
    {
      messageId: 'propsNotRead',
      data: {
        props: '"data", "isLoading"',
        fnName: 'useQuery',
        customMsg: 'Both data and isLoading should be destructured.',
      },
    },
  ],
)

tests.addValid(
  'direct destructuring - wildcard member call',
  `
    const { data } = api.useElement()
    console.log(data)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: '*.useElement',
        prop: 'data',
        errorMsg: 'Element data should be destructured.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'direct destructuring - wildcard member call missing prop',
  `
    const { status } = store.useElement()
    console.log(status)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: '*.useElement',
        prop: 'data',
        errorMsg: 'Element data should be destructured.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: '*.useElement',
        customMsg: 'Element data should be destructured.',
      },
    },
  ],
)

tests.addValid(
  'direct destructuring - specific member call',
  `
    const { data } = chatMessagesList.useListQuery()
    console.log(data)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'chatMessagesList.useListQuery',
        prop: 'data',
        errorMsg: 'Chat messages data should be destructured.',
      },
    ],
  },
)

tests.addInvalidWithOptions(
  'direct destructuring - specific member call missing prop',
  `
    const { loading } = chatMessagesList.useListQuery()
    console.log(loading)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'chatMessagesList.useListQuery',
        prop: 'data',
        errorMsg: 'Chat messages data should be destructured.',
      },
    ],
  },
  [
    {
      data: {
        prop: 'data',
        fnName: 'chatMessagesList.useListQuery',
        customMsg: 'Chat messages data should be destructured.',
      },
    },
  ],
)

tests.addValid(
  'direct destructuring - extra props allowed',
  `
    const { data, isLoading, error, extra } = useQuery()
    console.log(data, isLoading, error, extra)
  `,
  {
    varsToCheck: [
      {
        fromFnCall: 'useQuery',
        prop: ['data', 'isLoading'],
        errorMsg: 'Required props should be destructured.',
      },
    ],
  },
)

tests.run()
