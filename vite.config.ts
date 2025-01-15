import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
// import devtools from 'solid-devtools/vite';
import basicSsl from '@vitejs/plugin-basic-ssl'

export default defineConfig({
  plugins: [
    /* 
    Uncomment the following line to enable solid-devtools.
    For more info see https://github.com/thetarnav/solid-devtools/tree/main/packages/extension#readme
    */
    // devtools(),
    solidPlugin(),
    // basicSsl(),
  ],
  // base: '/',
  server: {
    port: 3000,
    host: '0.0.0.0',
    // proxy: {
    //   '/api': {
    //     target: 'http://127.0.0.1:8080',
    //     changeOrigin: true,
    //   },
    // },
  },
  preview: {
    port: 4173,
    // proxy: {
    //   '/api': {
    //     target: 'http://127.0.0.1:8080',
    //     changeOrigin: true
    //   },
    // }
  },
  // assetsInclude: ['./src/assets/favicon.ico', './src/meme'],
  build: {
    assetsInlineLimit: 0,
    manifest: true,
    outDir: 'dist/vite',
    emptyOutDir: true, // also necessary
    rollupOptions: {
      input: "./app/index.tsx",
      output: {
        entryFileNames: `assets/[hash].js`,
        chunkFileNames: `assets/[hash].js`,
        assetFileNames: `assets/[hash].[ext]`,
      }
    },
    target: 'esnext',
  },
});
