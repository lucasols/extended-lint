import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { reactNoModuleLevelSingleUseValues } from './react-no-module-level-single-use-values'

const { valid, invalid } = createNewTester(reactNoModuleLevelSingleUseValues)

test('valid: module level array used in multiple components', async () => {
  await valid({
    code: dedent`
      const options = ['a', 'b']

      export function FirstList() {
        return <div>{options.join(', ')}</div>
      }

      export function SecondList() {
        return <span>{options.length}</span>
      }
    `,
    filename: 'lists.tsx',
  })
})

test('valid: module level array used outside of component', async () => {
  await valid({
    code: dedent`
      const options = ['a', 'b']
      const optionsSet = new Set(options)
      const optionsCount = optionsSet.size

      export function List() {
        return <div>{optionsCount}</div>
      }
    `,
    filename: 'lists.tsx',
  })
})

test('valid: module level primitive value used in one component', async () => {
  await valid({
    code: dedent`
      const maxItems = 5
      const label = 'items'

      export function List() {
        return <div>{maxItems} {label}</div>
      }
    `,
    filename: 'lists.tsx',
  })
})

test('valid: non tsx files are ignored', async () => {
  await valid({
    code: dedent`
      const options = ['a', 'b']

      export function List() {
        return options.join(', ')
      }
    `,
    filename: 'lists.ts',
  })
})

test('valid: same-file component declaration as arrow function is ignored', async () => {
  await valid({
    code: dedent`
      const CopyPublicLinkButton = ({ appId }: { appId: string }) => {
        return <button>{appId}</button>
      }

      export function AppViewHeader({ appId }: { appId: string }) {
        return (
          <div>
            <CopyPublicLinkButton appId={appId} />
          </div>
        )
      }
    `,
    filename: 'AppViewHeader.tsx',
  })
})

test('valid: module helper arrow function used by one component is ignored', async () => {
  await valid({
    code: dedent`
      const formatLabel = (value: string) => value.toUpperCase()

      export function Header() {
        return <div>{formatLabel('item')}</div>
      }
    `,
    filename: 'header.tsx',
  })
})

test('valid: inferable alias reference is ignored', async () => {
  await valid({
    code: dedent`
      const options = ['a', 'b']
      const listOptions = options

      export function List() {
        return <div>{listOptions.join(', ')}</div>
      }
    `,
    filename: 'list.tsx',
  })
})

test('valid: inferable member reference is ignored', async () => {
  await valid({
    code: dedent`
      const config = { labels: ['a', 'b'] }
      const listOptions = config.labels

      export function List() {
        return <div>{listOptions.join(', ')}</div>
      }
    `,
    filename: 'list.tsx',
  })
})

test('valid: regex value used by one component is ignored', async () => {
  await valid({
    code: dedent`
      const userIdRegex = /^user_[a-z0-9]+$/

      export function List({ value }: { value: string }) {
        return <div>{userIdRegex.test(value) ? 'ok' : 'invalid'}</div>
      }
    `,
    filename: 'list.tsx',
  })
})

test('valid: class instantiation used by one component is ignored', async () => {
  await valid({
    code: dedent`
      class Store {
        state: Record<string, true | 'inProgress'>

        constructor() {
          this.state = {}
        }
      }

      const notificationWasForwardedState = new Store()

      export function List({ id }: { id: string }) {
        const value = notificationWasForwardedState.state[id]
        return <div>{value ? 'forwarded' : 'pending'}</div>
      }
    `,
    filename: 'list.tsx',
  })
})

test('valid: ignoreConstCaseVarRegex ignores matching constant-case variable', async () => {
  await valid({
    code: dedent`
      const MOCK_APP_PERMISSIONS = {
        canViewApp: true,
      }

      export function GridPositioningStory() {
        return <div>{MOCK_APP_PERMISSIONS.canViewApp ? 'yes' : 'no'}</div>
      }
    `,
    filename: 'GridPositioningStory.tsx',
    options: [{ ignoreConstCaseVarRegex: '^MOCK_' }],
  })
})

test('invalid: set instantiation used in one component', async () => {
  const { result } = await invalid({
    code: dedent`
      const selectedIds = new Set<string>()

      export function List({ id }: { id: string }) {
        return <div>{selectedIds.has(id) ? 'selected' : 'not selected'}</div>
      }
    `,
    filename: 'list.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})

test('invalid: map instantiation used in one component', async () => {
  const { result } = await invalid({
    code: dedent`
      const nameById = new Map<string, string>()

      export function List({ id }: { id: string }) {
        return <div>{nameById.get(id) ?? 'unknown'}</div>
      }
    `,
    filename: 'list.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})

test('invalid: ignoreConstCaseVarRegex does not ignore non constant-case variable', async () => {
  const { result } = await invalid({
    code: dedent`
      const mockAppPermissions = {
        canViewApp: true,
      }

      export function GridPositioningStory() {
        return <div>{mockAppPermissions.canViewApp ? 'yes' : 'no'}</div>
      }
    `,
    filename: 'GridPositioningStory.tsx',
    options: [{ ignoreConstCaseVarRegex: '^mock' }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})

test('invalid: object used twice in one component', async () => {
  const { result } = await invalid({
    code: dedent`
      const mockAppPermissions = {
        canViewApp: true,
      }

      export function GridPositioningStory() {
        return (
          <div>
            <Editable appPermissions={mockAppPermissions} />
            <Readonly appPermissions={mockAppPermissions} />
          </div>
        )
      }

      function Editable({ appPermissions }: { appPermissions: { canViewApp: boolean } }) {
        return <div>{appPermissions.canViewApp ? 'yes' : 'no'}</div>
      }

      function Readonly({ appPermissions }: { appPermissions: { canViewApp: boolean } }) {
        return <div>{appPermissions.canViewApp ? 'yes' : 'no'}</div>
      }
    `,
    filename: 'GridPositioningStory.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})

test('invalid: module array used only in one component', async () => {
  const { result } = await invalid({
    code: dedent`
      const options = ['a', 'b']

      export function List() {
        return <div>{options.join(', ')}</div>
      }
    `,
    filename: 'list.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})

test('invalid: module object used only in one component', async () => {
  const { result } = await invalid({
    code: dedent`
      const style = { color: 'red' }

      export function List() {
        return <div style={style}>hello</div>
      }
    `,
    filename: 'list.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})

test('invalid: long module string used only in one component', async () => {
  const { result } = await invalid({
    code: dedent`
      const longLabel = 'This is a very long static string that should be moved into a single component according to this rule'

      export function List() {
        return <div>{longLabel}</div>
      }
    `,
    filename: 'list.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})

test('invalid: module array used only inside nested callback of one component', async () => {
  const { result } = await invalid({
    code: dedent`
      const options = ['a', 'b'] as const

      const List = () => {
        const pickFirst = () => options[0]
        return <div>{pickFirst()}</div>
      }

      export default List
    `,
    filename: 'list.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'moveValueInsideComponent', line: 1 }
    "
  `)
})
