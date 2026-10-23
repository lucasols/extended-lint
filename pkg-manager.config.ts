import { defineConfig } from '@ls-stack/pkg-manager'

export default defineConfig({
  requireMajorConfirmation: true,
  monorepo: {
    packages: [
      { name: '@ls-stack/extended-lint', path: 'packages/extended-lint' },
    ],
  },
})
