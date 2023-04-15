# Extended rules for eslint

# `@lucasols/extended-lint/no-unused-type-props-in-args`

Checks if there are unused type props in function arguments.

Examples

```ts
// Bad
function foo({ a }: { a: string; b: string }) {
                              /* ^ b is declared but unused in the function */

  // ...
}

// Good
function foo({ a, b }: { a: string; b: string }) {
  // ...
}
```

It will also work with the react `FC` type:

```ts
// Bad
const Foo: FC<{ a: string; b: string }> = ({ a }) => {
                        /* ^ b is declared but unused in the component */
  // ...
}

// Good
const Foo: FC<{ a: string; b: string }> = ({ a, b }) => {
  // ...
}
```

- This rule will not work in all situations, as it does not checks all possible usages of referenced types.
