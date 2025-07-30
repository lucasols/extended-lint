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

tests.addValid(
  'variable used - different property access',
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

tests.addInvalid(
  'detects missing property with let',
  `
    let result = createTest()
    console.log('no data access')
  `,
  [
    {
      data: {
        varName: 'result',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
  {
    options: {
      varsToCheck: [
        {
          selector:
            'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
          prop: 'data',
          errorMsg: 'The data from createTest() should be used.',
        },
      ],
    },
  },
)

tests.addInvalid(
  'detects missing property with var',
  `
    var result = createTest()
    console.log('no data access')
  `,
  [
    {
      data: {
        varName: 'result',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
  {
    options: {
      varsToCheck: [
        {
          selector:
            'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
          prop: 'data',
          errorMsg: 'The data from createTest() should be used.',
        },
      ],
    },
  },
)

// Invalid cases - only when variable is completely unused
tests.addInvalid(
  'variable never used',
  `
    const result = createTest()
    console.log('done')
  `,
  [
    {
      data: {
        varName: 'result',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
  {
    options: {
      varsToCheck: [
        {
          selector:
            'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
          prop: 'data',
          errorMsg: 'The data from createTest() should be used.',
        },
      ],
    },
  },
)

tests.addInvalid(
  'multiple variables, some unused',
  `
    const result1 = createTest()
    const result2 = createTest()
    
    console.log(result1.data)
    console.log('result2 not used')
  `,
  [
    {
      data: {
        prop: 'data',
        varName: 'result2',
        customMsg: 'The data from createTest() should be used.',
      },
    },
  ],
  {
    options: {
      varsToCheck: [
        {
          selector:
            'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
          prop: 'data',
          errorMsg: 'The data from createTest() should be used.',
        },
      ],
    },
  },
)

// Test with different selector and property
tests.addInvalid(
  'useQuery variable unused',
  `
    const queryResult = useQuery()
    console.log('not using queryResult')
  `,
  [{ data: { varName: 'queryResult', customMsg: '' } }],
  {
    options: {
      varsToCheck: [
        {
          fromFnCall: 'useQuery',
          prop: 'isLoading',
        },
      ],
    },
  },
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

tests.addInvalid(
  'multiple selectors with mixed usage',
  `
    const testResult = createTest()
    const queryResult = useQuery()
    
    console.log(testResult.data) // used
    console.log('queryResult not used') // unused
  `,
  [{ data: { varName: 'queryResult', customMsg: '' } }],
  {
    options: {
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
  },
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

tests.addInvalid(
  'wildcard member call pattern unused',
  `
    const element = api.useElement()
    console.log('not using element')
  `,
  [
    {
      data: {
        varName: 'element',
        customMsg: 'Elements from useElement() should be used.',
      },
    },
  ],
  {
    options: {
      varsToCheck: [
        {
          fromFnCall: '*.useElement',
          prop: 'data',
          errorMsg: 'Elements from useElement() should be used.',
        },
      ],
    },
  },
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

tests.addInvalid(
  'specific member call pattern unused',
  `
    const chatMessages = chatMessagesList.useListQuery()
    console.log('messages not used')
  `,
  [
    {
      data: {
        varName: 'chatMessages',
        customMsg: 'Chat messages should be used.',
      },
    },
  ],
  {
    options: {
      varsToCheck: [
        {
          fromFnCall: 'chatMessagesList.useListQuery',
          prop: 'data',
          errorMsg: 'Chat messages should be used.',
        },
      ],
    },
  },
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
  'useActionFn with isLoading check',
  `
    import { css } from '@linaria/core';
    import { ActionModalContent } from '@src/_components/ActionModal';
    import { RichTextVisualizer } from '@src/_components/RichTextVisualizer';
    import { NewTextField } from '@src/components/newInputs/TextField/NewTextField';
    import { colorScheme } from '@src/style/lightDark';
    import { cx } from '@utils/cx';
    import { useActionFn } from '@utils/hooks/useAsyncActionFn';
    import { __ } from '@utils/i18n/i18n';
    import { type FC, useState } from 'react';
    import type { Result } from 't-result';

    type Props = {
      title: string | null;
      description?: string;
      placeholder?: string;
      inputType?: 'text' | 'number';
      confirmBtnLabel: string;
      variant?: 'danger' | 'default';
      onConfirm: (value: string) => Promise<Result<void, { retryErrMsg: string }>>;
      onClose: () => void;
      initialValue?: string;
      optional?: boolean;
      isValid?: (value: string) => true | string;
      hint?: string;
      headerTitle?: string;
    };

    export const PromptModalContent: FC<Props> = ({
      title,
      description,
      placeholder,
      inputType,
      confirmBtnLabel,
      variant,
      onConfirm: onConfirmFn,
      onClose,
      initialValue,
      optional,
      isValid: isValidFn,
      hint,
      headerTitle,
    }) => {
      const [inputValue, setInputValue] = useState(initialValue ?? '');

      const [forceShowErrors, setForceShowErrors] = useState(false);

      const [confirmError, setConfirmError] = useState<string | undefined>(
        undefined,
      );

      const requiredError =
        !optional && !inputValue.trim() ? 'This field is required' : undefined;

      const isValid = isValidFn?.(inputValue);
      const validationError = isValid === true ? undefined : isValid;

      const onConfirm = useActionFn(async (newValue: string) => {
        const result = await onConfirmFn(newValue.trim());

        if (result.error) {
          setConfirmError(result.error.retryErrMsg);
        }
      });

      const hasError = !!confirmError || !!requiredError || !!validationError;

      return (
        <ActionModalContent
          title={title}
          header={headerTitle}
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
                debounce={0}
                placeholder={placeholder}
                type={inputType}
                showErrorOnFocus={!!confirmError || forceShowErrors}
                hint={hint}
                errors={
                  confirmError ? confirmError
                  : forceShowErrors && requiredError ?
                    requiredError
                  : validationError ?
                    validationError
                  : undefined
                }
              />
            </>
          }
          actionLabel={confirmBtnLabel}
          onClose={onClose}
          confirmButtonColor={variant === 'danger' ? 'newCustomRed' : 'newPrimary'}
          closeOnConfirm={false}
          disableButton={hasError || onConfirm.isInProgress}
          isConfirming={onConfirm.isInProgress}
          onClickDisabled={() => {
            setForceShowErrors(true);
          }}
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
)

tests.run()
