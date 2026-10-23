mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-duplicate-key";

const ALL_VALID: &[&str] = &[
    "const CreateElementComponent = () => React.createElement('div', null, null)",
    "const FunctionComponent = () => <div></div>",
    indoc! {r#"
        /// <reference types="react" />
        /// <reference types="react-dom" />

        function FunctionComponent() {
            return <div></div>
        }
    "#},
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

        import { forwardRef, memo } from 'react'

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

#[test]
fn all_valid() {
    for code in ALL_VALID {
        valid(RULE, *code);
    }
}

#[test]
fn single_keyed_array_item() {
    valid(
        RULE,
        indoc! {r#"
            const App = () => {
                return [<div key="1">1</div>]
            };
        "#},
    );
}

#[test]
fn unique_keys_in_array() {
    valid(
        RULE,
        indoc! {r#"
            const App = () => {
                return [
                        <div key="1">1</div>,
                        <div key="2">2</div>,
                        <div key="3">3</div>,
                     ]
            };
        "#},
    );
}

#[test]
fn variable_key_in_map_block_body() {
    valid(
        RULE,
        indoc! {"
            const App = () => {
                return [1, 2, 3].map((item) => { const key = item; return <div key={key}>{item}</div> })
            };
        "},
    );
}

#[test]
fn variable_keys_in_nested_maps() {
    valid(
        RULE,
        indoc! {"
            const App = () => {
                return nested.map((item) => {
                    return <div key={item}>{item.map((i) => { return <div key={i}>{i}</div> })}</div>
                })
            };
        "},
    );
}

#[test]
fn duplicate_keys_in_array() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return [
                        <div key="1">1</div>,
                        <div key="1">2</div>,
                        <div key="1">3</div>,
                     ]
            };
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 3
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 4
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 5
    "#);
}

#[test]
fn duplicate_keys_in_jsx_children() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return  (<div>
                            <div key="1">1</div>
                            <div key="1">2</div>
                            <div key="1">3</div>
                        </div>)
            };
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 3
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 4
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 5
    "#);
}

#[test]
fn constant_key_in_map_arrow() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return [1, 2, 3].map((item) => <div key="1">{item}</div>)
            };
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 2
    "#);
}

#[test]
fn constant_key_in_map_block_body() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return [1, 2, 3].map((item) => { return <div key="1">{item}</div> })
            };
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 2
    "#);
}

#[test]
fn constant_keys_in_nested_maps() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return nested.map((item) => {
                    return <div key="1">{item.map((i) => <div key="a">{i}</div>)}</div>
                })
            };
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 3
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="a"' is duplicated.'
      line: 3
    "#);
}

#[test]
fn constant_keys_in_three_nested_maps() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return nested.map((foo) => {
                    return <div key="foo">{foo.map((bar) => <div key="bar">{bar.map((baz) => <div key="baz">{baz}</div>)}</div>)}</div>
                })
            };
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="foo"' is duplicated.'
      line: 3
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="bar"' is duplicated.'
      line: 3
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="baz"' is duplicated.'
      line: 3
    "#);
}

#[test]
fn constant_keys_in_maps_with_optional_calls_and_type_expressions() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return nested?.map((foo) => {
                    return <div key="foo">{foo!.map((bar) => <div key="bar">{bar!!.map(((baz) => <div key="baz">{baz}</div>)!!! as A satisfies B)}</div>)}</div>
                })
            };
        "#},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 3 }
    - { messageId: 'noDuplicateKey', line: 3 }
    - { messageId: 'noDuplicateKey', line: 3 }
    ");
}

#[test]
fn constant_key_in_non_map_callback() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const App = () => {
                return nested.map((foo) => {
                    return <div key="foo">{foo.notmap((bar) => <div key="bar">{bar.map((baz) => <div key="baz">{baz}</div>)}</div>)}</div>
                })
            };
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="foo"' is duplicated.'
      line: 3
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="baz"' is duplicated.'
      line: 3
    "#);
}

#[test]
fn duplicate_keys_in_fragment_and_self_closing_elements() {
    let result = invalid(
        RULE,
        indoc! {"
            const App = () => (
                <>
                    <Item key='a' />
                    <Item key=\"a\" />
                </>
            );
        "},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 3 }
    - { messageId: 'noDuplicateKey', line: 4 }
    ");
}

#[test]
fn parenthesized_array_items() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const items = [
                (<div key="a" />),
                (
                    <div key="a" />
                ),
            ];
        "#},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 2 }
    - { messageId: 'noDuplicateKey', line: 4 }
    ");
}

#[test]
fn last_key_duplicating_an_earlier_one_reports_the_whole_group() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const items = [<a key="x" />, <b key="y" />, <c key="x" />];
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="x"' is duplicated.'
      line: 1
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="y"' is duplicated.'
      line: 1
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="x"' is duplicated.'
      line: 1
    "#);
}

#[test]
fn duplicate_followed_by_a_unique_key() {
    valid(
        RULE,
        r#"const items = [<a key="x" />, <b key="x" />, <c key="y" />];"#,
    );
}

#[test]
fn non_string_literal_keys_are_not_compared() {
    valid(
        RULE,
        indoc! {r#"
            const items = [<a key={1} />, <b key={1} />];
            const other = [<a key={"x"} />, <b key={"x"} />];
            const ids = [<a key={id} />, <b key={id} />];
        "#},
    );
}

#[test]
fn constant_key_in_flat_map_and_method_callbacks() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const a = items.flatMap((item) => <div key="x">{item}</div>);
            class List {
                render() {
                    return items.map(function (item) { return <div key="y">{item}</div> });
                }
            }
        "#},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 1 }
    - { messageId: 'noDuplicateKey', line: 4 }
    ");
}

#[test]
fn map_callback_passed_by_reference() {
    valid(
        RULE,
        indoc! {r#"
            const render = (item) => <div key="x">{item}</div>;
            const a = items.map(render);
            const b = items.map((item) => [1].forEach(() => <div key="z" />));
        "#},
    );
}

#[test]
fn only_the_last_key_of_a_map_call_counts() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const a = items.map((item) => (item ? <div key="x" /> : <div key={item} />));
            const b = items.map((item) => (item ? <div key={item} /> : <div key="x" />));
        "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noDuplicateKey', line: 2 }");
}

#[test]
fn file_without_key_attributes() {
    valid(RULE, "const a = [<div />, <div />];");
}
