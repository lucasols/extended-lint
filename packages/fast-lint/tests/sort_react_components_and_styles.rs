mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "sort-react-components-and-styles";

#[test]
fn correctly_ordered_components_and_styles() {
    valid(
        RULE,
        indoc! {r#"
        const Container = styled.div`
          padding: 20px;
        `

        export function MainComponent() {
          return <Container>Main content</Container>
        }

        const Wrapper = styled.div`
          margin: 10px;
        `

        function SubComponent() {
          return <Wrapper>Sub content</Wrapper>
        }
    "#},
    );
}

#[test]
fn styled_component_above_first_usage() {
    valid(
        RULE,
        indoc! {r#"
        const ButtonStyle = styled.button`
          background: blue;
        `

        function Button() {
          return <ButtonStyle>Click me</ButtonStyle>
        }
    "#},
    );
}

#[test]
fn css_template_tag_above_first_usage() {
    valid(
        RULE,
        indoc! {r#"
        const buttonStyles = css`
          background: blue;
          padding: 10px;
        `

        function Button() {
          return <button css={buttonStyles}>Click me</button>
        }
    "#},
    );
}

#[test]
fn ignores_non_component_code_between_styles_and_components() {
    valid(
        RULE,
        indoc! {r#"
        const Container = styled.div`
          display: flex;
        `

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
    "#},
    );
}

#[test]
fn moves_styled_component_above_first_usage() {
    let result = invalid(
        RULE,
        indoc! {r#"
        export function MainComponent() {
          return <Container>Main content</Container>
        }

        function SubComponent() {
          return <div>Sub</div>
        }

        const Container = styled.div`
          padding: 20px;
        `
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'stylesShouldBeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r#"
    const Container = styled.div`
      padding: 20px;
    `

    export function MainComponent() {
      return <Container>Main content</Container>
    }

    function SubComponent() {
      return <div>Sub</div>
    }

    "#);
}

#[test]
fn moves_css_template_above_first_usage() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { css } from '@emotion/react'

        export function Button() {
          return <button css={buttonStyles}>Click</button>
        }

        const buttonStyles = css`
          background: blue;
        `
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'stylesShouldBeAboveUsage', line: 7 }
    "#);
    assert_snapshot!(result.output, @r#"
    import { css } from '@emotion/react'

    const buttonStyles = css`
      background: blue;
    `

    export function Button() {
      return <button css={buttonStyles}>Click</button>
    }

    "#);
}

#[test]
fn skips_non_tsx_files() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            export const Component: FC = (): JSX.Element => {
              return null
            }

            const buttonStyles = css`
              background: blue;
            `
        "#},
            filename: Some("buttonStyles.ts"),
            options: None,
        },
    );
}

#[test]
fn multiple_styles_with_different_usages() {
    let result = invalid(
        RULE,
        indoc! {r#"
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

        const Container = styled.div`
          padding: 20px;
        `

        const ButtonWrapper = styled.button`
          background: blue;
        `

        const CardWrapper = styled.div`
          border: 1px solid gray;
        `
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'stylesShouldBeAboveUsage', line: 18 }
    "#);
    assert_snapshot!(result.output, @r#"
    const Container = styled.div`
      padding: 20px;
    `

    export function App() {
      return (
        <Container>
          <Button />
          <Card />
        </Container>
      )
    }

    const ButtonWrapper = styled.button`
      background: blue;
    `

    function Button() {
      return <ButtonWrapper>Click me</ButtonWrapper>
    }

    const CardWrapper = styled.div`
      border: 1px solid gray;
    `

    function Card() {
      return <CardWrapper>Card content</CardWrapper>
    }





    "#);
}

#[test]
fn styles_used_by_non_exported_components() {
    let result = invalid(
        RULE,
        indoc! {r#"
        export function App() {
          return <Header />
        }

        function Header() {
          return <HeaderWrapper>Header</HeaderWrapper>
        }

        const HeaderWrapper = styled.div`
          background: gray;
        `
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'stylesShouldBeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r#"
    export function App() {
      return <Header />
    }

    const HeaderWrapper = styled.div`
      background: gray;
    `

    function Header() {
      return <HeaderWrapper>Header</HeaderWrapper>
    }

    "#);
}

#[test]
fn preserves_comments_with_moved_styles() {
    let result = invalid(
        RULE,
        indoc! {r#"
        export function Button() {
          return <StyledButton>Click</StyledButton>
        }

        // This is a styled button component
        const StyledButton = styled.button`
          background: blue;
        `
    "#},
    );
    assert_snapshot!(result.output, @r#"
    // This is a styled button component
    const StyledButton = styled.button`
      background: blue;
    `

    export function Button() {
      return <StyledButton>Click</StyledButton>
    }

    "#);
}

#[test]
fn handles_memo_and_forwardref_wrapped_components() {
    let result = invalid(
        RULE,
        indoc! {r#"
        export const App = memo(() => {
          return <Container>App</Container>
        })

        const Button = forwardRef(() => {
          return <ButtonStyle>Button</ButtonStyle>
        })

        const Container = styled.div`
          padding: 20px;
        `

        const ButtonStyle = styled.button`
          background: blue;
        `
    "#},
    );
    assert_snapshot!(result.output, @r#"
    const Container = styled.div`
      padding: 20px;
    `

    export const App = memo(() => {
      return <Container>App</Container>
    })

    const ButtonStyle = styled.button`
      background: blue;
    `

    const Button = forwardRef(() => {
      return <ButtonStyle>Button</ButtonStyle>
    })



    "#);
}

#[test]
fn handles_styled_components_with_function_call_syntax() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function Button() {
          return <StyledButton>Click</StyledButton>
        }

        const StyledButton = styled(BaseButton)`
          color: red;
        `
    "#},
    );
    assert_snapshot!(result.output, @r#"
    const StyledButton = styled(BaseButton)`
      color: red;
    `

    function Button() {
      return <StyledButton>Click</StyledButton>
    }

    "#);
}

#[test]
fn unused_styles_are_not_moved() {
    valid(
        RULE,
        indoc! {r#"
        export function App() {
          return <div>App</div>
        }

        function Button() {
          return <div>Button</div>
        }

        const UnusedStyle = styled.div`
          color: red;
        `
    "#},
    );
}

#[test]
fn main_component_regex_option() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            function RegularComponent() {
              return <Container>Regular</Container>
            }

            function AppComponent() {
              return <div>App</div>
            }

            const Container = styled.div`
              padding: 20px;
            `
        "#},
            options: Some(json!([{"mainComponentRegex":".*Component$"}])),
            filename: None,
        },
    );
    assert_snapshot!(result.output, @r#"
    const Container = styled.div`
      padding: 20px;
    `

    function RegularComponent() {
      return <Container>Regular</Container>
    }

    function AppComponent() {
      return <div>App</div>
    }

    "#);
}

#[test]
fn maincomponentregex_takes_precedence_over_exported_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            export default function MainApp() {
              return <Container>Main</Container>
            }

            function SubComponent() {
              return <div>Sub</div>
            }

            const Container = styled.div`
              padding: 20px;
            `
        "#},
            options: Some(json!([{"mainComponentRegex":"Sub.*"}])),
            filename: None,
        },
    );
    assert_snapshot!(result.output, @r#"


    function SubComponent() {
      return <div>Sub</div>
    }
    const Container = styled.div`
      padding: 20px;
    `

    export default function MainApp() {
      return <Container>Main</Container>
    }



    "#);
}

#[test]
fn exported_components_are_main_when_no_regex_provided() {
    valid(
        RULE,
        indoc! {r#"
        const Container = styled.div`
          padding: 20px;
        `

        export default function MainApp() {
          return <Container>Main</Container>
        }

        function SubComponent() {
          return <div>Sub</div>
        }
    "#},
    );
}

#[test]
fn handles_arrow_function_components() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const Button = () => {
          return <ButtonStyle>Click</ButtonStyle>
        }

        const ButtonStyle = styled.button`
          background: blue;
        `
    "#},
    );
    assert_snapshot!(result.output, @r#"
    const ButtonStyle = styled.button`
      background: blue;
    `

    const Button = () => {
      return <ButtonStyle>Click</ButtonStyle>
    }

    "#);
}

#[test]
fn handles_fc_typed_components() {
    let result = invalid(
        RULE,
        indoc! {r#"

        const Wrapper = styled.div`
          margin: 10px;
        `

        const Container = styled.div`
          padding: 20px;
        `

        type Props = {
          title: string
        }

        export const MainComponent: FC<Props> = ({ title }) => {
          return <Container>{title}</Container>
        }

        const SubComponent: FC = () => {
          return <Wrapper>Sub</Wrapper>
        }
    "#},
    );
    assert_snapshot!(result.output, @r#"


    const Container = styled.div`
      padding: 20px;
    `

    type Props = {
      title: string
    }

    export const MainComponent: FC<Props> = ({ title }) => {
      return <Container>{title}</Container>
    }

    const Wrapper = styled.div`
      margin: 10px;
    `

    const SubComponent: FC = () => {
      return <Wrapper>Sub</Wrapper>
    }
    "#);
}

#[test]
fn respects_references_between_styled_components() {
    valid(
        RULE,
        indoc! {r#"
        const Wrapper = styled.div`
          margin: 10px;
        `

        const Container = styled.div`
          padding: 20px;

          ${Wrapper} {
            background: red;
          }
        `

        type Props = {
          title: string
        }

        export const MainComponent: FC<Props> = ({ title }) => {
          return <Container>{title}</Container>
        }

        const SubComponent: FC = () => {
          return <Wrapper>Sub</Wrapper>
        }
    "#},
    );
}

#[test]
fn does_not_move_derived_styled_before_its_base_styled_call() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const Base = styled.div`
          color: blue;
        `

        export const App = () => {
          return <Derived>Hi</Derived>
        }

        const Derived = styled(Base)`
          font-weight: bold;
        `
    "#},
    );
    assert_snapshot!(result.output, @r#"
    const Base = styled.div`
      color: blue;
    `

    const Derived = styled(Base)`
      font-weight: bold;
    `

    export const App = () => {
      return <Derived>Hi</Derived>
    }

    "#);
}

#[test]
fn respects_css_mixin_dependencies_inside_styled_template() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const mixin = css`
          display: flex;
        `

        export const App = () => {
          return <Container>Hello</Container>
        }

        const Container = styled.div`
          ${mixin};
          gap: 8px;
        `
    "#},
    );
    assert_snapshot!(result.output, @r#"
    const mixin = css`
      display: flex;
    `

    const Container = styled.div`
      ${mixin};
      gap: 8px;
    `

    export const App = () => {
      return <Container>Hello</Container>
    }

    "#);
}

#[test]
fn main_component_is_first() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const NonMainComponent: FC = () => {
          return <div>NonMain</div>
        }

        export const MainComponent: FC = () => {
          return <div>Main</div>
        }
    "#},
    );
    assert_snapshot!(result.output, @r#"
    export const MainComponent: FC = () => {
      return <div>Main</div>
    }
    const NonMainComponent: FC = () => {
      return <div>NonMain</div>
    }

    "#);
}

#[test]
fn main_component_with_configured_regex_takes_precedence() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"

            export const Component: FC = () => {
              return <div>Component</div>
            }

            const NonExportedComponentMain: FC = () => {
              return <div>NonMain</div>
            }
        "#},
            options: Some(json!([{"mainComponentRegex":"NonExportedComponentMain"}])),
            filename: None,
        },
    );
    assert_snapshot!(result.output, @r#"
    const NonExportedComponentMain: FC = () => {
      return <div>NonMain</div>
    }
    export const Component: FC = () => {
      return <div>Component</div>
    }

    "#);
}

#[test]
fn main_component_can_be_selected_via_ast_selector() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"

            export const Component: FC = () => {
              return <div>Component</div>
            }

            const NonExportedComponentMain: FC = () => {
              return <div>NonMain</div>
            }
        "#},
            options: Some(
                json!([{"mainComponentSelector":"VariableDeclarator[id.name='NonExportedComponentMain']"}]),
            ),
            filename: None,
        },
    );
    assert_snapshot!(result.output, @r#"
    const NonExportedComponentMain: FC = () => {
      return <div>NonMain</div>
    }
    export const Component: FC = () => {
      return <div>Component</div>
    }

    "#);
}

#[test]
fn maincomponentselector_takes_precedence_over_maincomponentregex() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"

            export const Component: FC = () => {
              return <div>Component</div>
            }

            const NonExportedComponentMain: FC = () => {
              return <div>NonMain</div>
            }
        "#},
            options: Some(
                json!([{"mainComponentRegex":"Component","mainComponentSelector":"VariableDeclarator[id.name='NonExportedComponentMain']"}]),
            ),
            filename: None,
        },
    );
    assert_snapshot!(result.output, @r#"
    const NonExportedComponentMain: FC = () => {
      return <div>NonMain</div>
    }
    export const Component: FC = () => {
      return <div>Component</div>
    }

    "#);
}
