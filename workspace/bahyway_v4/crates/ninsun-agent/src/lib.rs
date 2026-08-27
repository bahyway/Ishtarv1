//! ninsun-agent — NINSUN: Healer + Progressive Refiner
//! 𒀭𒊩𒇻 — Fourth member of the sovereign Agent Quartet.
//!
//! NINSUN proposes, the Sovereign decides.
//! All refinement suggestions are emitted as NINSUN_REFINE EAV particles —
//! they are advisory, never corrective. No particle is modified in place.
#![forbid(unsafe_code)]
pub mod drift;
pub mod emit;
pub mod error;
pub mod explain;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineProposal {
    pub proposal_id: String,
    pub target_kaki: String, // KAKI ID being advised on
    pub tribe_id: String,
    pub drift_pattern: String,                // e.g. "FUZZY_SUSTAINED_7DAY"
    pub explanation: String,                  // human-readable from DeepSeek
    pub suggested_eav: Vec<(String, String)>, // proposed EAV corrections
    pub confidence: f64,                      // 0.0..1.0
}

/// Top-level analysis entry point.
/// Takes a batch of recent KAKI particle summaries, returns proposals.
pub fn analyze(tribe_id: &str, kaki_batch: &[KakiSummary]) -> Vec<RefineProposal> {
    let drift_events = drift::detect_drift(kaki_batch);
    drift_events
        .into_iter()
        .filter_map(|d| {
            let explanation = explain::offline_explain(&d);
            match emit::build_proposal(tribe_id, &d, explanation) {
                Ok(p) => Some(p),
                Err(e) => {
                    tracing::warn!("NINSUN proposal build failed: {e}");
                    None
                }
            }
        })
        .collect()
}

/// Minimal KAKI summary fed to NINSUN from the HeptaScript query result.
#[derive(Debug, Clone)]
pub struct KakiSummary {
    pub kaki_id: String,
    pub eav_state: String,     // GOLDEN / FUZZY / DEAD
    pub orbital_state: String, // GOLDEN_ALIVE / FUZZY_AGED / DEAD_EXPIRED etc
    pub tribe_id: String,
    pub days_in_state: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(state: &str, days: u32) -> KakiSummary {
        KakiSummary {
            kaki_id: "kaki-1".into(),
            eav_state: state.into(),
            orbital_state: "N/A".into(),
            tribe_id: "test_tribe".into(),
            days_in_state: days,
        }
    }

    #[test]
    fn analyze_emits_proposal_for_sustained_fuzzy() {
        let proposals = analyze("test_tribe", &[summary("FUZZY", 10)]);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].drift_pattern, "FUZZY_SUSTAINED_7DAY");
        assert_eq!(proposals[0].tribe_id, "test_tribe");
    }

    #[test]
    fn analyze_ignores_healthy_particle() {
        let proposals = analyze("test_tribe", &[summary("GOLDEN", 30)]);
        assert!(proposals.is_empty());
    }

    #[test]
    fn analyze_emits_proposal_for_unreviewed_dead() {
        let proposals = analyze("test_tribe", &[summary("DEAD", 1)]);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].drift_pattern, "DEAD_UNREVIEWED");
    }
}
