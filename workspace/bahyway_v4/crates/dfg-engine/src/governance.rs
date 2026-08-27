#![forbid(unsafe_code)]
//! Governance types — GovernanceDecision, ParticleState, DataSteward trait.
//!
//! Formalised from UEK architecture doc §7, §10:
//!   Accept                  — event passes all .akk rules
//!   Flag                    — event flagged but not blocked
//!   SelfHealAllowed         — autonomous correction permitted (Tier-1 heal)
//!   RequiresHuman           — escalate to Data Steward (Tier-2 authority)
//!   RejectImmutableViolation — KAKI identity constraint violated — hard reject

use super::node::DfgNode;
use super::plan::FeatureAttrib;

/// Five-state governance verdict from a Data Steward evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceDecision {
    Accept,
    Flag,
    SelfHealAllowed,
    RequiresHuman,
    RejectImmutableViolation,
}

/// Coarse health state of a data particle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticleState {
    Healthy,
    Degraded,
    Anomalous,
    Corrupted,
}

impl ParticleState {
    /// Map a normalised quality score [0,1] to a health state.
    /// Boundary: 0.8 = Healthy, 0.5 = Degraded, 0.2 = Anomalous, else Corrupted.
    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            Self::Healthy
        } else if score >= 0.5 {
            Self::Degraded
        } else if score >= 0.2 {
            Self::Anomalous
        } else {
            Self::Corrupted
        }
    }
}

/// Trait implemented by any component that evaluates a DFG node + its
/// feature vector and produces a governance decision.
pub trait DataSteward {
    fn evaluate(&self, node: &DfgNode, attribs: &[FeatureAttrib]) -> GovernanceDecision;
}

// ── Built-in pass-through steward ─────────────────────────────────────────────

/// Permissive steward: accepts everything unless an explicit `risk:level`
/// attrib is present with value HIGH or CRITICAL, in which case it escalates
/// to RequiresHuman.  Spatial `condition:` fields are NOT governance triggers —
/// they are sector topology metadata and are ignored here.
pub struct DefaultSteward;

impl DataSteward for DefaultSteward {
    fn evaluate(&self, _node: &DfgNode, attribs: &[FeatureAttrib]) -> GovernanceDecision {
        for a in attribs {
            if a.domain == "risk" && a.key == "level" {
                match a.value.to_uppercase().as_str() {
                    "HIGH" | "CRITICAL" => return GovernanceDecision::RequiresHuman,
                    _ => {}
                }
            }
        }
        GovernanceDecision::Accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{DfgNode, NodeKind};
    use crate::plan::FeatureAttrib;

    fn dummy_node() -> DfgNode {
        DfgNode {
            id: 0,
            kind: NodeKind::ExecutionCluster,
            label: "TEST".into(),
            sector_index: 0,
            condition: None,
            attribs: vec![],
        }
    }

    #[test]
    fn default_steward_accepts_clean() {
        let d = DefaultSteward;
        assert_eq!(d.evaluate(&dummy_node(), &[]), GovernanceDecision::Accept);
    }

    #[test]
    fn default_steward_escalates_high_risk() {
        let d = DefaultSteward;
        let attribs = vec![FeatureAttrib {
            domain: "risk".into(),
            key: "level".into(),
            value: "HIGH".into(),
        }];
        assert_eq!(
            d.evaluate(&dummy_node(), &attribs),
            GovernanceDecision::RequiresHuman
        );
    }

    #[test]
    fn particle_state_thresholds() {
        assert_eq!(ParticleState::from_score(1.0), ParticleState::Healthy);
        assert_eq!(ParticleState::from_score(0.8), ParticleState::Healthy);
        assert_eq!(ParticleState::from_score(0.79), ParticleState::Degraded);
        assert_eq!(ParticleState::from_score(0.5), ParticleState::Degraded);
        assert_eq!(ParticleState::from_score(0.49), ParticleState::Anomalous);
        assert_eq!(ParticleState::from_score(0.19), ParticleState::Corrupted);
    }
}
