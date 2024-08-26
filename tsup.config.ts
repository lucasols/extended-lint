import { defineConfig } from 'tsup'

export default defineConfig({
  entry: ['src/extended-lint.ts'],
  clean: true,
  format: ['esm', 'cjs'],
})
