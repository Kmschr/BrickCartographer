import { defineConfig } from 'vite';

export default defineConfig({
  assetsInclude: ['**/*.brs', '**/*.brz'],
  server: { port: 31401, open: true },
  build: { outDir: 'dist/public' },
});
