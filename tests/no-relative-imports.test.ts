import { noRelativeImports } from '../src/rules/no-relative-imports'
import { createTester } from './utils/createTester'

const tests = createTester(noRelativeImports)

const aliases = [
  {
    find: '@utils',
    replacement: '/src/utils',
  },
  {
    find: '@src',
    replacement: '/src',
  },
]

tests.addValid(
  'absolute imports with aliases',
  `
    import { something } from '@src/components/Button';
    import { utils } from '@utils/helpers';
  `,
  { aliases },
)

tests.addValid(
  'third party imports',
  `
    import React from 'react';
    import { useState } from 'react';
  `,
  { aliases },
)

tests.addInvalidWithOptions(
  'relative import same directory',
  `
    import { Button } from './Button';
  `,
  {
    aliases,
    rootDir: '/Users/username/',
    _dev_simulateFileName: '/Users/username/src/components/Component.ts',
  },
  [{ messageId: 'noRelativeImports' }],
  {
    output: `
      import { Button } from '@src/components/Button';
    `,
  },
)

tests.addInvalidWithOptions(
  'relative import parent directory',
  `
    import { helper } from '../utils/helper';
  `,
  {
    aliases,
    rootDir: '/Users/username/',
    _dev_simulateFileName: '/Users/username/src/components/Component.ts',
  },
  [{ messageId: 'noRelativeImports' }],
  {
    output: `
    import { helper } from '@utils/helper';
  `,
  },
)

tests.addInvalidWithOptions(
  'multiple relative imports',
  `
    import { Button } from './Button';
    import { helper } from '../utils/helper';
    import { something } from '../../src/something';
  `,
  {
    aliases,
    rootDir: '/Users/username/',
    _dev_simulateFileName: '/Users/username/src/components/Component.ts',
  },
  [
    { messageId: 'noRelativeImports' },
    { messageId: 'noRelativeImports' },
    { messageId: 'noRelativeImports' },
  ],
  {
    output: `
    import { Button } from '@src/components/Button';
    import { helper } from '@utils/helper';
    import { something } from '@src/something';
  `,
  },
)

tests.run()
