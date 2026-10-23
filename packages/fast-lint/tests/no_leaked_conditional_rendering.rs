mod common;

use common::{invalid, lint, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-leaked-conditional-rendering";

fn react_version(version: &str) -> Option<serde_json::Value> {
    Some(json!({ "reactVersion": version }))
}

#[test]
fn empty_string_literal_before_react_18() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                /// <reference types="react" />
                /// <reference types="react-dom" />

                const a = <>{"" && <Something />}</>;
            "#},
            options: react_version("17.0.0"),
            filename: None,
        },
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value "" that might cause unintentionally rendered values or rendering crashes.'
      line: 4
    "###);
}

#[test]
fn empty_string_variable_before_react_18() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                /// <reference types="react" />
                /// <reference types="react-dom" />

                const someString = "";
                const a = <>{someString && <Something />}</>;
            "#},
            options: react_version("17.0.0"),
            filename: None,
        },
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 5 }");
}

#[test]
fn possibly_empty_string_variable_before_react_18() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                /// <reference types="react" />
                /// <reference types="react-dom" />

                const anyString = Math.random() > 0.5 ? "" : "foo";
                const a = <>{anyString && <Something />}</>;
            "#},
            options: react_version("17.0.0"),
            filename: None,
        },
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 5 }");
}

#[test]
fn zero_and_nan_literals() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const a = <>{0 && <Foo />}</>;
            const b = <>{NaN && <Foo />}</>;
        "#},
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value 0 that might cause unintentionally rendered values or rendering crashes.'
      line: 4
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value NaN that might cause unintentionally rendered values or rendering crashes.'
      line: 5
    "###);
}

#[test]
fn parenthesized_zero_and_nan() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const a = <>{(0) && <Foo />}</>;
            const b = <>{(NaN) && <Foo />}</>;
        "#},
    );

    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'noLeakedConditionalRendering', line: 4, column: 15, endLine: 4, endColumn: 16 }
    - { messageId: 'noLeakedConditionalRendering', line: 5, column: 15, endLine: 5, endColumn: 18 }
    "###);
}

#[test]
fn zero_and_negative_zero_number_variables() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const x = -1
            const y = -0
            const z = 0
            const w = 1
            const a = <>{x && <Foo />}</>;
            const b = <>{y && <Foo />}</>;
            const c = <>{z && <Foo />}</>;
            const d = <>{w && <Foo />}</>;
        "#},
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value y that might cause unintentionally rendered values or rendering crashes.'
      line: 9
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value z that might cause unintentionally rendered values or rendering crashes.'
      line: 10
    "###);
}

#[test]
fn zero_and_negative_zero_bigint_variables() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const x = -1n
            const y = -0n
            const z = 0n
            const w = 1n
            const a = <>{x && <Foo />}</>;
            const b = <>{y && <Foo />}</>;
            const c = <>{z && <Foo />}</>;
            const d = <>{w && <Foo />}</>;
        "#},
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value y that might cause unintentionally rendered values or rendering crashes.'
      line: 9
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value z that might cause unintentionally rendered values or rendering crashes.'
      line: 10
    "###);
}

#[test]
fn unknown_condition() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = JSON.parse("") as unknown;
            const SomeComponent = () => <div />;
            const a = <>{someCondition && <SomeComponent prop1={val1} prop2={val2} />}</>;
        "#},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 6 }");
}

#[test]
fn zero_condition_variable() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = 0;
            const SomeComponent = () => <div />;
            const a = <>{someCondition && <SomeComponent prop1={val1} prop2={val2} />}</>;
        "#},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 6 }");
}

#[test]
fn negative_zero_condition_variable() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = -0;
            const SomeComponent = () => <div />;
            const a = <>{someCondition && <SomeComponent prop1={val1} prop2={val2} />}</>;
        "#},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 6 }");
}

#[test]
fn zero_in_ternary_alternate() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = 0;
            const SomeComponent = () => <div />;
            const a = <>{!!someCondition ? <SomeComponent prop1={val1} prop2={val2} /> : someCondition && <div />}</>;
        "#},
    );

    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'noLeakedConditionalRendering', line: 6, column: 78, endLine: 6, endColumn: 91 }");
}

#[test]
fn number_or_undefined_prop() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const App = ({
              someCondition,
            }: {
              someCondition: number | undefined;
            }) => {
              return (
                <>
                  {someCondition && <Foo />}
                </>
              )
            }
        "#},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 11 }");
}

#[test]
fn optional_number_prop() {
    let result = invalid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const SomeComponent = () => <div />;

            const App = ({
              someCondition,
            }:{
              someCondition?: number | undefined;
            }) => {
              return (
                <>
                  {someCondition && <SomeComponent />}
                </>
              )
            }
        "#},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 13 }");
}

#[test]
fn shared_component_and_function_samples() {
    let samples = [
        "const FunctionComponent = () => <div></div>",
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            function FunctionComponent() {
                return <div></div>
            }
        "#},
        "const CreateElementComponent = () => React.createElement('div', null, null)",
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            import { memo } from 'react'

            const MemoComponent = memo(() => <div></div>)
        "#},
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            import { forwardRef } from 'react'

            const ForwardRefComponent = forwardRef((props, ref) => <div></div>)
        "#},
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            import { memo, forwardRef } from 'react'

            const MemoForwardRefComponent = memo(forwardRef((props, ref) => <div></div>))
        "#},
        "const fn = () => null",
        indoc! {"
            function fnWithReturn() {
                return null
            }
        "},
        "const footer = () => <div />",
        "const renderFunction = (id: string, name: string) => <div key={id} id={id}>{name}</div>",
        indoc! {"
            function renderFunctionWithReturn(id: string, name: string) {
                return <div key={id} id={id}>{name}</div>
            }
        "},
        indoc! {"
            function renderFunctionWithNestedRenderFunction(id: string, name: string) {
                return <Component footer={() => <div />} />
            }
        "},
    ];
    for sample in samples {
        valid(RULE, sample);
    }
}

#[test]
fn negated_zero_and_nan() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const a = <>{!(0) && <Foo />}</>;
            const b = <>{!(NaN) && <Foo />}</>;
        "#},
    );
}

#[test]
fn double_negated_zero_and_nan() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const a = <>{!!(0) && <Foo />}</>;
            const b = <>{!!(NaN) && <Foo />}</>;
        "#},
    );
}

#[test]
fn triple_negated_zero_and_nan() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const a = <>{!!!(0) && <Foo />}</>;
            const b = <>{!!!(NaN) && <Foo />}</>;
        "#},
    );
}

#[test]
fn negated_number_or_undefined() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            let x: number | undefined;
            const a = <>{!x && <Foo />}</>;
        "#},
    );
}

#[test]
fn ternary_with_negated_and_truthy_number_branches() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            let x: number | undefined;
            const y = 2;
            const a = <>{!x ? !x && <Foo /> : y && <Bar />}</>;
        "#},
    );
}

#[test]
fn truthy_numbers_and_negated_zeros() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const x = -1
            const y = -0
            const z = 0
            const w = 1
            const a = <>{x && <Foo />}</>;
            const b = <>{!y && <Foo />}</>;
            const c = <>{!z && <Foo />}</>;
            const d = <>{w && <Foo />}</>;
        "#},
    );
}

#[test]
fn truthy_bigints_and_negated_zeros() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const x = -1n
            const y = -0n
            const z = 0n
            const w = 1n
            const a = <>{x && <Foo />}</>;
            const b = <>{!y && <Foo />}</>;
            const c = <>{!z && <Foo />}</>;
            const d = <>{w && <Foo />}</>;
        "#},
    );
}

#[test]
fn logical_or_with_zero() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const foo = Math.random() > 0.5;
            const bar = "bar";
            const a = <div>{0 || bar}</div>
        "#},
    );
}

#[test]
fn logical_or_with_boolean() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const foo = Math.random() > 0.5;
            const bar = "bar";
            const a = <div>{foo || bar}</div>
        "#},
    );
}

#[test]
fn plain_string_prop() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            type AppProps = { foo: string; }
            const App = ({ foo }: AppProps) => <div>{foo}</div>
        "#},
    );
}

#[test]
fn rendered_length() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            type AppProps = {
              items: string[];
            }
            const App = ({ items }: AppProps) => {
              return <div>There are {items.length} elements</div>
            }
        "#},
    );
}

#[test]
fn negated_count_with_string() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            type AppProps = {
              items: string[];
              count: number;
            }
            const App = ({ items, count }: AppProps) => {
              return <div>{!count && 'No results found'}</div>
            }
        "#},
    );
}

const LIST_COMPONENT: &str = indoc! {r#"
    /// <reference types="react" />
    /// <reference types="react-dom" />

    type ListProps = {
      items: string[];
    }
    const List = ({ items }: ListProps) => {
      return <div>{items.map(item => <div key={item}>{item}</div>)}</div>
    }
"#};

fn with_list_component(app: &str) -> String {
    format!("{LIST_COMPONENT}{app}")
}

#[test]
fn double_negated_length() {
    valid(
        RULE,
        &with_list_component(indoc! {"
            type AppProps = {
              items: string[];
            }
            const App = ({ items }: AppProps) => {
              return <div>{!!items.length && <List items={items}/>}</div>
            }
        "}),
    );
}

#[test]
fn boolean_call_on_length() {
    valid(
        RULE,
        &with_list_component(indoc! {"
            type AppProps = {
              items: string[];
            }
            const App = ({ items }: AppProps) => {
              return <div>{Boolean(items.length) && <List items={items}/>}</div>
            }
        "}),
    );
}

#[test]
fn length_comparison() {
    valid(
        RULE,
        &with_list_component(indoc! {"
            type AppProps = {
              items: string[];
            }
            const App = ({ items }: AppProps) => {
              return <div>{items.length > 0 && <List items={items}/>}</div>
            }
        "}),
    );
}

#[test]
fn length_ternary() {
    valid(
        RULE,
        &with_list_component(indoc! {"
            type AppProps = {
              items: string[];
            }
            const App = ({ items }: AppProps) => {
              return <div>{items.length ? <List items={items}/> : null}</div>
            }
        "}),
    );
}

#[test]
fn count_ternary() {
    valid(
        RULE,
        &with_list_component(indoc! {"
            type AppProps = {
              items: string[];
              count: number;
            }
            const App = ({ items, count }: AppProps) => {
              return <div>{count ? <List items={items}/> : null}</div>
            }
        "}),
    );
}

#[test]
fn double_negated_count() {
    valid(
        RULE,
        &with_list_component(indoc! {"
            type AppProps = {
              items: string[];
              count: number;
            }
            const App = ({ items, count }: AppProps) => {
              return <div>{!!count && <List items={items}/>}</div>
            }
        "}),
    );
}

#[test]
fn boolean_literal_constants() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const alwaysTruthy = true;
            const alwaysFalsy = false;

            const App = () => {
                return (
                  <div>
                    {alwaysTruthy && <div />}
                    {alwaysFalsy && <div />}
                  </div>
                )
            }
        "#},
    );
}

#[test]
fn nested_ternary_with_strings() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            type AppProps = {
              items: string[];
              count: number;
            }
            const App = ({ items, count }: AppProps) => {
                return <div>{direction ? (direction === "down" ? "▼" : "▲") : ""}</div>
            }
        "#},
    );
}

#[test]
fn ternaries_and_non_empty_string() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const a = <>
                      {0 ? <Foo /> : null}
                      {'0' && <Foo />}
                      {NaN ? <Foo /> : null}
                      </>
        "#},
    );
}

#[test]
fn boolean_and_comparison_in_attribute() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const foo = Math.random() > 0.5;
            const bar = 0;
            function App() {
              return (
                <button
                  type="button"
                  disabled={foo && bar === 0}
                  onClick={() => {}}
                />
              );
            }
        "#},
    );
}

#[test]
fn double_negated_boolean() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = JSON.parse("true") as boolean;
            const SomeComponent = () => <div />;
            const a = <>{!!someCondition && (<SomeComponent prop1={val1} prop2={val2} />)}</>
        "#},
    );
}

#[test]
fn double_negated_any() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = JSON.parse("") as any;
            const SomeComponent = () => <div />;
            const a = <>{!!someCondition && (<SomeComponent prop1={val1} prop2={val2} />)}</>
        "#},
    );
}

#[test]
fn double_negated_unknown() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = JSON.parse("") as unknown;
            const SomeComponent = () => <div />;
            const a = <>{!!someCondition && (<SomeComponent prop1={val1} prop2={val2} />)}</>
        "#},
    );
}

#[test]
fn double_negated_zero_variable() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = 0
            const SomeComponent = () => <div />;
            const a = <>{!!someCondition && (<SomeComponent prop1={val1} prop2={val2} />)}</>
        "#},
    );
}

#[test]
fn double_negated_one_variable() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = 1
            const SomeComponent = () => <div />;
            const a = <>{!!someCondition && (<SomeComponent prop1={val1} prop2={val2} />)}</>
        "#},
    );
}

#[test]
fn zero_used_only_as_ternary_test() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = 0;
            const SomeComponent = () => <div />;
            const a = <>{!!someCondition ? (<SomeComponent prop1={val1} prop2={val2} />) : someCondition ? null : <div />}</>
        "#},
    );
}

#[test]
fn optional_number_rendered_in_ternary() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const SomeComponent = () => <div />;
            const App = ({
              someCondition,
            }: {
              someCondition?: number | undefined;
            }) => {
              return (
                <>
                  {someCondition
                    ? someCondition
                    : <SomeComponent />
                  }
                </>
              )
            }
        "#},
    );
}

#[test]
fn nested_ternaries_with_and_in_test() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = 0;
            const SomeComponent = () => <div />;
            const App = () => {
              return (
                <>
                  {!!someCondition
                    ? (
                    <SomeComponent
                      prop1={val1}
                      prop2={val2}
                    />)
                    : someCondition ? "aaa"
                    : someCondition && someCondition
                    ? <div />
                    : null
                  }
                </>
              )
            }
        "#},
    );
}

#[test]
fn boolean_ternary_with_string_alternate() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = true
            const SomeComponent = () => <div />;
            const App = () => {
              return (
                <>
                  {someCondition
                    ? (
                    <SomeComponent
                      prop1={val1}
                      prop2={val2}
                    />)
                    : "else"
                  }
                </>
              )
            }
        "#},
    );
}

#[test]
fn nested_ternary_rendering_zero_variable() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = 0;
            const SomeComponent = () => <div />;
            const App = () => {
              return (
                <>
                  {!!someCondition
                    ? (
                    <SomeComponent
                      prop1={val1}
                      prop2={val2}
                    />)
                    : someCondition
                    ? someCondition
                    : <div />
                  }
                </>
              )
            }
        "#},
    );
}

#[test]
fn ternary_with_function_call() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const SomeComponent = () => <div />;
            const someFunction = (input: unknown): 10 => 10
            const App = ({ someCondition }: { someCondition?: number | undefined }) => {
              return <>{someCondition ? someFunction(someCondition) : <SomeComponent />}</>;
            };
        "#},
    );
}

#[test]
fn optional_boolean_prop() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const SomeComponent = () => <div />;
            const App = ({
              someCondition,
            }:{
              someCondition?: boolean | undefined;
            }) => {
              return <>{someCondition && <SomeComponent />}</>;
            }
        "#},
    );
}

#[test]
fn double_negated_function_with_instantiation_call() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            type AppProps<T> = {
              someFunction: (data: T) => React.ReactNode;
            };
            function App<T>({ someFunction }: AppProps<T>) {
              return <>{!!someFunction && someFunction<number>(1)}</>;
            }
        "#},
    );
}

#[test]
fn any_condition() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            const someCondition = JSON.parse("") as any;
            const SomeComponent = () => <div />;
            const App = () => {
              return (
                <>
                  {someCondition && (
                    <SomeComponent
                      prop1={val1}
                      prop2={val2}
                    />
                  )}
                </>
              )
            }
        "#},
    );
}

#[test]
fn object_and_nullable_object_conditions() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            function App() {
              const a = {} as {};
              const b = {} as {} | null;
              const b = {} as {} | undefined;
              return (
                <>
                  <>{a && <div />}</>
                  <>{b && <div />}</>
                  <>{c && <div />}</>
                </>
              );
            }
        "#},
    );
}

#[test]
fn narrowed_array_variable() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            declare function getData(): { id: number; name: string }[] | undefined;
            function List({ items }: { items: string[] }) {
              return <div>{items.map(item => <div key={item}>{item}</div>)}</div>;
            }
            type Item = { id: number; name: string };
            function App() {
              let data: Item[] | undefined = getData();
              return (
                <div>
                  {data && <List items={data} />}
                </div>
              );
            }
        "#},
    );
}

#[test]
fn boolean_conditions() {
    valid(
        RULE,
        indoc! {r#"
            /// <reference types="react" />
            /// <reference types="react-dom" />

            export const MyComponent = ({ isVisible1 }: { isVisible1: boolean }) => {
              const isVisible2 = true;
              const isVisible3 = 1 > 2;
              return (
                <>
                  {isVisible1 && <div />}
                  {isVisible2 && <div />}
                  {isVisible3 && <div />}
                </>
              );
            };
        "#},
    );
}

#[test]
fn empty_string_variable_in_react_18() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                /// <reference types="react" />
                /// <reference types="react-dom" />

                const someString = "";
                const a = <>{someString && <Something />}</>;
            "#},
            options: react_version("18.0.0"),
            filename: None,
        },
    );
}

#[test]
fn possibly_empty_string_variable_in_react_18() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                /// <reference types="react" />
                /// <reference types="react-dom" />

                const anyString = Math.random() > 0.5 ? "" : "foo";
                const a = <>{anyString && <Something />}</>;
            "#},
            options: react_version("18.0.0"),
            filename: None,
        },
    );
}

#[test]
fn empty_string_is_valid_by_default() {
    valid(
        RULE,
        indoc! {r#"
            const someString = "";
            const a = <>{someString && <Something />}</>;
        "#},
    );
}

#[test]
fn string_nan_static_value() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const label = "NaN";
            const a = <>{"NaN" && <Foo />}</>;
            const b = <>{label && <Foo />}</>;
            const c = <>{"nan" && <Foo />}</>;
        "#},
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value "NaN" that might cause unintentionally rendered values or rendering crashes.'
      line: 2
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value label that might cause unintentionally rendered values or rendering crashes.'
      line: 3
    "###);
}

#[test]
fn type_expressions_are_unwrapped() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const count: number;
            const a = <>{(count && <Foo />) as any}</>;
            const b = <>{(count as number) && <Foo />}</>;
            const c = <>{count! && <Foo />}</>;
        "},
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value count that might cause unintentionally rendered values or rendering crashes.'
      line: 2
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value count as number that might cause unintentionally rendered values or rendering crashes.'
      line: 3
    - messageId: 'noLeakedConditionalRendering'
      msg: 'Potential leaked value count! that might cause unintentionally rendered values or rendering crashes.'
      line: 4
    "###);
}

#[test]
fn identifier_resolves_to_leaked_initializer() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const count: number;
            const content = count && <Foo />;
            const a = <>{content}</>;
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 2 }");
}

#[test]
fn reports_once_per_container() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const count: number;
            declare const total: number;
            const a = <>{flag ? count && <Foo /> : total && <Bar />}</>;
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 3 }");
}

#[test]
fn spread_children_are_ignored() {
    valid(
        RULE,
        indoc! {"
            declare const count: number;
            const a = <div>{...(count && [<Foo />])}</div>;
        "},
    );
}

#[test]
fn cyclic_identifier_initializers_do_not_overflow() {
    valid(
        RULE,
        indoc! {"
            const first = second;
            const second = first;
            const a = <>{first}</>;
            const b = true && 1;
        "},
    );
}

#[test]
fn react_version_accepts_v_prefix() {
    let result = lint(
        RULE,
        &Case {
            code: "const s = ''; const a = <>{s && <Foo />}</>;",
            options: react_version("v17.0.2"),
            filename: None,
        },
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'noLeakedConditionalRendering', line: 1 }");
}

#[test]
fn invalid_react_version_is_rejected() {
    let error = fast_lint::rules::create_rule(RULE, Some(&json!({ "reactVersion": "latest" })))
        .err()
        .expect("invalid reactVersion should be rejected");

    assert_snapshot!(error, @r###"invalid options for rule no-leaked-conditional-rendering: invalid reactVersion: "latest""###);
}

#[test]
fn unknown_options_are_rejected() {
    let error = fast_lint::rules::create_rule(RULE, Some(&json!({ "version": "17.0.0" })))
        .err()
        .expect("unknown options should be rejected");

    assert_snapshot!(error, @"invalid options for rule no-leaked-conditional-rendering: unknown field `version`, expected `reactVersion`");
}

#[test]
fn react_version_accepts_range_prefixes() {
    for version in ["^17.0.2", "~17.0.2", ">=17", "18.0.0-rc.0"] {
        let result = lint(
            RULE,
            &Case {
                code: "const s = ''; const a = <>{s && <Foo />}</>;",
                options: react_version(version),
                filename: None,
            },
        );
        assert_eq!(
            result.errors(),
            "- { messageId: 'noLeakedConditionalRendering', line: 1 }\n",
            "{version}"
        );
    }
    for version in ["^18.2.0", "18", "18.x", "19.0.0-rc.1"] {
        valid(
            RULE,
            Case {
                code: "const s = ''; const a = <>{s && <Foo />}</>;",
                options: react_version(version),
                filename: None,
            },
        );
    }
}

#[test]
fn react_version_build_metadata_and_fourth_part() {
    for version in ["18.0.0+build-123", "18.0.0.1-rc"] {
        valid(
            RULE,
            Case {
                code: "const s = ''; const a = <>{s && <Foo />}</>;",
                options: react_version(version),
                filename: None,
            },
        );
    }
    for version in ["18-rc", "18.0.0.foo", "18.0.0.0.0"] {
        assert!(
            fast_lint::rules::create_rule(RULE, Some(&json!({ "reactVersion": version }))).is_err(),
            "{version}"
        );
    }
}

#[test]
fn self_referencing_initializers_are_searched_once() {
    valid(
        RULE,
        indoc! {"
            declare const f: boolean
            declare const g: boolean
            var a: any = f ? a : g ? a : f ? a : a
            const el = <>{a}</>
        "},
    );
}

#[test]
fn detect_and_empty_react_version_mean_the_default() {
    for version in ["detect", ""] {
        valid(
            RULE,
            Case {
                code: "const s = ''; const a = <>{s && <Foo />}</>;",
                options: react_version(version),
                filename: None,
            },
        );
    }
}
