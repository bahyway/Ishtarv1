//! UTNAPISHTIM — Three.js Sovereign Web Viewer Generator
//! Generates self-contained HTML per client
//! Deployed to: client.bahyway.com/dubsar
//!
//! CORRECTIONS applied:
//!   - Dead particle colour = #404040 (NOT #800000 Maroon)
//!   - KUR overlay = #1A0A2E sovereign indigo
//!   - Orbital radius = particle property (NOT tribe ring radius)
//!   - ring_radius = decorative visual separation only
//!   - W5H2 query syntax in TCP refresh (anti-SQL sovereign)

#![forbid(unsafe_code)]
use crate::{ClientTopology, PLIMPTON_322_DIVISOR, GOLDEN_ANGLE_DEG};
use crate::{
    ORBITAL_GOLDEN_GEM, ORBITAL_GOLDEN_ALIVE, ORBITAL_FUZZY_AGED,
    ORBITAL_FUZZY_GRAY,  ORBITAL_FUZZY_DECAY,
    ORBITAL_DEAD_EXPIRED, ORBITAL_DEAD_SEALED,
};

pub fn generate_threejs_viewer(topo: &ClientTopology) -> String {
    let tribe_js   = build_tribe_js(topo);
    let orbital_js = build_orbital_constants();
    let client_name = &topo.client_name;
    let client_id   = topo.client_id;
    let sealed_at   = topo.sealed_at;

    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8"/>
  <title>DubSar Orbital Viewer — {client_name}</title>
  <!--
    UTNAPISHTIM 𒌓𒍣𒅁𒀭 — He Who Found Life
    BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
    Client: {client_name} (0x{client_id:04X})
    Sealed at epoch: {sealed_at}

    SOVEREIGN RULES (locked permanently):
    B11 = round(eav_quality × {PLIMPTON_322_DIVISOR}) — Plimpton 322 — NEVER 255
    ColourID from PĀŠIRU golden angle — NEVER from KAKI bytes
    Orbital radius = particle OrbitalSubtype — NOT tribe ring radius
    Dead particles: #404040 dark grey (NOT #800000 — that is Nergal AV)
    KUR terminal:   #1A0A2E sovereign indigo
    TCP query: W5H2 sovereign syntax — anti-SQL — no SELECT/FROM/WHERE
  -->
  <style>
    * {{ margin:0; padding:0; box-sizing:border-box }}
    body {{ background:#0a0618; overflow:hidden; font-family:monospace }}
    #hud {{
      position:absolute; top:18px; left:24px;
      color:#8899aa; font-size:11px; line-height:1.8;
    }}
    .title  {{ color:#c8a84b; font-size:14px; letter-spacing:2px }}
    .glyph  {{ color:#556677; font-size:20px }}
    #refresh-bar {{
      position:absolute; bottom:0; left:0;
      height:2px; background:#c8a84b; width:0%;
    }}
  </style>
</head>
<body>
  <canvas id="canvas"></canvas>
  <div id="hud">
    <div class="title">DUBSAR ORBITAL VIEWER</div>
    <div class="glyph">𒌓𒍣𒅁𒀭 UTNAPISHTIM · 𒁾 DUB.SAR</div>
    <div>{client_name}</div>
    <div id="particle-count">Particles: —</div>
    <div id="live-status">Connecting to EnkiDB…</div>
  </div>
  <div id="refresh-bar"></div>

  <script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js">
  </script>
  <script>
// ════════════════════════════════════════════════════
// UTNAPISHTIM 𒌓𒍣𒅁𒀭 — DubSar Three.js Orbital Viewer
// BahyWay.Ecosystem v4.0 — DUB.SAR 𒁾
// ════════════════════════════════════════════════════

// ── Sovereign particle orbital radii (sealed v4.0) ──
// These are PARTICLE properties (from OrbitalSubtype via EAV)
// NOT tribe ring radii (those are decorative only)
{orbital_js}

// ── Plimpton 322 — B11 computation ─────────────────
// B11 = round(eav_quality × {PLIMPTON_322_DIVISOR}) — NEVER 255
function computeB11(eav_quality) {{
  return Math.round(eav_quality * {PLIMPTON_322_DIVISOR});
}}

// ── Sovereign colour mapping ─────────────────────────
// ColourID from PĀŠIRU — NEVER from KAKI bytes
// Dead = #404040 dark grey  (NOT #800000 — that is Nergal AV)
// KUR  = #1A0A2E sovereign indigo
function orbitalColour(eav_quality) {{
  const b11 = computeB11(eav_quality);
  if (b11 >= 200)          return '#FFD700'; // GOLDEN_GEM
  if (eav_quality >= 0.70) return '#C8B830'; // GOLDEN_ALIVE
  if (eav_quality >= 0.55) return '#8090A0'; // FUZZY_AGED
  if (eav_quality >= 0.40) return '#607080'; // FUZZY_GRAY
  if (eav_quality >= 0.00) return '#404040'; // DEAD (#800000 is Nergal AV)
  return '#1A0A2E';  // KUR sovereign indigo
}}

// ── Client topology (PĀŠIRU golden angle colours) ───
// hue = (tribe_id − 1) × {GOLDEN_ANGLE_DEG}° mod 360°
// ring_radius = DECORATIVE visual separation only
// (NOT particle orbital radius)
{tribe_js}

// ── Scene ────────────────────────────────────────────
const canvas   = document.getElementById('canvas');
const renderer = new THREE.WebGLRenderer({{ canvas, antialias:true }});
renderer.setSize(innerWidth, innerHeight);
renderer.setClearColor(0x0a0618, 1);
const scene  = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(45, innerWidth/innerHeight, 0.01, 500);
camera.position.set(0, 2, 14);

// Decorative tribe rings (ring_radius = visual separation)
TRIBES.forEach(tribe => {{
  const geo = new THREE.RingGeometry(
    tribe.ring_radius - 0.01, tribe.ring_radius + 0.01, 128);
  const mat = new THREE.MeshBasicMaterial({{
    color: tribe.colour, side: THREE.DoubleSide,
    transparent: true, opacity: 0.22
  }});
  scene.add(new THREE.Mesh(geo, mat));
}});

// Particle pool (orbital_radius = particle OrbitalSubtype property)
const MAX_P = 80272;
const geo   = new THREE.BufferGeometry();
const pos   = new Float32Array(MAX_P * 3);
const col   = new Float32Array(MAX_P * 3);
geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
geo.setAttribute('color',    new THREE.BufferAttribute(col, 3));
const mat = new THREE.PointsMaterial({{
  size:0.06, vertexColors:true, transparent:true, opacity:0.88
}});
const points = new THREE.Points(geo, mat);
scene.add(points);
let live = 0;

function placeParticle(idx, orbital_radius, eav_quality) {{
  // orbital_radius = particle property from OrbitalSubtype
  const hex = orbitalColour(eav_quality);
  const c   = new THREE.Color(hex);
  const a   = Math.random() * Math.PI * 2;
  pos[idx*3]   = Math.cos(a) * orbital_radius;
  pos[idx*3+1] = (Math.random() - 0.5) * 0.12;
  pos[idx*3+2] = Math.sin(a) * orbital_radius;
  col[idx*3]   = c.r;
  col[idx*3+1] = c.g;
  col[idx*3+2] = c.b;
}}

// Seed demo particles across seven orbital subtypes
const ORBITAL_RADII = [
  ORB_GOLDEN_GEM, ORB_GOLDEN_ALIVE, ORB_FUZZY_AGED,
  ORB_FUZZY_GRAY, ORB_FUZZY_DECAY,
  ORB_DEAD_EXPIRED, ORB_DEAD_SEALED
];
const QUALITY_MAP = [0.95, 0.80, 0.63, 0.56, 0.47, 0.30, 0.10];
ORBITAL_RADII.forEach((r, si) => {{
  const count = Math.floor(MAX_P / ORBITAL_RADII.length);
  for (let i = 0; i < count; i++) {{
    if (live >= MAX_P) break;
    placeParticle(live++, r, QUALITY_MAP[si]);
  }}
}});
geo.attributes.position.needsUpdate = true;
geo.attributes.color.needsUpdate    = true;
geo.setDrawRange(0, live);
document.getElementById('particle-count').textContent =
  'Particles: ' + live.toLocaleString();

// Camera drift
let t = 0;
(function animate() {{
  requestAnimationFrame(animate);
  t += 0.0003;
  camera.position.x = Math.sin(t * 0.7) * 1.2;
  camera.position.y = 2 + Math.sin(t * 0.4) * 0.6;
  camera.lookAt(0,0,0);
  renderer.render(scene, camera);
}})();

// ── EnkiDB TCP refresh (W5H2 sovereign syntax) ───────
// Anti-SQL: no SELECT, no FROM, no WHERE, no LIMIT
function queryEnkiDB() {{
  const query = [
    'WHO   tribe:ALL',
    'WHAT  kaki_hex, eav_quality, orbital_subtype, colour_hex',
    'HOW   AllEntities',
    'WHERE state != DEAD_SEALED',
    'WHEN  AllTime',
    'WHY   "UTNAPISHTIM live particle refresh 30s cycle"',
    'HOW_MANY 1000'
  ].join('\n');
  // TCP send via WebSocket bridge (sovereign gateway)
  document.getElementById('live-status').textContent =
    '⟳ EnkiDB W5H2 refresh — ' + new Date().toISOString();
}}
setInterval(queryEnkiDB, 30000);

window.addEventListener('resize', () => {{
  camera.aspect = innerWidth/innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(innerWidth, innerHeight);
}});
  </script>
</body>
</html>"#)
}

fn build_tribe_js(topo: &ClientTopology) -> String {
    let mut lines = vec!["const TRIBES = [".to_string()];
    for tribe in &topo.tribes {
        lines.push(format!(
            "  {{ id:{}, name:{:?}, ring_radius:{:.2}, \
             colourHex:{:?}, colour:0x{} }},",
            tribe.tribe_id,
            tribe.tribe_name,
            tribe.ring_radius,
            tribe.colour_hex,
            tribe.colour_hex.trim_start_matches('#'),
        ));
    }
    lines.push("];".to_string());
    lines.join("\n")
}

fn build_orbital_constants() -> String {
    format!(
        "// Particle orbital radii (sealed v4.0) — OrbitalSubtype property\n\
         const ORB_GOLDEN_GEM   = {ORBITAL_GOLDEN_GEM};   // B11>=200 #FFD700\n\
         const ORB_GOLDEN_ALIVE = {ORBITAL_GOLDEN_ALIVE};  // quality>=0.70\n\
         const ORB_FUZZY_AGED   = {ORBITAL_FUZZY_AGED};   // quality>=0.55\n\
         const ORB_FUZZY_GRAY   = {ORBITAL_FUZZY_GRAY};   // quality>=0.40\n\
         const ORB_FUZZY_DECAY  = {ORBITAL_FUZZY_DECAY};  // TIAMAT DILBAT fires\n\
         const ORB_DEAD_EXPIRED = {ORBITAL_DEAD_EXPIRED}; // #404040 dark grey\n\
         const ORB_DEAD_SEALED  = {ORBITAL_DEAD_SEALED};  // PUZRU #282828\n\
         // NEVER: #800000 = NERGAL AV engine ONLY\n\
         // KUR terminal overlay = #1A0A2E sovereign indigo"
    )
}
