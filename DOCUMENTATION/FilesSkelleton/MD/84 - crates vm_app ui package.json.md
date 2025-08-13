```md
Pre-Coding Essentials (Component: crates/vm_app/ui/package.json, Version/FormulaID: VM-ENGINE v0) — 84/89
```

## 1) Goal & Success

**Goal:** Define the UI workspace manifest so the desktop app ships a **fully bundled, offline** UI (no external fonts/styles/tiles) and renders presentation-only views with **one-decimal** percentages.
**Success:** `npm run build` produces a static bundle Vite can serve to Tauri with **zero runtime network fetches**; dependency versions are **pinned** and builds are reproducible.

## 2) Scope

* **In:** `package.json` metadata, engines, scripts (build/dev/lint/typecheck), pinned deps for Vite+TS and (optionally) MapLibre (loaded from **local** styles/tiles).
* **Out:** Backend (Tauri) wiring and security policy (lives in `tauri.conf.json`), pipeline math (Rust crates), report content (Doc 7).

## 3) Inputs → Outputs

* **Inputs:** UI sources under `crates/vm_app/ui/`, local assets (`/assets/*`, `/maps/*` styles/tiles/fonts).
* **Outputs:** Static bundle (e.g., `dist/`) consumed by Tauri; **all URLs are relative/local**.

## 4) Files (minimal)

* `package.json` (this file) + lockfile (`package-lock.json` or `pnpm-lock.yaml`).
* `vite.config.ts` with `base: './'` to keep **relative** asset paths (critical for offline packaging).

## 5) package.json — Skeleton (pinned, offline-friendly)

```json
{
  "name": "vm-ui",
  "version": "0.1.0",
  "private": true,
  "description": "Deterministic, offline UI bundle for the VM Engine app",
  "license": "Apache-2.0 OR MIT",
  "type": "module",
  "packageManager": "npm@10.8.1",
  "engines": {
    "node": ">=18.18 <23",
    "npm": ">=9"
  },
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview --strictPort --port 4173",
    "lint": "eslint \"src/**/*.{ts,tsx,js,jsx}\" && stylelint \"src/**/*.{css,scss}\"",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "maplibre-gl": "3.6.2"
  },
  "devDependencies": {
    "vite": "5.4.2",
    "typescript": "5.5.4",
    "@types/node": "20.14.10",
    "eslint": "8.57.0",
    "@typescript-eslint/eslint-plugin": "7.16.1",
    "@typescript-eslint/parser": "7.16.1",
    "stylelint": "16.6.1",
    "stylelint-config-standard": "36.0.1",
    "postcss": "8.4.41",
    "autoprefixer": "10.4.19"
  },
  "browserslist": [
    "chrome >= 110",
    "edge >= 110",
    "safari >= 16",
    "ios_saf >= 16"
  ],
  "sideEffects": false
}
```

### Notes

* **Pin** exact versions (no `^`/`~`). Commit the **lockfile** for reproducibility.
* Keep `maplibre-gl` only if you actually render maps; ensure styles/tiles are **local files** referenced relatively (e.g., `maps/style.json`, `maps/tiles/{z}/{x}/{y}.pbf`).

## 6) Implementation Guardrails

* **Vite config (must-do):**

  * `base: './'` for relative URLs.
  * No plugins that fetch network resources at build or runtime.
* **Assets:**

  * Bundle fonts/styles/tiles locally (no CDNs). Import via relative paths.
  * If using web fonts, ship `.woff2` locally and reference them in CSS with relative URLs.
* **Rendering precision:** The UI **displays** one-decimal percentages already prepared by report code; **do not** recompute or re-round in the UI.

## 7) Determinism & Offline Rules

* Build determinism: pinned versions + lockfile; avoid post-install scripts that mutate outputs.
* Runtime offline: **no HTTP/DNS**. All resources must resolve to packaged files.
* Do not include analytics/telemetry deps.

## 8) Edge Cases & Failure Policy

* Any absolute `http(s)://` reference in built assets ⇒ **fail the build** and replace with local files.
* If maps are disabled, you may remove `maplibre-gl` entirely to reduce surface area.
* If Tauri cannot find assets due to non-relative paths, fix `vite.config.ts` `base` and asset imports.

## 9) Test Checklist (must pass)

* `npm ci && npm run build` succeeds with the lockfile and **no network at runtime**.
* Inspect `dist/` — all asset URLs are **relative**; no external hosts.
* Packaged app renders reports with **one-decimal** percents; IDs pulled solely from Result/RunRecord.
* Map view (if present) loads **local** style/tiles; works offline.
