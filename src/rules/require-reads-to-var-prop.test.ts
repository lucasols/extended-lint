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
        varName: 'result',
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
        varName: 'result',
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
        varName: 'result',
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
        varName: 'result',
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
        varName: 'result2',
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
  [{ data: { prop: 'isLoading', varName: 'queryResult', customMsg: '' } }],
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
  [{ data: { prop: 'isLoading', varName: 'queryResult', customMsg: '' } }],
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
        varName: 'element',
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
        varName: 'chatMessages',
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
  [{ data: { prop: 'call', varName: 'onConfirm', customMsg: '' } }],
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

tests.run()
