#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { existsSync } from 'node:fs'
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
  const path = fileURLToPath(
    new URL(`../dist/${platform}/${exe}`, import.meta.url),
  )

  if (!existsSync(path)) {
    throw new Error(
      `no prebuilt binary for ${platform}. Build one with \`cargo build --release\` in packages/fast-lint and point FAST_LINT_BINARY at it`,
    )
  }

  return path
}

function findConfigFlag(args) {
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    if (arg === undefined) continue

    if (arg === '-c' || arg === '--config') {
      return { index: i, length: 2, path: args[i + 1] }
    }

    if (arg.startsWith('--config=')) {
      return { index: i, length: 1, path: arg.slice('--config='.length) }
    }
  }

  return undefined
}

function isScriptConfig(path) {
  return scriptConfigExtensions.some((ext) => path.endsWith(ext))
}

async function loadScriptConfig(path) {
  const module = await import(pathToFileURL(path).href)
  const config = module.default ?? module

  return JSON.stringify(config)
}

async function resolveArgs(args) {
  if (args.includes('--config-json')) return args

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

let exe
let args

try {
  exe = binaryPath()
  args = await resolveArgs(process.argv.slice(2))
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
  console.error(`fast-lint: failed to run ${exe}: ${error.message}`)
  process.exit(2)
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 2)
})
