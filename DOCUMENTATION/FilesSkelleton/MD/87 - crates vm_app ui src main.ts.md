```md
Pre-Coding Essentials (Component: crates/vm_app/ui/src/main.ts, Version/FormulaID: VM-ENGINE v0) — 87/89
```

## 1) Goal & Success

**Goal:** Bootstrap the desktop UI, call Tauri backend commands, and render Doc 7 report sections strictly from local artifacts (Result, RunRecord, optional FrontierMap)—**no network**, **no outcome math**.
**Success:** App mounts offline; backend is invoked deterministically; UI shows sections in fixed order with **one-decimal** presentation only.

## 2) Scope

* **In:** App init; Tauri IPC; safe file reads; one-decimal formatting; optional local MapLibre glue.
* **Out:** Any computation of allocations/gates/frontier; any remote fetch/CDN assets; RNG/time use.

## 3) Inputs → Outputs

* **Inputs:** Tauri commands (`cmd_engine_info`, `cmd_load_inputs`, `cmd_run_pipeline`, `cmd_export_report`, `cmd_hash_artifacts`), plus canonical JSON artifacts read from local disk.
* **Outputs:** DOM updates for Doc 7 sections (Cover/Snapshot → Eligibility → Ballot → Legitimacy Panel → Outcome → Frontier → Sensitivity → Integrity).

## 4) Entities/Tables (minimal)

UI-only view models derived from `Result`, `RunRecord`, `FrontierMap` (exact shapes live in core crates).

## 5) Variables (only ones used here)

None configurable; UI follows offline & one-decimal rules.

## 6) Functions (signatures)

```ts
async function bootstrap(): Promise<void>;
async function loadBundle(paths: { registry?: string; ballots?: string; params?: string; manifest?: string; }): Promise<LoadedContextSummary>;
async function runPipeline(paths: { registry?: string; ballots?: string; params?: string; manifest?: string; }, outDir: string): Promise<RunSummary>;
async function readArtifact<T>(path: string): Promise<T>;
function renderReport(result: ResultDb, run: RunRecordDb, frontier?: FrontierMapDb): void;
function formatPercent(numer: bigint, denom: bigint): string; // one-decimal, integer-only
function initMap(containerId: string, styleUrl: string): void; // local style/tiles only
```

## 7) Algorithm Outline

* `bootstrap`:

  * Wait DOM ready, mount `#app`, fetch `engine_info`, wire UI buttons.
* `loadBundle` (optional preflight): invoke backend to echo IDs/labels.
* `runPipeline`: invoke backend to execute the fixed pipeline; receive `{ result_id, run_id, label, result_path, run_path, frontier_path? }`.
* `readArtifact`: local read via Tauri FS (scoped); JSON parse.
* `renderReport`: populate sections **only** from artifacts; include mandatory approval-denominator sentence for approval ballots; one-decimal display; do not recompute outcomes.
* `initMap`: if `frontier` exists and local map assets are bundled, initialize MapLibre from a **relative** `style.json`; otherwise hide panel.

## 8) Determinism & Numeric Rules

* No network calls; no OS time used in UI.
* One-decimal formatting done **once** at display; integer math (BigInt) to avoid float drift.
* Stable section order; read-only rendering.

## 9) Edge Cases & Failure Policy

* Missing artifacts: show deterministic error banner; do **not** fetch remotely.
* Invalid runs: still render Cover/Eligibility/Ballot + Panel with ❌ and Outcome “Invalid (…)”; omit Frontier.
* Map assets missing: hide map panel; do not attempt remote tiles.

---

## 10) Reference Implementation Skeleton (TypeScript)

```ts
/* crates/vm_app/ui/src/main.ts
 * Offline UI bootstrap & renderer (no network, no outcome math).
 */

import { invoke } from '@tauri-apps/api/tauri';
import { readTextFile } from '@tauri-apps/api/fs';
import { join } from '@tauri-apps/api/path';

/* ===== Minimal types (subset—align with core crates DB types) ===== */
type EngineInfo = { formula_id: string; engine_version: string; targets: string[] };
type LoadedContextSummary = { registry_id: string; tally_id?: string; params_id: string; options_count: number; units_count: number };
type RunSummary = {
  result_id: string; run_id: string; label: 'Decisive' | 'Marginal' | 'Invalid';
  result_path: string; run_record_path: string; frontier_path?: string | null;
};

type ResultDb = {
  id: string;
  label: { label: 'Decisive' | 'Marginal' | 'Invalid'; reason: string };
  gates?: any; // Legitimacy panel payloads (UI treats as opaque)
  snapshot?: any; // optional snapshot block
  meta?: any;
  // Add fields as needed by the UI
};

type RunRecordDb = {
  id: string;
  engine: { formula_id: string; version: string };
  inputs: { registry_id: string; parameter_set_id: string; ballot_tally_id?: string };
  determinism: { rng_seed?: string | null };
  timestamps: { started_utc: string; finished_utc: string };
};

type FrontierMapDb = {
  id: string;
  summary: { by_status: Record<string, number>; flags: { mediation: number; enclave: number; protected_blocked: number; quorum_blocked: number } };
  units?: Record<string, any>;
};

/* ===== Bootstrap ===== */
async function bootstrap(): Promise<void> {
  const root = document.getElementById('app');
  if (!root) throw new Error('#app not found');

  // Engine info (provenance)
  let info: EngineInfo | null = null;
  try {
    info = await invoke<EngineInfo>('cmd_engine_info');
    setText('engine-ident', `${info.formula_id} • ${info.engine_version}`);
  } catch (err) {
    showError('Failed to get engine info', err);
  }

  // Wire demo buttons (IDs expected in index.html)
  bind('#btn-run', async () => {
    clearError();
    try {
      // Collect paths from inputs (all local)
      const registry = getValue('#in-registry');
      const ballots = getValue('#in-ballots');
      const params  = getValue('#in-params');
      const manifest = getValue('#in-manifest') || undefined;
      const outDir = getValue('#in-out') || (await join(await cwd(), 'out'));

      const run = await runPipeline({ registry, ballots, params, manifest }, outDir);

      // Read artifacts (from local disk; backend wrote canonical JSON)
      const res  = await readArtifact<ResultDb>(run.result_path);
      const rr   = await readArtifact<RunRecordDb>(run.run_record_path);
      const fr   = run.frontier_path ? await readArtifact<FrontierMapDb>(run.frontier_path).catch(() => null) : null;

      renderReport(res, rr, fr ?? undefined);
      setText('#last-run', `Result ${run.result_id} • Run ${run.run_id} • ${run.label}`);
    } catch (err) {
      showError('Pipeline failed', err);
    }
  });

  bind('#btn-open', async () => {
    clearError();
    try {
      const resPath = getValue('#in-result');
      const runPath = getValue('#in-runrecord');
      if (!resPath || !runPath) throw new Error('Select both Result and RunRecord paths');

      const res = await readArtifact<ResultDb>(resPath);
      const rr  = await readArtifact<RunRecordDb>(runPath);

      // Optional FrontierMap path input
      const frPath = getValue('#in-frontier');
      const fr = frPath ? await readArtifact<FrontierMapDb>(frPath).catch(() => null) : null;

      renderReport(res, rr, fr ?? undefined);
      setText('#last-run', `Loaded ${res.id} / ${rr.id}`);
    } catch (err) {
      showError('Open artifacts failed', err);
    }
  });
}

/* ===== Backend IPC wrappers ===== */

async function loadBundle(paths: { registry?: string; ballots?: string; params?: string; manifest?: string; }): Promise<LoadedContextSummary> {
  return invoke<LoadedContextSummary>('cmd_load_inputs', paths);
}

async function runPipeline(
  paths: { registry?: string; ballots?: string; params?: string; manifest?: string; },
  outDir: string
): Promise<RunSummary> {
  return invoke<RunSummary>('cmd_run_pipeline', { ...paths, outDir });
}

/* Local JSON read (scoped by tauri.conf.json fs scope) */
async function readArtifact<T>(path: string): Promise<T> {
  const txt = await readTextFile(path, { dir: undefined }); // absolute path within scope
  return JSON.parse(txt) as T;
}

/* ===== Rendering (Doc 7 order; presentation-only) ===== */

function renderReport(result: ResultDb, run: RunRecordDb, frontier?: FrontierMapDb): void {
  // 1) Cover & Snapshot
  setText('#cover-label', `${result.label.label}`);
  setText('#cover-reason', result.label.reason ?? '');
  setText('#snapshot-engine', `${run.engine.formula_id} • ${run.engine.version}`);
  setText('#snapshot-runid', run.id);
  setText('#snapshot-seed', run.determinism?.rng_seed ? `Seed ${run.determinism.rng_seed}` : 'Seed — n/a');

  // 2) Eligibility & Rolls (values come from Result—do not recompute)
  // Bind whatever fields are present; keep UI tolerant of missing data.
  setText('#eligibility-policy', getSafePath(result, ['snapshot', 'eligibility_policy']) ?? '—');

  // 3) Ballot (method paragraph)
  const ballotType = getSafePath(result, ['snapshot', 'ballot_type']);
  setText('#ballot-type', ballotType ?? '—');
  const approvalSentenceNeeded = ballotType === 'approval';
  setVisible('#approval-denominator-note', !!approvalSentenceNeeded);

  // 4) Legitimacy Panel (raw numbers copied; UI does not recompute)
  const panel = result.gates ?? {};
  setText('#gate-quorum', renderGate(panel.quorum));
  setText('#gate-majority', renderGate(panel.majority));
  setText('#gate-double', renderGate(panel.double_majority));
  setText('#gate-symmetry', renderGate(panel.symmetry));

  // 5) Outcome / Label
  setText('#outcome-label', result.label.label);
  setText('#outcome-reason', result.label.reason ?? '');

  // 6) Frontier (only if artifact exists)
  if (frontier) {
    setVisible('#frontier-section', true);
    const flags = frontier.summary?.flags ?? { mediation: 0, enclave: 0, protected_blocked: 0, quorum_blocked: 0 };
    setText('#frontier-flags',
      `Mediation ${flags.mediation} • Enclave ${flags.enclave} • Protected ${flags.protected_blocked} • Quorum-blocked ${flags.quorum_blocked}`
    );

    // Optional map (uses local style; leave empty if assets absent)
    const containerId = 'map';
    const styleUrl = './maps/style.json'; // must be packaged locally
    initMap(containerId, styleUrl);
  } else {
    setVisible('#frontier-section', false);
  }

  // 7) Sensitivity (UI shows "N/A" unless compare scenarios included in Result)
  const sens = getSafePath(result, ['snapshot', 'sensitivity']) ?? 'N/A (not executed)';
  setText('#sensitivity', String(sens));

  // 8) Integrity & Reproducibility (IDs/UTC only)
  setText('#integrity-result-id', result.id);
  setText('#integrity-run-id', run.id);
  setText('#integrity-start', run.timestamps.started_utc);
  setText('#integrity-finish', run.timestamps.finished_utc);
}

/* Render a single gate row from payload already computed upstream. */
function renderGate(g: any): string {
  if (!g) return '—';
  // Prefer preformatted strings if present; otherwise display compact raw.
  const pct = g.support_pct_str ?? g.turnout_pct_str ?? '';
  const thr = g.threshold_str ?? '';
  const pass = g.pass === true ? 'Pass' : (g.pass === false ? 'Fail' : '—');
  return [pct && `Value ${pct}`, thr && `vs ${thr}`, pass].filter(Boolean).join(' — ');
}

/* ===== Formatting helpers (display only; integer math) ===== */

/** One-decimal percent string using integers (round half up). */
function formatPercent(numer: bigint, denom: bigint): string {
  if (denom === 0n) return '0.0%';
  const scaled = numer * 1000n;                  // percent*10
  const tenths = (scaled + denom / 2n) / denom;  // rounded to nearest tenth
  const whole = tenths / 10n;
  const dec = tenths % 10n;
  return `${whole}.${dec}%`;
}

/* ===== Map (optional; local assets only) ===== */

function initMap(containerId: string, styleUrl: string): void {
  const el = document.getElementById(containerId);
  if (!el) return;
  // Expect a locally-bundled MapLibre GL (no CDN). If absent, hide panel.
  const gl = (window as any).maplibregl;
  if (!gl) { setVisible('#frontier-map-wrap', false); return; }

  try {
    // @ts-ignore minimal init; styleUrl must point to a local packaged file
    const map = new gl.Map({
      container: containerId,
      style: styleUrl,
      attributionControl: false,
      interactive: false
    });
    map.on('error', () => { /* keep silent; assets may be intentionally absent */ });
  } catch {
    setVisible('#frontier-map-wrap', false);
  }
}

/* ===== Tiny DOM utilities ===== */

function bind(sel: string, fn: () => void) {
  const el = document.querySelector<HTMLButtonElement>(sel);
  if (el) el.addEventListener('click', fn);
}

function setText(sel: string, txt: string) {
  const el = typeof sel === 'string' ? document.querySelector<HTMLElement>(sel) : null;
  if (el) el.textContent = txt ?? '';
}

function setVisible(sel: string, on: boolean) {
  const el = document.querySelector<HTMLElement>(sel);
  if (el) el.style.display = on ? '' : 'none';
}

function getValue(sel: string): string {
  const el = document.querySelector<HTMLInputElement>(sel);
  return (el?.value ?? '').trim();
}

function showError(msg: string, err: unknown) {
  const box = document.getElementById('error');
  if (!box) return;
  box.textContent = `${msg}: ${String((err as any)?.message ?? err)}`;
  box.style.display = '';
}

function clearError() {
  const box = document.getElementById('error');
  if (box) box.style.display = 'none';
}

/* Safe nested getter without recomputation */
function getSafePath(o: any, path: Array<string | number>): any {
  return path.reduce((acc, k) => (acc && k in acc ? acc[k] : undefined), o);
}

/* Placeholder for cwd() when needed (optional). Prefer letting backend provide paths. */
async function cwd(): Promise<string> {
  // Using tauri/path if required; here we keep a no-op to avoid platform drift.
  return '.';
}

/* ===== Kick it off ===== */
document.addEventListener('DOMContentLoaded', () => {
  bootstrap().catch(err => showError('Bootstrap failed', err));
});
```

### Notes

* **No network:** this file never fetches remote URLs; it relies solely on Tauri IPC and local filesystem reads within the configured scope.
* **One-decimal only:** `formatPercent` is provided for UI-only formatting when a raw numerator/denominator must be displayed; if artifacts already contain formatted strings, the UI uses them **as-is** (no double rounding).
* **Map:** `initMap` assumes a **bundled** MapLibre build and a **local** `style.json` (e.g., `ui/public/maps/style.json`). If assets are absent, the panel is hidden gracefully.
