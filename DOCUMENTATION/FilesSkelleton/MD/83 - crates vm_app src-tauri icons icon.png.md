```md
Perfect Skeleton Sheet — crates/vm_app/src-tauri/icons/icon.png — 83/89
(Aligned with VM-ENGINE v0: offline, bundled, deterministic)
```

## 1) Goal & Success

**Goal:** Ship a **local** desktop-app icon with the Tauri bundle—no network fetch, no build-time tooling that reaches the internet.
**Success:** App displays the icon on Win/macOS/Linux; packaging uses only local files; icon choice never changes pipeline artifacts or their hashes.

## 2) Scope

* **In:** A single PNG at `crates/vm_app/src-tauri/icons/icon.png`, referenced by Tauri bundling.
* **Out:** In-report glyphs/visuals (Doc 7) — handled by renderers, not this asset.

## 3) Inputs → Outputs

* **Input:** `src-tauri/icons/icon.png` (committed binary file).
* **Output:** Icon embedded in installer/app bundles (DMG/MSI/AppImage/DEB), no runtime downloads.

## 4) Quality Bar & Constraints

* **Format:** PNG, **sRGB**, 8-bit, no color profile surprises; fully **offline**.
* **Dimensions:** Prefer **1024×1024** square source; Tauri/bundlers downscale as needed (Windows ICO, macOS ICNS, Linux PNGs).
* **Alpha:** Allowed; avoid semi-transparent edges that blur on downscale—pad to safe margins.
* **Determinism:** File content must be stable; no pipeline generates it at build time.
* **Licensing:** Icon must be owned or permissively licensed; include attribution in repo `LICENSES/` if required.

## 5) Placement & Bundler Reference

* Put file at:

  ```
  crates/vm_app/src-tauri/icons/icon.png
  ```
* Ensure `tauri.conf.json` (82/89) includes resources/bundle icons. Example (keep offline settings intact):

  ```json
  {
    "tauri": {
      "bundle": {
        "resources": ["../ui/dist/**","../assets/**","../maps/**","icons/icon.png"]
      }
    }
  }
  ```

  *If your Tauri version supports an explicit `icon` key, set it to `"icons/icon.png"`; otherwise keep it in `resources` so the bundler picks it up for platform icon generation.*

## 6) Build & Packaging Notes

* Build with `--locked`; **no** image downloads or conversions at build-time.
* If you maintain platform-specialized icons (ICO/ICNS), place them alongside PNG and reference them **locally** (optional; PNG 1024² usually suffices—Tauri generates formats).

## 7) Determinism & Safety

* Icon presence **must not** affect Result/RunRecord canonical bytes.
* No remote fonts/tiles/styles via icon metadata; keep the file clean (strip ancillary text chunks if any).
* Store the canonical version in Git LFS only if your policy requires; otherwise plain Git is fine.

## 8) Edge Cases & Failure Policy

* **Missing icon:** Packaging may fall back to defaults or fail—treat as build error and restore the local file.
* **Wrong path:** If the bundler cannot find the icon (`icons/icon.png`), fix the relative path in `tauri.conf.json`.
* **Oversized/CMYK:** Re-export to sRGB 8-bit; CMYK/16-bit can render inconsistently.

## 9) Test Checklist (must pass)

* App bundles on Win/macOS/Linux show the intended icon.
* Build and run **fully offline** (no HTTP/DNS).
* Removing the icon breaks packaging (expected); restoring it fixes the build.
* Result/RunRecord hashes unchanged whether icon is 512² vs 1024² (icon never enters canonical artifacts).

## 10) Optional Nice-to-haves

* Keep a `source/` SVG (local) and export a deterministic PNG (fixed renderer/version); **do not** export at build-time.
* Provide a tiny favicon `ui/dist/favicon.png` for the webview shell (also local, optional).

