mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "strict-casting";

fn to_number_cast_option() -> Option<serde_json::Value> {
    Some(json!({
        "additionalCastFunctions": [{ "name": "toNumber", "expectedType": "number" }]
    }))
}

#[test]
fn valid_casts_between_different_types() {
    valid(
        RULE,
        indoc! {"
            declare const num: number
            declare const bool: boolean
            declare const str: string

            const a = String(num)
            const b = String(bool)
            const c = Number(str)
            const d = Number(bool)
        "},
    );
}

#[test]
fn number_on_object_with_numeric_value_of_is_valid() {
    valid(
        RULE,
        indoc! {"
            declare const date: Date

            const timestamp = Number(date)
        "},
    );
}

#[test]
fn any_and_unknown_casts() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const unknownValue: unknown
            declare function getAny(): any

            const a = String(unknownValue)
            const b = Number(getAny())
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unknownCasting', line: 4 }
    - { messageId: 'anyCasting', line: 5 }
    ");
}

#[test]
fn any_and_unknown_casts_are_allowed_with_options() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                declare const unknownValue: unknown
                declare function getAny(): any

                const a = String(unknownValue)
                const b = Number(getAny())
            "},
            options: Some(json!({ "allowAnyCasting": true, "allowUnknownCasting": true })),
            filename: None,
        },
    );
}

#[test]
fn any_and_unknown_checks_do_not_apply_to_custom_cast_functions() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                declare function toNumber(val: string | number): number

                declare const anyValue: any

                const result = toNumber(anyValue)
            "},
            options: to_number_cast_option(),
            filename: None,
        },
    );
}

#[test]
fn calls_with_no_args_or_multiple_args_are_skipped() {
    valid(
        RULE,
        indoc! {"
            const a = String()
            const b = Number()
        "},
    );
}

#[test]
fn nullish_casting_is_allowed_with_allow_nullish_casting() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                declare const value: string | null

                const a = String(value)
                const b = Number(value)
            "},
            options: Some(json!({ "allowNullishCasting": true })),
            filename: None,
        },
    );
}

#[test]
fn invalid_casting_is_allowed_with_allow_invalid_casting() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                declare const obj: { a: string }
                declare const sym: symbol

                const a = Number(obj)
                const b = Number(sym)
            "},
            options: Some(json!({ "allowInvalidCasting": true })),
            filename: None,
        },
    );
}

#[test]
fn custom_cast_function_with_different_type_is_valid() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                declare function toNumber(val: string | number): number

                declare const str: string

                const result = toNumber(str)
            "},
            options: to_number_cast_option(),
            filename: None,
        },
    );
}

#[test]
fn unnecessary_string_cast_on_string() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const str: string

            const a = String(str)
            const b = String('literal')
            const c = String(`template ${str}`)
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unnecessaryCasting', line: 3 }
    - { messageId: 'unnecessaryCasting', line: 4 }
    - { messageId: 'unnecessaryCasting', line: 5 }
    ");
    assert_snapshot!(result.output, @r"
    declare const str: string

    const a = str
    const b = 'literal'
    const c = `template ${str}`
    ");
}

#[test]
fn unnecessary_number_cast_on_number_expression() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const x: number
            declare const y: number

            const num = Number(x + y)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryCasting', line: 4 }");
    assert_snapshot!(result.output, @r"
    declare const x: number
    declare const y: number

    const num = x + y
    ");
}

#[test]
fn unnecessary_cast_on_generic_with_string_constraint() {
    let result = invalid(
        RULE,
        indoc! {"
            function toStr<T extends string>(value: T): string {
              return String(value)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryCasting', line: 2 }");
    assert_snapshot!(result.output, @r"
    function toStr<T extends string>(value: T): string {
      return String(value)
    }
    ");
}

#[test]
fn unnecessary_cast_of_enum_or_type_parameter_is_not_autofixed() {
    let result = invalid(
        RULE,
        indoc! {"
            enum SE { A = 'a', B = 'b' }
            enum NE { A, B }
            declare const member: SE

            let a = String(SE.A)
            const b = String(member)
            const c = Number(NE.B)
            const d = <T extends number>(x: T) => Number(x)
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unnecessaryCasting', line: 5 }
    - { messageId: 'unnecessaryCasting', line: 6 }
    - { messageId: 'unnecessaryCasting', line: 7 }
    - { messageId: 'unnecessaryCasting', line: 8 }
    ");
    assert_eq!(result.output, result.code);
}

#[test]
fn unnecessary_cast_of_literal_union_is_autofixed() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const status: 'a' | 'b'

            const e = String(status)
        "},
    );

    assert_snapshot!(result.output, @r"
    declare const status: 'a' | 'b'

    const e = status
    ");
}

#[test]
fn unnecessary_cast_fix_adds_parens_when_needed() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const cond: boolean
            declare const a: string
            declare const b: string

            const len = String(cond ? a : b).length
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryCasting', line: 5 }");
    assert_snapshot!(result.output, @r"
    declare const cond: boolean
    declare const a: string
    declare const b: string

    const len = (cond ? a : b).length
    ");
}

#[test]
fn nullish_string_cast() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const value: string | null

            const str = String(value)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'nullishStringCasting', line: 3 }");
    assert_snapshot!(result.suggestion_output(0), @r"
    declare const value: string | null

    const str = String(value ?? '')
    ");
}

#[test]
fn nullish_string_cast_on_optional_value() {
    let result = invalid(
        RULE,
        indoc! {"
            function format(value?: number) {
              return String(value)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'nullishStringCasting', line: 2 }");
    assert_snapshot!(result.suggestion_output(0), @r"
    function format(value?: number) {
      return String(value ?? '')
    }
    ");
}

#[test]
fn nullish_string_cast_suggestion_adds_parens_when_needed() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const a: string | null
            declare const b: string | null

            const str = String(a || b)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'nullishStringCasting', line: 4 }");
    assert_snapshot!(result.suggestion_output(0), @r"
    declare const a: string | null
    declare const b: string | null

    const str = String((a || b) ?? '')
    ");
}

#[test]
fn cast_of_pure_null_literal() {
    let result = invalid(RULE, "const str = String(null)");

    assert_snapshot!(result.errors(), @"- { messageId: 'nullishStringCasting', line: 1 }");
}

#[test]
fn nullish_number_cast() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const value: number | undefined

            const num = Number(value)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'nullishNumberCasting', line: 3 }");
    assert_snapshot!(result.suggestion_output(0), @r"
    declare const value: number | undefined

    const num = Number(value ?? 0)
    ");
}

#[test]
fn number_cast_of_symbol() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const sym: symbol

            const num = Number(sym)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'symbolNumberCasting', line: 3 }");
}

#[test]
fn number_cast_of_objects_without_numeric_value_of() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const obj: { a: string }
            declare const arr: number[]
            declare const fn: () => void

            const a = Number(obj)
            const b = Number(arr)
            const c = Number(fn)
        "},
    );

    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'nanNumberCasting'
      msg: 'Number() cast of a value of type "{ a: string; }" will result in NaN'
      line: 5
    - messageId: 'nanNumberCasting'
      msg: 'Number() cast of a value of type "number[]" will result in NaN'
      line: 6
    - messageId: 'nanNumberCasting'
      msg: 'Number() cast of a value of type "() => void" will result in NaN'
      line: 7
    "#);
}

#[test]
fn unnecessary_custom_cast_function() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                declare function toNumber(val: string | number): number

                declare const num: number

                const result = toNumber(num)
            "},
            options: to_number_cast_option(),
            filename: None,
        },
    );

    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'unnecessaryCasting'
      msg: 'Unnecessary toNumber() cast of a value that is already of type number'
      line: 5
    ");
    assert_snapshot!(result.output, @r"
    declare function toNumber(val: string | number): number

    declare const num: number

    const result = num
    ");
}

#[test]
fn nullish_check_does_not_apply_to_custom_cast_functions() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                declare function toNumber(val: string | number | null): number

                declare const value: string | null

                const result = toNumber(value)
            "},
            options: to_number_cast_option(),
            filename: None,
        },
    );
}

#[test]
fn nullish_string_cast_with_object_in_union() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const value: { a: string } | undefined

            const str = String(value)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'nullishStringCasting', line: 3 }");
}

#[test]
fn parenthesized_arg_and_callee_are_looked_through() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const a: string | null
            declare const b: string | null
            declare const str: string

            const x = (String)((str))
            const y = String((a || b))
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unnecessaryCasting', line: 5 }
    - { messageId: 'nullishStringCasting', line: 6 }
    ");
    assert_snapshot!(result.output, @r"
    declare const a: string | null
    declare const b: string | null
    declare const str: string

    const x = str
    const y = String((a || b))
    ");
    assert_snapshot!(result.suggestion_output_of(&result.diagnostics[1], 0), @r"
    declare const a: string | null
    declare const b: string | null
    declare const str: string

    const x = (String)((str))
    const y = String(((a || b) ?? ''))
    ");
}

#[test]
fn parenthesized_cast_parent_is_looked_through() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const x: number
            declare const y: number

            const num = (Number(x + y))
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryCasting', line: 4 }");
    assert_snapshot!(result.output, @r"
    declare const x: number
    declare const y: number

    const num = (x + y)
    ");
}

#[test]
fn optional_chain_arg_is_not_a_simple_arg() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const obj: { value: string } | undefined
            declare const str: { value: string }

            const a = String(obj?.value)
            const b = String(str.value)
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'nullishStringCasting', line: 4 }
    - { messageId: 'unnecessaryCasting', line: 5 }
    ");
    assert_snapshot!(result.suggestion_output(0), @r"
    declare const obj: { value: string } | undefined
    declare const str: { value: string }

    const a = String((obj?.value) ?? '')
    const b = String(str.value)
    ");
}

#[test]
fn optional_call_of_cast_function_fix_keeps_parens() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const x: number
            declare const y: number

            const num = Number?.(x + y)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryCasting', line: 4 }");
    assert_snapshot!(result.output, @r"
    declare const x: number
    declare const y: number

    const num = (x + y)
    ");
}

#[test]
fn unnecessary_cast_fix_parens_by_parent() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const x: number
            declare const y: number
            declare function fn(value: number): void
            let target = 0

            target = Number(x + y)
            fn(Number(x + y))
            const obj = { a: Number(x + y), [Number(x + y)]: 1 }
            const arr = [Number(x + y)]
            const arrow = () => Number(x + y)
            const neg = -Number(x + y)
        "},
    );

    assert_snapshot!(result.output, @r"
    declare const x: number
    declare const y: number
    declare function fn(value: number): void
    let target = 0

    target = x + y
    fn(x + y)
    const obj = { a: x + y, [x + y]: 1 }
    const arr = [x + y]
    const arrow = () => x + y
    const neg = -(x + y)
    ");
}

#[test]
fn unnecessary_cast_fix_keeps_statements_apart() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const a: string
            declare const b: string
            declare const n: number
            declare function sideEffect(): void

            sideEffect()
            String(`${a}`).split(',')
            sideEffect()
            String(a + b).length
            sideEffect()
            Number(-n).toFixed()
            sideEffect();
            String(a + b).length
            const f = () => String({} as unknown as string)
        "},
    );

    assert_snapshot!(result.output, @r###"
    declare const a: string
    declare const b: string
    declare const n: number
    declare function sideEffect(): void

    sideEffect()
    ;`${a}`.split(',')
    sideEffect()
    ;(a + b).length
    sideEffect()
    ;(-n).toFixed()
    sideEffect();
    (a + b).length
    const f = () => ({} as unknown as string)
    "###);
}

#[test]
fn unnecessary_number_literal_cast_fix_keeps_member_access_valid() {
    let result = invalid(RULE, "const s = Number(1).toString()");

    assert_snapshot!(result.output, @"const s = (1).toString()");
}

#[test]
fn unnecessary_cast_with_comments_is_not_autofixed() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const s: string

            const a = String(/* keep */ s)
            const b = String(s /* keep */)
        "},
    );

    assert_snapshot!(result.errors(), @r###"
    - { messageId: 'unnecessaryCasting', line: 3 }
    - { messageId: 'unnecessaryCasting', line: 4 }
    "###);
    assert_snapshot!(result.output, @r###"
    declare const s: string

    const a = String(/* keep */ s)
    const b = String(s /* keep */)
    "###);
}

#[test]
fn shadowed_builtin_casts_are_ignored() {
    valid(
        RULE,
        indoc! {"
            export {}

            function Number(value: number) {
              return value + 1
            }

            declare const s: string | undefined
            const n = Number(41)
            const String = (value: string | undefined) => value ?? ''
            const t = String(s)
        "},
    );
}

#[test]
fn optional_call_of_possibly_missing_cast_function_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                declare const toNumber: ((value: unknown) => number) | undefined

                const n = toNumber?.(1)
            "},
            options: to_number_cast_option(),
            filename: None,
        },
    );
}

#[test]
fn unnecessary_cast_fix_in_braceless_bodies() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const debug: boolean
            declare const a: string
            declare const b: string
            declare const n: number
            declare function log(value: string): void

            if (debug) String(a || b).split(',').forEach(log)
            else Number(-n).toFixed()
            while (debug) Number(-n)
            label: String(`${a}`).trim()
        "},
    );

    assert_snapshot!(result.output, @r###"
    declare const debug: boolean
    declare const a: string
    declare const b: string
    declare const n: number
    declare function log(value: string): void

    if (debug) (a || b).split(',').forEach(log)
    else (-n).toFixed()
    while (debug) -n
    label: `${a}`.trim()
    "###);
}

#[test]
fn unnecessary_cast_fix_keeps_directives_and_export_default_valid() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const s: unknown

            export function f(x = 1) {
              String('use strict')
              return x
            }
            const x = s
            String(<string>s)
            export default String(class {}.name)
        "},
    );

    assert_snapshot!(result.output, @r###"
    declare const s: unknown

    export function f(x = 1) {
      ('use strict')
      return x
    }
    const x = s
    ;<string>s
    export default (class {}.name)
    "###);
}

#[test]
fn number_cast_of_branded_primitive_or_to_primitive_object() {
    valid(
        RULE,
        indoc! {"
            declare const id: string & { __brand: 'id' }
            declare const count: number & { __brand: 'count' }
            declare const temperature: { [Symbol.toPrimitive](hint: string): number }

            const a = Number(id)
            const b = Number(count)
            const c = Number(temperature)
        "},
    );
}

#[test]
fn comment_like_text_in_a_type_argument_keeps_the_fix() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                declare function toStr<T>(value: unknown): string
                declare const s: string

                const x = toStr<'http://x'>(s)
            "},
            options: Some(json!({
                "additionalCastFunctions": [{ "name": "toStr", "expectedType": "string" }]
            })),
            filename: None,
        },
    );

    assert_snapshot!(result.output, @r###"
    declare function toStr<T>(value: unknown): string
    declare const s: string

    const x = s
    "###);
}

#[test]
fn values_that_may_hold_primitives_are_not_nan() {
    valid(
        RULE,
        indoc! {"
            declare const empty: {}
            declare const obj: Object
            declare const bool: Boolean
            declare const custom: { toString(): string }
            declare const valueOfString: { valueOf(): string }

            function narrowed(value: unknown) {
                if (value != null) return Number(value)
            }
            function generic<T extends {}>(value: T) {
                return Number(value)
            }
            const a = Number(empty)
            const b = Number(obj)
            const c = Number(bool)
            const d = Number(custom)
            const e = Number(valueOfString)
        "},
    );
}

#[test]
fn to_primitive_returning_a_symbol_is_invalid() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const weird: { [Symbol.toPrimitive](): symbol }
            const a = Number(weird)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'nanNumberCasting', line: 2 }");
}

#[test]
fn optional_call_of_an_any_cast_function_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                const toNumber: any = undefined
                let calls = 0
                const a = toNumber?.(++calls)
            "},
            options: to_number_cast_option(),
            filename: None,
        },
    );
}

#[test]
fn asi_guard_ignores_jsdoc_tokens() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const n: number
            const x = 1
            /** @type {{ */
            Number(-n)
        "},
    );

    assert_snapshot!(result.output, @r"
    declare const n: number
    const x = 1
    /** @type {{ */
    ;-n
    ");
}

#[test]
fn conversion_hooks_that_may_give_a_number_are_valid() {
    valid(
        RULE,
        indoc! {"
            declare const toPrimitiveKey: unique symbol
            declare const toPrimitiveNull: { [Symbol.toPrimitive]: null; valueOf(): number }
            declare const toPrimitiveOptional: { [Symbol.toPrimitive]?(): symbol; valueOf(): number }
            declare const valueOfNull: { valueOf(): null }
            declare const valueOfBranded: { valueOf(): number & { __brand: 'n' } }
            declare const userSymbolKey: { [toPrimitiveKey](): object; valueOf(): number }
            declare class Box<T> {
              valueOf(): T
            }

            function boxed<T extends number>(box: Box<T>) {
              return Number(box)
            }
            const a = Number(toPrimitiveNull)
            const b = Number(toPrimitiveOptional)
            const c = Number(valueOfNull)
            const d = Number(valueOfBranded)
            const e = Number(userSymbolKey)
        "},
    );
}

#[test]
fn conversion_hooks_that_never_give_a_number_are_invalid() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const toPrimitiveKey: unique symbol
            declare const toStringNumber: { toString: number }
            declare const toStringObject: { toString(): object }
            declare const userSymbolOnly: { [toPrimitiveKey](): number }

            const a = Number(toStringNumber)
            const b = Number(toStringObject)
            const c = Number(userSymbolOnly)
        "},
    );

    assert_snapshot!(result.errors(), @r###"
    - { messageId: 'nanNumberCasting', line: 6 }
    - { messageId: 'nanNumberCasting', line: 7 }
    - { messageId: 'nanNumberCasting', line: 8 }
    "###);
}

#[test]
fn unnecessary_cast_fix_keeps_in_operators_of_a_for_initializer() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const z: { a?: number }
            for (var q = Number('a' in z ? 1 : 2); ; ) break
        "},
    );

    assert_snapshot!(result.output, @r###"
    declare const z: { a?: number }
    for (var q = ('a' in z ? 1 : 2); ; ) break
    "###);
}
