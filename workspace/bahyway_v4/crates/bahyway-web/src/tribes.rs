//! Sovereign Tribes — pure Rust, no web_sys.
//! All rendering is done in lib.rs.

use std::f32::consts::{PI, TAU};

pub struct TribeDef {
    pub id: u16,
    pub name: &'static str,
    pub tag: &'static str,
    pub rgb: [u8; 3],
    pub map_x: f32,
    pub map_y: f32,
    pub orbits: &'static [OrbitDef],
}

pub struct OrbitDef {
    pub name: &'static str,
    pub radius: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PType {
    Zikru,
    Musaru,
    Kali,
    Alu,
    Eresh,
}

impl PType {
    pub fn label(self) -> &'static str {
        match self {
            PType::Zikru => "Zikru",
            PType::Musaru => "Musarû",
            PType::Kali => "Kali",
            PType::Alu => "Alu",
            PType::Eresh => "Eresh",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            PType::Zikru => "Identity anchor",
            PType::Musaru => "Security observer",
            PType::Kali => "Archive node",
            PType::Alu => "Drift monitor",
            PType::Eresh => "Alert emitter",
        }
    }

    pub fn dot_radius(self) -> f64 {
        match self {
            PType::Zikru => 5.0,
            PType::Musaru => 4.0,
            PType::Kali => 3.5,
            PType::Alu => 5.0,
            PType::Eresh => 6.5,
        }
    }

    pub fn css_color(self) -> &'static str {
        match self {
            PType::Zikru => "var(--tok-fld)",
            PType::Musaru => "var(--accent)",
            PType::Kali => "var(--text-muted)",
            PType::Alu => "var(--orange)",
            PType::Eresh => "var(--red)",
        }
    }
}

pub struct RParticle {
    pub name: &'static str,
    pub tribe_idx: usize,
    pub orbit_idx: usize,
    pub ptype: PType,
    pub angle0: f32,
    pub rgb: [u8; 3],
    pub status: &'static str,
    pub drift: f32,
    pub epoch: u32,
    pub tags: &'static [&'static str],
}

pub const MAX_ORBIT_PX: f64 = 88.0;

// ── Tribe definitions ────────────────────────────────────────────────────────

static AL_RASHID_ORBITS: &[OrbitDef] = &[
    OrbitDef {
        name: "Core",
        radius: 0.28,
        speed: 0.022,
    },
    OrbitDef {
        name: "Verified",
        radius: 0.57,
        speed: 0.014,
    },
    OrbitDef {
        name: "Extended",
        radius: 0.88,
        speed: 0.009,
    },
];

static ZENITH_ORBITS: &[OrbitDef] = &[
    OrbitDef {
        name: "Nucleus",
        radius: 0.36,
        speed: 0.031,
    },
    OrbitDef {
        name: "Rim",
        radius: 0.82,
        speed: 0.016,
    },
];

static SEREN_ORBITS: &[OrbitDef] = &[
    OrbitDef {
        name: "Inner",
        radius: 0.38,
        speed: 0.018,
    },
    OrbitDef {
        name: "Outer",
        radius: 0.78,
        speed: 0.010,
    },
];

static DUSK_ORBITS: &[OrbitDef] = &[
    OrbitDef {
        name: "Anchor",
        radius: 0.27,
        speed: 0.040,
    },
    OrbitDef {
        name: "Survey",
        radius: 0.55,
        speed: 0.021,
    },
    OrbitDef {
        name: "Horizon",
        radius: 0.84,
        speed: 0.011,
    },
];

pub const TRIBES: &[TribeDef] = &[
    TribeDef {
        id: 0x0001,
        name: "Al-Rashid",
        tag: "Sovereign Identity Registry",
        rgb: [77, 166, 255],
        map_x: 0.208,
        map_y: 0.308,
        orbits: AL_RASHID_ORBITS,
    },
    TribeDef {
        id: 0x0002,
        name: "Zenith",
        tag: "Critical Monitoring Node",
        rgb: [255, 82, 82],
        map_x: 0.792,
        map_y: 0.308,
        orbits: ZENITH_ORBITS,
    },
    TribeDef {
        id: 0x0003,
        name: "Seren",
        tag: "Long-Epoch Archival Tribe",
        rgb: [105, 240, 174],
        map_x: 0.208,
        map_y: 0.769,
        orbits: SEREN_ORBITS,
    },
    TribeDef {
        id: 0x0004,
        name: "Dusk",
        tag: "Cross-Boundary Drift Monitor",
        rgb: [206, 147, 216],
        map_x: 0.792,
        map_y: 0.769,
        orbits: DUSK_ORBITS,
    },
];

// ── Particle corpus ──────────────────────────────────────────────────────────

pub const PARTICLES: &[RParticle] = &[
    // Al-Rashid (5 particles)
    RParticle {
        name: "Ali_Karim",
        tribe_idx: 0,
        orbit_idx: 0,
        ptype: PType::Zikru,
        angle0: 0.00,
        rgb: [80, 200, 255],
        status: "Active",
        drift: 0.02,
        epoch: 1,
        tags: &["identity", "verified"],
    },
    RParticle {
        name: "Fatima_Hassan",
        tribe_idx: 0,
        orbit_idx: 0,
        ptype: PType::Zikru,
        angle0: PI,
        rgb: [120, 200, 100],
        status: "Active",
        drift: 0.01,
        epoch: 2,
        tags: &["identity", "verified"],
    },
    RParticle {
        name: "Nour_Ibrahim",
        tribe_idx: 0,
        orbit_idx: 1,
        ptype: PType::Musaru,
        angle0: 1.57,
        rgb: [180, 180, 50],
        status: "Fuzzy",
        drift: 0.18,
        epoch: 3,
        tags: &["diagnosis", "monitoring", "fuzzy"],
    },
    RParticle {
        name: "Khalid_Nasser",
        tribe_idx: 0,
        orbit_idx: 2,
        ptype: PType::Kali,
        angle0: 4.71,
        rgb: [40, 40, 200],
        status: "Archive",
        drift: 0.00,
        epoch: 1,
        tags: &["cold", "sealed"],
    },
    RParticle {
        name: "Ahmed_Fadel",
        tribe_idx: 0,
        orbit_idx: 1,
        ptype: PType::Alu,
        angle0: 0.78,
        rgb: [255, 140, 0],
        status: "Fuzzy",
        drift: 0.62,
        epoch: 4,
        tags: &["diagnosis", "drift", "fuzzy"],
    },
    // Zenith (4 particles)
    RParticle {
        name: "Omar_Said",
        tribe_idx: 1,
        orbit_idx: 0,
        ptype: PType::Eresh,
        angle0: 0.00,
        rgb: [200, 80, 80],
        status: "Dead",
        drift: 0.88,
        epoch: 1,
        tags: &["alert", "drift", "dead", "steward-review"],
    },
    RParticle {
        name: "Zahra_Ali",
        tribe_idx: 1,
        orbit_idx: 1,
        ptype: PType::Zikru,
        angle0: 2.09,
        rgb: [80, 200, 255],
        status: "Active",
        drift: 0.05,
        epoch: 2,
        tags: &["identity", "cleared"],
    },
    RParticle {
        name: "Hassan_Rauf",
        tribe_idx: 1,
        orbit_idx: 0,
        ptype: PType::Musaru,
        angle0: 4.18,
        rgb: [255, 100, 50],
        status: "Warning",
        drift: 0.44,
        epoch: 1,
        tags: &["monitoring", "anomaly"],
    },
    RParticle {
        name: "Layla_Mohsen",
        tribe_idx: 1,
        orbit_idx: 1,
        ptype: PType::Kali,
        angle0: 1.05,
        rgb: [150, 50, 200],
        status: "Archive",
        drift: 0.00,
        epoch: 3,
        tags: &["cold", "historical"],
    },
    // Seren (3 particles)
    RParticle {
        name: "Sara_Younis",
        tribe_idx: 2,
        orbit_idx: 0,
        ptype: PType::Zikru,
        angle0: 1.57,
        rgb: [200, 200, 200],
        status: "Pending",
        drift: 0.08,
        epoch: 1,
        tags: &["staging", "new-entry"],
    },
    RParticle {
        name: "Yousif_Reza",
        tribe_idx: 2,
        orbit_idx: 1,
        ptype: PType::Musaru,
        angle0: 4.71,
        rgb: [50, 220, 120],
        status: "Active",
        drift: 0.03,
        epoch: 2,
        tags: &["archival", "long-epoch"],
    },
    RParticle {
        name: "Mona_Taher",
        tribe_idx: 2,
        orbit_idx: 0,
        ptype: PType::Alu,
        angle0: PI,
        rgb: [180, 220, 60],
        status: "Watch",
        drift: 0.36,
        epoch: 1,
        tags: &["drift", "watch-list"],
    },
    // Dusk (4 particles)
    RParticle {
        name: "Rania_Said",
        tribe_idx: 3,
        orbit_idx: 0,
        ptype: PType::Zikru,
        angle0: 0.00,
        rgb: [150, 50, 255],
        status: "Active",
        drift: 0.07,
        epoch: 1,
        tags: &["cross-boundary", "identity"],
    },
    RParticle {
        name: "Tariq_Abu_Zeid",
        tribe_idx: 3,
        orbit_idx: 1,
        ptype: PType::Musaru,
        angle0: 2.09,
        rgb: [200, 100, 255],
        status: "Active",
        drift: 0.21,
        epoch: 2,
        tags: &["observer", "boundary-scan"],
    },
    RParticle {
        name: "Dalia_Faraj",
        tribe_idx: 3,
        orbit_idx: 2,
        ptype: PType::Kali,
        angle0: 4.18,
        rgb: [100, 0, 200],
        status: "Archive",
        drift: 0.00,
        epoch: 3,
        tags: &["cold", "cross-ref"],
    },
    RParticle {
        name: "Sami_Ko",
        tribe_idx: 3,
        orbit_idx: 0,
        ptype: PType::Eresh,
        angle0: PI,
        rgb: [255, 50, 150],
        status: "Dead",
        drift: 0.75,
        epoch: 1,
        tags: &["alert", "drift", "dead", "steward-review"],
    },
];

// ── Runtime state ────────────────────────────────────────────────────────────

pub struct TribesState {
    pub angles: Vec<f32>,
    pub selected: Option<usize>,
    pub compare_a: usize,
    pub compare_b: usize,
    pub view: TribesView,
}

#[derive(PartialEq, Clone, Copy)]
pub enum TribesView {
    Map,
    Compare,
}

impl TribesState {
    pub fn new() -> Self {
        TribesState {
            angles: PARTICLES.iter().map(|p| p.angle0).collect(),
            selected: None,
            compare_a: 0,
            compare_b: 1,
            view: TribesView::Map,
        }
    }
}

impl Default for TribesState {
    fn default() -> Self {
        Self::new()
    }
}

impl TribesState {
    pub fn animate(&mut self) {
        for (i, p) in PARTICLES.iter().enumerate() {
            let speed = TRIBES[p.tribe_idx].orbits[p.orbit_idx].speed;
            self.angles[i] = (self.angles[i] + speed) % TAU;
        }
    }
}

// ── Geometry helpers ─────────────────────────────────────────────────────────

pub fn particle_pos(p: &RParticle, angle: f32, cx: f64, cy: f64) -> (f64, f64) {
    let r = TRIBES[p.tribe_idx].orbits[p.orbit_idx].radius as f64 * MAX_ORBIT_PX;
    (cx + r * (angle as f64).cos(), cy + r * (angle as f64).sin())
}

pub fn status_css_color(s: &str) -> &'static str {
    match s {
        "Active" => "var(--s-active)",
        "Critical" => "var(--s-critical)",
        "Warning" => "var(--s-warning)",
        "Watch" => "var(--s-watch)",
        "Pending" => "var(--s-pending)",
        "Archive" => "var(--s-archive)",
        "Dead" => "var(--s-dead)",
        "Fuzzy" => "var(--s-fuzzy)",
        _ => "var(--text-muted)",
    }
}

// Hex colors for canvas drawing (CSS vars don't work in canvas fillStyle)
pub fn status_hex_color(s: &str) -> &'static str {
    match s {
        "Active" => "#50fa7b",
        "Critical" => "#ff5555",
        "Warning" => "#ffb86c",
        "Watch" => "#f1fa8c",
        "Pending" => "#b0b0c0",
        "Archive" => "#6c7086",
        "Dead" => "#7a3a3a",
        "Fuzzy" => "#9b7a3e",
        _ => "#6c7086",
    }
}
