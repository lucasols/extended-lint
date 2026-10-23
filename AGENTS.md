# Project Overview

This is a pnpm monorepo for **fast-lint** (`@ls-stack/fast-lint`, `packages/fast-lint`): a Rust linter on
tsrs/tsrslint with type-aware lint rules for TypeScript and React projects (see its README).

The `@ls-stack/extended-lint` ESLint plugin that used to live here was retired: its rules were moved to fast-lint or
removed. All rules are written in fast-lint.

**Key Requirements:**

- Always run tests before and after changes: `pnpm test` and `pnpm lint`
- Use test-driven development: Write tests first, then implement
- Performance matters: These rules run on large codebases

# Commands

Run from the repo root:

- `pnpm test` - Run all tests (`cargo test`)
- `pnpm lint` - `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` (MUST pass)

Run inside `packages/fast-lint`:

- `cargo test --test <rule_name>` - Run specific rule tests
- `cargo clippy --all-targets` - Lint
- Build with `cargo test`/`cargo clippy`, not a plain `cargo build` (which rebuilds every tsrs crate with different
  features). To run the CLI on a real project, use `cargo build --release --bin fast-lint`.

# Rule Development

1. Write the tests first: `packages/fast-lint/tests/<rule_name>.rs`, via `tests/common/mod.rs` (`valid`, `invalid`,
   insta inline snapshots of `errors()`/`errors_with_msg()`, `output` for fixes and suggestions)
2. Implement the rule in `packages/fast-lint/src/rules/<rule_name>.rs` and register it in `src/rules/mod.rs`
3. Add its options type to `lib/index.d.ts` and document it in the fast-lint README
4. Run `cargo test --test <rule_name>` and `cargo clippy --all-targets`

- Rule names may carry a plugin prefix (`@ls-stack/<rule>`), so configs and disable comments from the former ESLint
  plugin keep working.
- Trailing or leading empty lines in fix output are not an issue, as the code will be formatted by the user. Don't
  add handling for them.

# Code maintainability and readability

- Extract and reuse common utility functions in `packages/fast-lint/src/utils.rs`
- Scope analysis (references, declared variables): use `src/scope.rs` (`ReferenceIndex`, `declared_names`,
  `resolve_reference`) instead of rule-specific symbol walks
- ESTree vs TS AST: rules mirror ESLint's ESTree semantics. Skip parenthesized expressions (`utils::skip_parens`) and
  mirror ESTree `ChainExpression` wrapping of optional chains (`utils::is_estree_chain_expression`)
- Selectors (`no-restricted-syntax`, `require-usage-explanation`) go through the ESQuery engine in `src/esquery`

# Performance

The rules will be used in large codebases, so performance should be a priority.

- Use early returns when rule doesn't apply
- Listen to specific node kinds only
- Use maps/sets for lookups
- Minimize expensive operations (regex, file system, checker queries)
