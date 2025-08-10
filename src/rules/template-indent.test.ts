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

tests.addInvalid(
  'basic dedent template',
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
      ••one
      ••two
      ••••three
      \`
    `),
  },
)

tests.addInvalidWithOptions(
  'custom indentable tag',
  fixInput(`
    foo = customIndentableTag\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
    foo = differentTagThatMightBeWhitespaceSensitive\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
    foo = \`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
  `),
  { tags: ['customIndentableTag'] },
  1,
  {
    output: fixInput(`
      foo = customIndentableTag\`
      ••one
      ••two
      ••••three
      \`
      foo = differentTagThatMightBeWhitespaceSensitive\`
      ••••••••one
      ••••••••two
      ••••••••••three
      ••••••••\`
      foo = \`
      ••••••••one
      ••••••••two
      ••••••••••three
      ••••••••\`
    `),
  },
)

tests.addInvalidWithOptions(
  'member expression tag',
  fixInput(`
    foo = utils.dedent\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
  `),
  { tags: ['utils.dedent'] },
  1,
  {
    output: fixInput(`
      foo = utils.dedent\`
      ••one
      ••two
      ••••three
      \`
    `),
  },
)

tests.addInvalid(
  'function context indentation',
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
      ••••one
      ••••two
      ••••••three
      ••\`
      }
    `),
  },
)

tests.addInvalid(
  'complex template with expressions',
  fixInput(`
    // a
    // bb
    // ccc
    // dddd
    function foo() {
    ••return dedent\`
    ••••••••one
    ••••••••two
    ••••••••••three \${3} four
    ••••••••••••five
    ••••••••••••••\${{f: 5}}
    ••••••••••••six
    ••••••••\`
    }
  `),
  1,
  {
    output: fixInput(`
      // a
      // bb
      // ccc
      // dddd
      function foo() {
      ••return dedent\`
      ••••one
      ••••two
      ••••••three \${3} four
      ••••••••five
      ••••••••••\${{f: 5}}
      ••••••••six
      ••\`
      }
    `),
  },
)

tests.addInvalid(
  'multiple template tags',
  fixInput(`
    foo = gql\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
    foo = sql\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
    foo = dedent\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
    foo = outdent\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
    foo = somethingElse\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
  `),
  4,
  {
    output: fixInput(`
      foo = gql\`
      ••one
      ••two
      ••••three
      \`
      foo = sql\`
      ••one
      ••two
      ••••three
      \`
      foo = dedent\`
      ••one
      ••two
      ••••three
      \`
      foo = outdent\`
      ••one
      ••two
      ••••three
      \`
      foo = somethingElse\`
      ••••••••one
      ••••••••two
      ••••••••••three
      ••••••••\`
    `),
  },
)

tests.addInvalid(
  'stripIndent function',
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
      ••one
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
      ••<div>
      ••••<span>hello</span>
      ••</div>
      \`
    `),
  },
)

tests.addInvalid(
  'html lowercase comment',
  fixInput(`
    html = /* html */ \`
    ••••••••<div>
    ••••••••••<span>hello</span>
    ••••••••</div>
    ••••••••\`
  `),
  1,
  {
    output: fixInput(`
      html = /* html */ \`
      ••<div>
      ••••<span>hello</span>
      ••</div>
      \`
    `),
  },
)

tests.addInvalid(
  'indent comment',
  fixInput(`
    html = /* indent */ \`
    ••••••••<div>
    ••••••••••<span>hello</span>
    ••••••••</div>
    ••••••••\`
  `),
  1,
  {
    output: fixInput(`
      html = /* indent */ \`
      ••<div>
      ••••<span>hello</span>
      ••</div>
      \`
    `),
  },
)

tests.addInvalidWithOptions(
  'custom comment',
  fixInput(`
    html = /* please indent me! */ \`
    ••••••••<div>
    ••••••••••<span>hello</span>
    ••••••••</div>
    ••••••••\`
  `),
  { comments: ['please indent me!'] },
  1,
  {
    output: fixInput(`
      html = /* please indent me! */ \`
      ••<div>
      ••••<span>hello</span>
      ••</div>
      \`
    `),
  },
)

tests.addInvalidWithOptions(
  'custom indent number',
  fixInput(`
    foo = dedent\`
    ••one
    ••two
    ••••three
    \`
  `),
  { indent: 10 },
  1,
  {
    output: fixInput(`
      foo = dedent\`
      ••••••••••one
      ••••••••••two
      ••••••••••••three
      \`
    `),
  },
)

tests.addInvalidWithOptions(
  'custom indent string',
  fixInput(`
    foo = dedent\`
    ••one
    ••two
    ••••three
    \`
  `),
  { indent: '\t\t\t\t' },
  1,
  {
    output: fixInput(`
      foo = dedent\`
      →→→→→→→→one
      →→→→→→→→two
      →→→→→→→→••three
      \`
    `),
  },
)

tests.addInvalidWithOptions(
  'custom functions',
  fixInput(`
    foo = customDedentFunction1(\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`)
    foo = utils.customDedentFunction2('some-other-arg', \`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`)
  `),
  { functions: ['customDedentFunction1', 'utils.customDedentFunction2'] },
  2,
  {
    output: fixInput(`
      foo = customDedentFunction1(\`
      ••one
      ••two
      ••••three
      \`)
      foo = utils.customDedentFunction2('some-other-arg', \`
      ••one
      ••two
      ••••three
      \`)
    `),
  },
)

tests.addInvalid(
  'template with expressions',
  fixInput(`
    outdent\`
    before
    before\${
    expression
    }after
    after
    \`
  `),
  1,
  {
    output: fixInput(`
      outdent\`
      ••before
      ••before\${
      expression
      }after
      ••after
      \`
    `),
  },
)

tests.addInvalid(
  'nested template',
  fixInput(`
    outdent\`
    ••before
    ••before\${
    →→→→→→outdent\`
    inner
    →→→→→→\`
    }after
    ••after
    \`
  `),
  1,
  {
    output: fixInput(`
      outdent\`
      ••before
      ••before\${
      →→→→→→outdent\`
      →→→→→→→→inner
      →→→→→→\`
      }after
      ••after
      \`
    `),
  },
)

tests.addValid('single line template', 'foo = dedent`one two three`')

tests.addValid(
  'properly indented tabs',
  fixInput(`
    function f() {
    →→foo = dedent\`
    →→→→one
    →→→→two
    →→→→→→three
    →→→→four
    →→\`
    }
  `),
)

tests.addValid(
  'properly indented with empty lines',
  fixInput(`
    function f() {
    →→foo = dedent\`
    →→→→one

    →→→→two
    →→→→→→three
    →→→→four
    →→\`
    }
  `),
)

tests.addValid(
  'properly indented spaces',
  fixInput(`
    function f() {
    ••foo = dedent\`
    ••••one
    ••••two
    ••••••three
    ••••four
    ••\`
    }
  `),
)

tests.addValid(
  'disabled tags and functions',
  fixInput(`
    foo = stripIndent(\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`)
    foo = dedent\`
    ••••••••one
    ••••••••two
    ••••••••••three
    ••••••••\`
  `),
  {
    tags: ['somethingOtherThanDedent'],
    functions: ['somethingOtherThanStripIndent'],
  },
)

tests.addValid('function without template', 'stripIndent(foo)')

tests.addValid('empty template', '``')

tests.addValid(
  'disabled comments',
  fixInput(`
    foo = /* indent */ \`
    ••••••one
    ••••••two
    ••••••••three
    \`
  `),
  { comments: [] },
)

tests.addValid(
  'properly indented with expressions',
  fixInput(`
    outdent\`
    ••before
    ••before\${
    expression
    }after
    ••after
    \`
  `),
)

tests.addValid(
  'nested normal template',
  fixInput(`
    outdent\`
    ••before
    ••before\${
    ••••••normalTemplate\`
    inner
    ••••••\`
    }after
    ••after
    \`
  `),
)

tests.addValid(
  'trailing spaces in last line preserved',
  fixInput(`
    outdent\`
    ••Line with trailing spaces••••
    \`
  `),
)

tests.addValid(
  'trailing spaces in non-last line preserved',
  fixInput(`
    outdent\`
    ••Line with trailing spaces••••
    ••Line without trailing spaces
    \`
  `),
)

tests.addValid(
  'empty lines preserved',
  fixInput(`
    outdent\`
    ••Line1

    ••Line2
    \`
  `),
)

tests.addValid(
  'expect toMatchInlineSnapshot optional call',
  'expect(foo).toMatchInlineSnapshot?.(`\n  one\n    three\n  `)',
)
tests.addValid(
  'expect toMatchInlineSnapshot optional chaining',
  'expect(foo)?.toMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'expect toMatchInlineSnapshot with extra argument after',
  'expect(foo).toMatchInlineSnapshot(`\n  one\n    three\n  `, extraArgument)',
)
tests.addValid(
  'expect toMatchInlineSnapshot with extra argument before',
  'expect(foo).toMatchInlineSnapshot(extraArgument, `\n  one\n    three\n  `)',
)
tests.addValid(
  'expect toMatchInlineSnapshot no args',
  'expect(foo).toMatchInlineSnapshot()',
)
tests.addValid(
  'expect with extra argument',
  'expect(foo, extraArgument).toMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'expect empty call',
  'expect().toMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'not toMatchInlineSnapshot',
  'expect(foo).notToMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'assert expect',
  'assert.expect(foo).toMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'expect property access',
  'expect.toMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'not expect function',
  'notExpect(foo).toMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'new expect',
  'new expect(foo).toMatchInlineSnapshot(`\n  one\n    three\n  `)',
)
tests.addValid(
  'new expect call',
  'new (expect(foo).toMatchInlineSnapshot)(`\n  one\n    three\n  `)',
)

tests.addValid(
  'properly indented snapshot',
  fixInput(`
    expect(foo).toMatchInlineSnapshot(\`
    ••foo
    ••bar
    \`)
  `),
)

tests.addInvalid(
  'expect toMatchInlineSnapshot',
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
      ••one
      ••••three
      \`)
    `),
  },
)

tests.addValid(
  'trailing spaces matching the indent should be allowed',
  fixInput(`
      const template = dedent\`
      ••<div>
      ••••<span>hello</span>
      ••
      ••</div>
      ••\`
    `),
)

tests.addValid(
  'trailing spaces matching the indent should be allowed in toMatchInlineSnapshot',
  fixInput(`
    expect(foo).toMatchInlineSnapshot(\`
    ••<div>
    ••••<span>hello</span>
    ••
    ••</div>
    \`)
  `),
)

tests.run()
