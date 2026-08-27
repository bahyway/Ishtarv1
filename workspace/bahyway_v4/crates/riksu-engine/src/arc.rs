//! RiksuArc — a sovereign cross-tribe bond in the 3D visualization.
//!
//! Riksu (𒊑𒆪𒍪, Akkadian: "bond, knot, treaty") are the visible arcs
//! connecting two tribes in the Big Ring visualization. Each arc is derived
//! from CrossTribeKaki (Type 3) events between the two tribes.
//!
//! Arc strength is the normalized frequency of cross-tribe interactions.
//! Arc state tracks the health of the connection as nodes fail.

/// Maximum arc strength (fully bonded tribes).
pub const ARC_STRENGTH_MAX: f32 = 1.0;
/// Minimum strength before the arc is considered severed.
pub const ARC_STRENGTH_MIN: f32 = 0.05;
/// Dimming rate per failed neighbour (fraction per failure event).
pub const ARC_DIM_PER_FAILURE: f32 = 0.35;

/// Visual and functional state of a Riksu arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcState {
    /// Both tribes healthy — arc renders at full brightness.
    Active,
    /// One tribe node degrading (FUZZY lane) — arc dims.
    Dimming,
    /// One tribe node DEAD or offline — arc is severed.
    Severed,
    /// Arc crystallized at the Quantum Freeze moment — read-only.
    Frozen,
}

impl ArcState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Dimming => "DIMMING",
            Self::Severed => "SEVERED",
            Self::Frozen => "FROZEN",
        }
    }

    pub fn is_visible(self) -> bool {
        !matches!(self, Self::Severed)
    }

    pub fn opacity(self, strength: f32) -> f32 {
        match self {
            Self::Active => strength,
            Self::Dimming => strength * 0.5,
            Self::Severed => 0.0,
            Self::Frozen => strength * 0.8,
        }
    }
}

/// A sovereign cross-tribe arc in the 3D Riksu topology.
#[derive(Debug, Clone)]
pub struct RiksuArc {
    /// Unique 16-byte arc identifier (derived from the two tribe IDs).
    pub arc_id: [u8; 16],
    /// Source tribe identifier.
    pub tribe_a: u16,
    /// Destination tribe identifier.
    pub tribe_b: u16,
    /// Bond strength: 0..1 normalised cross-tribe interaction frequency.
    pub strength: f32,
    /// Current visual/functional state.
    pub state: ArcState,
    /// Number of CrossTribeKaki events that created this arc.
    pub event_count: u64,
}

impl RiksuArc {
    /// Create a new arc between two tribes.
    pub fn new(tribe_a: u16, tribe_b: u16, event_count: u64, max_events: u64) -> Self {
        let strength = if max_events == 0 {
            0.0
        } else {
            (event_count as f32 / max_events as f32).clamp(0.0, 1.0)
        };
        Self {
            arc_id: arc_id_from_tribes(tribe_a, tribe_b),
            tribe_a,
            tribe_b,
            strength,
            state: ArcState::Active,
            event_count,
        }
    }

    /// Whether this arc connects to the given tribe.
    pub fn connects(&self, tribe_id: u16) -> bool {
        self.tribe_a == tribe_id || self.tribe_b == tribe_id
    }

    /// The opposite tribe of the given one in this arc.
    pub fn other_tribe(&self, tribe_id: u16) -> Option<u16> {
        if self.tribe_a == tribe_id {
            Some(self.tribe_b)
        } else if self.tribe_b == tribe_id {
            Some(self.tribe_a)
        } else {
            None
        }
    }

    /// Apply a dimming event (one of the connected nodes degraded).
    pub fn apply_dim(&mut self) {
        self.strength = (self.strength - ARC_DIM_PER_FAILURE).max(0.0);
        if self.strength < ARC_STRENGTH_MIN {
            self.state = ArcState::Severed;
        } else {
            self.state = ArcState::Dimming;
        }
    }

    /// Sever this arc (connected node went DEAD).
    pub fn sever(&mut self) {
        self.strength = 0.0;
        self.state = ArcState::Severed;
    }

    /// Freeze this arc (Quantum Freeze fired).
    pub fn freeze(&mut self) {
        self.state = ArcState::Frozen;
    }

    pub fn opacity(&self) -> f32 {
        self.state.opacity(self.strength)
    }
}

/// Derive a canonical 16-byte arc identifier from two tribe IDs.
/// Canonical form: smaller tribe_id is always tribe_a for uniqueness.
pub fn arc_id_from_tribes(tribe_a: u16, tribe_b: u16) -> [u8; 16] {
    let (a, b) = if tribe_a <= tribe_b {
        (tribe_a, tribe_b)
    } else {
        (tribe_b, tribe_a)
    };

    const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
    const FNV_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

    let mut data = [0u8; 6];
    data[..2].copy_from_slice(&a.to_le_bytes());
    data[2..4].copy_from_slice(&b.to_le_bytes());
    data[4] = 0xAC; // Akkadian marker
    data[5] = 0xC5;

    let mut hash = FNV_BASIS;
    for &byte in &data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    let hash2 = hash.wrapping_mul(FNV_PRIME);

    let mut id = [0u8; 16];
    id[..8].copy_from_slice(&hash.to_le_bytes());
    id[8..].copy_from_slice(&hash2.to_le_bytes());
    id[6] = 0x03; // CrossTribeKaki type byte
    id
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_id_symmetric() {
        let id1 = arc_id_from_tribes(1, 2);
        let id2 = arc_id_from_tribes(2, 1);
        assert_eq!(id1, id2, "arc_id must be symmetric");
    }

    #[test]
    fn arc_id_type_byte_is_cross_tribe() {
        let id = arc_id_from_tribes(1, 2);
        assert_eq!(id[6], 0x03, "arc KAKI must have CrossTribeKaki type");
    }

    #[test]
    fn arc_id_distinct_pairs() {
        let id12 = arc_id_from_tribes(1, 2);
        let id13 = arc_id_from_tribes(1, 3);
        assert_ne!(id12, id13);
    }

    #[test]
    fn new_arc_active_state() {
        let arc = RiksuArc::new(1, 2, 50, 100);
        assert_eq!(arc.state, ArcState::Active);
        assert!((arc.strength - 0.5).abs() < 1e-5);
    }

    #[test]
    fn dim_reduces_strength() {
        let mut arc = RiksuArc::new(1, 2, 100, 100);
        arc.apply_dim();
        assert!(arc.strength < 1.0);
        assert_eq!(arc.state, ArcState::Dimming);
    }

    #[test]
    fn sever_makes_invisible() {
        let mut arc = RiksuArc::new(1, 2, 100, 100);
        arc.sever();
        assert_eq!(arc.state, ArcState::Severed);
        assert!(!arc.state.is_visible());
        assert_eq!(arc.opacity(), 0.0);
    }

    #[test]
    fn freeze_preserves_strength_at_80pct() {
        let mut arc = RiksuArc::new(1, 2, 100, 100);
        arc.freeze();
        assert_eq!(arc.state, ArcState::Frozen);
        assert!(arc.opacity() > 0.0, "frozen arc should still be visible");
    }

    #[test]
    fn connects_returns_true_for_both_tribes() {
        let arc = RiksuArc::new(3, 7, 10, 100);
        assert!(arc.connects(3));
        assert!(arc.connects(7));
        assert!(!arc.connects(1));
    }

    #[test]
    fn other_tribe_returns_opposite() {
        let arc = RiksuArc::new(3, 7, 10, 100);
        assert_eq!(arc.other_tribe(3), Some(7));
        assert_eq!(arc.other_tribe(7), Some(3));
        assert_eq!(arc.other_tribe(1), None);
    }

    #[test]
    fn multiple_dims_can_sever() {
        let mut arc = RiksuArc::new(1, 2, 100, 100);
        // ARC_DIM_PER_FAILURE = 0.35, so 3 dims should sever (1.0 - 3×0.35 = -0.05 < MIN)
        arc.apply_dim();
        arc.apply_dim();
        arc.apply_dim();
        assert_eq!(arc.state, ArcState::Severed);
    }
}
