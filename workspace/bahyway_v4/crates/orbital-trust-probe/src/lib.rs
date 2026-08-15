//! 𒁾 orbital-trust-probe — Causal Orbital Deviation Detection v4.0
//!
//! Distinguishes LEGITIMATE 7D particle motion from UNEXPLAINED trust violations.
//!
//! # The Problem
//! At billion-particle scale, any static snapshot of orbital positions is
//! immediately obsolete. Particles move in 7D space when their data evolves,
//! when neighbours shift the SPH density field, when freshness decays, or when
//! the fuzzy rule thresholds are updated. Naively flagging all orbital deviation
//! as "untrusted" produces cascading false-positive trust penalties that corrupt
//! the entire scoring field.
//!
//! # The Solution — 4-Step Causal Attribution
//! Before issuing a trust penalty the probe works through an ordered causal chain:
//!
//! ```text
//! Step 1 — FuzzyRules fingerprint changed?  → ABORT (structural, not suspicious)
//! Step 2 — StoryEngine shows new EAV events? → LEGITIMATE STATE EVOLUTION
//! Step 3 — ScoreEngine explains the move?
//!            3a. Neighbour density shift ≥ δ?  → NEIGHBOR_DENSITY_SHIFT
//!            3b. Freshness BLUE byte dropped?   → FRESHNESS_DECAY
//!            3c. B11 within hysteresis band?    → THRESHOLD_BOUNDARY_NOISE
//! Step 4 — No explanation found              → UNEXPLAINED → AuditJournal
//! ```
//!
//! Only `Unexplained` deviations route to `OrbitalDeviationJournal` and apply
//! a trust_penalty to the particle's SourceTrust fuzzy dimension.
//!
//! # Integration Points
//! - Consumes `OrbitalSnapshot` pairs produced after every `score-engine` pass.
//! - Reads `StoryEngine::event_count` delta for Step 2.
//! - Writes `OrbitalDeviationJournal` entries (parallel to `enkidullm-audit`).
//! - Returns `trust_penalty` for feed-back into `fuzzy-engine::SourceTrust`.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

#![forbid(unsafe_code)]

pub mod cause;
pub mod snapshot;
pub mod probe;
pub mod journal;

pub use cause::DeviationCause;
pub use snapshot::{OrbitalSnapshot, HYSTERESIS, near_boundary};
pub use probe::{
    probe, batch_probe, ProbeInput, ProbeResult,
    MIN_DEVIATION_DISTANCE, FRESHNESS_DECAY_THRESHOLD, NEIGHBOUR_DELTA_THRESHOLD,
};
pub use journal::{OrbitalDeviationJournal, OrbitalDeviationEntry};

/// High-level convenience: run the probe, and if the result is a genuine trust
/// violation automatically record it in the journal.
///
/// Returns the `ProbeResult` in all cases so the caller can read the cause and
/// trust_penalty regardless of whether a journal entry was written.
pub fn probe_and_journal(
    input:       &ProbeInput<'_>,
    particle_id: u64,
    journal:     &mut OrbitalDeviationJournal,
) -> ProbeResult {
    let result = probe(input);
    if result.is_trust_violation() {
        let entry = OrbitalDeviationEntry::new(
            particle_id,
            input.current.epoch,
            input.previous.epoch,
            result.cartesian_delta,
            result.ring_changed,
            result.cause,
            result.trust_penalty,
        );
        journal.append(entry);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tribe_orbit_engine::OrbitalAssignment;
    use fuzzy_engine::QualityTier;

    const FP: u64 = 0xBEEF_CAFE_0000_0001;

    fn snap(epoch: u64, b11: u8, kaki: [u8; 16]) -> OrbitalSnapshot {
        let assignment = OrbitalAssignment::from_kaki(&kaki, b11);
        let tier = if b11 >= 200 { QualityTier::Gem } else { QualityTier::NonActive };
        OrbitalSnapshot::new(epoch, b11, tier, assignment, 3, 200, FP)
    }

    #[test]
    fn probe_and_journal_records_unexplained() {
        // Deep Gem → deep Dead, different KAKI, no events, stable rules
        let kaki_a = [0u8; 16];
        let kaki_b = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 255, 0];
        let prev   = snap(1, 240, kaki_a);
        let curr   = snap(2, 20,  kaki_b);

        let mut journal = OrbitalDeviationJournal::new();
        let input   = ProbeInput { previous: &prev, current: &curr, new_eav_events: 0 };
        let result  = probe_and_journal(&input, 0xDEAD_BEEF, &mut journal);

        assert!(result.is_trust_violation());
        assert_eq!(journal.len(), 1);
        assert!(journal.entries()[0].verify());
    }

    #[test]
    fn probe_and_journal_does_not_record_legitimate() {
        let prev = snap(1, 210, [0u8; 16]);
        let curr = snap(2, 155, [0u8; 16]);

        let mut journal = OrbitalDeviationJournal::new();
        let input  = ProbeInput { previous: &prev, current: &curr, new_eav_events: 5 };
        let result = probe_and_journal(&input, 0xAAAA, &mut journal);

        assert!(!result.is_trust_violation());
        assert!(journal.is_empty(), "legitimate evolution must not be journaled");
    }

    #[test]
    fn accumulated_penalty_feeds_back_correctly() {
        let mut journal = OrbitalDeviationJournal::new();
        let kaki_b = [0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 255, 0];

        // Two distinct unexplained deviations at different epochs
        for obs_epoch in [10u64, 20u64] {
            let prev = snap(obs_epoch - 1, 240, [0u8; 16]);
            let curr = snap(obs_epoch,     20,  kaki_b);
            let input = ProbeInput { previous: &prev, current: &curr, new_eav_events: 0 };
            probe_and_journal(&input, 0x1111, &mut journal);
        }

        let penalty = journal.accumulated_penalty(0x1111);
        assert!(penalty > 0.0, "accumulated penalty must be positive");
        assert!(penalty <= 1.0, "accumulated penalty must not exceed 1.0");
    }
}
