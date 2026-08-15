//! AMMAS Agent classes — mirrors the 3 KAKI identity types.
//!
//! AgentClass maps to KakiType:
//!   Identity  → IdentityKaki (Type 1) — permanent sovereign particle
//!   Event     → EventKaki    (Type 2) — mutable state carrier
//!   CrossTribe → CrossTribeKaki (Type 3) — inter-tribe connector
//!
//! AgentState encodes the current J_mem lane of the particle:
//!   Active, Fuzzy, Dead — matches QualityLane bands.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

/// Mirrors the 3 KAKI discriminator bytes (kaki_type byte at offset 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentClass {
    /// Type 1 — IdentityKaki: permanent, sovereign nucleus.
    Identity,
    /// Type 2 — EventKaki: mutable EAV carrier, drives J_mem.
    Event,
    /// Type 3 — CrossTribeKaki: inter-tribe kinetic connector.
    CrossTribe,
}

impl AgentClass {
    /// Decode from the kaki_type discriminator byte (offset 6 of KAKI pk).
    pub fn from_kaki_byte(byte: u8) -> Self {
        match byte {
            1 => Self::Identity,
            3 => Self::CrossTribe,
            _ => Self::Event, // type 2 and any unrecognised byte
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Identity   => "IDENTITY",
            Self::Event      => "EVENT",
            Self::CrossTribe => "CROSS_TRIBE",
        }
    }
}

/// Current kinetic state of a sovereign particle — drives law-transition logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentState {
    /// B11 ≥ 200 — GEM quality, tightly bound to inner ring.
    Gem,
    /// B11 ≥ 140 — TRIBE quality.
    Tribe,
    /// B11 ≥ 100 — ACTIVE quality.
    Active,
    /// B11 ≥ 60 — FUZZY quality, near dead boundary.
    Fuzzy,
    /// B11 < 60 — DEAD, outer ring, candidate for Rule 7 sink.
    Dead,
}

impl AgentState {
    /// Decode from a B11 quality byte.
    pub fn from_b11(b11: u8) -> Self {
        if      b11 >= 200 { Self::Gem    }
        else if b11 >= 140 { Self::Tribe  }
        else if b11 >= 100 { Self::Active }
        else if b11 >=  60 { Self::Fuzzy  }
        else               { Self::Dead   }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Gem    => "GEM",
            Self::Tribe  => "TRIBE",
            Self::Active => "ACTIVE",
            Self::Fuzzy  => "FUZZY",
            Self::Dead   => "DEAD",
        }
    }

    /// Whether this state permits quality scoring updates (Structural-Facts rule §2.4).
    pub fn permits_scoring(self) -> bool {
        !matches!(self, Self::Dead)
    }
}

/// Snapshot of a single agent for one kinetic timestep.
#[derive(Debug, Clone)]
pub struct AgentSnapshot {
    /// 16-byte KAKI primary key.
    pub kaki: [u8; 16],
    /// Agent class derived from kaki_type byte.
    pub class: AgentClass,
    /// Current quality state derived from B11.
    pub state: AgentState,
    /// Raw B11 byte (0–240 normalised quality).
    pub b11: u8,
    /// 3D position in torus space [x, y, z].
    pub position: [f32; 3],
}

impl AgentSnapshot {
    pub fn new(kaki: [u8; 16], b11: u8, position: [f32; 3]) -> Self {
        Self {
            class:    AgentClass::from_kaki_byte(kaki[6]),
            state:    AgentState::from_b11(b11),
            kaki,
            b11,
            position,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_class_from_kaki_byte() {
        assert_eq!(AgentClass::from_kaki_byte(1), AgentClass::Identity);
        assert_eq!(AgentClass::from_kaki_byte(2), AgentClass::Event);
        assert_eq!(AgentClass::from_kaki_byte(3), AgentClass::CrossTribe);
        assert_eq!(AgentClass::from_kaki_byte(0), AgentClass::Event); // default
    }

    #[test]
    fn agent_state_gem_boundary() {
        assert_eq!(AgentState::from_b11(240), AgentState::Gem);
        assert_eq!(AgentState::from_b11(200), AgentState::Gem);
        assert_eq!(AgentState::from_b11(199), AgentState::Tribe);
    }

    #[test]
    fn agent_state_tribe_boundary() {
        assert_eq!(AgentState::from_b11(140), AgentState::Tribe);
        assert_eq!(AgentState::from_b11(139), AgentState::Active);
    }

    #[test]
    fn agent_state_active_boundary() {
        assert_eq!(AgentState::from_b11(100), AgentState::Active);
        assert_eq!(AgentState::from_b11(99),  AgentState::Fuzzy);
    }

    #[test]
    fn agent_state_dead_boundary() {
        assert_eq!(AgentState::from_b11(60), AgentState::Fuzzy);
        assert_eq!(AgentState::from_b11(59), AgentState::Dead);
        assert_eq!(AgentState::from_b11(0),  AgentState::Dead);
    }

    #[test]
    fn dead_agent_does_not_permit_scoring() {
        assert!(!AgentState::Dead.permits_scoring());
        assert!(AgentState::Gem.permits_scoring());
        assert!(AgentState::Fuzzy.permits_scoring());
    }

    #[test]
    fn agent_snapshot_derives_class_and_state() {
        let mut kaki = [0u8; 16];
        kaki[6] = 3; // CrossTribe
        let snap = AgentSnapshot::new(kaki, 210, [0.0, 0.0, 0.0]);
        assert_eq!(snap.class, AgentClass::CrossTribe);
        assert_eq!(snap.state, AgentState::Gem);
    }
}
