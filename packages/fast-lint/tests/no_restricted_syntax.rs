mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::{json, Value};

const RULE: &str = "no-restricted-syntax";

fn case(code: &str, options: Value) -> Case<'_> {
    Case {
        code,
        options: Some(options),
        ..Default::default()
    }
}

fn disallow(selector: &str) -> Value {
    json!({ "disallow": [{ "selector": selector, "message": format!("Using '{selector}' is not allowed.") }] })
}

fn options_error(options: Value) -> String {
    fast_lint::rules::create_rule(RULE, Some(&options))
        .err()
        .unwrap_or_default()
}

fn must_call_fn(literal: &str) -> Value {
    json!([{
        "anyCall": [{ "fn": "shouldCallFn", "withArgs": [{ "atIndex": 0, "literal": literal }] }],
    }])
}

fn open_manage_plan(extra: Value) -> Value {
    let mut entry = json!({
        "fn": "openModal",
        "withArgs": [{ "atIndex": 0, "value": "managePlan" }],
        "message": "Use openManagePlan function instead",
        "replaceWith": "openManagePlan()",
    });
    if let (Some(entry), Value::Object(extra)) = (entry.as_object_mut(), extra) {
        entry.extend(extra);
    }
    json!({ "disallowFnCalls": [entry] })
}

#[test]
fn disallow_valid() {
    valid(
        RULE,
        case("doSomething();", disallow("ConditionalExpression")),
    );
    valid(
        RULE,
        case("var foo = 42;", disallow("ConditionalExpression")),
    );
    valid(
        RULE,
        case(
            "foo += 42;",
            json!({ "disallow": [
                { "selector": "VariableDeclaration", "message": "no" },
                { "selector": "FunctionExpression", "message": "no" },
            ] }),
        ),
    );
    valid(RULE, case("foo;", disallow("Identifier[name=\"bar\"]")));
    valid(
        RULE,
        case(
            "() => 5",
            disallow("ArrowFunctionExpression > BlockStatement"),
        ),
    );
    valid(
        RULE,
        case("({ foo: 1, bar: 2 })", disallow("Property > Literal.key")),
    );
}

#[test]
fn disallow_variable_declaration() {
    let result = invalid(RULE, case("var foo = 41;", disallow("VariableDeclaration")));
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Using 'VariableDeclaration' is not allowed.'
      line: 1
    "###);
}

#[test]
fn disallow_empty_statement() {
    let result = invalid(
        RULE,
        case(
            ";function lol(a) { return 42; }",
            disallow("EmptyStatement"),
        ),
    );
    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'default', line: 1, column: 1, endLine: 1, endColumn: 2 }");
}

#[test]
fn disallow_reports_in_source_order() {
    let result = invalid(
        RULE,
        case(
            "try { voila(); } catch (e) { oops(); }",
            json!({ "disallow": [
                { "selector": "TryStatement", "message": "Using 'TryStatement' is not allowed." },
                { "selector": "CallExpression", "message": "Using 'CallExpression' is not allowed." },
                { "selector": "CatchClause", "message": "Using 'CatchClause' is not allowed." },
            ] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Using 'TryStatement' is not allowed.'
      line: 1
    - messageId: 'default'
      msg: 'Using 'CallExpression' is not allowed.'
      line: 1
    - messageId: 'default'
      msg: 'Using 'CatchClause' is not allowed.'
      line: 1
    - messageId: 'default'
      msg: 'Using 'CallExpression' is not allowed.'
      line: 1
    "###);
}

#[test]
fn disallow_with_attribute_on_list_length() {
    let result = invalid(
        RULE,
        case(
            "function foo(bar, baz, qux) {}\nfunction ok(a, b) {}",
            json!({ "disallow": [{ "selector": "FunctionDeclaration[params.length>2]", "message": "custom error message." }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'custom error message.'
      line: 1
    "###);
}

#[test]
fn disallow_chain_expression() {
    let result = invalid(
        RULE,
        case("var foo = foo?.bar?.();", disallow("ChainExpression")),
    );
    assert_snapshot!(result.errors_with_columns(), @"- { messageId: 'default', line: 1, column: 11, endLine: 1, endColumn: 23 }");
}

#[test]
fn disallow_attribute_only_selector() {
    let result = invalid(
        RULE,
        case("var foo = foo?.bar?.();", disallow("[optional=true]")),
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'default', line: 1, column: 11, endLine: 1, endColumn: 23 }
    - { messageId: 'default', line: 1, column: 11, endLine: 1, endColumn: 19 }
    "###);
}

#[test]
fn replace_suggestion() {
    let result = invalid(
        RULE,
        case(
            "invalidFnName();",
            json!({ "disallow": [{
                "selector": "CallExpression[callee.name=\"invalidFnName\"]",
                "message": "invalidFnName is not allowed",
                "replace": "validFnName()",
            }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'invalidFnName is not allowed'
      line: 1
    "###);
    assert_snapshot!(result.diagnostics[0].suggestions[0].message, @r###"Replace with "validFnName()""###);
    assert_snapshot!(result.suggestion_output(0), @"validFnName();");
}

#[test]
fn replace_with_regex_autofix() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                invalidFnName(validArg);

                invalidFnName(invalidArg2);
            "},
            json!({ "disallow": [{
                "selector": "CallExpression[callee.name=\"invalidFnName\"]",
                "message": "invalidFnName is not allowed",
                "replace": { "regex": "invalidFnName(.+)", "with": "validFnName$1" },
                "replaceType": "autofix",
            }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'invalidFnName is not allowed'
      line: 1
    - messageId: 'default'
      msg: 'invalidFnName is not allowed'
      line: 3
    "###);
    assert_snapshot!(result.output, @r###"
    validFnName(validArg);

    validFnName(invalidArg2);
    "###);
}

#[test]
fn replace_regex_uses_js_replacement_patterns() {
    let result = invalid(
        RULE,
        case(
            "oldName(a, b);",
            json!({ "disallow": [{
                "selector": "CallExpression[callee.name=\"oldName\"]",
                "message": "renamed",
                "replace": { "regex": "old(?<rest>Name)\\((.)", "with": "[$&|$<rest>|$2|$$|$9]" },
            }] }),
        ),
    );
    assert_snapshot!(result.diagnostics[0].suggestions[0].message, @r###"Replace with "[$&|$<rest>|$2|$$|$9]""###);
    assert_snapshot!(result.suggestion_output(0), @"[oldName(a|Name|a|$|$9], b);");
}

#[test]
fn reports_cover_the_estree_node() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                export function f() {}
                export default class Store extends Base {
                  method?(a: number) {}
                }
                const { b = 1 } = obj
                function g(...rest: number[]) {}
            "},
            json!({ "disallow": [
                { "selector": "FunctionDeclaration", "message": "function" },
                { "selector": "ClassDeclaration", "message": "class" },
                { "selector": "ClassBody", "message": "body" },
                { "selector": "MethodDefinition > FunctionExpression", "message": "method value" },
                { "selector": "AssignmentPattern", "message": "default" },
                { "selector": "RestElement", "message": "rest" },
            ] }),
        ),
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'default', line: 1, column: 8, endLine: 1, endColumn: 23 }
    - { messageId: 'default', line: 2, column: 16, endLine: 4, endColumn: 2 }
    - { messageId: 'default', line: 2, column: 41, endLine: 4, endColumn: 2 }
    - { messageId: 'default', line: 3, column: 10, endLine: 3, endColumn: 24 }
    - { messageId: 'default', line: 5, column: 9, endLine: 5, endColumn: 14 }
    - { messageId: 'default', line: 6, column: 1, endLine: 6, endColumn: 33 }
    - { messageId: 'default', line: 6, column: 12, endLine: 6, endColumn: 29 }
    "###);
}

#[test]
fn must_call_fn_valid() {
    valid(
        RULE,
        case(
            indoc! {"
                function foo(bar) {
                  shouldCallFn('foo');
                }
            "},
            json!({ "mustMatchSyntax": [{ "includeRegex": ".*", "mustCallFn": must_call_fn("foo") }] }),
        ),
    );
}

#[test]
fn must_call_fn_wrong_argument() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                function foo(bar) {
                  shouldCallFn('bar');
                }
            "},
            json!({ "mustMatchSyntax": [{ "includeRegex": ".*", "mustCallFn": must_call_fn("foo") }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Argument should have the value "foo"'
      line: 2
    "###);
    assert_snapshot!(result.output, @r###"
    function foo(bar) {
      shouldCallFn('foo');
    }
    "###);
}

#[test]
fn must_call_fn_missing_argument() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                function foo(bar) {
                  shouldCallFn();
                }
            "},
            json!({ "mustMatchSyntax": [{ "includeRegex": ".*", "mustCallFn": must_call_fn("foo") }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Missing argument with value "foo" at index 0'
      line: 2
    "###);
}

#[test]
fn must_call_fn_non_literal_argument_and_custom_message() {
    let result = invalid(
        RULE,
        case(
            "shouldCallFn(value);",
            json!({ "mustMatchSyntax": [{
                "includeRegex": ".*",
                "mustCallFn": [{
                    "anyCall": [{ "fn": "shouldCallFn", "withArgs": [{ "atIndex": 0, "literal": 1 }] }],
                    "message": "pass the id",
                }],
            }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Argument at position 0 should the literal "1": pass the id'
      line: 1
    "###);
}

#[test]
fn must_call_fn_missing_call() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                function foo(bar) {
                  const test = 1;
                }
            "},
            json!({ "mustMatchSyntax": [{ "includeRegex": ".*", "mustCallFn": must_call_fn("foo") }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Expected file to call the function: shouldCallFn'
      line: 1
    "###);
}

#[test]
fn must_call_fn_with_file_name_vars() {
    let code = indoc! {"
        function foo(bar) {
          shouldCallFn('string');
        }
    "};
    valid(
        RULE,
        case(
            code,
            json!({
                "__dev_simulateFileName": "stringFile.ts",
                "mustMatchSyntax": [{ "includeRegex": "(.+)File", "mustCallFn": must_call_fn("$1") }],
            }),
        ),
    );
    valid(
        RULE,
        case(
            code,
            json!({
                "__dev_simulateFileName": "StringFile.ts",
                "mustMatchSyntax": [{ "includeRegex": "(.+)File", "mustCallFn": must_call_fn("$1_lowercase") }],
            }),
        ),
    );
    valid(
        RULE,
        case(
            "shouldCallFn('stringType');",
            json!({
                "__dev_simulateFileName": "StringTypeFile.ts",
                "mustMatchSyntax": [{ "includeRegex": "(.+)File", "mustCallFn": must_call_fn("$1_uncapitalize") }],
            }),
        ),
    );
}

#[test]
fn must_call_fn_with_file_name_vars_wrong_argument() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                function foo(bar) {
                  shouldCallFn('number');
                }
            "},
            json!({
                "__dev_simulateFileName": "stringFile.ts",
                "mustMatchSyntax": [{ "includeRegex": "(.+)File.ts", "mustCallFn": must_call_fn("$1") }],
            }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Argument should have the value "string"'
      line: 2
    "###);
    assert_snapshot!(result.output, @r###"
    function foo(bar) {
      shouldCallFn('string');
    }
    "###);
}

#[test]
fn must_call_fn_multiple_calls_with_some_invalid() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                function foo(bar) {
                  shouldCallFn('string');
                  shouldCallFn('number');
                }
            "},
            json!({ "mustMatchSyntax": [{ "includeRegex": ".*", "mustCallFn": must_call_fn("string") }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Argument should have the value "string"'
      line: 3
    "###);
    assert_snapshot!(result.output, @r###"
    function foo(bar) {
      shouldCallFn('string');
      shouldCallFn('string');
    }
    "###);
}

#[test]
fn must_call_fn_any_of_several_calls() {
    valid(
        RULE,
        case(
            indoc! {"
                function foo(bar) {
                  shouldCallFn2(true, 'string');
                }
            "},
            json!({ "mustMatchSyntax": [{
                "includeRegex": ".*",
                "mustCallFn": [{
                    "anyCall": [
                        { "fn": "shouldCallFn", "withArgs": [{ "atIndex": 0, "literal": "string" }] },
                        { "fn": "shouldCallFn2", "withArgs": [{ "atIndex": 1, "literal": "string" }] },
                    ],
                }],
            }] }),
        ),
    );
}

fn must_match_selector(selector: &str, message: &str) -> Value {
    json!({ "mustMatchSyntax": [{
        "includeRegex": ".*",
        "mustMatchSelector": [{ "selector": selector, "message": message }],
    }] })
}

#[test]
fn must_match_selector_valid() {
    valid(
        RULE,
        case(
            "const foo = 'bar';",
            must_match_selector("Identifier[name=\"foo\"]", "noop"),
        ),
    );
    valid(
        RULE,
        case(
            "const { foo } = bar;",
            must_match_selector("Property[key.name=\"foo\"]", "noop"),
        ),
    );
    valid(
        RULE,
        case(
            "const string = 'bar';",
            json!({
                "__dev_simulateFileName": "stringFile.ts",
                "mustMatchSyntax": [{
                    "includeRegex": "(.+)File.ts$",
                    "mustMatchSelector": [{ "selector": "Identifier[name=\"$1\"]", "message": "noop" }],
                }],
            }),
        ),
    );
}

#[test]
fn must_match_selector_missing() {
    let result = invalid(
        RULE,
        case(
            "const test = 'bar';",
            must_match_selector("Identifier[name=\"foo\"]", "File should declare \"foo\""),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'File should declare "foo"'
      line: 1
    "###);
}

#[test]
fn must_match_selector_missing_in_destructuring() {
    let result = invalid(
        RULE,
        case(
            "const { test } = bar;",
            must_match_selector(
                "Property[key.name=\"foo\"]",
                "Missing declaration of \"foo\"",
            ),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Missing declaration of "foo"'
      line: 1
    "###);
}

#[test]
fn must_match_selector_with_file_name_var() {
    let result = invalid(
        RULE,
        case(
            "const number = 'bar';",
            json!({
                "__dev_simulateFileName": "stringFile.ts",
                "mustMatchSyntax": [{
                    "includeRegex": "(.+)File.ts$",
                    "mustMatchSelector": [{ "selector": "Identifier[name=\"$1\"]", "message": "File should declare \"$1\"" }],
                }],
            }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'File should declare "string"'
      line: 1
    "###);
}

#[test]
fn must_match_syntax_exclude_regex() {
    let options = |file: &str| {
        json!({
            "__dev_simulateFileName": file,
            "mustMatchSyntax": [{
                "includeRegex": ".*",
                "excludeRegex": "\\.test\\.",
                "mustMatchSelector": [{ "selector": "Identifier[name=\"foo\"]", "message": "File should declare \"foo\"" }],
            }],
        })
    };
    valid(
        RULE,
        case("const test = 'bar';", options("stringFile.test.ts")),
    );
    let result = invalid(RULE, case("const test = 'bar';", options("stringFile.ts")));
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'File should declare "foo"'
      line: 1
    "###);
    valid(
        RULE,
        case(
            "function foo(bar) { const test = 1; }",
            json!({
                "__dev_simulateFileName": "stringFile.spec.ts",
                "mustMatchSyntax": [{
                    "includeRegex": ".*",
                    "excludeRegex": "\\.(spec|test)\\.",
                    "mustCallFn": must_call_fn("foo"),
                }],
            }),
        ),
    );
    valid(
        RULE,
        case(
            "const testVar = 'test';",
            json!({
                "__dev_simulateFileName": "components/Test.stories.tsx",
                "mustMatchSyntax": [{
                    "includeRegex": ".*",
                    "excludeRegex": "\\.stories\\.",
                    "mustHaveExport": [{ "name": "testVar", "type": "variable", "message": "Must export testVar" }],
                }],
            }),
        ),
    );
}

#[test]
fn disallow_fn_calls_valid() {
    valid(
        RULE,
        case("someOtherFunction();", open_manage_plan(json!({}))),
    );
    valid(
        RULE,
        case("openModal(\"otherArg\");", open_manage_plan(json!({}))),
    );
    valid(
        RULE,
        case(
            "import { openModal as om } from 'modals';\nom(\"otherArg\");",
            open_manage_plan(json!({})),
        ),
    );
    valid(
        RULE,
        case(
            "const om = openModal;\nom(\"otherArg\");",
            open_manage_plan(json!({})),
        ),
    );
    valid(
        RULE,
        case("openModal(modalId);", open_manage_plan(json!({}))),
    );
    valid(
        RULE,
        case("openModal(getModalId());", open_manage_plan(json!({}))),
    );
    valid(
        RULE,
        case("openModal(config.modalId);", open_manage_plan(json!({}))),
    );
    valid(
        RULE,
        case(
            "modals.openModal(\"managePlan\");",
            open_manage_plan(json!({})),
        ),
    );
}

#[test]
fn disallow_fn_call_with_argument() {
    let result = invalid(
        RULE,
        case("openModal(\"managePlan\");", open_manage_plan(json!({}))),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Use openManagePlan function instead'
      line: 1
    "###);
    assert_snapshot!(result.diagnostics[0].suggestions[0].message, @r###"Replace with "openManagePlan()""###);
    assert_snapshot!(result.suggestion_output(0), @"openManagePlan();");
}

#[test]
fn disallow_fn_call_through_import_alias() {
    let result = invalid(
        RULE,
        case(
            "import { openModal as om } from 'modals';\nom(\"managePlan\");",
            open_manage_plan(json!({})),
        ),
    );
    assert_snapshot!(result.suggestion_output(0), @r###"
    import { openModal as om } from 'modals';
    openManagePlan();
    "###);
}

#[test]
fn disallow_fn_call_through_variable_alias() {
    let result = invalid(
        RULE,
        case(
            "const om = openModal;\nom(\"managePlan\");",
            open_manage_plan(json!({})),
        ),
    );
    assert_snapshot!(result.suggestion_output(0), @r###"
    const om = openModal;
    openManagePlan();
    "###);
}

#[test]
fn disallow_fn_call_missing_argument() {
    let result = invalid(RULE, case("openModal();", open_manage_plan(json!({}))));
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Missing argument with value "managePlan" at index 0: Use openManagePlan function instead'
      line: 1
    "###);
}

#[test]
fn disallow_fn_call_without_with_args() {
    let result = invalid(
        RULE,
        case(
            "openModal(modalId);",
            json!({ "disallowFnCalls": [{
                "fn": "openModal",
                "message": "Use openManagePlan function instead",
                "replaceWith": "openManagePlan()",
            }] }),
        ),
    );
    assert_snapshot!(result.suggestion_output(0), @"openManagePlan();");
    let result = invalid(
        RULE,
        case(
            "setVirtualBackendRequestHandler(handler);",
            json!({ "disallowFnCalls": [{ "fn": "setVirtualBackendRequestHandler", "message": "internal" }] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'internal'
      line: 1
    "###);
    assert!(result.diagnostics[0].suggestions.is_empty());
}

#[test]
fn disallow_fn_calls_ignore_regex() {
    let ignored = |file: &str, regex: &str| {
        let mut options = open_manage_plan(json!({ "ignoreRegex": regex }));
        if let Some(options) = options.as_object_mut() {
            options.insert("__dev_simulateFileName".to_string(), json!(file));
        }
        options
    };
    valid(
        RULE,
        case(
            "openModal(\"managePlan\");",
            ignored("test.spec.ts", "\\.spec\\."),
        ),
    );
    valid(
        RULE,
        case(
            "openModal(\"managePlan\");",
            ignored("components/test.stories.tsx", "\\.(stories|spec|test)\\."),
        ),
    );
    valid(
        RULE,
        case(
            "import { openModal as om } from 'modals';\nom(\"managePlan\");",
            ignored("Modal.test.ts", "\\.test\\."),
        ),
    );
    let result = invalid(
        RULE,
        case(
            "openModal(\"managePlan\");",
            ignored("components/Modal.tsx", "\\.spec\\."),
        ),
    );
    assert_snapshot!(result.suggestion_output(0), @"openManagePlan();");
}

fn must_have_export(requirements: Value) -> Value {
    json!({ "mustMatchSyntax": [{ "includeRegex": ".*", "mustHaveExport": requirements }] })
}

#[test]
fn must_have_export_valid() {
    let function = "export function testFunction() {\n  return 'test';\n}";
    let variable = "export const testVar = 'test';";
    let require = |name: &str, export_type: &str| {
        must_have_export(json!([{ "name": name, "type": export_type, "message": "noop" }]))
    };
    valid(RULE, case(function, require("testFunction", "function")));
    valid(RULE, case(variable, require("testVar", "variable")));
    valid(RULE, case(function, require("testFunction", "any")));
    valid(RULE, case(variable, require("testVar", "any")));
    valid(
        RULE,
        case(
            "const testFunction = () => 'test';\nexport { testFunction };",
            must_have_export(json!([{ "name": "testFunction", "message": "noop" }])),
        ),
    );
    valid(
        RULE,
        case(
            "export function TestComponent() {\n  return 'test';\n}",
            json!({
                "__dev_simulateFileName": "TestComponent.tsx",
                "mustMatchSyntax": [{
                    "includeRegex": "(.*)\\.tsx$",
                    "mustHaveExport": [{ "name": "$1", "type": "function", "message": "Must export $1" }],
                }],
            }),
        ),
    );
}

#[test]
fn must_have_export_missing() {
    let requirements = json!([
        { "name": "testFunction", "type": "function", "message": "Must export testFunction" },
        { "name": "testVar", "type": "variable", "message": "Must export testVar" },
        { "name": "anotherFunction", "type": "any", "message": "Must export anotherFunction" },
    ]);
    let result = invalid(
        RULE,
        case(
            "const testVar = 'test';",
            must_have_export(requirements.clone()),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Missing required export "testFunction" of type function: Must export testFunction'
      line: 1
    - messageId: 'default'
      msg: 'Missing required export "testVar" of type variable: Must export testVar'
      line: 1
    - messageId: 'default'
      msg: 'Missing required export "anotherFunction" of type any: Must export anotherFunction'
      line: 1
    "###);
    let result = invalid(
        RULE,
        case(
            "export function testVar() {\n  return 'test';\n}",
            must_have_export(requirements.clone()),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Missing required export "testFunction" of type function: Must export testFunction'
      line: 1
    - messageId: 'default'
      msg: 'Missing required export "testVar" of type variable: Must export testVar'
      line: 1
    - messageId: 'default'
      msg: 'Missing required export "anotherFunction" of type any: Must export anotherFunction'
      line: 1
    "###);
    let result = invalid(
        RULE,
        case(
            "export const testFunction = () => 'test';",
            must_have_export(requirements),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Missing required export "testFunction" of type function: Must export testFunction'
      line: 1
    - messageId: 'default'
      msg: 'Missing required export "testVar" of type variable: Must export testVar'
      line: 1
    - messageId: 'default'
      msg: 'Missing required export "anotherFunction" of type any: Must export anotherFunction'
      line: 1
    "###);
}

#[test]
fn identifier_selectors_match_every_identifier_but_jsx_names() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                import { dt } from 'debug'
                const value: dt = dt(1)
                ;({ dt: renamed } = obj)
                const { dt: other } = obj
                const el = <dt dt={1} />
            "},
            json!({ "disallow": [{ "selector": "Identifier[name=\"dt\"]", "message": "Beware!! `dt` should not be used in production" }] }),
        ),
    );
    assert_snapshot!(result.errors_with_columns(), @r###"
    - { messageId: 'default', line: 1, column: 10, endLine: 1, endColumn: 12 }
    - { messageId: 'default', line: 2, column: 14, endLine: 2, endColumn: 16 }
    - { messageId: 'default', line: 2, column: 19, endLine: 2, endColumn: 21 }
    - { messageId: 'default', line: 3, column: 5, endLine: 3, endColumn: 7 }
    - { messageId: 'default', line: 4, column: 9, endLine: 4, endColumn: 11 }
    "###);
}

#[test]
fn literal_value_selectors() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                throw new Error('Function not implemented.')
                const n = 1.50
                const ok = 'other'
            "},
            json!({ "disallow": [
                { "selector": "Literal[value='Function not implemented.']", "message": "not implemented" },
                { "selector": "Literal[value=1.5]", "message": "one and a half" },
            ] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'not implemented'
      line: 1
    - messageId: 'default'
      msg: 'one and a half'
      line: 2
    "###);
}

/// web-app-3's `eslint-core/no-restricted-syntax` selectors, as `disallow` entries.
fn web_app_selectors() -> Value {
    json!({ "disallow": [
        { "selector": "ForInStatement", "message": "for..in" },
        { "selector": "Identifier[name=\"dt\"]", "message": "dt" },
        { "selector": "Property[key.name=\"_debug\"]", "message": "_debug" },
        { "selector": "Identifier[name=\"watchValue\"]", "message": "watchValue" },
        { "selector": "Identifier[name=\"watchCount\"]", "message": "watchCount" },
        { "selector": "CallExpression[callee.property.name='sort']", "message": "sort" },
        { "selector": "CallExpression[callee.property.name='toSorted']", "message": "toSorted" },
        { "selector": "CallExpression[callee.name='Number']", "message": "Number" },
        { "selector": "Identifier[name='DEV_TEST_ENABLED_FLAG']", "message": "DEV_TEST_ENABLED_FLAG" },
        { "selector": "Literal[value='Function not implemented.']", "message": "not implemented" },
        { "selector": "VariableDeclarator[id.name='mut'][parent.kind='const']", "message": "mut" },
        { "selector": "ImportSpecifier[imported.name='NoInfer']", "message": "NoInfer" },
        {
            "selector": "TaggedTemplateExpression[tag.name='__'][quasi.quasis.0.tail=true][quasi.quasis.0.value.raw='']",
            "message": "empty translation",
        },
        { "selector": "JSXExpressionContainer[expression.type='SequenceExpression']", "message": "sequence in jsx" },
        {
            "selector": "TaggedTemplateExpression[tag.object.name='styled'] ArrowFunctionExpression, TaggedTemplateExpression[tag.callee.name='styled'] ArrowFunctionExpression, TaggedTemplateExpression[tag.callee.object.callee.name='styled'] ArrowFunctionExpression",
            "message": "styled arrow",
        },
        {
            "selector": "MemberExpression[property.name='onChange'][object.object.name='fieldProps']",
            "message": "fieldProps onChange",
        },
        { "selector": "UnaryExpression[operator='typeof'][argument.name='window']", "message": "typeof window" },
        {
            "selector": "TaggedTemplateExpression[tag.object.name='styled'][tag.property.name='button']",
            "message": "styled.button",
        },
        { "selector": "TSQualifiedName[left.name='React']", "message": "React namespace" },
    ] })
}

#[test]
fn web_app_selectors_valid() {
    valid(
        RULE,
        case(
            indoc! {"
                import { NoInferAlias } from 'types'
                import type { FC } from 'react'
                for (const key of Object.keys(obj)) {}
                const debug = { debug: true }
                sortBy(items, (item) => item.id)
                items.sorted()
                castToNumber(value)
                const message = 'Function implemented.'
                using mut = createMut()
                let mutable = 1
                const translated = __`text`
                const withValue = __`${name}`
                const title = __('')
                const element = <div prop={a} other={(b)} />
                const Button = styled(ButtonElement)`color: red;`
                const onClick = () => styled.div
                fieldProps.onChange
                props.value.onChange
                typeof document
                typeof window.x
                const Comp: FC = () => null
                type Props = ReactNode
            "},
            web_app_selectors(),
        ),
    );
}

#[test]
fn web_app_selectors_invalid() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                import { NoInfer } from 'ts-toolbelt'
                import { NoInfer as Alias } from 'other'
                for (const key in obj) {}
                const debug = { _debug: true }
                dt(value)
                watchValue(a)
                const { watchCount } = hooks
                if (DEV_TEST_ENABLED_FLAG) {}
                items.sort()
                items?.toSorted((a, b) => a - b)
                Number(value)
                throw new Error('Function not implemented.')
                const mut = createMut()
                const empty = __``
                const element = <div prop={(a, b)} />
                const Wrapper = styled.div`color: ${(p) => p.color};`
                const Styled = styled(Button)`color: ${(p) => p.color};`
                const Attrs = styled(Button).attrs({})`color: ${(p) => p.color};`
                const Nested = styled.div`${() => css`a: ${() => 1};`}`
                fieldProps.name.onChange(value)
                if (typeof window === 'undefined') {}
                const Btn = styled.button`color: red;`
                const Comp: React.FC = () => null
            "},
            options: Some(web_app_selectors()),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'NoInfer'
      line: 1
    - messageId: 'default'
      msg: 'NoInfer'
      line: 2
    - messageId: 'default'
      msg: 'for..in'
      line: 3
    - messageId: 'default'
      msg: '_debug'
      line: 4
    - messageId: 'default'
      msg: 'dt'
      line: 5
    - messageId: 'default'
      msg: 'watchValue'
      line: 6
    - messageId: 'default'
      msg: 'watchCount'
      line: 7
    - messageId: 'default'
      msg: 'DEV_TEST_ENABLED_FLAG'
      line: 8
    - messageId: 'default'
      msg: 'sort'
      line: 9
    - messageId: 'default'
      msg: 'toSorted'
      line: 10
    - messageId: 'default'
      msg: 'Number'
      line: 11
    - messageId: 'default'
      msg: 'not implemented'
      line: 12
    - messageId: 'default'
      msg: 'mut'
      line: 13
    - messageId: 'default'
      msg: 'empty translation'
      line: 14
    - messageId: 'default'
      msg: 'sequence in jsx'
      line: 15
    - messageId: 'default'
      msg: 'styled arrow'
      line: 16
    - messageId: 'default'
      msg: 'styled arrow'
      line: 17
    - messageId: 'default'
      msg: 'styled arrow'
      line: 18
    - messageId: 'default'
      msg: 'styled arrow'
      line: 19
    - messageId: 'default'
      msg: 'styled arrow'
      line: 19
    - messageId: 'default'
      msg: 'fieldProps onChange'
      line: 20
    - messageId: 'default'
      msg: 'typeof window'
      line: 21
    - messageId: 'default'
      msg: 'styled.button'
      line: 22
    - messageId: 'default'
      msg: 'React namespace'
      line: 23
    "###);
}

#[test]
fn web_app_must_match_selectors() {
    let options = |selector: &str| must_match_selector(selector, "missing");
    let variant = "ObjectPattern[parent.type=\"ArrowFunctionExpression\"] Property Identifier[name=\"variant\"]";
    valid(
        RULE,
        case(
            "export const FieldBtn = ({ variant, label }: Props) => null",
            options(variant),
        ),
    );
    valid(
        RULE,
        case(
            "export const FieldBtn = ({ props: { variant } }: Props) => null",
            options(variant),
        ),
    );
    let result = invalid(
        RULE,
        case(
            "export function FieldBtn({ variant }: Props) {}\nconst f = ({ label }: Props) => variant",
            options(variant),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'missing'
      line: 1
    "###);

    let props_story = "JSXElement[openingElement.name.name=\"FieldPropsStory\"]";
    valid(
        RULE,
        case(
            "export const Story = () => <FieldPropsStory a={1} />",
            options(props_story),
        ),
    );
    valid(
        RULE,
        case(
            "export const Story = () => <FieldPropsStory>x</FieldPropsStory>",
            options(props_story),
        ),
    );
    let result = invalid(
        RULE,
        case(
            "export const Story = () => <Other.FieldPropsStory />",
            options(props_story),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'missing'
      line: 1
    "###);

    let update_story = "JSXOpeningElement[name.name=\"FieldEditorStory\"] JSXAttribute[name.name=\"mode\"] ObjectExpression Property[key.name=\"type\"] Literal[value=\"update\"]";
    valid(
        RULE,
        case(
            "export const S = () => <FieldEditorStory mode={{ type: 'update', id: 1 }} />",
            options(update_story),
        ),
    );
    let result = invalid(
        RULE,
        case(
            indoc! {"
                export const S = () => <FieldEditorStory mode={{ type: 'create' }} other={{ type: 'update' }} />
                export const T = () => <Other mode={{ type: 'update' }} />
            "},
            options(update_story),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'missing'
      line: 1
    "###);
}

#[test]
fn web_app_must_call_fn_and_must_have_export() {
    let options = |file: &str| {
        json!({
            "__dev_simulateFileName": file,
            "mustMatchSyntax": [
                {
                    "includeRegex": "fields\\/(.+)Field\\/TableCellEditor.tsx$",
                    "mustCallFn": [{ "anyCall": [
                        { "fn": "useEditCellTypedProps", "withArgs": [{ "atIndex": 1, "literal": "$1_uncapitalize" }] },
                        { "fn": "useTypedProps", "withArgs": [{ "atIndex": 0, "literal": "$1_uncapitalize" }] },
                    ] }],
                },
                {
                    "includeRegex": "fields\\/(.+)\\/cfg.ts$",
                    "mustHaveExport": [{ "name": "$1Config", "message": "cfg.ts files should export a `$1Config` variable" }],
                },
            ],
        })
    };
    let editor = "src/tableFields/fields/ShortTextField/TableCellEditor.tsx";
    let cfg = "src/tableFields/fields/ShortTextField/cfg.ts";
    valid(
        RULE,
        case(
            "const props = useTypedProps('shortText', p)",
            options(editor),
        ),
    );
    let result = invalid(
        RULE,
        case(
            "const props = useEditCellTypedProps(p, 'longText')",
            options(editor),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Argument should have the value "shortText"'
      line: 1
    "###);
    assert_snapshot!(result.output, @"const props = useEditCellTypedProps(p, 'shortText')");
    valid(
        RULE,
        case("export const ShortTextFieldConfig = {}", options(cfg)),
    );
    let result = invalid(RULE, case("export const config = {}", options(cfg)));
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'Missing required export "ShortTextFieldConfig" of type any: cfg.ts files should export a `ShortTextFieldConfig` variable'
      line: 1
    "###);
}

#[test]
fn estree_shapes() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                export default function main() {}
                export class Store extends Base { method() {} }
                const { a = 1, ...rest } = obj
                ;[first, second = 2] = list
                label: for (;;) { break label }
                const fn = async function* () { yield* other }
                new Thing(...args)
                x ??= a?.b!.c
                const s = (1, 2, 3)
            "},
            json!({ "disallow": [
                { "selector": "ExportDefaultDeclaration > FunctionDeclaration[id.name='main']", "message": "default fn" },
                { "selector": "ExportNamedDeclaration > ClassDeclaration > ClassBody > MethodDefinition[kind='method'] > FunctionExpression", "message": "method" },
                { "selector": "ClassDeclaration[superClass.name='Base']", "message": "extends Base" },
                { "selector": "ObjectPattern > Property > AssignmentPattern[left.name='a'][right.value=1]", "message": "default a" },
                { "selector": "ObjectPattern > RestElement[argument.name='rest']", "message": "rest" },
                { "selector": "AssignmentExpression > ArrayPattern > AssignmentPattern.elements", "message": "array default" },
                { "selector": "BreakStatement[label.name='label']", "message": "break label" },
                { "selector": "FunctionExpression[async=true][generator=true] YieldExpression[delegate=true]", "message": "yield*" },
                { "selector": "NewExpression > SpreadElement", "message": "spread" },
                { "selector": "AssignmentExpression[operator='??='] > ChainExpression", "message": "chain" },
                { "selector": "SequenceExpression[expressions.length=3]", "message": "sequence" },
            ] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'default fn'
      line: 1
    - messageId: 'default'
      msg: 'extends Base'
      line: 2
    - messageId: 'default'
      msg: 'method'
      line: 2
    - messageId: 'default'
      msg: 'default a'
      line: 3
    - messageId: 'default'
      msg: 'rest'
      line: 3
    - messageId: 'default'
      msg: 'array default'
      line: 4
    - messageId: 'default'
      msg: 'break label'
      line: 5
    - messageId: 'default'
      msg: 'yield*'
      line: 6
    - messageId: 'default'
      msg: 'spread'
      line: 7
    - messageId: 'default'
      msg: 'chain'
      line: 8
    - messageId: 'default'
      msg: 'sequence'
      line: 9
    "###);
}

#[test]
fn matches_not_and_field_selectors() {
    let result = invalid(
        RULE,
        case(
            indoc! {"
                const __a = 1
                function __b(__c) {}
                class lower {}
                interface ok_name {}
                const ok = (x) => x
            "},
            json!({ "disallow": [
                {
                    "selector": ":matches(VariableDeclarator > Identifier.id, :function > Identifier.id, :function > Identifier.params)[name=/^__/]",
                    "message": "double underscore",
                },
                {
                    "selector": ":matches(ClassDeclaration, TSInterfaceDeclaration) > Identifier.id[name=/^[a-z]|_/]",
                    "message": "type name",
                },
                { "selector": "ArrowFunctionExpression:not([params.length=0])", "message": "arrow with params" },
            ] }),
        ),
    );
    assert_snapshot!(result.errors_with_msg(), @r###"
    - messageId: 'default'
      msg: 'double underscore'
      line: 1
    - messageId: 'default'
      msg: 'double underscore'
      line: 2
    - messageId: 'default'
      msg: 'double underscore'
      line: 2
    - messageId: 'default'
      msg: 'type name'
      line: 3
    - messageId: 'default'
      msg: 'type name'
      line: 4
    - messageId: 'default'
      msg: 'arrow with params'
      line: 5
    "###);
}

#[test]
fn unsupported_selectors_are_options_errors() {
    let errors: Vec<String> = [
        "Identifier ~ Identifier",
        "Identifier + Identifier",
        "Program:has(Identifier)",
        "Identifier:first-child",
        "Identifier:nth-child(2)",
        "!Identifier",
        ":statement",
        "TSAnyKeyword",
        "Foo",
        "Identifier[params]",
        "[nonexistent=1]",
        "Identifier[name.bogus=1]",
        "JSXExpressionContainer[expression.type='JSXEmptyExpression']",
        "*",
        ":not(Identifier)",
        "Identifier[name=/(?=a)/]",
        "Identifier[",
    ]
    .iter()
    .map(|selector| options_error(disallow(selector)))
    .collect();
    assert_snapshot!(errors.join("\n"), @r###"
    invalid options for rule no-restricted-syntax: invalid selector "Identifier ~ Identifier": sibling combinators (`~`, `+`) are not supported at position 11
    invalid options for rule no-restricted-syntax: invalid selector "Identifier + Identifier": sibling combinators (`~`, `+`) are not supported at position 11
    invalid options for rule no-restricted-syntax: invalid selector "Program:has(Identifier)": `:has` is not supported
    invalid options for rule no-restricted-syntax: invalid selector "Identifier:first-child": `:first-child` is not supported
    invalid options for rule no-restricted-syntax: invalid selector "Identifier:nth-child(2)": `:nth-child` is not supported
    invalid options for rule no-restricted-syntax: invalid selector "!Identifier": subject indicators (`!`) are not supported at position 0
    invalid options for rule no-restricted-syntax: invalid selector ":statement": `:statement` is not supported
    invalid options for rule no-restricted-syntax: invalid selector "TSAnyKeyword": unsupported node type `TSAnyKeyword`
    invalid options for rule no-restricted-syntax: invalid selector "Foo": unsupported node type `Foo`
    invalid options for rule no-restricted-syntax: invalid selector "Identifier[params]": field `params` is not supported on Identifier
    invalid options for rule no-restricted-syntax: invalid selector "[nonexistent=1]": unsupported field `nonexistent`
    invalid options for rule no-restricted-syntax: invalid selector "Identifier[name.bogus=1]": unsupported field `bogus`
    invalid options for rule no-restricted-syntax: invalid selector "JSXExpressionContainer[expression.type='JSXEmptyExpression']": unsupported node type `JSXEmptyExpression`
    invalid options for rule no-restricted-syntax: invalid selector "*": the matched node must be narrowed to node types (a type, `:function`, `[type=...]` or an attribute of specific node types)
    invalid options for rule no-restricted-syntax: invalid selector ":not(Identifier)": the matched node must be narrowed to node types (a type, `:function`, `[type=...]` or an attribute of specific node types)
    invalid options for rule no-restricted-syntax: invalid selector "Identifier[name=/(?=a)/]": invalid regex `/(?=a)/`: regex parse error:
        (?=a)
        ^^^
    error: look-around, including look-ahead and look-behind, is not supported
    invalid options for rule no-restricted-syntax: invalid selector "Identifier[": expected an attribute name at position 11
    "###);
}

#[test]
fn invalid_options_are_errors() {
    assert_snapshot!(options_error(json!({ "disalow": [] })), @"invalid options for rule no-restricted-syntax: unknown field `disalow`, expected one of `disallow`, `disallowFnCalls`, `mustMatchSyntax`, `__dev_simulateFileName`");
    assert_snapshot!(
        options_error(json!({ "disallow": [{ "selector": "Identifier", "message": "x", "replace": { "regex": "(", "with": "" } }] })),
        @r###"
    invalid options for rule no-restricted-syntax: invalid regex "(": regex parse error:
        (
        ^
    error: unclosed group
    "###
    );
    assert_snapshot!(
        options_error(json!({ "mustMatchSyntax": [{ "includeRegex": ".*", "mustMatchSelector": [{ "selector": "Bogus", "message": "x" }] }] })),
        @r###"invalid options for rule no-restricted-syntax: invalid selector "Bogus": unsupported node type `Bogus`"###
    );
}

#[test]
fn eslint_and_former_names_resolve_to_the_rules() {
    assert_eq!(
        fast_lint::rules::normalize_rule_name("@ls-stack/advanced-no-restricted-syntax"),
        RULE
    );
    assert_eq!(
        fast_lint::rules::normalize_rule_name("advanced-no-restricted-syntax"),
        RULE
    );
    assert_eq!(
        fast_lint::rules::normalize_rule_name("eslint-core/no-restricted-syntax"),
        RULE
    );
    assert_eq!(
        fast_lint::rules::normalize_rule_name("eslint-core/naming-convention"),
        "naming-convention"
    );
}
