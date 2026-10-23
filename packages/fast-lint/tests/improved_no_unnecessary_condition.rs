mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "improved-no-unnecessary-condition";

#[test]
fn valid_code_with_no_typeof_usage() {
    valid(
        RULE,
        indoc! {"
            const str = 'string'
            if (str > 0) {
              console.log(str)
            }
        "},
    );
}

#[test]
fn valid_typeof_check_on_variable_with_unknown_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: any) {
              if (typeof value === 'string') {
                console.log(value.toUpperCase())
              }
            }
        "},
    );
}

#[test]
fn valid_typeof_check_on_variable_with_union_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: string | number) {
              if (typeof value === 'string') {
                console.log(value.toUpperCase())
              }
            }
        "},
    );
}

#[test]
fn valid_typeof_check_in_ternary_with_union_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: string | number) {
              const result = typeof value === 'string' ? value.toUpperCase() : value.toString();
              console.log(result);
            }
        "},
    );
}

#[test]
fn invalid_typeof_check_in_ternary_with_literal_types() {
    let result = invalid(
        RULE,
        indoc! {"
            function test(value: 'str' | 1) {
              const result = typeof value === 'boolean' ? value.toUpperCase() : value.toString();
              console.log(result);
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_on_string_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const str = 'hello'
            if (typeof str === 'string') {
              console.log(str)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_on_string_variable_typeof_on_right_side() {
    let result = invalid(
        RULE,
        indoc! {"
            const str = 'hello'
            if ('string' === typeof str) {
              console.log(str)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_string_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const str = 'hello'
            if (typeof str === 'number') {
              console.log(str)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_on_number_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42
            if (typeof num === 'number') {
              console.log(num)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_number_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42
            if (typeof num === 'string') {
              console.log(num)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_on_boolean_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const bool = true
            if (typeof bool === 'boolean') {
              console.log(bool)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_boolean_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const bool = true
            if (typeof bool === 'object') {
              console.log(bool)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_on_object_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj = { a: 1 }
            if (typeof obj === 'object') {
              console.log(obj)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn ignore_operator() {
    valid(
        RULE,
        indoc! {"
            const str = 'hello'
            if (typeof str == 'string') {
              console.log(str)
            }
        "},
    );
}

#[test]
fn unnecessary_typeof_check_with_operator() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42
            if (typeof num !== 'string') {
              console.log(num)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_with_operator() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42
            if (typeof num !== 'number') {
              console.log(num)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_in_ternary_expression() {
    let result = invalid(
        RULE,
        indoc! {"
            const str = 'hello'
            const result = typeof str === 'string' ? str.toUpperCase() : str
            console.log(result)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_in_ternary_expression() {
    let result = invalid(
        RULE,
        indoc! {"
            const str = 'hello'
            const result = typeof str === 'number' ? str.toFixed(2) : str.toLowerCase()
            console.log(result)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_with_not_operator_in_ternary_expression() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42
            const result = typeof num !== 'string' ? num.toFixed(2) : 'Not a number'
            console.log(result)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_with_not_operator_in_ternary_expression() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42
            const result = typeof num !== 'number' ? 'Not a number' : num.toFixed(2)
            console.log(result)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_check_in_variable_with_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42 as string | number

            if (typeof num === 'boolean') {
              console.log(num)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 3 }");
}

#[test]
fn unnecessary_typeof_check_in_variable_with_union_type_in_ternary_expression() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42 as string | number

            const result = typeof num === 'boolean' ? num.toFixed(2) : num.toString()
            console.log(result)
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 3 }");
}

#[test]
fn unnecessary_typeof_check_in_variable_with_union_type_with_operator() {
    let result = invalid(
        RULE,
        indoc! {"
            const num = 42 as string | number

            if (typeof num !== 'boolean') {
              console.log(num)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 3 }");
}

#[test]
fn valid_typeof_check_for_function_parameter_that_could_be_a_function() {
    valid(
        RULE,
        indoc! {"
            function test(callback: any) {
              if (typeof callback === 'function') {
                callback()
              }
            }
        "},
    );
}

#[test]
fn unnecessary_typeof_check_on_function_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const myFunction = () => { console.log('hello') }
            if (typeof myFunction === 'function') {
              myFunction()
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_function_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const myFunction = () => { console.log('hello') }
            if (typeof myFunction === 'string') {
              console.log(myFunction)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn valid_typeof_check_for_symbol_parameter_that_could_be_a_different_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: any) {
              if (typeof value === 'symbol') {
                console.log(value.description)
              }
            }
        "},
    );
}

#[test]
fn unnecessary_typeof_check_on_symbol_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const sym = Symbol('description')
            if (typeof sym === 'symbol') {
              console.log(sym.description)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_symbol_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const sym = Symbol('description')
            if (typeof sym === 'number') {
              console.log(sym)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn valid_typeof_check_for_bigint_parameter_that_could_be_a_different_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: any) {
              if (typeof value === 'bigint') {
                console.log(value.toString())
              }
            }
        "},
    );
}

#[test]
fn unnecessary_typeof_check_on_bigint_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const big = 42n
            if (typeof big === 'bigint') {
              console.log(big.toString())
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_bigint_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const big = 42n
            if (typeof big === 'number') {
              console.log(big)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn valid_typeof_check_for_null_parameter_that_could_be_a_different_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: any) {
              if (typeof value === 'object' && value === null) {
                console.log('value is null')
              }
            }
        "},
    );
}

#[test]
fn unnecessary_typeof_check_on_null_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const nullVar = null
            if (typeof nullVar === 'object') {
              console.log('null is an object in JavaScript')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_null_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const nullVar = null
            if (typeof nullVar === 'string') {
              console.log(nullVar)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn valid_typeof_check_for_undefined_parameter_that_could_be_a_different_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: any) {
              if (typeof value === 'undefined') {
                console.log('value is undefined')
              }
            }
        "},
    );
}

#[test]
fn unnecessary_typeof_check_on_undefined_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const undefinedVar: undefined = undefined
            if (typeof undefinedVar === 'undefined') {
              console.log('variable is undefined')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_on_undefined_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const undefinedVar: undefined = undefined
            if (typeof undefinedVar === 'number') {
              console.log(undefinedVar)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn valid_typeof_check_for_complex_union_type_that_includes_the_checked_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: string | number | boolean) {
              if (typeof value === 'boolean') {
                console.log(value ? 'true' : 'false')
              }
            }
        "},
    );
}

#[test]
fn always_false_typeof_check_on_complex_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            const complexUnion = 'test' as string | number | boolean
            if (typeof complexUnion === 'object') {
              console.log(complexUnion)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_typeof_negation_check_on_complex_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            const complexUnion = 'test' as string | number | boolean
            if (typeof complexUnion !== 'object') {
              console.log(complexUnion)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn valid_typeof_check_with_optional_chaining_on_parameter_that_could_be_null_undefined() {
    valid(
        RULE,
        indoc! {"
            function test(obj: any) {
              if (typeof obj?.name === 'string') {
                console.log(obj.name.toUpperCase())
              }
            }
        "},
    );
}

#[test]
fn unnecessary_typeof_check_with_optional_chaining() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj = { name: 'test' }
            if (typeof obj?.name === 'string') {
              console.log(obj.name)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn always_false_typeof_check_with_optional_chaining() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj = { name: 'test' }
            if (typeof obj?.name === 'number') {
              console.log(obj.name)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn always_false_in_check_for_missing_property() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj: { a: string } = { a: 's' }

            if ('b' in obj) {
              console.log(obj.a)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseInCondition', line: 3 }");
}

#[test]
fn unnecessary_in_check_for_existing_property() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj: { a: string } = { a: 's' }

            if ('a' in obj) {
              console.log(obj.a)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryInCondition', line: 3 }");
}

#[test]
fn always_false_in_check_for_union_object() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj: { a: string } | { b: number } =
              Math.random() > 0.5 ? { a: 's' } : { b: 1 }

            if ('c' in obj) {
              console.log(obj)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseInCondition', line: 4 }");
}

#[test]
fn valid_in_check_for_union_object_property() {
    valid(
        RULE,
        indoc! {"
            declare const obj: { a: string } | { b: number }

            if ('a' in obj) {
              console.log(obj)
            }
        "},
    );
}

#[test]
fn unnecessary_in_check_for_intersection_object() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj: { a: string } & { b: number } = { a: 's', b: 1 }

            if ('a' in obj) {
              console.log(obj.a)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryInCondition', line: 3 }");
}

#[test]
fn always_false_in_check_for_intersection_object() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj: { a: string } & { b: number } = { a: 's', b: 1 }

            if ('c' in obj) {
              console.log(obj)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseInCondition', line: 3 }");
}

#[test]
fn valid_in_check_for_optional_property() {
    valid(
        RULE,
        indoc! {"
            const obj: { a?: string } = {}

            if ('a' in obj) {
              console.log(obj)
            }
        "},
    );
}

#[test]
fn valid_in_check_for_record_type() {
    valid(
        RULE,
        indoc! {"
            declare const obj: Record<string, string>

            if ('a' in obj) {
              console.log(obj)
            }
        "},
    );
}

#[test]
fn valid_logical_or_typeof_checks_on_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            function test(value: string | number) {
              if (typeof value === 'string' || typeof value === 'number') {
                console.log(value)
              }
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryTypeofCondition', line: 2 }");
}

#[test]
fn unnecessary_logical_or_typeof_checks_on_string_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const str = 'hello'
            if (typeof str === 'string' || typeof str === 'number') {
              console.log(str)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    ");
}

#[test]
fn always_false_logical_and_typeof_checks_on_string_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const str = 'hello'
            if (typeof str === 'string' && typeof str === 'number') {
              console.log(str)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"
    - { messageId: 'unnecessaryTypeofCondition', line: 2 }
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    ");
}

#[test]
fn non_nullable_typeof_check() {
    valid(
        RULE,
        indoc! {"
            function test(value: unknown) {
              if (value && typeof value === 'string') {
                console.log(value)
              }
            }
        "},
    );
}

#[test]
fn non_nullable_typeof_check_2() {
    valid(
        RULE,
        indoc! {"
            function test(value: {}) {
              if (typeof value === 'string') {
                console.log(value)
              }
            }
        "},
    );
}

#[test]
fn non_nullable_typeof_check_3() {
    let result = invalid(
        RULE,
        indoc! {"
            function test(value: {}) {
              if (typeof value === 'undefined') {
                console.log(value)
              }
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn argument_with_generic_type() {
    valid(
        RULE,
        indoc! {"
            function test<T>(value: T) {
              if (typeof value === 'string') {
                console.log(value)
              }
            }
        "},
    );
}

#[test]
fn valid_starts_with_check_on_union_type() {
    valid(
        RULE,
        indoc! {"
            function test(status: 'active' | 'inactive' | 'pending') {
              if (status.startsWith('act')) {
                console.log('starts with act')
              }
            }
        "},
    );
}

#[test]
fn always_false_starts_with_check_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' = 'active'
            if (status.startsWith('xyz')) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseStartsWithCondition', line: 2 }");
}

#[test]
fn always_true_starts_with_check_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' = 'active'
            if (status.startsWith('act')) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryStartsWithCondition', line: 2 }");
}

#[test]
fn always_false_ends_with_check_on_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
            if (status.endsWith('xyz')) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseEndsWithCondition', line: 2 }");
}

#[test]
fn always_true_ends_with_check_on_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
            if (status.endsWith('e')) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryEndsWithCondition', line: 2 }");
}

#[test]
fn always_false_includes_check_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const text: 'hello' = 'hello'
            if (text.includes('xyz')) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseIncludesCondition', line: 2 }");
}

#[test]
fn always_true_includes_check_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const text: 'hello' = 'hello'
            if (text.includes('ell')) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryIncludesCondition', line: 2 }");
}

#[test]
fn valid_includes_check_on_union_with_mixed_results() {
    valid(
        RULE,
        indoc! {"
            function test(text: 'hello' | 'world') {
              if (text.includes('o')) {
                console.log('some include o')
              }
            }
        "},
    );
}

#[test]
fn unnecessary_array_isarray_check_on_array_variable() {
    let result = invalid(
        RULE,
        indoc! {"
            const list: string[] = []
            if (Array.isArray(list)) {
              console.log(list)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryArrayIsArrayCondition', line: 2 }");
}

#[test]
fn always_false_array_isarray_check_on_string_literal() {
    let result = invalid(
        RULE,
        indoc! {"
            const value = 'text'
            if (Array.isArray(value)) {
              console.log(value)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseArrayIsArrayCondition', line: 2 }");
}

#[test]
fn always_false_array_isarray_check_on_record_string_unknown() {
    let result = invalid(
        RULE,
        indoc! {"
            const value: Record<string, unknown> = {}
            if (Array.isArray(value)) {
              console.log(value)
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseArrayIsArrayCondition', line: 2 }");
}

#[test]
fn valid_array_isarray_check_on_empty_object_type() {
    valid(
        RULE,
        indoc! {"
            function hasMeaningfulValue(value: {}): boolean {
              if (Array.isArray(value)) return value.some(Boolean)
              return true
            }
        "},
    );
}

#[test]
fn always_false_length_comparison_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const text: 'hello' = 'hello'
            if (text.length > 10) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseLengthCondition', line: 2 }");
}

#[test]
fn always_true_length_comparison_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const text: 'hello' = 'hello'
            if (text.length === 5) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryLengthCondition', line: 2 }");
}

#[test]
fn always_false_length_comparison_with_operator() {
    let result = invalid(
        RULE,
        indoc! {"
            const text: 'hello' = 'hello'
            if (text.length !== 5) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseLengthCondition', line: 2 }");
}

#[test]
fn valid_length_comparison_on_union_with_different_lengths() {
    valid(
        RULE,
        indoc! {"
            function test(text: 'hi' | 'hello') {
              if (text.length > 3) {
                console.log('some are longer than 3')
              }
            }
        "},
    );
}

#[test]
fn always_true_length_comparison_on_union_with_same_lengths() {
    let result = invalid(
        RULE,
        indoc! {"
            const text: 'cat' | 'dog' = 'cat' as 'cat' | 'dog'
            if (text.length === 3) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryLengthCondition', line: 2 }");
}

#[test]
fn string_methods_with_empty_string_literal() {
    let result = invalid(
        RULE,
        indoc! {"
            const empty: '' = ''
            if (empty.startsWith('a')) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseStartsWithCondition', line: 2 }");
}

#[test]
fn string_methods_with_empty_string_always_true_case() {
    let result = invalid(
        RULE,
        indoc! {"
            const empty: '' = ''
            if (empty.startsWith('')) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryStartsWithCondition', line: 2 }");
}

#[test]
fn valid_string_method_on_any_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: any) {
              if (value.startsWith && value.startsWith('test')) {
                console.log('might start with test')
              }
            }
        "},
    );
}

#[test]
fn valid_string_method_on_unknown_string_type() {
    valid(
        RULE,
        indoc! {"
            function test(value: string) {
              if (value.startsWith('test')) {
                console.log('might start with test')
              }
            }
        "},
    );
}

#[test]
fn valid_optional_chaining_starts_with_on_possibly_undefined() {
    valid(
        RULE,
        indoc! {"
            function test(text: string | undefined) {
              if (text?.startsWith('a')) {
                console.log('maybe starts with a')
              }
            }
        "},
    );
}

#[test]
fn always_false_starts_with_with_optional_chaining_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' = 'active'
            if (status?.startsWith('xyz')) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseStartsWithCondition', line: 2 }");
}

#[test]
fn always_true_starts_with_with_optional_chaining_on_literal_string() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' = 'active'
            if (status?.startsWith('act')) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryStartsWithCondition', line: 2 }");
}

#[test]
fn always_false_ends_with_with_optional_chaining_on_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
            if (status?.endsWith('xyz')) {
              console.log('never happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseEndsWithCondition', line: 2 }");
}

#[test]
fn always_true_ends_with_with_optional_chaining_on_union_type() {
    let result = invalid(
        RULE,
        indoc! {"
            const status: 'active' | 'inactive' = 'active' as 'active' | 'inactive'
            if (status?.endsWith('e')) {
              console.log('always happens')
            }
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'unnecessaryEndsWithCondition', line: 2 }");
}

#[test]
fn in_check_message_includes_configured_alternative() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                const obj: { a: number } = { a: 1 }
                if ('a' in obj) {}
            "},
            options: Some(serde_json::json!({ "alternativeForInCondition": "objectHasKey()" })),
            filename: None,
        },
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'unnecessaryInCondition'
      msg: 'This in check is unnecessary. Property "a" always exists on type "{ a: number; }". If this is a valid condition, you can use as alternative: objectHasKey()'
      line: 2
    "###);
}

#[test]
fn parenthesized_operands_are_checked() {
    let result = invalid(
        RULE,
        indoc! {"
            const str: 'a' | 'b' = 'a'
            if (typeof (str) === 'number') {}
            if ((str).startsWith('c')) {}
            if (Array.isArray((str))) {}
        "},
    );

    assert_snapshot!(result.errors(), @r"
    - { messageId: 'alwaysFalseTypeofCondition', line: 2 }
    - { messageId: 'alwaysFalseStartsWithCondition', line: 3 }
    - { messageId: 'alwaysFalseArrayIsArrayCondition', line: 4 }
    ");
}

#[test]
fn optional_chain_length_comparison_is_ignored() {
    valid(
        RULE,
        indoc! {"
            function test(str: 'abc' | undefined) {
              if (str?.length === 3) {}
            }
        "},
    );
}

#[test]
fn typeof_function_on_class_constructor() {
    let result = invalid(
        RULE,
        indoc! {"
            class Example {}

            if (typeof Example === 'function') {}
            if (typeof Example === 'object') {}
        "},
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'unnecessaryTypeofCondition'
      msg: 'This condition is unnecessary. The type of "Example" is always "function".'
      line: 3
    - messageId: 'alwaysFalseTypeofCondition'
      msg: 'This condition will always be false. The type of "Example" is "function" so the condition has no overlap with "object".'
      line: 4
    "###);
}

#[test]
fn typeof_object_on_non_primitive_object_type() {
    valid(
        RULE,
        indoc! {"
            function f(value: object) {
              if (typeof value === 'object') {}
              if (typeof value === 'function') {}
            }
        "},
    );
}

#[test]
fn typeof_on_non_primitive_object_type_never_string() {
    let result = invalid(
        RULE,
        indoc! {"
            function f(value: object) {
              if (typeof value === 'string') {}
            }
        "},
    );

    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'alwaysFalseTypeofCondition'
      msg: 'This condition will always be false. The type of "value" is "object | function" so the condition has no overlap with "string".'
      line: 2
    "###);
}

#[test]
fn string_method_annotation_fallback_respects_scope() {
    valid(
        RULE,
        indoc! {"
            const status: 'idle' | 'loading' = 'idle'

            function f(status: string) {
              return status.startsWith('err')
            }
        "},
    );
}

#[test]
fn in_check_with_empty_key_is_skipped() {
    valid(
        RULE,
        indoc! {"
            declare const obj: { a: string }

            if ('' in obj) {}
        "},
    );
}

#[test]
fn length_comparison_with_literal_on_the_left() {
    let result = invalid(
        RULE,
        indoc! {"
            declare const s: 'abc'

            if (5 > s.length) {}
            if (2 > s.length) {}
        "},
    );

    assert_snapshot!(result.errors(), @r###"
    - { messageId: 'unnecessaryLengthCondition', line: 3 }
    - { messageId: 'alwaysFalseLengthCondition', line: 4 }
    "###);
}

#[test]
fn array_is_array_on_generic_interfaces_is_valid() {
    valid(
        RULE,
        indoc! {"
            export function toArray<T>(value: Iterable<T> | ArrayLike<T>): T[] {
              if (Array.isArray(value)) return value
              return Array.from(value)
            }
        "},
    );
}

#[test]
fn typeof_function_on_function_type() {
    valid(
        RULE,
        indoc! {"
            declare const fn: Function | undefined
            if (typeof fn === 'function') {}
        "},
    );
    let result = invalid(
        RULE,
        indoc! {"
            declare const fn: Function
            if (typeof fn === 'object') {}
        "},
    );

    assert_snapshot!(result.errors(), @"- { messageId: 'alwaysFalseTypeofCondition', line: 2 }");
}

#[test]
fn in_check_with_template_literal_index_signature_is_valid() {
    valid(
        RULE,
        indoc! {"
            declare const attrs: { [key: `data-${string}`]: string }
            if ('data-x' in attrs) {}
        "},
    );
}

#[test]
fn string_method_annotation_fallback_ignores_nested_declarations() {
    valid(
        RULE,
        indoc! {"
            function f() {
              const status: 'idle' = 'idle'
              return status
            }

            function g() {
              return status.startsWith('err')
            }
        "},
    );
}
