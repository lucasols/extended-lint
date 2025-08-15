import { createTester } from '../../tests/utils/createTester'
import { noStaticStyleProp } from './no-static-style-prop'

const tests = createTester(noStaticStyleProp, {
  defaultErrorId: 'noStaticStyleProp',
})

// Valid cases - dynamic style props are allowed
tests.addValid(
  'variable reference',
  `
    const styles = { color: 'red' }
    function Component() {
      return <div style={styles}>Content</div>
    }
  `,
)

tests.addValid(
  'function call',
  `
    function getStyles() {
      return { color: 'blue' }
    }
    function Component() {
      return <div style={getStyles()}>Content</div>
    }
  `,
)

tests.addValid(
  'conditional expression',
  `
    function Component({ isDark }: { isDark: boolean }) {
      return <div style={{ color: isDark ? 'white' : 'black' }}>Content</div>
    }
  `,
)

tests.addValid(
  'template literal with expression',
  `
    function Component({ r, g, b }: { r: number; g: number; b: number }) {
      return <div style={{ color: \`rgb(\${r}, \${g}, \${b})\` }}>Content</div>
    }
  `,
)

tests.addValid(
  'computed property',
  `
    function Component({ prop }: { prop: string }) {
      return <div style={{ [prop]: 'value' }}>Content</div>
    }
  `,
)

tests.addValid(
  'spread operator',
  `
    const baseStyles = { color: 'red' }
    function Component() {
      return <div style={{ ...baseStyles, fontSize: 16 }}>Content</div>
    }
  `,
)

tests.addValid(
  'function call in property value',
  `
    function getColor() {
      return 'red'
    }
    function Component() {
      return <div style={{ color: getColor() }}>Content</div>
    }
  `,
)

tests.addValid(
  'variable in property value',
  `
    const color = 'red'
    function Component() {
      return <div style={{ color }}>Content</div>
    }
  `,
)

tests.addValid(
  'mixed dynamic and static properties',
  `
    const color = 'red'
    function Component() {
      return <div style={{ color, fontSize: 16 }}>Content</div>
    }
  `,
)

tests.addValid(
  'no style prop',
  `
    function Component() {
      return <div className="container">Content</div>
    }
  `,
)

// Invalid cases - static style props should be flagged
tests.addInvalid(
  'static object literal',
  `
    function Component() {
      return <div style={{ color: 'red', fontSize: 16 }}>Content</div>
    }
  `,
  [{ messageId: 'noStaticStyleProp', data: { customMessage: '' } }],
)

tests.addInvalid(
  'static string literal',
  `
    function Component() {
      return <div style="color: red; font-size: 16px;">Content</div>
    }
  `,
  [{ messageId: 'noStaticStyleProp', data: { customMessage: '' } }],
)

tests.addInvalid(
  'empty object literal',
  `
    function Component() {
      return <div style={{}}>Content</div>
    }
  `,
  [{ messageId: 'noStaticStyleProp', data: { customMessage: '' } }],
)

tests.addInvalid(
  'nested static object',
  `
    function Component() {
      return (
        <div>
          <span style={{ color: 'blue' }}>Text</span>
        </div>
      )
    }
  `,
  [{ messageId: 'noStaticStyleProp', data: { customMessage: '' } }],
)

// Test with custom message option
tests.addInvalidWithOptions(
  'custom error message',
  `
    function Component() {
      return <div style={{ color: 'red' }}>Content</div>
    }
  `,
  { customMessage: 'Use CSS classes instead' },
  [{ messageId: 'noStaticStyleProp', data: { customMessage: ' Use CSS classes instead' } }],
)

tests.addInvalidWithOptions(
  'empty custom message',
  `
    function Component() {
      return <div style={{ color: 'red' }}>Content</div>
    }
  `,
  { customMessage: '' },
  [{ messageId: 'noStaticStyleProp', data: { customMessage: '' } }],
)

tests.run()