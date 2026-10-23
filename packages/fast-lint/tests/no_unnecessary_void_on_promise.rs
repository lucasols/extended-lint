mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-unnecessary-void-on-promise";

#[test]
fn void_on_async_function_call() {
    valid(
        RULE,
        indoc! {"
            async function fetchData() {
              return await fetch('/api/data')
            }

            void fetchData()
        "},
    );
}

#[test]
fn void_on_promise_returning_function() {
    valid(
        RULE,
        indoc! {"
            function getPromise(): Promise<void> {
              return Promise.resolve()
            }

            void getPromise()
        "},
    );
}

#[test]
fn void_on_fetch_call() {
    valid(RULE, "void fetch('/api/data')");
}

#[test]
fn void_on_promise_resolve() {
    valid(RULE, "void Promise.resolve()");
}

#[test]
fn void_on_method_returning_promise() {
    valid(
        RULE,
        indoc! {"
            const api = {
              fetchData(): Promise<string> {
                return Promise.resolve('data')
              }
            }

            void api.fetchData()
        "},
    );
}

#[test]
fn void_on_synchronous_function_call() {
    let result = invalid(
        RULE,
        indoc! {"
            function syncFn() {
              return 'hello'
            }

            void syncFn()
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryVoid', line: 5 }");
    assert_snapshot!(result.suggestion_output(0), @r"
    function syncFn() {
      return 'hello'
    }

    syncFn()
    ");
}

#[test]
fn void_on_console_log() {
    let result = invalid(RULE, "void console.log('test')");

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryVoid', line: 1 }");
    assert_snapshot!(result.suggestion_output(0), @"console.log('test')");
}

#[test]
fn void_on_math_max() {
    let result = invalid(RULE, "void Math.max(1, 2, 3)");

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryVoid', line: 1 }");
    assert_snapshot!(result.suggestion_output(0), @"Math.max(1, 2, 3)");
}

#[test]
fn void_on_function_returning_number() {
    let result = invalid(
        RULE,
        indoc! {"
            function getNumber(): number {
              return 42
            }

            void getNumber()
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryVoid', line: 5 }");
    assert_snapshot!(result.suggestion_output(0), @r"
    function getNumber(): number {
      return 42
    }

    getNumber()
    ");
}

#[test]
fn void_on_arrow_function_call() {
    let result = invalid(
        RULE,
        indoc! {"
            const greet = () => 'hello'

            void greet()
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryVoid', line: 3 }");
    assert_snapshot!(result.suggestion_output(0), @r"
    const greet = () => 'hello'

    greet()
    ");
}

#[test]
fn void_on_parenthesized_call() {
    let result = invalid(RULE, "void (Math.max(1, 2))");

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryVoid', line: 1 }");
    assert_snapshot!(result.suggestion_output(0), @"Math.max(1, 2)");
}

#[test]
fn void_on_variable_is_ignored() {
    valid(
        RULE,
        indoc! {"
            const value = 123
            void value
        "},
    );
}

#[test]
fn void_on_literal_is_ignored() {
    valid(RULE, "void 0");
}

#[test]
fn void_on_object_property_access_is_ignored() {
    valid(
        RULE,
        indoc! {"
            const obj = { prop: 123 }
            void obj.prop
        "},
    );
}

#[test]
fn void_on_optional_call_is_ignored() {
    valid(
        RULE,
        indoc! {"
            const obj: { fn?: () => number } = {}
            void obj.fn?.()
        "},
    );
}

#[test]
fn void_with_promise_all() {
    valid(
        RULE,
        indoc! {"
            async function fn1() {}
            async function fn2() {}

            void Promise.all([fn1(), fn2()])
        "},
    );
}

#[test]
fn void_with_chained_then() {
    valid(
        RULE,
        indoc! {"
            function getPromise(): Promise<number> {
              return Promise.resolve(42)
            }

            void getPromise().then(x => x + 1)
        "},
    );
}

#[test]
fn void_on_call_returning_void_or_promise() {
    valid(
        RULE,
        indoc! {"
            declare const onSubmit: () => void | Promise<void>

            void onSubmit()
        "},
    );
}

#[test]
fn remove_void_suggestion_wraps_a_function_expression_statement() {
    let result = invalid(
        RULE,
        indoc! {"
            declare function f(): void

            f()
            void (function once() {}())
        "},
    );

    assert_snapshot!(result.suggestion_output(0), @r###"
    declare function f(): void

    f()
    ;(function once() {}())
    "###);
}

#[test]
fn remove_void_suggestion_keeps_the_statement_apart_from_the_previous_line() {
    let result = invalid(
        RULE,
        indoc! {"
            declare function f(): void
            declare const g: (() => void) | undefined

            f()
            void (g || f)()
        "},
    );

    assert_snapshot!(result.suggestion_output(0), @r###"
    declare function f(): void
    declare const g: (() => void) | undefined

    f()
    ;(g || f)()
    "###);
}

#[test]
fn void_on_call_returning_promise_constrained_type_parameter() {
    valid(
        RULE,
        indoc! {"
            function run<T extends Promise<void> | void>(callback: () => T) {
              void callback()
            }
        "},
    );
}

#[test]
fn remove_void_suggestion_in_braceless_body_and_export_default() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const cond: boolean
            declare const h: (() => void) | undefined
            declare function g(): void

            if (cond) void (h ?? g)()
        "},
    );

    assert_snapshot!(result.suggestion_output(0), @r###"
    declare const cond: boolean
    declare const h: (() => void) | undefined
    declare function g(): void

    if (cond) (h ?? g)()
    "###);

    let result = invalid(RULE, "export default void function () {}()");

    assert_snapshot!(result.suggestion_output(0), @"export default (function () {}())");
}
