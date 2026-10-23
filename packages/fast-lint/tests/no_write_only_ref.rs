mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-write-only-ref";

#[test]
fn ref_created_but_never_read_single_ref() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(null)
          return <div ref={ref} />
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'refNotRead'
      msg: 'Ref "ref" is never read. Consider removing it if not needed.'
      line: 4
    "#);
}

#[test]
fn ref_created_but_never_read_input_ref() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const inputRef = useRef(null)
          return <input ref={inputRef} />
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'refNotRead'
      msg: 'Ref "inputRef" is never read. Consider removing it if not needed.'
      line: 4
    "#);
}

#[test]
fn multiple_refs_only_used_as_ref_prop() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(0)
          const anotherRef = useRef(null)
          return (
            <>
              <Child ref={ref} />
              <Other ref={anotherRef} />
            </>
          )
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'refNotRead'
      msg: 'Ref "ref" is never read. Consider removing it if not needed.'
      line: 4
    - messageId: 'refNotRead'
      msg: 'Ref "anotherRef" is never read. Consider removing it if not needed.'
      line: 5
    "#);
}

#[test]
fn ref_used_in_use_effect() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(null)

          useEffect(() => {
            if (ref.current) {
              ref.current.focus()
            }
          }, [])

          return <input ref={ref} />
        }
    "#},
    );
}

#[test]
fn ref_current_modified_and_read() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const countRef = useRef(0)

          const handleClick = () => {
            countRef.current++
            console.log(countRef.current)
          }

          return <button onClick={handleClick}>Click</button>
        }
    "#},
    );
}

#[test]
fn ref_current_read_directly() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(null)
          const value = ref.current
          return <div ref={ref}>{value}</div>
        }
    "#},
    );
}

#[test]
fn ref_current_passed_to_function() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(null)
          doSomething(ref.current)
          return null
        }
    "#},
    );
}

#[test]
fn ref_destructured() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(null)
          const { current } = ref
          return <div>{current}</div>
        }
    "#},
    );
}

#[test]
fn ref_current_used_in_object_property() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(null)
          const obj = { prop: ref.current }
          return null
        }
    "#},
    );
}

#[test]
fn ref_passed_to_function() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef()
          doSomething(ref)
          return null
        }
    "#},
    );
}

#[test]
fn ref_passed_in_jsx_expression() {
    valid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref = useRef(null)
          return <Child someRef={ref} />
        }
    "#},
    );
}

#[test]
fn ref_only_used_as_ref_prop() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const myRef = useRef(null)
          const anotherRef = useRef(null)
          return (
            <>
              <div ref={myRef} />
              <span ref={anotherRef} />
            </>
          )
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'refNotRead'
      msg: 'Ref "myRef" is never read. Consider removing it if not needed.'
      line: 4
    - messageId: 'refNotRead'
      msg: 'Ref "anotherRef" is never read. Consider removing it if not needed.'
      line: 5
    "#);
}

#[test]
fn renamed_use_ref_import() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useRef as useReference } from 'react'

        function Component() {
          const ref = useReference(null)
          return <div ref={ref} />
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'refNotRead'
      msg: 'Ref "ref" is never read. Consider removing it if not needed.'
      line: 4
    "#);
}

#[test]
fn multiple_refs_with_only_one_read() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { useRef } from 'react'

        function Component() {
          const ref1 = useRef(null)
          const ref2 = useRef(null)
          const ref3 = useRef(null)

          console.log(ref2.current)

          return (
            <>
              <div ref={ref1} />
              <div ref={ref3} />
            </>
          )
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'refNotRead'
      msg: 'Ref "ref1" is never read. Consider removing it if not needed.'
      line: 4
    - messageId: 'refNotRead'
      msg: 'Ref "ref3" is never read. Consider removing it if not needed.'
      line: 6
    "#);
}
