mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-unused-t-state-field";

#[test]
fn with_no_unused_field() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                const Component = () => {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      text: { initialValue: 'text' },
                    },
                  })

                  const { formFields } = useFormState(formTypedCtx)

                  return <div>
                    <TextField value={formFields.name.value} />
                    <TextField value={formFields.text.value} />
                  </div>
                }
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn with_no_unused_field_2() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                const Component = () => {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      text: { initialValue: 'text' },
                    },
                  })

                  const { formFields: fields } = useFormState(formTypedCtx)

                  return <div>
                    <TextField value={fields.name.value} />
                    <TextField value={fields.text.value} />
                  </div>
                }
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn files_with_missing_import_are_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                const Component = () => {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      unused: { initialValue: 'unused' },
                    },
                  })

                  const { formFields } = useFormState(formTypedCtx)

                  return <div>
                    <TextField value={formFields.name.value} />
                  </div>
                }
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn ignore_when_formfields_is_returned_from_a_hook_in_an_object() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                function useFormFields() {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      unused: { initialValue: 'unused' },
                    },
                  })

                  const { formFields, formIsValid } = useFormState(formTypedCtx)

                  return { formFields }
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn ignore_when_formfields_is_returned_from_a_hook() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                function useFormFields() {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      unused: { initialValue: 'unused' },
                    },
                  })

                  const { formFields, formIsValid } = useFormState(formTypedCtx)

                  return formFields
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn has_unused_field() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                const Component = () => {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      unused: { initialValue: 'unused' },
                    },
                  })

                  const { formFields } = useFormState(formTypedCtx)

                  return <div>
                    <TextField value={formFields.name.value} />
                  </div>
                }
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedField', line: 7 }");
}

#[test]
fn has_unused_field_with_lazy_initialization() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                const Component = () => {
                  const { formTypedCtx } = useForm({
                    initialConfig: () => ({
                      name: { initialValue: 'John' },
                      unused: { initialValue: 'unused' },
                    }),
                  })

                  const { formFields } = useFormState(formTypedCtx)

                  return <div>
                    <TextField value={formFields.name.value} />
                  </div>
                }
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedField', line: 7 }");
}

#[test]
fn has_unused_field_with_lazy_initialization_2() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                const Component = () => {
                  const { formTypedCtx } = useForm({
                    initialConfig() {
                      return {
                        name: { initialValue: 'John' },
                        unused: { initialValue: 'unused' },
                      }
                    },
                  })

                  const { formFields } = useFormState(formTypedCtx)

                  return <div>
                    <TextField value={formFields.name.value} />
                  </div>
                }
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedField', line: 8 }");
}

#[test]
fn unused_fields_are_checked_when_spread_is_used() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                const Component = ({ config }) => {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      unused: { initialValue: 'unused' },
                      ...config,
                    },
                  })

                  const { formFields, formIsValid } = useFormState(formTypedCtx)

                  return <div>
                    <TextField value={formFields.name.value} />
                  </div>
                }
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedField', line: 7 }");
}

#[test]
fn unused_fields_are_checked_when_formfields_is_referenced_directly() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form';

                const Component = ({ config }) => {
                  const { formTypedCtx } = useForm({
                    initialConfig: {
                      name: { initialValue: 'John' },
                      unused: { initialValue: 'unused' },
                    },
                  })

                  const { formFields, formIsValid } = useFormState(formTypedCtx)

                  return <Container formFields={formFields}>
                    <TextField value={formFields.name.value} />
                  </Container>
                }
            "#},
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedField', line: 7 }");
}

#[test]
fn non_jsx_files_are_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form'

                function useFields() {
                  const { formTypedCtx } = useForm({
                    initialConfig: { name: { initialValue: '' }, unused: { initialValue: '' } },
                  })
                  const { formFields } = useFormState(formTypedCtx)
                  console.log(formFields.name)
                }
            "#},
            ..Default::default()
        },
    );
}

#[test]
fn element_access_with_a_non_identifier_key_bails() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { useForm } from 't-state-form'

                function useFields() {
                  const { formTypedCtx } = useForm({
                    initialConfig: { name: { initialValue: '' }, unused: { initialValue: '' } },
                  })
                  const { formFields } = useFormState(formTypedCtx)
                  console.log(formFields['name'])
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}
