# Project Overview

This is an **ESLint plugin** (`extended-lint`) that provides custom linting rules for TypeScript/JavaScript projects.

**Key Requirements:**

- Always run tests before and after changes: `pnpm test` and `pnpm lint`
- Follow TypeScript strict typing: No `any`, no type assertions, no null assertions
- Use test-driven development: Write tests first, then implement
- Performance matters: These rules run on large codebases

# Commands

- `pnpm test` - Run all tests (Vitest)
- `pnpm test <rule-name>` - Run specific rule tests
- `pnpm lint` - TypeScript compiler check (MUST pass)
- `pnpm tsc` - Type check only

# Testing

- Use Vitest framework
- Tests: `src/rules/*.test.ts`
- Always use `createTester` from `@tests/utils/createTester.ts`
- Write tests BEFORE implementing rules
- Use `_dev_simulateFileName` for file-path dependent rules
- Methods: `tests.addValid()`, `tests.addInvalid()`, `tests.addInvalidWithOptions()`, `tests.run()`
- No describe blocks for simple files
- Tests will have the code lines in which the test is located in the test result, e.g. `1: Test name`
- Trailing or leading empty lines in output are not a issue as the code will be formatted by the user, so don't worry about it. Juste use `appendToOutput` or `prependToOutput` to add newlines to the output if needed. DON'T change the rule code to handle this.

# Architecture

- **Entry point**: `src/extended-lint.ts`
- **Rule creation**: Use `createExtendedLintRule` from `@src/createRule.ts`
- **Rules registry**: `src/rules/rules.ts` - add all new rules here
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

Extract and reuse common utility functions in @src/astUtils.ts
