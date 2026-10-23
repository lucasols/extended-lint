// Builds the release binary for every platform into dist/<platform>-<arch>/.
// Usage: node scripts/build.mjs [--clean] [platform...]   (platforms: see `targets`; default all)
// Linux targets need zig + cargo-zigbuild, the Windows target needs cargo-xwin.
import { execFileSync } from 'node:child_process'
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
} from 'node:fs'
import { fileURLToPath } from 'node:url'

const targets = [
  { platform: 'darwin-arm64', triple: 'aarch64-apple-darwin', command: ['build'] },
  { platform: 'darwin-x64', triple: 'x86_64-apple-darwin', command: ['build'] },
  {
    platform: 'linux-x64',
    triple: 'x86_64-unknown-linux-gnu',
    command: ['zigbuild'],
    targetArg: 'x86_64-unknown-linux-gnu.2.17',
  },
  {
    platform: 'linux-arm64',
    triple: 'aarch64-unknown-linux-gnu',
    command: ['zigbuild'],
    targetArg: 'aarch64-unknown-linux-gnu.2.17',
  },
  {
    platform: 'win32-x64',
    triple: 'x86_64-pc-windows-msvc',
    command: ['xwin', 'build'],
    exe: 'fast-lint.exe',
    needsLlvmTools: true,
  },
]

function isOnPath(tool) {
  return (process.env.PATH ?? '')
    .split(':')
    .some((dir) => existsSync(`${dir}/${tool}`))
}

// cc-rs needs `llvm-lib` to archive the C code of the Windows target; Homebrew's llvm is keg-only.
function llvmToolsPath() {
  if (isOnPath('llvm-lib')) return process.env.PATH
  const brew = '/opt/homebrew/opt'
  const keg = existsSync(brew)
    ? readdirSync(brew)
        .filter((name) => /^llvm(@\d+)?$/.test(name))
        .map((name) => `${brew}/${name}/bin`)
        .find((dir) => existsSync(`${dir}/llvm-lib`))
    : undefined
  if (!keg) {
    throw new Error('llvm-lib not found: install llvm (e.g. `brew install llvm`)')
  }
  return `${keg}:${process.env.PATH ?? ''}`
}

const root = fileURLToPath(new URL('..', import.meta.url))
const { version } = JSON.parse(readFileSync(`${root}/package.json`, 'utf8'))

const args = process.argv.slice(2)
const clean = args.includes('--clean')
const requested = args.filter((a) => !a.startsWith('--'))

const unknown = requested.filter((p) => !targets.some((t) => t.platform === p))
if (unknown.length > 0) {
  console.error(
    `unknown platform(s): ${unknown.join(', ')}; available: ${targets.map((t) => t.platform).join(', ')}`,
  )
  process.exit(1)
}

const selected =
  requested.length > 0
    ? targets.filter((t) => requested.includes(t.platform))
    : targets

for (const target of selected) {
  console.log(`\nbuilding fast-lint ${version} for ${target.platform}`)

  execFileSync(
    'cargo',
    [
      ...target.command,
      '--profile',
      'dist',
      '--bin',
      'fast-lint',
      '--target',
      target.targetArg ?? target.triple,
    ],
    {
      cwd: root,
      stdio: 'inherit',
      env: {
        ...process.env,
        FAST_LINT_VERSION: version,
        ...(target.needsLlvmTools ? { PATH: llvmToolsPath() } : {}),
      },
    },
  )

  const exe = target.exe ?? 'fast-lint'
  const outDir = `${root}/dist/${target.platform}`
  mkdirSync(outDir, { recursive: true })
  copyFileSync(`${root}/target/${target.triple}/dist/${exe}`, `${outDir}/${exe}`)
  chmodSync(`${outDir}/${exe}`, 0o755)

  if (clean) {
    rmSync(`${root}/target/${target.triple}`, { recursive: true, force: true })
  }
}

console.log(`\nbuilt ${selected.map((t) => t.platform).join(', ')}`)
