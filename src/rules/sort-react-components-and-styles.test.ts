import { dedent } from '@ls-stack/utils/dedent'
import { describe, expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { sortReactComponentsAndStyles } from './sort-react-components-and-styles'

const { valid, invalid } = createNewTester(sortReactComponentsAndStyles)

test('correctly ordered components and styles', async () => {
  await valid(
    dedent`
      const Container = styled.div\`
        padding: 20px;
      \`

      export function MainComponent() {
        return <Container>Main content</Container>
      }

      const Wrapper = styled.div\`
        margin: 10px;
      \`

      function SubComponent() {
        return <Wrapper>Sub content</Wrapper>
      }
    `,
  )
})

test('styled component above first usage', async () => {
  await valid(
    dedent`
      const ButtonStyle = styled.button\`
        background: blue;
      \`

      function Button() {
        return <ButtonStyle>Click me</ButtonStyle>
      }
    `,
  )
})

test('CSS template tag above first usage', async () => {
  await valid(
    dedent`
      const buttonStyles = css\`
        background: blue;
        padding: 10px;
      \`

      function Button() {
        return <button css={buttonStyles}>Click me</button>
      }
    `,
  )
})

test('ignores non-component code between styles and components', async () => {
  await valid(
    dedent`
      const Container = styled.div\`
        display: flex;
      \`

      export function App() {
        return <Container><Button /></Container>
      }

      type ButtonProps = {
        onClick: () => void
      }

      const BUTTON_VARIANT = 'primary'

      interface Config {
        debug: boolean
      }

      function Button() {
        return <Container>Button</Container>
      }
    `,
  )
})

test('moves styled component above first usage', async () => {
  const { result } = await invalid(dedent`
    export function MainComponent() {
      return <Container>Main content</Container>
    }

    function SubComponent() {
      return <div>Sub</div>
    }

    const Container = styled.div\`
      padding: 20px;
    \`
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'stylesShouldBeAboveUsage', line: 9 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const Container = styled.div\`
      padding: 20px;
    \`

    export function MainComponent() {
      return <Container>Main content</Container>
    }

    function SubComponent() {
      return <div>Sub</div>
    }

    "
  `)
})

test('moves CSS template above first usage', async () => {
  const { result } = await invalid(dedent`
    export function Button() {
      return <button css={buttonStyles}>Click</button>
    }

    const buttonStyles = css\`
      background: blue;
    \`
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'stylesShouldBeAboveUsage', line: 5 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const buttonStyles = css\`
      background: blue;
    \`

    export function Button() {
      return <button css={buttonStyles}>Click</button>
    }

    "
  `)
})

test('multiple styles with different usages', async () => {
  const { result } = await invalid(dedent`
    export function App() {
      return (
        <Container>
          <Button />
          <Card />
        </Container>
      )
    }

    function Button() {
      return <ButtonWrapper>Click me</ButtonWrapper>
    }

    function Card() {
      return <CardWrapper>Card content</CardWrapper>
    }

    const Container = styled.div\`
      padding: 20px;
    \`

    const ButtonWrapper = styled.button\`
      background: blue;
    \`

    const CardWrapper = styled.div\`
      border: 1px solid gray;
    \`
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'stylesShouldBeAboveUsage', line: 18 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const Container = styled.div\`
      padding: 20px;
    \`

    export function App() {
      return (
        <Container>
          <Button />
          <Card />
        </Container>
      )
    }

    const ButtonWrapper = styled.button\`
      background: blue;
    \`

    function Button() {
      return <ButtonWrapper>Click me</ButtonWrapper>
    }

    const CardWrapper = styled.div\`
      border: 1px solid gray;
    \`

    function Card() {
      return <CardWrapper>Card content</CardWrapper>
    }





    "
  `)
})

test('styles used by non-exported components', async () => {
  const { result } = await invalid(dedent`
    export function App() {
      return <Header />
    }

    function Header() {
      return <HeaderWrapper>Header</HeaderWrapper>
    }

    const HeaderWrapper = styled.div\`
      background: gray;
    \`
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'stylesShouldBeAboveUsage', line: 9 }
    "
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "export function App() {
      return <Header />
    }

    const HeaderWrapper = styled.div\`
      background: gray;
    \`

    function Header() {
      return <HeaderWrapper>Header</HeaderWrapper>
    }

    "
  `)
})

test('preserves comments with moved styles', async () => {
  const { result } = await invalid(dedent`
    export function Button() {
      return <StyledButton>Click</StyledButton>
    }

    // This is a styled button component
    const StyledButton = styled.button\`
      background: blue;
    \`
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "// This is a styled button component
    const StyledButton = styled.button\`
      background: blue;
    \`

    export function Button() {
      return <StyledButton>Click</StyledButton>
    }

    "
  `)
})

test('handles memo and forwardRef wrapped components', async () => {
  const { result } = await invalid(dedent`
    export const App = memo(() => {
      return <Container>App</Container>
    })

    const Button = forwardRef(() => {
      return <ButtonStyle>Button</ButtonStyle>
    })

    const Container = styled.div\`
      padding: 20px;
    \`

    const ButtonStyle = styled.button\`
      background: blue;
    \`
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const Container = styled.div\`
      padding: 20px;
    \`

    export const App = memo(() => {
      return <Container>App</Container>
    })

    const ButtonStyle = styled.button\`
      background: blue;
    \`

    const Button = forwardRef(() => {
      return <ButtonStyle>Button</ButtonStyle>
    })



    "
  `)
})

test('handles styled components with function call syntax', async () => {
  const { result } = await invalid(dedent`
    function Button() {
      return <StyledButton>Click</StyledButton>
    }

    const StyledButton = styled(BaseButton)\`
      color: red;
    \`
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const StyledButton = styled(BaseButton)\`
      color: red;
    \`

    function Button() {
      return <StyledButton>Click</StyledButton>
    }

    "
  `)
})

test('unused styles are not moved', async () => {
  await valid(
    dedent`
      export function App() {
        return <div>App</div>
      }

      function Button() {
        return <div>Button</div>
      }

      const UnusedStyle = styled.div\`
        color: red;
      \`
    `,
  )
})

test('main component regex option', async () => {
  const { result } = await invalid({
    code: dedent`
      function RegularComponent() {
        return <Container>Regular</Container>
      }

      function AppComponent() {
        return <div>App</div>
      }

      const Container = styled.div\`
        padding: 20px;
      \`
    `,
    options: [{ mainComponentRegex: '.*Component$' }],
  })

  expect(result.output).toMatchInlineSnapshot(`
    "const Container = styled.div\`
      padding: 20px;
    \`

    function RegularComponent() {
      return <Container>Regular</Container>
    }

    function AppComponent() {
      return <div>App</div>
    }

    "
  `)
})

test('mainComponentRegex takes precedence over exported components', async () => {
  const { result } = await invalid({
    code: dedent`
      export default function MainApp() {
        return <Container>Main</Container>
      }

      function SubComponent() {
        return <div>Sub</div>
      }

      const Container = styled.div\`
        padding: 20px;
      \`
    `,
    options: [{ mainComponentRegex: 'Sub.*' }],
  })

  expect(result.output).toMatchInlineSnapshot(`
    "

    function SubComponent() {
      return <div>Sub</div>
    }
    const Container = styled.div\`
      padding: 20px;
    \`

    export default function MainApp() {
      return <Container>Main</Container>
    }



    "
  `)
})

test('exported components are main when no regex provided', async () => {
  await valid(
    dedent`
      const Container = styled.div\`
        padding: 20px;
      \`

      export default function MainApp() {
        return <Container>Main</Container>
      }

      function SubComponent() {
        return <div>Sub</div>
      }
    `,
  )
})

test('handles arrow function components', async () => {
  const { result } = await invalid(dedent`
    const Button = () => {
      return <ButtonStyle>Click</ButtonStyle>
    }

    const ButtonStyle = styled.button\`
      background: blue;
    \`
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const ButtonStyle = styled.button\`
      background: blue;
    \`

    const Button = () => {
      return <ButtonStyle>Click</ButtonStyle>
    }

    "
  `)
})

test('handles FC typed components', async () => {
  const { result } = await invalid(dedent`

    const Wrapper = styled.div\`
      margin: 10px;
    \`

    const Container = styled.div\`
      padding: 20px;
    \`

    type Props = {
      title: string
    }

    export const MainComponent: FC<Props> = ({ title }) => {
      return <Container>{title}</Container>
    }

    const SubComponent: FC = () => {
      return <Wrapper>Sub</Wrapper>
    }
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "

    const Container = styled.div\`
      padding: 20px;
    \`

    type Props = {
      title: string
    }

    export const MainComponent: FC<Props> = ({ title }) => {
      return <Container>{title}</Container>
    }

    const Wrapper = styled.div\`
      margin: 10px;
    \`

    const SubComponent: FC = () => {
      return <Wrapper>Sub</Wrapper>
    }"
  `)
})

test('Respects references between styled components', async () => {
  await valid(dedent`
    const Wrapper = styled.div\`
      margin: 10px;
    \`

    const Container = styled.div\`
      padding: 20px;

      \${Wrapper} {
        background: red;
      }
    \`

    type Props = {
      title: string
    }

    export const MainComponent: FC<Props> = ({ title }) => {
      return <Container>{title}</Container>
    }

    const SubComponent: FC = () => {
      return <Wrapper>Sub</Wrapper>
    }
  `)
})

test('does not move derived styled before its base (styled call)', async () => {
  const { result } = await invalid(dedent`
    const Base = styled.div\`
      color: blue;
    \`

    export const App = () => {
      return <Derived>Hi</Derived>
    }

    const Derived = styled(Base)\`
      font-weight: bold;
    \`
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const Base = styled.div\`
      color: blue;
    \`

    const Derived = styled(Base)\`
      font-weight: bold;
    \`

    export const App = () => {
      return <Derived>Hi</Derived>
    }

    "
  `)
})

test('respects css mixin dependencies inside styled template', async () => {
  const { result } = await invalid(dedent`
    const mixin = css\`
      display: flex;
    \`

    export const App = () => {
      return <Container>Hello</Container>
    }

    const Container = styled.div\`
      \${mixin};
      gap: 8px;
    \`
  `)

  expect(result.output).toMatchInlineSnapshot(`
    "const mixin = css\`
      display: flex;
    \`

    const Container = styled.div\`
      \${mixin};
      gap: 8px;
    \`

    export const App = () => {
      return <Container>Hello</Container>
    }

    "
  `)
})

describe('components reordering', () => {
  test('main component is first', async () => {
    const { result } = await invalid(dedent`
      const NonMainComponent: FC = () => {
        return <div>NonMain</div>
      }

      export const MainComponent: FC = () => {
        return <div>Main</div>
      }
    `)

    expect(result.output).toMatchInlineSnapshot(`
      "export const MainComponent: FC = () => {
        return <div>Main</div>
      }
      const NonMainComponent: FC = () => {
        return <div>NonMain</div>
      }

      "
    `)
  })

  test('main component with configured regex takes precedence', async () => {
    const { result } = await invalid({
      code: dedent`

        export const Component: FC = () => {
          return <div>Component</div>
        }
        
        const NonExportedComponentMain: FC = () => {
          return <div>NonMain</div>
        }
      `,
      options: [{ mainComponentRegex: 'NonExportedComponentMain' }],
    })

    expect(result.output).toMatchInlineSnapshot(`
      "const NonExportedComponentMain: FC = () => {
        return <div>NonMain</div>
      }
      export const Component: FC = () => {
        return <div>Component</div>
      }

      "
    `)
  })
})
