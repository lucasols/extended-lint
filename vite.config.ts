import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    lib: {
      entry: './src/extended-lint.ts',
      name: 'extended-lint',
      formats: ['cjs', 'es'],
    },
    rollupOptions: {
      external: ['esutils', 'espree', 'escape-string-regexp'],
    },
  },
  test: {
    include: ['tests/*.test.{ts,tsx}'],
    testTimeout: 2_000,
  },
})
