import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 10000,
    include: ['test/**/*.{test,spec}.{js,ts}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/**',
        'test/**',
        '*.config.js',
        'example.js',
        'index.js',
        'types.d.ts',
        'build.rs',
        'src/**'
      ]
    }
  },
  esbuild: {
    target: 'node18'
  },
  define: {
    'import.meta.vitest': 'undefined'
  }
})