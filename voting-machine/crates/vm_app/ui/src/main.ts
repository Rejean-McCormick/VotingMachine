/* crates/vm_app/ui/src/main.ts
 * Offline UI bootstrap & renderer (no network, no outcome math).
 */
import { invoke } from '@tauri-apps/api/tauri';
import { readTextFile } from '@tauri-apps/api/fs';
import { join } from '@tauri-apps/api/path';

/* ===== Minimal types (subset—align with backend 81/89) ===== */
type EngineInfo = { formula_id: string; engine_version: string; targets: string[] };

type LoadedContextSummary = {
  registry_id: string;
  ballot_or_tally_id: string;
  parameter_set_id: string;
  has_adjacency: boolean;
};

type RunSummary = {
  result_id: string;
  run_id: string;
  frontier_id?: string | null;
  label: 'Decisive' | 'Marginal' | 'Invalid';
};

type ResultDb = {
  id: string;
  label: { label: 'Decisive' | 'Marginal' | 'Invalid'; reason?: string };
  gates?: any;         // Legitimacy panel payloads (UI treats as opaque)
  snapshot?: any;      // optional snapshot block
  meta?: any;          // optional misc
};

type RunRecordDb = {
  id: string;
  engine: { formula_id: string; version: string };
  inputs: { registry_id: string; parameter_set_id: string; ballot_tally_id?: string };
  determinism?: { rng_seed?: string | null };
  timestamps: { started_utc: string; finished_utc: string };
};

type FrontierMapDb = {
  id: string;
  summary?: {
    by_status?: Record<string, number>;
    flags?: { mediation: number; enclave: number; protected_blocked: number; quorum_blocked: number };
  };
  units?: Record<string, unknown>;
};

/* ===== Bootstrap ===== */
async function bootstrap(): Promise<void> {
  const root = document.getElementById('app');
  if (!root) throw new Error('#app not found');

  // Engine info (provenance)
  try {
    const info = await invoke<EngineInfo>('cmd_engine_info');
    setText('#engine-ident', `${info.formula_id} • ${info.engine_version}`);
  } catch (err) {
    showError('Failed to get engine info', err);
  }

  // Wire demo buttons (IDs are optional; no-op if missing)
  bind('#btn-run', async () => {
    clearError();
    try {
      // Collect paths from inputs (all local)
      const registry = getValue('#in-registry');
      const ballots_or_tally = getValue('#in-ballots') || getValue('#in-tally');
      const params = getValue('#in-params');
      const manifest = getValue('#in-manifest') || undefined;
      const outDir = getValue('#in-out') || '.';

      if (!registry || !params || !ballots_or_tally) {
        throw new Error('Missing required paths (registry, params, and ballots/tally).');
      }

      const run = await runPipeline({ registry, ballots_or_tally, params, manifest }, outDir);

      // The backend writes canonical JSON into outDir; compute paths deterministically.
      const resultPath = await join(outDir, 'result.json');
      const runPath = await join(outDir, 'run_record.json');
      const frontierPath = await join(outDir, 'frontier_map.json');

      // Read artifacts (from local disk; canonical JSON)
      const res = await readArtifact<ResultDb>(resultPath);
      const rr = await readArtifact<RunRecordDb>(runPath);
      // Frontier is optional; try to read but tolerate absence.
      const fr = await readArtifact<FrontierMapDb>(frontierPath).catch(() => null as FrontierMapDb | null);

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
      const rr = await readArtifact<RunRecordDb>(runPath);
      // Optional FrontierMap path input
      const frPath = getValue('#in-frontier');
      const fr = frPath ? await readArtifact<FrontierMapDb>(frPath).catch(() => null as FrontierMapDb | null) : null;
      renderReport(res, rr, fr ?? undefined);
      setText('#last-run', `Loaded ${res.id} / ${rr.id}`);
    } catch (err) {
      showError('Open artifacts failed', err);
    }
  });
}

/* ===== Backend IPC wrappers ===== */
async function loadBundle(paths: {
  registry?: string;
  ballots_or_tally?: string;
  params?: string;
  manifest?: string;
}): Promise<LoadedContextSummary> {
  return invoke<LoadedContextSummary>('cmd_load_inputs', paths);
}

async function runPipeline(
  paths: { registry?: string; ballots_or_tally?: string; params?: string; manifest?: string },
  outDir: string
): Promise<RunSummary> {
  return invoke<RunSummary>('cmd_run_pipeline', { ...paths, out_dir: outDir });
}

/* Local JSON read (scoped by tauri.conf.json fs scope) */
async function readArtifact<T>(path: string): Promise<T> {
  const txt = await readTextFile(path);
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

  // 2) Eligibility & Rolls (values come from artifacts—do not recompute)
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
    setVisible('#sec-frontier', true);
    const f = frontier.summary?.flags ?? { mediation: 0, enclave: 0, protected_blocked: 0, quorum_blocked: 0 };
    setText(
      '#frontier-flags',
      `Mediation ${f.mediation} • Enclave ${f.enclave} • Protected ${f.protected_blocked} • Quorum-blocked ${f.quorum_blocked}`
    );
    // Optional map (uses local style; leave empty if assets absent)
    initMap('map', './maps/style.json'); // must be packaged locally
  } else {
    setVisible('#sec-frontier', false);
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
export function formatPercent(numer: bigint, denom: bigint): string {
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
  if (!gl) { setVisible('#sec-frontier', false); return; }
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
    setVisible('#sec-frontier', false);
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
  const detail = (err as any)?.message ?? String(err ?? '');
  box.textContent = `${msg}: ${detail}`;
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

/* ===== Kick it off ===== */
document.addEventListener('DOMContentLoaded', () => {
  bootstrap().catch(err => showError('Bootstrap failed', err));
});
