import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  assetsInclude: ['**/*.brs', '**/*.brz'],
  server: { port: 31401, open: true },
  build: { outDir: 'dist/public' },
});
