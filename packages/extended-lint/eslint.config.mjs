// @ts-check
import { cfgFlags, lsStackEslintCfg } from '@ls-stack/eslint-cfg'

const { OFF, ERROR } = cfgFlags

export default lsStackEslintCfg({
  tsconfigRootDir: import.meta.dirname,
  globalRules: {
    '@typescript-eslint/no-explicit-any': OFF,
    '@typescript-eslint/no-unsafe-assignment': OFF,
    '@typescript-eslint/no-non-null-assertion': ERROR,
    '@typescript-eslint/no-unnecessary-condition': ERROR,
    '@typescript-eslint/consistent-type-assertions': [
      ERROR,
      {
        assertionStyle: 'never',
      },
    ],

    'vitest/expect-expect': OFF,
  },
  ignore: ['dist/**/*', '**/*.js', 'debug/**/*', 'tests/fixture/**/*'],
})
