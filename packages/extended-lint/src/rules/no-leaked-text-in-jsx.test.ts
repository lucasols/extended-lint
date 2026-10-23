import { createTester } from '../../tests/utils/createTester'
import { noLeakedTextInJSX } from './no-leaked-text-in-jsx'

const tests = createTester(noLeakedTextInJSX)

tests.addValid(
  'no leaked text in jsx',
  `
    <div>Hello</div>
  `,
)

tests.addInvalid(
  'leaked , text in jsx',
  `
    <div>
      <p>Hello</p>
      ,
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: ',' } }],
)

tests.addInvalid(
  'leaked ; text in jsx',
  `
    <div>
      <p>Hello</p>
      ;
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: ';' } }],
)

tests.addInvalid(
  'leaked [ text in jsx',
  `
    <div>
      <p>Hello</p>
      [
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: '[' } }],
)

tests.addInvalid(
  'leaked ( text in jsx',
  `
    <div>
      <p>Hello</p>
      (
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: '(' } }],
)

tests.addInvalid(
  'leaked ) text in jsx',
  `
    <div>
      <p>Hello</p>
      )
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: ')' } }],
)

tests.addValid(
  'no leaked text in jsx 2',
  `
    <div>
      <p>Hello</p>
      {","}
    </div>
  `,
)

tests.addValid(
  'no leaked text in jsx 3',
  `
    <div>
      <p>Hello</p>
      {'[]'}
    </div>
  `,
)

tests.addInvalid(
  'leaked && text in jsx',
  `
    <div>
      <p>Hello</p>
      test && (
      <div>
        <p>Hello</p>
      </div>
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: '&&' } }],
)

tests.addInvalid(
  'leaked || text in jsx',
  `
    <div>
      <p>Hello</p>
      test || (
      <div>
        <p>Hello</p>
      </div>
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: '||' } }],
)

tests.addInvalid(
  'leaked ternary text in jsx',
  `
    <div>
      <p>Hello</p>
      test ? (
      <div>
        <p>Hello</p>
      </div>
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: '? (' } }],
)

tests.addInvalid(
  'leaked ] text in jsx',
  `
    <div>
      <p>Hello</p>
      ]
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: ']' } }],
)

tests.run()
