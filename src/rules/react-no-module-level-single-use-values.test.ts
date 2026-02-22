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
