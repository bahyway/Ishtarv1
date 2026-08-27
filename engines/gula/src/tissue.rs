//! UTTU — the tissue loom (candidate name, pending CSR-08 seal).
//! A pattern is not finished when it is recovered; it is finished when it
//! becomes TISSUE — woven into the membrane's fabric, its rigidity carried
//! as ribs, its meaning expressed as new curvature of the manifold itself.
//!
//! Maturation ladder (rigidity grows ONLY by independent engine
//! confirmation — never by time, never by assertion):
//!
//!   PHANTOM      rigidity 0.00   predicted, translucent, swaying
//!   CONDENSING   1st engine      edges harden, sway stops
//!   RECOVERED    2nd engine      two-witness rule — solid flesh
//!   TISSUE       3rd engine      woven into the membrane; four Bārûtu
//!                                rib channels (GL-NIM-003) come alive
//!
//! UTILITY GATE: only Tissue-stage patterns may claim UTILITY-WITNESSED.
//! A phantom cannot carry a train. Recovery is existence; tissue is service.

use std::collections::BTreeSet;

pub const RIGIDITY_PER_ENGINE: f64 = 0.34;
pub const TISSUE_RIBS: usize = 4; // GL-NIM-003 — four surface channels
pub const WPD_SIGMA_GATE_M: f64 = 0.10; // WPDEngine Cramér–Rao gate

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaturationStage {
    Phantom,
    Condensing,
    Recovered,
    Tissue,
}

impl MaturationStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            MaturationStage::Phantom => "PHANTOM",
            MaturationStage::Condensing => "CONDENSING",
            MaturationStage::Recovered => "RECOVERED",
            MaturationStage::Tissue => "TISSUE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TissueRecord {
    pub pattern_key: String,
    pub confirming_engines: BTreeSet<String>,
    pub rigidity: f64,
    pub stage: MaturationStage,
    /// Bārûtu rib channels woven — zero until Tissue.
    pub ribs: usize,
}

impl TissueRecord {
    pub fn phantom(pattern_key: &str) -> Self {
        Self {
            pattern_key: pattern_key.to_string(),
            confirming_engines: BTreeSet::new(),
            rigidity: 0.0,
            stage: MaturationStage::Phantom,
            ribs: 0,
        }
    }

    /// One engine's confirmation. Repeat confirmations by the SAME engine
    /// add nothing: rigidity is independence, not enthusiasm.
    pub fn confirm(&mut self, engine: &str) -> MaturationStage {
        let fresh = self.confirming_engines.insert(engine.trim().to_lowercase());
        if fresh {
            self.rigidity = (self.rigidity + RIGIDITY_PER_ENGINE).min(1.02);
        }
        self.stage = match self.confirming_engines.len() {
            0 => MaturationStage::Phantom,
            1 => MaturationStage::Condensing,
            2 => MaturationStage::Recovered,
            _ => MaturationStage::Tissue,
        };
        self.ribs = if self.stage == MaturationStage::Tissue { TISSUE_RIBS } else { 0 };
        self.stage
    }
}

/// Utility claims a Tissue-stage pattern may witness. Each is a concrete,
/// measurable service — never an assertion.
#[derive(Debug, Clone)]
pub enum UtilityKind {
    /// Transport service through the woven vault.
    Transit { passages: u64 },
    /// WPDEngine leak localization on a rib; sigma must clear the gate.
    LeakPinpoint { sigma_m: f64, rib: usize },
    /// Structural service: load cycles absorbed by the vault curvature.
    LoadHeld { cycles: u64 },
}

/// The utility gate. Refusals name their law.
pub fn witness_utility(record: &TissueRecord, kind: &UtilityKind) -> Result<String, String> {
    if record.stage != MaturationStage::Tissue {
        return Err(format!(
            "UTILITY REFUSED · {} is {} — a phantom cannot carry a train; \
             tissue admission requires three independent engines",
            record.pattern_key,
            record.stage.as_str()
        ));
    }
    match kind {
        UtilityKind::Transit { passages } => Ok(format!(
            "UTILITY-WITNESSED · {} · transit service: {} passages through the woven vault",
            record.pattern_key, passages
        )),
        UtilityKind::LeakPinpoint { sigma_m, rib } => {
            if *rib == 0 || *rib > record.ribs {
                return Err(format!(
                    "UTILITY REFUSED · rib {} does not exist ({} Bārûtu channels woven)",
                    rib, record.ribs
                ));
            }
            if *sigma_m > WPD_SIGMA_GATE_M {
                Err(format!(
                    "UTILITY REFUSED · σ_d = {:.2} m exceeds the WPDEngine Cramér–Rao \
                     gate ({:.2} m) — a vague pinpoint is not a pinpoint",
                    sigma_m, WPD_SIGMA_GATE_M
                ))
            } else {
                Ok(format!(
                    "UTILITY-WITNESSED · {} · leak localized on rib {} at σ_d = {:.2} m — \
                     within the Cramér–Rao gate",
                    record.pattern_key, rib, sigma_m
                ))
            }
        }
        UtilityKind::LoadHeld { cycles } => Ok(format!(
            "UTILITY-WITNESSED · {} · vault curvature absorbed {} load cycles — \
             the pattern IS the structure",
            record.pattern_key, cycles
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_climbs_only_on_independent_engines() {
        let mut r = TissueRecord::phantom("rupture-limit-pipe");
        assert_eq!(r.stage, MaturationStage::Phantom);
        assert_eq!(r.confirm("nanshe"), MaturationStage::Condensing);
        // same engine again: enthusiasm adds nothing
        assert_eq!(r.confirm("nanshe"), MaturationStage::Condensing);
        assert!((r.rigidity - RIGIDITY_PER_ENGINE).abs() < 1e-9);
        assert_eq!(r.confirm("wpdengine"), MaturationStage::Recovered);
        assert_eq!(r.ribs, 0, "no ribs before tissue");
        assert_eq!(r.confirm("igigi"), MaturationStage::Tissue);
        assert_eq!(r.ribs, TISSUE_RIBS, "four Bārûtu channels at tissue");
        assert!(r.rigidity >= 1.0);
    }

    #[test]
    fn a_phantom_cannot_carry_a_train() {
        let mut r = TissueRecord::phantom("k");
        r.confirm("nanshe");
        r.confirm("wpdengine"); // Recovered, not Tissue
        let v = witness_utility(&r, &UtilityKind::Transit { passages: 1 });
        assert!(v.is_err());
        assert!(v.unwrap_err().contains("cannot carry a train"));
    }

    #[test]
    fn leak_pinpoint_respects_the_cramer_rao_gate() {
        let mut r = TissueRecord::phantom("k");
        for e in ["nanshe", "wpdengine", "igigi"] {
            r.confirm(e);
        }
        assert!(witness_utility(&r, &UtilityKind::LeakPinpoint { sigma_m: 0.28, rib: 3 }).is_err());
        assert!(witness_utility(&r, &UtilityKind::LeakPinpoint { sigma_m: 0.08, rib: 3 }).is_ok());
        assert!(
            witness_utility(&r, &UtilityKind::LeakPinpoint { sigma_m: 0.08, rib: 9 }).is_err(),
            "a rib that was never woven cannot testify"
        );
    }

    #[test]
    fn tissue_witnesses_transit_and_load() {
        let mut r = TissueRecord::phantom("k");
        for e in ["a", "b", "c"] {
            r.confirm(e);
        }
        assert!(witness_utility(&r, &UtilityKind::Transit { passages: 144 }).is_ok());
        assert!(witness_utility(&r, &UtilityKind::LoadHeld { cycles: 12 }).is_ok());
    }
}
