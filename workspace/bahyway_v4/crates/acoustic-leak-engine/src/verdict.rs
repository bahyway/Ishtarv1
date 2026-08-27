//! The barû judgment layer. LAW-ALE-1: proven, never asserted.
//!
//! A verdict is signed ONLY when:
//!   - the stage is authorized to claim position at all, and
//!   - sigma_d <= SOVEREIGN_MARGIN_M (0.10 m), and
//!   - the peak is not inside the junction guard (else Stage 3 mandatory).
//!
//! Anything less is an Escalation, never a Verdict.

use crate::{JUNCTION_GUARD_M, SOVEREIGN_MARGIN_M};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    /// Hydraulic King-plot residuals -- flags a SEGMENT, never a position.
    Stage1Hydraulic,
    /// Fixed-sensor correlation -- metres-class, position candidate only.
    Stage2Correlation,
    /// In-pipe pass / tracer-gas pinpoint / joint-map snap -- decimetre class.
    Stage3Pinpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeakSite {
    PipeBody,
    Junction,
}

#[derive(Debug, Clone, Copy)]
pub struct PositionClaim {
    pub stage: Stage,
    pub position_m: f64,
    pub sigma_d_m: f64,
    pub segment_length_m: f64,
    pub site: LeakSite,
}

#[derive(Debug, Clone, Copy)]
pub struct SignedVerdict {
    pub position_m: f64,
    pub sigma_d_m: f64,
    pub site: LeakSite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escalation {
    /// Stage 1 never localizes; run Stage 2 on the flagged segment.
    RunStage2Correlation,
    /// sigma_d above margin, or peak inside junction guard: run Stage 3
    /// (in-pipe sensor pass, tracer-gas pinpoint, or joint-map snap).
    RunStage3Pinpoint,
    /// Overlapping sensor pairs disagree: v calibration invalid --
    /// recalibrate before any further claim (LAW-ALE-3).
    RecalibrateVelocity,
}

pub fn near_junction(position_m: f64, segment_length_m: f64) -> bool {
    position_m < JUNCTION_GUARD_M || (segment_length_m - position_m) < JUNCTION_GUARD_M
}

/// The gate. Returns a signed verdict or the mandated escalation.
pub fn judge(claim: &PositionClaim) -> Result<SignedVerdict, Escalation> {
    match claim.stage {
        Stage::Stage1Hydraulic => Err(Escalation::RunStage2Correlation),
        Stage::Stage2Correlation => {
            if near_junction(claim.position_m, claim.segment_length_m) {
                return Err(Escalation::RunStage3Pinpoint);
            }
            if claim.sigma_d_m <= SOVEREIGN_MARGIN_M {
                // Even a compliant Stage-2 sigma_d is only signable on pipe
                // body away from nodes.
                Ok(SignedVerdict {
                    position_m: claim.position_m,
                    sigma_d_m: claim.sigma_d_m,
                    site: LeakSite::PipeBody,
                })
            } else {
                Err(Escalation::RunStage3Pinpoint)
            }
        }
        Stage::Stage3Pinpoint => {
            if claim.sigma_d_m <= SOVEREIGN_MARGIN_M {
                Ok(SignedVerdict {
                    position_m: claim.position_m,
                    sigma_d_m: claim.sigma_d_m,
                    site: claim.site,
                })
            } else {
                // Stage 3 above margin means the measurement itself must be
                // repeated under better conditions (night survey).
                Err(Escalation::RunStage3Pinpoint)
            }
        }
    }
}

/// Cross-check of overlapping sensor pairs (A-B and B-C sharing node B):
/// if both brackets claim the leak, the velocity model is lying.
pub fn cross_check_overlap(pair_ab_claims: bool, pair_bc_claims: bool) -> Option<Escalation> {
    if pair_ab_claims && pair_bc_claims {
        Some(Escalation::RecalibrateVelocity)
    } else {
        None
    }
}
