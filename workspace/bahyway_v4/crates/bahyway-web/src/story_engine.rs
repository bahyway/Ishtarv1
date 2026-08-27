//! StoryEngine — per-particle event journal and Houdini-style node graph.
//! Pure Rust, no web_sys. All rendering is done in lib.rs.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EventKind {
    Ingestion,
    Validation,
    Promotion,
    EpochAdvance,
    DriftAlert,
    StatusChange,
    Quarantine,
    Archive,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            EventKind::Ingestion => "Ingestion",
            EventKind::Validation => "Validation",
            EventKind::Promotion => "Promotion",
            EventKind::EpochAdvance => "Epoch+",
            EventKind::DriftAlert => "Drift Alert",
            EventKind::StatusChange => "Status",
            EventKind::Quarantine => "Quarantine",
            EventKind::Archive => "Archive",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            EventKind::Ingestion => "↓",
            EventKind::Validation => "✓",
            EventKind::Promotion => "▲",
            EventKind::EpochAdvance => "⊕",
            EventKind::DriftAlert => "⚡",
            EventKind::StatusChange => "↻",
            EventKind::Quarantine => "⛔",
            EventKind::Archive => "☁",
        }
    }

    // Hex color for canvas drawing (not a CSS var — must be a literal for canvas fillStyle)
    pub fn hex_color(self) -> &'static str {
        match self {
            EventKind::Ingestion => "#8be9fd",
            EventKind::Validation => "#50fa7b",
            EventKind::Promotion => "#bd93f9",
            EventKind::EpochAdvance => "#f1fa8c",
            EventKind::DriftAlert => "#ffb86c",
            EventKind::StatusChange => "#cdd6f4",
            EventKind::Quarantine => "#ff5555",
            EventKind::Archive => "#6c7086",
        }
    }
}

pub struct StoryEvent {
    pub kind: EventKind,
    pub tick: u32,
    pub detail: &'static str,
    pub state: &'static str, // particle state after this event
}

// One story per particle — same index order as PARTICLES in tribes.rs (0..15)
pub const STORIES: &[&[StoryEvent]] = &[
    // 0: Ali_Karim — Zikru, Active, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,  tick: 1, detail: "Particle arrived in SDB staging buffer from tribe 0x0001", state: "Pending" },
        StoryEvent { kind: EventKind::Validation, tick: 2, detail: "KakiCore verified Zikru identity anchor — kaki hash matches registry", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,  tick: 5, detail: "ValidationSweep promoted particle SDB → ODB, epoch 1 sealed", state: "Active" },
    ],
    // 1: Fatima_Hassan — Zikru, Active, epoch=2
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB staging buffer", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 3,  detail: "KakiCore verified Zikru identity anchor — dual-key verification passed", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 12, detail: "Scheduled sweep advanced epoch 1 → 2; no drift detected (0.01)", state: "Active" },
    ],
    // 2: Nour_Ibrahim — Musaru, Fuzzy, drift=0.18, epoch=3
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — Musarû observer designation assigned at intake", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Musarû security-observer role", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 10, detail: "Epoch 1 → 2; drift index 0.04 (within threshold)", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 20, detail: "Epoch 2 → 3; drift index 0.12 (approaching threshold 0.15)", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 24, detail: "Drift index 0.18 exceeded watch threshold (0.15) — Musarû sweep triggered", state: "Watch" },
        StoryEvent { kind: EventKind::StatusChange, tick: 24, detail: "Status updated Active → Watch by automated Musarû sweep protocol", state: "Watch" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 28, detail: "Repeat drift measurement: 0.18 ± 0.09 — conflicting sensor readings, second sweep inconclusive", state: "Fuzzy" },
        StoryEvent { kind: EventKind::StatusChange, tick: 28, detail: "Watch evaluation inconclusive — drift measurements contradictory — state designated Fuzzy; awaiting steward re-validation", state: "Fuzzy" },
    ],
    // 3: Khalid_Nasser — Kali, Archive, drift=0.00, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,  tick: 1, detail: "Particle arrived in SDB — Kali cold-storage designation", state: "Pending" },
        StoryEvent { kind: EventKind::Validation, tick: 2, detail: "KakiCore verified Kali archive designation — zero-drift sealed", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,  tick: 5, detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::Archive,    tick: 8, detail: "Kali seal applied — particle transferred to cold storage (read-only, immutable)", state: "Archive" },
    ],
    // 4: Ahmed_Fadel — Alu, Fuzzy, drift=0.62, epoch=4
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — Alu drift-monitor role assigned", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 3,  detail: "KakiCore verified Alu drift-monitor identity", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 10, detail: "Epoch 1 → 2; drift 0.08 — nominal", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 20, detail: "Epoch 2 → 3; drift 0.31 — rising trend detected", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 30, detail: "Epoch 3 → 4; drift 0.55 — warning band approaching", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 35, detail: "Drift 0.62 crossed warning band — Alu emitter triggered protocol", state: "Warning" },
        StoryEvent { kind: EventKind::StatusChange, tick: 35, detail: "Status updated Active → Warning; drift continues rising", state: "Warning" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 40, detail: "Drift oscillating 0.58–0.67 — Warning/Critical threshold indeterminate; Alu sensor interference suspected", state: "Fuzzy" },
        StoryEvent { kind: EventKind::StatusChange, tick: 40, detail: "Status Warning → Fuzzy; trajectory unclear, re-validation required — steward correction needed to restore clear state", state: "Fuzzy" },
    ],
    // 5: Omar_Said — Eresh, Dead, drift=0.88, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — Eresh alert-emitter designation", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Eresh alert-emitter role — red-flag pre-registered", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 7,  detail: "First drift event: index 0.61 — Eresh emitter fires initial alert to Zenith coordinator", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 9,  detail: "Drift 0.88 — red-flag threshold (0.80) breached; Eresh escalation protocol engaged", state: "Critical" },
        StoryEvent { kind: EventKind::StatusChange, tick: 9,  detail: "Status escalated Active → Critical — Zenith tribe coordinator notified", state: "Critical" },
        StoryEvent { kind: EventKind::Quarantine,   tick: 11, detail: "Kaki identity anchor unresponsive after 3 consecutive missed validation cycles — particle isolated from ODB into QDB", state: "Dead" },
        StoryEvent { kind: EventKind::StatusChange, tick: 11, detail: "Status Critical → Dead; Zenith tribe data steward review required to restore identity anchor or re-classify particle", state: "Dead" },
    ],
    // 6: Zahra_Ali — Zikru, Active, drift=0.05, epoch=2
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB staging buffer", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Zikru identity — cleared flag applied", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 14, detail: "Epoch 1 → 2; drift 0.05 — well within threshold, cleared status confirmed", state: "Active" },
    ],
    // 7: Hassan_Rauf — Musaru, Warning, drift=0.44, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — anomaly flag noted at Zenith intake", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Musarû observer — anomaly tag preserved in kaki metadata", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 11, detail: "Drift 0.44 — anomaly pattern confirmed by Musarû sweep", state: "Warning" },
        StoryEvent { kind: EventKind::StatusChange, tick: 11, detail: "Status updated Active → Warning by Musarû sweep protocol", state: "Warning" },
    ],
    // 8: Layla_Mohsen — Kali, Archive, drift=0.00, epoch=3
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — historical record designation for Zenith tribe", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Kali archive designation", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 10, detail: "Epoch 1 → 2; zero drift maintained", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 22, detail: "Epoch 2 → 3; Kali archive sequence complete — seal imminent", state: "Active" },
        StoryEvent { kind: EventKind::Archive,      tick: 25, detail: "Kali seal applied — historical record sealed, cross-referenced in Dusk tribe index", state: "Archive" },
    ],
    // 9: Sara_Younis — Zikru, Pending, drift=0.08, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,  tick: 1, detail: "Particle arrived in SDB — new entry from Seren tribe intake queue", state: "Pending" },
        StoryEvent { kind: EventKind::Validation, tick: 3, detail: "KakiCore initiated Zikru validation — awaiting identity confirmation from external registry (in progress)", state: "Pending" },
    ],
    // 10: Yousif_Reza — Musaru, Active, drift=0.03, epoch=2
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — long-epoch archival role assigned by Seren tribe", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Musarû long-epoch observer role", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 16, detail: "Epoch 1 → 2; drift 0.03 — archival integrity confirmed", state: "Active" },
    ],
    // 11: Mona_Taher — Alu, Watch, drift=0.36, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — Alu drift monitor for Seren tribe assigned", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Alu drift-monitor identity", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 19, detail: "Drift 0.36 crossed watch-list threshold (0.30) — Alu sweep engaged", state: "Watch" },
        StoryEvent { kind: EventKind::StatusChange, tick: 19, detail: "Status updated Active → Watch; added to Seren watch-list", state: "Watch" },
    ],
    // 12: Rania_Said — Zikru, Active, drift=0.07, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,  tick: 1, detail: "Particle arrived in SDB — cross-boundary origin detected (Dusk tribe)", state: "Pending" },
        StoryEvent { kind: EventKind::Validation, tick: 2, detail: "KakiCore verified Zikru identity anchor — cross-boundary reference verified", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,  tick: 5, detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
    ],
    // 13: Tariq_Abu_Zeid — Musaru, Active, drift=0.21, epoch=2
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — boundary-scan observer role for Dusk tribe", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Musarû boundary-scan role", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 18, detail: "Epoch 1 → 2; drift 0.21 — elevated but within acceptable range for boundary scans", state: "Active" },
    ],
    // 14: Dalia_Faraj — Kali, Archive, drift=0.00, epoch=3
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — Kali cross-reference archive designation for Dusk tribe", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Kali cross-reference archive role", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 11, detail: "Epoch 1 → 2; zero drift maintained", state: "Active" },
        StoryEvent { kind: EventKind::EpochAdvance, tick: 23, detail: "Epoch 2 → 3; cross-reference index updated across Dusk boundary", state: "Active" },
        StoryEvent { kind: EventKind::Archive,      tick: 27, detail: "Kali cross-reference seal applied — cold storage, cross-boundary index finalized and locked", state: "Archive" },
    ],
    // 15: Sami_Ko — Eresh, Dead, drift=0.75, epoch=1
    &[
        StoryEvent { kind: EventKind::Ingestion,    tick: 1,  detail: "Particle arrived in SDB — Eresh boundary-breach emitter for Dusk tribe", state: "Pending" },
        StoryEvent { kind: EventKind::Validation,   tick: 2,  detail: "KakiCore verified Eresh boundary-breach emitter role", state: "Validated" },
        StoryEvent { kind: EventKind::Promotion,    tick: 5,  detail: "ValidationSweep promoted SDB → ODB", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 6,  detail: "Drift 0.52 — first boundary alert emitted from Dusk perimeter", state: "Active" },
        StoryEvent { kind: EventKind::DriftAlert,   tick: 8,  detail: "Drift 0.75 — boundary-breach confirmed; Eresh escalation protocol triggered", state: "Critical" },
        StoryEvent { kind: EventKind::StatusChange, tick: 8,  detail: "Status escalated Active → Critical — Dusk tribe boundary-breach protocol engaged", state: "Critical" },
        StoryEvent { kind: EventKind::Quarantine,   tick: 10, detail: "Boundary-breach signal lost; particle cannot resolve its orbit anchor — isolated into QDB quarantine zone", state: "Dead" },
        StoryEvent { kind: EventKind::StatusChange, tick: 10, detail: "Status Critical → Dead; boundary-breach unresolvable by automation — Dusk tribe steward must intervene to restore or reclassify", state: "Dead" },
    ],
];

// ── Runtime state ────────────────────────────────────────────────────────────

pub struct StoryEngineState {
    pub particle_idx: Option<usize>,
    pub selected_node: Option<usize>,
    pub pan_x: f64,
}

impl StoryEngineState {
    pub fn new() -> Self {
        StoryEngineState {
            particle_idx: None,
            selected_node: None,
            pan_x: 0.0,
        }
    }
}

impl Default for StoryEngineState {
    fn default() -> Self {
        Self::new()
    }
}

impl StoryEngineState {
    pub fn open(&mut self, pi: usize) {
        self.particle_idx = Some(pi);
        self.selected_node = None;
        self.pan_x = 0.0;
    }

    pub fn close(&mut self) {
        self.particle_idx = None;
        self.selected_node = None;
    }

    pub fn pan(&mut self, delta: f64, n_events: usize) {
        let max_pan = 0.0_f64;
        let min_pan = -(((n_events as f64 - 1.0) * 158.0).max(0.0));
        self.pan_x = (self.pan_x + delta).clamp(min_pan, max_pan);
    }
}

// ── Geometry ────────────────────────────────────────────────────────────────

// Returns (x, y, w, h) for node i in the story canvas coordinate space.
// Nodes are 132×54, spaced 158px apart horizontally.
pub fn node_bounds(i: usize, pan_x: f64) -> (f64, f64, f64, f64) {
    (24.0 + i as f64 * 158.0 + pan_x, 90.0, 132.0, 54.0)
}

// Returns the index of the node that contains canvas point (cx, cy), if any.
pub fn hit_node(n: usize, cx: f64, cy: f64, pan_x: f64) -> Option<usize> {
    for i in 0..n {
        let (x, y, w, h) = node_bounds(i, pan_x);
        if cx >= x && cx <= x + w && cy >= y && cy <= y + h {
            return Some(i);
        }
    }
    None
}

// ── Causal-event analysis ────────────────────────────────────────────────────

/// Returns event indices considered root causes for "Dead" or "Fuzzy" particles.
/// Highlights DriftAlert, Quarantine and StatusChange events that led to the state.
pub fn causal_events(status: &str, events: &[StoryEvent]) -> Vec<usize> {
    if status != "Dead" && status != "Fuzzy" {
        return vec![];
    }
    events
        .iter()
        .enumerate()
        .filter(|(_, ev)| {
            matches!(
                ev.kind,
                EventKind::DriftAlert | EventKind::Quarantine | EventKind::StatusChange
            )
        })
        .map(|(i, _)| i)
        .collect()
}

// ── Steward state ────────────────────────────────────────────────────────────

pub struct ParticleOverride {
    pub status: String,
    pub ptype: String, // label string, e.g. "Zikru"
    pub note: String,
}

pub struct StewardState {
    pub target: Option<usize>,
    pub overrides: Vec<Option<ParticleOverride>>,
    pub edit_status: String,
    pub edit_ptype: String,
    pub edit_note: String,
}

impl StewardState {
    pub fn new(n: usize) -> Self {
        StewardState {
            target: None,
            overrides: (0..n).map(|_| None).collect(),
            edit_status: String::new(),
            edit_ptype: String::new(),
            edit_note: String::new(),
        }
    }

    pub fn effective_status<'a>(&'a self, pi: usize, original: &'a str) -> &'a str {
        self.overrides
            .get(pi)
            .and_then(|o| o.as_ref())
            .map(|o| o.status.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(original)
    }

    pub fn effective_ptype<'a>(&'a self, pi: usize, original: &'a str) -> &'a str {
        self.overrides
            .get(pi)
            .and_then(|o| o.as_ref())
            .map(|o| o.ptype.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(original)
    }
}
