mod common;

use common::{invalid, valid, Case, LintResult};
use indoc::indoc;
use serde_json::json;

const RULE: &str = "prefer-react-hook-alternative";

/// `(messageId, message)` of every diagnostic, in order.
fn messages(result: &LintResult) -> Vec<(&str, &str)> {
    result
        .diagnostics
        .iter()
        .map(|d| (d.message_id, d.message.as_str()))
        .collect()
}

#[test]
fn non_react_function_using_settimeout() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function regularFunction() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn class_method_using_settimeout() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                class MyClass {
                  method() {
                    setTimeout(() => {
                      console.log('hello')
                    }, 1000)
                  }
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn function_with_lowercase_name_using_settimeout() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function normalFunction() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_allowed_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  console.log('allowed')
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn custom_hook_using_allowed_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function useMyHook() {
                  console.log('allowed')
                  return true
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn nested_function_in_component_using_settimeout_is_allowed() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  function handleClick() {
                    setTimeout(() => {
                      console.log('hello')
                    }, 1000)
                  }
                  return <div onClick={handleClick}>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn function_call_inside_jsx_props_in_non_react_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function helper() {
                  return <div onClick={() => setTimeout(() => {}, 1000)}>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_settimeout_inside_allowed_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  function getCurrentPlan(plan) {
                    setTimeout(() => {
                      console.log('hello')
                    }, 1000)
                    return plan
                  }
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUseInside":["getCurrentPlan"]}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_setinterval_inside_allowed_arrow_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  const getCurrentPlan = (plan) => {
                    setInterval(() => {
                      console.log('hello')
                    }, 1000)
                    return plan
                  }
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setInterval","hookAlternative":"useInterval","allowUseInside":["getCurrentPlan"]}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_disallowed_function_inside_allowed_function_call() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  return userDocRef.value.store.useSelector((state) =>
                    getCurrentPlan(state.data?.plan),
                  )
                }

                function getCurrentPlan(plan) {
                  setTimeout(() => {
                    console.log('processing plan')
                  }, 100)
                  return plan
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUseInside":["getCurrentPlan"]}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_hook_calling_allowed_function_with_useselector() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export function useCurrentPlan(): CurrentPlan {
                  return userDocRef.value.store.useSelector((state) =>
                    getCurrentPlan(state.data?.plan),
                  );
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"getCurrentPlan","hookAlternative":"useCurrentPlanHook","allowUseInside":["useSelector"]}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_setinterval() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setInterval(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useInterval instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useInterval(() => {
                console.log('hello')
              }, 1000)
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_addeventlistener() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  window.addEventListener('click', handler)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react addEventListener should not be used directly in React components.."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              window.useEventListener('click', handler)
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn custom_hook_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function useMyHook() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return true
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function useMyHook() {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return true
            }
        "#}
    );
}

#[test]
fn arrow_function_component_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const MyComponent = () => {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            const MyComponent = () => {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn arrow_hook_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const useMyHook = () => {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return true
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            const useMyHook = () => {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return true
            }
        "#}
    );
}

#[test]
fn forwardref_callback_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const MyComponent = React.forwardRef((props, ref) => {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div ref={ref}>test</div>
                })
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            const MyComponent = React.forwardRef((props, ref) => {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div ref={ref}>test</div>
            })
        "#}
    );
}

#[test]
fn memo_callback_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const MyComponent = React.memo((props) => {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                })
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            const MyComponent = React.memo((props) => {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div>test</div>
            })
        "#}
    );
}

#[test]
fn forwardref_without_react_prefix_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const MyComponent = forwardRef((props, ref) => {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div ref={ref}>test</div>
                })
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            const MyComponent = forwardRef((props, ref) => {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div ref={ref}>test</div>
            })
        "#}
    );
}

#[test]
fn memo_without_react_prefix_using_settimeout() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const MyComponent = memo((props) => {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                })
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            const MyComponent = memo((props) => {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div>test</div>
            })
        "#}
    );
}

#[test]
fn component_with_nested_disallowed_call_in_useeffect() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  useEffect(() => {
                    setTimeout(() => {
                      console.log('hello')
                    }, 1000)
                  }, [])
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useEffect(() => {
                useTimeout(() => {
                  console.log('hello')
                }, 1000)
              }, [])
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn component_with_disallowed_function_in_jsx_prop_callback() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  return <button onClick={() => setTimeout(() => {}, 1000)}>Click</button>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              return <button onClick={() => useTimeout(() => {}, 1000)}>Click</button>
            }
        "#}
    );
}

#[test]
fn component_with_disallowed_function_in_jsx_prop_with_member_expression() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  return <div onLoad={() => window.addEventListener('resize', handler)}>Content</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react addEventListener should not be used directly in React components.."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              return <div onLoad={() => window.useEventListener('resize', handler)}>Content</div>
            }
        "#}
    );
}

#[test]
fn hook_with_disallowed_function_in_jsx_prop_callback() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function useMyHook() {
                  return <div onClick={() => setInterval(() => {}, 1000)}>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"},{"name":"setInterval","hookAlternative":"useInterval"},{"name":"addEventListener","hookAlternative":"useEventListener","message":"addEventListener should not be used directly in React components."}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useInterval instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function useMyHook() {
              return <div onClick={() => useInterval(() => {}, 1000)}>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_settimeout_with_allowedfunctions_but_not_inside_allowed_function() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_settimeout_inside_non_allowed_function() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  function someOtherFunction(plan) {
                    setTimeout(() => {
                      console.log('hello')
                    }, 1000)
                    return plan
                  }
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUseInside":["getCurrentPlan"]}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              function someOtherFunction(plan) {
                useTimeout(() => {
                  console.log('hello')
                }, 1000)
                return plan
              }
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_different_disallowed_functions_with_different_allowed_lists() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  function getCurrentPlan(plan) {
                    setTimeout(() => {
                      console.log('allowed setTimeout')
                    }, 1000)
                    return plan
                  }

                  function processData(data) {
                    setInterval(() => {
                      console.log('allowed setInterval')
                    }, 1000)
                    return data
                  }

                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUseInside":["getCurrentPlan"]},{"name":"setInterval","hookAlternative":"useInterval","allowUseInside":["processData"]}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_settimeout_in_wrong_allowed_function() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  function processData(data) {
                    setTimeout(() => {
                      console.log('not allowed here')
                    }, 1000)
                    return data
                  }

                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUseInside":["getCurrentPlan"]},{"name":"setInterval","hookAlternative":"useInterval","allowUseInside":["processData"]}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              function processData(data) {
                useTimeout(() => {
                  console.log('not allowed here')
                }, 1000)
                return data
              }

              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_settimeout_with_arguments_when_allowusingwithargs_is_true() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":true}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_addeventlistener_with_arguments_when_allowusingwithargs_is_true() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  window.addEventListener('click', handler)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"addEventListener","hookAlternative":"useEventListener","allowUsingWithArgs":true}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn hook_using_settimeout_with_arguments_when_allowusingwithargs_is_true() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function useMyHook() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return true
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":true}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_settimeout_with_undefined_argument_when_allowusingwithargs_is_true() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout(undefined)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":true}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useTimeout(undefined)
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_settimeout_with_multiple_undefined_arguments_when_allowusingwithargs() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout(undefined, undefined)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":true}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useTimeout(undefined, undefined)
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_settimeout_with_one_meaningful_and_one_undefined_argument_when_allow() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout(() => console.log('hello'), undefined)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":true}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn react_component_using_settimeout_without_arguments_when_allowusingwithargs_is_true() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout()
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":true}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useTimeout()
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_settimeout_without_arguments_when_allowusingwithargs_is_false() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout()
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":false}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useTimeout()
              return <div>test</div>
            }
        "#}
    );
}

#[test]
fn react_component_using_settimeout_with_arguments_when_allowusingwithargs_is_false() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function MyComponent() {
                  setTimeout(() => {
                    console.log('hello')
                  }, 1000)
                  return <div>test</div>
                }
            "#},
            options: Some(
                json!([{"disallowedFunctions":[{"name":"setTimeout","hookAlternative":"useTimeout","allowUsingWithArgs":false}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_eq!(
        messages(&result),
        vec![(
            "preferHookAlternative",
            r#"This function should not be used in react use useTimeout instead."#
        )]
    );
    assert_eq!(result.diagnostics[0].suggestions.len(), 1);
    assert_eq!(
        result.diagnostics[0].suggestions[0].message_id,
        "preferHookAlternative"
    );
    assert_eq!(
        result.suggestion_output_of(&result.diagnostics[0], 0),
        indoc! {r#"
            function MyComponent() {
              useTimeout(() => {
                console.log('hello')
              }, 1000)
              return <div>test</div>
            }
        "#}
    );
}
