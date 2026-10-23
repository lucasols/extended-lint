# @ls-stack/fast-lint

Type-aware lint rules running natively on [tsrs](https://github.com/maschwenk/tsrs), the Rust port of the TypeScript 7
checker, through [tsrslint](https://github.com/maschwenk/tsrslint)'s rule framework. No ESLint, no Node type checker:
only these rules run, against one shared program per tsconfig. It replaces the `@ls-stack/extended-lint` ESLint
plugin, whose rules were all moved here or removed.

## Rules

| rule | origin |
| --- | --- |
| `improved-no-unnecessary-condition` | moved from `@ls-stack/extended-lint` |
| `strict-casting` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unnecessary-void-on-promise` | moved from `@ls-stack/extended-lint` |
| `no-leaked-conditional-rendering` | port of `@eslint-react/no-leaked-conditional-rendering` (v2.3.13); option `reactVersion` (default `"19.0.0"`, also used for `""` and `"detect"`: the installed version isn't read) |
| `no-duplicate-key` | stricter version of `@eslint-react/no-duplicate-key` (v2.3.13, see below) |
| `no-restricted-syntax` | ESLint's `no-restricted-syntax` merged with `@ls-stack/advanced-no-restricted-syntax` (moved from `@ls-stack/extended-lint`; see below) |
| `naming-convention` | new, replaces `no-restricted-syntax` naming selectors (see below) |
| `no-default-export` | moved from `@ls-stack/extended-lint` |
| `no-reexport` | moved from `@ls-stack/extended-lint` |
| `no-relative-imports` | moved from `@ls-stack/extended-lint` (see below) |
| `no-sync-dynamic-import` | moved from `@ls-stack/extended-lint` |
| `no-leaked-text-in-jsx` | moved from `@ls-stack/extended-lint` |
| `no-static-style-prop` | moved from `@ls-stack/extended-lint` (see below) |
| `no-non-camel-case-functions` | moved from `@ls-stack/extended-lint` (see below) |
| `prefer-named-functions` | moved from `@ls-stack/extended-lint` (see below) |
| `no-call-with-explicit-generics` | moved from `@ls-stack/extended-lint` (see below) |
| `no-call-with-inferred-generics` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unnecessary-typing` | moved from `@ls-stack/extended-lint` (see below) |
| `no-type-guards` | moved from `@ls-stack/extended-lint` (see below) |
| `use-state-setter-naming` | moved from `@ls-stack/extended-lint` (see below) |
| `require-lazy-execution` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unnecessary-iife` | moved from `@ls-stack/extended-lint` (see below) |
| `require-description` | moved from `@ls-stack/extended-lint` (see below) |
| `no-write-only-ref` | moved from `@ls-stack/extended-lint` (see below) |
| `no-commented-out-code` | moved from `@ls-stack/extended-lint` (see below) |
| `react-single-export` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unused-optional-args` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unused-type-props-in-args` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unused-t-state-field` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unused-selected-values` | moved from `@ls-stack/extended-lint` (see below) |
| `no-optional-root-props` | moved from `@ls-stack/extended-lint` (see below) |
| `no-redundant-function-params` | moved from `@ls-stack/extended-lint` (see below) |
| `react-no-module-level-single-use-values` | moved from `@ls-stack/extended-lint` (see below) |
| `require-reads-to-var-prop` | moved from `@ls-stack/extended-lint` (see below) |
| `prefer-react-hook-alternative` | moved from `@ls-stack/extended-lint` (see below) |
| `sort-react-components-and-styles` | moved from `@ls-stack/extended-lint` (see below) |
| `use-types-directly-above-usage` | moved from `@ls-stack/extended-lint` (see below) |
| `react-compiler-extra` | moved from `@ls-stack/extended-lint` (see below) |
| `no-restricted-comments` | moved from `@ls-stack/extended-lint` (see below) |
| `require-usage-explanation` | moved from `@ls-stack/extended-lint` (see below) |
| `use-top-level-regex` | moved from `@ls-stack/extended-lint` (see below) |
| `no-unused-obj-props` | moved from `@ls-stack/extended-lint`, stricter (see below) |
| `require-type-guard-explanation` | new, a stricter alternative to `no-type-guards` (see below) |

Options are the same as the former ESLint rules' (see `lib/index.d.ts`). Rule names may carry a plugin prefix, so
existing `@ls-stack/strict-casting` configs and disable comments keep working.

### `strict-casting`

Enforces safe and necessary `String()`/`Number()` casts.

Checks performed:

- **Unnecessary casts** (auto-fixable): casting a value that is already of the target type
- **Nullish casts**: `String(value)` produces the strings `"null"`/`"undefined"`, and `Number(value)` produces `0` for `null` and `NaN` for `undefined`. A suggestion is provided to add a `?? ''`/`?? 0` fallback
- **Invalid `Number()` casts**: casting a `symbol` (throws at runtime) or an object without a numeric `valueOf` (always results in `NaN`)
- **`any`/`unknown` casts**: casting untyped values is unsafe, type or narrow the value before casting

Bad:

```ts
declare const str: string
declare const nullable: string | null
declare const obj: { a: string }

String(str) // Error: unnecessary cast
String(nullable) // Error: may produce "null"
Number(obj) // Error: results in NaN
```

Good:

```ts
String(num)
String(nullable ?? '')
Number(date) // Date has a numeric valueOf
```

Options:

```ts
type Options = [
  {
    /** Extra functions checked for unnecessary casts */
    additionalCastFunctions?: { name: string; expectedType: 'string' | 'number' }[]
    /** Allow casting possibly nullish values (default: false) */
    allowNullishCasting?: boolean
    /** Allow symbol/object Number() casts (default: false) */
    allowInvalidCasting?: boolean
    /** Allow casting `any` values (default: false) */
    allowAnyCasting?: boolean
    /** Allow casting `unknown` values, e.g. `String(err)` in catch blocks (default: false) */
    allowUnknownCasting?: boolean
  },
]
```

For `String(obj)` producing `"[object Object]"`, use `@typescript-eslint/no-base-to-string`, which covers that case well.

### `no-duplicate-key`

Based on `@eslint-react/no-duplicate-key` (v2.3.13), with the same message (`A key must be unique. 'key="1"' is
duplicated.`, `noDuplicateKey`) and no options, but stricter:

- Keys of sibling elements (children of one JSX element or fragment, or items of one array literal) are compared by
  static value, as React stringifies keys: `key="a"`, `key='a'`, `key={"a"}`, `` key={`a`} `` and `key={("a")}` are
  all the same key, and so are `key={1}`, `key={1.0}` and `key="1"`. Upstream only compares string literals.
- Other key expressions are duplicates when their source text is the same, ignoring whitespace: `key={item.id}` on
  two siblings is reported. Upstream never compares them.
- Every key that duplicates another key of its group is reported, wherever it is:
  `[<a key="x" />, <b key="x" />, <c key="y" />]` reports the two `x` keys. Upstream only reports a group when its
  last key duplicates an earlier one, and then reports every key of the group, unique ones too.
- In the callback of a `.map()` call (or of a method ending with `Map`, e.g. `flatMap`) or of `Array.from(items,
  callback)`, an element is reported when its key is the same for every item: a static value (string, number or
  template literal without expressions) or an identifier bound to a `const` declared outside the callback. Every such
  key is reported, in all returns and branches. Upstream only checks string literals in `.map()`-like callbacks, and
  only the last key of each call.
- Files without `key=` are skipped.

### `no-restricted-syntax`

Reports code matching [ESQuery](https://github.com/estools/esquery) selectors over an ESTree view of the TS AST, plus
the function-call and per-file checks of the former `@ls-stack/advanced-no-restricted-syntax`.
`advanced-no-restricted-syntax` (with any plugin prefix) is an alias of this rule, in configs and in disable comments,
and so is `eslint-core/no-restricted-syntax` and any other `<prefix>/no-restricted-syntax`. One config block can't
set both names.

```ts
type Options = [
  {
    /** ESLint's `no-restricted-syntax` entries, plus an optional fix */
    disallow?: {
      selector: string
      message: string
      /** Replacement text, or a JS regex replacement of the matched text (`$1`, `$<name>`, `$&`...) */
      replace?: string | { regex: string; with: string }
      /** default: 'suggestion' */
      replaceType?: 'suggestion' | 'autofix'
    }[]
    /** Calls of `fn` (also through import and `const` aliases), optionally only with these literal arguments */
    disallowFnCalls?: {
      fn: string
      withArgs?: { atIndex: number; value: string | number | boolean }[]
      message: string
      /** Autofix replacing the whole call */
      replaceWith?: string
      /** Files to skip */
      ignoreRegex?: string
    }[]
    /**
     * Requirements for files whose name matches `includeRegex`. `$1`, `$1_lowercase`, `$1_capitalize` and
     * `$1_uncapitalize` in the values below are its capture groups.
     */
    mustMatchSyntax?: {
      includeRegex: string
      excludeRegex?: string
      /** At least one call of each `anyCall` list, with these literal arguments */
      mustCallFn?: {
        anyCall: { fn: string; withArgs: { atIndex: number; literal: string | number | boolean }[] }[]
        message?: string
      }[]
      /** At least one node matching the selector */
      mustMatchSelector?: { selector: string; message: string }[]
      /** An export named `name`; type defaults to 'any' */
      mustHaveExport?: { name: string; type?: 'function' | 'variable' | 'any'; message: string }[]
    }[]
  },
]
```

ESLint's array form (`['error', { selector, message }, ...]`) isn't accepted: put the entries in `disallow`. Since an
override that sets options replaces all of them, spread the base options into overrides that add `mustMatchSyntax`,
and turn off one group of checks for some files by setting only the other groups, instead of `'off'`.

Supported selector syntax: node types (`CallExpression`, `*`), attributes (`[attr]`, `[a.b.c="x"]`, `!=`, `=/regex/`,
`!=/regex/`, `=type(string)`, `<`, `<=`, `>`, `>=`), field selectors (`.id`), `:not(...)`, `:matches(...)`/`:is(...)`,
`:function`, the descendant (` `) and child (`>`) combinators and comma lists. Like ESQuery, type names ignore case,
`=` compares the value as a string and regexes only match strings.

These are config errors, reported when the config is loaded:

- sibling combinators (`~`, `+`), subject indicators (`!`), `:has`, `:first-child`, `:last-child`, `:nth-child`,
  `:nth-last-child`, `:statement`, `:expression`, `:declaration`, `:pattern`
- node types and fields the ESTree view doesn't model (see `src/esquery/estree.rs`), and fields set on types that
  don't have them (`Identifier[callee.name="x"]`)
- regexes using JS-only syntax (lookarounds, backreferences)
- selectors whose matched node isn't narrowed to node types (`*`, `:not(Identifier)`; `[name="x"]` alone is fine:
  it narrows to the types with a `name` field)

Selectors are compiled once and indexed by the node type they match: only the TS node kinds that can produce those
types are visited, `Identifier[name="x"]` and `Literal[value="x"]` are hash lookups, and the left side of a descendant
or child combinator is checked only on matching nodes, walking up through ancestors of the possible kinds.

Differences from ESLint:

- `TSTypeAnnotation` isn't modeled, so `returnType` and the `typeAnnotation` of identifiers, parameters and
  properties are config errors (a type alias's or an `as` expression's `typeAnnotation` works).
- A shorthand property's key and value are one node, reported once (ESLint reports both).
- `JSXEmptyExpression`, `MetaProperty.meta`, abstract and `accessor` class members aren't modeled; `TSEnumDeclaration`
  has `members` directly.
- Several `disallow` entries with the same selector each report (the ESLint plugin kept only the last).

### `naming-convention`

Reports declared names: type names (classes, interfaces, type aliases, enums and type parameters) matching `/^[a-z]|_/`,
and values (variables, functions, parameters and destructured names) starting with `__`. Each check's regex and
message can be changed, and each can be turned off with `false`.

```ts
type Check = boolean | { /** Rust regex the name must not match */ disallow?: string; message?: string }

type Options = [
  {
    /** default: { disallow: '^[a-z]|_', message: 'Type names must be PascalCase' } */
    typeNames?: Check
    /** default: { disallow: '^__', message: 'Variable, function and parameter names must not start with `__`' } */
    values?: Check
  },
]
```

It replaces these `no-restricted-syntax` selectors, so `eslint-core/naming-convention` disable comments keep working:

```ts
':matches(ClassDeclaration, ClassExpression, TSInterfaceDeclaration, TSTypeAliasDeclaration, TSEnumDeclaration) > Identifier.id[name=/^[a-z]|_/], TSTypeParameter > Identifier.name[name=/^[a-z]|_/]'
':matches(VariableDeclarator > Identifier.id, :function > Identifier.id, :function > Identifier.params, ObjectPattern > Property > Identifier.value, ArrayPattern > Identifier, AssignmentPattern > Identifier.left, RestElement > Identifier.argument, TSParameterProperty > Identifier.parameter)[name=/^__/]'
```

### `no-default-export`

Disallows `export default` (declarations, expressions and `export { x as default }`), use named exports instead.
`export =` is allowed.

### `no-reexport`

Disallows re-exports: `export ... from`, `export { local }` lists, and exporting an imported binding directly
(`export const x = imported`, `export const y = imported.prop`, `export default imported`,
`export type T = ImportedType`). Use direct exports only.

### `no-relative-imports`

Disallows relative imports (`./` or `../`) and auto-fixes them to the matching alias.

```ts
type Options = [
  {
    /** Path aliases, the first whose `replacement` (relative to `rootDir`) contains the import wins */
    aliases: { find: string; replacement: string }[]
    /** Directory the replacements are relative to (default: the current working directory) */
    rootDir?: string
    /** Allow relative imports that match no alias (default: false) */
    allowNotFoundAliases?: boolean
  },
]
```

```ts
// aliases: [{ find: '@src', replacement: '/src' }], file: src/components/Component.ts
import { Button } from './Button' // fixed to '@src/components/Button'
```

### `no-sync-dynamic-import`

Disallows synchronous `import()` usage (including `import('./x').Type` type imports) and `require()` /
`require.resolve()`. `await import()`, `import().then()`, `() => import()` and `Promise.all([import()])` are allowed.

### `no-leaked-text-in-jsx`

Reports JSX text that looks like leaked code: `,`, `;`, `[`, `]`, `(`, `)`, or text containing `&&`, `||` or ending
with `? (`. Wrap it in an expression container (`{','}`) if it is intended.

### `no-static-style-prop`

Disallows static `style` props (string literals, or object/array literals with only literal values), use CSS instead.
Option `customMessage` is appended to the error message.

### `no-non-camel-case-functions`

Function declarations must be named in camelCase (an optional `$` prefix is allowed). Functions with an explicit
`JSX.Element` return type (or a union containing it) are skipped, so React components can keep PascalCase names.
Arrow functions and function expressions aren't checked. No options.

### `prefer-named-functions`

Reports `const fn = () => {}` declarations without a type annotation and suggests converting them to
`function fn() {}`. Arrow functions with an implicit return are allowed unless `disallowArrowFnWithImplicitReturns` is
set.

```ts
type Options = [
  {
    /** Names matching this regex are allowed to be arrow functions */
    ignoreRegex?: string
    /** Also report arrow functions with an implicit return (default: false) */
    disallowArrowFnWithImplicitReturns?: boolean
  },
]
```

### `no-call-with-explicit-generics`

Disallows explicit type arguments on calls to the configured functions, so their generics are inferred. Entries can be
a function name (`useForm`), a method (`api.fetch`) or a wildcard method (`*.fetch`), optionally with a custom
message.

```ts
type Options = [{ functions: (string | { name: string; message: string })[] }]
```

### `no-call-with-inferred-generics`

Requires the configured functions to be called with explicit type arguments, and can forbid `any`, `unknown`,
`typeof` or specific type names in them.

```ts
type Options = [
  {
    functions: {
      name: string
      /** Minimum number of type arguments (default: 1) */
      minGenerics?: number
      /** Skip the `any`/`unknown`/`typeof` checks unless `disallowTypes` is set */
      allowAny?: boolean
      /** Type names to report as `any`, replaces the global `disallowTypes` */
      disallowTypes?: string[]
      /** Report `typeof x` type arguments, also through local type aliases */
      disallowTypeOf?: boolean
      /** Report `unknown` type arguments */
      disallowUnknown?: boolean
    }[]
    /** Type names to report as `any` for every function */
    disallowTypes?: string[]
  },
]
```

Bad:

```ts
// ["error", { functions: [{ name: "foo" }] }]
foo('bar') // missing generic
foo<any>('bar') // 'any' in generics
```

Good:

```ts
foo<string>('bar')
```

### `no-unnecessary-typing`

Reports type annotations on callback parameters passed as the first argument of array methods, where the type is
inferred, and removes them (auto-fixable).

```ts
type Options = [
  {
    /** Method names to check, or 'array' for find, filter, map, forEach, some, every, reduce, findIndex and sort (default) */
    methods?: string[] | 'array'
  },
]
```

### `no-type-guards`

Disallows type predicates (`value is T`) on function declarations, arrow functions and class methods outside
`*.typeGuards.ts(x)` / `*.type-guards.ts(x)` files. Assertion signatures (`asserts value is T`) are allowed.

```ts
type Options = [
  {
    /** Custom messages for type guards in `.find()`/`.filter()` callbacks */
    alternativeMsgs?: { inArrayFind?: string; inArrayFilter?: string }
  },
]
```

### `use-state-setter-naming`

Requires the setter of `const [value, setter] = useState(...)` (or `React.useState`) to be named `set` + the
capitalized value name. A suggestion renames the setter.

```ts
const [count, setCount] = useState(0) // ok
const [records, setGroups] = useState([]) // Error: expected "setRecords"
```

Option `ignorePrefixes` (default `[]`): prefixes stripped from both names before comparing, so with `['_']`
`const [_value, _setValue]` and `const [_value, setValue]` are accepted.

### `require-lazy-execution`

Reports calls (and tagged templates) of the configured functions at module level: they must run inside a function,
method, getter, setter or constructor. A call's name is its identifier or the property of a member callee
(`i18n.__()` matches `__`).

```ts
// require-lazy-execution: ["error", { functions: ["__"] }]
const label = __('Save') // Error
const getLabel = () => __('Save') // ok
```

Option `functions` (default `[]`). The ESLint rule's `selectors` option (ESQuery selectors) is not supported: a
config using it is rejected, list the function names in `functions` instead.

### `no-unnecessary-iife`

Reports a zero-argument, non-async IIFE without parameters whose body only returns an expression, and autofixes it
to that expression (`(() => { return a.b })()` becomes `a.b`). An IIFE with more statements is reported only when
`return (() => { ... })()` is the whole body of a function, and the fix moves the IIFE body into that function. An
IIFE containing comments is not reported.

### `require-description`

Requires a description (after ` -- `) on directive comments: `eslint-*`, `oxlint-*` and `fast-lint-*` disable/enable
comments, plus `/* eslint */`, `/* eslint-env */`, `/* global(s) */` and `/* exported */`.

```ts
// fast-lint-disable-next-line strict-casting -- the value is checked above
```

Option `ignore`: directive kinds to skip, e.g. `['eslint']` (see `RequireDescriptionDirective` in
`lib/index.d.ts`).

### `no-write-only-ref`

Reports a `useRef` (imported from `react`, or any `X.useRef`) ref whose variable is never used except in `ref={...}`
JSX props: it is written by React but never read.

### `no-commented-out-code`

Reports comments that look like commented-out code. Line comments are matched against code patterns (`const `,
`return `, `if (`, `=>`, `foo.bar(`, ...); block comments only against JSX/HTML tags. JSDoc tags, directives
(`eslint-disable`, `@ts-`, `prettier-ignore`), URLs, descriptive `Header: text` comments and inline code in backticks
surrounded by prose are ignored.

Bad:

```ts
// function foo() {
//   // ...
// }
```

Good:

```ts
// TODO: re-enable once the API is stable
// function foo() {
//   // ...
// }
```

Comments starting with `INFO:`, `TODO:`, `DOCS:`, `FIX:`, `FIXME:`, `HACK:`, `NOTE:`, `WARNING:`, `WARN:`, `BUG:`,
`ISSUE:`, `TEMP:`, `TEMPORARY:`, `XXX:`, `REVIEW:`, `eslint`, `cspell:` or `spell-checker:` are allowed. The rule is
heuristic: it can miss commented code.

### `react-single-export`

In React component files (by default `*.tsx`), a file that exports a component (a PascalCase function, arrow,
`memo`/`forwardRef` call, or a HOC returning JSX) may have no other value export, so fast refresh keeps working. The
first other value export is reported; type-only exports are allowed.

```ts
type Options = [{ /** File name suffixes checked (default: ['tsx']) */ extensions?: string[] }]
```

### `no-unused-optional-args`

Reports optional parameters of non-exported functions that no call passes, and optional props of an object parameter
(or of an `FC<{ ... }>` component's props) that no call or JSX usage provides. Functions passed as an argument,
returned, or used as a JSX attribute value are skipped, as are functions called with spread arguments or JSX spread
attributes.

```ts
function f(a?: string, b?: string) {} // Error: Optional parameter 'b' is never used
f('x')
```

Options:

```ts
type Options = [
  {
    /** Regex of parameter names to ignore */
    ignoreArgsMatching?: string
  },
]
```

### `no-unused-type-props-in-args`

Reports properties of a destructured parameter's object type that aren't destructured, in functions, arrow functions
and `FC<Props>` components. Auto-fixable: the missing properties are added to the destructuring pattern. Types
referenced more than once are ignored, as are exported types referenced from a function's parameters when
`alwaysCheckFunctionOptionTypes` is `false`.

```ts
function foo({ a }: { a: string; b: string }) {} // Error: 'b' is defined but never used
const Foo: FC<{ a: string; b: string }> = ({ a }) => null // Error: 'b' is defined but never used
```

Options:

```ts
type Options = [
  {
    /** Regexes of FC prop type names that are checked even when referenced more than once */
    forceCheckOnFCPropTypesWithName?: string[]
    /** Check exported types used in function parameters (default: true) */
    alwaysCheckFunctionOptionTypes?: boolean
  },
]
```

### `no-unused-t-state-field`

In `.tsx`/`.jsx` files importing `t-state-form`, reports fields declared in the first `useForm({ initialConfig })` that
are never read through the `formFields` of the following `useFormState(...)`. Returning `formFields` skips the check.

### `no-unused-selected-values`

Reports values a configured selector hook selects but the caller never uses, either by destructuring the result or by
reading its properties.

```ts
// selectors: [{ name: 'useSelector' }]
const { a } = useSelector((state) => ({ a: state.a, b: state.b })) // Error: "b" is not being used
```

Options:

```ts
type Options = [
  {
    selectors: {
      /** Name of the selector function (or method) */
      name: string
      /** Property of an options object argument holding the selector function */
      selectorProp?: string
      /** Index of the argument holding the selector (default: 0) */
      selectorArgPos?: number
      /** Property of the result holding the selected values */
      returnProp?: string
    }[]
  },
]
```

### `no-optional-root-props`

Disallows optional properties (`prop?: T`) at the root of a local, non-exported type alias or interface that is
referenced only once, as the parameter type of a function that is itself used once (and not exported), or as the
props of a non-exported `FC<...>` component used once (and not wrapped in `memo`). Inline `FC<{ ... }>` props are
checked the same way. Suggests `prop: undefined | T` instead, so every caller has to pass the prop explicitly.

```ts
type Props = { a?: number } // Error: optional property "a"

function test(arg: Props) {}

console.log(test)
```

### `no-redundant-function-params`

Reports calls passing arguments equal to the configured defaults of a function (matched by its name, or by the
property name of a member call), with an autofix that removes trailing redundant arguments and redundant properties
of object arguments. Only literals, arrays of literals and object literals with literal values are compared.

```ts
// { functions: [{ name: 'inline', defaults: [{ align: 'center', justify: 'left' }] }] }
inline({ align: 'center' }) // Error, fixed to `inline()`
inline({ align: 'center', justify: 'right' }) // Error, fixed to `inline({ justify: 'right' })`
```

Options: `{ functions: { name: string; defaults: unknown[] }[] }`. The fix removes the arguments between the call's
own parentheses, so `fn<T>(a)` and `fn?.(a)` are fixed correctly (the ESLint rule broke them).

### `react-no-module-level-single-use-values`

In `.tsx`/`.jsx` files, reports module-level non-exported variables holding allocated data (array, object and class
expressions, `new Set`/`new Map`) or strings of 80+ characters that are only used inside one top-level React component
(a PascalCase function, or a PascalCase variable holding a function, possibly wrapped in `memo`/`forwardRef`): moving
them inside the component frees their memory when it isn't rendered. Aliases (`a`, `a.b`), functions, regexes and
other class instantiations are ignored.

Options: `{ ignoreConstCaseVarRegex?: string }`: CONSTANT_CASE variables whose name matches the regex are ignored.

### `require-reads-to-var-prop`

Requires reading some properties of the value returned by a call, either through member access (`result.data`) or
destructuring (`const { data } = result`). Passing the variable itself anywhere (`fn(result)`, `{ result }`) counts
as reading every property.

```ts
// { varsToCheck: [{ fromFnCall: 'useActionFn', prop: 'call' }] }
const onConfirm = useActionFn(fn) // Error: "useActionFn" requires reads to the prop "call".
console.log(onConfirm.isInProgress)
```

Options: `{ varsToCheck: { fromFnCall?: string; selector?: string; prop: string | string[]; errorMsg?: string }[] }`.
`fromFnCall` is a function name (`useQuery`), `*.method` or `object.method`. `selector` only supports
`VariableDeclarator` with attribute filters (`VariableDeclarator[init.callee.name="createTest"]`, `!=`, `[path]`,
over `type`, `name`, `id`, `init`, `callee`, `object`, `property`, `computed`, `optional`, `arguments`, `argument`,
`expression` and `value`); other selectors are a config error.

### `prefer-react-hook-alternative`

Flags calls to configured functions made directly inside React components and hooks (PascalCase or `use*` functions,
`memo`/`forwardRef` callbacks, including callbacks passed to hooks and JSX props), with a suggestion to use the hook
alternative instead.

```ts
type Options = [
  {
    disallowedFunctions: {
      /** Function name, matched on the callee or on the member property (`obj.name()`) */
      name: string
      /** Allow calls with arguments other than `undefined` */
      allowUsingWithArgs?: boolean
      /** Hook to use instead, offered as a suggestion */
      hookAlternative?: string
      /** Custom message, replaces `use <hookAlternative> instead` */
      message?: string
      /** Allow calls inside these functions (by declared name or call name), even when nested */
      allowUseInside?: string[]
    }[]
  },
]
```

### `sort-react-components-and-styles`

In `.tsx` files, places the main component before the other components (auto-fixable), and each `styled`/`css`
style above the first component or style that uses it.

The main component is the one selected by `mainComponentSelector`, else the one whose name matches
`mainComponentRegex` (falling back to the exported components when none matches), else the exported ones.

```ts
type Options = [
  {
    /** Regex (Rust `regex` syntax, no look-around) matched against the component names */
    mainComponentRegex?: string
    /**
     * Selector of the main component's declaration. Only `VariableDeclarator`, `FunctionDeclaration` or
     * `ClassDeclaration`, optionally followed by `[id.name='Name']`, are supported
     */
    mainComponentSelector?: string
  },
]
```

### `use-types-directly-above-usage`

Requires top-level type aliases and interfaces to be placed directly above their first usage (auto-fixable).

```ts
type Options = [
  {
    /** Only check usages in these contexts (default: all usages) */
    checkOnly?: ('function-args' | 'FC' | 'generic-args-at-fn-calls')[]
    /** Accepted for compatibility, unused */
    checkTypesFromSelectors?: string[]
  },
]
```

### `react-compiler-extra`

Extra checks for code optimized by the React compiler:

- object methods (`{ method() {} }`) in objects passed to or returned from hooks should be arrow functions
  (auto-fixable unless the method uses `this`)
- React components and hooks should create JSX, call hooks or use the `"use memo"` directive (with a suggestion to
  add it)
- functions calling hooks must be components or hooks, and functions using `"use memo"` must be named like one
- with `disallowManualMemoization`, `useMemo` and `useCallback` calls are reported, as the compiler memoizes for you
- functions starting with the `"use no memo"` directive are skipped (only the nearest enclosing function counts)

```ts
type Options = [
  {
    /** Only run in files with an `eslint react-compiler/react-compiler: ["error"` comment (default: false) */
    runOnlyWithEnableCompilerDirective?: boolean
    /** Report `useMemo` and `useCallback` calls (default: false) */
    disallowManualMemoization?: boolean
  },
]
```

### `no-restricted-comments`

Disallows comments matching configured patterns:

- each pattern matches the comment text (without `//` or `/* */`) by `regex` or, without one, by `includes`
- a comment is reported once, for the first pattern it matches, with the pattern's `message` or
  `Comment matches restricted pattern '<pattern>'`
- the comment's removal is offered as a suggestion, or applied as an autofix with `autoFix: true`

```ts
type Options = [
  {
    patterns: {
      /** Rust `regex` syntax (no lookarounds or backreferences) */
      regex?: string
      includes?: string
      /** Replaces the default message */
      message?: string
      /** Remove the comment as an autofix instead of a suggestion (default: false) */
      autoFix?: boolean
    }[]
  },
]
```

### `require-usage-explanation`

Requires an explanatory comment with a given prefix above matched code:

- calls of a function (`fn`, matching `name()` and `obj.name()`) or nodes matching an ESQuery `selector`
- the comment can be right before the matched node or before any of its ancestors up to the statement holding it
  (e.g. above the `const` or `return` statement, or inside a ternary branch, array or argument list)
- the comment must start with `commentPrefix` and have some text after it (`// WORKAROUND:` alone doesn't count)

```ts
type Options = [
  {
    matches: {
      /** Name of the function whose calls need an explanation */
      fn?: string
      /** ESQuery selector of the nodes that need an explanation */
      selector?: string
      /** Prefix the explanation comment must start with, e.g. `WORKAROUND:` */
      commentPrefix: string
      /** Text appended to the error message */
      message?: string
    }[]
  },
]
```

### `use-top-level-regex`

Requires regex literals to be declared at the top level, so functions don't create a new `RegExp` object on each call.

- regex literals inside functions, arrow functions and methods (including their default parameters) are reported
- regexes with the stateful `g` or `y` flags are allowed, since hoisting them would share `lastIndex` between calls
- regexes passed directly to `new RegExp(...)` are allowed

```ts
const WORD = /\w+/ // ok

function isWord(input: string) {
  return /\w+/.test(input) // Error: move it to the top level
}
```

No options.

### `no-unused-obj-props`

Reports object literal properties that are never read:

- properties of a non-exported `const`/`let` initialized with an object literal (`as const` included) that no
  `obj.key`, `obj['key']`, literal-typed `obj[key]`, destructuring or JSX `<obj.Key />` reads
  (`unusedObjectProperty`); a property that is only assigned (`obj.key = value`) is reported too
- properties of the objects a non-exported local function returns that none of its callers read, when every call
  initializes a variable (`const { a } = useThing()` or `const thing = useThing()` followed by property reads)
  (`unusedReturnObjectProperty`)

The object is skipped whenever it escapes: spread, passed or returned, aliased, enumerated (`in`, `Object.keys`,
`for...in`), typed with `typeof`, exported, reassigned, read with a non-literal key, or when one of its methods,
accessors or function values uses `this`. Accessors, spreads, `__proto__` and dynamic computed keys are never
reported. Typed declarations, `satisfies` and other assertions are skipped.

```ts
const config = { retries: 3, timeout: 100 }
fetchWithRetries(config.retries) // Error: "timeout" is not being used

function useForm() {
  return { values, errors, reset } // Error: "reset" is not used by any caller
}
const { values, errors } = useForm()
```

No options.

### `require-type-guard-explanation`

A stricter alternative to `no-type-guards`: TypeScript doesn't check type guards, so each one must explain why it is
safe and why TS inference or an alternative can't be used.

- every function with a body whose return type is `x is T`, `this is T` or `asserts x is T` (declarations, function
  expressions, arrows, object and class methods, inline callbacks like `.filter((x): x is T => ...)`) needs a
  comment starting with `commentPrefix` and at least `minLength` characters of explanation (`missingExplanation`)
- the comment goes right before the function or before the statement, `export`, class member or object property
  holding it; consecutive `//` lines form one block, and block/JSDoc comments count too
- an explicit predicate TypeScript (5.5+) would infer identically is reported even with a comment, with a suggestion
  to remove the annotation (`inferablePredicate`); only null/undefined and `typeof` checks it is sure about are
  reported
- body-less signatures (overloads, `declare function`, interface members, function types) and `asserts x` are ignored

```ts
// Error: missingExplanation
function isNode(value: unknown): value is Node {
  return isObject(value) && 'kind' in value
}

// ok
// type-guard-safety: values come from our serializer, which always sets `kind`; a zod schema would parse every
// render
function isNode(value: unknown): value is Node {
  return isObject(value) && 'kind' in value
}

// Error: inferablePredicate, TS infers `value is null | undefined`
function isNullish(value: unknown): value is null | undefined {
  return value === null || value === undefined
}
```

```ts
type Options = [
  {
    /** Prefix the explanation comment must start with (default: `type-guard-safety:`) */
    commentPrefix?: string
    /** Minimum characters of explanation after the prefix (default: 20) */
    minLength?: number
  },
]
```

## Usage

```sh
fast-lint                      # every file of ./tsconfig.json
fast-lint src/feature          # only files under these paths
fast-lint --fix                # apply autofixes (strict-casting's unnecessary casts)
fast-lint --format json        # stylish (default), compact or json, in ESLint's formats
fast-lint --max-warnings 0 --timing   # --max-warnings -1: no limit
fast-lint -p packages/app      # a tsconfig, or a directory with a tsconfig.json
fast-lint --color -- -odd-name.ts     # force colors; `--` ends the options
fast-lint --report-unused-disable-directives   # warn about fast-lint-disable comments that suppress nothing
```

A solution-style tsconfig (empty `files` plus `references`) lints its referenced projects; a file in several
projects is linted once.

Paths are printed relative to the current directory. A file with a syntax error gets a single `Parsing error`
instead of rule diagnostics. `--quiet` hides warnings and doesn't apply their fixes; like ESLint, it skips `warn` rules
entirely unless `--max-warnings` is given, in which case warnings still count. `--format json` prints ESLint's JSON
result shape, listing only files with problems or fixes, with the file's `source` when it has problems and wasn't
fixed.

Exit code: `0` clean, `1` errors (or more warnings than `--max-warnings`), `2` fatal error: a tsconfig with a missing
`files` entry or `extends` target, no files to lint, a path argument that matches no project file, a directory
argument whose files are all ignored, or a fixed file that couldn't be written (reported after the results). A file
argument that is ignored, a declaration file or in `node_modules` is skipped with a warning, and so is fixing a file
that isn't UTF-8 (its problems are still reported).

## Config

`fast-lint.config.ts` (or `.mts`/`.mjs`/`.js`, evaluated by the Node launcher) or `fast-lint.config.json`, in the
current directory or passed with `--config`. Without a config the type-aware rules (`improved-no-unnecessary-condition`,
`strict-casting`, `no-unnecessary-void-on-promise` and `no-leaked-conditional-rendering`) run as `error` with default
options; every other rule only runs when configured.

```ts
import { defineConfig } from '@ls-stack/fast-lint'

export default defineConfig({
  projects: ['tsconfig.json'],
  ignores: ['dist/**'],
  rules: {
    'improved-no-unnecessary-condition': ['error', { alternativeForInCondition: 'use objectHasKey' }],
    'strict-casting': ['warn', { additionalCastFunctions: [{ name: 'castToString', expectedType: 'string' }] }],
    'no-unnecessary-void-on-promise': 'error',
  },
  overrides: [
    {
      files: ['src/**'],
      ignores: ['**/*.test.ts'],
      rules: { 'strict-casting': 'error', 'no-leaked-conditional-rendering': 'error' },
    },
  ],
  reportUnusedDisableDirectives: true,
})
```

Overrides apply in order, like ESLint flat config blocks: a severity-only entry keeps the options set by an earlier
block. Globs are relative to the config file's directory (`../` for files outside it). Global `ignores` follow ESLint's
semantics: a pattern matching a directory (`src/gen`, `**/out`, `dist/`, `dist/**`) ignores everything in it, and
`!pattern` un-ignores files ignored by earlier patterns, but not files inside an ignored directory: to keep a file of
an ignored directory, ignore its contents with `dir/**/*` instead. An override's `ignores` also accept `!pattern`,
applied in order. Of the extglobs, only `@(a|b)` is supported (write the others with `{a,b}` or several globs). Rule
names may carry a plugin prefix (`@ls-stack/strict-casting`), but one block can't name a rule twice (two prefixes, or
`no-restricted-syntax` and its alias `advanced-no-restricted-syntax`).

A script config's default export must be the config object, or a promise of it.

## Disable comments

`eslint-disable`, `eslint-enable`, `eslint-disable-line` and `eslint-disable-next-line` comments (and their
`oxlint-` and `fast-lint-` forms) are honored, with or without a rule list and a `-- description`, so existing
directives such as `// eslint-disable-next-line @ls-stack/improved-no-unnecessary-condition` keep working. Rule names
are matched without their prefix, so `eslint-core/no-restricted-syntax` and `eslint-core/naming-convention` (or
`no-restricted-syntax` meant for ESLint core) disable these rules, and so does the former name
`@ls-stack/advanced-no-restricted-syntax`. They are
parsed like ESLint does: the comment text must start with the directive, `disable` and `enable` need a block comment
(`/* */`), and a multi-line `disable-next-line` block comment applies to the line after the comment ends. Inline
config comments (`/* eslint rule: "off" */`) are ignored.

`--report-unused-disable-directives` (or `reportUnusedDisableDirectives: true` in the config) warns about
`fast-lint-disable*` directives that suppressed nothing, like ESLint's option of the same name: a directive without a
rule list when no problem was suppressed, else each listed rule that ran on the file and reported nothing there. Only
`fast-lint-` directives are checked, since `eslint-` and `oxlint-` ones may belong to rules of those linters. The
warnings have no rule id and no fix. Under `--quiet` they are hidden, and they aren't computed unless
`--max-warnings` is given (warn rules don't run then, so their directives would look unused).

### Running alongside ESLint

When these rules are moved out of ESLint (set to `off` there, or their plugin removed), ESLint sees their
`eslint-disable` comments as unused (`reportUnusedDisableDirectives`, a warning by default in ESLint 9), and
`eslint --fix` deletes them, or as errors for an unknown rule when the plugin isn't loaded. Then fast-lint reports the
problems again. Either write these rules' directives as `fast-lint-disable*` (or `oxlint-disable*`), which ESLint
ignores, or set `linterOptions: { reportUnusedDisableDirectives: 'off' }` in the ESLint config, which also stops
reporting directives that are unused for the remaining ESLint rules.

## Differences from ESLint

- `--fix` applies one pass of non-overlapping fixes (ESLint repeats up to 10 passes).
- `improved-no-unnecessary-condition` reports nothing in a project without `strictNullChecks` (or `strict`), where
  every type may be nullish.
- `no-leaked-text-in-jsx` matches the raw JSX text, so HTML entities (`&amp;&amp;`) are not decoded.
- Types come from tsrs (TypeScript 7.1-dev semantics); printed types in messages can differ slightly from TS 5.x.
- Prebuilt binaries ship for macOS (arm64, x64), Linux (x64, arm64; glibc >= 2.17, not musl/Alpine) and Windows
  x64 (also used on Windows arm64, through emulation). Elsewhere, set `FAST_LINT_BINARY` to a `cargo build --release`
  binary. The launcher prints the package version for `--version`; the binary itself prints the crate version.
- The regex options of `no-unused-optional-args` (`ignoreArgsMatching`) and `no-unused-type-props-in-args`
  (`forceCheckOnFCPropTypesWithName`) use Rust `regex` syntax, without lookaround or backreferences.
- The regex options of `no-unused-optional-args` (`ignoreArgsMatching`) and `no-unused-type-props-in-args`
  (`forceCheckOnFCPropTypesWithName`) use Rust `regex` syntax, without lookaround or backreferences.

## Development

```sh
cargo test                     # rule tests (tests/*.rs)
cargo clippy --all-targets
node scripts/build.mjs         # release binaries for every platform into dist/<platform>-<arch>/
node scripts/build.mjs --clean linux-x64 win32-x64   # some platforms, deleting their build dirs afterwards
```

Cross-compiling from macOS needs the Rust targets (`rustup target add x86_64-apple-darwin
x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-msvc`), `zig` + `cargo-zigbuild` for Linux, and
`cargo-xwin` + Homebrew `llvm` (for `llvm-lib`) for Windows. Each platform takes about 2 minutes (fat LTO).

The first build fetches tsrslint's git dependency, including its `typescript-go` submodule (~2 GB, cached in
`~/.cargo/git`). tsrslint is pinned by commit in `Cargo.toml`; `Cargo.lock` pins the tsrs commit it was tested with.
