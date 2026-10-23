#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { constants } from 'node:os'
import { dirname, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const scriptConfigNames = [
  'fast-lint.config.ts',
  'fast-lint.config.mts',
  'fast-lint.config.mjs',
  'fast-lint.config.js',
]

const scriptConfigExtensions = ['.ts', '.mts', '.cts', '.mjs', '.cjs', '.js']

function binaryPath() {
  if (process.env.FAST_LINT_BINARY) return process.env.FAST_LINT_BINARY

  const platform = `${process.platform}-${process.arch}`
  const exe = process.platform === 'win32' ? 'fast-lint.exe' : 'fast-lint'
  const candidates =
    platform === 'win32-arm64' ? [platform, 'win32-x64'] : [platform]

  for (const candidate of candidates) {
    const path = fileURLToPath(
      new URL(`../dist/${candidate}/${exe}`, import.meta.url),
    )
    if (existsSync(path)) return path
  }

  throw new Error(
    `no prebuilt binary for ${platform}. Build one with \`cargo build --release\` in packages/fast-lint and point FAST_LINT_BINARY at it`,
  )
}

function findConfigFlag(args) {
  let found

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    if (arg === undefined) continue

    if (arg === '-c' || arg === '--config') {
      found = { index: i, length: 2, path: args[i + 1] }
      i++
    } else if (arg.startsWith('--config=')) {
      found = { index: i, length: 1, path: arg.slice('--config='.length) }
    }
  }

  return found
}

function isScriptConfig(path) {
  return scriptConfigExtensions.some((ext) => path.endsWith(ext))
}

async function loadScriptConfig(path) {
  const module = await import(pathToFileURL(path).href)

  if (!('default' in module)) {
    throw new Error(`${path} has no default export`)
  }

  const config = await module.default

  if (typeof config !== 'object' || config === null || Array.isArray(config)) {
    throw new Error(
      `${path}: the default export must be a config object or a promise of one`,
    )
  }

  return JSON.stringify(config)
}

function hasInlineConfig(args) {
  return args.some(
    (arg) => arg === '--config-json' || arg.startsWith('--config-json='),
  )
}

function packageVersion() {
  const packageJson = JSON.parse(
    readFileSync(new URL('../package.json', import.meta.url), 'utf8'),
  )
  return packageJson.version
}

async function resolveArgs(args) {
  if (hasInlineConfig(args)) return args

  const flag = findConfigFlag(args)

  if (flag) {
    if (!flag.path || !isScriptConfig(flag.path)) return args

    const path = resolve(flag.path)
    const rest = [...args]
    rest.splice(flag.index, flag.length)

    return [
      ...rest,
      '--config-json',
      await loadScriptConfig(path),
      '--config-root',
      dirname(path),
    ]
  }

  const found = scriptConfigNames
    .map((name) => resolve(name))
    .find((path) => existsSync(path))

  if (!found) return args

  return [
    ...args,
    '--config-json',
    await loadScriptConfig(found),
    '--config-root',
    dirname(found),
  ]
}

const rawArgs = process.argv.slice(2)

if (rawArgs.includes('-v') || rawArgs.includes('--version')) {
  console.log(packageVersion())
  process.exit(0)
}

const isHelp = rawArgs.includes('-h') || rawArgs.includes('--help')

let exe
let args

try {
  exe = binaryPath()
  args = isHelp ? rawArgs : await resolveArgs(rawArgs)
} catch (error) {
  console.error(
    `fast-lint: ${error instanceof Error ? error.message : String(error)}`,
  )
  process.exit(2)
}

const child = spawn(exe, args, { stdio: 'inherit' })

const signals = ['SIGINT', 'SIGTERM', 'SIGHUP']

for (const signal of signals) {
  process.on(signal, () => child.kill(signal))
}

child.on('error', (error) => {
  const hint =
    process.platform === 'linux' &&
    !process.env.FAST_LINT_BINARY &&
    'code' in error &&
    error.code === 'ENOENT'
      ? ' (the prebuilt Linux binaries need glibc >= 2.17; on musl/Alpine point FAST_LINT_BINARY at a binary built there)'
      : ''
  console.error(`fast-lint: failed to run ${exe}: ${error.message}${hint}`)
  process.exit(2)
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.exitCode = 128 + (constants.signals[signal] ?? 0)

    if (signals.includes(signal)) {
      process.removeAllListeners(signal)
      process.kill(process.pid, signal)
      return
    }

    process.exit()
  }

  process.exit(code ?? 2)
})
