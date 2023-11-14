import { defineConfig } from 'vite'
import { ViteGetCodeLinePlugin } from './src/getCodeLineVitePlugin'

const isDev =
  process.env.NODE_ENV === 'development' || process.env.NODE_ENV === 'test'

export default defineConfig({
  plugins: [ViteGetCodeLinePlugin()],
  test: {
    include: ['tests/*.test.{ts,tsx,js}'],
    testTimeout: 2_000,
    allowOnly: isDev,
    setupFiles: 'tests/fixture/setup.ts',
  },
})
