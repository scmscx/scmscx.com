import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

// Transitional Cloudflare Pages build (migration Phase 1).
//
// Produces a self-contained SPA shell — `index.html` with vite-fingerprinted
// JS/CSS/favicon refs injected — into `dist/pages`, WITHOUT touching the existing
// `vite.config.ts` build (`dist/vite` + `.vite/manifest.json`) that the Rust
// handlebars server still consumes during the migration. Both builds coexist:
// `npm run build` feeds Rust, `npm run build:pages` produces the Pages bundle.
//
// When the Rust SSR/static-serving path is deleted in a later phase, this becomes
// the single canonical build and `vite.config.ts` goes away.
// See docs/cloudflare-pages-migration-plan.md.
export default defineConfig({
  plugins: [solidPlugin()],
  // Emit the static extras Pages serves from the deploy root: robots.txt, the
  // legacy /map & /replay redirect pages, and the Cloudflare `_routes.json` /
  // `_redirects` control files. (`_`-prefixed control files are inert to the Rust
  // path that also copies this dir.)
  publicDir: 'app/web/public',
  build: {
    assetsInlineLimit: 0,
    outDir: 'dist/pages',
    emptyOutDir: true,
    rollupOptions: {
      // HTML entry (vite's native mode) — it injects hashed asset refs itself,
      // unlike the handlebars templates which looked them up from the manifest.
      input: './index.html',
      output: {
        entryFileNames: `assets/[hash].js`,
        chunkFileNames: `assets/[hash].js`,
        assetFileNames: `assets/[hash].[ext]`,
      },
    },
    target: 'esnext',
  },
});
