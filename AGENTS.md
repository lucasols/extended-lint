# Project Overview

This is a pnpm monorepo with two linters for TypeScript/JavaScript projects:

- **fast-lint** (`@ls-stack/fast-lint`, `packages/fast-lint`): a Rust linter on tsrs/tsrslint. **This is where rules
  go.** See the "fast-lint (Rust)" section.
- **ESLint plugin** (`@ls-stack/extended-lint`, `packages/extended-lint`): legacy. Its rules must be migrated to
  fast-lint; don't add new rules to it.

**Key Requirements:**

- Always run tests before and after changes: `pnpm test` and `pnpm lint`
- Follow TypeScript strict typing: No `any`, no type assertions, no null assertions
- Use test-driven development: Write tests first, then implement
- Performance matters: These rules run on large codebases

# Commands

Run from the repo root (applies to all packages):

- `pnpm test` - Run all tests (Vitest)
- `pnpm lint` - TypeScript compiler + ESLint check (MUST pass)

Run inside `packages/extended-lint` (or from the root with `pnpm --filter @ls-stack/extended-lint <script>`):

- `pnpm test <rule-name>` - Run specific rule tests
- `pnpm tsc` - Type check only

# Testing

- Use Vitest framework
- Tests: `packages/extended-lint/src/rules/*.test.ts`
- Always use `createNewTester` from `@packages/extended-lint/tests/utils/createTester.ts`
- Write tests BEFORE implementing rules
- Use describe only for grouping tests
- Trailing or leading empty lines in output are not a issue as the code will be formatted by the user, so don't worry about it. Handling it in the rule will make the rule code more complex, so don't do it.

## Test Pattern

```typescript
import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { ruleFunction } from './rule-name'

const { valid, invalid } = createNewTester(ruleFunction)

// Valid tests - no errors expected
test('valid case description', async () => {
  await valid(
    dedent`
      type UserData = { name: string }
      
      function processUser(data: UserData) {
        return data.name
      }
    `,
  )
})

// Invalid tests - expect errors and check output
test('invalid case description', async () => {
  const { result } = await invalid(dedent`
    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }

    export type ProduceRecipe<T> = (draft: T) => void | undefined | T;
  `)

  // use getErrorsWithMsgFromResult(result) to include final message in the snapshot when there is dynamic data in the message
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "
  `)

  // use result.output when there is auto fixable code
  expect(result.output).toMatchInlineSnapshot(`
    "export type ProduceRecipe<T> = (draft: T) => void | undefined | T;

    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }
    "
  `)
})

test('valid example with options', async () => {
  await valid({
    code: dedent`
      const config: Config = { debug: true }
    `,
    options: [{ checkOnly: ['FC'] }],
    filename: 'config.ts', // use only if needed
  })
})

test('invalid example with options', async () => {
  const { result } = await invalid({
    code: dedent`
      const config: Config = { debug: true }
    `,
    options: [{ checkOnly: ['FC'] }],
    filename: 'config.ts', // use only if needed
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(/* ... */)
  expect(result.output).toMatchInlineSnapshot(/* ... */)
})
```

- Dont use `expect(result.output).toMatchInlineSnapshot(...)` if the test do not have auto fixable code.
- For code with fix suggestions, use `expect(getSuggestionOutput(result)).toMatchInlineSnapshot(...)` to check the fix.

# ESLint plugin architecture (legacy)

This and the next sections apply only to maintaining the ESLint plugin until its rules are migrated to fast-lint.
All paths below are relative to `packages/extended-lint`.

- **Entry point**: `src/extended-lint.ts`
- **Rule creation**: Use `createExtendedLintRule` from `@packages/extended-lint/src/createRule.ts`
- **Rules registry**: `src/rules/rules.ts`
- **Rule files**: `src/rules/rule-name.ts` + `src/rules/rule-name.test.ts`

# Rule Development Pattern

**Workflow:**

1. Create test file: `src/rules/rule-name.test.ts`
2. Implement rule: `src/rules/rule-name.ts` using `createExtendedLintRule`
3. Register in `src/rules/rules.ts`
4. Test: `pnpm test rule-name`

## Rule Example

```typescript
// src/rules/rule-name.ts
import { z } from 'zod/v4'
import { createExtendedLintRule, getJsonSchemaFromZod } from '../createRule'

const optionsSchema = z.object({
  option: z.boolean().optional(),
})

type Options = z.infer<typeof optionsSchema>

export const ruleName = createExtendedLintRule<[Options], 'errorId'>({
  name: 'rule-name',
  meta: {
    type: 'problem',
    docs: { description: 'Rule description' },
    schema: getJsonSchemaFromZod(optionsSchema),
    messages: { errorId: 'Error message' },
  },
  defaultOptions: [{ option: false }],
  create(context, [options]) {
    return {
      // Rule implementation
    }
  },
})
```

# Code Style

- Never add code comments
- Use `type` instead of `interface`
- Use `for of` instead of `array.forEach`
- NEVER use `array.reduce`
- Never use `as` assertions except `as const`
- No `any`, no non-null assertions (`!`), no `@ts-ignore`

# Performance

The rules will be used in large codebases, so performance should be a priority.

- Use early returns when rule doesn't apply
- Target specific AST node types only
- Use Maps/Sets for lookups
- Minimize expensive operations (regex, file system)

# Code maintainability and readability

Extract and reuse common utility functions in `packages/fast-lint/src/utils.rs`

# Fixing @typescript-eslint/no-unnecessary-condition

Common when working with AST nodes. **Fix strategy: Remove unnecessary checks - trust TypeScript and eslint.** If TypeScript doesn't error, the check is unnecessary

## Quick Fixes

```typescript
// ❌ Remove unnecessary optional chaining
node.parent?.type → node.parent.type

// ❌ Remove unnecessary conditionals
if (alwaysTrue && node.parent.type) { ... }
should be:
if (node.parent.type) { ... }

// ❌ Double check impossible conditions
if (alwaysFalse && node.parent.type) { ... }
// in this case check if the condition is really necessary or the code has a bug
```

# Best Practices

- For analyzing code usage/references use eslint scope analysis utilities

# fast-lint (Rust)

Rules must be migrated to `packages/fast-lint` (see its README); the ESLint plugin is being phased out.

- **New rules** are written in fast-lint only, never in the ESLint plugin.
- **Existing ESLint rules** are migrated instead of extended: when a rule in `packages/extended-lint` needs a change,
  port it to fast-lint first, then make the change there.
- **Migrating a rule:**
  1. Port the rule to `packages/fast-lint/src/rules/<rule_name>.rs` and register it in `src/rules/mod.rs`.
  2. Port every TS test case to `packages/fast-lint/tests/<rule_name>.rs`, with the same messageIds and lines.
  3. Add its options type to `lib/index.d.ts` and document it in the fast-lint README.
  4. Delete the TS rule, its tests, its `rules.ts` entry and its README section, plus helpers only it used.
- Already migrated: `improved-no-unnecessary-condition`, `strict-casting` (also replaces the removed
  `no-unnecessary-casting`), `no-unnecessary-void-on-promise`, `no-restricted-syntax` (the former
  `advanced-no-restricted-syntax`, merged with ESLint's `no-restricted-syntax`; the old name is an alias),
  `no-default-export`, `no-reexport`, `no-relative-imports`, `no-sync-dynamic-import`, `no-leaked-text-in-jsx`,
  `no-static-style-prop`, `no-non-camel-case-functions`, `prefer-named-functions`, `no-call-with-explicit-generics`,
  `no-call-with-inferred-generics`, `no-unnecessary-typing`, `no-type-guards`, `use-state-setter-naming`,
  `require-lazy-execution`, `no-unnecessary-iife`, `require-description`, `no-write-only-ref`,
  `no-commented-out-code`, `react-single-export`, `no-unused-optional-args`, `no-unused-type-props-in-args`,
  `no-unused-t-state-field`, `no-unused-selected-values`, `no-optional-root-props`, `no-redundant-function-params`,
  `react-no-module-level-single-use-values`, `require-reads-to-var-prop`, `prefer-react-hook-alternative`,
  `sort-react-components-and-styles`, `use-types-directly-above-usage`, `react-compiler-extra`.
- New in fast-lint: `naming-convention` (replaces an ESLint core `no-restricted-syntax` config with naming selectors).
- ESLint scope analysis (references, declared variables): use `src/scope.rs` (`ReferenceIndex`, `declared_names`,
  `resolve_reference`) instead of rule-specific symbol walks.
- Rule names may carry a plugin prefix (`@ls-stack/<rule>`), so existing configs and disable comments keep working.
- Tests: `packages/fast-lint/tests/<rule_name>.rs`, via `tests/common/mod.rs` (`valid`, `invalid`, insta snapshots)
- Run: `cargo test --test <rule_name>` and `cargo clippy --all-targets` inside `packages/fast-lint`
- Build with `cargo test`/`cargo clippy`, not a plain `cargo build` (which rebuilds every tsrs crate with different
  features)
- ESTree vs TS AST: skip parenthesized expressions (`utils::skip_parens`) and mirror ESTree `ChainExpression`
  wrapping of optional chains (`utils::is_estree_chain_expression`)
