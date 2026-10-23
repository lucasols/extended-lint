mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-unused-selected-values";

#[test]
fn no_selector_usage() {
    valid(
        RULE,
        Case {
            code: r#"const x = 5;"#,
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn all_selected_values_are_used() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency } = store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                }));
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn all_selected_values_are_used_with_direct_selector() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency, locale } = store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  locale: state.data?.locale_settings?.locale,
                }));
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn all_selected_values_are_used_with_nested_property() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const {
                  data: { deleteOrgAt, isTest, orgId },
                } = configDoc.useDocument({
                  selector: (data) => ({
                    deleteOrgAt: data?.delete_at,
                    isTest: data?.is_test,
                    orgId: data?.id,
                  }),
                });
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useDocument.selector","returnProp":"data"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn using_rest_operator() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency, ...rest } = configDoc.store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue: state.data.id,
                }));
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn non_object_selector_return_values() {
    valid(
        RULE,
        Case {
            code: r#"const value = useSimpleSelector(state => state.value);"#,
            options: Some(json!([{"selectors":[{"name":"useSimpleSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn not_using_destructuring() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const result = configDoc.store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue: state.data.id,
                }));
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn not_using_property_access() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const result = configDoc.store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue: state.data.id,
                }));

                console.log(result);
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn with_partial_property_access() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const result = configDoc.store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue: state.data.id,
                }));

                console.log(result.currency);
                console.log(result);
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn with_return_prop() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const config = configStore.useDocument({
                  selector: (data) => ({
                    icon: data?.logo_customizado,
                    name: data?.name,
                  }),
                });

                console.log(config.data.icon);
                console.log(config.data.name);
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useDocument","returnProp":"data","selectorProp":"selector"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn unused_selected_value() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency } = useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue: state.data.id,
                }));
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 3 }");
}

#[test]
fn unused_selected_value_2() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency } = useSelector((state) => {
                  return {
                    currency: state.data?.locale_settings?.currency_symbol,
                    unusedValue: state.data.id,
                  }
                });
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 4 }");
}

#[test]
fn unused_selected_value_3() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency } = store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue: state.data.id,
                }));
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 3 }");
}

#[test]
fn multiple_unused_selected_values() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency } = test.store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue1: state.data.id,
                  unusedValue2: state.data.name,
                }));
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedSelectedValue', line: 3 }
    - { messageId: 'unusedSelectedValue', line: 4 }
    ");
}

#[test]
fn unused_selected_value_with_selectorprop() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { deleteOrgAt, isTest } = configDoc.useDocument({
                  selector: (data) => ({
                    deleteOrgAt: data?.delete_at,
                    isTest: data?.is_test,
                    orgId: data?.id,
                  }),
                });
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useDocument","selectorProp":"selector"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 5 }");
}

#[test]
fn unused_selected_value_with_selectorprop_and_selectorargpos() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { deleteOrgAt, isTest } = configDoc.useItem('test', {
                  selector: (data) => ({
                    deleteOrgAt: data?.delete_at,
                    isTest: data?.is_test,
                    orgId: data?.id,
                  }),
                });
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useItem","selectorProp":"selector","selectorArgPos":1}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 5 }");
}

#[test]
fn unused_selected_value_with_selectorprop_2() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { deleteOrgAt, isTest } = configDoc.useDocument({
                  selector(data) {
                    return {
                      deleteOrgAt: data?.delete_at,
                      isTest: data?.is_test,
                      orgId: data?.id,
                    }
                  },
                });
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useDocument","selectorProp":"selector"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 6 }");
}

#[test]
fn unused_selected_value_with_returnprop() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const {
                  data: { deleteOrgAt, isTest },
                } = configDoc.useDocument({
                  selector: (data) => ({
                    deleteOrgAt: data?.delete_at,
                    isTest: data?.is_test,
                    orgId: data?.id,
                  }),
                });
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useDocument","returnProp":"data","selectorProp":"selector"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 7 }");
}

#[test]
fn multiple_unused_nested_selected_values() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const {
                  data: { deleteOrgAt },
                } = configDoc.useDocument({
                  selector: (data) => ({
                    deleteOrgAt: data?.delete_at,
                    isTest: data?.is_test,
                    orgId: data?.id,
                  }),
                });
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useDocument","returnProp":"data","selectorProp":"selector"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedSelectedValue', line: 6 }
    - { messageId: 'unusedSelectedValue', line: 7 }
    ");
}

#[test]
fn complex_member_expression_match() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const { currency } = myApp.features.user.settings.useUserPreferences((state) => ({
                  currency: state.settings.currency,
                  language: state.settings.language,
                }));
            "#},
            options: Some(json!([{"selectors":[{"name":"useUserPreferences"}]}])),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 3 }");
}

#[test]
fn with_property_access() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const result = configDoc.store.useSelector((state) => ({
                  currency: state.data?.locale_settings?.currency_symbol,
                  unusedValue: state.data.id,
                }));

                console.log(result.currency);
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedSelectedValue', line: 3 }");
}

#[test]
fn element_access_with_a_string_key_bails() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const result = useSelector((s) => ({ a: s.a, b: s.b }))
                console.log(result['a'])
            "#},
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn optional_call_is_ignored() {
    valid(
        RULE,
        Case {
            code: r#"const { a } = useSelector?.((s) => ({ a: s.a, b: s.b }))"#,
            options: Some(json!([{"selectors":[{"name":"useSelector"}]}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn returnprop_with_an_identifier_binding_is_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const config = useDocument({ selector: (d) => ({ a: d.a, b: d.b }) })
                console.log(config.data.a)
            "#},
            options: Some(
                json!([{"selectors":[{"name":"useDocument","selectorProp":"selector","returnProp":"data"}]}]),
            ),
            filename: Some("react.tsx"),
        },
    );
}
