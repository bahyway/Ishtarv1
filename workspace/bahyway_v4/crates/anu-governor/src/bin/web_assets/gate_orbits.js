// Shala4 · Gate Orbits — a real Three.js particle-orbit view of the 7
// sealed `bahyway_core::hepta_gate::HeptaGate` sectors, adapted from the
// Architect's own uploaded BIGRING prototype (kept: Enlil's concentric
// rings, Adad's helix towers, Apsu's quantum-chaos attractor; four new
// scenes designed here for Shedu/Mummu/Enkidu/Dubsar).
//
// Deliberately its OWN ES module, loosely coupled to app.js via one
// global hook (`window.gateOrbits.onShown`, called from switchTab()) --
// every response shape here (`/api/gates/*`) is nothing like the
// dashboard-state shape app.js's shared `api()`/`apply()`/`render()`
// pipeline assumes everywhere else, so this file never touches that
// pipeline at all, and uses its own plain `fetch()` helper instead.
//
// Particle density in every scene below is decorative (tens of thousands
// of particles, tuned for visual effect) -- it is NOT a literal
// one-particle-per-playbook mapping. Real counts/titles come from the
// API and render as text (button labels, the detail panel's title list),
// never as a literal particle count.

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

// Order matches HeptaGate::all() / gate_number 1..7 exactly.
const GATE_ORDER = ['APSU', 'ADAD', 'SHEDU', 'MUMMU', 'ENKIDU', 'DUBSAR', 'ENLIL'];
const GATE_COLOR = {
  APSU: 0x4488ff, ADAD: 0xffd700, SHEDU: 0x66ddaa,
  MUMMU: 0xff66cc, ENKIDU: 0xff8844, DUBSAR: 0x88ccff, ENLIL: 0xffaa00,
};

function esc(s) {
  return String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
}

async function fetchJson(path, opts) {
  const res = await fetch(path, opts);
  if (res.status === 401) { window.location.href = '/login'; return null; }
  if (!res.ok) { return { error: await res.text().catch(() => res.statusText) }; }
  return res.json();
}

// ── Sidru layout law, adapted from the Architect's own uploaded DubSar
// Theater design (Design_Tablet_Sidru_Formatter.md + its PB-186 Rust
// scaffold's `nat_key`/`layout_orbit`, not yet a crate in this repo) for
// reading a hub-and-spoke graph without a hairball: natural-sort labels
// so numbered playbooks read in true order ("playbook_2" before
// "playbook_10", not after), and grow a ring's radius until its slots
// cannot collide instead of a fixed radius that overlaps once there are
// enough of them.

function natKey(s) {
  const out = [];
  let num = '', txt = '';
  for (const ch of String(s)) {
    if (ch >= '0' && ch <= '9') {
      if (txt) { out.push([0, 0, txt.toLowerCase()]); txt = ''; }
      num += ch;
    } else {
      if (num) { out.push([1, parseInt(num, 10), '']); num = ''; }
      txt += ch;
    }
  }
  if (num) out.push([1, parseInt(num, 10), '']);
  if (txt) out.push([0, 0, txt.toLowerCase()]);
  return out;
}

function natCompare(a, b) {
  const ka = natKey(a), kb = natKey(b);
  const len = Math.max(ka.length, kb.length);
  for (let i = 0; i < len; i++) {
    const [ta, na, sa] = ka[i] || [0, 0, ''];
    const [tb, nb, sb] = kb[i] || [0, 0, ''];
    if (ta !== tb) return ta - tb;
    if (ta === 1) { if (na !== nb) return na - nb; }
    else if (sa !== sb) return sa < sb ? -1 : 1;
  }
  return 0;
}

// `minRadius` is the visual floor (keeps a 1-2 item ring from looking
// tiny); `spacing` is the real cube footprint (BoxGeometry(2,2,2) plus a
// gap) -- matches the crate's `r = max(r_min, n * label_h / 2π)` exactly,
// just in 3D scene units instead of SVG pixels.
function collisionFreeRadius(n, minRadius, spacing = 3.2) {
  return Math.max(minRadius, (n * spacing) / (Math.PI * 2));
}

function hsvToRgb(h, s, v) {
  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s), q = v * (1 - f * s), t = v * (1 - (1 - f) * s);
  switch (i % 6) {
    case 0: return [v, t, p]; case 1: return [q, v, p]; case 2: return [p, v, t];
    case 3: return [p, q, v]; case 4: return [t, p, v]; default: return [v, p, q];
  }
}

// ── Three.js state (one scene, lazily created, reused across gate switches) ──
let scene, camera, renderer, controls, clock;
let cores = [];
let particles = null;
let particleData = [];
let sceneMode = 'ENLIL';
let sceneTime = 0;
let started = false;

function ensureScene() {
  if (started) return;
  started = true;
  const container = document.getElementById('gate-orbit-canvas');

  scene = new THREE.Scene();
  scene.fog = new THREE.FogExp2(0x020208, 0.0018);
  camera = new THREE.PerspectiveCamera(55, container.clientWidth / Math.max(container.clientHeight, 1), 0.1, 2000);
  camera.position.set(0, 40, 120);

  renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(container.clientWidth, container.clientHeight);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
  container.appendChild(renderer.domElement);

  controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  controls.autoRotate = true;
  controls.autoRotateSpeed = 0.25;
  controls.maxDistance = 500;
  controls.minDistance = 15;

  scene.add(new THREE.AmbientLight(0x1a1a3a, 0.5));
  const sun = new THREE.PointLight(0xffaa44, 4, 400);
  scene.add(sun);

  clock = new THREE.Clock();
  window.addEventListener('resize', onResize);
  wireRaycastClicks();
  buildScene('ENLIL');
  animate();
}

function onResize() {
  const container = document.getElementById('gate-orbit-canvas');
  if (!container || !renderer || !container.clientWidth) return;
  camera.aspect = container.clientWidth / Math.max(container.clientHeight, 1);
  camera.updateProjectionMatrix();
  renderer.setSize(container.clientWidth, container.clientHeight);
}

function makeTextSprite(text, color) {
  const c = document.createElement('canvas');
  c.width = 256; c.height = 64;
  const ctx = c.getContext('2d');
  ctx.font = 'bold 24px monospace';
  ctx.fillStyle = color || '#00ccff';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 128, 32);
  const tex = new THREE.CanvasTexture(c);
  const mat = new THREE.SpriteMaterial({ map: tex, transparent: true, depthWrite: false });
  const sprite = new THREE.Sprite(mat);
  sprite.scale.set(16, 4, 1);
  return sprite;
}

// ── Click targets: domain stations (Phase 4 drill-down) and playbook
// cubes (StoryEngine) both raycast against this list, separate from the
// decorative particle systems the orbit scenes use -- clicking a
// particle was never meaningful (tens of thousands of them, purely
// decorative), but clicking a station or a real playbook cube is.
let clickTargets = [];
const raycaster = new THREE.Raycaster();
const mouseVec = new THREE.Vector2();
let isDragging = false;
let mouseDownPos = { x: 0, y: 0 };

// Two DELIBERATELY separate indicators, matching the Architect's own
// reference design (a Ṣabātu bracket reticle replacing the OS cursor,
// squash-and-stretch bounce reserved for the CAUGHT cube -- no ring mesh,
// no cursor-icon swap):
//   - HOVER (`hoveredMesh`/`hoveredTarget`) drives ONLY the `#gates-
//     reticle`/`#gates-hoverchip` DOM overlay below -- a 4-bracket
//     reticle that follows the raw mouse position and locks (shrinks,
//     turns gold) over a real clickable target. It never touches the
//     3D scene.
//   - SELECTION (`selectedMesh`/`selectedTarget`), set only on a real
//     click ("catch"), drives the persistent squash-and-stretch bounce
//     in `updateScene`'s `cores.forEach` tail -- the cube itself IS the
//     "you got it" feedback. Cleared whenever `clearScene()` runs (the
//     view changed, so the old mesh no longer exists).
let hoveredMesh = null;
let hoveredTarget = null;
let selectedMesh = null;
let selectedTarget = null;

let reticleEl = null;
let chipEl = null;
let overCanvas = false;
// Raycast every ANIMATION FRAME (not just on mousemove) while the pointer
// sits over the canvas -- otherwise a click that changes the scene (a
// station drilling into its cube grid, a cube opening StoryEngine) would
// leave the reticle "locked" on gold and the chip showing stale text
// until the mouse physically moves again, since the target it was
// locked on no longer exists.
let lastMouseX = -1;
let lastMouseY = -1;

function chipTextFor(target) {
  if (target.kind === 'cube') return target.title;
  if (target.kind === 'station') return `${target.domainName} · ${target.gateName}`;
  return '';
}

function updateHoverTarget(clientX, clientY) {
  if (clickTargets.length === 0) {
    hoveredMesh = null;
    hoveredTarget = null;
  } else {
    const dom = renderer.domElement;
    const rect = dom.getBoundingClientRect();
    mouseVec.x = ((clientX - rect.left) / rect.width) * 2 - 1;
    mouseVec.y = -((clientY - rect.top) / rect.height) * 2 + 1;
    raycaster.setFromCamera(mouseVec, camera);
    const hits = raycaster.intersectObjects(clickTargets.map(t => t.mesh));
    if (hits.length) {
      hoveredMesh = hits[0].object;
      hoveredTarget = clickTargets.find(t => t.mesh === hoveredMesh) || null;
    } else {
      hoveredMesh = null;
      hoveredTarget = null;
    }
  }

  if (!reticleEl || !chipEl) return;
  if (overCanvas) {
    reticleEl.classList.remove('hidden');
    reticleEl.style.left = clientX + 'px';
    reticleEl.style.top = clientY + 'px';
  }
  if (hoveredTarget) {
    reticleEl.classList.add('lock');
    chipEl.textContent = chipTextFor(hoveredTarget);
    chipEl.style.left = clientX + 'px';
    chipEl.style.top = clientY + 'px';
    chipEl.style.display = 'block';
  } else {
    reticleEl.classList.remove('lock');
    chipEl.style.display = 'none';
  }
}

function wireRaycastClicks() {
  const dom = renderer.domElement;
  reticleEl = document.getElementById('gates-reticle');
  chipEl = document.getElementById('gates-hoverchip');

  dom.addEventListener('pointerenter', () => { overCanvas = true; });
  dom.addEventListener('pointerleave', () => {
    overCanvas = false;
    hoveredMesh = null;
    hoveredTarget = null;
    if (reticleEl) reticleEl.classList.add('hidden');
    if (chipEl) chipEl.style.display = 'none';
  });

  dom.addEventListener('mousedown', e => { isDragging = false; mouseDownPos = { x: e.clientX, y: e.clientY }; });
  dom.addEventListener('mousemove', e => {
    const dx = e.clientX - mouseDownPos.x, dy = e.clientY - mouseDownPos.y;
    if (Math.sqrt(dx * dx + dy * dy) > 4) isDragging = true;
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
  });
  dom.addEventListener('click', e => {
    if (isDragging || clickTargets.length === 0) return;
    const rect = dom.getBoundingClientRect();
    mouseVec.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
    mouseVec.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;
    raycaster.setFromCamera(mouseVec, camera);
    const hits = raycaster.intersectObjects(clickTargets.map(t => t.mesh));
    if (!hits.length) return;
    const target = clickTargets.find(t => t.mesh === hits[0].object);
    if (!target) return;
    if (target.kind === 'station') showDomainCubes(target.gateName, target.domainName);
    else if (target.kind === 'cube') {
      // "Catch" -- this is now the selected cube, so it gets the
      // persistent squash-and-stretch bounce (`updateScene`'s
      // `cores.forEach` tail) instead of whatever was selected before.
      selectedMesh = target.mesh;
      selectedTarget = target;
      if (sceneMode === 'PLAYBOOK_ORBIT' && currentStoryTitle) {
        returnToView = () => openStoryEngine(currentStoryTitle);
      } else if (sceneMode === 'SEARCH_CUBES' && lastSearchQuery) {
        returnToView = () => runSearch(lastSearchQuery);
      } else {
        returnToView = null;
      }
      openStoryEngine(target.title);
    }
  });
}

function clearScene() {
  if (particles) {
    scene.remove(particles);
    particles.geometry.dispose();
    particles.material.dispose();
    particles = null;
  }
  particleData = [];
  // `cores` doubles as the generic "every extra mesh this scene owns"
  // cleanup list -- core spheres in the orbit scenes, but also domain
  // station cylinders/labels and playbook cubes in the drill-down views
  // below, all get pushed here specifically so this one removal loop
  // covers every scene mode. `clickTargets` only tracks WHICH of those
  // meshes are clickable and what they mean -- it never owns removal.
  cores.forEach(c => scene.remove(c.mesh));
  cores = [];
  clickTargets = [];
  // The selected mesh belongs to whichever scene just got cleared -- it
  // no longer exists, so the catch bounce must not keep chasing it.
  selectedMesh = null;
  selectedTarget = null;
  hoveredMesh = null;
  hoveredTarget = null;
}

function makeCore(pos, color, size) {
  const geo = new THREE.IcosahedronGeometry(size, 3);
  const mat = new THREE.MeshStandardMaterial({ color, emissive: color, emissiveIntensity: 0.7, roughness: 0.25, metalness: 0.7 });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.position.copy(pos);
  scene.add(mesh);
  cores.push({ mesh, basePos: pos.clone(), phase: Math.random() * Math.PI * 2 });
}

function createPoints(positions, colors) {
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  const mat = new THREE.PointsMaterial({
    size: 1.3, vertexColors: true, transparent: true, opacity: 0.85,
    blending: THREE.AdditiveBlending, depthWrite: false, sizeAttenuation: true,
  });
  particles = new THREE.Points(geo, mat);
  scene.add(particles);
}

// ── Scene builders ──────────────────────────────────────────────────────

function buildEnlil() { // Governance -- concentric rings around a core
  clearScene();
  const count = 24000;
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  const data = [];
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR.ENLIL, 6);
  const rings = [
    { rMin: 14, rMax: 22, speed: 0.6, hue0: 0.12, hue1: 0.18 },
    { rMin: 30, rMax: 42, speed: 0.35, hue0: 0.1, hue1: 0.15 },
    { rMin: 52, rMax: 68, speed: 0.2, hue0: 0.08, hue1: 0.13 },
  ];
  let idx = 0;
  rings.forEach((ring, ri) => {
    const per = Math.floor(count / rings.length);
    for (let i = 0; i < per && idx < count; i++, idx++) {
      const r = ring.rMin + Math.random() * (ring.rMax - ring.rMin);
      const theta = Math.random() * Math.PI * 2;
      const y = (Math.random() - 0.5) * (2 + ri);
      pos[idx * 3] = Math.cos(theta) * r; pos[idx * 3 + 1] = y; pos[idx * 3 + 2] = Math.sin(theta) * r;
      const [rr, gg, bb] = hsvToRgb(ring.hue0 + Math.random() * (ring.hue1 - ring.hue0), 0.75, 0.7 + Math.random() * 0.3);
      col[idx * 3] = rr; col[idx * 3 + 1] = gg; col[idx * 3 + 2] = bb;
      data.push({ angle: theta, radius: r, speed: ring.speed, yBase: y, yPhase: Math.random() * Math.PI * 2 });
    }
  });
  createPoints(pos, col);
  particleData = data;
}

function buildAdad() { // ETL/Ingestion -- helix towers
  clearScene();
  const count = 26000;
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  const data = [];
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR.ADAD, 5);
  const towers = 6, towerRadius = 26;
  for (let i = 0; i < count; i++) {
    const tower = Math.floor(Math.random() * towers);
    const towerAngle = (tower / towers) * Math.PI * 2;
    const r = Math.random() * 10;
    const localAngle = Math.random() * Math.PI * 2;
    const y = (Math.random() - 0.5) * 80;
    const spiral = y * 0.08;
    const x = Math.cos(towerAngle + spiral) * towerRadius + Math.cos(localAngle) * r;
    const z = Math.sin(towerAngle + spiral) * towerRadius + Math.sin(localAngle) * r;
    pos[i * 3] = x; pos[i * 3 + 1] = y; pos[i * 3 + 2] = z;
    const hue = 0.1 + (y + 40) / 80 * 0.15;
    const [rr, gg, bb] = hsvToRgb(hue, 0.7, 0.7 + Math.random() * 0.3);
    col[i * 3] = rr; col[i * 3 + 1] = gg; col[i * 3 + 2] = bb;
    data.push({ towerAngle, towerRadius, y, r, localAngle, speed: 0.3 + Math.random() * 0.2, yPhase: Math.random() * Math.PI * 2 });
  }
  createPoints(pos, col);
  particleData = data;
}

function buildApsu() { // Storage -- quantum-chaos attractor swarm
  clearScene();
  const count = 20000;
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  const data = [];
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR.APSU, 5);
  makeCore(new THREE.Vector3(-26, 16, -18), 0x2255cc, 2.5);
  makeCore(new THREE.Vector3(26, -14, 22), 0x66aaff, 2.5);
  for (let i = 0; i < count; i++) {
    const x = (Math.random() - 0.5) * 110, y = (Math.random() - 0.5) * 70, z = (Math.random() - 0.5) * 110;
    pos[i * 3] = x; pos[i * 3 + 1] = y; pos[i * 3 + 2] = z;
    const [rr, gg, bb] = hsvToRgb(0.55 + Math.random() * 0.15, 0.7, 0.6 + Math.random() * 0.4);
    col[i * 3] = rr; col[i * 3 + 1] = gg; col[i * 3 + 2] = bb;
    data.push({ x, y, z, vx: 0, vy: 0, vz: 0 });
  }
  createPoints(pos, col);
  particleData = data;
}

function buildShedu() { // Security -- dense shield-dome shell guarding the core
  clearScene();
  const count = 22000;
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  const data = [];
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR.SHEDU, 5);
  const shells = [30, 38, 46];
  let idx = 0;
  shells.forEach((radius, si) => {
    const per = Math.floor(count / shells.length);
    for (let i = 0; i < per && idx < count; i++, idx++) {
      // Uniform points on a sphere (Marsaglia), biased toward the upper
      // hemisphere so the "dome" reads as guarding from above.
      let u, v, s;
      do { u = Math.random() * 2 - 1; v = Math.random() * 2 - 1; s = u * u + v * v; } while (s >= 1);
      const factor = 2 * Math.sqrt(1 - s);
      const x = u * factor, y = Math.abs(1 - 2 * s), z = v * factor;
      const r = radius + (Math.random() - 0.5) * 2;
      pos[idx * 3] = x * r; pos[idx * 3 + 1] = y * r * 0.8; pos[idx * 3 + 2] = z * r;
      const [rr, gg, bb] = hsvToRgb(0.42 + si * 0.02, 0.6, 0.6 + Math.random() * 0.4);
      col[idx * 3] = rr; col[idx * 3 + 1] = gg; col[idx * 3 + 2] = bb;
      data.push({ dir: new THREE.Vector3(x, y * 0.8, z), radius: r, shell: si, phase: Math.random() * Math.PI * 2 });
    }
  });
  createPoints(pos, col);
  particleData = data;
}

function buildMummu() { // Algebra -- a 3D Lissajous/torus-knot curve
  clearScene();
  const count = 24000;
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  const data = [];
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR.MUMMU, 4);
  const a = 3, b = 4, c = 5, scale = 34;
  for (let i = 0; i < count; i++) {
    const t = (i / count) * Math.PI * 2 * 6 + Math.random() * 0.15;
    const jitter = 1.5 + Math.random() * 2.5;
    const x = Math.sin(a * t) * scale, y = Math.sin(b * t) * scale * 0.6, z = Math.sin(c * t + Math.PI / 4) * scale;
    const nx = x + (Math.random() - 0.5) * jitter, ny = y + (Math.random() - 0.5) * jitter, nz = z + (Math.random() - 0.5) * jitter;
    pos[i * 3] = nx; pos[i * 3 + 1] = ny; pos[i * 3 + 2] = nz;
    const [rr, gg, bb] = hsvToRgb(0.83 + Math.sin(t) * 0.05, 0.65, 0.65 + Math.random() * 0.35);
    col[i * 3] = rr; col[i * 3 + 1] = gg; col[i * 3 + 2] = bb;
    data.push({ t0: t, jitter });
  }
  createPoints(pos, col);
  particleData = data;
}

function buildEnkidu() { // AI Agents -- a flocking swarm following a moving leader
  clearScene();
  const count = 18000;
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  const data = [];
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR.ENKIDU, 4);
  for (let i = 0; i < count; i++) {
    const offset = new THREE.Vector3((Math.random() - 0.5) * 60, (Math.random() - 0.5) * 40, (Math.random() - 0.5) * 60);
    pos[i * 3] = offset.x; pos[i * 3 + 1] = offset.y; pos[i * 3 + 2] = offset.z;
    const [rr, gg, bb] = hsvToRgb(0.06 + Math.random() * 0.06, 0.75, 0.6 + Math.random() * 0.4);
    col[i * 3] = rr; col[i * 3 + 1] = gg; col[i * 3 + 2] = bb;
    data.push({ offset, jitterPhase: Math.random() * Math.PI * 2, jitterAmp: 4 + Math.random() * 8 });
  }
  createPoints(pos, col);
  particleData = data;
}

function buildDubsar() { // Languages -- particles on a rotating 3D lattice grid
  clearScene();
  const dims = 30; // 30^3 would be too many; sample a subset of grid cells
  const count = 24000;
  const pos = new Float32Array(count * 3), col = new Float32Array(count * 3);
  const data = [];
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR.DUBSAR, 4);
  const spacing = 6;
  for (let i = 0; i < count; i++) {
    const gx = Math.floor(Math.random() * dims) - dims / 2;
    const gy = Math.floor(Math.random() * dims) - dims / 2;
    const gz = Math.floor(Math.random() * dims) - dims / 2;
    const base = new THREE.Vector3(gx * spacing, gy * spacing, gz * spacing);
    // Keep only a shell of the cube -- a solid dims^3 fill reads as noise.
    if (base.length() > dims * spacing * 0.55) { i--; continue; }
    pos[i * 3] = base.x; pos[i * 3 + 1] = base.y; pos[i * 3 + 2] = base.z;
    const [rr, gg, bb] = hsvToRgb(0.56 + Math.random() * 0.06, 0.55, 0.6 + Math.random() * 0.35);
    col[i * 3] = rr; col[i * 3 + 1] = gg; col[i * 3 + 2] = bb;
    data.push({ base, wavePhase: base.length() * 0.05 });
  }
  createPoints(pos, col);
  particleData = data;
}

const SCENE_BUILDERS = {
  ENLIL: buildEnlil, ADAD: buildAdad, APSU: buildApsu,
  SHEDU: buildShedu, MUMMU: buildMummu, ENKIDU: buildEnkidu, DUBSAR: buildDubsar,
};

function buildScene(gateName) {
  sceneMode = gateName;
  (SCENE_BUILDERS[gateName] || buildEnlil)();
}

// ── Per-frame update, one branch per scene mode ─────────────────────────
function updateScene(dt) {
  // FIXED, real bug: this used to be `if (!particles) return;` at the top
  // -- but the drill-down scenes (DOMAINS/DOMAIN_CUBES/SEARCH_CUBES/
  // PLAYBOOK_ORBIT) never call `createPoints()` (they only populate
  // `cores`), so `particles` is null there and that early return skipped
  // the `cores.forEach` tail below ENTIRELY for every one of those views
  // -- not just the catch bounce, the ambient idle bob too. Only the
  // particle-position update actually needs `particles`; `cores.forEach`
  // must run every frame regardless of scene mode.
  if (particles) {
    const positions = particles.geometry.attributes.position.array;

    if (sceneMode === 'ENLIL') {
      for (let i = 0; i < particleData.length; i++) {
        const d = particleData[i];
        d.angle += d.speed * dt;
        positions[i * 3] = Math.cos(d.angle) * d.radius;
        positions[i * 3 + 1] = d.yBase + Math.sin(sceneTime + d.yPhase) * 1.5;
        positions[i * 3 + 2] = Math.sin(d.angle) * d.radius;
      }
    } else if (sceneMode === 'ADAD') {
      for (let i = 0; i < particleData.length; i++) {
        const d = particleData[i];
        d.towerAngle += d.speed * dt * 0.3;
        const spiral = d.y * 0.08 + sceneTime * 0.2;
        positions[i * 3] = Math.cos(d.towerAngle + spiral) * d.towerRadius + Math.cos(d.localAngle + sceneTime) * d.r;
        positions[i * 3 + 1] = d.y + Math.sin(sceneTime + d.yPhase) * 2;
        positions[i * 3 + 2] = Math.sin(d.towerAngle + spiral) * d.towerRadius + Math.sin(d.localAngle + sceneTime) * d.r;
      }
    } else if (sceneMode === 'APSU') {
      const attractors = [{ x: 0, y: 0, z: 0, s: 0.02 }, { x: -26, y: 16, z: -18, s: 0.012 }, { x: 26, y: -14, z: 22, s: 0.012 }];
      for (let i = 0; i < particleData.length; i++) {
        const d = particleData[i];
        let ax = 0, ay = 0, az = 0;
        attractors.forEach(a => {
          const dx = a.x - d.x, dy = a.y - d.y, dz = a.z - d.z;
          const dist = Math.sqrt(dx * dx + dy * dy + dz * dz) + 1;
          const f = a.s / (dist * dist);
          ax += dx * f; ay += dy * f; az += dz * f;
        });
        ax += 0.008 * (d.y - d.x); ay += 0.008 * (d.x * (24 - d.z) * 0.01 - d.y); az += 0.008 * (d.x * d.y * 0.001 - d.z * 0.22);
        d.vx = (d.vx + ax) * 0.985; d.vy = (d.vy + ay) * 0.985; d.vz = (d.vz + az) * 0.985;
        d.x += d.vx; d.y += d.vy; d.z += d.vz;
        const bound = 90;
        if (Math.abs(d.x) > bound) { d.x *= 0.95; d.vx *= -0.3; }
        if (Math.abs(d.y) > bound) { d.y *= 0.95; d.vy *= -0.3; }
        if (Math.abs(d.z) > bound) { d.z *= 0.95; d.vz *= -0.3; }
        positions[i * 3] = d.x; positions[i * 3 + 1] = d.y; positions[i * 3 + 2] = d.z;
      }
    } else if (sceneMode === 'SHEDU') {
      for (let i = 0; i < particleData.length; i++) {
        const d = particleData[i];
        const pulse = 1 + Math.sin(sceneTime * 0.6 + d.phase) * 0.03;
        positions[i * 3] = d.dir.x * d.radius * pulse;
        positions[i * 3 + 1] = d.dir.y * d.radius * pulse;
        positions[i * 3 + 2] = d.dir.z * d.radius * pulse;
      }
    } else if (sceneMode === 'MUMMU') {
      const a = 3, b = 4, c = 5, scale = 34;
      for (let i = 0; i < particleData.length; i++) {
        const d = particleData[i];
        const t = d.t0 + sceneTime * 0.15;
        positions[i * 3] = Math.sin(a * t) * scale + (Math.random() - 0.5) * 0.05;
        positions[i * 3 + 1] = Math.sin(b * t) * scale * 0.6;
        positions[i * 3 + 2] = Math.sin(c * t + Math.PI / 4) * scale;
      }
    } else if (sceneMode === 'ENKIDU') {
      const leader = new THREE.Vector3(Math.sin(sceneTime * 0.3) * 24, Math.sin(sceneTime * 0.21) * 12, Math.cos(sceneTime * 0.27) * 24);
      for (let i = 0; i < particleData.length; i++) {
        const d = particleData[i];
        const jitter = Math.sin(sceneTime * 1.4 + d.jitterPhase) * d.jitterAmp;
        positions[i * 3] = leader.x + d.offset.x * 0.4 + jitter * 0.2;
        positions[i * 3 + 1] = leader.y + d.offset.y * 0.4 + Math.cos(sceneTime + d.jitterPhase) * d.jitterAmp * 0.15;
        positions[i * 3 + 2] = leader.z + d.offset.z * 0.4 + jitter * 0.2;
      }
      cores[0].mesh.position.copy(leader);
    } else if (sceneMode === 'DUBSAR') {
      const angle = sceneTime * 0.15;
      const cosA = Math.cos(angle), sinA = Math.sin(angle);
      for (let i = 0; i < particleData.length; i++) {
        const d = particleData[i];
        const wave = Math.sin(sceneTime * 0.8 - d.wavePhase) * 1.5;
        const x = d.base.x, z = d.base.z;
        positions[i * 3] = x * cosA - z * sinA;
        positions[i * 3 + 1] = d.base.y + wave;
        positions[i * 3 + 2] = x * sinA + z * cosA;
      }
    }

    particles.geometry.attributes.position.needsUpdate = true;
  }

  cores.forEach((c, i) => {
    if (sceneMode === 'ENKIDU' && i === 0) return; // leader-driven, positioned above
    if (c.mesh === selectedMesh) {
      // The CAUGHT cube's own squash-and-stretch bounce IS the "you got
      // it" feedback -- no ring mesh, no cursor swap. `s` is a 0->1 hop
      // phase (`Math.abs(sin(...))`, a real bounce cadence: sharp launch
      // off the floor, sharp return, not a smooth sway). No continuous
      // rotation while caught -- a spinning cube reads as "still idle,"
      // not "selected"; the bounce+squash alone is the signal.
      const s = Math.abs(Math.sin(sceneTime * 4));
      c.mesh.position.y = c.basePos.y + s * 2.4;
      c.mesh.scale.set(1.12 - 0.12 * s, 0.82 + 0.32 * s, 1.12 - 0.12 * s); // squash at floor, stretch at apex
      if (c.mesh.material.emissive) {
        const base = c.mesh.userData.baseEmissiveIntensity ?? c.mesh.material.emissiveIntensity ?? 0.4;
        c.mesh.userData.baseEmissiveIntensity = base;
        c.mesh.material.emissiveIntensity = base + s * 0.8; // faint breathing glow, reads at distance
      }
    } else if (c.static) {
      // Domain stations, playbook cube grids, and dependency orbit-ring
      // cubes -- FIXED, real complaint: these used to idle-bob/rotate at
      // `speed * (i + 1)`, meaning a 60-cube grid's LAST cube spun ~60x
      // faster than its first, reading as chaotic "shivering," not motion
      // with any meaning. Only the caught cube moves now; everything
      // else sits exactly at rest so the catch bounce is the one thing
      // drawing the eye, matching the reference design's actual intent.
      if (c.mesh.position.y !== c.basePos.y) c.mesh.position.y = c.basePos.y;
      if (c.mesh.scale.x !== 1 || c.mesh.scale.y !== 1 || c.mesh.scale.z !== 1) c.mesh.scale.set(1, 1, 1);
      if (c.mesh.material.emissive && c.mesh.userData.baseEmissiveIntensity !== undefined) {
        c.mesh.material.emissiveIntensity = c.mesh.userData.baseEmissiveIntensity;
      }
    } else {
      // True decorative cores only (the 1-3 orbit-scene nucleus spheres
      // from `makeCore`) -- a gentle, FIXED-speed idle bob, never scaled
      // by array index since there are only ever a couple of these.
      c.mesh.rotation.x += 0.003;
      c.mesh.rotation.y += 0.005;
      c.mesh.position.y = c.basePos.y + Math.sin(sceneTime * 0.5 + c.phase) * 2;
      if (c.mesh.scale.x !== 1 || c.mesh.scale.y !== 1 || c.mesh.scale.z !== 1) c.mesh.scale.set(1, 1, 1); // reset after a catch ends
      if (c.mesh.material.emissive && c.mesh.userData.baseEmissiveIntensity !== undefined) {
        c.mesh.material.emissiveIntensity = c.mesh.userData.baseEmissiveIntensity;
      }
    }
  });
}

function animate() {
  requestAnimationFrame(animate);
  const dt = clock.getDelta();
  sceneTime += dt;
  if (overCanvas) updateHoverTarget(lastMouseX, lastMouseY);
  updateScene(dt);
  controls.update();
  renderer.render(scene, camera);
}

// ── Data wiring: buttons, detail panel, review flow ─────────────────────

let lastSummary = null;
let selectedGate = null;
let architectView = false;

function renderButtons() {
  const el = document.getElementById('gates-buttons');
  if (!lastSummary) { el.innerHTML = '<span class="dim">loading…</span>'; return; }
  el.innerHTML = lastSummary.gates.map(g => `
    <button class="gate-btn${g.akkadian_name === selectedGate ? ' active' : ''}" data-gate="${esc(g.akkadian_name)}">
      ${esc(g.akkadian_name)} (${g.playbook_count})
    </button>
  `).join('');
  el.querySelectorAll('.gate-btn').forEach(btn => {
    btn.addEventListener('click', () => selectGate(btn.dataset.gate));
  });
}

function renderSummaryLine() {
  const el = document.getElementById('gates-summary-line');
  if (!lastSummary) { el.textContent = 'loading…'; return; }
  // FIXED, live: gate/domain counts, dependency edges, and playbook_detail
  // all read from the published `current` EnkiDDB generation -- if it was
  // never published (or is stale after a fresh catalog scan), every one of
  // those reads back empty/404 even though the catalog itself is real and
  // full. Spelling that out here (rather than just showing "0 unclassified")
  // is what tells an Architect "click Refresh," not "something is broken."
  const publishNote = lastSummary.current_published
    ? ''
    : ' · ⚠ EnkiDDB not published yet -- gate/domain counts and playbook detail will be empty until an Architect clicks Refresh Knowledge Graph';
  el.textContent =
    `${lastSummary.total_playbooks} catalogued playbook(s) · ${lastSummary.unclassified_count} unclassified` +
    ` · ${lastSummary.approved_dependency_count || 0} depends-on edge(s) approved` +
    publishNote;

  const reviewBtn = document.getElementById('btn-gates-review');
  if (architectView && lastSummary.unclassified_count > 0) {
    reviewBtn.textContent = `Review ${lastSummary.unclassified_count} unclassified`;
    reviewBtn.classList.remove('hidden');
  } else {
    reviewBtn.classList.add('hidden');
  }

  // No cheap live count for dependency suggestions (the scan is an O(n^2)
  // whole-word text scan, not indexed) -- shown whenever an Architect is
  // viewing at all, rather than gated on a count fetched every summary
  // refresh; `openDepReview` reports "nothing pending" itself when empty.
  document.getElementById('btn-gates-dep-review').classList.toggle('hidden', !architectView);
  document.getElementById('btn-gates-refresh').classList.toggle('hidden', !architectView);
}

async function refreshSummary() {
  const data = await fetchJson('/api/gates/summary');
  if (!data || data.error) return;
  lastSummary = data;
  renderButtons();
  renderSummaryLine();
}

function gateHeaderHtml(gate) {
  return `<p><strong>${esc(gate.sector)}</strong> · ${esc(gate.pipeline_function)} · G${gate.gate_number}</p>
    <p class="dim">${gate.playbook_count} playbook(s) tagged ${esc(gate.akkadian_name)}</p>`;
}

function neighborButtonHtml(neighbor, label) {
  if (!neighbor) return `<button class="btn" disabled>${label}: none</button>`;
  return `<button class="btn gate-neighbor-btn" data-gate="${esc(neighbor.akkadian_name)}">
    ${label}: ${esc(neighbor.akkadian_name)} (${neighbor.playbook_count})
  </button>`;
}

async function selectGate(gateName) {
  selectedGate = gateName;
  renderButtons();
  buildScene(gateName);

  const title = document.getElementById('gates-detail-title');
  const body = document.getElementById('gates-detail-body');
  title.textContent = gateName;
  body.innerHTML = '<p class="dim">loading…</p>';

  const data = await fetchJson(`/api/gates/${encodeURIComponent(gateName)}/playbooks`);
  if (!data || data.error) { body.innerHTML = `<p class="dim">${esc(data && data.error || 'failed to load')}</p>`; return; }

  const listHtml = data.playbooks.length
    ? `<ul class="gate-pb-list">${data.playbooks.map(t => `<li>${esc(t)}</li>`).join('')}</ul>`
    : '<p class="dim">No playbooks classified into this gate yet.</p>';

  body.innerHTML = `
    ${gateHeaderHtml(data.gate)}
    ${data.note ? `<p class="dim">${esc(data.note)}</p>` : ''}
    <div class="gate-neighbor-row">
      ${neighborButtonHtml(data.lower, '◀ lower')}
      ${neighborButtonHtml(data.upper, 'upper ▶')}
    </div>
    <button class="btn btn-primary" id="btn-show-domains">Show 7 Domains ▼</button>
    ${listHtml}
  `;
  body.querySelectorAll('.gate-neighbor-btn').forEach(btn => {
    btn.addEventListener('click', () => selectGate(btn.dataset.gate));
  });
  const domainsBtn = document.getElementById('btn-show-domains');
  if (domainsBtn) domainsBtn.addEventListener('click', () => showDomainStations(gateName));
}

// ── Domain stations + StoryEngine (Phase 4 drill-down) ──────────────────
//
// Adapted from the Architect's own uploaded ETL Transparency Dashboard
// prototype: 7 gate-domain "stations" arranged in a ring around a core,
// click one to see the real playbooks tagged into it as a cube grid,
// click a cube to open a StoryEngine panel -- but unlike that prototype's
// SIMULATED particle intake and synthetic event templates, every count,
// title, and attribute shown here comes straight from a live
// /api/gates/... or /api/playbook_detail query against real EnkiDDB data.

let domainStationGate = null;

async function showDomainStations(gateName) {
  clearScene();
  sceneMode = 'DOMAINS';
  domainStationGate = gateName;
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR[gateName] || 0xffaa00, 4);

  const title = document.getElementById('gates-detail-title');
  const body = document.getElementById('gates-detail-body');
  title.textContent = `${gateName} · Domains`;
  body.innerHTML = '<p class="dim">loading…</p>';

  const data = await fetchJson(`/api/gates/${encodeURIComponent(gateName)}/domains/summary`);
  if (!data || data.error) { body.innerHTML = `<p class="dim">${esc((data && data.error) || 'failed to load')}</p>`; return; }

  const domains = data.domains || [];
  domains.forEach((d, i) => {
    const angle = (i / domains.length) * Math.PI * 2;
    const x = Math.cos(angle) * 34, z = Math.sin(angle) * 34;

    const cylMat = new THREE.MeshBasicMaterial({ color: 0x224466, transparent: true, opacity: 0.18, wireframe: true });
    const cyl = new THREE.Mesh(new THREE.CylinderGeometry(6, 6, 14, 12, 1, true), cylMat);
    cyl.position.set(x, -7, z);
    scene.add(cyl);
    cores.push({ mesh: cyl, basePos: cyl.position.clone(), phase: 0, static: true });
    clickTargets.push({ mesh: cyl, kind: 'station', gateName, domainName: d.domain_name });

    const label = makeTextSprite(`${d.domain_name} (${d.playbook_count})`, d.playbook_count > 0 ? '#00ccff' : '#556677');
    label.position.set(x, 2, z);
    scene.add(label);
    cores.push({ mesh: label, basePos: label.position.clone(), phase: 0, static: true });
  });

  body.innerHTML = `
    <button class="btn" id="btn-back-to-gate">◀ back to ${esc(gateName)}</button>
    <p class="dim">${data.gate_total} playbook(s) in ${esc(gateName)} · ${data.unclassified_in_gate} not yet assigned a domain</p>
    ${data.note ? `<p class="dim">${esc(data.note)}</p>` : ''}
    <p class="dim">Click a station to see its playbooks.</p>
  `;
  document.getElementById('btn-back-to-gate').addEventListener('click', () => selectGate(gateName));
}

async function showDomainCubes(gateName, domainName) {
  clearScene();
  sceneMode = 'DOMAIN_CUBES';
  makeCore(new THREE.Vector3(0, 0, 0), GATE_COLOR[gateName] || 0xffaa00, 3);

  const title = document.getElementById('gates-detail-title');
  const body = document.getElementById('gates-detail-body');
  title.textContent = `${gateName} · ${domainName}`;
  body.innerHTML = '<p class="dim">loading…</p>';

  const data = await fetchJson(`/api/gates/${encodeURIComponent(gateName)}/domains/${encodeURIComponent(domainName)}/playbooks`);
  if (!data || data.error) { body.innerHTML = `<p class="dim">${esc((data && data.error) || 'failed to load')}</p>`; return; }

  // Sidru's ORBIT law again -- natural-sort so the grid (and the CUBE_CAP
  // slice below) reads in true playbook-number order, not whatever order
  // the API happened to return.
  const all = (data.playbooks || []).slice().sort((a, b) => natCompare(a.title, b.title));
  const CUBE_CAP = 60; // decorative cap -- perf, not a data limit; full list still shown in the panel
  const shown = all.slice(0, CUBE_CAP);
  const cols = 6, spacing = 3.4;
  const cubeGeo = new THREE.BoxGeometry(2.2, 2.2, 2.2);
  shown.forEach((pb, i) => {
    const col = i % cols, row = Math.floor(i / cols);
    const mat = new THREE.MeshStandardMaterial({ color: 0x00cc66, emissive: 0x004422, emissiveIntensity: 0.5 });
    const mesh = new THREE.Mesh(cubeGeo, mat);
    mesh.position.set((col - (cols - 1) / 2) * spacing, row * spacing - 4, 12);
    scene.add(mesh);
    cores.push({ mesh, basePos: mesh.position.clone(), phase: 0, static: true });
    clickTargets.push({ mesh, kind: 'cube', title: pb.title, kakiHex: pb.kaki_hex });
  });

  const listHtml = all.length
    ? `<ul class="gate-pb-list">${all.map(pb => `<li data-title="${esc(pb.title)}" class="gate-pb-clickable">${esc(pb.title)}</li>`).join('')}</ul>`
    : '<p class="dim">No playbooks classified into this domain yet.</p>';

  body.innerHTML = `
    <button class="btn" id="btn-back-to-domains">◀ back to ${esc(gateName)} domains</button>
    <p class="dim">${all.length} playbook(s)${all.length > CUBE_CAP ? ` -- showing the first ${CUBE_CAP} as cubes, full list below` : ''}</p>
    ${listHtml}
  `;
  document.getElementById('btn-back-to-domains').addEventListener('click', () => showDomainStations(gateName));
  body.querySelectorAll('.gate-pb-clickable').forEach(li => {
    li.addEventListener('click', () => {
      returnToView = () => showDomainCubes(gateName, domainName);
      openStoryEngine(li.dataset.title);
    });
  });
}

// Set by whichever view opens a StoryEngine panel (a domain's cube grid,
// a search-results list, or a dependency link inside another StoryEngine
// panel) just before calling openStoryEngine, so its "back" button
// returns to wherever the Architect actually came from -- walking a
// chain of "X depends on Y depends on Z" one hop back at a time, not
// always jumping to the domain-stations view.
let returnToView = null;
let currentStoryTitle = null;

function dependencyListHtml(titles, label, mutualTitles) {
  if (!titles.length) return `<p class="dim">${esc(label)}: none.</p>`;
  const rows = [...titles].sort(natCompare).map(t => {
    const badge = mutualTitles && mutualTitles.has(t)
      ? ' <span style="color:#fff" title="A real mutual/circular dependency -- also see it highlighted white in the 3D ring">⇄ mutual</span>'
      : '';
    return `<li class="gate-pb-clickable" data-title="${esc(t)}">${esc(t)}${badge}</li>`;
  }).join('');
  return `<p class="dim">${esc(label)} (${titles.length}):</p><ul class="gate-pb-list">${rows}</ul>`;
}

// Real chronicle -- every card here is a real, timestamped fact already
// durably recorded in one of the 4 append-only registries (catalog/gate/
// domain/dependency), returned by /api/playbook_detail's `story` field.
// Never a simulated Epoch/VGCA/B11 template like the uploaded prototypes'
// StoryEngine -- if nothing real has happened yet, this says so plainly.
const STORY_KIND_CLASS = {
  CATALOGUED: 'se-catalogued',
  GATE_APPROVED: 'se-gate',
  DOMAIN_APPROVED: 'se-domain',
  DEPENDS_ON_APPROVED: 'se-depends',
  DEPENDED_ON_BY_APPROVED: 'se-depended',
};

function chronicleHtml(story) {
  if (!story || !story.length) {
    return `
      <p class="story-chronicle-heading">Chronicle</p>
      <p class="dim">No recorded events yet -- nothing catalogued/classified/linked for this KAKI so far.</p>
    `;
  }
  const cards = story.map(ev => `
    <div class="story-epoch ${STORY_KIND_CLASS[ev.kind] || ''}">
      <div class="story-epoch-ts">${esc(ev.ts || '')}</div>
      <div class="story-epoch-title">${esc(ev.title || ev.kind || '')}</div>
      <div class="story-epoch-detail">${esc(ev.detail || '')}</div>
    </div>
  `).join('');
  return `<p class="story-chronicle-heading">Chronicle — ${story.length} real event(s)</p>${cards}`;
}

async function openStoryEngine(title) {
  const detailTitle = document.getElementById('gates-detail-title');
  const body = document.getElementById('gates-detail-body');
  detailTitle.textContent = title;
  body.innerHTML = '<p class="dim">loading…</p>';

  const data = await fetchJson(`/api/playbook_detail?title=${encodeURIComponent(title)}`);
  const goBack = returnToView || (domainStationGate ? () => showDomainStations(domainStationGate) : null);
  if (!data || data.error) {
    body.innerHTML = `
      <button class="btn" id="btn-story-back">◀ back</button>
      <p class="dim">${esc((data && data.error) || 'could not load detail')}</p>
    `;
    document.getElementById('btn-story-back').addEventListener('click', () => { if (goBack) goBack(); });
    return;
  }

  currentStoryTitle = title;
  const dependsOn = data.depends_on || [];
  const dependedOnBy = data.depended_on_by || [];
  showPlaybookOrbit(dependsOn, dependedOnBy);

  const attrs = data.attrs || {};
  // The real state this playbook currently holds -- gate/domain
  // classification is the only lifecycle state actually built and
  // written so far (via the Review flows above); the fuller Shala_<State>
  // PB Compare State (Active/Fuzzy/Golden/Deprecated/Aged/Rejected/
  // PartiallyAccepted/Dead) is still a DESIGNED-but-not-yet-built next
  // step (see the Roadmap/Manual/Glossary), so it never appears here.
  const gate = attrs['meta.gate'];
  const domain = attrs['meta.domain'];
  const classification = gate ? `${gate}${domain ? ' / ' + domain : ''}` : 'Unclassified';

  // FIXED, real gap: `attrs['body.paragraph']` was always just
  // pb_catalog's own `[playbook-catalog] source_path=...` bookkeeping
  // line (an EAV scalar, last-write-wins collapses every real paragraph
  // down to whichever was journaled last) -- never the playbook's own
  // stated reason for existing, even though that text was genuinely
  // captured. `data.why` is the real fix (server-side: matches the
  // child sections that survive un-collapsed, by their real link.target
  // bytes) -- shown here as its own real "why does this exist" block,
  // and `body.paragraph` is dropped from the raw attrs dump below since
  // it was never anything but that same noise.
  const whyHtml = data.why
    ? `<p class="story-chronicle-heading">Why this exists</p><p class="story-detail" style="white-space:pre-wrap">${esc(data.why)}</p>`
    : '';

  // An explicit, honest statement either way -- "empty" alone doesn't
  // tell an Architect whether this playbook is genuinely independent or
  // simply hasn't been reviewed for dependencies yet.
  const dependencyNote = (dependsOn.length === 0 && dependedOnBy.length === 0)
    ? '<p class="dim">No dependency edges recorded for this playbook -- either genuinely independent, or not yet reviewed via Review Dependencies.</p>'
    : '';

  const mutualTitles = new Set(dependsOn.filter(t => dependedOnBy.includes(t)));

  const rows = Object.entries(attrs)
    .filter(([k]) => k !== 'body.paragraph')
    .map(([k, v]) => `<div class="story-event"><div class="story-type">${esc(k)}</div><div class="story-detail">${esc(String(v))}</div></div>`)
    .join('');
  body.innerHTML = `
    <button class="btn" id="btn-story-back">◀ back</button>
    <p class="story-kaki dim">KAKI: ${esc(data.kaki_hex || '')}</p>
    <p class="dim">Classification: <strong>${esc(classification)}</strong></p>
    ${whyHtml}
    ${chronicleHtml(data.story)}
    ${dependencyNote}
    ${dependencyListHtml(dependsOn, '◀ depends on (orange ring)', mutualTitles)}
    ${dependencyListHtml(dependedOnBy, 'depended on by ▶ (purple ring)', mutualTitles)}
    ${rows || '<p class="dim">no other attributes found on this entity.</p>'}
  `;
  document.getElementById('btn-story-back').addEventListener('click', () => { if (goBack) goBack(); });
  body.querySelectorAll('.gate-pb-clickable').forEach(li => {
    li.addEventListener('click', () => {
      returnToView = () => openStoryEngine(title);
      openStoryEngine(li.dataset.title);
    });
  });
}

// The "root as Tribe, orbits of related particles" view the Architect
// asked for: the current playbook as a core sphere, its real depends-on
// edges as an orange ring, its real depended-on-by edges as a purple
// ring -- click any cube (here or in the text lists above) to recenter
// on it and keep walking the chain outward.
function showPlaybookOrbit(dependsOn, dependedOnBy) {
  clearScene();
  sceneMode = 'PLAYBOOK_ORBIT';
  makeCore(new THREE.Vector3(0, 0, 0), 0x00ccff, 4);

  // A REAL "resonance" -- the Architect's uploaded Ṭāluku prototype flashes
  // a chord white when two orbiting particles' simulated phases happen to
  // align, but simulated motion here would just be the same meaningless
  // per-cube shivering already stripped out of this scene (see the fix a
  // few commits back). The grounded version of that idea: a title present
  // in BOTH rings means A depends on B AND B depends on A -- a real,
  // structural mutual/circular dependency, worth flagging in white whether
  // or not anything is moving.
  const mutualTitles = new Set(dependsOn.filter(t => dependedOnBy.includes(t)));

  const placeRing = (titles, minRadius, color) => {
    const sorted = [...titles].sort(natCompare);
    const radius = collisionFreeRadius(sorted.length, minRadius);
    sorted.forEach((t, i) => {
      const angle = (i / Math.max(sorted.length, 1)) * Math.PI * 2;
      const isMutual = mutualTitles.has(t);
      const geo = new THREE.BoxGeometry(2, 2, 2);
      const mat = new THREE.MeshStandardMaterial({
        color: isMutual ? 0xffffff : color,
        emissive: isMutual ? 0xffffff : color,
        emissiveIntensity: isMutual ? 0.9 : 0.4,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.set(Math.cos(angle) * radius, 0, Math.sin(angle) * radius);
      scene.add(mesh);
      cores.push({ mesh, basePos: mesh.position.clone(), phase: 0, static: true });
      clickTargets.push({ mesh, kind: 'cube', title: t });
    });
  };
  placeRing(dependsOn, 18, 0xff8844);
  placeRing(dependedOnBy, 30, 0x8844ff);
}

// ── Architect-only review/approve flow ──────────────────────────────────

async function openReview() {
  const overlay = document.getElementById('gates-review-overlay');
  const list = document.getElementById('gates-review-list');
  const status = document.getElementById('gates-review-status');
  overlay.classList.remove('hidden');
  status.textContent = '';
  list.innerHTML = '<p class="dim">scanning…</p>';

  const data = await fetchJson('/api/gates/scan_suggestions', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
  if (!data || data.error) { list.innerHTML = `<p class="dim">${esc(data && data.error || 'scan failed')}</p>`; return; }

  const suggestions = data.suggestions || [];
  if (suggestions.length === 0) {
    list.innerHTML = '<p class="dim">Nothing pending -- every catalogued playbook already has an approved gate.</p>';
    return;
  }

  list.innerHTML = suggestions.map((s, i) => {
    const options = ['<option value="">Unclassified</option>']
      .concat(GATE_ORDER.map(g => `<option value="${g}"${s.suggested_gates[0] === g ? ' selected' : ''}>${g}</option>`));
    return `
      <div class="gate-review-row" data-idx="${i}" data-kaki="${esc(s.pb_kaki_hex)}">
        <input type="checkbox" class="gr-check" ${s.suggested_gates.length ? 'checked' : ''}>
        <span class="gr-title" title="${esc(s.source_path)}">${esc(s.title)}</span>
        <select class="gr-select">${options.join('')}</select>
      </div>
    `;
  }).join('');
}

function closeReview() {
  document.getElementById('gates-review-overlay').classList.add('hidden');
}

async function approveReview() {
  const status = document.getElementById('gates-review-status');
  const rows = document.querySelectorAll('#gates-review-list .gate-review-row');
  const approvals = [];
  rows.forEach(row => {
    const checked = row.querySelector('.gr-check').checked;
    const gate = row.querySelector('.gr-select').value;
    if (checked && gate) {
      approvals.push({ pb_kaki_hex: row.dataset.kaki, gate_akkadian_name: gate });
    }
  });
  if (approvals.length === 0) {
    status.textContent = 'nothing checked with a gate chosen -- nothing to approve';
    return;
  }
  status.textContent = `approving ${approvals.length}…`;
  const result = await fetchJson('/api/gates/approve', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(approvals) });
  if (!result || result.error || result.ok === false) {
    status.textContent = `error: ${esc((result && result.error) || 'approve failed')}`;
    return;
  }
  status.textContent = `✓ tagged ${approvals.length} playbook(s)`;
  lastSummary = result.summary || lastSummary;
  renderButtons();
  renderSummaryLine();
  setTimeout(closeReview, 900);
}

// ── Architect-only dependency review/approve flow ───────────────────────
//
// Same suggest-then-approve shape as `openReview`/`approveReview` above,
// against `/api/dependencies/scan_suggestions` and `/api/dependencies/
// approve` -- no `<select>` needed here (unlike gate/domain review) since
// there's no classification ambiguity to resolve, just a real mention
// found in another playbook's own text that the Architect confirms or
// rejects one row at a time.

async function openDepReview() {
  const overlay = document.getElementById('gates-dep-review-overlay');
  const list = document.getElementById('gates-dep-review-list');
  const status = document.getElementById('gates-dep-review-status');
  overlay.classList.remove('hidden');
  status.textContent = '';
  list.innerHTML = '<p class="dim">scanning…</p>';

  const data = await fetchJson('/api/dependencies/scan_suggestions', {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}',
  });
  if (!data || data.error) { list.innerHTML = `<p class="dim">${esc((data && data.error) || 'scan failed')}</p>`; return; }

  const suggestions = data.suggestions || [];
  if (suggestions.length === 0) {
    list.innerHTML = '<p class="dim">Nothing pending -- no new whole-word mentions of one catalogued playbook inside another\'s text.</p>';
    return;
  }

  list.innerHTML = suggestions.map((s, i) => `
    <div class="dep-review-row" data-idx="${i}"
         data-source-kaki="${esc(s.source_kaki_hex)}" data-source-title="${esc(s.source_title)}"
         data-target-kaki="${esc(s.target_kaki_hex)}" data-target-title="${esc(s.target_title)}">
      <input type="checkbox" class="dr-check" checked>
      <span class="dr-pair">
        <span class="dr-source">${esc(s.source_title)}</span> --depends-on--&gt; <span class="dr-target">${esc(s.target_title)}</span>
      </span>
    </div>
  `).join('');
}

function closeDepReview() {
  document.getElementById('gates-dep-review-overlay').classList.add('hidden');
}

async function approveDepReview() {
  const status = document.getElementById('gates-dep-review-status');
  const rows = document.querySelectorAll('#gates-dep-review-list .dep-review-row');
  const approvals = [];
  rows.forEach(row => {
    if (row.querySelector('.dr-check').checked) {
      approvals.push({
        source_kaki_hex: row.dataset.sourceKaki, source_title: row.dataset.sourceTitle,
        target_kaki_hex: row.dataset.targetKaki, target_title: row.dataset.targetTitle,
      });
    }
  });
  if (approvals.length === 0) {
    status.textContent = 'nothing checked -- nothing to approve';
    return;
  }
  status.textContent = `approving ${approvals.length}…`;
  const result = await fetchJson('/api/dependencies/approve', {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(approvals),
  });
  if (!result || result.error || result.ok === false) {
    status.textContent = `error: ${esc((result && result.error) || 'approve failed')}`;
    return;
  }
  status.textContent = `✓ minted ${approvals.length} depends-on edge(s)`;
  setTimeout(closeDepReview, 900);
}

// ── Manual publish/refresh ───────────────────────────────────────────────
//
// Rebuilds the FULL EnkiDDB generation Gate Orbits reads from (every
// catalogued playbook plus every already-approved gate/domain tag and
// depends-on edge) and republishes it as `current` -- the same rebuild
// every approve action already runs before adding its own new tag/edge,
// exposed here with zero new approvals so an Architect can fix a "not
// published yet" state, or re-sync after a fresh catalog scan added new
// locations, without needing to approve anything first.
async function refreshKnowledgeGraph() {
  const btn = document.getElementById('btn-gates-refresh');
  const original = btn.textContent;
  btn.disabled = true;
  btn.textContent = 'Refreshing…';
  const result = await fetchJson('/api/refresh_current', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
  btn.disabled = false;
  btn.textContent = original;
  if (!result || result.error || result.ok === false) {
    alert(`Refresh failed: ${(result && result.error) || 'unknown error'}`);
    return;
  }
  await refreshSummary();
  if (selectedGate) selectGate(selectedGate);
}

// ── Wiring, run once ─────────────────────────────────────────────────────

let wired = false;
function wireControls() {
  if (wired) return;
  wired = true;
  document.getElementById('btn-gates-review').addEventListener('click', openReview);
  document.getElementById('btn-gates-review-close').addEventListener('click', closeReview);
  document.getElementById('btn-gates-review-approve').addEventListener('click', approveReview);

  document.getElementById('btn-gates-dep-review').addEventListener('click', openDepReview);
  document.getElementById('btn-gates-dep-review-close').addEventListener('click', closeDepReview);
  document.getElementById('btn-gates-dep-review-approve').addEventListener('click', approveDepReview);
  document.getElementById('btn-gates-refresh').addEventListener('click', refreshKnowledgeGraph);

  const input = document.getElementById('gates-search-input');
  document.getElementById('btn-gates-search').addEventListener('click', () => runSearch(input.value));
  input.addEventListener('keydown', e => { if (e.key === 'Enter') runSearch(input.value); });
}

const MATCH_KIND_LABEL = { title: 'title', path: 'file path', content: 'body text' };
let lastSearchQuery = null;

// ── Search: the answer to "I don't know which of the 7 gates this is
// in yet" -- reads straight from pb_catalog's own registry server-side
// (HeptaScript's WHERE clause has no substring operator, only exact
// equality), so this works even before anything has been gate/domain-
// classified at all. Scans each playbook's title, source path, AND full
// body text -- FIXED, live: a search that only checked title/path missed
// real hits where the term only appears in a playbook's own WHY/task
// prose (e.g. "Ezida" showing up in a playbook's body but never in its
// filename or header title line). `match_kind` tells the Architect WHERE
// it matched, since a body-text hit isn't self-evident the way a title
// hit is.
async function runSearch(rawQuery) {
  const q = (rawQuery || '').trim();
  const title = document.getElementById('gates-detail-title');
  const body = document.getElementById('gates-detail-body');
  if (!q) { body.innerHTML = '<p class="dim">Type something to search for first.</p>'; return; }

  title.textContent = `Search: "${q}"`;
  body.innerHTML = '<p class="dim">searching…</p>';
  lastSearchQuery = q;

  const data = await fetchJson(`/api/playbooks/search?q=${encodeURIComponent(q)}`);
  if (!data || data.error) { body.innerHTML = `<p class="dim">${esc((data && data.error) || 'search failed')}</p>`; return; }

  const results = data.results || [];
  if (results.length === 0) {
    body.innerHTML = `<p class="dim">No matches among ${data.total_scanned} catalogued playbook(s) -- checked title, file path, and full body text.</p>`;
    clearScene();
    sceneMode = 'ENLIL';
    return;
  }

  showSearchCubes(results);

  const rows = results.map(r => {
    const tag = r.gate ? `${esc(r.gate)}${r.domain ? ' / ' + esc(r.domain) : ''}` : 'Unclassified';
    const kind = MATCH_KIND_LABEL[r.match_kind] || r.match_kind;
    return `
      <li class="gate-pb-clickable" data-title="${esc(r.title)}">
        ${esc(r.title)} <span class="dim">— ${tag} · matched in ${esc(kind)}</span>
      </li>
    `;
  }).join('');

  body.innerHTML = `
    <p class="dim">${results.length} match(es) of ${data.total_scanned} catalogued playbook(s)${results.length >= 50 ? ' (showing the first 50)' : ''} -- also plotted on the left as cubes, click either.</p>
    <ul class="gate-pb-list">${rows}</ul>
  `;
  body.querySelectorAll('.gate-pb-clickable').forEach(li => {
    li.addEventListener('click', () => {
      returnToView = () => runSearch(q);
      openStoryEngine(li.dataset.title);
    });
  });
}

// The left "Knowledge Graph" panel used to stay on whatever scene was
// showing before a search ran (an orbit sphere or a domain's cube grid),
// so search results only ever appeared in the right-side text list --
// FIXED, live: results now render as their own cube grid here too, same
// visual language `showDomainCubes` already established, so search
// integrates with the graph instead of bypassing it.
function showSearchCubes(results) {
  clearScene();
  sceneMode = 'SEARCH_CUBES';
  makeCore(new THREE.Vector3(0, 0, 0), 0x00ccff, 3);

  const shown = results.slice().sort((a, b) => natCompare(a.title, b.title)).slice(0, 60);
  const cols = 6, spacing = 3.4;
  const cubeGeo = new THREE.BoxGeometry(2.2, 2.2, 2.2);
  shown.forEach((r, i) => {
    const col = i % cols, row = Math.floor(i / cols);
    const color = r.gate ? 0x00cc66 : 0xff8844; // green = classified, orange = unclassified
    const mat = new THREE.MeshStandardMaterial({ color, emissive: color, emissiveIntensity: 0.4 });
    const mesh = new THREE.Mesh(cubeGeo, mat);
    mesh.position.set((col - (cols - 1) / 2) * spacing, row * spacing - 4, 12);
    scene.add(mesh);
    cores.push({ mesh, basePos: mesh.position.clone(), phase: 0, static: true });
    clickTargets.push({ mesh, kind: 'cube', title: r.title, kakiHex: r.kaki_hex });
  });
}

// Called by app.js's switchTab() whenever Shala4 becomes visible.
async function onShown(isArchitect) {
  architectView = isArchitect;
  wireControls();
  ensureScene();
  onResize();
  await refreshSummary();
}

window.gateOrbits = { onShown };
