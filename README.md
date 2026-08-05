# Extended rules for eslint

To use any of these rules, you need to install the package:

```sh
pnpm add -D @lucasols/eslint-plugin-extended-lint
```

And then configure it in your eslint config:

```json
{
  "plugins": ["@lucasols/extended-lint"],
  "rules": {
    "@lucasols/extended-lint/no-unused-type-props-in-args": "error"
  }
}
```

## `no-unused-type-props-in-args`

Checks if there are unused type props in function arguments.

Examples

Bad

```ts
function foo({ a }: { a: string; b: string }) {
  /*                             ^ b is declared but not used in the function */
  // ...
}
```

Good

```ts
function foo({ a, b }: { a: string; b: string }) {
  // ...
}
```

It will also work with the react `FC` type:

Bad

```ts
const Foo: FC<{ a: string; b: string }> = ({ a }) => {
  /*                       ^ b is declared but not used in the component */
  // ...
}
```

Good

```
const Foo: FC<{ a: string; b: string }> = ({ a, b }) => {
  // ...
}
```

- This rule will not work in all situations, as it does not checks all possible usages of referenced types.

## `no-commented-out-code`

Checks if there are commented code. Block comments are ignored.

Examples

Bad:

```ts
// function foo() {
//   // ...
// }
```

Good:

```ts
/*
function foo() {
  // ...
}
*/
```

- This rule will not detect all possible commented code, and can give false negatives.

## `no-call-with-infered-generics`

This rule disallows calling functions with inferred generics.

The rule options types are as follows:

````ts
type Options = [
  {
    functions: Array<{
      name: string;
      minGenerics?: number;
      allowAny?: boolean;
      disallowTypes?: string[];
    }>;
    anyAliases?: string[];
  }
];

Examples:

Bad:

```ts
// @typescript-eslint/no-call-with-inferred-generics: ["error", { functions: ["foo"] }]

function foo<T>(arg: T) {
  // ...
}

foo('bar') // Error: Generics must be defined explicitly
foo<any>('bar') // Error: 'any' is not allowed in generics
````

Good:

```ts
// @typescript-eslint/no-call-with-inferred-generics: ["error", { functions: ["foo"] }]
function foo<T>(arg: T) {
  // ...
}

foo<string>('bar') // OK
```

## `rules-of-hooks`

A fork of the `eslint-plugin-react-hooks` `rules-of-hooks` rule, but with the following changes:

- Identifies hooks in camelCase namespaced functions (e.g. `test.useFoo()`)

## `strict-casting`

Enforces safe and necessary `String()`/`Number()` casts. Replaces the deprecated `no-unnecessary-casting` rule.

Checks performed:

- **Unnecessary casts** (auto-fixable): casting a value that is already of the target type
- **Nullish casts**: `String(value)` produces the strings `"null"`/`"undefined"`, and `Number(value)` produces `0` for `null` and `NaN` for `undefined`. A suggestion is provided to add a `?? ''`/`?? 0` fallback
- **Invalid `Number()` casts**: casting a `symbol` (throws at runtime) or an object without a numeric `valueOf` (always results in `NaN`)

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
  },
]
```

For `String(obj)` producing `"[object Object]"`, use `@typescript-eslint/no-base-to-string`, which covers that case well.
