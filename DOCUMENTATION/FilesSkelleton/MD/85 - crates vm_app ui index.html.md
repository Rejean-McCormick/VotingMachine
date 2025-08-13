```md
Pre-Coding Essentials (Component: crates/vm_app/ui/index.html, Version/FormulaID: VM-ENGINE v0) — 85/89
```

## 1) Goal & Success

**Goal:** Provide a **minimal, offline** HTML shell that mounts the UI bundle, renders Doc 7 sections in a fixed order, and never references remote assets (fonts/styles/JS/map tiles).
**Success:** App loads **only packaged files**; no HTTP/DNS; MapLibre (if used) reads a **local** style; numbers are presentation-only (one-decimal already formatted upstream).

## 2) Scope

* **In:** Static HTML skeleton, strict CSP for Tauri/offline, section anchors, optional map container.
* **Out:** Any analytics/CDNs/network fetches; runtime math (all values come from Result/RunRecord/FrontierMap).

## 3) Determinism & Offline Rules

* **No external URLs**. All `<link>`/`<script>`/images/fonts/styles/tiles are local, referenced **relatively**.
* Keep section DOM **stable** so renderers can target by `id`/`data-section`.
* One-decimal display is handled in renderers; **do not** compute/round in the page.

## 4) Recommended CSP (Tauri-safe, offline)

* Block network; allow Tauri’s `asset:`/`tauri:` (and `ipc:` if your Tauri version requires it).
* Allow inline styles only if your bundle injects critical CSS (otherwise remove `'unsafe-inline'`).

```html
<meta http-equiv="Content-Security-Policy"
      content="
        default-src 'self' asset: tauri: ipc:;
        img-src 'self' asset: data: blob:;
        style-src 'self' 'unsafe-inline';
        font-src 'self' asset: data:;
        script-src 'self';
        connect-src 'self' asset: tauri: ipc:;
        object-src 'none';
        frame-ancestors 'none';
      ">
```

> If Vite dev needs relaxed rules, your **dev-only** index (not the packaged one) can loosen `script-src` to include `'unsafe-eval'`. Production must stay strict.

## 5) File Layout Contracts

* Place bundled CSS at `./assets/app.css`.
* Place map style at `./maps/style.json` and tiles under `./maps/tiles/...` (if maps used).
* Keep `vite.config.ts` with `base: './'` so asset URLs are **relative**.

---

## 6) **Final `index.html` skeleton (production)**

> Save exactly as `crates/vm_app/ui/index.html`. All paths are **relative** and offline-safe.

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta
      http-equiv="Content-Security-Policy"
      content="
        default-src 'self' asset: tauri: ipc:;
        img-src 'self' asset: data: blob:;
        style-src 'self' 'unsafe-inline';
        font-src 'self' asset: data:;
        script-src 'self';
        connect-src 'self' asset: tauri: ipc:;
        object-src 'none';
        frame-ancestors 'none';
      ">
    <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
    <meta name="color-scheme" content="light dark">
    <title>VM Engine — Report Viewer</title>

    <!-- Local, bundled stylesheet (no external fonts or CDNs) -->
    <link rel="stylesheet" href="./assets/app.css">
  </head>

  <body>
    <noscript>
      This application requires JavaScript (offline, no network). Please enable it to view reports.
    </noscript>

    <!-- App root -->
    <div id="app" role="main" aria-live="polite">
      <!-- Section anchors in Doc 7 order; content injected by bundled JS from artifacts -->

      <section id="sec-cover-snapshot" data-section="cover_snapshot" aria-labelledby="h-cover">
        <h1 id="h-cover" class="visually-hidden">Cover & Snapshot</h1>
      </section>

      <section id="sec-eligibility" data-section="eligibility" aria-labelledby="h-eligibility">
        <h2 id="h-eligibility" class="visually-hidden">Eligibility & Rolls</h2>
      </section>

      <section id="sec-ballot" data-section="ballot" aria-labelledby="h-ballot">
        <h2 id="h-ballot" class="visually-hidden">Ballot Method</h2>
      </section>

      <section id="sec-allocation" data-section="allocation" aria-labelledby="h-allocation">
        <h2 id="h-allocation" class="visually-hidden">Allocation & Aggregation</h2>
      </section>

      <section id="sec-legitimacy" data-section="legitimacy_panel" aria-labelledby="h-legitimacy">
        <h2 id="h-legitimacy" class="visually-hidden">Legitimacy Panel</h2>
      </section>

      <section id="sec-outcome" data-section="outcome_label" aria-labelledby="h-outcome">
        <h2 id="h-outcome" class="visually-hidden">Outcome / Label</h2>
      </section>

      <!-- Optional map panel (only shown when a FrontierMap exists).
           data-map-style points to LOCAL style.json (no remote tiles). -->
      <section id="sec-frontier" data-section="frontier" aria-labelledby="h-frontier" hidden>
        <h2 id="h-frontier" class="visually-hidden">Frontier Map</h2>
        <div id="map"
             data-map-style="./maps/style.json"
             style="width:100%;height:420px"
             role="img"
             aria-label="Frontier map (offline)">
        </div>
      </section>

      <section id="sec-ties" data-section="ties" aria-labelledby="h-ties">
        <h2 id="h-ties" class="visually-hidden">Tie Resolution</h2>
      </section>

      <section id="sec-sensitivity" data-section="sensitivity" aria-labelledby="h-sensitivity">
        <h2 id="h-sensitivity" class="visually-hidden">Sensitivity</h2>
      </section>

      <section id="sec-integrity" data-section="integrity" aria-labelledby="h-integrity">
        <h2 id="h-integrity" class="visually-hidden">Integrity & Reproducibility</h2>
      </section>

      <footer id="fixed-footer" data-section="footer" aria-label="Fixed footer">
        <!-- Renderer writes fixed footer line with IDs from RunRecord/Result -->
      </footer>
    </div>

    <!-- Production bundle (generated by Vite). Keep relative path; no external scripts. -->
    <script type="module" src="./assets/main.js"></script>
  </body>
</html>
```

### 6.a) Dev-only variant (optional)

During local dev with Vite, you typically reference the source entry instead of the built file:

```html
<!-- DEV ONLY: replace the production script line with: -->
<script type="module" src="/src/main.ts"></script>
```

Do **not** ship that line in the packaged app.

---

## 7) Accessibility & UX Notes

* Use `.visually-hidden` (in local CSS) for heading anchors to preserve keyboard landmarks without visual noise.
* `aria-live="polite"` on `#app` helps announce updates after pipeline runs.
* The map gets `role="img"` + label; hide the entire `#sec-frontier` until a FrontierMap exists.

## 8) Failure Policy

* Any absolute `http(s)://` URL or remote font is a policy violation—replace with local files.
* If map assets are missing, keep `#sec-frontier` **hidden**; all other sections must render from artifacts.

## 9) Tests (must pass)

* Launch in Tauri **offline** → no HTTP/DNS; all resources load from packaged paths.
* Renderer populates sections strictly in Doc 7 order; values are already one-decimal.
* If FrontierMap exists, map loads from `./maps/style.json` and `./maps/tiles/...` (local).

That’s the complete, policy-compliant `index.html` skeleton for your engine.
