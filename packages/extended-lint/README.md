# Extended rules for eslint

To use any of these rules, you need to install the package:

```sh
pnpm add -D @lucasols/eslint-plugin-extended-lint
```

There is no `typescript` peer dependency (only an optional `eslint` one), so it works with any TypeScript version (including TypeScript 7) or none at all, and also as an [oxlint JS plugin](https://oxc.rs/docs/guide/usage/linter/js-plugins).

And then configure it in your eslint config:

```json
{
  "plugins": ["@lucasols/extended-lint"],
  "rules": {
    "@lucasols/extended-lint/<rule-name>": "error"
  }
}
```

Rules are being migrated to [`@ls-stack/fast-lint`](../fast-lint), where these now live (same names and options, so
`@ls-stack/<rule>` configs and disable comments keep working there): `improved-no-unnecessary-condition`,
`strict-casting` (also replaces the removed `no-unnecessary-casting`), `no-unnecessary-void-on-promise`,
`advanced-no-restricted-syntax` (now `no-restricted-syntax`), `no-default-export`, `no-reexport`, `no-relative-imports`,
`no-sync-dynamic-import`, `no-leaked-text-in-jsx`, `no-static-style-prop`, `no-non-camel-case-functions`,
`prefer-named-functions`, `no-call-with-explicit-generics`, `no-call-with-inferred-generics`, `no-unnecessary-typing`,
`no-type-guards`, `use-state-setter-naming`, `require-lazy-execution`, `no-unnecessary-iife`, `require-description`,
`no-write-only-ref`, `no-commented-out-code`, `react-single-export`, `no-unused-optional-args`,
`no-unused-type-props-in-args`, `no-unused-t-state-field`, `no-unused-selected-values`, `no-optional-root-props`,
`no-redundant-function-params`, `react-no-module-level-single-use-values`, `require-reads-to-var-prop`,
`prefer-react-hook-alternative`, `sort-react-components-and-styles`, `use-types-directly-above-usage` and
`react-compiler-extra`. The `rules-of-hooks` and `exhaustive-deps` forks were removed; use `eslint-plugin-react-hooks`
(or oxlint's built-in rules) instead. `collapse-simple-objs-in-one-line`, `prefer-single-line-if`,
`react-compiler-migration`, `template-indent`, `no-unnecessary-describe` and `no-data-attributes-on-components` were
removed.

Remaining rules: `no-restricted-comments`, `no-unnecessary-async-on-jsx-props`, `no-unused-obj-props`,
`require-usage-explanation` and `use-top-level-regex`.
