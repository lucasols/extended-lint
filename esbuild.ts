import esbuild from 'esbuild'

const sharedConfig: esbuild.BuildOptions = {
  entryPoints: ['src/extended-lint.ts'],
  bundle: true,
  // minify: true,
  external: ['escape-string-regexp', 'espree', 'esutils'],
}

esbuild.buildSync({
  ...sharedConfig,
  outfile: 'dist/extended-lint.js',
  format: 'cjs',
  target: ['node16'],
})

esbuild.buildSync({
  ...sharedConfig,
  outfile: 'dist/extended-lint.mjs',
  format: 'esm',
  target: ['esnext'],
})
