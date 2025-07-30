import { createTester } from '../../tests/utils/createTester'
import { requireReadsToVarProp } from './require-reads-to-var-prop'

const tests = createTester(requireReadsToVarProp, {
  defaultErrorId: 'propNotRead',
})

// Test configuration for createTest() calls requiring 'data' property to be read
const createTestConfig = {
  varsToCheck: [
    {
      selector: 'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
      prop: 'data',
      errorMsg: 'The data from createTest() should be used.',
    },
  ],
}

// Test configuration for useQuery() calls requiring 'isLoading' property to be read
const useQueryConfig = {
  varsToCheck: [
    {
      selector: 'VariableDeclarator[init.type="CallExpression"][init.callee.name="useQuery"]',
      prop: 'isLoading',
    },
  ],
}

// Valid cases
tests.addValid(
  'variable used - any property access',
  `
    const result = createTest()
    console.log(result.data)
  `,
  createTestConfig
)

tests.addValid(
  'variable used - destructuring',
  `
    const result = createTest()
    const { data } = result
    console.log(data)
  `,
  createTestConfig
)

tests.addValid(
  'variable used - different property access',
  `
    const result = createTest()
    console.log(result.otherProp)
  `,
  createTestConfig
)

tests.addValid(
  'variable used - passed to function',
  `
    const result = createTest()
    doSomething(result)
  `,
  createTestConfig
)

tests.addValid(
  'variable used - JSX prop',
  `
    const result = createTest()
    return <Component data={result} />
  `,
  createTestConfig
)

tests.addValid(
  'variable used - in expression',
  `
    const result = createTest()
    const items = [result, otherItem]
  `,
  createTestConfig
)

tests.addValid(
  'no tracked variables',
  `
    const someOtherVar = someOtherFunction()
    console.log(someOtherVar)
  `,
  createTestConfig
)

tests.addValid(
  'accessing any property is fine when variable not tracked',
  `
    const result = someOtherFunction()
    console.log(result.otherProp)
  `,
  createTestConfig
)

tests.addValid(
  'works with let declaration',
  `
    let result = createTest()
    console.log(result.data)
  `,
  createTestConfig
)

tests.addValid(
  'works with var declaration',
  `
    var result = createTest()
    console.log(result.data)
  `,
  createTestConfig
)

tests.addInvalid(
  'detects missing property with let',
  `
    let result = createTest()
    console.log('no data access')
  `,
  [{ data: { varName: 'result', customMsg: 'The data from createTest() should be used.' } }],
  { options: createTestConfig }
)

tests.addInvalid(
  'detects missing property with var',
  `
    var result = createTest()
    console.log('no data access')
  `,
  [{ data: { varName: 'result', customMsg: 'The data from createTest() should be used.' } }],
  { options: createTestConfig }
)


// Invalid cases - only when variable is completely unused
tests.addInvalid(
  'variable never used',
  `
    const result = createTest()
    console.log('done')
  `,
  [{ data: { varName: 'result', customMsg: 'The data from createTest() should be used.' } }],
  { options: createTestConfig }
)

tests.addInvalid(
  'multiple variables, some unused',
  `
    const result1 = createTest()
    const result2 = createTest()
    
    console.log(result1.data)
    console.log('result2 not used')
  `,
  [{ data: { prop: 'data', varName: 'result2', customMsg: 'The data from createTest() should be used.' } }],
  { options: createTestConfig }
)

// Test with different selector and property
tests.addInvalid(
  'useQuery variable unused',
  `
    const queryResult = useQuery()
    console.log('not using queryResult')
  `,
  [{ data: { varName: 'queryResult', customMsg: '' } }],
  { options: useQueryConfig }
)

tests.addValid(
  'useQuery with isLoading check',
  `
    const queryResult = useQuery()
    if (queryResult.isLoading) {
      return <Spinner />
    }
    console.log(queryResult.data)
  `,
  useQueryConfig
)

// Test with multiple selectors
const multiConfig = {
  varsToCheck: [
    {
      selector: 'VariableDeclarator[init.type="CallExpression"][init.callee.name="createTest"]',
      prop: 'data',
    },
    {
      selector: 'VariableDeclarator[init.type="CallExpression"][init.callee.name="useQuery"]',
      prop: 'isLoading',
    },
  ],
}

tests.addInvalid(
  'multiple selectors with mixed usage',
  `
    const testResult = createTest()
    const queryResult = useQuery()
    
    console.log(testResult.data) // used
    console.log('queryResult not used') // unused
  `,
  [{ data: { varName: 'queryResult', customMsg: '' } }],
  { options: multiConfig }
)

tests.run()