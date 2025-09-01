import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { reactSingleExport } from './react-single-export'

const { valid, invalid } = createNewTester(reactSingleExport)

test('valid: single default export in React component', async () => {
  await valid({
    code: dedent`
      import React from 'react'

      const MyComponent = () => <div>Hello</div>
      
      export default MyComponent
    `,
    filename: 'MyComponent.tsx',
  })
})

test('valid: single named export in React component', async () => {
  await valid({
    code: dedent`
      import React from 'react'

      export const MyComponent = () => <div>Hello</div>
    `,
    filename: 'MyComponent.tsx',
  })
})

test('valid: single export with JSX', async () => {
  await valid({
    code: dedent`
      export const MyComponent = () => {
        return <div>Hello World</div>
      }
    `,
    filename: 'MyComponent.tsx',
  })
})

test('valid: non-React files with multiple exports', async () => {
  await valid({
    code: dedent`
      export const utils = {}
      export const helper = () => {}
      export const constant = 'value'
    `,
    filename: 'utils.ts',
  })
})

test('valid: React component with only type exports', async () => {
  await valid({
    code: dedent`
      import React from 'react'

      export type Props = {
        name: string
      }

      const MyComponent = ({ name }: Props) => <div>{name}</div>
      
      export default MyComponent
    `,
    filename: 'MyComponent.tsx',
  })
})

test('invalid: multiple value exports in React component', async () => {
  const { result } = await invalid({
    code: dedent`
      import React from 'react'

      export const MyComponent = () => <div>Hello</div>
      export const OtherComponent = () => <div>World</div>
    `,
    filename: 'MyComponent.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 4 }
    "
  `)
})

test('invalid: default export + named export in React component', async () => {
  const { result } = await invalid({
    code: dedent`
      import React from 'react'

      const MyComponent = () => <div>Hello</div>
      export const helper = () => {}
      
      export default MyComponent
    `,
    filename: 'MyComponent.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 6 }
    "
  `)
})

test('invalid: TSX file with multiple exports', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Component1 = () => <div>One</div>
      export const Component2 = () => <div>Two</div>
    `,
    filename: 'components.tsx',
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})

test('valid: React component file with custom extension and single export', async () => {
  await valid({
    code: dedent`
      export const MyComponent = () => {
        return <button>Click me</button>
      }
    `,
    filename: 'MyComponent.component.tsx',
    options: [{ extensions: ['component.tsx'] }],
  })
})

test('invalid: custom extension with multiple exports', async () => {
  const { result } = await invalid({
    code: dedent`
      export const Component = () => <div>Hello</div>
      export const utils = {}
    `,
    filename: 'MyComponent.component.tsx',
    options: [{ extensions: ['component.tsx'] }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'multipleExports', line: 2 }
    "
  `)
})