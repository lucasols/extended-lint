import { defineConfig } from 'vite'
import { ViteAddCodeLineToTestNamePlugin } from './src/addCodeLineToTestName'
import { ViteGetCodeLinePlugin } from './src/getCodeLineVitePlugin'

const isDev =
  process.env.NODE_ENV === 'development' || process.env.NODE_ENV === 'test'

export default defineConfig({
  plugins: [ViteAddCodeLineToTestNamePlugin(), ViteGetCodeLinePlugin()],
  test: {
    include: ['tests/*.test.{ts,tsx,js}', 'src/**/*.test.{ts,tsx,js}'],
    testTimeout: 20_000,
    allowOnly: isDev,
    setupFiles: 'tests/fixture/setup.ts',
  },
})
