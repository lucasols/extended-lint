mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::{json, Value};

const RULE: &str = "no-relative-imports";

fn aliases() -> Value {
    json!([
        { "find": "@utils", "replacement": "/src/utils" },
        { "find": "@src", "replacement": "/src" },
    ])
}

fn component_options() -> Option<Value> {
    Some(json!({
        "aliases": aliases(),
        "rootDir": "/Users/username/",
        "_dev_simulateFileName": "/Users/username/src/components/Component.ts",
    }))
}

#[test]
fn absolute_imports_with_aliases() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                import { something } from '@src/components/Button';
                import { utils } from '@utils/helpers';
            "},
            options: Some(json!({ "aliases": aliases() })),
            filename: None,
        },
    );
}

#[test]
fn third_party_imports() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                import React from 'react';
                import { useState } from 'react';
            "},
            options: Some(json!({ "aliases": aliases() })),
            filename: None,
        },
    );
}

#[test]
fn relative_import_same_directory() {
    let result = invalid(
        RULE,
        Case {
            code: "import { Button } from './Button';",
            options: component_options(),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noRelativeImportsWithAlias'
      msg: 'Relative imports are not allowed. Use the "@src" alias instead.'
      line: 1
    "#);
    assert_snapshot!(result.output, @"import { Button } from '@src/components/Button';");
}

#[test]
fn relative_import_parent_directory() {
    let result = invalid(
        RULE,
        Case {
            code: "import { helper } from '../utils/helper';",
            options: component_options(),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noRelativeImportsWithAlias'
      msg: 'Relative imports are not allowed. Use the "@utils" alias instead.'
      line: 1
    "#);
    assert_snapshot!(result.output, @"import { helper } from '@utils/helper';");
}

#[test]
fn multiple_relative_imports() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {"
                import { Button } from './Button';
                import { helper } from '../utils/helper';
                import { something } from '../../src/something';
            "},
            options: component_options(),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r#"
    - messageId: 'noRelativeImportsWithAlias'
      msg: 'Relative imports are not allowed. Use the "@src" alias instead.'
      line: 1
    - messageId: 'noRelativeImportsWithAlias'
      msg: 'Relative imports are not allowed. Use the "@utils" alias instead.'
      line: 2
    - messageId: 'noRelativeImportsWithAlias'
      msg: 'Relative imports are not allowed. Use the "@src" alias instead.'
      line: 3
    "#);
    assert_snapshot!(result.output, @r"
    import { Button } from '@src/components/Button';
    import { helper } from '@utils/helper';
    import { something } from '@src/something';
    ");
}

#[test]
fn allow_not_found_aliases() {
    valid(
        RULE,
        Case {
            code: indoc! {"
                import { something } from './components/Button';
                import { utils } from './helpers';
            "},
            options: Some(json!({
                "aliases": aliases(),
                "allowNotFoundAliases": true,
                "_dev_simulateFileName": "/Users/username/tests/components/Component.ts",
            })),
            filename: None,
        },
    );
}

#[test]
fn relative_import_without_matching_alias() {
    let result = invalid(
        RULE,
        Case {
            code: "import { x } from '../../other/x';",
            options: component_options(),
            filename: None,
        },
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'noRelativeImports'
      msg: 'Relative imports are not allowed. Use one of the valid aliases in tsconfig.json instead.'
      line: 1
    ");
    assert_snapshot!(result.output, @"import { x } from '../../other/x';");
}

#[test]
fn import_outside_root_dir_has_no_alias() {
    let result = invalid(
        RULE,
        Case {
            code: "import { x } from '../../../outside/x';",
            options: component_options(),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'noRelativeImports', line: 1 }");
}

#[test]
fn aliases_are_required_when_options_are_given() {
    let error = fast_lint::rules::create_rule(RULE, Some(&json!({ "rootDir": "/" })))
        .err()
        .unwrap_or_default();
    assert_snapshot!(error, @"invalid options for rule no-relative-imports: missing field `aliases`");
}
