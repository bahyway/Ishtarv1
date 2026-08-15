// AnuGovernor browser face. No framework: fetches /api/state, renders the
// DOM from it, and posts small JSON commands back. selectedPb is the only
// client-side state — everything else is a mirror of the server's truth,
// re-rendered whole on every poll (simple beats clever for a governor UI
// a human looks at every few seconds, not a high-frequency feed).

let selectedPb = null;
let lastState = null;
let currentIdentity = null;
let currentTab = 'resource';
let lastResourceCheck = null; // { checked, green, ... } from /api/resource_check
let lastDpState = null;

function esc(s) {
  return String(s).replace(/[&<>"']/g, c => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
  }[c]));
}

// Every request carries the session cookie automatically (same-origin,
// HttpOnly). A 401 here always means the session is gone (never issued,
// expired, or the server restarted and invalidated every session key) —
// there is nothing this page can do but send the Architect back to /login.
async function api(path, body) {
  const opts = body === undefined
    ? { method: 'GET' }
    : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) };
  const res = await fetch(path, opts);
  if (res.status === 401) {
    window.location.href = '/login';
    return null;
  }
  const state = await res.json();
  apply(state);
  return state;
}

function apply(state) {
  lastState = state;
  render(state);
}

function statusClass(st) {
  return { ok: 'st-ok', warned: 'st-warned', failed: 'st-failed', running: 'st-running', skipped: 'st-skipped', pending: 'st-pending' }[st] || '';
}
function statusIcon(st) {
  return { ok: '✓', warned: '⚠', failed: '✖', running: '▶', skipped: '»', pending: '·' }[st] || '·';
}

function render(state) {
  // title bar
  document.getElementById('status-line').textContent = state.status_line;
  const badge = document.getElementById('phase-badge');
  const phaseText = {
    idle: 'IDLE',
    running: 'RUNNING',
    halted: `⏸ HALTED — MAJOR at PB ${state.halted_pb + 1} · awaiting Architect (CSR-08)`,
    finished: 'FINISHED',
    aborted: 'ABORTED',
  }[state.phase] || state.phase.toUpperCase();
  badge.textContent = phaseText;
  badge.className = 'phase-badge ' + state.phase;

  // toolbar -- Run/Abort (and every other governing control below) stay
  // gated by the real server-side privilege check regardless of what this
  // client-side disabling does; this is UX, not the enforcement boundary.
  const isArchitect = !!(currentIdentity && currentIdentity.is_architect);
  document.getElementById('btn-run').disabled = !state.can_run || !isArchitect;
  document.getElementById('btn-abort').disabled = !state.can_abort || !isArchitect;
  document.getElementById('btn-load').disabled = !isArchitect;
  document.getElementById('btn-add-param').disabled = !isArchitect;
  if (document.activeElement !== document.getElementById('cfg-path')) {
    document.getElementById('cfg-path').value = state.cfg_path;
  }

  // params
  const chips = document.getElementById('params-chips');
  chips.innerHTML = state.params.map((p, i) =>
    `<span class="chip">${esc(p.key)} = ${esc(p.value)}${isArchitect ? `<button data-idx="${i}" class="chip-remove">×</button>` : ''}</span>`
  ).join('');
  chips.querySelectorAll('.chip-remove').forEach(btn => {
    btn.addEventListener('click', () => api('/api/params/remove', { index: Number(btn.dataset.idx) }));
  });

  // progress
  const c = state.counts;
  document.getElementById('pb-counter').textContent = `PB ${c.done}/${c.total}`;
  document.getElementById('counts').textContent = `ok ${c.ok} · warnings ${c.warnings} · errors ${c.errors}`;
  const pct = c.total > 0 ? (100 * c.done / c.total) : 0;
  document.getElementById('progress-fill').style.width = pct + '%';

  const haltBanner = document.getElementById('halt-banner');
  if (state.phase === 'halted') {
    haltBanner.classList.remove('hidden');
    document.getElementById('halt-text').textContent =
      `MAJOR · PB ${state.halted_pb + 1} — execution halted by law. Warnings proceed; majors await the Architect.`;
    const haltEv = state.events.find(e => e.pb_index === state.halted_pb);
    document.getElementById('btn-retry').disabled = !(haltEv && haltEv.remedy) || !isArchitect;
    document.getElementById('btn-skip').disabled = !isArchitect;
  } else {
    haltBanner.classList.add('hidden');
  }

  // queue
  const queue = document.getElementById('queue-list');
  queue.innerHTML = state.playbooks.map((pb, i) => {
    const st = state.statuses[i] || 'pending';
    return `<div class="queue-row ${statusClass(st)} ${selectedPb === i ? 'selected' : ''}" data-pb="${i}">${statusIcon(st)} ${esc(pb)}</div>`;
  }).join('');
  queue.querySelectorAll('.queue-row').forEach(row => {
    row.addEventListener('click', () => { selectedPb = Number(row.dataset.pb); render(lastState); });
  });

  // events grid
  document.getElementById('grid-title').textContent = `Errors & warnings — ${state.events.length} events`;
  const body = document.getElementById('events-body');
  body.innerHTML = state.events.map(ev => `
    <tr class="${selectedPb === ev.pb_index ? 'selected' : ''}" data-pb="${ev.pb_index}">
      <td>${ev.pb_index + 1}</td>
      <td>${esc(ev.task)}</td>
      <td class="${ev.severity === 'MAJOR' ? 'sev-major' : 'sev-warning'}">${esc(ev.severity)}</td>
      <td>${esc(ev.error_type)}</td>
    </tr>`).join('');
  body.querySelectorAll('tr').forEach(row => {
    row.addEventListener('click', () => { selectedPb = Number(row.dataset.pb); render(lastState); });
  });

  renderRemedy(state);

  // footer
  if (state.report_path) {
    document.getElementById('report-line').textContent = `sealed → ${state.report_path}`;
  }
  document.getElementById('btn-report').disabled = !(state.phase === 'finished' || state.phase === 'aborted') || !isArchitect;
  document.getElementById('save-as').classList.toggle('hidden', !state.report_path);
  const lastLog = state.log_tail.length ? state.log_tail[state.log_tail.length - 1] : '';
  document.getElementById('log-line').textContent = lastLog;

  renderIdentity();
}

function renderIdentity() {
  const badge = document.getElementById('identity-badge');
  const logoutBtn = document.getElementById('btn-logout');
  if (!currentIdentity) {
    badge.textContent = '(not signed in — Box Resources is free to view; AnuGovernor & Hala need a passport)';
    badge.className = 'identity-badge';
    logoutBtn.classList.add('hidden');
    return;
  }
  const role = currentIdentity.is_architect ? 'Architect' : 'gardener';
  badge.textContent = `${role} · ${currentIdentity.realm} · level ${currentIdentity.privilege_level}`;
  badge.className = 'identity-badge ' + (currentIdentity.is_architect ? 'architect' : 'gardener');
  logoutBtn.classList.remove('hidden');
}

// ── Save the sealed report in a chosen format, at a chosen location ──
//
// The canonical artifact is always the sealed .md file generate_report()
// wrote server-side (AkkadianSeal, docs/SHEDU/anu_governor_reports/). This
// is purely an export/view of that same content into a location and
// format the Architect picks in the browser.
const SAVE_MIME = {
  md: 'text/markdown',
  html: 'text/html',
  docx: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
};

async function fetchExport(format) {
  const res = await fetch(`/api/report/export?format=${format}`);
  if (!res.ok) {
    throw new Error(await res.text());
  }
  const filename = res.headers.get('X-Suggested-Filename') || `anu_governor_report.${format}`;
  const blob = await res.blob();
  return { blob, filename };
}

async function writeBlobWithPicker(blob, filename, mime) {
  // File System Access API: a real native "Save As" dialog with folder
  // navigation, when the browser supports it (Chromium-based browsers,
  // secure contexts). AbortError means the Architect cancelled the
  // picker -- not a failure, just do nothing.
  if (window.showSaveFilePicker) {
    try {
      const ext = filename.slice(filename.lastIndexOf('.'));
      const handle = await window.showSaveFilePicker({
        suggestedName: filename,
        types: [{ description: ext.slice(1).toUpperCase(), accept: { [mime]: [ext] } }],
      });
      const writable = await handle.createWritable();
      await writable.write(blob);
      await writable.close();
      return true;
    } catch (e) {
      if (e.name === 'AbortError') return true;
      throw e;
    }
  }
  // Fallback for browsers without the picker API (Firefox, WebKit/GNOME
  // Web, etc.): the browser's own download flow. No in-page folder
  // choice is possible here -- that's a real browser sandbox limit, not
  // something a website can work around -- but it still saves the file
  // (to wherever the browser's download setting points, or its own
  // "ask where to save" prompt if that's enabled).
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
  return true;
}

async function saveReport(format) {
  const status = document.getElementById('save-status');
  status.textContent = 'working…';
  status.classList.remove('dim');
  try {
    if (format === 'pdf') {
      // No server-side PDF: open the same HTML export in a new tab and
      // hand off to the browser's own print dialog, whose "Save as PDF"
      // destination gives the same native save-location picker.
      const { blob } = await fetchExport('html');
      const url = URL.createObjectURL(blob);
      const win = window.open(url, '_blank');
      if (win) {
        win.addEventListener('load', () => { win.print(); }, { once: true });
      }
      status.textContent = 'opened print view — choose "Save as PDF" in the print dialog';
    } else {
      const { blob, filename } = await fetchExport(format);
      await writeBlobWithPicker(blob, filename, SAVE_MIME[format] || 'application/octet-stream');
      status.textContent = `saved ${filename}`;
    }
  } catch (e) {
    status.textContent = String(e.message || e);
  }
  status.classList.add('dim');
}

function renderRemedy(state) {
  const el = document.getElementById('remedy-body');
  if (selectedPb === null) {
    el.innerHTML = `<p class="dim">select a playbook in the queue or an event in the grid</p>`;
    return;
  }
  const ev = state.events.find(e => e.pb_index === selectedPb);
  if (!ev) {
    const st = state.statuses[selectedPb] || 'pending';
    const name = state.playbooks[selectedPb] || '';
    const msg = {
      skipped: 'Skipped by the Architect at a MAJOR halt — no error/warning was recorded for this row itself; check the PB that actually halted execution for the remedy.',
      ok: 'Completed clean — no error or warning was recorded for this playbook.',
      pending: 'Not yet reached in the queue.',
      running: 'Currently running.',
    }[st] || 'No event on record for this playbook.';
    el.innerHTML = `<p class="${statusClass(st)}">${statusIcon(st)} ${esc(name)}</p><p class="dim">${esc(msg)}</p>`;
    return;
  }
  const sevClass = ev.severity === 'MAJOR' ? 'sev-major' : 'sev-warning';
  const isArchitect = !!(currentIdentity && currentIdentity.is_architect);
  let html = `<p class="${sevClass}">${esc(ev.severity)} — ${esc(ev.playbook)}</p>`;
  if (ev.remedy) {
    html += `
      <h3>Diagnosis</h3><p>${esc(ev.remedy.diagnosis)}</p>
      <h3>How to solve</h3><p>${esc(ev.remedy.solution)}</p>
      <button class="btn" id="btn-gen-fix" ${isArchitect ? '' : 'disabled'}>Generate fix playbook (${esc(ev.remedy.id)})</button>
      <p class="dim" style="margin-top:8px">Sealed on generation · running it = CSR-08 · chronicled</p>`;
  } else {
    html += `<p>No rule in the knowledge base matches this message.</p>
      <p class="dim">Solve it once — then add a [[remedy]] rule so AnuGovernor remembers forever.</p>`;
  }
  html += `<h3>Message</h3><p class="msg">${esc(ev.message)}</p>`;
  el.innerHTML = html;
  const genBtn = document.getElementById('btn-gen-fix');
  if (genBtn) {
    genBtn.addEventListener('click', () => api('/api/generate_fix', { pb_index: ev.pb_index }));
  }
}

// ── Tab switching ────────────────────────────────────────────────
// Tabs 2/3 are disabled (not just hidden) until the resource check is
// green AND recent -- this mirrors the server-side gate in
// resource_check_permits_start(), which is the real enforcement; this
// is only the UX that keeps someone from clicking into a tab whose
// start button won't work anyway.
function gateOpen() {
  return !!(lastResourceCheck && lastResourceCheck.checked && lastResourceCheck.green);
}

function switchTab(name) {
  // FIXED 2026-08-02 (Architect's own instruction): Tab 1 (Box Resources)
  // is free for every stakeholder, signed in or not -- start() no longer
  // hard-redirects an anonymous visitor away before they ever see it.
  // Tab 4 (Gate Orbits) joined it 2026-08-02: browsing the playbook
  // catalog by gate/domain isn't sensitive infrastructure state, only
  // suggesting/approving a classification is (Architect-only, enforced
  // server-side regardless of what this client-side check allows).
  // Tabs 2/3 (AnuGovernor, Hala) still need a real passport --
  // this is the moment that passport requirement actually surfaces, as
  // the login screen, rather than at page load for the whole app.
  if (name !== 'resource' && name !== 'gates' && !currentIdentity) { window.location.href = '/login'; return; }
  // Resource-check freshness no longer blocks NAVIGATING to Tabs 2/3 (that
  // used to disable the tab buttons outright, which also silently ate the
  // login-redirect above -- an anonymous visitor could never even reach
  // the login prompt while resources were red). It still blocks the
  // actual Run/Start buttons inside those tabs, unchanged -- see
  // renderDocpulse()'s btn-dp-start and render()'s btn-run.
  currentTab = name;
  ['resource', 'anu', 'hala', 'gates'].forEach(t => {
    document.getElementById('tab-' + t).classList.toggle('hidden', t !== name);
    document.getElementById('tab-btn-' + t).classList.toggle('active', t === name);
  });
  // gate_orbits.js is its own ES module (needs Three.js's import map) --
  // loosely coupled to app.js via this one global hook rather than a
  // shared render/state pipeline, since its response shapes are nothing
  // like the dashboard-state shape apply()/render() assume everywhere
  // else in this file.
  if (name === 'gates' && window.gateOrbits) {
    window.gateOrbits.onShown(!!(currentIdentity && currentIdentity.is_architect));
  }
}

function renderGate() {
  // FIXED 2026-08-02: this used to also disable the Tab 2/3 BUTTONS
  // themselves whenever resources were red/stale -- which silently ate
  // the login-redirect in switchTab() too (a disabled button never fires
  // its click handler), so an anonymous visitor with red resources could
  // never even reach the login prompt. Tab navigation (and the login
  // prompt it triggers) is always available now; this status line is
  // purely informational about what actually gates the Run/Start
  // buttons once you're on those tabs.
  const gateEl = document.getElementById('gate-status');
  if (!lastResourceCheck || !lastResourceCheck.checked) {
    gateEl.textContent = '⏸ run the resource check before starting a AnuGovernor run or Hala pulse';
    gateEl.className = 'mono gate-status gate-unknown';
  } else if (lastResourceCheck.green) {
    gateEl.textContent = '✓ green — AnuGovernor runs and Hala pulses are allowed';
    gateEl.className = 'mono gate-status gate-green';
  } else {
    gateEl.textContent = '✖ red — runs blocked: ' + lastResourceCheck.warnings.join('; ');
    gateEl.className = 'mono gate-status gate-red';
  }
}

// ── Tab 1: Resource Check ───────────────────────────────────────
async function checkResourceNow() {
  document.getElementById('resource-summary').textContent = 'checking…';
  const res = await fetch('/api/resource_check');
  if (res.status === 401) { window.location.href = '/login'; return; }
  lastResourceCheck = await res.json();
  renderResource();
  renderGate();
}

function renderResource() {
  const r = lastResourceCheck;
  if (!r || !r.checked) return;
  document.getElementById('resource-summary').textContent = r.green ? 'GREEN — no threshold crossed' : 'RED — see warnings below';
  document.getElementById('rt-ram').textContent = r.available_ram_gb.toFixed(1) + ' GiB';
  document.getElementById('rt-swap').textContent = r.swap_used_pct.toFixed(1) + '%';
  document.getElementById('rt-load').textContent = r.load_per_core.toFixed(2);
  document.getElementById('rt-time').textContent = new Date(r.checked_at).toLocaleTimeString();
  const grid = document.getElementById('resource-grid');
  grid.classList.toggle('resource-red', !r.green);
  grid.classList.toggle('resource-green', r.green);
  const warn = document.getElementById('resource-warnings');
  warn.innerHTML = r.warnings.length
    ? '<h3>Warnings</h3><ul>' + r.warnings.map(w => `<li class="sev-warning">⚠ ${esc(w)}</li>`).join('') + '</ul>'
    : '<p class="dim">No warnings.</p>';
}

// ── Tab 3: Hala (was Uruinimgina) ────────────────────────────────
async function dpApi(path, body) {
  const opts = body === undefined
    ? { method: 'GET' }
    : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) };
  const res = await fetch(path, opts);
  if (res.status === 401) { window.location.href = '/login'; return null; }
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    document.getElementById('dp-log-body').textContent = 'ERROR: ' + (err.error || res.statusText);
    return null;
  }
  const state = await res.json();
  lastDpState = state;
  renderDocpulse(state);
  return state;
}

function renderDocpulse(state) {
  if (!state) return;
  const isArchitect = !!(currentIdentity && currentIdentity.is_architect);
  document.getElementById('btn-dp-start').disabled = !state.can_start || !isArchitect || !gateOpen();
  document.getElementById('btn-dp-abort').disabled = !state.can_abort || !isArchitect;

  const queue = document.getElementById('dp-queue-list');
  queue.innerHTML = state.stages.map((name, i) => {
    const st = state.statuses[i] || 'pending';
    return `<div class="queue-row ${statusClass(st)}">${statusIcon(st)} ${esc(name)}</div>`;
  }).join('');

  const haltBanner = document.getElementById('dp-halt-banner');
  if (state.phase === 'halted') {
    haltBanner.classList.remove('hidden');
    const ev = state.events.find(e => e.pb_index === state.halted_stage);
    document.getElementById('dp-halt-text').textContent =
      ev ? `${ev.severity} · ${ev.task} — ${ev.message}` : 'halted — awaiting the Architect (CSR-08)';
    document.getElementById('btn-dp-retry').disabled = !isArchitect;
    document.getElementById('btn-dp-skip').disabled = !isArchitect;
  } else {
    haltBanner.classList.add('hidden');
  }

  document.getElementById('dp-log-body').innerHTML = state.log_tail.map(l => esc(l)).join('<br>');
}

function collectDocpulseRequest() {
  return {
    repo: document.getElementById('dp-repo').value,
    message: document.getElementById('dp-message').value,
    docs_tribe: document.getElementById('dp-tribe').value,
    archive: document.getElementById('dp-archive').value,
    ingest_manifest_dir: document.getElementById('dp-manifest').value,
    enkiddb_root: document.getElementById('dp-enkiddb').value,
  };
}

function wireDocpulseControls() {
  document.getElementById('btn-dp-start').addEventListener('click', () => dpApi('/api/docpulse/start', collectDocpulseRequest()));
  document.getElementById('btn-dp-abort').addEventListener('click', () => dpApi('/api/docpulse/abort', {}));
  document.getElementById('btn-dp-retry').addEventListener('click', () => dpApi('/api/docpulse/retry', {}));
  document.getElementById('btn-dp-skip').addEventListener('click', () => dpApi('/api/docpulse/skip', {}));
}

function wireStaticControls() {
  document.getElementById('btn-load').addEventListener('click', () =>
    api('/api/load', { cfg_path: document.getElementById('cfg-path').value }));
  document.getElementById('btn-run').addEventListener('click', () => api('/api/run', {}));
  document.getElementById('btn-abort').addEventListener('click', () => api('/api/abort', {}));
  document.getElementById('btn-skip').addEventListener('click', () => api('/api/skip', {}));
  document.getElementById('btn-retry').addEventListener('click', () => api('/api/retry', {}));
  document.getElementById('btn-report').addEventListener('click', () => api('/api/report', {}));
  document.getElementById('btn-save').addEventListener('click', () =>
    saveReport(document.getElementById('save-format').value));
  document.getElementById('btn-add-param').addEventListener('click', () => {
    const key = document.getElementById('param-key');
    const value = document.getElementById('param-value');
    if (!key.value) return;
    api('/api/params/add', { key: key.value, value: value.value });
    key.value = '';
    value.value = '';
  });
  document.getElementById('btn-logout').addEventListener('click', async () => {
    await fetch('/api/logout', { method: 'POST' });
    window.location.href = '/login';
  });

  document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => switchTab(btn.dataset.tab));
  });
  document.getElementById('btn-check-now').addEventListener('click', checkResourceNow);
  wireDocpulseControls();
}

// FIXED 2026-08-02 (Architect's own instruction): an anonymous visitor is
// no longer redirected away before seeing anything -- Tab 1 (Box
// Resources) is free for every stakeholder. Only currentIdentity gates
// Tabs 2/3 (see switchTab()); this function just establishes whether a
// session exists, without treating "none" as an error.
async function start() {
  const res = await fetch('/api/session');
  currentIdentity = (res.status === 401) ? null : await res.json();
  renderIdentity();
  wireStaticControls();
  renderGate();

  if (currentIdentity) {
    // Signed in: full experience -- Tab 2/3 state polling included.
    await api('/api/state');
    await dpApi('/api/docpulse/state');
    setInterval(() => api('/api/state'), 700);
    // Hala's own halts are attended-but-infrequent (a pulse runs for
    // minutes, not seconds), so this polls less aggressively than Tab 2's
    // corpus state -- still fast enough to notice a halt promptly.
    setInterval(() => dpApi('/api/docpulse/state'), 1500);
  } else {
    // Anonymous: Tab 1 only. /api/state and /api/docpulse/state both still
    // require a passport server-side (never relaxed) -- don't even poll
    // them, since every response would just be a 401 no one asked for.
    switchTab('resource');
  }
}

start();
