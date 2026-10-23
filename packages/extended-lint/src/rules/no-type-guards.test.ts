import { createTester } from '../../tests/utils/createTester'
import { noTypeGuards } from './no-type-guards'

const { addValid, addInvalid, addInvalidWithOptions, run } = createTester(
  noTypeGuards,
  {
    defaultErrorId: 'typeGuardNotAllowed',
  },
)

addValid(
  'allows type guard in typeGuards file',
  `
    function isString(value: unknown): value is string {
      return typeof value === 'string'
    }
  `,
  { __dev_simulateFileName: 'utils.typeGuards.ts' },
)

addValid(
  'allows type guard in typeGuards tsx file',
  `
    function isNumber(value: unknown): value is number {
      return typeof value === 'number'
    }
  `,
  { __dev_simulateFileName: 'helpers.typeGuards.tsx' },
)

addValid(
  'allows normal function without type guard',
  `
    function getString(value: unknown): string {
      return String(value)
    }
  `,
)

addValid(
  'allows function with regular return type annotation',
  `
    function process(): boolean {
      return true
    }
  `,
)

addInvalid(
  'disallows type guard in regular file',
  `
    function isString(value: unknown): value is string {
      return typeof value === 'string'
    }
  `,
  'default-error',
)

addInvalid(
  'disallows type guard in method',
  `
    class Validator {
      isString(value: unknown): value is string {
        return typeof value === 'string'
      }
    }
  `,
  'default-error',
)

addInvalid(
  'disallows type guard in arrow function',
  `
    const isNumber = (value: unknown): value is number => typeof value === 'number'
  `,
  'default-error',
)

addInvalid(
  'disallows multiple type guards',
  `
    function isString(value: unknown): value is string {
      return typeof value === 'string'
    }
    
    function isNumber(value: unknown): value is number {
      return typeof value === 'number'
    }
  `,
  2,
)

addValid(
  'allows type guard in type-guards file',
  `
    function isBoolean(value: unknown): value is boolean {
      return typeof value === 'boolean'
    }
  `,
  { __dev_simulateFileName: 'utils.type-guards.ts' },
)

addValid(
  'allows type guard when suggestions disabled',
  `
    const items = ['a', 1, 'b']
    const strings = items.filter((item): item is string => typeof item === 'string')
  `,
  { __dev_simulateFileName: 'utils.typeGuards.ts' },
)

addInvalidWithOptions(
  'shows custom message for filter with type guard',
  `
    const items = ['a', 1, 'b']
    const strings = items.filter((item): item is string => typeof item === 'string')
  `,
  {
    alternativeMsgs: {
      inArrayFilter: 'Replace type guard with type annotation',
    },
  },
  [
    {
      messageId: 'useFilterWithTypeCheck',
      data: { message: 'Replace type guard with type annotation' },
    },
  ],
)

addInvalidWithOptions(
  'shows custom message for find with type guard',
  `
    const items = ['a', 1, 'b']
    const firstString = items.find((item): item is string => typeof item === 'string')
  `,
  {
    alternativeMsgs: { inArrayFind: 'Replace type guard with type annotation' },
  },
  [
    {
      messageId: 'useFindWithTypeCheck',
      data: { message: 'Replace type guard with type annotation' },
    },
  ],
)

addInvalidWithOptions(
  'no suggestions when specific option not set',
  `
    const items = ['a', 1, 'b']
    const strings = items.filter((item): item is string => typeof item === 'string')
  `,
  { alternativeMsgs: { inArrayFind: 'Find suggestion' } },
  'default-error',
)

addInvalid(
  'no suggestions when specific option not set',
  `
  const test = {
    appLeftSidebarMenu: apiAppConfig.bundle?.menu_bar ? {
      type: 'left_sidebar' as const,
      title: apiAppConfig.bundle.menu_bar.title ?? '',
      background_color: apiAppConfig.bundle.menu_bar.top_bg_color,
      items: apiAppConfig.bundle.menu_bar.items
        .map(convertMenuBarItemToSimplified)
        .filter((item): item is MenuItem => item !== null),
      show_shortcuts: {
        apps: apiAppConfig.bundle.menu_bar.shortcuts.apps,
        favorites: apiAppConfig.bundle.menu_bar.shortcuts.favorites,
        settings: apiAppConfig.bundle.menu_bar.shortcuts.settings,
      },
    } : null,
    }
  `,
)

run()
