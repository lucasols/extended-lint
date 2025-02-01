import { createTester } from '../../tests/utils/createTester'
import { noLeakedTextInJS } from './no-leaked-text-in-jsx'

const tests = createTester(noLeakedTextInJS)

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
  'leaked ; text in jsx',
  `
    <div>
      <p>Hello</p>
      [
    </div>
  `,
  [{ messageId: 'leakedTextInJSX', data: { text: '[' } }],
)

tests.addValid(
  'no leaked text in jsx',
  `
    <div>
      <p>Hello</p>
      {","}
    </div>
  `,
)

tests.addValid(
  'no leaked text in jsx',
  `
    <div>
      <p>Hello</p>
      {'[]'}
    </div>
  `,
)

tests.run()
