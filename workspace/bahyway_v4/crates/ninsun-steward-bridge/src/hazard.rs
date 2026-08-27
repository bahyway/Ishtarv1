//! hazard.rs — Hazard-domain urgency classification for the Steward inbox.
//! Wires ESARHADDON's real Structural Mortality Index (SMI) into the
//! generic NINSUN advisory queue's priority order.
//!
//! LAW: "priority is not high/low, priority is prediction" (DUB.SAR,
//! 2026-07-10). A case's real-world time-to-irreversibility must
//! outrank NINSUN's confidence score whenever the two conflict — a
//! DeadCritical earthquake structural case always outranks a high-
//! confidence but ordinary FUZZY business-data proposal, because the
//! cost of a late decision is not symmetric between those two domains.
//!
//! Reserved tribe namespace (new law, this module): ESARHADDON survivor
//! and structural-integrity particles use tribe_id range 0x2000-0x20FF,
//! following the precedent already set by EnkiduLLM (books: 0x1001-0x10FF,
//! conversations: 0x1200). ESARHADDON itself is a pure computation crate
//! with no tribe/KAKI integration — this range is the connecting
//! convention that lets a hazard-tagged particle be recognised here.
#![forbid(unsafe_code)]

use esarhaddon::smi::{Smi, SmiState};

pub const ESARHADDON_TRIBE_RANGE: std::ops::RangeInclusive<u16> = 0x2000..=0x20FF;

/// Which hazard domain (if any) a case belongs to, derived from its
/// tribe_id. Only Earthquake is wired today (ESARHADDON is the only real
/// hazard-domain engine); Wildfire/Cyclone/Tower are placeholders for
/// when GL-ADU-002 (currently concept-sealed only, no runtime code) is
/// actually implemented — adding a variant here is the future wiring
/// point, not a promise this module already covers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardDomain {
    None,
    Earthquake,
}

/// Parses a RefineProposal's tribe_id string (e.g. "8192" or "0x2001")
/// against the reserved hazard ranges.
pub fn classify_hazard(tribe_id: &str) -> HazardDomain {
    let parsed = if let Some(hex) = tribe_id
        .strip_prefix("0x")
        .or_else(|| tribe_id.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16).ok()
    } else {
        tribe_id.parse::<u16>().ok()
    };
    match parsed {
        Some(t) if ESARHADDON_TRIBE_RANGE.contains(&t) => HazardDomain::Earthquake,
        _ => HazardDomain::None,
    }
}

/// Combined priority key for the Steward inbox. Ord is derived so that
/// (in ascending Ord terms) higher urgency and higher confidence both
/// sort last-to-first correctly when used with `sort_by_key` + reverse,
/// or directly with `sort_by` comparing keys descending — see
/// advisory_queue.rs's use of this type.
///
/// `urgency_rank`: hazard cases rank 1 (Golden) through 4 (DeadCritical).
/// Non-hazard cases rank 0 — always below any real hazard case,
/// regardless of confidence. This is the load-bearing property: no
/// amount of NINSUN confidence on an ordinary case can outrank a real
/// structural emergency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UrgencyKey {
    pub urgency_rank: u8,
    pub confidence: f64,
}

impl UrgencyKey {
    pub fn ordinary(confidence: f64) -> Self {
        UrgencyKey {
            urgency_rank: 0,
            confidence,
        }
    }

    pub fn from_smi(smi: &Smi, confidence: f64) -> Self {
        let urgency_rank = match smi.state {
            SmiState::Golden => 1,
            SmiState::Fuzzy => 2,
            SmiState::Dead => 3,
            SmiState::DeadCritical => 4,
        };
        UrgencyKey {
            urgency_rank,
            confidence,
        }
    }
}

impl PartialOrd for UrgencyKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.urgency_rank.cmp(&other.urgency_rank) {
            std::cmp::Ordering::Equal => self.confidence.partial_cmp(&other.confidence),
            ord => Some(ord),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_hazard_recognises_esarhaddon_range() {
        assert_eq!(classify_hazard("8192"), HazardDomain::Earthquake); // 0x2000
        assert_eq!(classify_hazard("0x2050"), HazardDomain::Earthquake);
        assert_eq!(classify_hazard("8447"), HazardDomain::Earthquake); // 0x20FF
    }

    #[test]
    fn classify_hazard_rejects_ordinary_tribes() {
        assert_eq!(classify_hazard("1"), HazardDomain::None);
        assert_eq!(classify_hazard("4096"), HazardDomain::None); // 0x1000, below range
        assert_eq!(classify_hazard("8448"), HazardDomain::None); // 0x2100, above range
        assert_eq!(classify_hazard("not-a-number"), HazardDomain::None);
    }

    #[test]
    fn dead_critical_always_outranks_ordinary_regardless_of_confidence() {
        let ordinary_high_conf = UrgencyKey::ordinary(0.99);
        let smi = Smi::compute(1.0, 0.9, 0.9, 0.9, 0.9).unwrap();
        assert_eq!(smi.state, SmiState::DeadCritical);
        let hazard_low_conf = UrgencyKey::from_smi(&smi, 0.1);

        assert!(
            hazard_low_conf > ordinary_high_conf,
            "a DeadCritical structural case at 0.1 confidence must outrank \
             an ordinary case at 0.99 confidence -- priority is prediction, \
             not confidence"
        );
    }

    #[test]
    fn within_hazard_tier_higher_smi_state_wins() {
        let golden = Smi::compute(1.0, 0.05, 0.05, 0.05, 0.05).unwrap();
        let dead_critical = Smi::compute(1.0, 0.9, 0.9, 0.9, 0.9).unwrap();
        let k_golden = UrgencyKey::from_smi(&golden, 0.9);
        let k_critical = UrgencyKey::from_smi(&dead_critical, 0.1);
        assert!(k_critical > k_golden);
    }

    #[test]
    fn within_same_urgency_tier_confidence_breaks_ties() {
        let a = UrgencyKey::ordinary(0.7);
        let b = UrgencyKey::ordinary(0.3);
        assert!(a > b);
    }
}
