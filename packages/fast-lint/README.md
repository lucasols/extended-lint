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
| `no-leaked-conditional-rendering` | `@eslint-react/no-leaked-conditional-rendering` (v2.3.13); option `reactVersion` (default `"19.0.0"`, also used for `""` and `"detect"`: the installed version isn't read) |

Options are the same as the ESLint rules'.

## Usage

```sh
fast-lint                      # every file of ./tsconfig.json
fast-lint src/feature          # only files under these paths
fast-lint --fix                # apply autofixes (strict-casting's unnecessary casts)
fast-lint --format json        # stylish (default), compact or json, in ESLint's formats
fast-lint --max-warnings 0 --timing   # --max-warnings -1: no limit
fast-lint -p packages/app      # a tsconfig, or a directory with a tsconfig.json
fast-lint --color -- -odd-name.ts     # force colors; `--` ends the options
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
semantics: a pattern matching a directory (`src/gen`, `**/out`, `dist/`, `dist/**`) ignores everything in it, and
`!pattern` un-ignores files ignored by earlier patterns, but not files inside an ignored directory: to keep a file of
an ignored directory, ignore its contents with `dir/**/*` instead. An override's `ignores` also accept `!pattern`,
applied in order. Of the extglobs, only `@(a|b)` is supported (write the others with `{a,b}` or several globs). Rule
names may carry a plugin prefix (`@ls-stack/strict-casting`), but not both forms of one rule in one block.

A script config's default export must be the config object, or a promise of it.

## Disable comments

`eslint-disable`, `eslint-enable`, `eslint-disable-line` and `eslint-disable-next-line` comments (and their
`oxlint-` and `fast-lint-` forms) are honored, with or without a rule list and a `-- description`, so existing
directives such as `// eslint-disable-next-line @ls-stack/improved-no-unnecessary-condition` keep working. They are
parsed like ESLint does: the comment text must start with the directive, `disable` and `enable` need a block comment
(`/* */`), and a multi-line `disable-next-line` block comment applies to the line after the comment ends. Inline
config comments (`/* eslint rule: "off" */`) are ignored, and unused directives aren't reported.

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
- Types come from tsrs (TypeScript 7.1-dev semantics); printed types in messages can differ slightly from TS 5.x.
- Prebuilt binaries ship for macOS (arm64, x64), Linux (x64, arm64; glibc >= 2.17, not musl/Alpine) and Windows
  x64 (also used on Windows arm64, through emulation). Elsewhere, set `FAST_LINT_BINARY` to a `cargo build --release`
  binary. The launcher prints the package version for `--version`; the binary itself prints the crate version.

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
