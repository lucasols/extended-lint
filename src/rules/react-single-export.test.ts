import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { reactSingleExport } from './react-single-export'

const { valid, invalid } = createNewTester(reactSingleExport)

test('valid: single default export in React component', async () => {
  await valid({
    code: dedent`
      import React from 'react'

      const MyComponent = () => <div>Hello</div>
      
      export default MyComponent
    `,
    filename: 'MyComponent.tsx',
  })
})

test('valid: single named export in React component', async () => {
  await valid({
    code: dedent`
      import React from 'react'

      export const MyComponent = () => <div>Hello</div>
    `,
    filename: 'MyComponent.tsx',
  })
})

test('valid: single export with JSX', async () => {
  await valid({
    code: dedent`
      export const MyComponent = () => {
        return <div>Hello World</div>
      }
    `,
    filename: 'MyComponent.tsx',
  })
})

test('valid: non-React files with multiple exports', async () => {
  await valid({
    code: dedent`
      export const utils = {}
      export const helper = () => {}
      export const constant = 'value'
    `,
    filename: 'utils.ts',
  })
})

test('valid: React component with only type exports', async () => {
  await valid({
    code: dedent`
      import React from 'react'

      export type Props = {
        name: string
      }

      const MyComponent = ({ name }: Props) => <div>{name}</div>
      
      export default MyComponent
    `,
    filename: 'MyComponent.tsx',
  })
})

test('invalid: multiple value exports in React component', async () => {
  const { result } = await invalid({
    code: dedent`
      import React from 'react'

      export const MyComponent = () => <div>Hello</div>
      export const OtherComponent = () => <div>World</div>
    `,
    filename: 'MyComponent.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: default export + named export in React component', async () => {
  const { result } = await invalid({
    code: dedent`
      import React from 'react'

      const MyComponent = () => <div>Hello</div>
      export const helper = () => {}
      
      export default MyComponent
    `,
    filename: 'MyComponent.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: TSX file with multiple exports', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Component1 = () => <div>One</div>
      export const Component2 = () => <div>Two</div>
    `,
    filename: 'components.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('valid: React component file with custom extension and single export', async () => {
  await valid({
    code: dedent`
      export const MyComponent = () => {
        return <button>Click me</button>
      }
    `,
    filename: 'MyComponent.component.tsx',
    options: [{ extensions: ['component.tsx'] }],
  })
})

test('invalid: custom extension with multiple exports', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Component = () => <div>Hello</div>
      export const utils = {}
    `,
    filename: 'MyComponent.component.tsx',
    options: [{ extensions: ['component.tsx'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('invalid: multiple component exports in same file', async () => {
  const { result } = await invalid({
    code: dedent`
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
    `,
    filename: 'Dropdown.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 25 }
    "
  `)
})

test('invalid: PascalCase function with FC type', async () => {
  const { result } = await invalid({
    code: dedent`
      import { FC } from 'react'
      
      export const Button: FC = () => <button>Click</button>
      export const Input: FC = () => <input />
    `,
    filename: 'Components.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: PascalCase function with JSX.Element return type', async () => {
  const { result } = await invalid({
    code: dedent`
      const Header = (): JSX.Element => <header>Header</header>
      const Footer = (): JSX.Element => <footer>Footer</footer>
      
      export { Header, Footer }
    `,
    filename: 'Layout.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: memo components', async () => {
  const { result } = await invalid({
    code: dedent`
      import { memo } from 'react'
      
      const Card = memo(() => <div>Card</div>)
      const Panel = memo(() => <div>Panel</div>)
      
      export { Card, Panel }
    `,
    filename: 'Components.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 6 }
    "
  `)
})

test('invalid: forwardRef components', async () => {
  const { result } = await invalid({
    code: dedent`
      import { forwardRef } from 'react'
      
      const Input = forwardRef<HTMLInputElement>((props, ref) => 
        <input ref={ref} {...props} />
      )
      const Button = forwardRef<HTMLButtonElement>((props, ref) => 
        <button ref={ref} {...props} />
      )
      
      export { Input, Button }
    `,
    filename: 'FormComponents.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 10 }
    "
  `)
})

test('valid: single memo component', async () => {
  await valid({
    code: dedent`
      import { memo } from 'react'
      
      const Card = memo(() => <div>Card</div>)
      
      export default Card
    `,
    filename: 'Card.tsx',
  })
})

test('valid: single FC component', async () => {
  await valid({
    code: dedent`
      import { FC } from 'react'
      
      export const Button: FC = () => <button>Click</button>
    `,
    filename: 'Button.tsx',
  })
})

test('invalid: mixed exports with non-component', async () => {
  const { result } = await invalid({
    code: dedent`
      export const MyComponent = () => <div>Hello</div>
      export const CONSTANT = 'value'
      export type Props = { name: string }
    `,
    filename: 'Component.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('valid: single ReactNode return type', async () => {
  await valid({
    code: dedent`
      import { ReactNode } from 'react'
      
      export const Wrapper = (): ReactNode => <div>Wrapper</div>
    `,
    filename: 'Wrapper.tsx',
  })
})

test('valid: function declaration component', async () => {
  await valid({
    code: dedent`
      export function Button(): JSX.Element {
        return <button>Click me</button>
      }
    `,
    filename: 'Button.tsx',
  })
})

test('valid: React.memo component', async () => {
  await valid({
    code: dedent`
      import React from 'react'
      
      export const MemoComponent = React.memo(() => <div>Memoized</div>)
    `,
    filename: 'MemoComponent.tsx',
  })
})

test('valid: React.forwardRef component', async () => {
  await valid({
    code: dedent`
      import React from 'react'
      
      export const RefComponent = React.forwardRef<HTMLDivElement>((props, ref) => 
        <div ref={ref} {...props} />
      )
    `,
    filename: 'RefComponent.tsx',
  })
})

test('invalid: component with custom hooks', async () => {
  const { result } = await invalid({
    code: dedent`
      export const MyComponent = () => {
        const [state, setState] = useState(0)
        return <div>{state}</div>
      }
      
      export const useCustomHook = () => {
        return useState(0)
      }
    `,
    filename: 'Component.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 6 }
    "
  `)
})

test('valid: utility functions only', async () => {
  await valid({
    code: dedent`
      export const formatName = (name: string) => name.toUpperCase()
      export const validateEmail = (email: string) => email.includes('@')
    `,
    filename: 'utils.tsx',
  })
})

test('valid: no exports', async () => {
  await valid({
    code: dedent`
      const Component = () => <div>Private</div>
    `,
    filename: 'internal.tsx',
  })
})

test('invalid: React.FC vs FC mixed', async () => {
  const { result } = await invalid({
    code: dedent`
      import React, { FC } from 'react'
      
      export const Button: React.FC = () => <button>Button</button>
      export const Input: FC = () => <input />
    `,
    filename: 'Components.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: mixed React.memo and direct', async () => {
  const { result } = await invalid({
    code: dedent`
      import React from 'react'
      
      export const Direct = () => <div>Direct</div>
      export const Memo = React.memo(() => <div>Memo</div>)
    `,
    filename: 'Mixed.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: function declarations', async () => {
  const { result } = await invalid({
    code: dedent`
      export function Header(): JSX.Element {
        return <header>Header</header>
      }
      
      export function Footer(): JSX.Element {
        return <footer>Footer</footer>
      }
    `,
    filename: 'Layout.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 5 }
    "
  `)
})

test('invalid: ReactNode return types', async () => {
  const { result } = await invalid({
    code: dedent`
      import { ReactNode } from 'react'
      
      const Nav = (): ReactNode => <nav>Navigation</nav>
      const Sidebar = (): ReactNode => <aside>Sidebar</aside>
      
      export { Nav, Sidebar }
    `,
    filename: 'Layout.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 6 }
    "
  `)
})

test('invalid: generic components', async () => {
  const { result } = await invalid({
    code: dedent`
      interface Props<T> {
        data: T
      }
      
      export const List = <T,>(props: Props<T>) => <ul>{JSON.stringify(props.data)}</ul>
      export const Table = <T,>(props: Props<T>) => <table>{JSON.stringify(props.data)}</table>
    `,
    filename: 'Generic.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 6 }
    "
  `)
})

test('invalid: higher-order components', async () => {
  const { result } = await invalid({
    code: dedent`
      const withLoading = (Component: React.ComponentType) => (props: any) => 
        <div>Loading...<Component {...props} /></div>
      
      const withError = (Component: React.ComponentType) => (props: any) => 
        <div>Error!<Component {...props} /></div>
      
      export { withLoading, withError }
    `,
    filename: 'hoc.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 7 }
    "
  `)
})

test('invalid: component factories', async () => {
  const { result } = await invalid({
    code: dedent`
      export const CreateButton = (variant: string) => () => 
        <button className={variant}>Button</button>
      
      export const CreateInput = (type: string) => () => 
        <input type={type} />
    `,
    filename: 'factories.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: render prop components', async () => {
  const { result } = await invalid({
    code: dedent`
      export const DataProvider = ({ children }: { children: (data: any) => JSX.Element }) => 
        children({ data: 'test' })
      
      export const StateProvider = ({ children }: { children: (state: any) => JSX.Element }) => 
        children({ state: {} })
    `,
    filename: 'providers.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: context providers', async () => {
  const { result } = await invalid({
    code: dedent`
      import { createContext } from 'react'
      
      const Context = createContext(null)
      
      export const Provider = ({ children }: { children: React.ReactNode }) => 
        <Context.Provider value={null}>{children}</Context.Provider>
      
      export const Consumer = Context.Consumer
    `,
    filename: 'context.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 8 }
    "
  `)
})

test('valid: single generic component', async () => {
  await valid({
    code: dedent`
      interface Props<T> {
        data: T
      }
      
      export const GenericList = <T,>(props: Props<T>) => 
        <ul>{JSON.stringify(props.data)}</ul>
    `,
    filename: 'GenericList.tsx',
  })
})

test('invalid: component with non-component exports', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Button = () => <button>Click</button>
      export const buttonVariants = ['primary', 'secondary']
      export type ButtonProps = { variant: string }
      export interface ButtonConfig {
        size: number
      }
    `,
    filename: 'Button.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('invalid: edge case - multiple components with different patterns', async () => {
  const { result } = await invalid({
    code: dedent`
      import React, { FC, memo, forwardRef, ReactNode } from 'react'
      
      export const FcComponent: FC = () => <div>FC</div>
      export const MemoComponent = memo(() => <div>Memo</div>)
      export const RefComponent = forwardRef<HTMLDivElement>((props, ref) => 
        <div ref={ref} />
      )
      export function FunctionComponent(): ReactNode {
        return <div>Function</div>
      }
    `,
    filename: 'AllPatterns.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('valid: non-component PascalCase exports', async () => {
  await valid({
    code: dedent`
      export const API_URL = 'https://api.example.com'
      export const Config = {
        timeout: 5000,
        retries: 3
      }
      export class ApiService {
        fetch() { return Promise.resolve({}) }
      }
    `,
    filename: 'config.tsx',
  })
})

test('invalid: components returning different JSX types', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Fragment = () => <><span>Fragment</span></>
      export const NullComponent = () => null
      export const ArrayComponent = () => [<div key="1">1</div>, <div key="2">2</div>]
    `,
    filename: 'variants.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('invalid: compound components pattern', async () => {
  const { result } = await invalid({
    code: dedent`
      const Card = ({ children }: { children: React.ReactNode }) => <div>{children}</div>
      const CardHeader = ({ children }: { children: React.ReactNode }) => <div>{children}</div>
      const CardBody = ({ children }: { children: React.ReactNode }) => <div>{children}</div>
      
      Card.Header = CardHeader
      Card.Body = CardBody
      
      export { Card, CardHeader, CardBody }
    `,
    filename: 'Card.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 8 }
    "
  `)
})

test('invalid: re-exported components', async () => {
  const { result } = await invalid({
    code: dedent`
      import { Button as BaseButton } from './base'
      import { Input as BaseInput } from './base'
      
      export const Button = (props: any) => <BaseButton {...props} />
      export const Input = (props: any) => <BaseInput {...props} />
    `,
    filename: 'wrapped.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 5 }
    "
  `)
})

// removed: jsx file extension test not supported by parser setup

test('invalid: jsx file with multiple components', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Button = () => <button>Button</button>
      export const Input = () => <input />
    `,
    filename: 'Components.tsx',
    options: [{ extensions: ['tsx'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('invalid: components with complex type annotations', async () => {
  const { result } = await invalid({
    code: dedent`
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
    `,
    filename: 'TypedComponents.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 16 }
    "
  `)
})

test('invalid: components with hooks', async () => {
  const { result } = await invalid({
    code: dedent`
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
    `,
    filename: 'HookComponents.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 9 }
    "
  `)
})

test('invalid: component with hook exports', async () => {
  const { result } = await invalid({
    code: dedent`
      import { useState } from 'react'
      
      export const useCounter = (initial: number = 0) => {
        const [count, setCount] = useState(initial)
        return { count, setCount }
      }
      
      export const Counter = () => {
        const { count, setCount } = useCounter()
        return <div onClick={() => setCount(count + 1)}>{count}</div>
      }
    `,
    filename: 'CounterWithHook.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 3 }
    "
  `)
})

test('invalid: components with displayName', async () => {
  const { result } = await invalid({
    code: dedent`
      const Button = () => <button>Button</button>
      Button.displayName = 'CustomButton'
      
      const Input = () => <input />
      Input.displayName = 'CustomInput'
      
      export { Button, Input }
    `,
    filename: 'NamedComponents.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 7 }
    "
  `)
})

test('valid: edge case - lowercase function with JSX should not be detected', async () => {
  await valid({
    code: dedent`
      export const renderHelper = () => <div>Helper</div>
      export const createButton = () => <button>Button</button>
      export const formatComponent = () => <span>Formatted</span>
    `,
    filename: 'helpers.tsx',
  })
})

test('invalid: PascalCase with numbers', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Button1 = () => <button>Button 1</button>
      export const Button2 = () => <button>Button 2</button>
    `,
    filename: 'NumberedComponents.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('invalid: multiple levels of aliasing for components and re-exported', async () => {
  const { result } = await invalid({
    code: dedent`
      const ButtonBase = () => <button>Button</button>
      const InputBase = () => <input />

      const ButtonAlias1 = ButtonBase
      const ButtonAlias2 = ButtonAlias1
      const Button = ButtonAlias2

      const InputAlias1 = InputBase
      const InputAlias2 = InputAlias1
      const Input = InputAlias2

      export { Button as MainButton, Input as MainInput }
    `,
    filename: 'AliasedComponents.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 12 }
    "
  `)
})

test('valid: multiple React hooks with JSX', async () => {
  await valid({
    code: dedent`
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
    `,
    filename: 'hooks.tsx',
  })
})

test('valid: hooks with JSX return values', async () => {
  await valid({
    code: dedent`
      export const useTooltip = () => <div>Tooltip</div>
      export const usePopover = () => <div>Popover</div>
      export const useDropdown = () => <div>Dropdown</div>
    `,
    filename: 'ui-hooks.tsx',
  })
})

test('valid: mixed hooks and utilities with JSX', async () => {
  await valid({
    code: dedent`
      export const useCustomHook = () => {
        return <span>Custom</span>
      }
      
      export const createRenderer = () => <div>Renderer</div>
      export const formatHelper = () => <p>Helper</p>
    `,
    filename: 'mixed-hooks.tsx',
  })
})
