mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "react-single-export";

#[test]
fn valid_single_default_export_in_react_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                const MyComponent = () => <div>Hello</div>

                export default MyComponent
            "#},
            filename: Some("MyComponent.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_single_named_export_in_react_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                export const MyComponent = () => <div>Hello</div>
            "#},
            filename: Some("MyComponent.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_single_export_with_jsx() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const MyComponent = () => {
                  return <div>Hello World</div>
                }
            "#},
            filename: Some("MyComponent.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_non_react_files_with_multiple_exports() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const utils = {}
                export const helper = () => {}
                export const constant = 'value'
            "#},
            filename: Some("utils.ts"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_react_component_with_only_type_exports() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                export type Props = {
                  name: string
                }

                const MyComponent = ({ name }: Props) => <div>{name}</div>

                export default MyComponent
            "#},
            filename: Some("MyComponent.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_multiple_value_exports_in_react_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                export const MyComponent = () => <div>Hello</div>
                export const OtherComponent = () => <div>World</div>
            "#},
            filename: Some("MyComponent.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_default_export_named_export_in_react_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                const MyComponent = () => <div>Hello</div>
                export const helper = () => {}

                export default MyComponent
            "#},
            filename: Some("MyComponent.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_tsx_file_with_multiple_exports() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const Component1 = () => <div>One</div>
                export const Component2 = () => <div>Two</div>
            "#},
            filename: Some("components.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 2 }");
}

#[test]
fn valid_react_component_file_with_custom_extension_and_single_export() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const MyComponent = () => {
                  return <button>Click me</button>
                }
            "#},
            options: Some(json!([{"extensions":["component.tsx"]}])),
            filename: Some("MyComponent.component.tsx"),
        },
    );
}

#[test]
fn invalid_custom_extension_with_multiple_exports() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const Component = () => <div>Hello</div>
                export const utils = {}
            "#},
            options: Some(json!([{"extensions":["component.tsx"]}])),
            filename: Some("MyComponent.component.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 2 }");
}

#[test]
fn invalid_multiple_component_exports_in_same_file() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const DropdownItem = ({ children, onClick, disabled, className }) => {
                  return (
                    <DropdownItemStyled
                      onClick={onClick}
                      disabled={disabled}
                      className={className}
                    >
                      {children}
                    </DropdownItemStyled>
                  )
                }

                const Dropdown = ({ children, onClick, disabled, className }) => {
                  return (
                    <DropdownItemStyled
                      onClick={onClick}
                      disabled={disabled}
                      className={className}
                    >
                      {children}
                    </DropdownItemStyled>
                  )
                }

                export { Dropdown, DropdownItem }
            "#},
            filename: Some("Dropdown.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 25 }");
}

#[test]
fn invalid_pascalcase_function_with_fc_type() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { FC } from 'react'

                export const Button: FC = () => <button>Click</button>
                export const Input: FC = () => <input />
            "#},
            filename: Some("Components.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_pascalcase_function_with_jsx_element_return_type() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const Header = (): JSX.Element => <header>Header</header>
                const Footer = (): JSX.Element => <footer>Footer</footer>

                export { Header, Footer }
            "#},
            filename: Some("Layout.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_memo_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { memo } from 'react'

                const Card = memo(() => <div>Card</div>)
                const Panel = memo(() => <div>Panel</div>)

                export { Card, Panel }
            "#},
            filename: Some("Components.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 6 }");
}

#[test]
fn invalid_forwardref_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { forwardRef } from 'react'

                const Input = forwardRef<HTMLInputElement>((props, ref) => 
                  <input ref={ref} {...props} />
                )
                const Button = forwardRef<HTMLButtonElement>((props, ref) => 
                  <button ref={ref} {...props} />
                )

                export { Input, Button }
            "#},
            filename: Some("FormComponents.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 10 }");
}

#[test]
fn valid_single_memo_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { memo } from 'react'

                const Card = memo(() => <div>Card</div>)

                export default Card
            "#},
            filename: Some("Card.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_single_fc_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { FC } from 'react'

                export const Button: FC = () => <button>Click</button>
            "#},
            filename: Some("Button.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_mixed_exports_with_non_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const MyComponent = () => <div>Hello</div>
                export const CONSTANT = 'value'
                export type Props = { name: string }
            "#},
            filename: Some("Component.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 2 }");
}

#[test]
fn valid_single_reactnode_return_type() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { ReactNode } from 'react'

                export const Wrapper = (): ReactNode => <div>Wrapper</div>
            "#},
            filename: Some("Wrapper.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_function_declaration_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export function Button(): JSX.Element {
                  return <button>Click me</button>
                }
            "#},
            filename: Some("Button.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_react_memo_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                export const MemoComponent = React.memo(() => <div>Memoized</div>)
            "#},
            filename: Some("MemoComponent.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_react_forwardref_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                export const RefComponent = React.forwardRef<HTMLDivElement>((props, ref) => 
                  <div ref={ref} {...props} />
                )
            "#},
            filename: Some("RefComponent.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_component_with_custom_hooks() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const MyComponent = () => {
                  const [state, setState] = useState(0)
                  return <div>{state}</div>
                }

                export const useCustomHook = () => {
                  return useState(0)
                }
            "#},
            filename: Some("Component.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 6 }");
}

#[test]
fn valid_utility_functions_only() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const formatName = (name: string) => name.toUpperCase()
                export const validateEmail = (email: string) => email.includes('@')
            "#},
            filename: Some("utils.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_no_exports() {
    valid(
        RULE,
        Case {
            code: r#"const Component = () => <div>Private</div>"#,
            filename: Some("internal.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_react_fc_vs_fc_mixed() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import React, { FC } from 'react'

                export const Button: React.FC = () => <button>Button</button>
                export const Input: FC = () => <input />
            "#},
            filename: Some("Components.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_mixed_react_memo_and_direct() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import React from 'react'

                export const Direct = () => <div>Direct</div>
                export const Memo = React.memo(() => <div>Memo</div>)
            "#},
            filename: Some("Mixed.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_function_declarations() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export function Header(): JSX.Element {
                  return <header>Header</header>
                }

                export function Footer(): JSX.Element {
                  return <footer>Footer</footer>
                }
            "#},
            filename: Some("Layout.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 5 }");
}

#[test]
fn invalid_reactnode_return_types() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { ReactNode } from 'react'

                const Nav = (): ReactNode => <nav>Navigation</nav>
                const Sidebar = (): ReactNode => <aside>Sidebar</aside>

                export { Nav, Sidebar }
            "#},
            filename: Some("Layout.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 6 }");
}

#[test]
fn invalid_generic_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                interface Props<T> {
                  data: T
                }

                export const List = <T,>(props: Props<T>) => <ul>{JSON.stringify(props.data)}</ul>
                export const Table = <T,>(props: Props<T>) => <table>{JSON.stringify(props.data)}</table>
            "#},
            filename: Some("Generic.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 6 }");
}

#[test]
fn invalid_higher_order_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const withLoading = (Component: React.ComponentType) => (props: any) => 
                  <div>Loading...<Component {...props} /></div>

                const withError = (Component: React.ComponentType) => (props: any) => 
                  <div>Error!<Component {...props} /></div>

                export { withLoading, withError }
            "#},
            filename: Some("hoc.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 7 }");
}

#[test]
fn invalid_component_factories() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const CreateButton = (variant: string) => () => 
                  <button className={variant}>Button</button>

                export const CreateInput = (type: string) => () => 
                  <input type={type} />
            "#},
            filename: Some("factories.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_render_prop_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const DataProvider = ({ children }: { children: (data: any) => JSX.Element }) => 
                  children({ data: 'test' })

                export const StateProvider = ({ children }: { children: (state: any) => JSX.Element }) => 
                  children({ state: {} })
            "#},
            filename: Some("providers.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn invalid_context_providers() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { createContext } from 'react'

                const Context = createContext(null)

                export const Provider = ({ children }: { children: React.ReactNode }) => 
                  <Context.Provider value={null}>{children}</Context.Provider>

                export const Consumer = Context.Consumer
            "#},
            filename: Some("context.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 8 }");
}

#[test]
fn valid_single_generic_component() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                interface Props<T> {
                  data: T
                }

                export const GenericList = <T,>(props: Props<T>) => 
                  <ul>{JSON.stringify(props.data)}</ul>
            "#},
            filename: Some("GenericList.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_component_with_non_component_exports() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const Button = () => <button>Click</button>
                export const buttonVariants = ['primary', 'secondary']
                export type ButtonProps = { variant: string }
                export interface ButtonConfig {
                  size: number
                }
            "#},
            filename: Some("Button.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 2 }");
}

#[test]
fn invalid_edge_case_multiple_components_with_different_patterns() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import React, { FC, memo, forwardRef, ReactNode } from 'react'

                export const FcComponent: FC = () => <div>FC</div>
                export const MemoComponent = memo(() => <div>Memo</div>)
                export const RefComponent = forwardRef<HTMLDivElement>((props, ref) => 
                  <div ref={ref} />
                )
                export function FunctionComponent(): ReactNode {
                  return <div>Function</div>
                }
            "#},
            filename: Some("AllPatterns.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 4 }");
}

#[test]
fn valid_non_component_pascalcase_exports() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const API_URL = 'https://api.example.com'
                export const Config = {
                  timeout: 5000,
                  retries: 3
                }
                export class ApiService {
                  fetch() { return Promise.resolve({}) }
                }
            "#},
            filename: Some("config.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_components_returning_different_jsx_types() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const Fragment = () => <><span>Fragment</span></>
                export const NullComponent = () => null
                export const ArrayComponent = () => [<div key="1">1</div>, <div key="2">2</div>]
            "#},
            filename: Some("variants.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 2 }");
}

#[test]
fn invalid_compound_components_pattern() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const Card = ({ children }: { children: React.ReactNode }) => <div>{children}</div>
                const CardHeader = ({ children }: { children: React.ReactNode }) => <div>{children}</div>
                const CardBody = ({ children }: { children: React.ReactNode }) => <div>{children}</div>

                Card.Header = CardHeader
                Card.Body = CardBody

                export { Card, CardHeader, CardBody }
            "#},
            filename: Some("Card.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 8 }");
}

#[test]
fn invalid_re_exported_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { Button as BaseButton } from './base'
                import { Input as BaseInput } from './base'

                export const Button = (props: any) => <BaseButton {...props} />
                export const Input = (props: any) => <BaseInput {...props} />
            "#},
            filename: Some("wrapped.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 5 }");
}

#[test]
fn invalid_jsx_file_with_multiple_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const Button = () => <button>Button</button>
                export const Input = () => <input />
            "#},
            options: Some(json!([{"extensions":["tsx"]}])),
            filename: Some("Components.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 2 }");
}

#[test]
fn invalid_components_with_complex_type_annotations() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                interface BaseProps {
                  id: string
                }

                type ButtonProps = BaseProps & {
                  variant: 'primary' | 'secondary'
                }

                type InputProps = BaseProps & {
                  type: string
                }

                export const Button = (props: ButtonProps): JSX.Element => 
                  <button id={props.id} className={props.variant}>Button</button>

                export const Input = (props: InputProps): JSX.Element => 
                  <input id={props.id} type={props.type} />
            "#},
            filename: Some("TypedComponents.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 16 }");
}

#[test]
fn invalid_components_with_hooks() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useState, useEffect } from 'react'

                export const Counter = () => {
                  const [count, setCount] = useState(0)
                  useEffect(() => {}, [])
                  return <div>{count}</div>
                }

                export const Timer = () => {
                  const [time, setTime] = useState(Date.now())
                  useEffect(() => {}, [])
                  return <div>{time}</div>
                }
            "#},
            filename: Some("HookComponents.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 9 }");
}

#[test]
fn invalid_component_with_hook_exports() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useState } from 'react'

                export const useCounter = (initial: number = 0) => {
                  const [count, setCount] = useState(initial)
                  return { count, setCount }
                }

                export const Counter = () => {
                  const { count, setCount } = useCounter()
                  return <div onClick={() => setCount(count + 1)}>{count}</div>
                }
            "#},
            filename: Some("CounterWithHook.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 3 }");
}

#[test]
fn invalid_components_with_displayname() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const Button = () => <button>Button</button>
                Button.displayName = 'CustomButton'

                const Input = () => <input />
                Input.displayName = 'CustomInput'

                export { Button, Input }
            "#},
            filename: Some("NamedComponents.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 7 }");
}

#[test]
fn valid_edge_case_lowercase_function_with_jsx_should_not_be_detected() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const renderHelper = () => <div>Helper</div>
                export const createButton = () => <button>Button</button>
                export const formatComponent = () => <span>Formatted</span>
            "#},
            filename: Some("helpers.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn invalid_pascalcase_with_numbers() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export const Button1 = () => <button>Button 1</button>
                export const Button2 = () => <button>Button 2</button>
            "#},
            filename: Some("NumberedComponents.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 2 }");
}

#[test]
fn invalid_multiple_levels_of_aliasing_for_components_and_re_exported() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const ButtonBase = () => <button>Button</button>
                const InputBase = () => <input />

                const ButtonAlias1 = ButtonBase
                const ButtonAlias2 = ButtonAlias1
                const Button = ButtonAlias2

                const InputAlias1 = InputBase
                const InputAlias2 = InputAlias1
                const Input = InputAlias2

                export { Button as MainButton, Input as MainInput }
            "#},
            filename: Some("AliasedComponents.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'multipleExports', line: 12 }");
}

#[test]
fn valid_multiple_react_hooks_with_jsx() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const useButton = () => {
                  return {
                    render: () => <button>Click me</button>
                  }
                }

                export const useModal = () => {
                  return {
                    modal: <div>Modal content</div>,
                    open: () => {}
                  }
                }
            "#},
            filename: Some("hooks.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_hooks_with_jsx_return_values() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const useTooltip = () => <div>Tooltip</div>
                export const usePopover = () => <div>Popover</div>
                export const useDropdown = () => <div>Dropdown</div>
            "#},
            filename: Some("ui-hooks.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn valid_mixed_hooks_and_utilities_with_jsx() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export const useCustomHook = () => {
                  return <span>Custom</span>
                }

                export const createRenderer = () => <div>Renderer</div>
                export const formatHelper = () => <p>Helper</p>
            "#},
            filename: Some("mixed-hooks.tsx"),
            ..Default::default()
        },
    );
}
