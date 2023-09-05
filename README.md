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
