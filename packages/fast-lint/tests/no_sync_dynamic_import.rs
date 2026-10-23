mod common;

use common::{invalid, valid};
use indoc::indoc;
use insta::assert_snapshot;

const RULE: &str = "no-sync-dynamic-import";

#[test]
fn allows_regular_es6_imports() {
    valid(
        RULE,
        indoc! {"
            import { something } from './module'
            import type { SomeType } from './types'
            import * as utils from './utils'
            import defaultExport from './default'
        "},
    );
}

#[test]
fn allows_async_dynamic_imports_with_await() {
    valid(
        RULE,
        indoc! {"
            async function loadModule() {
              const module = await import('./dynamic-module')
              return module.default
            }
        "},
    );
}

#[test]
fn allows_dynamic_imports_with_promise_chaining() {
    valid(
        RULE,
        indoc! {"
            function loadModule() {
              return import('./dynamic-module').then(module => module.default)
            }
        "},
    );
}

#[test]
fn allows_dynamic_imports_in_promise_all() {
    valid(
        RULE,
        indoc! {"
            const modules = await Promise.all([
              import('./module1'),
              import('./module2')
            ])
        "},
    );
}

#[test]
fn disallows_synchronous_dynamic_import_in_type_annotation() {
    let result = invalid(
        RULE,
        indoc! {"
            type GetUserPermissionsOptions = {
              planId: import('@src/config/plansCfg').NormalizedPlansIds;
            }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'syncDynamicImport', line: 2 }");
}

#[test]
fn disallows_synchronous_dynamic_import_in_variable_assignment() {
    let result = invalid(RULE, "const config = import('./config')");
    assert_snapshot!(result.errors(), @"- { messageId: 'syncDynamicImport', line: 1 }");
}

#[test]
fn disallows_synchronous_dynamic_import_in_function_return() {
    let result = invalid(
        RULE,
        indoc! {"
            function getConfig() {
              return import('./config')
            }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'syncDynamicImport', line: 2 }");
}

#[test]
fn disallows_synchronous_dynamic_import_in_object_property() {
    let result = invalid(
        RULE,
        indoc! {"
            const obj = {
              config: import('./config')
            }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'syncDynamicImport', line: 2 }");
}

#[test]
fn disallows_require_statements() {
    let result = invalid(
        RULE,
        indoc! {"
            const fs = require('fs')
            const path = require('path')
        "},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'requireNotAllowed', line: 1 }
    - { messageId: 'requireNotAllowed', line: 2 }
    ");
}

#[test]
fn disallows_require_resolve() {
    let result = invalid(RULE, "const modulePath = require.resolve('some-module')");
    assert_snapshot!(result.errors(), @"- { messageId: 'requireNotAllowed', line: 1 }");
}

#[test]
fn disallows_require_with_destructuring() {
    let result = invalid(
        RULE,
        indoc! {"
            const { readFile } = require('fs')
            const { join } = require('path')
        "},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'requireNotAllowed', line: 1 }
    - { messageId: 'requireNotAllowed', line: 2 }
    ");
}

#[test]
fn disallows_require_in_conditional_context() {
    let result = invalid(
        RULE,
        indoc! {"
            if (condition) {
              const module = require('./module')
            }
        "},
    );
    assert_snapshot!(result.errors(), @"- { messageId: 'requireNotAllowed', line: 2 }");
}

#[test]
fn allows_nested_await_import() {
    valid(
        RULE,
        indoc! {"
            async function loadModules() {
              const modules = await Promise.all([
                import('./module1'),
                import('./module2')
              ])

              for (const moduleName of moduleNames) {
                await import(`./${moduleName}`)
              }
            }
        "},
    );
}

#[test]
fn allows_dynamic_import_as_arrow_function_return() {
    valid(
        RULE,
        indoc! {"
            const i18n = i18ninitialize({
              ackLocale: 'en',
              es: {
                d: 'en',
                translations: () => import('@src/i18n/en-US.json')
              },
              pt: {
                d: 'pt',
                translations: () => import('@src/i18n/pt-BR.json')
              }
            })
        "},
    );
}

#[test]
fn disallows_dynamic_import_in_regular_function_return_but_allows_arrow_function() {
    valid(
        RULE,
        indoc! {"
            // This should be allowed - arrow function returning import
            const getLoader = () => import('./loader')

            // This should be allowed - arrow function in object
            const config = {
              loader: () => import('./config-loader')
            }
        "},
    );
}

#[test]
fn disallows_complex_synchronous_dynamic_import_usage() {
    let result = invalid(
        RULE,
        indoc! {"
            type ApiProfile = import('./api-types').Profile

            type GetUserPermissionsOptions = {
              profile: Partial<ApiProfile> | null;
              sessionType?: 'user' | 'public';
              planId: import('@src/config/plansCfg').NormalizedPlansIds;
            }
        "},
    );
    assert_snapshot!(result.errors(), @r"
    - { messageId: 'syncDynamicImport', line: 1 }
    - { messageId: 'syncDynamicImport', line: 6 }
    ");
}

#[test]
fn parenthesized_forms_are_looked_through() {
    valid(
        RULE,
        indoc! {"
            const a = () => (import('./a'))
            const b = (import('./b')).then(m => m)
            const c = Promise.all(([import('./c')]))
        "},
    );
}

#[test]
fn messages() {
    let result = invalid(
        RULE,
        indoc! {"
            const config = import('./config')
            const fs = require('fs')
        "},
    );
    assert_snapshot!(result.errors_with_msg(), @r"
    - messageId: 'syncDynamicImport'
      msg: 'Synchronous dynamic import() is not allowed. Use regular import statement or await import() for dynamic imports.'
      line: 1
    - messageId: 'requireNotAllowed'
      msg: 'require() statements are not allowed. Use ES6 import statements instead.'
      line: 2
    ");
}
