mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "react-compiler-extra";

#[test]
fn non_hook_call_with_object_method() {
    valid(
        RULE,
        indoc! {r#"
        const result = someFunction({
          method() {
            return 42
          }
        });
    "#},
    );
}

#[test]
fn hook_call_without_object_method() {
    valid(
        RULE,
        indoc! {r#"
        const result = useState({
          method: () => {
            return 42
          }
        });
    "#},
    );
}

#[test]
fn hook_call_with_function_expression() {
    valid(
        RULE,
        indoc! {r#"
        const result = useCallback({
          method: () => {
            return 42
          }
        }, []);
    "#},
    );
}

#[test]
fn hook_call_with_object_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useState({
          method() {
            return 42
          }
        });
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useState({
      method: () => {
        return 42
      }
    });
    "#);
}

#[test]
fn hook_call_with_object_method_using_this() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useState({
          method() {
            this.value = 42;
            return this.value;
          }
        });
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'thisKeywordInMethod', line: 2 }
    "#);
}

#[test]
fn hook_call_with_nested_this_usage() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useState({
          method() {
            if (true) {
              console.log(this.value);
            }
            return 42;
          }
        });
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'thisKeywordInMethod', line: 2 }
    "#);
}

#[test]
fn usecallback_with_object_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useCallback({
          method() {
            return 42
          }
        }, []);
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useCallback({
      method: () => {
        return 42
      }
    }, []);
    "#);
}

#[test]
fn hook_call_with_async_object_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useState({
          async method() {
            return 42
          }
        });
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useState({
      method: async () => {
        return 42
      }
    });
    "#);
}

#[test]
fn usememo_with_object_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useMemo(() => ({
          method() {
            return 42
          }
        }), []);
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useMemo(() => ({
      method: () => {
        return 42
      }
    }), []);
    "#);
}

#[test]
fn multiple_object_methods_in_hook_call() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useCallback({
          method1() {
            return 42
          },
          method2() {
            console.log('hello')
          }
        }, []);
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    - { messageId: 'objectMethodIsNotSupported', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useCallback({
      method1: () => {
        return 42
      },
      method2: () => {
        console.log('hello')
      }
    }, []);
    "#);
}

#[test]
fn mixed_methods_with_and_without_this() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = useCallback({
              method1() {
                return 42;
              },
              method2() {
                return this.value;
              }
            }, []);
        "#},
            options: None,
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    - { messageId: 'thisKeywordInMethod', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useCallback({
      method1: () => {
        return 42;
      },
      method2() {
        return this.value;
      }
    }, []);
    "#);
}

#[test]
fn hook_call_with_generator_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useMemo(() => ({
          *generator() {
            yield 42
          }
        }), []);
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useMemo(() => ({
      generator: function* () {
        yield 42
      }
    }), []);
    "#);
}

#[test]
fn hook_call_with_object_method_with_parameters() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useState({
          method(a, b, c) {
            return a + b + c
          }
        });
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = useState({
      method: (a, b, c) => {
        return a + b + c
      }
    });
    "#);
}

#[test]
fn hook_call_with_object_method_with_parameters_in_namespace_hook() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = namespace.useState({
          method(a, b, c) {
            return a + b + c
          }
        });
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 2 }
    "#);
    assert_snapshot!(result.output, @r#"
    const result = namespace.useState({
      method: (a, b, c) => {
        return a + b + c
      }
    });
    "#);
}

#[test]
fn nested_hook_call_with_object_method_is_ok() {
    valid(
        RULE,
        indoc! {r#"
        const result = useState({
          outer: {
            inner() {
              return 42
            }
          }
        });
    "#},
    );
}

#[test]
fn nested_hook_call_with_object_method_using_this() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const result = useState({
          outer: {
            inner() {
              return this.value
            }
          }
        });
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'thisKeywordInMethod', line: 3 }
    "#);
}

#[test]
fn hook_function_returning_object_with_object_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function useResult() {
          const state = useState(0)
          return {
            method() {
              return state
            }
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 4 }
    "#);
    assert_snapshot!(result.output, @r#"
    function useResult() {
      const state = useState(0)
      return {
        method: () => {
          return state
        }
      }
    }
    "#);
}

#[test]
fn arrow_function_hook_returning_object_with_object_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const useResult = () => {
          const state = useState(0)
          return {
            method() {
              return state
            }
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 4 }
    "#);
    assert_snapshot!(result.output, @r#"
    const useResult = () => {
      const state = useState(0)
      return {
        method: () => {
          return state
        }
      }
    }
    "#);
}

#[test]
fn hook_returning_object_with_this_usage() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function useResult() {
          const state = useState(0)
          return {
            method() {
              return this.value
            }
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'thisKeywordInMethod', line: 4 }
    "#);
}

#[test]
fn regular_function_returning_object_with_method_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        function getResult() {
          return {
            method() {
              return 42
            }
          }
        }
    "#},
    );
}

#[test]
fn pascalcase_component_returning_object_with_method_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        function MyComponent() {
          return {
            method() {
              return 42
            }
          }
        }
    "#},
    );
}

#[test]
fn hook_with_use_no_memo_returning_object_with_method_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        function useResult() {
          "use no memo"
          return {
            method() {
              return 42
            }
          }
        }
    "#},
    );
}

#[test]
fn hook_call_with_object_method_without_directive() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = useState({
              method() {
                return 42
              }
            });
        "#},
            options: Some(json!([{"runOnlyWithEnableCompilerDirective":true}])),
            filename: None,
        },
    );
}

#[test]
fn hook_call_with_object_method_with_directive() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            // eslint react-compiler/react-compiler: ["error"]
            const result = useState({
              method() {
                return 42
              }
            });
        "#},
            options: Some(json!([{"runOnlyWithEnableCompilerDirective":true}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 3 }
    "#);
    assert_snapshot!(result.output, @r#"
    // eslint react-compiler/react-compiler: ["error"]
    const result = useState({
      method: () => {
        return 42
      }
    });
    "#);
}

#[test]
fn hook_call_returning_object_with_object_method_with_directive() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            // eslint react-compiler/react-compiler: ["error"]
            function useResult() {
              "use memo"

              return {
                method() {
                  return 42
                }
              }
            }
        "#},
            options: Some(json!([{"runOnlyWithEnableCompilerDirective":true}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'objectMethodIsNotSupported', line: 6 }
    "#);
    assert_snapshot!(result.output, @r#"
    // eslint react-compiler/react-compiler: ["error"]
    function useResult() {
      "use memo"

      return {
        method: () => {
          return 42
        }
      }
    }
    "#);
}

#[test]
fn hook_call_with_object_method_using_this_with_directive() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            // eslint react-compiler/react-compiler: ["error"]
            const result = useState({
              method() {
                return this.value
              }
            });
        "#},
            options: Some(json!([{"runOnlyWithEnableCompilerDirective":true}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'thisKeywordInMethod', line: 3 }
    "#);
}

#[test]
fn fc_component_creating_jsx_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const Component: FC = () => {
          return <div>Hello</div>
        }
    "#},
    );
}

#[test]
fn react_fc_component_creating_jsx_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import React from 'react'

        const Component: React.FC = () => {
          return <div>Hello</div>
        }
    "#},
    );
}

#[test]
fn fc_component_calling_hooks_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC, useState } from 'react'

        const Component: FC = () => {
          const [count, setCount] = useState(0)
          return count.toString()
        }
    "#},
    );
}

#[test]
fn fc_component_calling_hooks_inside_namespace_is_not_valid() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { FC, useState } from 'react'

        const Component: FC = () => {
          const [count, setCount] = namespace.useTest(0)
          return count.toString()
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'fcComponentShouldReturnJsx', line: 3 }
    "#);
}

#[test]
fn fc_component_creating_jsx_in_variable_assignment_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const Component: FC = () => {
          const element = <div>Hello</div>
          return element
        }
    "#},
    );
}

#[test]
fn hook_function_creating_jsx_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        function useCustomHook() {
          return <div>Hook JSX</div>
        }
    "#},
    );
}

#[test]
fn hook_function_calling_other_hooks_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        function useCounter() {
          const [count, setCount] = useState(0)
          return { count, setCount }
        }
    "#},
    );
}

#[test]
fn fc_component_with_neither_jsx_nor_hooks_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const Component: FC = () => {
          return 'Hello World'
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'fcComponentShouldReturnJsx', line: 3 }
    "#);
}

#[test]
fn hook_function_with_neither_jsx_nor_hooks_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function useData() {
          return { data: 'some data' }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'fcComponentShouldReturnJsx', line: 1 }
    "#);
}

#[test]
fn pascalcase_function_without_fc_type_should_not_be_checked_no_jsx() {
    valid(
        RULE,
        indoc! {r#"
        const RegularFunction = () => {
          return 'This should not be checked'
        }
    "#},
    );
}

#[test]
fn pascalcase_function_with_jsx_should_be_considered_valid_component() {
    valid(
        RULE,
        indoc! {r#"
        const MyComponent = () => {
          return <div>Hello World</div>
        }
    "#},
    );
}

#[test]
fn pascalcase_function_with_jsx_and_hooks_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        const Counter = () => {
          const [count, setCount] = useState(0)
          return <div>{count}</div>
        }
    "#},
    );
}

#[test]
fn pascalcase_function_declaration_with_jsx_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        function MyComponent() {
          return <div>Hello</div>
        }
    "#},
    );
}

#[test]
fn regular_function_should_not_be_checked() {
    valid(
        RULE,
        indoc! {r#"
        const regularFunction = () => {
          return 'This is fine'
        }
    "#},
    );
}

#[test]
fn component_with_different_type_annotation_should_not_be_checked() {
    valid(
        RULE,
        indoc! {r#"
        const Component: () => string = () => {
          return 'This is fine'
        }
    "#},
    );
}

#[test]
fn fc_component_with_mixed_jsx_and_non_jsx_returns_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const isLoading = true

        const Component: FC = () => {
          return isLoading ? 'Loading...' : <div>Content</div>
        }
    "#},
    );
}

#[test]
fn hook_function_with_react_createelement_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import React from 'react'

        function useElement() {
          return React.createElement('div', null, 'Hello')
        }
    "#},
    );
}

#[test]
fn fc_component_with_createportal_containing_jsx_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'
        import { createPortal } from 'react-dom'

        const TableActionsMenuPlugin: FC<{anchorElem: HTMLElement}> = ({ anchorElem }) => {
          return createPortal(
            <div>Menu content</div>,
            anchorElem,
          )
        }
    "#},
    );
}

#[test]
fn fc_component_with_nested_function_calls_containing_jsx_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const Component: FC = () => {
          return someFunction(
            anotherFunction(<span>Nested JSX</span>)
          )
        }
    "#},
    );
}

#[test]
fn fc_component_with_use_memo_directive_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const Component: FC = () => {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
}

#[test]
fn hook_function_with_use_memo_directive_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        function useExpensiveValue() {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
}

#[test]
fn fc_component_without_jsx_hooks_should_suggest_use_memo() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const Component: FC = () => {
          return 'Hello World'
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'fcComponentShouldReturnJsx', line: 3 }
    "#);
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "addUseMemoDirective"
    );
}

#[test]
fn hook_function_without_jsx_hooks_should_suggest_use_memo() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function useData() {
          return { data: 'some data' }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'fcComponentShouldReturnJsx', line: 1 }
    "#);
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "addUseMemoDirective"
    );
}

#[test]
fn fc_component_with_comment_directive_should_still_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        // "use memo" 
        const Component: FC = () => {
          return 'Hello World'
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'fcComponentShouldReturnJsx', line: 4 }
    "#);
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "addUseMemoDirective"
    );
}

#[test]
fn react_component_calling_hooks_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { FC, useState } from 'react'

        const Component: FC = () => {
          const [count] = useState(0)
          return <div>{count}</div>
        }
    "#},
    );
}

#[test]
fn hook_function_calling_hooks_is_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        function useCounter() {
          const [count, setCount] = useState(0)
          return { count, setCount }
        }
    "#},
    );
}

#[test]
fn regular_function_not_calling_hooks_should_be_ignored() {
    valid(
        RULE,
        indoc! {r#"
        function regularFunction() {
          return 'regular function'
        }
    "#},
    );
}

#[test]
fn regular_function_calling_hooks_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        function regularFunction() {
          const [count] = useState(0)
          return count
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'functionCallingHooksMustBeComponent', line: 3 }
    "#);
}

#[test]
fn arrow_function_calling_hooks_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        const regularFunction = () => {
          const [count] = useState(0)
          return count
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'functionCallingHooksMustBeComponent', line: 3 }
    "#);
}

#[test]
fn function_calling_namespaced_hooks_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function regularFunction() {
          const data = test.useState(0)
          return data
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'functionCallingHooksMustBeComponent', line: 1 }
    "#);
}

#[test]
fn pascalcase_function_without_fc_type_calling_hooks_no_jsx_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        const Component = () => {
          const [count] = useState(0)
          return count
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'functionCallingHooksMustBeComponent', line: 3 }
    "#);
}

#[test]
fn pascalcase_function_calling_hooks_with_jsx_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        const Component = () => {
          const [count] = useState(0)
          return <div>{count}</div>
        }

        function MyComponent() {
          const [count] = useState(0)
          return <div>{count}</div>
        }
    "#},
    );
}

#[test]
fn function_expression_calling_hooks_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        const obj = {
          method: function() {
            const [count] = useState(0)
            return count
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'functionCallingHooksMustBeComponent', line: 4 }
    "#);
}

#[test]
fn nested_function_calling_hooks_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useState } from 'react'

        function parentFunction() {
          function nestedFunction() {
            const [count] = useState(0)
            return count
          }
          return nestedFunction()
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'functionCallingHooksMustBeComponent', line: 4 }
    "#);
}

#[test]
fn valid_hook_function_with_usecallback() {
    valid(
        RULE,
        indoc! {r#"
        import { useCallback } from 'react'

        function useCustomHook() {
          const callback = useCallback(() => {}, [])
          return callback
        }
    "#},
    );
}

#[test]
fn pascalcase_function_with_type_other_than_fc_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        export const TestFn: CommandProcessor = ({ ctx }) => {
          const loadedItems = testList.useListQuery({ filter: 'test' });

          return useSetLoadedCmds(ctx, {
            cmds: loadedItems.results.map((item) => {
              return {
                name: item.title,
              };
            }),
            isLoading: loadedItems.isLoading,
            error: loadedItems.error || null,
          });
        };
    "#},
    );
}

#[test]
fn pascalcase_function_with_use_memo_directive_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        const MyComponent = () => {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
}

#[test]
fn hook_function_with_use_memo_directive_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        function useCustomHook() {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
}

#[test]
fn camelcase_function_with_use_memo_directive_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const regularFunction = () => {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'useMemoDirectiveNaming', line: 1 }
    "#);
}

#[test]
fn lowercase_function_with_use_memo_directive_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function myfunction() {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'useMemoDirectiveNaming', line: 1 }
    "#);
}

#[test]
fn arrow_function_with_use_memo_directive_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const helper = () => {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'useMemoDirectiveNaming', line: 1 }
    "#);
}

#[test]
fn function_expression_with_use_memo_directive_should_show_error() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const obj = {
          helper: function() {
            "use memo"
            return calculateExpensiveValue()
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'useMemoDirectiveNaming', line: 2 }
    "#);
}

#[test]
fn pascalcase_function_declaration_with_use_memo_directive_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        function MyComponent() {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
}

#[test]
fn hook_function_starting_with_use_prefix_should_be_valid() {
    valid(
        RULE,
        indoc! {r#"
        const useCalculateValue = () => {
          "use memo"
          return calculateExpensiveValue()
        }
    "#},
    );
}

#[test]
fn function_without_use_memo_directive_should_not_be_checked() {
    valid(
        RULE,
        indoc! {r#"
        const regularFunction = () => {
          return 'no use memo directive'
        }
    "#},
    );
}

#[test]
fn hook_call_with_object_method_skipped_with_use_no_memo() {
    valid(
        RULE,
        indoc! {r#"
        function Component() {
          "use no memo";
          const result = useState({
            method() {
              return 42
            }
          });
        }
    "#},
    );
}

#[test]
fn fc_component_without_jsx_or_hooks_skipped_with_use_no_memo() {
    valid(
        RULE,
        indoc! {r#"
        const MyComponent: FC = () => {
          "use no memo";
          return calculateValue()
        }
    "#},
    );
}

#[test]
fn function_declaration_without_jsx_or_hooks_skipped_with_use_no_memo() {
    valid(
        RULE,
        indoc! {r#"
        function useMyHook() {
          "use no memo";
          return calculateValue()
        }
    "#},
    );
}

fn manual_memo_case(code: &str) -> Case<'_> {
    Case {
        code,
        options: Some(json!([{"disallowManualMemoization":true}])),
        filename: None,
    }
}

#[test]
fn manual_memoization_is_allowed_by_default() {
    valid(
        RULE,
        indoc! {r#"
        function useValue(a: number) {
          const doubled = useMemo(() => a * 2, [a])
          return useCallback(() => doubled, [doubled])
        }
    "#},
    );
}

#[test]
fn manual_memoization_is_reported_when_disallowed() {
    let result = invalid(
        RULE,
        manual_memo_case(indoc! {r#"
        function useValue(a: number) {
          const doubled = useMemo(() => a * 2, [a])
          return useCallback(() => doubled, [doubled])
        }

        const Component: FC = () => {
          const value = useMemo(() => 1, [])
          return <div onClick={useCallback(() => value, [value])} />
        }

        const fn = useMemo?.(() => 1, [])
        const parenthesized = (useMemo)(() => 1, [])
    "#}),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useMemo" is not necessary when using React Compiler.'
      line: 2
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useCallback" is not necessary when using React Compiler.'
      line: 3
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useMemo" is not necessary when using React Compiler.'
      line: 7
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useCallback" is not necessary when using React Compiler.'
      line: 8
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useMemo" is not necessary when using React Compiler.'
      line: 11
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useMemo" is not necessary when using React Compiler.'
      line: 12
    "#);
}

#[test]
fn manual_memoization_in_use_no_memo_functions_is_allowed() {
    valid(
        RULE,
        manual_memo_case(indoc! {r#"
        function useValue(a: number) {
          'use no memo'
          return useMemo(() => a * 2, [a])
        }

        const useOther = () => {
          'use no memo'
          return useCallback(() => 1, [])
        }

        function createX() {
          function useInner() {
            'use no memo'
            return useMemo(() => 1, [])
          }
          return useInner
        }

        const Input = forwardRef((props, ref) => {
          'use no memo'
          const value = useMemo(() => props.value, [props.value])
          return <input ref={ref} value={value} />
        })
    "#}),
    );
}

#[test]
fn manual_memoization_only_checks_the_nearest_function_directive() {
    let result = invalid(
        RULE,
        manual_memo_case(indoc! {r#"
        function useOuter() {
          'use no memo'
          function useInner() {
            return useMemo(() => 1, [])
          }
          const callback = () => useCallback(() => 1, [])
          return [useInner, callback]
        }

        function useDirectiveNotFirst() {
          const a = 1
          'use no memo'
          return useMemo(() => a, [a])
        }
    "#}),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useMemo" is not necessary when using React Compiler.'
      line: 4
    - messageId: 'functionCallingHooksMustBeComponent'
      msg: 'Functions calling hooks must be React components (PascalCase with FC type) or hooks (start with "use").'
      line: 6
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useCallback" is not necessary when using React Compiler.'
      line: 6
    - messageId: 'unnecessaryManualMemoization'
      msg: '"useMemo" is not necessary when using React Compiler.'
      line: 13
    "#);
}

#[test]
fn manual_memoization_ignores_other_callees() {
    valid(
        RULE,
        manual_memo_case(indoc! {r#"
        function useValue(a: number) {
          const memo = React.useMemo(() => a, [a])
          const other = useMemoized(() => a, [a])
          return [memo, other]
        }
    "#}),
    );
}
