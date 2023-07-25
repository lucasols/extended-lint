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
