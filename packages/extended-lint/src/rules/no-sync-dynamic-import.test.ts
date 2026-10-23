import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { noSyncDynamicImport } from './no-sync-dynamic-import'

const { valid, invalid } = createNewTester(noSyncDynamicImport)

test('allows regular ES6 imports', async () => {
  await valid(
    dedent`
      import { something } from './module'
      import type { SomeType } from './types'
      import * as utils from './utils'
      import defaultExport from './default'
    `,
  )
})

test('allows async dynamic imports with await', async () => {
  await valid(
    dedent`
      async function loadModule() {
        const module = await import('./dynamic-module')
        return module.default
      }
    `,
  )
})

test('allows dynamic imports with Promise chaining', async () => {
  await valid(
    dedent`
      function loadModule() {
        return import('./dynamic-module').then(module => module.default)
      }
    `,
  )
})

test('allows dynamic imports in Promise.all', async () => {
  await valid(
    dedent`
      const modules = await Promise.all([
        import('./module1'),
        import('./module2')
      ])
    `,
  )
})

test('disallows synchronous dynamic import in type annotation', async () => {
  const { result } = await invalid(dedent`
    type GetUserPermissionsOptions = {
      planId: import('@src/config/plansCfg').NormalizedPlansIds;
    }
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'syncDynamicImport', line: 2 }
    "
  `)
})

test('disallows synchronous dynamic import in variable assignment', async () => {
  const { result } = await invalid(dedent`
    const config = import('./config')
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'syncDynamicImport', line: 1 }
    "
  `)
})

test('disallows synchronous dynamic import in function return', async () => {
  const { result } = await invalid(dedent`
    function getConfig() {
      return import('./config')
    }
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'syncDynamicImport', line: 2 }
    "
  `)
})

test('disallows synchronous dynamic import in object property', async () => {
  const { result } = await invalid(dedent`
    const obj = {
      config: import('./config')
    }
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'syncDynamicImport', line: 2 }
    "
  `)
})

test('disallows require statements', async () => {
  const { result } = await invalid(dedent`
    const fs = require('fs')
    const path = require('path')
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'requireNotAllowed', line: 1 }
    - { messageId: 'requireNotAllowed', line: 2 }
    "
  `)
})

test('disallows require.resolve', async () => {
  const { result } = await invalid(dedent`
    const modulePath = require.resolve('some-module')
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'requireNotAllowed', line: 1 }
    "
  `)
})

test('disallows require with destructuring', async () => {
  const { result } = await invalid(dedent`
    const { readFile } = require('fs')
    const { join } = require('path')
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'requireNotAllowed', line: 1 }
    - { messageId: 'requireNotAllowed', line: 2 }
    "
  `)
})

test('disallows require in conditional context', async () => {
  const { result } = await invalid(dedent`
    if (condition) {
      const module = require('./module')
    }
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'requireNotAllowed', line: 2 }
    "
  `)
})

test('allows nested await import', async () => {
  await valid(
    dedent`
      async function loadModules() {
        const modules = await Promise.all([
          import('./module1'),
          import('./module2')
        ])

        for (const moduleName of moduleNames) {
          await import(\`./\${moduleName}\`)
        }
      }
    `,
  )
})

test('allows dynamic import as arrow function return', async () => {
  await valid(
    dedent`
      const i18n = i18ninitialize({
        ackLocale: 'en',
        es: {
          d: 'en', 
          translations: () => import('@src/i18n/en-US.json')
        },
        pt: {
          d: 'pt', 
          translations: () => import('@src/i18n/pt-BR.json')
        }
      })
    `,
  )
})

test('disallows dynamic import in regular function return but allows arrow function', async () => {
  await valid(
    dedent`
      // This should be allowed - arrow function returning import
      const getLoader = () => import('./loader')
      
      // This should be allowed - arrow function in object
      const config = {
        loader: () => import('./config-loader')
      }
    `,
  )
})

test('disallows complex synchronous dynamic import usage', async () => {
  const { result } = await invalid(dedent`
    type ApiProfile = import('./api-types').Profile
    
    type GetUserPermissionsOptions = {
      profile: Partial<ApiProfile> | null;
      sessionType?: 'user' | 'public';
      planId: import('@src/config/plansCfg').NormalizedPlansIds;
    }
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'syncDynamicImport', line: 1 }
    - { messageId: 'syncDynamicImport', line: 6 }
    "
  `)
})
