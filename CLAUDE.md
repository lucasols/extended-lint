# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

- **Test**: `pnpm test` (vitest)
- **Lint**: `pnpm lint` (TypeScript compiler check)
- **Type check**: `pnpm tsc`

## Testing

- Use Vitest for testing
- Tests are located in `src/**/*.test.ts` and `tests/*.test.js`
- Use the `createTester` utility from @tests/utils/createTester.ts for rule testing
- Test setup is in `tests/fixture/setup.ts`
- Single rule tests can be run with `pnpm test <rule-name>`
- Use `__dev_simulateFileName` for simulating file names if needed
- Use the same test structure of `src/rules/react-compiler-extra.test.ts`, using `tests.addValid`, `tests.addInvalid` and `tests.addInvalidWithOptions` if the rule has options
- Prefer inline test configurations directly in the test calls instead of creating separate config objects. Prefer to abstract the config elements instead.

## Architecture

This is an ESLint plugin that provides extended linting rules for TypeScript/JavaScript projects.

### Core Structure

- **Entry point**: `src/extended-lint.ts` exports the plugin
- **Rule creation**: Use `createExtendedLintRule` from @src/createRule.ts instead of direct ESLint rule creation
- **Rules registry**: All rules are exported from `src/rules/rules.ts`
- **Individual rules**: Each rule is in `src/rules/` with corresponding `.test.ts` file

### Rule Development Pattern

1. Start by writing tests first using the `createTester` utility
2. Follow the structure of existing rules like `react-compiler-extra.ts`
3. Use `createExtendedLintRule` from `createRule.ts`
4. Add the rule to the exports in `src/rules/rules.ts`
5. Use `zod/v4` for options with `getJsonSchemaFromZod`

### Build System

- Uses `tsup` for building with both ESM and CJS formats
- Entry point: `src/extended-lint.ts`
- Output: `dist/` directory
- TypeScript configuration: `tsconfig.prod.json` for production builds

### Code Style Guidelines

- Never add comments to code (enforced by Cursor rules)
- Use `type` instead of `interface` when possible
- Use `for of` instead of `array.forEach`
- Never use `array.reduce`
- Never use `as` assertions except for `as const`
- Write strongly typed TypeScript

## Typesafety

- Do not use `any`
- Do not use `as Type` casts, except for `as const`
- Do not use non-null assertions (`!`)

## Performance Considerations

- Prefer using ast selector when using it will make the rule more optimized
- Write with performance in mind, the rules should be fast and efficient as they will be used in large codebases
