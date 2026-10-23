# @ls-stack/fast-lint

The type-aware rules of [`@ls-stack/extended-lint`](../extended-lint) (plus a port of eslint-react's
`no-leaked-conditional-rendering`), running natively on [tsrs](https://github.com/maschwenk/tsrs), the Rust port of
the TypeScript 7 checker, through [tsrslint](https://github.com/maschwenk/tsrslint)'s rule framework. No ESLint, no
Node type checker: only these rules run, against one shared program per tsconfig.

## Rules

| rule | ported from |
| --- | --- |
| `improved-no-unnecessary-condition` | `@ls-stack/improved-no-unnecessary-condition` |
| `strict-casting` | `@ls-stack/strict-casting` |
| `no-unnecessary-void-on-promise` | `@ls-stack/no-unnecessary-void-on-promise` |
| `no-leaked-conditional-rendering` | `@eslint-react/no-leaked-conditional-rendering` (v2.3.13); option `reactVersion` (default `"19.0.0"`) |

Options are the same as the ESLint rules'.

## Usage

```sh
fast-lint                      # every file of ./tsconfig.json
fast-lint src/feature          # only files under these paths
fast-lint --fix                # apply autofixes (strict-casting's unnecessary casts)
fast-lint --format json        # stylish (default), compact or json
fast-lint --max-warnings 0 --timing
```

A solution-style tsconfig (empty `files` plus `references`) lints its referenced projects; a file in several
projects is linted once.

Paths are printed relative to the current directory. A file with a syntax error gets a single `Parsing error`
instead of rule diagnostics. `--quiet` hides warnings and doesn't apply their fixes; like ESLint, it skips `warn` rules
entirely unless `--max-warnings` is given, in which case warnings still count. `--format json` prints ESLint's JSON
result shape, listing only files with problems or fixes.

Exit code: `0` clean, `1` errors (or more warnings than `--max-warnings`), `2` fatal error: a tsconfig with a missing
`files` entry or `extends` target, no files to lint, a path argument that matches no project file, or a directory
argument whose files are all ignored. A file argument that is ignored, a declaration file or in `node_modules` is
skipped with a warning.

## Config

`fast-lint.config.ts` (or `.mts`/`.mjs`/`.js`, evaluated by the Node launcher) or `fast-lint.config.json`, in the
current directory or passed with `--config`. Without a config every rule runs as `error` with default options.

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
})
```

Overrides apply in order, like ESLint flat config blocks: a severity-only entry keeps the options set by an earlier
block. Globs are relative to the config file's directory (`../` for files outside it). Global `ignores` follow ESLint's
semantics: a pattern matching a directory (`src/gen`, `**/out`, `dist/`) ignores everything in it, and `!pattern`
un-ignores files ignored by earlier patterns, but not files inside an ignored directory. Rule names may carry a plugin
prefix (`@ls-stack/strict-casting`).

## Disable comments

`eslint-disable`, `eslint-enable`, `eslint-disable-line` and `eslint-disable-next-line` comments (and their
`oxlint-` forms) are honored, with or without a rule list and a `-- description`, so existing directives such as
`// eslint-disable-next-line @ls-stack/improved-no-unnecessary-condition` keep working. They are parsed like ESLint
does: the comment text must start with the directive, `disable` and `enable` need a block comment (`/* */`), and a
multi-line `disable-next-line` block comment applies to the line after the comment ends.

## Differences from ESLint

- `--fix` applies one pass of non-overlapping fixes (ESLint repeats up to 10 passes).
- Types come from tsrs (TypeScript 7.1-dev semantics); printed types in messages can differ slightly from TS 5.x.
- Prebuilt binaries ship for macOS (arm64, x64), Linux (x64, arm64; glibc >= 2.17, not musl/Alpine) and Windows
  x64. Elsewhere, set `FAST_LINT_BINARY` to a `cargo build --release` binary.

## Development

```sh
cargo test                     # rule tests (tests/*.rs), ported from the TS rule tests
cargo clippy --all-targets
node scripts/build.mjs         # release binaries for every platform into dist/<platform>-<arch>/
node scripts/build.mjs --clean linux-x64 win32-x64   # some platforms, deleting their build dirs afterwards
```

Cross-compiling from macOS needs the Rust targets (`rustup target add x86_64-apple-darwin
x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-msvc`), `zig` + `cargo-zigbuild` for Linux, and
`cargo-xwin` + Homebrew `llvm` (for `llvm-lib`) for Windows. Each platform takes about 2 minutes (fat LTO).

The first build fetches tsrslint's git dependency, including its `typescript-go` submodule (~2 GB, cached in
`~/.cargo/git`). tsrslint is pinned by commit in `Cargo.toml`; `Cargo.lock` pins the tsrs commit it was tested with.
