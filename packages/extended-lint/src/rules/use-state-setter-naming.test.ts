import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsWithMsgFromResult,
  getSuggestionOutput,
} from '../../tests/utils/createTester'
import { useStateSetterNaming } from './use-state-setter-naming'

const { valid, invalid } = createNewTester(useStateSetterNaming)

test('valid useState destructuring with correct naming', async () => {
  await valid(
    dedent`
      const [count, setCount] = useState(0);
    `,
  )
})

test('use without setter', async () => {
  await valid(
    dedent`
      const [count] = useState(0);
    `,
  )
})

test('valid useState destructuring with camelCase naming', async () => {
  await valid(
    dedent`
      const [userName, setUserName] = useState('');
    `,
  )
})

test('valid useState destructuring with abbreviation', async () => {
  await valid(
    dedent`
      const [isLoading, setIsLoading] = useState(false);
    `,
  )
})

test('valid useState destructuring with underscore', async () => {
  await valid(
    dedent`
      const [user_data, setUser_data] = useState({});
    `,
  )
})

test('valid useState destructuring with numbers', async () => {
  await valid(
    dedent`
      const [step1, setStep1] = useState(0);
    `,
  )
})

test('invalid useState destructuring with incorrect setter name', async () => {
  const { result } = await invalid(dedent`
    const [records, setGroups] = useState([]);
  `)

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setRecords" with a "set" prefix but got "setGroups"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [records, setRecords] = useState([]);"
  `)
})

test('invalid useState destructuring with completely different name', async () => {
  const { result } = await invalid(dedent`
    const [data, updateItems] = useState([]);
  `)

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setData" with a "set" prefix but got "updateItems"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [data, setData] = useState([]);"
  `)
})

test('invalid useState destructuring with wrong prefix', async () => {
  const { result } = await invalid(dedent`
    const [user, changeUser] = useState(null);
  `)

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setUser" with a "set" prefix but got "changeUser"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [user, setUser] = useState(null);"
  `)
})

test('valid React.useState usage', async () => {
  await valid(
    dedent`
      const [items, setItems] = React.useState([]);
    `,
  )
})

test('invalid React.useState usage', async () => {
  const { result } = await invalid(dedent`
    const [items, updateItems] = React.useState([]);
  `)

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setItems" with a "set" prefix but got "updateItems"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [items, setItems] = React.useState([]);"
  `)
})

test('ignores non-useState calls', async () => {
  await valid(
    dedent`
      const [a, b] = someFunction();
      const [x, y] = useCustomHook();
    `,
  )
})

test('ignores useState with single destructuring', async () => {
  await valid(
    dedent`
      const [value] = useState('test');
    `,
  )
})

test('ignores useState with more than two destructuring elements', async () => {
  await valid(
    dedent`
      const [value, setValue, extra] = useState('test');
    `,
  )
})

test('handles complex camelCase names correctly', async () => {
  const { result } = await invalid(dedent`
    const [userProfileData, updateProfile] = useState({});
  `)

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setUserProfileData" with a "set" prefix but got "updateProfile"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [userProfileData, setUserProfileData] = useState({});"
  `)
})

test('valid with ignorePrefixes - underscore prefix in setter', async () => {
  await valid({
    code: dedent`
      const [value, _setValue] = useState('');
    `,
    options: [{ ignorePrefixes: ['_'] }],
  })
})

test('valid with ignorePrefixes - underscore prefix in value', async () => {
  await valid({
    code: dedent`
      const [_value, setValue] = useState('');
    `,
    options: [{ ignorePrefixes: ['_'] }],
  })
})

test('valid with ignorePrefixes - underscore in both', async () => {
  await valid({
    code: dedent`
      const [_value, _setValue] = useState('');
    `,
    options: [{ ignorePrefixes: ['_'] }],
  })
})

test('valid with ignorePrefixes - single underscore value', async () => {
  await valid({
    code: dedent`
      const [_, setValue] = useState('');
    `,
    options: [{ ignorePrefixes: ['_'] }],
  })
})

test('valid with ignorePrefixes - multiple prefixes', async () => {
  await valid({
    code: dedent`
      const [$value, $setValue] = useState('');
    `,
    options: [{ ignorePrefixes: ['_', '$'] }],
  })
})

test('invalid with ignorePrefixes - wrong setter after stripping', async () => {
  const { result } = await invalid({
    code: dedent`
      const [value, _wrong] = useState('');
    `,
    options: [{ ignorePrefixes: ['_'] }],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setValue" with a "set" prefix but got "_wrong"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [value, setValue] = useState('');"
  `)
})

test('invalid with ignorePrefixes - wrong setter with prefix in value', async () => {
  const { result } = await invalid({
    code: dedent`
      const [_value, updateValue] = useState('');
    `,
    options: [{ ignorePrefixes: ['_'] }],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setValue" with a "set" prefix but got "updateValue"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [_value, setValue] = useState('');"
  `)
})

test('invalid with ignorePrefixes - should preserve prefix in suggestion', async () => {
  const { result } = await invalid({
    code: dedent`
      const [_data, wrongSetter] = useState([]);
    `,
    options: [{ ignorePrefixes: ['_'] }],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "setData" with a "set" prefix but got "wrongSetter"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [_data, setData] = useState([]);"
  `)
})

test('works without ignorePrefixes option', async () => {
  const { result } = await invalid(dedent`
    const [_value, _wrongSetter] = useState('');
  `)

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'incorrectSetterName'
      msg: 'useState setter should follow the pattern "set_value" with a "set" prefix but got "_wrongSetter"'
      line: 1
    "
  `)

  expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
    "const [_value, set_value] = useState('');"
  `)
})
