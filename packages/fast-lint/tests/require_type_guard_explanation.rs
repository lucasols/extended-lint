mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "require-type-guard-explanation";

#[test]
fn function_declaration_without_comment() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used.'
      line: 3
    "#);
}

#[test]
fn function_declaration_with_explanation() {
    valid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: values come from our serializer, which always sets `kind`
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
}

#[test]
fn exported_function_with_explanation_above_export() {
    valid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: values come from our serializer, which always sets `kind`
        export function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
}

#[test]
fn explanation_too_short() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: it is safe
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 4 }");
}

#[test]
fn comment_without_prefix() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // values come from our serializer, which always sets `kind`
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 4 }");
}

#[test]
fn prefix_with_empty_text() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety:
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 4 }");
}

#[test]
fn consecutive_line_comments_form_one_block() {
    valid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: values come
        // from our serializer, which always
        // sets `kind`
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
}

#[test]
fn prefix_on_a_later_line_of_the_block() {
    valid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // Checks whether a value is a node.
        // type-guard-safety: values come from our serializer, which always sets `kind`
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
}

#[test]
fn blank_line_splits_blocks() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: short

        // values come from our serializer, which always sets `kind`
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 6 }");
}

#[test]
fn jsdoc_explanation() {
    valid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        /**
         * Checks whether a value is a node.
         *
         * type-guard-safety: values come from our serializer, which always
         * sets `kind`
         */
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
}

#[test]
fn single_line_block_comment_explanation() {
    valid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        /* type-guard-safety: values come from our serializer, which always sets `kind` */
        function isNode(value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
}

#[test]
fn const_arrow_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: values come from our serializer, which always sets `kind`
        export const isNode = (value: unknown): value is Node =>
          typeof value === 'object' && value !== null && 'kind' in value

        const isOtherNode = (value: unknown): value is Node =>
          typeof value === 'object' && value !== null && 'kind' in value
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 7 }");
}

#[test]
fn function_expression() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        const isNode = function (value: unknown): value is Node {
          return typeof value === 'object' && value !== null && 'kind' in value
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 3 }");
}

#[test]
fn object_literal_members() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: this comment is about the object, not its members
        const guards = {
          // type-guard-safety: values come from our serializer, which always sets `kind`
          isNode(value: unknown): value is Node {
            return typeof value === 'object' && value !== null && 'kind' in value
          },
          isOtherNode: (value: unknown): value is Node =>
            typeof value === 'object' && value !== null && 'kind' in value,
          isThirdNode(value: unknown): value is Node {
            return typeof value === 'object' && value !== null && 'kind' in value
          },
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'missingExplanation', line: 9 }
    - { messageId: 'missingExplanation', line: 11 }
    "#);
}

#[test]
fn class_members() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        // type-guard-safety: this comment is about the class, not its members
        class Guards {
          // type-guard-safety: values come from our serializer, which always sets `kind`
          isNode(value: unknown): value is Node {
            return typeof value === 'object' && value !== null && 'kind' in value
          }

          static isOtherNode(value: unknown): value is Node {
            return typeof value === 'object' && value !== null && 'kind' in value
          }

          isLoaded(): this is { data: string } {
            return true
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'missingExplanation', line: 10 }
    - { messageId: 'missingExplanation', line: 14 }
    "#);
}

#[test]
fn assertion_functions() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        function assertIsNode(value: unknown): asserts value is Node {
          if (typeof value !== 'object' || value === null) throw new Error('not a node')
        }

        function assertTruthy(value: unknown): asserts value {
          if (!value) throw new Error('falsy')
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 3 }");
}

#[test]
fn inline_callbacks() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }
        declare function looksLikeNode(value: unknown): boolean
        declare const items: unknown[]

        // type-guard-safety: items only ever hold nodes or strings, see loadItems
        const nodes = items.filter((item): item is Node => looksLikeNode(item))

        function getNodes() {
          // type-guard-safety: items only ever hold nodes or strings, see loadItems
          const explained = items.filter((item): item is Node => looksLikeNode(item))
          return [
            ...explained,
            ...items.filter((item): item is Node => looksLikeNode(item)),
          ]
        }
    "#},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingExplanation', line: 13 }");
}

#[test]
fn signatures_without_body_are_ignored() {
    valid(
        RULE,
        indoc! {r#"
        type Node = { kind: string }

        declare function isNode(value: unknown): value is Node

        function isThing(value: unknown): value is Node
        function isThing(value: unknown, strict: boolean): value is Node
        // type-guard-safety: the implementation signature itself has no predicate
        function isThing(value: unknown): boolean {
          return value !== null
        }

        interface Guards {
          isNode(value: unknown): value is Node
          isOther: (value: unknown) => value is Node
        }

        type Guard = (value: unknown) => value is Node

        abstract class Base {
          abstract isNode(value: unknown): value is Node
        }
    "#},
    );
}

#[test]
fn custom_prefix_and_min_length() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            type Node = { kind: string }

            // SAFETY: trusted
            function isNode(value: unknown): value is Node {
              return typeof value === 'object' && value !== null && 'kind' in value
            }

            // SAFETY: ok
            function isOtherNode(value: unknown): value is Node {
              return typeof value === 'object' && value !== null && 'kind' in value
            }
        "#},
            options: Some(json!([{ "commentPrefix": "SAFETY:", "minLength": 5 }])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// SAFETY: ..." comment (at least 5 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used.'
      line: 9
    "#);
}

#[test]
fn unknown_option_is_rejected() {
    let result = std::panic::catch_unwind(|| {
        valid(
            RULE,
            Case {
                code: "const a = 1",
                options: Some(json!([{ "prefix": "x" }])),
                filename: None,
            },
        );
    });
    assert!(result.is_err());
}

#[test]
fn inferable_filter_callback() {
    let result = invalid(
        RULE,
        indoc! {r#"
        declare const items: (string | null)[]

        // type-guard-safety: an explanation does not help when TS infers the predicate
        const strings = items.filter((item): item is string => item !== null)
    "#},
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'inferablePredicate'
      msg: 'TypeScript infers this type predicate, remove the return type annotation.'
      line: 4
    "#);
    assert_snapshot!(result.suggestion_output(0), @r#"
    declare const items: (string | null)[]

    // type-guard-safety: an explanation does not help when TS infers the predicate
    const strings = items.filter((item) => item !== null)
    "#);
}

#[test]
fn inferable_null_and_undefined_checks() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function isDefined(value: string | undefined): value is string {
          return value !== undefined
        }

        function isNotNullish(value: string | null | undefined): value is string {
          return value != null
        }

        function isPresent(value: number | null | undefined): value is number {
          return value !== null && undefined !== value
        }

        const isNotNull = (value: { id: string } | null): value is { id: string } => null !== value

        function second(first: string | null, value: boolean | null): value is boolean {
          return value !== null
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'inferablePredicate', line: 1 }
    - { messageId: 'inferablePredicate', line: 5 }
    - { messageId: 'inferablePredicate', line: 9 }
    - { messageId: 'inferablePredicate', line: 13 }
    - { messageId: 'inferablePredicate', line: 15 }
    "#);
}

#[test]
fn inferable_typeof_checks() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function isString(value: unknown): value is string {
          return typeof value === 'string'
        }

        function isNumber(value: string | number): value is number {
          return typeof value === 'number'
        }

        function isNotString(value: string | number | null): value is number | null {
          return typeof value !== 'string'
        }

        const isLiteral = (value: 'a' | 'b' | 1 | { id: string }): value is 'a' | 'b' =>
          typeof value === 'string'
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'inferablePredicate', line: 1 }
    - { messageId: 'inferablePredicate', line: 5 }
    - { messageId: 'inferablePredicate', line: 9 }
    - { messageId: 'inferablePredicate', line: 13 }
    "#);
}

#[test]
fn not_reported_as_inferable() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function narrower(value: string | null): value is 'a' {
          return value !== null
        }

        function generic<T>(value: T | null): value is T {
          return value !== null
        }

        function maybeString(value: { length: number } | number): value is string {
          return typeof value === 'string'
        }

        function emptyObject(value: {} | number): value is string {
          return typeof value === 'string'
        }

        function twoStatements(value: string | null): value is string {
          const result = value !== null
          return result
        }

        function truthy(value: string | null): value is string {
          return !!value
        }

        function unknownNotNull(value: unknown): value is {} {
          return value !== null
        }

        function anyTypeof(value: any): value is string {
          return typeof value === 'string'
        }

        function voidUnion(value: string | void): value is string {
          return value !== undefined
        }

        function nothingRemoved(value: string): value is string {
          return value !== null
        }

        function otherParameter(value: string | null, other: string | null): value is string {
          return other !== null
        }

        function typeofObject(value: string | { id: string }): value is { id: string } {
          return typeof value === 'object'
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'missingExplanation', line: 1 }
    - { messageId: 'missingExplanation', line: 5 }
    - { messageId: 'missingExplanation', line: 9 }
    - { messageId: 'missingExplanation', line: 13 }
    - { messageId: 'missingExplanation', line: 17 }
    - { messageId: 'missingExplanation', line: 22 }
    - { messageId: 'missingExplanation', line: 26 }
    - { messageId: 'missingExplanation', line: 30 }
    - { messageId: 'missingExplanation', line: 34 }
    - { messageId: 'missingExplanation', line: 38 }
    - { messageId: 'missingExplanation', line: 42 }
    - { messageId: 'missingExplanation', line: 46 }
    "#);
}

#[test]
fn inferable_block_body_and_method() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const guards = {
          isDefined(value: string | undefined): value is string {
            return value !== undefined
          },
        }

        class Guards {
          isDefined(value: string | undefined): value is string {
            return value !== undefined
          }
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'inferablePredicate', line: 2 }
    - { messageId: 'inferablePredicate', line: 8 }
    "#);
    assert_snapshot!(result.suggestion_output(0), @r#"
    const guards = {
      isDefined(value: string | undefined) {
        return value !== undefined
      },
    }

    class Guards {
      isDefined(value: string | undefined): value is string {
        return value !== undefined
      }
    }
    "#);
}

#[test]
fn inferable_positive_null_checks() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function isNullish(value: unknown): value is null | undefined {
          return value === null || value === undefined
        }

        const isLooseNullish = (value: unknown): value is null | undefined => value == null

        const isNull = (value: string | null): value is null => value === null

        const isUndefined = (value: unknown): value is undefined => undefined === value

        function isMissing(value: string | null | undefined): value is null | undefined {
          return value === undefined || value === null
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'inferablePredicate', line: 1 }
    - { messageId: 'inferablePredicate', line: 5 }
    - { messageId: 'inferablePredicate', line: 7 }
    - { messageId: 'inferablePredicate', line: 9 }
    - { messageId: 'inferablePredicate', line: 11 }
    "#);
}

#[test]
fn mixed_null_checks_not_reported_as_inferable() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function either(value: string | null | undefined): value is string {
          return value !== null || value !== undefined
        }

        function both(value: string | null | undefined): value is null {
          return value === null && value === undefined
        }

        function emptyOrNullish(value: unknown): value is null | '' | undefined {
          return value === null || value === '' || value === undefined
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"
    - { messageId: 'missingExplanation', line: 1 }
    - { messageId: 'missingExplanation', line: 5 }
    - { messageId: 'missingExplanation', line: 9 }
    "#);
}

fn alternative_msgs_case(code: &str) -> Case<'_> {
    Case {
        code,
        options: Some(json!([{
            "alternativeMsgs": {
                "inArrayFind": "Use findAndMap instead.",
                "inArrayFilter": "Use filterAndMap instead."
            }
        }])),
        filename: None,
    }
}

#[test]
fn alternative_msgs_are_appended_in_filter_and_find_callbacks() {
    let result = invalid(
        RULE,
        alternative_msgs_case(indoc! {r#"
        type Node = { kind: string }
        declare const items: unknown[]
        declare function isObject(value: unknown): value is object

        items.filter((item): item is Node => isObject(item) && 'kind' in item)
        items.find((item): item is Node => isObject(item) && 'kind' in item)
        items?.filter(((item): item is Node => isObject(item) && 'kind' in item))
        items['find'](function (item): item is Node {
          return isObject(item) && 'kind' in item
        })
    "#}),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used. Use filterAndMap instead.'
      line: 5
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used. Use findAndMap instead.'
      line: 6
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used. Use filterAndMap instead.'
      line: 7
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used. Use findAndMap instead.'
      line: 8
    "#);
}

#[test]
fn alternative_msgs_only_apply_to_direct_callbacks() {
    let result = invalid(
        RULE,
        alternative_msgs_case(indoc! {r#"
        type Node = { kind: string }
        declare const items: unknown[]
        declare function isObject(value: unknown): value is object

        function isNode(value: unknown): value is Node {
          return isObject(value) && 'kind' in value
        }

        items.filter((item) => {
          const check = (value: unknown): value is Node => isObject(value) && 'kind' in value
          return check(item)
        })
        items.map((item): item is Node => isObject(item) && 'kind' in item)
    "#}),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used.'
      line: 5
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used.'
      line: 10
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used.'
      line: 13
    "#);
}

#[test]
fn alternative_msgs_with_explanation_are_valid() {
    valid(
        RULE,
        alternative_msgs_case(indoc! {r#"
        type Node = { kind: string }
        declare const items: unknown[]
        declare function isObject(value: unknown): value is object

        items.filter(
          // type-guard-safety: items come from our serializer, which always sets `kind`
          (item): item is Node => isObject(item) && 'kind' in item,
        )
    "#}),
    );
}

#[test]
fn alternative_msgs_are_not_added_to_inferable_predicates() {
    let result = invalid(
        RULE,
        alternative_msgs_case(indoc! {r#"
        declare const items: (string | null)[]

        items.filter((item): item is string => item !== null)
    "#}),
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'inferablePredicate'
      msg: 'TypeScript infers this type predicate, remove the return type annotation.'
      line: 3
    "#);
}

#[test]
fn empty_alternative_msg_is_ignored() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            type Node = { kind: string }
            declare const items: unknown[]
            declare function isObject(value: unknown): value is object

            items.filter((item): item is Node => isObject(item) && 'kind' in item)
        "#},
            options: Some(json!([{ "alternativeMsgs": { "inArrayFilter": "" } }])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'missingExplanation'
      msg: 'Type guards are not checked by TypeScript. Add a "// type-guard-safety: ..." comment (at least 20 characters) explaining why this guard is safe and why TS inference or an alternative (narrowing with typeof/in/instanceof, a discriminated union, a schema parser) can't be used.'
      line: 5
    "#);
}

fn all_hints_case(code: &str) -> Case<'_> {
    Case {
        code,
        options: Some(json!([{
            "alternativeMsgs": {
                "default": "DEFAULT hint.",
                "unknownInput": "UNKNOWN hint.",
                "inArrayFind": "FIND hint.",
                "inArrayFilter": "FILTER hint."
            }
        }])),
        filename: None,
    }
}

fn hints(result: &common::LintResult) -> String {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let message = diagnostic.message.as_str();
            let hint = message.rsplit(". ").next().unwrap_or(message);
            format!("{}: {hint}\n", diagnostic.line)
        })
        .collect()
}

#[test]
fn most_specific_hint_wins() {
    let result = invalid(
        RULE,
        all_hints_case(indoc! {r#"
        type Node = { kind: string }
        declare const items: unknown[]
        declare const nodes: (Node | string)[]
        declare function isObject(value: unknown): value is object

        function isNode(value: unknown): value is Node {
          return isObject(value) && 'kind' in value
        }
        function isNodeOrString(value: Node | string | number): value is Node {
          return isObject(value)
        }
        class Wrapper {
          isWrapped(): this is { inner: Node } {
            return true
          }
        }
        items.filter((item): item is Node => isObject(item) && 'kind' in item)
        nodes.find((item): item is Node => isObject(item))
        const check = (value: unknown): asserts value is Node => {
          if (!isObject(value)) throw new Error()
        }
    "#}),
    );
    assert_snapshot!(hints(&result), @r#"
    6: UNKNOWN hint.
    9: DEFAULT hint.
    13: DEFAULT hint.
    17: FILTER hint.
    18: FIND hint.
    19: UNKNOWN hint.
    "#);
}

#[test]
fn unknown_input_falls_back_to_default() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            type Node = { kind: string }
            declare function isObject(value: unknown): value is object

            function isNode(value: unknown): value is Node {
              return isObject(value) && 'kind' in value
            }
        "#},
            options: Some(json!([{ "alternativeMsgs": { "default": "DEFAULT hint." } }])),
            filename: None,
        },
    );
    assert_snapshot!(hints(&result), @"4: DEFAULT hint.");
}

#[test]
fn any_input_is_not_unknown_input() {
    let result = invalid(
        RULE,
        all_hints_case(indoc! {r#"
        type Node = { kind: string }

        function isNode(value: any): value is Node {
          return typeof value === 'object' && 'kind' in value
        }
    "#}),
    );
    assert_snapshot!(hints(&result), @"3: DEFAULT hint.");
}
