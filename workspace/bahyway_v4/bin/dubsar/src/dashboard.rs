//! ShoWay HTML Dashboard generator for BahyWay Sovereign Ecosystem v4.0.
//!
//! Generates a fully self-contained `showway.html` file with:
//!   - Animated starfield (CSS keyframes)
//!   - Three-panel layout: akk Registry | KAKI Inspector | Sovereign Metrics
//!   - EAV Data Grid tab
//!   - DAMA knowledge-area tree tab
//!   - Live data embedded as a JSON const block

/// A single particle row to embed in the dashboard.
pub struct ParticleRow {
    pub uuid_hash: u32,
    pub state: &'static str, // "GOLDEN" / "FUZZY" / "DEAD"
    pub epoch: u32,
    pub events: usize,
}

/// Generate a fully self-contained showway.html as a String.
///
/// Embeds all metrics and particle data as inline JSON — no external deps.
pub fn generate_dashboard(
    tribe_id: u16,
    golden: usize,
    fuzzy: usize,
    dead: usize,
    epoch: u32,
    particles: &[ParticleRow],
) -> String {
    let total = golden + fuzzy + dead;
    let entropy = ((total - golden) * 100).checked_div(total).unwrap_or(0);

    // Build JSON particle array
    let mut particle_json = String::from("[\n");
    for (i, p) in particles.iter().enumerate() {
        let comma = if i + 1 < particles.len() { "," } else { "" };
        particle_json.push_str(&format!(
            "        {{\"uuid_hash\":{},\"state\":\"{}\",\"epoch\":{},\"events\":{}}}{}\n",
            p.uuid_hash, p.state, p.epoch, p.events, comma
        ));
    }
    particle_json.push_str("      ]");

    // Build DAMA knowledge area data
    let dama_areas = build_dama_areas_json();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>BahyWay ShoWay Dashboard v4.0</title>
<style>
/* ── Reset & Base ─────────────────────────────────────────────────────── */
*,*::before,*::after{{box-sizing:border-box;margin:0;padding:0}}
html,body{{height:100%;overflow:hidden;font-family:'Courier New',Courier,monospace;background:#050a12;color:#c0d8f0}}
/* ── Starfield canvas ─────────────────────────────────────────────────── */
#starfield{{position:fixed;top:0;left:0;width:100%;height:100%;z-index:0;pointer-events:none}}
.star{{position:absolute;border-radius:50%;animation:twinkle var(--dur,3s) ease-in-out infinite alternate}}
@keyframes twinkle{{0%{{opacity:0.1;transform:scale(0.8)}}100%{{opacity:1;transform:scale(1.2)}}}}
.gstar{{position:absolute;border-radius:50%;background:radial-gradient(circle,#ffd700 0%,#ff8800 60%,transparent 100%);animation:orbit var(--dur,8s) ease-in-out infinite alternate,twinkle calc(var(--dur,8s)/2) ease-in-out infinite alternate}}
@keyframes orbit{{0%{{transform:translateY(0) translateX(0)}}50%{{transform:translateY(-12px) translateX(8px)}}100%{{transform:translateY(4px) translateX(-6px)}}}}
/* ── Layout ───────────────────────────────────────────────────────────── */
#app{{position:relative;z-index:1;display:flex;flex-direction:column;height:100vh}}
/* ── Top Navigation Bar ───────────────────────────────────────────────── */
#navbar{{background:rgba(0,20,40,0.95);border-bottom:1px solid #00d4ff;padding:6px 16px;display:flex;align-items:center;gap:20px;flex-shrink:0}}
#navbar .brand{{color:#ffd700;font-weight:bold;font-size:14px;letter-spacing:1px;text-shadow:0 0 10px #ffd70088}}
.tab{{padding:4px 12px;cursor:pointer;color:#4090b0;font-size:12px;border-radius:3px;border:1px solid transparent;transition:all .2s;user-select:none}}
.tab:hover{{color:#00d4ff;border-color:#00d4ff44}}
.tab.active{{color:#00d4ff;border-color:#00d4ff;background:rgba(0,212,255,0.1);text-shadow:0 0 6px #00d4ff66}}
/* ── Main content area ────────────────────────────────────────────────── */
#main{{flex:1;overflow:hidden;display:flex;flex-direction:column}}
.tab-content{{display:none;flex:1;overflow:hidden}}
.tab-content.active{{display:flex}}
/* ── Three-panel layout ───────────────────────────────────────────────── */
#panel-home{{display:flex;gap:8px;padding:8px;height:100%}}
.panel{{background:rgba(0,15,30,0.85);border:1px solid #00d4ff44;border-radius:6px;padding:12px;display:flex;flex-direction:column;overflow:hidden}}
.panel-title{{color:#00d4ff;font-size:11px;font-weight:bold;letter-spacing:2px;text-transform:uppercase;border-bottom:1px solid #00d4ff33;padding-bottom:6px;margin-bottom:8px;flex-shrink:0}}
#panel-left{{width:260px;flex-shrink:0}}
#panel-center{{flex:1}}
#panel-right{{width:220px;flex-shrink:0}}
/* ── Particle list ────────────────────────────────────────────────────── */
.particle-list{{overflow-y:auto;flex:1}}
.particle-item{{display:flex;align-items:center;gap:8px;padding:5px 4px;cursor:pointer;border-radius:3px;font-size:11px;border-bottom:1px solid rgba(0,212,255,0.08)}}
.particle-item:hover{{background:rgba(0,212,255,0.08)}}
.particle-item.selected{{background:rgba(0,212,255,0.15);border:1px solid #00d4ff44}}
.state-dot{{width:10px;height:10px;border-radius:50%;flex-shrink:0}}
.dot-golden{{background:#ffd700;box-shadow:0 0 6px #ffd70088}}
.dot-fuzzy{{background:#ffa500;box-shadow:0 0 4px #ffa50066}}
.dot-dead{{background:#606060}}
.p-hash{{color:#4090b0;font-size:10px}}
.p-epoch{{color:#306080;font-size:10px;margin-left:auto}}
/* ── KAKI Inspector ───────────────────────────────────────────────────── */
#inspector{{flex:1;overflow-y:auto}}
.insp-row{{display:flex;margin-bottom:6px;font-size:11px}}
.insp-label{{color:#306080;width:90px;flex-shrink:0}}
.insp-value{{color:#c0d8f0}}
.insp-golden{{color:#ffd700;text-shadow:0 0 8px #ffd70066}}
.insp-fuzzy{{color:#ffa500}}
.insp-dead{{color:#606060}}
.empty-inspector{{color:#204060;font-size:12px;padding:20px;text-align:center}}
/* ── Sovereign Metrics panel ──────────────────────────────────────────── */
.metric-row{{margin-bottom:10px}}
.metric-label{{font-size:10px;color:#4090b0;margin-bottom:3px}}
.metric-bar-bg{{background:#0a1a2a;border-radius:2px;height:8px;overflow:hidden}}
.metric-bar{{height:8px;border-radius:2px;transition:width .5s ease}}
.bar-golden{{background:linear-gradient(90deg,#ffd700,#ff8800)}}
.bar-fuzzy{{background:linear-gradient(90deg,#ffa500,#cc6600)}}
.bar-dead{{background:linear-gradient(90deg,#606060,#303030)}}
.metric-val{{font-size:10px;color:#a0b8d0;margin-top:2px}}
.naba-status{{margin-top:12px;padding:8px;border:1px solid #00d4ff33;border-radius:4px;background:rgba(0,30,60,0.5)}}
.naba-label{{font-size:9px;color:#4090b0;letter-spacing:2px}}
.naba-val{{font-size:14px;font-weight:bold;margin-top:4px}}
.status-absolute{{color:#00ff90;text-shadow:0 0 10px #00ff9066}}
.status-active{{color:#ffd700;text-shadow:0 0 8px #ffd70066}}
.status-partial{{color:#ffa500}}
.status-risk{{color:#ff4040;text-shadow:0 0 8px #ff404066}}
/* ── EAV Grid tab ─────────────────────────────────────────────────────── */
#tab-eav{{padding:8px;flex-direction:column}}
.grid-wrap{{flex:1;overflow:auto}}
table{{width:100%;border-collapse:collapse;font-size:11px}}
thead{{position:sticky;top:0;background:#020810}}
th{{color:#00d4ff;padding:6px 10px;text-align:left;border-bottom:1px solid #00d4ff44;font-size:10px;letter-spacing:1px;text-transform:uppercase}}
td{{padding:5px 10px;border-bottom:1px solid #0a1a2a;color:#a0b8d0}}
tr:hover td{{background:rgba(0,212,255,0.04)}}
.td-state{{font-weight:bold}}
.td-golden{{color:#ffd700}}
.td-fuzzy{{color:#ffa500}}
.td-dead{{color:#606060}}
/* ── DAMA tab ─────────────────────────────────────────────────────────── */
#tab-dama{{padding:8px;flex-direction:column}}
.dama-scroll{{flex:1;overflow-y:auto}}
.ka-block{{margin-bottom:12px;border:1px solid #00d4ff22;border-radius:4px;overflow:hidden}}
.ka-header{{background:rgba(0,212,255,0.08);padding:6px 10px;cursor:pointer;display:flex;align-items:center;gap:8px;user-select:none}}
.ka-header:hover{{background:rgba(0,212,255,0.12)}}
.ka-code{{color:#00d4ff;font-size:10px;font-weight:bold;width:40px;flex-shrink:0}}
.ka-name{{color:#c0d8f0;font-size:11px}}
.ka-count{{margin-left:auto;color:#4090b0;font-size:10px}}
.ka-chevron{{color:#306080;font-size:10px;transition:transform .2s}}
.ka-body{{display:none;padding:4px 0}}
.ka-body.open{{display:block}}
.term-row{{display:flex;gap:8px;padding:4px 14px;font-size:10px;border-bottom:1px solid #050a12}}
.term-row:hover{{background:rgba(0,212,255,0.04)}}
.term-code{{color:#00a0c8;width:160px;flex-shrink:0}}
.term-label{{color:#a0b8d0;flex:1}}
/* ── Status bar ───────────────────────────────────────────────────────── */
#statusbar{{background:rgba(0,10,20,0.97);border-top:1px solid #00d4ff33;padding:4px 16px;display:flex;align-items:center;gap:16px;font-size:10px;flex-shrink:0;color:#4090b0}}
.sb-live{{color:#00ff90;font-weight:bold;animation:blink 2s ease-in-out infinite}}
@keyframes blink{{0%,100%{{opacity:1}}50%{{opacity:0.4}}}}
.sb-sep{{color:#1a3050}}
.sb-ok{{color:#00ff90}}
.sb-warn{{color:#ffa500}}
</style>
</head>
<body>
<div id="starfield"></div>
<div id="app">
  <!-- Navigation Bar -->
  <div id="navbar">
    <div class="brand">&#x1F702; BahyWay Ecosystem v4.0</div>
    <div class="tab active" onclick="showTab('home')">DubSar IDE</div>
    <div class="tab" onclick="showTab('kaki')">KAKI Explorer</div>
    <div class="tab" onclick="showTab('home')">akk Registry</div>
    <div class="tab" onclick="showTab('eav')">EAV Grid</div>
    <div class="tab" onclick="showTab('home')">NABA</div>
    <div class="tab" onclick="showTab('dama')">DAMA</div>
  </div>

  <!-- Main Content -->
  <div id="main">
    <!-- Home / Three-Panel View -->
    <div id="tab-home" class="tab-content active">
      <div id="panel-home">
        <!-- LEFT: akk Registry -->
        <div class="panel" id="panel-left">
          <div class="panel-title">akk Registry</div>
          <div class="particle-list" id="particle-list"></div>
        </div>
        <!-- CENTER: KAKI Inspector -->
        <div class="panel" id="panel-center">
          <div class="panel-title">KAKI Inspector</div>
          <div id="inspector">
            <div class="empty-inspector">&#x2219; Select a particle to inspect</div>
          </div>
        </div>
        <!-- RIGHT: Sovereign Metrics -->
        <div class="panel" id="panel-right">
          <div class="panel-title">Sovereign Metrics</div>
          <div id="metrics-panel"></div>
        </div>
      </div>
    </div>

    <!-- EAV Grid tab -->
    <div id="tab-eav" class="tab-content">
      <div class="grid-wrap">
        <table id="eav-table">
          <thead>
            <tr>
              <th>KAKI (uuid_hash)</th>
              <th>State</th>
              <th>Epoch</th>
              <th>Events</th>
              <th>Quality</th>
              <th>Timestamp</th>
            </tr>
          </thead>
          <tbody id="eav-tbody"></tbody>
        </table>
      </div>
    </div>

    <!-- DAMA tab -->
    <div id="tab-dama" class="tab-content">
      <div class="dama-scroll" id="dama-tree"></div>
    </div>
  </div>

  <!-- Status Bar -->
  <div id="statusbar">
    <span class="sb-live">&#x25cf; LIVE</span>
    <span class="sb-sep">|</span>
    <span>EnkiDB: <span class="sb-ok">OK</span></span>
    <span class="sb-sep">|</span>
    <span>Tribe: <span id="sb-tribe">--</span></span>
    <span class="sb-sep">|</span>
    <span>KAKI Count: <span id="sb-count">--</span></span>
    <span class="sb-sep">|</span>
    <span>Golden: <span id="sb-golden">--</span></span>
    <span class="sb-sep">|</span>
    <span>Entropy: <span id="sb-entropy">--%</span></span>
    <span class="sb-sep">|</span>
    <span style="color:#306080">v4.0.0</span>
  </div>
</div>

<script>
// ── Embedded Data ─────────────────────────────────────────────────────────────
const DATA = {{
  tribe_id: {tribe_id},
  golden:   {golden},
  fuzzy:    {fuzzy},
  dead:     {dead},
  total:    {total},
  epoch:    {epoch},
  entropy:  {entropy},
  particles: {particle_json}
}};

// ── DAMA Knowledge Areas ──────────────────────────────────────────────────────
const DAMA = {dama_areas};

// ── Tab switching ─────────────────────────────────────────────────────────────
function showTab(name) {{
  document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
  document.querySelectorAll('.tab').forEach(el => el.classList.remove('active'));
  const tc = document.getElementById('tab-' + name);
  if (tc) tc.classList.add('active');
  const tabs = document.querySelectorAll('.tab');
  const map = {{home:0, kaki:1, eav:3, dama:5}};
  if (map[name] !== undefined && tabs[map[name]]) tabs[map[name]].classList.add('active');
}}

// ── Starfield ─────────────────────────────────────────────────────────────────
function buildStarfield() {{
  const sf = document.getElementById('starfield');
  // 200 background stars
  for (let i = 0; i < 200; i++) {{
    const s = document.createElement('div');
    s.className = 'star';
    const x = Math.random() * 100, y = Math.random() * 100;
    const sz = Math.random() * 2 + 0.5;
    const dur = (Math.random() * 4 + 2).toFixed(1);
    const gold = Math.random() < 0.15;
    s.style.cssText = `left:${{x}}%;top:${{y}}%;width:${{sz}}px;height:${{sz}}px;` +
      `background:${{gold ? '#ffd700' : '#c0d8f0'}};--dur:${{dur}}s;` +
      `animation-delay:${{(Math.random()*4).toFixed(1)}}s`;
    sf.appendChild(s);
  }}
  // 20 glowing golden orbit dots
  for (let i = 0; i < 20; i++) {{
    const g = document.createElement('div');
    g.className = 'gstar';
    const x = Math.random() * 100, y = Math.random() * 100;
    const sz = Math.random() * 6 + 4;
    const dur = (Math.random() * 8 + 6).toFixed(1);
    g.style.cssText = `left:${{x}}%;top:${{y}}%;width:${{sz}}px;height:${{sz}}px;--dur:${{dur}}s;` +
      `animation-delay:${{(Math.random()*6).toFixed(1)}}s;opacity:0.4`;
    sf.appendChild(g);
  }}
}}

// ── Particle list ─────────────────────────────────────────────────────────────
let selectedIdx = null;

function stateClass(s) {{
  return s === 'GOLDEN' ? 'dot-golden' : s === 'FUZZY' ? 'dot-fuzzy' : 'dot-dead';
}}

function stateStyle(s) {{
  return s === 'GOLDEN' ? 'insp-golden' : s === 'FUZZY' ? 'insp-fuzzy' : 'insp-dead';
}}

function buildParticleList() {{
  const el = document.getElementById('particle-list');
  if (!DATA.particles.length) {{
    el.innerHTML = '<div style="color:#204060;padding:12px;font-size:11px">No particles — run .demo in DubSar</div>';
    return;
  }}
  DATA.particles.forEach((p, i) => {{
    const item = document.createElement('div');
    item.className = 'particle-item';
    item.dataset.idx = i;
    item.innerHTML = `
      <div class="state-dot ${{stateClass(p.state)}}"></div>
      <span class="p-hash">0x${{p.uuid_hash.toString(16).padStart(8,'0')}}</span>
      <span class="p-epoch">ep ${{p.epoch}}</span>`;
    item.addEventListener('click', () => selectParticle(i));
    el.appendChild(item);
  }});
}}

function selectParticle(idx) {{
  // Deselect previous
  document.querySelectorAll('.particle-item').forEach(el => el.classList.remove('selected'));
  const item = document.querySelector(`[data-idx="${{idx}}"]`);
  if (item) item.classList.add('selected');
  selectedIdx = idx;
  buildInspector(idx);
}}

function buildInspector(idx) {{
  const p = DATA.particles[idx];
  const el = document.getElementById('inspector');
  const sc = stateStyle(p.state);
  const qualPct = p.state === 'GOLDEN' ? 95 : p.state === 'FUZZY' ? 55 : 15;
  el.innerHTML = `
    <div class="insp-row"><span class="insp-label">State</span><span class="insp-value ${{sc}}">${{p.state}}</span></div>
    <div class="insp-row"><span class="insp-label">uuid_hash</span><span class="insp-value">0x${{p.uuid_hash.toString(16).padStart(8,'0')}}</span></div>
    <div class="insp-row"><span class="insp-label">Epoch</span><span class="insp-value">${{p.epoch}}</span></div>
    <div class="insp-row"><span class="insp-label">Events</span><span class="insp-value">${{p.events}}</span></div>
    <div class="insp-row"><span class="insp-label">Tribe</span><span class="insp-value">0x${{DATA.tribe_id.toString(16).padStart(4,'0')}}</span></div>
    <div class="insp-row"><span class="insp-label">Quality ~</span><span class="insp-value">${{qualPct}}%</span></div>
    <div style="margin-top:12px;font-size:10px;color:#4090b0;letter-spacing:1px">EAV ATTRIBUTES</div>
    <div style="margin-top:6px;font-size:10px;color:#306080;padding:6px;background:rgba(0,20,40,0.5);border-radius:3px">
      <div>attr:state → ${{p.state}}</div>
      <div>attr:epoch → ${{p.epoch}}</div>
      <div>attr:tribe → 0x${{DATA.tribe_id.toString(16).padStart(4,'0')}}</div>
    </div>`;
}}

// ── Metrics panel ─────────────────────────────────────────────────────────────
function buildMetrics() {{
  const el = document.getElementById('metrics-panel');
  const total = DATA.total || 1;
  const gp = (DATA.golden / total * 100).toFixed(0);
  const fp = (DATA.fuzzy  / total * 100).toFixed(0);
  const dp = (DATA.dead   / total * 100).toFixed(0);
  const status = DATA.golden / total >= 0.95 ? 'ABSOLUTE'
               : DATA.golden / total >= 0.75 ? 'ACTIVE'
               : DATA.golden / total >= 0.50 ? 'PARTIAL'
               : 'AT RISK';
  const statusClass = 'status-' + status.toLowerCase().replace(' ','-');

  el.innerHTML = `
    <div class="metric-row">
      <div class="metric-label">&#x25cf; GOLDEN (${{DATA.golden}})</div>
      <div class="metric-bar-bg"><div class="metric-bar bar-golden" style="width:${{gp}}%"></div></div>
      <div class="metric-val">${{gp}}%</div>
    </div>
    <div class="metric-row">
      <div class="metric-label">&#x25d0; FUZZY (${{DATA.fuzzy}})</div>
      <div class="metric-bar-bg"><div class="metric-bar bar-fuzzy" style="width:${{fp}}%"></div></div>
      <div class="metric-val">${{fp}}%</div>
    </div>
    <div class="metric-row">
      <div class="metric-label">&#x25cb; DEAD (${{DATA.dead}})</div>
      <div class="metric-bar-bg"><div class="metric-bar bar-dead" style="width:${{dp}}%"></div></div>
      <div class="metric-val">${{dp}}%</div>
    </div>
    <div class="naba-status">
      <div class="naba-label">NABA SOVEREIGNTY</div>
      <div class="naba-val ${{statusClass}}">${{status}}</div>
    </div>
    <div style="margin-top:10px;font-size:10px;color:#306080">
      <div>Epoch: <span style="color:#a0b8d0">${{DATA.epoch}}</span></div>
      <div>Total particles: <span style="color:#a0b8d0">${{DATA.total}}</span></div>
      <div>Entropy: <span style="color:${{DATA.entropy < 5 ? '#00ff90' : '#ffa500'}}">${{DATA.entropy}}%</span></div>
    </div>`;
}}

// ── Status bar ────────────────────────────────────────────────────────────────
function buildStatusBar() {{
  document.getElementById('sb-tribe').textContent   = '0x' + DATA.tribe_id.toString(16).padStart(4,'0');
  document.getElementById('sb-count').textContent   = DATA.total;
  document.getElementById('sb-golden').textContent  = DATA.golden;
  document.getElementById('sb-entropy').textContent = DATA.entropy + '%';
  const entropyEl = document.getElementById('sb-entropy');
  entropyEl.className = DATA.entropy < 20 ? 'sb-ok' : 'sb-warn';
}}

// ── EAV Grid ──────────────────────────────────────────────────────────────────
function buildEavGrid() {{
  const tbody = document.getElementById('eav-tbody');
  if (!DATA.particles.length) {{
    tbody.innerHTML = '<tr><td colspan="6" style="text-align:center;color:#204060;padding:20px">No data</td></tr>';
    return;
  }}
  const now = Date.now();
  DATA.particles.forEach(p => {{
    const tr = document.createElement('tr');
    const sc = p.state === 'GOLDEN' ? 'td-golden' : p.state === 'FUZZY' ? 'td-fuzzy' : 'td-dead';
    const qual = p.state === 'GOLDEN' ? '&#x2605;&#x2605;&#x2605; GEM' : p.state === 'FUZZY' ? '&#x2605;&#x2605; TRIBE' : '&#x2605; DEAD';
    tr.innerHTML = `
      <td>0x${{p.uuid_hash.toString(16).padStart(8,'0')}}</td>
      <td class="td-state ${{sc}}">${{p.state}}</td>
      <td>${{p.epoch}}</td>
      <td>${{p.events}}</td>
      <td>${{qual}}</td>
      <td style="color:#306080">ep+${{p.epoch}}</td>`;
    tbody.appendChild(tr);
  }});
}}

// ── DAMA Tree ─────────────────────────────────────────────────────────────────
function buildDamaTree() {{
  const el = document.getElementById('dama-tree');
  DAMA.forEach(ka => {{
    const block = document.createElement('div');
    block.className = 'ka-block';
    const header = document.createElement('div');
    header.className = 'ka-header';
    header.innerHTML = `
      <span class="ka-code">${{ka.code}}</span>
      <span class="ka-name">${{ka.label}}</span>
      <span class="ka-count">${{ka.terms.length}} terms</span>
      <span class="ka-chevron" id="chev-${{ka.code}}">&#9656;</span>`;
    const body = document.createElement('div');
    body.className = 'ka-body';
    body.id = 'body-' + ka.code;
    ka.terms.forEach(t => {{
      const row = document.createElement('div');
      row.className = 'term-row';
      row.innerHTML = `<span class="term-code">${{t.code}}</span><span class="term-label">${{t.label}}</span>`;
      body.appendChild(row);
    }});
    header.addEventListener('click', () => {{
      const isOpen = body.classList.toggle('open');
      document.getElementById('chev-' + ka.code).innerHTML = isOpen ? '&#9662;' : '&#9656;';
    }});
    block.appendChild(header);
    block.appendChild(body);
    el.appendChild(block);
  }});
}}

// ── Init ──────────────────────────────────────────────────────────────────────
window.addEventListener('DOMContentLoaded', () => {{
  buildStarfield();
  buildParticleList();
  buildMetrics();
  buildStatusBar();
  buildEavGrid();
  buildDamaTree();
  if (DATA.particles.length > 0) selectParticle(0);
}});
</script>
</body>
</html>
"#,
        tribe_id = tribe_id,
        golden = golden,
        fuzzy = fuzzy,
        dead = dead,
        total = total,
        epoch = epoch,
        entropy = entropy,
        particle_json = particle_json,
        dama_areas = dama_areas,
    )
}

/// Build the DAMA knowledge-area JSON for the dashboard.
fn build_dama_areas_json() -> String {
    use damadmbok_dictionary::{by_area, KnowledgeArea};

    let mut out = String::from("[\n");
    let areas = KnowledgeArea::all();
    for (i, ka) in areas.iter().enumerate() {
        let comma = if i + 1 < areas.len() { "," } else { "" };
        out.push_str(&format!(
            "  {{\"code\":\"{}\",\"label\":\"{}\",\"chapter\":{},\"terms\":[\n",
            ka.code(),
            escape_json(ka.label()),
            ka.chapter_number()
        ));
        let terms: Vec<_> = by_area(*ka).collect();
        for (j, t) in terms.iter().enumerate() {
            let tc = if j + 1 < terms.len() { "," } else { "" };
            out.push_str(&format!(
                "    {{\"code\":\"{}\",\"label\":\"{}\"}}{}\n",
                t.code,
                escape_json(t.label),
                tc
            ));
        }
        out.push_str(&format!("  ]}}{}\n", comma));
    }
    out.push(']');
    out
}

/// Escape a string for safe JSON embedding.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
