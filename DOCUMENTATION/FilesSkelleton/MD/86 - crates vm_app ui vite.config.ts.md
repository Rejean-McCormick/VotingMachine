```md
Pre-Coding Essentials (Component: crates/vm_app/ui/vite.config.ts, Version/FormulaID: VM-ENGINE v0) — 86/89
```

## 1) Goal & Success

**Goal:** Emit a fully offline, reproducible static bundle for the desktop app (Tauri).
**Success:** `npm run build` produces `dist/` with **relative** asset URLs (e.g., `./assets/...`), no CDN/external references, deterministic file naming, and zero sourcemaps.

## 2) Scope

* **In:** Vite base path, output dirs, Rollup naming, asset rules, dev-server hardening (dev only), a guard plugin that **fails** on any `http(s)://` imports or CSS URLs.
* **Out:** Tauri backend config (tauri.conf.json), pipeline code, security posture (handled elsewhere).

## 3) Inputs → Outputs

**Inputs:** `index.html`, `src/main.ts`, local CSS/fonts, local MapLibre assets under `ui/public/maps/`.
**Outputs:** `dist/` with `assets/[name]-[hash].js|css|…` and **relative** `base:'./'` so Tauri loads from packaged files.

## 4) Variables

None beyond standard Vite/Rollup knobs. Precision/display rules live in report renderers (Doc 7).

## 5) Functions (config)

TypeScript config + a tiny Vite plugin to block external URLs.

## 6) Algorithm Outline (build config)

* Force `base: './'`, `outDir: 'dist'`, `assetsDir: 'assets'`.
* Disable sourcemaps & inlining variability (`assetsInlineLimit: 0`).
* Stable Rollup names: `entryFileNames`, `chunkFileNames`, `assetFileNames`.
* **Offline guard plugin:** error on `http(s)://` in imports or CSS `url(...)`.
* Dev server: no proxies/CDNs; strict port; don’t auto-open; don’t expose host.

## 7) State Flow

`vite build` → `dist/` → Tauri packages it → App loads all assets locally; backend writes canonical artifacts.

## 8) Determinism & Offline Rules

No network at runtime; all URLs are relative; no external fonts/tiles; reproducible output names (content-hash).

## 9) Edge Cases & Failure Policy

* Any absolute `http(s)://` in JS/CSS → **build error**.
* Map style must reference tiles **relatively** (e.g., `./tiles/...`).
* Sourcemaps off to avoid path leaks / non-reproducible bytes.

## 10) Test Checklist

* `npm run build` works offline; `dist/` references only relative paths.
* Tauri app runs with **no HTTP/DNS**.
* Map renders using `dist/maps/style.json` & local tiles.

---

## 11) `vite.config.ts` (production-ready, offline-safe)

```ts
import { defineConfig } from 'vite';
import { resolve } from 'node:path';

// Blocks any remote URL references at build time.
function offlineGuard() {
  const bad = /^(?:https?:)?\/\//i;

  return {
    name: 'offline-guard',
    enforce: 'pre' as const,

    resolveId(id: string) {
      if (bad.test(id)) {
        throw new Error(`External URL imports are forbidden: ${id}`);
      }
      return null; // continue normal resolution
    },

    transform(code: string, id: string) {
      // Check CSS url(...) and inline @import
      const isCss = /\.(css|scss|sass|less|styl|stylus)$/i.test(id);
      if (isCss) {
        const urlRx = /url\(\s*(['"]?)([^'")]+)\1\s*\)/gi;
        let m: RegExpExecArray | null;
        while ((m = urlRx.exec(code))) {
          const url = m[2].trim();
          if (bad.test(url)) {
            throw new Error(`External URL in CSS is forbidden: ${url} (in ${id})`);
          }
        }
        const importRx = /@import\s+(['"])([^'"]+)\1/gi;
        while ((m = importRx.exec(code))) {
          const url = m[2].trim();
          if (bad.test(url)) {
            throw new Error(`External @import is forbidden: ${url} (in ${id})`);
          }
        }
      }
      return null;
    }
  };
}

export default defineConfig(({ mode }) => ({
  // Ensure relative URLs so packaged files load from app resources.
  base: './',

  root: resolve(__dirname),
  publicDir: resolve(__dirname, 'public'),

  build: {
    outDir: resolve(__dirname, 'dist'),
    assetsDir: 'assets',

    // Reproducibility knobs
    sourcemap: false,
    assetsInlineLimit: 0,   // avoid unpredictable data: URIs
    cssCodeSplit: true,
    minify: 'esbuild',
    target: 'es2020',
    emptyOutDir: true,

    rollupOptions: {
      output: {
        entryFileNames: 'assets/[name]-[hash].js',
        chunkFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash][extname]',
        manualChunks: undefined,   // avoid env-dependent chunking strategies
        compact: true,
        // Keep stable semantics
        preserveModules: false
      }
    }
  },

  // Freeze any env-dependent code paths in a deterministic way if needed.
  define: {
    __BUILD_ENV__: JSON.stringify(mode)
  },

  plugins: [offlineGuard()],

  // Dev-only hardening (production loads via Tauri, not this server).
  server: {
    strictPort: true,
    open: false,
    host: false,       // do not expose externally
    proxy: undefined,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  },

  preview: {
    port: 4173,
    strictPort: true
  },

  esbuild: {
    legalComments: 'none'
  }
}));
```

### Notes

* Put map assets under `ui/public/maps/` (e.g., `public/maps/style.json`, `public/maps/tiles/...`).
  In `style.json`, reference tiles **relatively**: `"url": "./tiles/world.mbtiles"` (or vector tile `pmtiles://` served locally if wrapped).
* Ensure `index.html` script and CSS links are relative (e.g., `./assets/main.js`, **not** absolute).
