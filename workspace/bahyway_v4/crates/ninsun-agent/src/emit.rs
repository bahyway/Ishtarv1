//! NINSUN_REFINE EAV particle builder.
//! Advisory only — no particle is modified in place.
#![forbid(unsafe_code)]
use crate::{drift::DriftEvent, error::NinsunError, RefineProposal};
use chrono::Utc;
use uuid::Uuid;

pub fn build_proposal(
    tribe_id: &str,
    drift: &DriftEvent,
    explanation: String,
) -> Result<RefineProposal, NinsunError> {
    Ok(RefineProposal {
        proposal_id: Uuid::new_v4().to_string(),
        target_kaki: drift.kaki_id.clone(),
        tribe_id: tribe_id.to_string(),
        drift_pattern: drift.pattern_code.clone(),
        explanation,
        suggested_eav: vec![
            ("ninsun_reviewed".to_string(), Utc::now().to_rfc3339()),
            ("drift_pattern".to_string(), drift.pattern_code.clone()),
        ],
        confidence: 0.7,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_proposal_carries_drift_fields() {
        let drift = DriftEvent {
            kaki_id: "k1".into(),
            tribe_id: "t1".into(),
            pattern_code: "DEAD_UNREVIEWED".into(),
            days_in_state: 2,
        };
        let proposal = build_proposal("t1", &drift, "explanation".into()).unwrap();
        assert_eq!(proposal.target_kaki, "k1");
        assert_eq!(proposal.drift_pattern, "DEAD_UNREVIEWED");
        assert!(proposal.confidence > 0.0 && proposal.confidence <= 1.0);
        assert_eq!(proposal.suggested_eav.len(), 2);
    }
}
