import { createTester } from '../../tests/utils/createTester'
import { improvedNoUnnecessaryCondition } from './improved-no-unnecessary-condition'

const tests = createTester(improvedNoUnnecessaryCondition)

tests.addValid(
  'valid code with no typeof usage',
  `
    const str = 'string'
    if (str.length > 0) {
      console.log(str)
    }
  `,
)

tests.addValid(
  'valid typeof check on variable with unknown type',
  `
    function test(value: any) {
      if (typeof value === 'string') {
        console.log(value.toUpperCase())
      }
    }
  `,
)

tests.addValid(
  'valid typeof check on variable with union type',
  `
    function test(value: string | number) {
      if (typeof value === 'string') {
        console.log(value.toUpperCase())
      }
    }
  `,
)

tests.addInvalid(
  'unnecessary typeof check on string variable',
  `
    const str = 'hello'
    if (typeof str === 'string') {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'str',
        type: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on string variable',
  `
    const str = 'hello'
    if (typeof str === 'number') {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'str',
        actualType: 'string',
        expectedType: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on number variable',
  `
    const num = 42
    if (typeof num === 'number') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'num',
        type: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on number variable',
  `
    const num = 42
    if (typeof num === 'string') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'num',
        actualType: 'number',
        expectedType: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on boolean variable',
  `
    const bool = true
    if (typeof bool === 'boolean') {
      console.log(bool)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'bool',
        type: 'boolean',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check on boolean variable',
  `
    const bool = true
    if (typeof bool === 'object') {
      console.log(bool)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'bool',
        actualType: 'boolean',
        expectedType: 'object',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check on object variable',
  `
    const obj = { a: 1 }
    if (typeof obj === 'object') {
      console.log(obj)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'obj',
        type: 'object',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check with different operators',
  `
    const str = 'hello'
    if (typeof str == 'string') {
      console.log(str)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'str',
        type: 'string',
      },
    },
  ],
)

tests.addInvalid(
  'unnecessary typeof check with !== operator',
  `
    const num = 42
    if (typeof num !== 'string') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'unnecessaryTypeofCondition',
      data: {
        name: 'num',
        type: 'number',
      },
    },
  ],
)

tests.addInvalid(
  'always false typeof check with !== operator',
  `
    const num = 42
    if (typeof num !== 'number') {
      console.log(num)
    }
  `,
  [
    {
      messageId: 'alwaysFalseTypeofCondition',
      data: {
        name: 'num',
        actualType: 'number',
        expectedType: 'number',
      },
    },
  ],
)

tests.run()
