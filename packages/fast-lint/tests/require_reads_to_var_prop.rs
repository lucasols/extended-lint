mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "require-reads-to-var-prop";

#[test]
fn variable_used_any_property_access() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log(result.data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn variable_used_destructuring() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                const { data } = result
                console.log(data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn variable_used_passed_to_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                doSomething(result)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn variable_used_jsx_prop() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                return <Component data={result} />
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn variable_used_in_expression() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                const items = [result, otherItem]
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn no_tracked_variables() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const someOtherVar = someOtherFunction()
                console.log(someOtherVar)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn accessing_any_property_is_fine_when_variable_not_tracked() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = someOtherFunction()
                console.log(result.otherProp)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn works_with_let_declaration() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                let result = createTest()
                console.log(result.data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn works_with_var_declaration() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                var result = createTest()
                console.log(result.data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn usequery_with_isloading_check() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const queryResult = useQuery()
                if (queryResult.isLoading) {
                  return <Spinner />
                }
                console.log(queryResult.data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "isLoading" }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn wildcard_member_call_pattern_used() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const element = api.useElement()
                console.log(element)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useElement", "prop": "data", "errorMsg": "Elements from useElement() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn different_object_same_method_used() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const element = store.useElement()
                doSomething(element)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useElement", "prop": "data", "errorMsg": "Elements from useElement() should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn specific_member_call_pattern_used() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const chatMessages = chatMessagesList.useListQuery()
                return <ChatList messages={chatMessages} />
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "chatMessagesList.useListQuery", "prop": "data", "errorMsg": "Chat messages should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn wrong_object_method_not_tracked() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const other = differentList.useListQuery()
                console.log('not tracked, so no error')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "chatMessagesList.useListQuery", "prop": "data", "errorMsg": "Chat messages should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn simple_function_call_with_fromfncall() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const query = useQuery()
                if (query.isLoading) return <Spinner />
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "isLoading" }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn property_used_in_variable_assignment() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const tableFieldsWithArchived = test.useItem();

                const tableFields = showInTrash ? tableFieldsWithArchived : tableFields_;
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useItem", "prop": "data" }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn property_used_in_object_assignment() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const tableFieldsWithArchived = test.useItem();

                const tableFields = { tableFieldsWithArchived };
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useItem", "prop": "data" }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn property_used_in_nested_object_assignment() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const tableFieldsWithArchived = test.useItem();

                const tableFields = { nested: { tableFieldsWithArchived } };
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useItem", "prop": "data" }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn multiple_props_all_accessed_via_member_expression() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log(result.data)
                console.log(result.loading)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading"], "errorMsg": "Both data and loading should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn multiple_props_all_accessed_via_destructuring() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                const { data, loading } = result
                console.log(data, loading)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading"], "errorMsg": "Both data and loading should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn multiple_props_mixed_access_patterns() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log(result.data)
                const { loading } = result
                console.log(loading)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading"], "errorMsg": "Both data and loading should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn multiple_props_variable_passed_entirely() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                doSomething(result)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading"], "errorMsg": "Both data and loading should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn multiple_props_accessing_different_property_not_required() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log(result.data)
                console.log(result.loading)
                console.log(result.otherProp)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading"], "errorMsg": "Both data and loading should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn direct_destructuring_single_required_prop_present() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const { data } = useQuery()
                console.log(data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data", "errorMsg": "The data should be used." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn direct_destructuring_multiple_required_props_present() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const { data, isLoading } = useQuery()
                console.log(data, isLoading)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": ["data", "isLoading"], "errorMsg": "Both data and isLoading should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn direct_destructuring_wildcard_member_call() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const { data } = api.useElement()
                console.log(data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useElement", "prop": "data", "errorMsg": "Element data should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn direct_destructuring_specific_member_call() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const { data } = chatMessagesList.useListQuery()
                console.log(data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "chatMessagesList.useListQuery", "prop": "data", "errorMsg": "Chat messages data should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn direct_destructuring_extra_props_allowed() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const { data, isLoading, error, extra } = useQuery()
                console.log(data, isLoading, error, extra)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": ["data", "isLoading"], "errorMsg": "Required props should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn different_property_accessed_but_not_required_one() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log(result.otherProp)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "data". The data from createTest() should be used.'
      line: 1
    "#);
}

#[test]
fn detects_missing_property_with_let() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                let result = createTest()
                console.log('no data access')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "data". The data from createTest() should be used.'
      line: 1
    "#);
}

#[test]
fn detects_missing_property_with_var() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                var result = createTest()
                console.log('no data access')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "data". The data from createTest() should be used.'
      line: 1
    "#);
}

#[test]
fn variable_never_used() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log('done')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "data". The data from createTest() should be used.'
      line: 1
    "#);
}

#[test]
fn multiple_variables_some_unused() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const result1 = createTest()
                const result2 = createTest()

                console.log(result1.data)
                console.log('result2 not used')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data", "errorMsg": "The data from createTest() should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "data". The data from createTest() should be used.'
      line: 2
    "#);
}

#[test]
fn usequery_variable_unused() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const queryResult = useQuery()
                console.log('not using queryResult')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "isLoading" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "isLoading". '
      line: 1
    "#);
}

#[test]
fn multiple_selectors_with_mixed_usage() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const testResult = createTest()
                const queryResult = useQuery()

                console.log(testResult.data) // used
                console.log('queryResult not used') // unused
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": "data" }, { "fromFnCall": "useQuery", "prop": "isLoading" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "isLoading". '
      line: 2
    "#);
}

#[test]
fn wildcard_member_call_pattern_unused() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const element = api.useElement()
                console.log('not using element')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useElement", "prop": "data", "errorMsg": "Elements from useElement() should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"*.useElement" requires reads to the prop "data". Elements from useElement() should be used.'
      line: 1
    "#);
}

#[test]
fn specific_member_call_pattern_unused() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const chatMessages = chatMessagesList.useListQuery()
                console.log('messages not used')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "chatMessagesList.useListQuery", "prop": "data", "errorMsg": "Chat messages should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"chatMessagesList.useListQuery" requires reads to the prop "data". Chat messages should be used.'
      line: 1
    "#);
}

#[test]
fn useactionfn_without_call_property_used() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
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
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useActionFn", "prop": "call" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useActionFn" requires reads to the prop "call". '
      line: 8
    "#);
}

#[test]
fn multiple_props_only_one_accessed() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log(result.data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading"], "errorMsg": "Both data and loading should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "loading". Both data and loading should be used.'
      line: 1
    "#);
}

#[test]
fn multiple_props_none_accessed() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                console.log('no props accessed')
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading"], "errorMsg": "Both data and loading should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propsNotRead'
      msg: '"createTest" requires reads to the props "data", "loading". Both data and loading should be used.'
      line: 1
    "#);
}

#[test]
fn multiple_props_partial_destructuring() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const result = createTest()
                const { data } = result
                console.log(data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"CallExpression\"][init.callee.name=\"createTest\"]", "prop": ["data", "loading", "error"], "errorMsg": "All props should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propsNotRead'
      msg: '"createTest" requires reads to the props "loading", "error". All props should be used.'
      line: 1
    "#);
}

#[test]
fn direct_destructuring_single_required_prop_missing() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const { isLoading } = useQuery()
                console.log(isLoading)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data", "errorMsg": "The data should be used." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "data". The data should be used.'
      line: 1
    "#);
}

#[test]
fn direct_destructuring_multiple_required_props_partial() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const { data } = useQuery()
                console.log(data)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": ["data", "isLoading"], "errorMsg": "Both data and isLoading should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "isLoading". Both data and isLoading should be destructured.'
      line: 1
    "#);
}

#[test]
fn direct_destructuring_multiple_required_props_none_present() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const { error } = useQuery()
                console.log(error)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": ["data", "isLoading"], "errorMsg": "Both data and isLoading should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propsNotRead'
      msg: '"useQuery" requires reads to the props "data", "isLoading". Both data and isLoading should be destructured.'
      line: 1
    "#);
}

#[test]
fn direct_destructuring_wildcard_member_call_missing_prop() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const { status } = store.useElement()
                console.log(status)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useElement", "prop": "data", "errorMsg": "Element data should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"*.useElement" requires reads to the prop "data". Element data should be destructured.'
      line: 1
    "#);
}

#[test]
fn direct_destructuring_specific_member_call_missing_prop() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const { loading } = chatMessagesList.useListQuery()
                console.log(loading)
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "chatMessagesList.useListQuery", "prop": "data", "errorMsg": "Chat messages data should be destructured." }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"chatMessagesList.useListQuery" requires reads to the prop "data". Chat messages data should be destructured.'
      line: 1
    "#);
}

#[test]
fn extra_optional_member_read() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const r = useQuery()
                r?.data
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_string_element_access() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const r = useQuery()
                r['data']
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_identifier_element_access() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const r = useQuery()
                r[data]
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_same_prop_read_twice() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const r = useQuery()
                r.data
                r.data
            "},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": ["data", "isLoading"] }] }),
            ),
            ..Default::default()
        },
    );
}

#[test]
fn extra_pattern_as_argument() {
    let result = invalid(
        RULE,
        Case {
            code: r"const r = fn(useQuery)",
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "data". '
      line: 1
    "#);
}

#[test]
fn extra_renamed_and_rest_destructuring() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const { data: d } = useQuery()
                const { ...rest } = useQuery()
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "data". '
      line: 2
    "#);
}

#[test]
fn extra_await_and_parens() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                async function f() {
                  const a = await useQuery()
                  const b = (useQuery)()
                  const c = useQuery?.()
                }
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "data". '
      line: 3
    "#);
}

#[test]
fn extra_same_name_in_two_scopes() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                function a() {
                  const r = useQuery()
                  r.isLoading
                }
                function b() {
                  const r = useQuery()
                }
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "data". '
      line: 6
    "#);
}

#[test]
fn extra_same_name_in_two_scopes_reversed() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                function a() {
                  const r = useQuery()
                }
                function b() {
                  const r = useQuery()
                  r.data
                }
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_var_in_block() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                if (x) {
                  var r = useQuery()
                }
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_selector_attributes() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const result = api.load()
                const other = api.load()
                const x = load()
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.callee.object.name=\"api\"][id.name!=\"other\"]", "prop": "data" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"api.load" requires reads to the prop "data". '
      line: 1
    "#);
}

#[test]
fn extra_selector_and_fromfncall_together() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const { a } = createTest()
                const r = createTest()
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.callee.name=\"createTest\"]", "prop": "data" }, { "fromFnCall": "createTest", "prop": "other" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "data". '
      line: 1
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "other". '
      line: 1
    - messageId: 'propNotRead'
      msg: '"createTest" requires reads to the prop "other". '
      line: 2
    "#);
}

#[test]
fn extra_wildcard_and_specific() {
    let result = invalid(
        RULE,
        Case {
            code: r"const { x } = api.useList()",
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useList", "prop": "items" }, { "fromFnCall": "api.useList", "prop": "data" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"*.useList" requires reads to the prop "items". '
      line: 1
    - messageId: 'propNotRead'
      msg: '"api.useList" requires reads to the prop "data". '
      line: 1
    "#);
}

#[test]
fn extra_typeof_usage() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const r = useQuery()
                type T = typeof r
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_reassignment() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                let r = useQuery()
                r = other()
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_destructuring_of_cast() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const r = useQuery()
                const { data } = r as X
            "},
            options: Some(json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": "data" }] })),
            ..Default::default()
        },
    );
}

#[test]
fn extra_destructuring_with_computed_key() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const r = useQuery()
                const { [data]: d } = r
                const { "isLoading": l } = r
            "#},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "useQuery", "prop": ["data", "isLoading"] }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"useQuery" requires reads to the prop "isLoading". '
      line: 1
    "#);
}

#[test]
fn extra_selector_without_call() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const r = 1
                const s = foo.bar
            "},
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"Literal\"]", "prop": "data" }, { "selector": "VariableDeclarator[init.type=MemberExpression]", "prop": "x" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"unknown function" requires reads to the prop "data". '
      line: 1
    - messageId: 'propNotRead'
      msg: '"unknown function" requires reads to the prop "x". '
      line: 2
    "#);
}

#[test]
fn extra_computed_method_name() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const a = api[useList]()
                const b = api["useList"]()
            "#},
            options: Some(
                json!({ "varsToCheck": [{ "fromFnCall": "*.useList", "prop": "items" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"*.useList" requires reads to the prop "items". '
      line: 1
    "#);
}

#[test]
fn extra_fn_name_unknown() {
    let result = invalid(
        RULE,
        Case {
            code: r"const r = (await x)",
            options: Some(
                json!({ "varsToCheck": [{ "selector": "VariableDeclarator[init.type=\"AwaitExpression\"]", "prop": "data" }] }),
            ),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'propNotRead'
      msg: '"unknown function" requires reads to the prop "data". '
      line: 1
    "#);
}

#[test]
fn unsupported_selector_is_a_config_error() {
    let error = fast_lint::rules::create_rule(
        RULE,
        Some(&json!({ "varsToCheck": [{ "selector": "CallExpression > Identifier", "prop": "data" }] })),
    )
    .err();
    assert_snapshot!(error.unwrap_or_default(), @r#"invalid options for rule require-reads-to-var-prop: unsupported selector "CallExpression > Identifier": only `VariableDeclarator` with attribute filters (`[init.callee.name="fn"]`, `!=`, `[path]`) is supported"#);
}
