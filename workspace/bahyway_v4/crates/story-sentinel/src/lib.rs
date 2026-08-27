//! story-sentinel — StoryEngine's alert bridge to the Data Steward Station.
//!
//! StoryEngine is "the ONLY path through which current state is obtained"
//! (§3.3) — every particle a query touches already passes through a
//! `ProjectedState`, so this crate turns that projection into a
//! diagnosis + alert without a separate poller.
//!
//! Shape reused deliberately from `bahyway_algebra::enbilulu` (the sealed
//! Têrtu/Mīlu diagnosis pattern: weighted factors → banding → an
//! `alert_engine::AlertSeverity` bridge → dominant-factor cause →
//! narration) — adapted to what a `ProjectedState` actually carries,
//! rather than inventing a new pattern for the same job.
//!
//! Lives in its own crate, not inside `story-engine`, because
//! `alert-engine` already depends on `story-engine` (via `score-engine`)
//! — `story-engine` depending back on `alert-engine` would cycle. This
//! crate sits above both instead.
#![forbid(unsafe_code)]

use alert_engine::alert::{Alert, AlertSeverity, DriftCause};
use bahyway_core::ParticleState;
use story_engine::ProjectedState;

/// Weighted factors scoring a projected particle's need for steward
/// attention. Same weighted-sum shape as `enbilulu::EnbiFactors`, scoped
/// to the fields a `ProjectedState` actually has.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StorySentinelFactors {
    /// 1.0 if the projected state is Dead (the ground/Gray state).
    pub dead_state: f32,
    /// 1.0 if the projected state is Fuzzy (uncertain, not yet settled).
    pub fuzzy_state: f32,
    /// 1.0 if an Identity-Kaki projected with zero replayed events — an
    /// orphan identity with no recorded history at all.
    pub orphan_identity: f32,
}

/// Dead outranks an orphan identity outranks Fuzzy. Dead alone (0.6) is
/// enough to cross the HighPriority band by itself; the other two only
/// escalate as far as Watch unless they co-occur.
pub const STORY_SENTINEL_WEIGHTS: [f32; 3] = [0.6, 0.15, 0.25];

impl StorySentinelFactors {
    fn as_weighted_array(&self) -> [f32; 3] {
        [self.dead_state, self.fuzzy_state, self.orphan_identity]
    }
}

/// Derive the sentinel factors directly from a StoryEngine projection.
pub fn derive_factors(projected: &ProjectedState) -> StorySentinelFactors {
    StorySentinelFactors {
        dead_state: if projected.state == ParticleState::Dead {
            1.0
        } else {
            0.0
        },
        fuzzy_state: if projected.state == ParticleState::Fuzzy {
            1.0
        } else {
            0.0
        },
        orphan_identity: if projected.events_seen == 0 { 1.0 } else { 0.0 },
    }
}

/// φ_story — the weighted attention score, on a 0..100 scale (same scale
/// convention as `enbilulu::phi_enbi`).
pub fn phi_story(f: &StorySentinelFactors) -> f32 {
    f.as_weighted_array()
        .iter()
        .zip(STORY_SENTINEL_WEIGHTS.iter())
        .map(|(v, w)| v * w * 100.0)
        .sum()
}

/// The banding StoryEngine uses to decide whether a finding is worth
/// escalating to the Data Steward, and at what severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryBand {
    Nominal,
    Watch,
    HighPriority,
}

impl StoryBand {
    pub fn from_phi(phi: f32) -> Self {
        if phi < 30.0 {
            StoryBand::Nominal
        } else if phi < 60.0 {
            StoryBand::Watch
        } else {
            StoryBand::HighPriority
        }
    }

    /// Bridge to `alert_engine::AlertSeverity` — `None` means "no alert,"
    /// same convention as `TiamatBand::to_alert_severity`.
    pub fn to_alert_severity(self) -> Option<AlertSeverity> {
        match self {
            StoryBand::Nominal => None,
            StoryBand::Watch => Some(AlertSeverity::Watch),
            StoryBand::HighPriority => Some(AlertSeverity::Critical),
        }
    }
}

/// Dominant-factor cause lookup — same shape as `enbilulu::diagnose`'s
/// `TERTU_TABLE`, sized to this module's three factors.
fn dominant_cause(f: &StorySentinelFactors) -> DriftCause {
    let candidates = [
        (f.dead_state, DriftCause::FreshnessDecay),
        (f.fuzzy_state, DriftCause::Unknown),
        (f.orphan_identity, DriftCause::Unknown),
    ];
    candidates
        .into_iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, cause)| cause)
        .unwrap_or(DriftCause::Unknown)
}

/// A human-readable narration for one finding — for NISABA's hologram
/// panel and for the steward's own review queue, same role as
/// `enbilulu::narrate`.
pub fn narrate(
    uuid_hash: u32,
    factors: &StorySentinelFactors,
    phi: f32,
    band: StoryBand,
) -> String {
    let reason = if factors.dead_state > 0.0 {
        "projected to DEAD (ground) state"
    } else if factors.orphan_identity > 0.0 {
        "Identity-Kaki with zero replayed events (orphan identity)"
    } else if factors.fuzzy_state > 0.0 {
        "projected to FUZZY (uncertain) state"
    } else {
        "nominal"
    };
    format!(
        "Particle {uuid_hash:#010x}: {reason}, phi_story={phi:.1}, band={band:?} — sent to the Data Steward for review."
    )
}

/// One StoryEngine sentinel finding: the alert to hand to
/// `data-steward-station::StewardStation::receive`, plus the narration
/// text for a human (or NISABA's hologram) to read.
#[derive(Debug, Clone)]
pub struct StorySentinelFinding {
    pub alert: Alert,
    pub band: StoryBand,
    pub narration: String,
}

/// Scan one projection. Returns `None` when the particle is nominal —
/// no alert, no steward action, no narration.
pub fn scan(projected: &ProjectedState) -> Option<StorySentinelFinding> {
    let factors = derive_factors(projected);
    let phi = phi_story(&factors);
    let band = StoryBand::from_phi(phi);
    let severity = band.to_alert_severity()?;
    let cause = dominant_cause(&factors);
    let uuid_hash = projected.particle.uuid_hash();
    let alert = Alert::new(uuid_hash, severity, cause, phi);
    let narration = narrate(uuid_hash, &factors, phi, band);
    Some(StorySentinelFinding {
        alert,
        band,
        narration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};

    fn projected_with(state: ParticleState, events_seen: usize) -> ProjectedState {
        let minter = KakiMinter::new(TribeId::from_u16(0x0001));
        let identity = minter.mint_identity(0xC0FFEE, KakiRole::Zikru);
        let identity = IdentityKaki::try_from_kaki(identity).unwrap();
        let mut projected = ProjectedState::new(identity);
        projected.state = state;
        projected.events_seen = events_seen;
        projected
    }

    #[test]
    fn weights_sum_to_one() {
        let sum: f32 = STORY_SENTINEL_WEIGHTS.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "weights={STORY_SENTINEL_WEIGHTS:?}"
        );
    }

    #[test]
    fn golden_state_with_events_is_nominal_no_alert() {
        let projected = projected_with(ParticleState::Golden, 5);
        assert!(scan(&projected).is_none());
    }

    #[test]
    fn dead_state_is_high_priority_critical_alert() {
        let projected = projected_with(ParticleState::Dead, 3);
        let finding = scan(&projected).expect("dead particle must alert");
        assert_eq!(finding.band, StoryBand::HighPriority);
        assert_eq!(finding.alert.severity, AlertSeverity::Critical);
        assert!(finding.narration.contains("DEAD"));
    }

    #[test]
    fn orphan_identity_with_fuzzy_state_still_escalates() {
        // fuzzy (0.15) + orphan (0.25) = 0.40 * 100 = 40 -> Watch band.
        let projected = projected_with(ParticleState::Fuzzy, 0);
        let finding = scan(&projected).expect("orphan+fuzzy must alert");
        assert_eq!(finding.band, StoryBand::Watch);
        assert_eq!(finding.alert.severity, AlertSeverity::Watch);
    }

    #[test]
    fn band_thresholds() {
        assert_eq!(StoryBand::from_phi(0.0), StoryBand::Nominal);
        assert_eq!(StoryBand::from_phi(29.9), StoryBand::Nominal);
        assert_eq!(StoryBand::from_phi(30.0), StoryBand::Watch);
        assert_eq!(StoryBand::from_phi(59.9), StoryBand::Watch);
        assert_eq!(StoryBand::from_phi(60.0), StoryBand::HighPriority);
    }

    #[test]
    fn dead_alert_can_be_received_by_steward_station() {
        // The whole point of this bridge: the resulting Alert must be a
        // real alert_engine::Alert the existing StewardStation can queue.
        let projected = projected_with(ParticleState::Dead, 1);
        let finding = scan(&projected).unwrap();
        let mut queue: Vec<Alert> = Vec::new();
        queue.push(finding.alert);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].severity, AlertSeverity::Critical);
    }
}
