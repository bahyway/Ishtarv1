//! SC+GA PROBE — Simplicial Complex & Geometric Algebra demo.
//! The "Graveyard Rat Detection" example: rats as Particles, graveyard sectors as Orbits.
//! Pure Rust, no web_sys.  All geometry in normalized [-1,1] coordinates.

// ── Rat particle data ────────────────────────────────────────────────────────
// (name, sector, angle_deg, status, rgb)
pub type RatRec = (&'static str, usize, f32, &'static str, [u8; 3]);

pub const SECTOR_RADII: [f32; 4] = [0.80, 0.52, 0.27, 0.08];
pub const SECTOR_NAMES: [&str; 4] = [
    "Outer Perimeter",
    "Mid Yard",
    "Inner Crypt",
    "Sewer Nucleus",
];

pub const RATS: &[RatRec] = &[
    // ── Outer Perimeter (sector 0, r=0.80) — lone scouts, 8×45° ─────────────
    ("Greywhisker", 0, 0.0, "Watch", [140, 140, 145]),
    ("Tombrunner", 0, 45.0, "Active", [150, 148, 140]),
    ("Shadowpaw", 0, 90.0, "Active", [135, 135, 142]),
    ("Nightcreep", 0, 135.0, "Watch", [143, 140, 133]),
    ("Dustscurry", 0, 180.0, "Active", [148, 145, 138]),
    ("Boneprowl", 0, 225.0, "Watch", [138, 140, 148]),
    ("Cemetail", 0, 270.0, "Active", [145, 143, 135]),
    ("Stoneback", 0, 315.0, "Watch", [140, 138, 143]),
    // ── Mid Yard (sector 1, r=0.52) — gang runners, 6×60° + 30° offset ──────
    ("Bonedancer", 1, 30.0, "Warning", [210, 162, 70]),
    ("Cryptclaw", 1, 90.0, "Active", [200, 155, 75]),
    ("Dustfur", 1, 150.0, "Watch", [205, 165, 80]),
    ("Gravemind", 1, 210.0, "Warning", [215, 158, 65]),
    ("Rotsnout", 1, 270.0, "Active", [195, 162, 72]),
    ("Moldstalker", 1, 330.0, "Watch", [208, 160, 68]),
    // ── Inner Crypt (sector 2, r=0.27) — nest guardians, 4×90° ──────────────
    ("Nestking", 2, 0.0, "Critical", [220, 70, 70]),
    ("Broodmother", 2, 90.0, "Critical", [215, 65, 65]),
    ("Charnelpaw", 2, 180.0, "Warning", [200, 100, 80]),
    ("Sootwhisker", 2, 270.0, "Warning", [210, 80, 70]),
    // ── Sewer Nucleus (sector 3, r=0.08) — the boss ──────────────────────────
    ("RatLord", 3, 90.0, "Dead", [90, 25, 25]),
];

pub fn rat_pos(rec: &RatRec) -> (f32, f32) {
    let r = SECTOR_RADII[rec.1];
    let a = rec.2.to_radians();
    (r * a.cos(), r * a.sin())
}

// ── Union-Find (for β₀) ──────────────────────────────────────────────────────
struct Uf {
    p: Vec<usize>,
}
impl Uf {
    fn new(n: usize) -> Self {
        Uf {
            p: (0..n).collect(),
        }
    }
    fn find(&mut self, x: usize) -> usize {
        if self.p[x] != x {
            self.p[x] = self.find(self.p[x]);
        }
        self.p[x]
    }
    fn union(&mut self, x: usize, y: usize) {
        let (rx, ry) = (self.find(x), self.find(y));
        if rx != ry {
            self.p[rx] = ry;
        }
    }
    fn components(&mut self, n: usize) -> usize {
        (0..n).filter(|&i| self.find(i) == i).count()
    }
}

// ── Simplicial complex result ────────────────────────────────────────────────
#[derive(Clone, Default)]
pub struct SimplicialResult {
    pub edges: Vec<(usize, usize)>,
    pub triangles: Vec<(usize, usize, usize)>,
    pub beta0: usize, // connected components
    pub beta1: usize, // independent loops  = E - V + β₀ - T
    pub epsilon: f32,
}

pub fn build_sc(epsilon: f32) -> SimplicialResult {
    let n = RATS.len();
    let pos: Vec<(f32, f32)> = RATS.iter().map(rat_pos).collect();

    let d = |i: usize, j: usize| -> f32 {
        let dx = pos[i].0 - pos[j].0;
        let dy = pos[i].1 - pos[j].1;
        (dx * dx + dy * dy).sqrt()
    };

    // 1-simplices
    let mut edges = Vec::<(usize, usize)>::new();
    let mut uf = Uf::new(n);
    for i in 0..n {
        for j in (i + 1)..n {
            if d(i, j) < epsilon {
                edges.push((i, j));
                uf.union(i, j);
            }
        }
    }

    // 2-simplices (all triangles where every edge is in the complex)
    let edge_set: std::collections::HashSet<(usize, usize)> = edges.iter().copied().collect();
    let has_edge = |a: usize, b: usize| {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        edge_set.contains(&(lo, hi))
    };
    let mut triangles = Vec::<(usize, usize, usize)>::new();
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                if has_edge(i, j) && has_edge(i, k) && has_edge(j, k) {
                    triangles.push((i, j, k));
                }
            }
        }
    }

    let beta0 = uf.components(n);
    // Euler char: χ = V - E + T  →  β₁ = E - V + β₀ - T
    let v = n;
    let e = edges.len();
    let t = triangles.len();
    let beta1 = (e + beta0).saturating_sub(v + t);

    SimplicialResult {
        edges,
        triangles,
        beta0,
        beta1,
        epsilon,
    }
}

// ── Geometric Algebra path (Cl(2,0) rotors) ──────────────────────────────────
// In Cl(2,0), a rotor R = cos(θ/2) + sin(θ/2)·e₁₂ rotates a 2D vector by θ.
// We represent the path as a sequence of (current_angle, current_r) waypoints.

#[derive(Clone)]
pub struct GaStep {
    pub rotor_deg: f32, // rotation in degrees (GA rotor angle θ)
    pub radius: f32,    // radial distance after this step
    pub note: &'static str,
}

#[derive(Clone)]
pub struct GaResult {
    pub steps: Vec<GaStep>,
    pub path_points: Vec<(f32, f32)>, // normalized coords of each waypoint
}

pub fn compute_ga_path() -> GaResult {
    // Strategy: approach from the east gate, spiral inward
    // using 3 rotors to slip between rat patrols.
    let steps = vec![
        GaStep {
            rotor_deg: 0.0,
            radius: 0.93,
            note: "Gate — bearing 0°  [start]",
        },
        GaStep {
            rotor_deg: 30.0,
            radius: 0.88,
            note: "R₁: e^(+15°·e₁₂) — slip past Greywhisker",
        },
        GaStep {
            rotor_deg: 0.0,
            radius: 0.68,
            note: "Translate Δr=−0.20 — approach Mid Yard ring",
        },
        GaStep {
            rotor_deg: -40.0,
            radius: 0.60,
            note: "R₂: e^(−20°·e₁₂) — thread Bonedancer gap",
        },
        GaStep {
            rotor_deg: 0.0,
            radius: 0.38,
            note: "Translate Δr=−0.22 — descend to Inner Crypt",
        },
        GaStep {
            rotor_deg: 55.0,
            radius: 0.30,
            note: "R₃: e^(+27.5°·e₁₂) — clear Nestking arc",
        },
        GaStep {
            rotor_deg: 0.0,
            radius: 0.11,
            note: "Translate Δr=−0.19 — Sewer Nucleus reached",
        },
        GaStep {
            rotor_deg: -15.0,
            radius: 0.08,
            note: "R₄: align — RatLord [Dead] confirmed ∠90°",
        },
    ];

    // Build path_points from cumulative angle + radius
    let mut pts = Vec::<(f32, f32)>::new();
    let mut angle: f32 = 0.0; // start pointing east
    for step in &steps {
        angle += step.rotor_deg;
        let rad = angle.to_radians();
        pts.push((step.radius * rad.cos(), step.radius * rad.sin()));
    }

    GaResult {
        steps,
        path_points: pts,
    }
}

// ── Probe state ──────────────────────────────────────────────────────────────
pub struct ScGaState {
    pub epsilon: f32,
    pub sc: Option<SimplicialResult>,
    pub ga: Option<GaResult>,
    pub ran: bool,
}

impl ScGaState {
    pub fn new() -> Self {
        ScGaState {
            epsilon: 0.65,
            sc: None,
            ga: None,
            ran: false,
        }
    }
}

impl Default for ScGaState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScGaState {
    pub fn run(&mut self) {
        self.sc = Some(build_sc(self.epsilon));
        self.ga = Some(compute_ga_path());
        self.ran = true;
    }

    pub fn adjust_epsilon(&mut self, delta: f32) {
        self.epsilon = (self.epsilon + delta).clamp(0.30, 0.95);
        if self.ran {
            self.run();
        } // recompute if already probed
    }
}
