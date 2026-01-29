import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import { createNewTester, getErrorsFromResult } from '../../tests/utils/createTester'
import { noDataAttributesOnComponents } from './no-data-attributes-on-components'

const { valid, invalid } = createNewTester(noDataAttributesOnComponents)

test('allows data-* attributes on native elements', async () => {
  await valid(dedent`
    const Component = () => (
      <div data-testid="test">
        <span data-foo="bar" />
        <button data-tracking="click" />
      </div>
    )
  `)
})

test('allows custom components without data-* attributes', async () => {
  await valid(dedent`
    const Component = () => (
      <MyComponent id="test" className="foo" />
    )
  `)
})

test('allows regular props on custom components', async () => {
  await valid(dedent`
    const Component = () => (
      <Button onClick={handleClick} disabled={false} testId="btn" />
    )
  `)
})

test('reports error for data-* attribute on custom component', async () => {
  const { result } = await invalid(dedent`
    const Component = () => (
      <MyComponent data-testid="test" />
    )
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noDataAttributesOnComponents', line: 2 }
    "
  `)
})

test('reports error for multiple data-* attributes on custom component', async () => {
  const { result } = await invalid(dedent`
    const Component = () => (
      <Button data-testid="btn" data-tracking="click" />
    )
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noDataAttributesOnComponents', line: 2 }
    - { messageId: 'noDataAttributesOnComponents', line: 2 }
    "
  `)
})

test('reports error for data-* attribute on JSXMemberExpression component', async () => {
  const { result } = await invalid(dedent`
    const Component = () => (
      <Foo.Bar data-test="x" />
    )
  `)

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noDataAttributesOnComponents', line: 2 }
    "
  `)
})

test('allows data-* attribute when component is in allow list', async () => {
  await valid({
    code: dedent`
      const Component = () => (
        <Button data-testid="btn" />
      )
    `,
    options: [{ allow: { Button: ['data-testid'] } }],
  })
})

test('allows multiple data-* attributes when all are in allow list', async () => {
  await valid({
    code: dedent`
      const Component = () => (
        <Link data-testid="link" data-tracking="nav" />
      )
    `,
    options: [{ allow: { Link: ['data-testid', 'data-tracking'] } }],
  })
})

test('reports error when data-* attribute is not in allow list for component', async () => {
  const { result } = await invalid({
    code: dedent`
      const Component = () => (
        <Button data-foo="bar" />
      )
    `,
    options: [{ allow: { Button: ['data-testid'] } }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noDataAttributesOnComponents', line: 2 }
    "
  `)
})

test('reports error for one attribute while allowing another on same component', async () => {
  const { result } = await invalid({
    code: dedent`
      const Component = () => (
        <Button data-testid="btn" data-foo="bar" />
      )
    `,
    options: [{ allow: { Button: ['data-testid'] } }],
  })

  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - { messageId: 'noDataAttributesOnComponents', line: 2 }
    "
  `)
})

test('handles namespaced JSX elements as native', async () => {
  await valid(dedent`
    const Component = () => (
      <svg:rect data-id="rect1" />
    )
  `)
})
