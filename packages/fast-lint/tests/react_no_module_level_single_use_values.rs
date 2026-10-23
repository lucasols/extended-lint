mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "react-no-module-level-single-use-values";

#[test]
fn valid_module_level_array_used_in_multiple_components() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = ['a', 'b']

                export function FirstList() {
                  return <div>{options.join(', ')}</div>
                }

                export function SecondList() {
                  return <span>{options.length}</span>
                }
            "},
            filename: Some("lists.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_module_level_array_used_outside_of_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = ['a', 'b']
                const optionsSet = new Set(options)
                const optionsCount = optionsSet.size

                export function List() {
                  return <div>{optionsCount}</div>
                }
            "},
            filename: Some("lists.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_module_level_primitive_value_used_in_one_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const maxItems = 5
                const label = 'items'

                export function List() {
                  return <div>{maxItems} {label}</div>
                }
            "},
            filename: Some("lists.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_non_tsx_files_are_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = ['a', 'b']

                export function List() {
                  return options.join(', ')
                }
            "},
            filename: Some("lists.ts"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_same_file_component_declaration_as_arrow_function_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const CopyPublicLinkButton = ({ appId }: { appId: string }) => {
                  return <button>{appId}</button>
                }

                export function AppViewHeader({ appId }: { appId: string }) {
                  return (
                    <div>
                      <CopyPublicLinkButton appId={appId} />
                    </div>
                  )
                }
            "},
            filename: Some("AppViewHeader.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_module_helper_arrow_function_used_by_one_component_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const formatLabel = (value: string) => value.toUpperCase()

                export function Header() {
                  return <div>{formatLabel('item')}</div>
                }
            "},
            filename: Some("header.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_inferable_alias_reference_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = ['a', 'b']
                const listOptions = options

                export function List() {
                  return <div>{listOptions.join(', ')}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_inferable_member_reference_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const config = { labels: ['a', 'b'] }
                const listOptions = config.labels

                export function List() {
                  return <div>{listOptions.join(', ')}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_regex_value_used_by_one_component_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const userIdRegex = /^user_[a-z0-9]+$/

                export function List({ value }: { value: string }) {
                  return <div>{userIdRegex.test(value) ? 'ok' : 'invalid'}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_class_instantiation_used_by_one_component_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                class Store {
                  state: Record<string, true | 'inProgress'>

                  constructor() {
                    this.state = {}
                  }
                }

                const notificationWasForwardedState = new Store()

                export function List({ id }: { id: string }) {
                  const value = notificationWasForwardedState.state[id]
                  return <div>{value ? 'forwarded' : 'pending'}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_ignoreconstcasevarregex_ignores_matching_constant_case_variable() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const MOCK_APP_PERMISSIONS = {
                  canViewApp: true,
                }

                export function GridPositioningStory() {
                  return <div>{MOCK_APP_PERMISSIONS.canViewApp ? 'yes' : 'no'}</div>
                }
            "},
            options: Some(json!({ "ignoreConstCaseVarRegex": "^MOCK_" })),
            filename: Some("GridPositioningStory.tsx"),
        },
    );
}

#[test]
fn invalid_set_instantiation_used_in_one_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const selectedIds = new Set<string>()

                export function List({ id }: { id: string }) {
                  return <div>{selectedIds.has(id) ? 'selected' : 'not selected'}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `selectedIds` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn invalid_map_instantiation_used_in_one_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const nameById = new Map<string, string>()

                export function List({ id }: { id: string }) {
                  return <div>{nameById.get(id) ?? 'unknown'}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `nameById` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn invalid_ignoreconstcasevarregex_does_not_ignore_non_constant_case_variable() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const mockAppPermissions = {
                  canViewApp: true,
                }

                export function GridPositioningStory() {
                  return <div>{mockAppPermissions.canViewApp ? 'yes' : 'no'}</div>
                }
            "},
            options: Some(json!({ "ignoreConstCaseVarRegex": "^mock" })),
            filename: Some("GridPositioningStory.tsx"),
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `mockAppPermissions` is only used in React component `GridPositioningStory`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn invalid_object_used_twice_in_one_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const mockAppPermissions = {
                  canViewApp: true,
                }

                export function GridPositioningStory() {
                  return (
                    <div>
                      <Editable appPermissions={mockAppPermissions} />
                      <Readonly appPermissions={mockAppPermissions} />
                    </div>
                  )
                }

                function Editable({ appPermissions }: { appPermissions: { canViewApp: boolean } }) {
                  return <div>{appPermissions.canViewApp ? 'yes' : 'no'}</div>
                }

                function Readonly({ appPermissions }: { appPermissions: { canViewApp: boolean } }) {
                  return <div>{appPermissions.canViewApp ? 'yes' : 'no'}</div>
                }
            "},
            filename: Some("GridPositioningStory.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `mockAppPermissions` is only used in React component `GridPositioningStory`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn invalid_module_array_used_only_in_one_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = ['a', 'b']

                export function List() {
                  return <div>{options.join(', ')}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn invalid_module_object_used_only_in_one_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const style = { color: 'red' }

                export function List() {
                  return <div style={style}>hello</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `style` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn invalid_long_module_string_used_only_in_one_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const longLabel = 'This is a very long static string that should be moved into a single component according to this rule'

                export function List() {
                  return <div>{longLabel}</div>
                }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `longLabel` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn invalid_module_array_used_only_inside_nested_callback_of_one_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = ['a', 'b'] as const

                const List = () => {
                  const pickFirst = () => options[0]
                  return <div>{pickFirst()}</div>
                }

                export default List
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_export_default_function_is_not_a_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                export default function List() { return options }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_memo_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                const List = memo(() => options)
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_nested_wrappers() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                const List = React.memo(forwardRef(function X() { return options }))
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_as_wrapped_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                const List = ((() => options) as FC)
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_smallest_component_wins() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                function Outer() { return options }
                const Inner = () => options
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_typeof_outside_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                type T = typeof options
                function List() { return options }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_typeof_inside_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = { a: 1 }
                function List() { type T = typeof options; return null }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_long_template_literal() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const label = `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
                function List() { return label }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `label` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_short_template_literal() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const label = `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
                function List() { return label }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_exported_value() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                export const options = [1]
                function List() { return options }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_used_in_helper_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const ids = new Set()
                function helper() { return ids }
                function List() { return ids }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_let_and_var() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                let a = [1]
                var b = { x: 1 }
                function List() { return [a, b] }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `a` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `b` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 2
    ");
}

#[test]
fn extra_unused_value() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                function List() { return null }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_lowercase_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                function list() { return options }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_optional_chain_wrapper() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                const List = React?.memo(() => options)
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn extra_declare_function() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1]
                declare function List(): void
                const X = () => options
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `X`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_satisfies_and_non_null() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const options = [1] satisfies number[]
                const o2 = ({ a: 1 })!
                function List() { return [options, o2] }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `options` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `o2` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 2
    ");
}

#[test]
fn extra_new_member_expression() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r"
                const m = new globalThis.Map()
                const d = new Date()
                function List() { return [m, d] }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'moveValueInsideComponent'
      msg: 'Module-level `m` is only used in React component `List`. Move it inside the component to improve memory usage.'
      line: 1
    ");
}

#[test]
fn extra_destructured_declarator() {
    valid(
        RULE,
        Case {
            code: indoc! {r"
                const { a } = { a: [1] }
                function List() { return a }
            "},
            filename: Some("list.tsx"),
            ..Default::default()
        },
    );
}
