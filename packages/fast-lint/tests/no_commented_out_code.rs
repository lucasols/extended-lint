mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-commented-out-code";

#[test]
fn allows_the_descriptive_type_comment_from_the_reported_false_positive() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // type doesn't declare it.
                const requestInit = {};
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn allows_descriptive_comments_starting_with_type() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // type does not declare this property.
                // type inference is handled by the compiler.
                // type RequestInit doesn't declare dispatcher.
                // type compatibility depends on the declared fields.
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn detects_commented_type_aliases() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // type Value = string;
                // type Value=string;
                // type Value<T> = T;
                // type Value<T extends object = object> = T;
                // type $Value = string;
                // type _Value =
                // type	Value = string;
                // type Résultat = string;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 7 }
    - { messageId: 'commentedOutCode', line: 8 }
    ");
}

#[test]
fn allows_descriptive_headers_containing_parenthetical_prose() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // chatbot: exposes UI config (colors, avatar, etc) but no sensitive fields
                // Step failures don't reject the Mastra call, but they must fail the
                // job: a batched flush would otherwise ack (and drop) a batch that got
                // no reply, while failing leaves it claimed for the retries to re-claim.
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn allows_descriptive_headers_containing_slashes() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // ticket-solver/whatsapp: only expose monitoredResponses + type
                // inactive agents are excluded from the list
                // virtual default agents are appended after DB rows
                // frontend/backend: expose shared config (colors, avatar) to both clients
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn detects_code_under_headers_with_parentheses_or_slashes() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // chatbot: getConfig(colors, avatar),
                // chatbot: getConfig (colors, avatar),
                // chatbot: async function getConfig (colors, avatar) {
                // chatbot: exposes UI config (colors = defaults) but no sensitive fields
                // chatbot: exposes UI config (config.getColors()) but no sensitive fields
                // chatbot: exposes UI config (colors => colors) but no sensitive fields
                // ticket-solver/whatsapp: getConfig(colors, avatar),
                // 'ticket-solver/whatsapp': 'value',
                // chatbot: async function getConfig(colors, avatar)
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 7 }
    - { messageId: 'commentedOutCode', line: 8 }
    - { messageId: 'commentedOutCode', line: 9 }
    ");
}

#[test]
fn valid_code() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // This comment isn't code.
                const answer = 42;

                // ignore triple slash directives
                /// <reference lib="WebWorker" />

                // TODO: salvar nome do menu localmente pq essa informação não é persistente no servidor
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn invalid_2() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // This comment includes some code:
                // const answer = 54;
                const answer = 42;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 2 }");
}

#[test]
fn invalid_block_of_code() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // function onClickAdd() {
                //   const coords = getLastCoords();

                //   if (!coords) return;

                //   const pos = editor.view.posAtCoords(coords);

                //   if (!pos) return;

                //   const blockInfo = getBlockInfoFromPos(editor.state.doc, pos.pos);

                //   if (!blockInfo) return;

                //   const { contentNode, endPos } = blockInfo;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 8 }
    - { messageId: 'commentedOutCode', line: 10 }
    - { messageId: 'commentedOutCode', line: 12 }
    - { messageId: 'commentedOutCode', line: 14 }
    ");
}

#[test]
fn invalid_jsx_commented_prop() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // getLastCoords={getLastCoords}
                // editor={editor}
                // editor="editor"
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    ");
}

#[test]
fn invalid_objects() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // TODO: add to back
                // {
                //   ...normalizedBase,
                //   operator: '==',
                //   valueAlias: {
                //     teste: '2',
                //     sdffsdfds: '3',
                //   },
                // },
                // TODO: add to back
                // {
                //   ...normalizedBase,
                //   operator: 'contains',
                // },
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 7 }
    - { messageId: 'commentedOutCode', line: 8 }
    - { messageId: 'commentedOutCode', line: 9 }
    - { messageId: 'commentedOutCode', line: 13 }
    - { messageId: 'commentedOutCode', line: 14 }
    ");
}

#[test]
fn invalid_jsx_commented_component() {
    let result = invalid(
        RULE,
        Case {
            code: r#"/* <Component /> */"#,
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 1 }");
}

#[test]
fn valid_jsx_commented_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                /* INFO: This is a comment
                  <Component />
                */
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn ingore_jsdocs_comments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                /**
                 * This is a comment
                 * <Component />
                 */
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn invalid_commented_object_property() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const obj = {
                  teste: '2',
                  // sdffsdfds: '3',
                  // ['teste']: '2',
                  // test_e: '2',
                  // test-e:
                  //   '2',
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 7 }
    ");
}

#[test]
fn invalid_commented_object_with_different_value_types() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const config = {
                  name: 'test',
                  // age: 25,
                  // isActive: true,
                  // items: [],
                  // settings: {},
                  // getValue: function() { return 42; }
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 7 }
    ");
}

#[test]
fn invalid_commented_object_methods() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const api = {
                  getData() { return data; },
                  // fetchUser() { return fetch('/user'); },
                  // async loadData() { return await fetch('/data'); },
                  // arrow: () => console.log('test'),
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn invalid_commented_object_with_computed_properties() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const obj = {
                  [key]: 'value',
                  // [computedKey]: 'computed',
                  // ['dynamic']: dynamicValue,
                  // [`template`]: templateValue,
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn invalid_commented_object_with_nested_structures() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const complex = {
                  user: { name: 'John' },
                  // profile: { age: 30, city: 'NYC' },
                  // preferences: {
                  //   theme: 'dark',
                  //   notifications: true
                  // },
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 7 }
    ");
}

#[test]
fn valid_commented_object_shorthand_properties() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const user = { name, age };
                const data = {
                  user,
                  // name,
                  // age,
                  // isActive,
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_commented_object_with_spread() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const base = { a: 1 };
                const extended = {
                  ...base,
                  // ...otherProps,
                  // ...userSettings,
                  b: 2
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_commented_object_getters_and_setters() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const obj = {
                  _value: 0,
                  // get value() { return this._value; },
                  // set value(val) { this._value = val; },
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_commented_object_with_quotes_and_special_chars() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const obj = {
                  'normal-key': 'value',
                  // 'quoted-key': 'quoted value',
                  // "double-quoted": "double value",
                  // 'key with spaces': 'spaced value',
                  // 'key:with:colons': 'colon value',
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    - { messageId: 'commentedOutCode', line: 6 }
    ");
}

#[test]
fn invalid_commented_object_trailing_commas() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const obj = {
                  first: 1,
                  // second: 2,
                  // third: 3,
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn valid_object_with_allowed_prefix_comments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const config = {
                  enabled: true,
                  // TODO: add timeout property
                  // NOTE: cache property is deprecated
                  // FIXME: fix the validation logic
                  debug: false
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_object_with_descriptive_comments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const settings = {
                  // Database configuration
                  host: 'localhost',
                  // API endpoints configuration  
                  apiUrl: '/api/v1'
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_condition() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                if (test
                  // && test_2
                  // || test_3
                  // ? test_4
                  // : test_5
                ) {
                  console.log('test')
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn invalid_react_hooks() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // const [state, setState] = useState(0)
                // useEffect(() => {
                //   console.log('effect')
                // }, [])
                // const memoValue = useMemo(() => value * 2, [value])
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn invalid_modern_js_patterns() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // const result = data?.map(item => item.value)
                // const fallback = value ?? 'default'
                // await fetchData()
                // items.filter(x => x.active)
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_console_and_dom() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // console.log('debug')
                // document.getElementById('test')
                // window.location.href = '/new-page'
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    ");
}

#[test]
fn invalid_try_catch() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // try {
                //   doSomething()
                // } catch (error) {
                //   console.error(error)
                // }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn valid_with_fixme_prefix() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // FIXME: const broken = getSomething()
                const answer = 42;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_with_note_prefix() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // NOTE: useState was causing issues here
                const answer = 42;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_with_hack_prefix() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // HACK: temporarily disable this feature
                const answer = 42;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_typescript_comments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // @ts-ignore
                // @ts-expect-error
                const answer = 42;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_jsdoc_patterns() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                /**
                 * @param name - The user name
                 * @returns The greeting message
                 * @throws Error when name is empty
                 */
                function greet(name: string) {
                  return `Hello ${name}`;
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_commented_array_strings() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const items = [
                  'first',
                  // 'second',
                  // 'third',
                  'fourth'
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_commented_array_numbers() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const numbers = [
                  1,
                  // 2,
                  // 3,
                  4
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_commented_array_objects() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const users = [
                  { name: 'John', age: 30 },
                  // { name: 'Jane', age: 25 },
                  // { name: 'Bob', age: 35 },
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_commented_mixed_array_items() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const mixed = [
                  'string',
                  // 42,
                  // true,
                  // null,
                  // undefined,
                  { key: 'value' }
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 3 }");
}

#[test]
fn invalid_commented_array_with_functions() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const handlers = [
                  onClick,
                  // onHover,
                  // () => console.log('click'),
                  // function handler() { return true; }
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn invalid_commented_nested_arrays() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const matrix = [
                  [1, 2, 3],
                  // [4, 5, 6],
                  // [7, 8, 9]
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 3 }");
}

#[test]
fn valid_array_with_allowed_prefix_comments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const items = [
                  'first',
                  // TODO: add second item later
                  // NOTE: third item is not ready yet
                  'fourth'
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_array_with_regular_comments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const config = [
                  // Configuration for development
                  'dev-setting',
                  // Configuration for production  
                  'prod-setting'
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_commented_array_with_template_literals() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const templates = [
                  `hello world`,
                  // `template ${variable}`,
                  // `another template`,
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_commented_jsx_array_items() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const components = [
                  <div>first</div>,
                  // <span>second</span>,
                  // <Component prop="value" />,
                ]
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_commented_array_methods() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const items = [
                  'item1',
                  // 'item2'.toUpperCase(),
                  // getValue(),
                  // obj.method(),
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn invalid_array_with_descriptive_comments() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const routes = [
                  '/home',
                  // Routes below are for admin users only
                  // '/admin',
                  '/profile'
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 4 }");
}

#[test]
fn valid_commented_array_spread() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const combined = [
                  ...firstArray,
                  // ...secondArray,
                  // ...otherItems,
                ]
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_commented_object_with_numeric_keys() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const data = {
                  0: 'first',
                  // 1: 'second',
                  // 2: 'third',
                  3: 'fourth'
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    ");
}

#[test]
fn invalid_commented_object_with_mixed_key_types() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const mixed = {
                  regularKey: 'value',
                  // 'string-key': 'string value',
                  // 42: 'numeric key',
                  // [dynamicKey]: 'dynamic value',
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn invalid_commented_object_destructuring_patterns() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const obj = {
                  a: 1,
                  // b: 2,
                  c: 3
                };
                // const { a, b, c } = obj;
                // const { x = 1, y = 2 } = props;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 6 }
    - { messageId: 'commentedOutCode', line: 7 }
    ");
}

#[test]
fn invalid_commented_object_with_function_values() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const handlers = {
                  onClick: handleClick,
                  // onSubmit: handleSubmit,
                  // validate: (data) => data.isValid,
                  // async process() { return await api.call(); }
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn valid_object_with_complex_comment_patterns() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const config = {
                  // TEMP: using development settings
                  apiUrl: 'dev-api.com',
                  // NOTE: production timeout should be higher
                  timeout: 5000,
                  // TODO: implement retry logic
                  retry: false
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_regular_block_comment() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                /* This is a regular block comment */
                const test = 1;

                /* 
                 * Multi-line block comment
                 * with code-like patterns like function() 
                 * and const x = 1 should be allowed
                 */
                const test2 = 2;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_jsx_in_block_comment() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                /* <div>JSX content</div> */
                const test = 1;
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 1 }");
}

#[test]
fn valid_eslint_comment() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                /* slint react-compiler/react-compiler: ["error"] */
                import { css } from '@linaria/core';
                import { styled } from '@linaria/react';
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_comments_with_inline_code_examples() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // The `dataTransfer.setDragImage(element)` method only works if element is attached
                // Use `useState()` hook for state management  
                // Call `myObject.method()` to execute the function
                // Set config option `enabled: true` in the settings
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_comments_with_markdown_inline_code() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // This function uses `Array.map()` internally
                // The `console.log()` is for debugging purposes
                // Set `config.enabled = true` to activate
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_commented_code_should_still_be_detected_outside_backticks() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // This is a comment but console.log() is actual code
                // Array.map() without backticks should be detected
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    ");
}

#[test]
fn invalid_commented_code_with_backticks() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // If there are droppables intersecting with the pointer, return those
                // If a container is matched and it contains items (columns 'A', 'B', 'C')  
                // https://www.google.com/maps/place?q=place_id:ChIJN1t_tDeuEmsRUsoyG83frY4
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_commented_code_with_return() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function test() {
                  // return 'test'
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 2 }");
}

#[test]
fn comment_with_colon_and_text_should_be_allowed() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // Third pass: find exported constants
                // Second: this if a fine comment too
                // Step 1: Create all tables and their fields
                // chats: sending one would deliver raw markdown pipe text to the recipient
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn comments_beginning_with_code_keyword_prefixes() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // pipes are escaped and line breaks encoded as `<br>` (the GFM way of
                // breaking lines inside cells, supported by the msg renderer) so they
                // don't corrupt the one-cell-per-column, one-line-per-row table syntax
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn commented_break_and_continue_statements() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // break
                // break;
                // break outerLoop;
                // continue
                // continue;
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'commentedOutCode', line: 1 }
    - { messageId: 'commentedOutCode', line: 2 }
    - { messageId: 'commentedOutCode', line: 3 }
    - { messageId: 'commentedOutCode', line: 4 }
    - { messageId: 'commentedOutCode', line: 5 }
    ");
}

#[test]
fn cspell_and_spell_checker_comments_should_be_allowed() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                // spell-checker: ignore
                // spell-checker: disable
                // spell-checker: enable
                // spell-checker: disable-next-line
                // cspell: ignore exported constants
                // cspell: ignore fine comment
                // cspell: ignore tables and their fields
                // cspell: disable
                // cspell: enable
                // cspell: disable-next-line
                // cspell: words myCustomWord anotherWord
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn explanation_comment_blocks_are_ignored() {
    valid(
        RULE,
        indoc! {r#"
            // type-guard-safety: same as Lexical's `$is*Node` helpers: node => node instanceof ImageNode
            // node-type: lexical registers one class per node type, see `$createImageNode()`
            // so the klass check can't be inferred through `LexicalNode.getType()`
            export function isImage(node: unknown) {
              return node
            }
        "#},
    );
}

#[test]
fn explanation_prefix_must_start_the_comment() {
    let result = invalid(
        RULE,
        indoc! {r#"
            // foo.bar();type-guard-safety:no
            export const b = 2
        "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 1 }");
}

#[test]
fn explanation_block_ends_at_a_blank_line() {
    let result = invalid(
        RULE,
        indoc! {r#"
            // type-guard-safety: values come from our serializer, which always sets kind

            // const a = 1
            export const b = 2
        "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 3 }");
}

#[test]
fn custom_explanation_prefixes() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                // WORKAROUND: the api-client: returns `null` for 404s
                // and-then: we map it to undefined
                export const a = 1
                // type-guard-safety: the api-client: returns `null`
                export const b = 2
            "#},
            options: Some(serde_json::json!([{ "explanationPrefixes": ["WORKAROUND:"] }])),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'commentedOutCode', line: 4 }");
}
