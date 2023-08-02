import { defineConfig } from 'vite'

const isDev = process.env.NODE_ENV === 'development'

export default defineConfig({
  test: {
    include: ['tests/*.test.{ts,tsx}'],
    testTimeout: 2_000,
    allowOnly: isDev,
    setupFiles: 'tests/fixture/setup.ts',
  },
})
