import { createTester } from '../../tests/utils/createTester'
import { templateIndent } from './template-indent'

const tests = createTester(templateIndent, {
  defaultErrorId: 'improperlyIndented',
})

const fixInput = (text: string): string => {
  return text
    .replace(/^[\s\n]+/, '')
    .replace(/[\s\n]+$/, '')
    .replaceAll('•', ' ')
    .replaceAll('→→', '\t')
}

tests.addValid('single line template', `foo = dedent\`one two three\``)

tests.addValid(
  'template without tag or function',
  `const text = \`
      Hello
      World
    \``,
)

tests.addValid('empty template', `\`\``)

tests.addInvalid(
  'improperly indented template with dedent tag',
  fixInput(`
      foo = dedent\`
      ••••••••one
      ••••••••two
      ••••••••••three
      ••••••••\`
    `),
  1,
  {
    output: fixInput(`
        foo = dedent\`
        one
        ••two
        ••••three
        \`
      `),
  },
)

tests.addInvalid(
  'improperly indented with CRLF line endings',
  ['dedent`', 'one', 'two', '`'].join('\r\n'),
  1,
  {
    output: ['dedent`', 'one', '  two', '`'].join('\r\n'),
  },
)

tests.addInvalid(
  'improperly indented template with function',
  fixInput(`
      function foo() {
      ••return dedent\`
      ••••••••one
      ••••••••two
      ••••••••••three
      ••••••••\`
      }
    `),
  1,
  {
    output: fixInput(`
        function foo() {
        ••return dedent\`
        one
        ••••two
        ••••••three
        ••\`
        }
      `),
  },
)

tests.addInvalid(
  'gql template with improper indentation',
  fixInput(`
      foo = gql\`
      ••••••••query GetUser {
      ••••••••••user {
      ••••••••••••id
      ••••••••••}
      ••••••••}
      ••••••••\`
    `),
  1,
  {
    output: fixInput(`
        foo = gql\`
        query GetUser {
        ••••user {
        ••••••id
        ••••}
        ••}
        \`
      `),
  },
)

tests.addInvalid(
  'stripIndent function call',
  fixInput(`
      foo = stripIndent(\`
      ••••••••one
      ••••••••two
      ••••••••••three
      ••••••••\`)
    `),
  1,
  {
    output: fixInput(`
        foo = stripIndent(\`
        one
        ••two
        ••••three
        \`)
      `),
  },
)

tests.addInvalid(
  'HTML comment',
  fixInput(`
      html = /* HTML */ \`
      ••••••••<div>
      ••••••••••<span>hello</span>
      ••••••••</div>
      ••••••••\`
    `),
  1,
  {
    output: fixInput(`
        html = /* HTML */ \`
        <div>
        ••••<span>hello</span>
        ••</div>
        \`
      `),
  },
)

tests.addInvalid(
  'jest inline snapshot',
  fixInput(`
      expect(foo).toMatchInlineSnapshot(\`
      ••••one
      ••••••three
      ••••\`)
    `),
  1,
  {
    output: fixInput(`
        expect(foo).toMatchInlineSnapshot(\`
        one
        ••••three
        \`)
      `),
  },
)

tests.addInvalidWithOptions(
  'custom indent size',
  fixInput(`
      foo = dedent\`
      ••one
      ••two
      ••••three
      \`
    `),
  { indent: 4 },
  1,
  {
    output: fixInput(`
        foo = dedent\`
        one
        ••••two
        ••••••three
        \`
      `),
  },
)

tests.addInvalidWithOptions(
  'custom tag',
  fixInput(`
      foo = customTag\`
      ••••••••one
      ••••••••two
      ••••••••\`
    `),
  { tags: ['customTag'] },
  1,
  {
    output: fixInput(`
        foo = customTag\`
        one
        ••two
        \`
      `),
  },
)

tests.addInvalidWithOptions(
  'member expression tag',
  fixInput(`
      foo = utils.dedent\`
      ••••••••one
      ••••••••two
      ••••••••\`
    `),
  { tags: ['utils.dedent'] },
  1,
  {
    output: fixInput(`
        foo = utils.dedent\`
        one
        ••two
        \`
      `),
  },
)

tests.addInvalidWithOptions(
  'custom function',
  fixInput(`
      foo = customFunction(\`
      ••••••••one
      ••••••••two
      ••••••••\`)
    `),
  { functions: ['customFunction'] },
  1,
  {
    output: fixInput(`
        foo = customFunction(\`
        one
        ••two
        \`)
      `),
  },
)

tests.addInvalidWithOptions(
  'custom comment',
  fixInput(`
      /* HTML */
      const template = \`
      ••••••••<div>
      ••••••••••<span>hello</span>
      ••••••••</div>
      ••••••••\`
    `),
  { comments: ['HTML'] },
  1,
  {
    output: fixInput(`
        /* HTML */
        const template = \`
        <div>
        ••••<span>hello</span>
        ••</div>
        \`
      `),
  },
)

tests.addValid(
  'disabled tags',
  fixInput(`
      foo = dedent\`
      ••••••••one
      ••••••••two
      ••••••••\`
    `),
  { tags: [] },
)

tests.addValid(
  'disabled functions',
  fixInput(`
      foo = stripIndent(\`
      ••••••••one
      ••••••••two
      ••••••••\`)
    `),
  { functions: [] },
)

tests.addValid(
  'disabled comments',
  fixInput(`
      /* HTML */
      const template = \`
      ••••••••<div>
      ••••••••••<span>hello</span>
      ••••••••</div>
      ••••••••\`
    `),
  { comments: [] },
)

tests.addValid(
  'trailing spaces in template',
  fixInput(`
      /* HTML */
      const template = dedent\`
      ••<div>
      ••••<span>hello</span>
      ••
      ••</div>
      ••\`
    `),
  { comments: [] },
)

tests.run()
