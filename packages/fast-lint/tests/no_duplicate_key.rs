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
fn only_the_duplicated_keys_of_a_group_are_reported() {
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
      msg: 'A key must be unique. 'key="x"' is duplicated.'
      line: 1
    "#);
}
#[test]
fn duplicates_anywhere_in_the_group() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const a = [<a key="x" />, <b key="x" />, <c key="y" />];
            const b = [<a key="y" />,
                <b key="x" />, <c key="x" />];
            const c = (
                <ul>
                    <li key="x" />
                    <li key="x" />
                    <li key="y" />
                </ul>
            );
        "#},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 1 }
    - { messageId: 'noDuplicateKey', line: 1 }
    - { messageId: 'noDuplicateKey', line: 3 }
    - { messageId: 'noDuplicateKey', line: 3 }
    - { messageId: 'noDuplicateKey', line: 6 }
    - { messageId: 'noDuplicateKey', line: 7 }
    ");
}
#[test]
fn keys_are_compared_by_static_value() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const a = [<a key="x" />, <b key={"x"} />, <c key={`x`} />, <d key={("x")} />];
            const b = [<a key={1} />, <b key={1.0} />, <c key="1" />];
            const c = [<a key="x" />, <b key="y" />, <c key={`z`} />, <d key={2} />];
        "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="x"' is duplicated.'
      line: 1
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key={"x"}' is duplicated.'
      line: 1
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key={`x`}' is duplicated.'
      line: 1
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key={("x")}' is duplicated.'
      line: 1
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key={1}' is duplicated.'
      line: 2
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key={1.0}' is duplicated.'
      line: 2
    - messageId: 'noDuplicateKey'
      msg: 'A key must be unique. 'key="1"' is duplicated.'
      line: 2
    "#);
}

#[test]
fn identical_expression_keys_are_duplicates() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const a = [<a key={id} />, <b key={id} />];
            const b = (
                <>
                    <Row key={item.id} />
                    <Row key={ item . id } />
                    <Row key={`row-${item.id}`} />
                    <Row key={`row-${item.id}`} />
                </>
            );
        "#},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 1 }
    - { messageId: 'noDuplicateKey', line: 1 }
    - { messageId: 'noDuplicateKey', line: 4 }
    - { messageId: 'noDuplicateKey', line: 5 }
    - { messageId: 'noDuplicateKey', line: 6 }
    - { messageId: 'noDuplicateKey', line: 7 }
    ");
}

#[test]
fn different_expression_keys_are_not_compared_by_value() {
    valid(
        RULE,
        indoc! {r#"
            const a = [<a key={id} />, <b key={otherId} />, <c key={item.id} />, <d key={`${id}`} />];
            const b = [<a key={-1} />, <b key={1} />, <c key="x" />, <d key={x} />];
            const c = [<a key />, <b key />, <c key={} />, <d key={} />];
        "#},
    );
}

#[test]
fn keys_of_different_groups_are_not_compared() {
    valid(
        RULE,
        indoc! {r#"
            const a = [<a key="x" />, <b key="y" />];
            const b = [<a key="x" />, <b key="y" />];
            const c = (
                <ul>
                    <li key="x"><span key="x" /></li>
                    <li key="y">{cond && <span key="y" />}{cond && <span key="y" />}</li>
                </ul>
            );
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
fn every_constant_key_of_a_map_callback_is_reported() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const a = items.map((item) => (item ? <div key="x" /> : <div key={item} />));
            const b = items.map((item) => (item ? <div key={item} /> : <div key="x" />));
            const c = items.map((item) => {
                if (item.a) return <div key="a" />;
                if (item.b) return item.c && <div key={1} />;
                return <div key={`c`} />;
            });
        "#},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 1 }
    - { messageId: 'noDuplicateKey', line: 2 }
    - { messageId: 'noDuplicateKey', line: 4 }
    - { messageId: 'noDuplicateKey', line: 5 }
    - { messageId: 'noDuplicateKey', line: 6 }
    ");
}

#[test]
fn constant_key_in_array_from_callback() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const a = Array.from(items, (item) => <div key="x">{item}</div>);
            const b = Array.from({ length: 3 }, (_, index) => <div key={index} />);
            const c = Array.from(items.map((item) => <div key={item.id} />));
            const d = Other.from(items, (item) => <div key="x" />);
        "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noDuplicateKey', line: 1 }");
}

#[test]
fn outer_const_key_in_map_callback() {
    let result = invalid(
        RULE,
        indoc! {r#"
            const KEY = "x";
            const { id } = props;
            const a = items.map((item) => <div key={KEY}>{item}</div>);
            const b = items.map((item) => <div key={id}>{item}</div>);
            const c = groups.map((group) => group.items.map((item) => <div key={group} />));
        "#},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'noDuplicateKey', line: 3 }
    - { messageId: 'noDuplicateKey', line: 4 }
    ");
}

#[test]
fn keys_bound_inside_the_callback_or_non_const_are_not_constant() {
    valid(
        RULE,
        indoc! {r#"
            let counter = 0;
            const a = items.map((item) => {
                const key = item.id;
                const { id } = item;
                return [<div key={key} />, <span key={id} />];
            });
            const b = items.map((item) => <div key={counter++} />);
            const c = items.map((item) => <div key={counter} />);
            const d = items.map(({ id }) => <div key={id} />);
            const e = items.map((item) => <div key={unknownGlobal} />);
        "#},
    );
}
#[test]
fn file_without_key_attributes() {
    valid(RULE, "const a = [<div />, <div />];");
}
