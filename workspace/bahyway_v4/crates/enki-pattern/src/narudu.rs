#![forbid(unsafe_code)]
//! NARUDU-PATTERN — event stream protocol for pattern lifecycle transitions.
//!
//! Every lifecycle state change emits a NaruduEvent.  Consumers subscribe
//! to the event stream to react to pattern changes (NARUDU-001).

use crate::nash::NashState;
use crate::tier::PatternRecord;
use enkidb_kaki::Kaki;

/// The five NARUDU-PATTERN event kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NaruduEventKind {
    /// A new pattern-kaki was minted and submitted to the registry.
    Published,
    /// The Nash equilibrium score changed significantly (delta ≥ 0.10).
    NashShift,
    /// Pattern was deprecated (superseded or manually retired).
    Deprecated,
    /// ENKI-GENESIS completed a simulation batch — synthetic clusters available.
    SimulationComplete,
    /// A DataSteward issued a Šàtamu decision for this pattern.
    StewardDecision,
}

impl NaruduEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NaruduEventKind::Published => "Published",
            NaruduEventKind::NashShift => "NashShift",
            NaruduEventKind::Deprecated => "Deprecated",
            NaruduEventKind::SimulationComplete => "SimulationComplete",
            NaruduEventKind::StewardDecision => "StewardDecision",
        }
    }
}

impl core::fmt::Display for NaruduEventKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A NARUDU-PATTERN lifecycle event.
#[derive(Clone, Debug)]
pub struct NaruduEvent {
    /// The pattern-kaki this event concerns.
    pub pattern_kaki: Kaki,
    /// Event classification.
    pub kind: NaruduEventKind,
    /// BASE orbital when this event was emitted.
    pub orbital: u64,
    /// Nash state snapshot at event time (None for Published/SimulationComplete).
    pub nash_snapshot: Option<NashState>,
    /// Optional detail text (≤ 128 ASCII bytes).
    pub detail: &'static str,
}

impl NaruduEvent {
    pub fn published(record: &PatternRecord) -> Self {
        Self {
            pattern_kaki: record.pattern_kaki,
            kind: NaruduEventKind::Published,
            orbital: record.orbital_published,
            nash_snapshot: None,
            detail: "pattern-kaki minted and registered",
        }
    }

    pub fn nash_shift(pattern_kaki: Kaki, nash: NashState, orbital: u64) -> Self {
        Self {
            pattern_kaki,
            kind: NaruduEventKind::NashShift,
            orbital,
            nash_snapshot: Some(nash),
            detail: "Nash equilibrium score changed",
        }
    }

    pub fn deprecated(pattern_kaki: Kaki, orbital: u64, reason: &'static str) -> Self {
        Self {
            pattern_kaki,
            kind: NaruduEventKind::Deprecated,
            orbital,
            nash_snapshot: None,
            detail: reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{derive_pattern_kaki, FixedCoord7D, PatternLifecycle, PatternType};

    fn sample_record() -> PatternRecord {
        let k = derive_pattern_kaki(
            PatternType::CrowdFlow,
            FixedCoord7D::zero(),
            [0u8; 32],
            8_500,
            10,
        );
        PatternRecord {
            pattern_kaki: k,
            pattern_type: PatternType::CrowdFlow,
            lifecycle: PatternLifecycle::Stable,
            confidence_millipercent: 8_500,
            constituent_count: 42,
            constituent_merkle_root: [0u8; 32],
            orbital_published: 10,
            orbital_now: 10,
            nash_equilibrium_score: 0.0,
        }
    }

    #[test]
    fn published_event_matches_record() {
        let rec = sample_record();
        let ev = NaruduEvent::published(&rec);
        assert_eq!(ev.kind, NaruduEventKind::Published);
        assert_eq!(ev.pattern_kaki, rec.pattern_kaki);
        assert!(ev.nash_snapshot.is_none());
    }

    #[test]
    fn nash_shift_captures_state() {
        let rec = sample_record();
        let nash = NashState {
            equilibrium_score: 0.70,
            anarchy_score: 1.2,
            deviation_count: 15,
            computed_at_orbital: 50,
        };
        let ev = NaruduEvent::nash_shift(rec.pattern_kaki, nash, 50);
        assert_eq!(ev.kind, NaruduEventKind::NashShift);
        assert!(ev.nash_snapshot.is_some());
    }
}
