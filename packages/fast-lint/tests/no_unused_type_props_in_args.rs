mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "no-unused-type-props-in-args";

#[test]
fn no_type_annotation() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function test({ usedType }: { [k: string]: string }) {
                  console.log(usedType);
                }

                function test(test: 'k') {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn no_unused_properties_with_object_type_literal() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function test({ usedType }: { usedType?: string }) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn no_unused_properties_with_object_type_reference() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type Test = {
                  usedType?: string;
                };

                function test({ usedType }: Test) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn ignore_param_type_unions() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type Test = {
                  usedType?: string;
                };

                function test({ usedType }: Test | { otherType?: string }) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn ignore_types_with_unions() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type Test = {
                  unusedType?: string;
                  usedType?: string;
                } | { otherType?: string };

                function test({ usedType }: Test) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn ignore_imported_types() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                import { Test } from './test';

                function test({ usedType }: Test) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn shared_types_still_ignored_by_default_multiple_references() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type Test = {
                  unusedType?: string;
                  usedType?: string;
                };

                function test({ usedType }: Test) {
                  console.log(usedType);
                }

                function test2({ usedType }: Test) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn ignore_rest_parameters() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };

                export const Component: FC<Props> = ({
                  title,
                  ...rest
                }) => {
                  return null;
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn ignore_exported_refs_rest_parameters_2() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };

                export type Props2 = Props

                export const Component: FC<Props> = ({
                  title,
                }) => {
                  return null;
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn shared_types_in_regular_functions_are_still_ignored_multiple_references() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };

                export const Component = ({
                  title,
                }: Props) => {
                  return null;
                };

                const usedProps: Props = {
                  title,
                  onClose,
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn shared_types_should_still_be_ignored_even_with_alwayscheckfunctionoptiontypes() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type RequestOptions = {
                  url: string;
                  method: string;
                  headers?: Record<string, string>;
                  body?: string;
                  timeout?: number;
                };

                function makeRequest({ url, method }: RequestOptions) {
                  return fetch(url, { method });
                }

                function anotherRequest({ url }: RequestOptions) {
                  return url;
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn disable_always_check_function_option_types_shared_type_should_be_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                type RequestOptions = {
                  url: string;
                  method: string;
                  headers?: Record<string, string>;
                  body?: string;
                  timeout?: number;
                };

                function makeRequest({ url, method }: RequestOptions) {
                  return fetch(url, { method });
                }

                function anotherRequest({ url }: RequestOptions) {
                  return url;
                }
            "#},
            options: Some(json!([{"alwaysCheckFunctionOptionTypes":false}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn disable_always_check_function_option_types_exported_type_should_be_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                export type AppCreationContext = {
                  creationRequest: string;
                  contextTables: string[];
                };

                export async function generateAppStructure({
                  creationRequest,
                }: AppCreationContext): Promise<string> {
                  return creationRequest;
                }
            "#},
            options: Some(json!([{"alwaysCheckFunctionOptionTypes":false}])),
            filename: Some("react.tsx"),
        },
    );
}

#[test]
fn unused_properties_with_object_type_literal() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function test({ usedType }: { unusedType?: string, usedType?: string }) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 1 }");
    assert_snapshot!(result.output, @r"
    function test({ usedType, unusedType }: { unusedType?: string, usedType?: string }) {
      console.log(usedType);
    }
    ");
}

#[test]
fn unused_properties_with_object_type_literal_2() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                const test = ({ usedType }: { unusedType?: string, usedType?: string }) => {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 1 }");
    assert_snapshot!(result.output, @r"
    const test = ({ usedType, unusedType }: { unusedType?: string, usedType?: string }) => {
      console.log(usedType);
    }
    ");
}

#[test]
fn unused_properties_with_object_type_reference() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Test = {
                  unusedType?: string;
                  usedType?: string;
                };

                function test({ usedType }: Test) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 2 }");
    assert_snapshot!(result.output, @r"
    type Test = {
      unusedType?: string;
      usedType?: string;
    };

    function test({ usedType, unusedType }: Test) {
      console.log(usedType);
    }
    ");
}

#[test]
fn unused_properties_with_object_interface_reference() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                interface Test {
                  unusedType?: string;
                  usedType?: string;
                };

                function test({ usedType }: Test) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 2 }");
    assert_snapshot!(result.output, @r"
    interface Test {
      unusedType?: string;
      usedType?: string;
    };

    function test({ usedType, unusedType }: Test) {
      console.log(usedType);
    }
    ");
}

#[test]
fn unused_properties_with_fc_object_type_reference() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };

                export const Component: FC<Props> = ({
                  title,
                }) => {
                  return null;
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 3 }");
    assert_snapshot!(result.output, @r"
    type Props = {
      title: ReactNode;
      onClose: () => void;
    };

    export const Component: FC<Props> = ({
      title, onClose,
    }) => {
      return null;
    };
    ");
}

#[test]
fn unused_properties_with_fc_object_type_literal() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };

                export const Component: FC<{
                  title: ReactNode;
                  onClose: () => void;
                }> = ({
                  title,
                }) => {
                  return null;
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 8 }");
    assert_snapshot!(result.output, @r"
    type Props = {
      title: ReactNode;
      onClose: () => void;
    };

    export const Component: FC<{
      title: ReactNode;
      onClose: () => void;
    }> = ({
      title, onClose,
    }) => {
      return null;
    };
    ");
}

#[test]
fn dont_ignore_types_with_intersections_referenced() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Test = {
                  unusedType?: string;
                  usedType?: string;
                } & { otherType?: string };

                function test({ usedType }: Test) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedObjectTypeProperty', line: 2 }
    - { messageId: 'unusedObjectTypeProperty', line: 4 }
    ");
    assert_snapshot!(result.output, @r"
    type Test = {
      unusedType?: string;
      usedType?: string;
    } & { otherType?: string };

    function test({ usedType, unusedType, otherType }: Test) {
      console.log(usedType);
    }
    ");
}

#[test]
fn dont_ignore_types_with_intersections() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function test({ usedType }: {
                  unusedType?: string;
                  usedType?: string;
                } & { otherType?: string }) {
                  console.log(usedType);
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedObjectTypeProperty', line: 2 }
    - { messageId: 'unusedObjectTypeProperty', line: 4 }
    ");
    assert_snapshot!(result.output, @r"
    function test({ usedType, unusedType, otherType }: {
      unusedType?: string;
      usedType?: string;
    } & { otherType?: string }) {
      console.log(usedType);
    }
    ");
}

#[test]
fn false_positive() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                import { sleep } from '@utils/sleep';

                export async function retryOnError<T>(
                  fn: () => Promise<T>,
                  maxRetries: number,
                  {
                    delayBetweenRetriesMs,
                    // retryCondition
                  }: {
                    delayBetweenRetriesMs?: number;
                    retryCondition?: (error: unknown) => boolean;
                  } = {},
                ): Promise<T> {
                  try {
                    return await fn();
                  } catch (error) {
                    if (maxRetries > 0) {
                      if (delayBetweenRetriesMs) {
                        await sleep(delayBetweenRetriesMs);
                      }

                      return retryOnError(fn, maxRetries - 1, { delayBetweenRetriesMs });
                    } else {
                      throw error;
                    }
                  }
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 11 }");
    assert_snapshot!(result.output, @r"
    import { sleep } from '@utils/sleep';

    export async function retryOnError<T>(
      fn: () => Promise<T>,
      maxRetries: number,
      {
        delayBetweenRetriesMs, retryCondition,
        // retryCondition
      }: {
        delayBetweenRetriesMs?: number;
        retryCondition?: (error: unknown) => boolean;
      } = {},
    ): Promise<T> {
      try {
        return await fn();
      } catch (error) {
        if (maxRetries > 0) {
          if (delayBetweenRetriesMs) {
            await sleep(delayBetweenRetriesMs);
          }

          return retryOnError(fn, maxRetries - 1, { delayBetweenRetriesMs });
        } else {
          throw error;
        }
      }
    }
    ");
}

#[test]
fn dont_ignore_exported_refs_in_fc_components() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };


                export const Component: FC<Props> = ({
                  title,
                }) => {
                  return null;
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 3 }");
    assert_snapshot!(result.output, @r"
    export type Props = {
      title: ReactNode;
      onClose: () => void;
    };


    export const Component: FC<Props> = ({
      title, onClose,
    }) => {
      return null;
    };
    ");
}

#[test]
fn dont_ignore_types_with_intersections_referenced_in_fc() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export type Props = {
                  onClose: () => void;
                };

                export const Component: FC<Props & {
                  otherType?: string;
                }> = ({
                  title,
                }) => {
                  return null;
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedObjectTypeProperty', line: 2 }
    - { messageId: 'unusedObjectTypeProperty', line: 6 }
    ");
    assert_snapshot!(result.output, @r"
    export type Props = {
      onClose: () => void;
    };

    export const Component: FC<Props & {
      otherType?: string;
    }> = ({
      title, onClose, otherType,
    }) => {
      return null;
    };
    ");
}

#[test]
fn test_bug() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type FormItemsInput = {
                  className?: string;
                  selected: FormItem[];
                  hint?: string;
                  label: string;
                  optional?: boolean;
                  errors: string[];
                  handleChange: (setter: (current: FormItem[]) => FormItem[]) => void;
                };

                export const FormItemsInput: FC<FormItemsInput> = ({
                  label,
                  hint,
                  errors,
                  selected,
                  handleChange,
                }) => {
                  return null
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedObjectTypeProperty', line: 2 }
    - { messageId: 'unusedObjectTypeProperty', line: 6 }
    ");
    assert_snapshot!(result.output, @r"
    type FormItemsInput = {
      className?: string;
      selected: FormItem[];
      hint?: string;
      label: string;
      optional?: boolean;
      errors: string[];
      handleChange: (setter: (current: FormItem[]) => FormItem[]) => void;
    };

    export const FormItemsInput: FC<FormItemsInput> = ({
      label,
      hint,
      errors,
      selected,
      handleChange, className, optional,
    }) => {
      return null
    };
    ");
}

#[test]
fn unused_properties_with_empty_destructed_props() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function test({}: { unusedType?: string }) {
                  console.log('ok');
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 1 }");
    assert_snapshot!(result.output, @r"
    function test({unusedType}: { unusedType?: string }) {
      console.log('ok');
    }
    ");
}

#[test]
fn fc_typed_with_props_but_not_used() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };

                export const Component: FC<Props> = () => {
                  return null;
                };
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'missingComponentParam', line: 6 }");
}

#[test]
fn props_used_in_fc_should_still_be_checked_even_if_referenced_more_than_once() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  title: ReactNode;
                  onClose: () => void;
                };

                export const Component: FC<Props> = ({
                  title,
                }) => {
                  return null;
                };

                const usedProps: Props = {
                  title,
                  onClose,
                };
            "#},
            options: Some(json!([{"forceCheckOnFCPropTypesWithName":["Props$"]}])),
            filename: Some("react.tsx"),
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 3 }");
    assert_snapshot!(result.output, @r"
    type Props = {
      title: ReactNode;
      onClose: () => void;
    };

    export const Component: FC<Props> = ({
      title, onClose,
    }) => {
      return null;
    };

    const usedProps: Props = {
      title,
      onClose,
    };
    ");
}

#[test]
fn always_check_function_option_types_by_default_exported_type() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export type AppCreationContext = {
                  creationRequest: string;
                  contextTables: string[];
                  contextApps: string[];
                  language: string;
                  companyColor: string | null;
                  tenantId: string;
                  agentId: string;
                  attachments: { file: string; fileType: string }[];
                };

                function generateAppStructure({
                  creationRequest,
                }: AppCreationContext): string {
                  return creationRequest; // only using creationRequest
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedObjectTypeProperty', line: 3 }
    - { messageId: 'unusedObjectTypeProperty', line: 4 }
    - { messageId: 'unusedObjectTypeProperty', line: 5 }
    - { messageId: 'unusedObjectTypeProperty', line: 6 }
    - { messageId: 'unusedObjectTypeProperty', line: 7 }
    - { messageId: 'unusedObjectTypeProperty', line: 8 }
    - { messageId: 'unusedObjectTypeProperty', line: 9 }
    ");
    assert_snapshot!(result.output, @r"
    export type AppCreationContext = {
      creationRequest: string;
      contextTables: string[];
      contextApps: string[];
      language: string;
      companyColor: string | null;
      tenantId: string;
      agentId: string;
      attachments: { file: string; fileType: string }[];
    };

    function generateAppStructure({
      creationRequest, contextTables, contextApps, language, companyColor, tenantId, agentId, attachments,
    }: AppCreationContext): string {
      return creationRequest; // only using creationRequest
    }
    ");
}

#[test]
fn always_check_function_option_types_single_use_type_should_still_work() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                function processData({ 
                  input, 
                }: { 
                  input: string; 
                  output?: string;
                  config?: Record<string, any>;
                }) {
                  return input;
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedObjectTypeProperty', line: 5 }
    - { messageId: 'unusedObjectTypeProperty', line: 6 }
    ");
    assert_snapshot!(result.output, @r"
    function processData({ 
      input, output, config, 
    }: { 
      input: string; 
      output?: string;
      config?: Record<string, any>;
    }) {
      return input;
    }
    ");
}

#[test]
fn debug_simple_exported_type_test() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                export type SimpleType = {
                  used: string;
                  unused: string;
                };

                function test({ used }: SimpleType) {
                  return used;
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 3 }");
    assert_snapshot!(result.output, @r"
    export type SimpleType = {
      used: string;
      unused: string;
    };

    function test({ used, unused }: SimpleType) {
      return used;
    }
    ");
}

#[test]
fn ignores_types_declared_outside_the_file() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f({ length }: Array<string>) {
                  return length
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn reports_props_of_an_fc_intersection_in_source_order() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Props = {
                  a: string
                }

                const Component: FC<{ z?: string } & Props> = ({}) => null
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'unusedObjectTypeProperty', line: 2 }
    - { messageId: 'unusedObjectTypeProperty', line: 5 }
    ");
    assert_snapshot!(result.output, @r"
    type Props = {
      a: string
    }

    const Component: FC<{ z?: string } & Props> = ({z, a}) => null
    ");
}

#[test]
fn type_alias_sharing_its_name_with_an_exported_component() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Item = {
                  a: string
                  b: string
                }

                export const Item: FC<Item> = ({ a }) => null
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 3 }");
    assert_snapshot!(result.output, @r"
    type Item = {
      a: string
      b: string
    }

    export const Item: FC<Item> = ({ a, b }) => null
    ");
}

#[test]
fn interface_extended_by_another_interface_counts_as_a_reference() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                interface Base {
                  a: string
                  b: string
                }

                interface Other extends Base {}

                function f({ a }: Base) {
                  return a
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}

#[test]
fn typeof_reference_is_not_a_type_reference() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
                type Opts = {
                  a: string
                  b: string
                }

                function f({ a }: Opts) {
                  return a
                }

                const g: typeof f = f
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'unusedObjectTypeProperty', line: 3 }");
    assert_snapshot!(result.output, @r"
    type Opts = {
      a: string
      b: string
    }

    function f({ a, b }: Opts) {
      return a
    }

    const g: typeof f = f
    ");
}

#[test]
fn overload_signatures_are_ignored() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
                function f({ a }: { a: string; b: string }): void
                function f(value: unknown) {
                  return value
                }
            "#},
            filename: Some("react.tsx"),
            ..Default::default()
        },
    );
}
