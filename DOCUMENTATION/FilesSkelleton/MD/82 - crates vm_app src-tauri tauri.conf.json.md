```md
Perfect Skeleton Sheet — crates/vm_app/src-tauri/tauri.conf.json — 82/89  
(Aligned with VM-ENGINE v0: offline, sandboxed, deterministic)
```

## 1) Goal & Success

**Goal:** Lock a Tauri desktop app config that is strictly offline (no updater/telemetry/network), sandboxed FS (allowlist only), and bundles all UI/assets locally.
**Success:** App runs on Win/macOS/Linux using only packaged files; HTTP/DNS APIs are disabled; canonical artifacts produced by the engine remain unaffected.

## 2) Scope

* **In:** `tauri.security`, `tauri.allowlist`, `tauri.updater=false`, `tauri.fs.scope`, local `build.distDir`, bundled resources (UI, fonts, map tiles/styles).
* **Out:** Backend code (81/89), UI build, map data contents.

## 3) Inputs → Outputs

* **Inputs:** Local UI bundle (`ui/dist`), local fonts/styles/map tiles.
* **Outputs:** Offline desktop app; backend commands read/write only within FS scope.

## 4) Determinism & Safety Rules

* Disable network/updater/shell.
* Strict CSP forbidding remote loads (`connect-src 'none'`).
* FS allowlist only (no symlink escape).
* All assets bundled; no remote fonts/tiles.

---

## 5) Config Skeleton (drop-in JSON)

> Adjust identifiers/paths to your workspace; keep **booleans false** where shown to stay offline.

```json
{
  "package": {
    "productName": "VM Engine",
    "version": "0.0.0"
  },
  "build": {
    "beforeBuildCommand": "",
    "beforeDevCommand": "",
    "distDir": "../ui/dist",
    "devPath": "../ui/dist"
  },
  "tauri": {
    "macOSPrivateApi": false,
    "bundle": {
      "active": true,
      "identifier": "org.vm.engine",
      "targets": ["dmg", "msi", "appimage", "deb"],
      "resources": [
        "../ui/dist/**",
        "../assets/**",
        "../maps/**"          // tiles/styles/fonts packed locally
      ],
      "windows": {
        "wix": { "language": "en-US" }
      }
    },
    "updater": {
      "active": false
    },
    "allowlist": {
      "all": false,

      "shell": { "all": false },
      "http":  { "all": false },     // disables fetch/http plugin
      "net":   { "all": false },     // v2: ensure network is off

      "process": { "all": false },
      "notification": { "all": false },
      "globalShortcut": { "all": false },
      "os": { "all": false },
      "path": { "all": false },

      "dialog": {
        "open": true,
        "save": true
      },

      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "createDir": true,
        "exists": true,
        "scope": [
          "$APP/**",
          "$RESOURCE/**",
          "$APPDATA/**",
          "$HOME/Documents/VM/**"
        ]
      },

      "window": {
        "all": false,
        "setTitle": true,
        "show": true,
        "hide": true,
        "close": true
      },

      "event": { "all": true },      // IPC events only; no network
      "clipboard": { "all": false }
    },
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: blob:; media-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; font-src 'self' data:; script-src 'self'; connect-src 'none'; frame-src 'none'; object-src 'none'"
    },
    "windows": [
      {
        "title": "VM Engine",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false,
        "visible": true,
        "center": true
      }
    ]
  }
}
```

### Notes on the skeleton

* **`allowlist.http/net/process/shell=false`** ensures no outbound calls / command execution.
* **`fs.scope`**: restrict to app dirs/resources and (optionally) a user doc folder. Add/trim entries to your policy.
* **`security.csp`**: forbids external connections; UI must reference only packaged assets.
* **`build.distDir/devPath`**: both point to the built bundle to keep dev runs offline.
* **`bundle.resources`**: include all fonts/styles/map tiles your UI needs.

---

## 6) Edge Cases & Policy

* Any UI reference to remote assets will be blocked by CSP; bundle them or remove.
* Paths outside `fs.scope` must error; do not widen scope casually.
* Keep `updater.active=false` to avoid accidental network checks.

## 7) Quick Test Checklist

* Launch offline: no HTTP/DNS in dev tools/network logs.
* Open/Save dialogs constrained to allowed folders.
* Reports/maps render using packaged assets only.
* Backend commands succeed on local files; network APIs are unavailable.
