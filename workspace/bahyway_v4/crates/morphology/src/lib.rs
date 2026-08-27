//! morphology — GL-VIZ-007 (Morphological Discovery): the mapping laws
//! (every visual = one sealed quantity) and the honesty coupling.
//! Pure Rust, zero dependencies. Zero new mathematics (A-1) -- this
//! crate is the mapping and the two honesty clauses, nothing else.
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealedQuantity {
    EdgeWeight,
    GraphRagEdge,
    EdgeAge,
    Epsilon,
    Tau,
    StateTrichotomy,
    CostAccumulation,
    ConceptCentrality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualPhenomenon {
    WalkSpeed,
    CarrierTransport,
    TrackFrayed,
    JitterAmplitude,
    MembraneThickness,
    ParticleColour,
    OutwardDrift,
    OrganelleGravity,
}

/// §1 — the mapping laws: each visual phenomenon reads exactly one
/// sealed quantity, computed by LamassuEngine at design cadence. The
/// visualization reads and renders; it never computes at frame time.
pub fn mapped_quantity(phenomenon: VisualPhenomenon) -> SealedQuantity {
    match phenomenon {
        VisualPhenomenon::WalkSpeed => SealedQuantity::EdgeWeight,
        VisualPhenomenon::CarrierTransport => SealedQuantity::GraphRagEdge,
        VisualPhenomenon::TrackFrayed => SealedQuantity::EdgeAge,
        VisualPhenomenon::JitterAmplitude => SealedQuantity::Epsilon,
        VisualPhenomenon::MembraneThickness => SealedQuantity::Tau,
        VisualPhenomenon::ParticleColour => SealedQuantity::StateTrichotomy,
        VisualPhenomenon::OutwardDrift => SealedQuantity::CostAccumulation,
        VisualPhenomenon::OrganelleGravity => SealedQuantity::ConceptCentrality,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub quantity: SealedQuantity,
    pub eav_key: String,
    pub lamassu_run_id: String,
}

/// D-2 — every visual cites its number: a `VisualProperty` cannot be
/// constructed without provenance whose quantity matches the
/// phenomenon's sealed mapping law. No unbacked motion exists.
#[derive(Debug, Clone)]
pub struct VisualProperty {
    pub phenomenon: VisualPhenomenon,
    pub value: f64,
    provenance: Provenance,
}

impl VisualProperty {
    pub fn new(phenomenon: VisualPhenomenon, value: f64, provenance: Provenance) -> Result<Self, &'static str> {
        if provenance.quantity != mapped_quantity(phenomenon) {
            return Err("provenance quantity does not match this phenomenon's sealed mapping law");
        }
        Ok(Self { phenomenon, value, provenance })
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}

/// D-1 — a pattern seen in the shape is a candidate, never a conclusion.
#[derive(Debug, Clone)]
pub struct MorphologyCandidate {
    pub description: String,
}

/// A finding exists only after confirmation by a HeptaScript query
/// against the same particles. There is no public constructor for
/// `ConfirmedFinding` other than `MorphologyCandidate::confirm_by_query`.
#[derive(Debug, Clone)]
pub struct ConfirmedFinding {
    pub description: String,
    pub query_evidence: String,
}

impl MorphologyCandidate {
    pub fn new(description: impl Into<String>) -> Self {
        Self { description: description.into() }
    }

    /// The eye discovers; the theorem certifies. Refuses without real
    /// query evidence text.
    pub fn confirm_by_query(self, query_evidence: &str) -> Result<ConfirmedFinding, &'static str> {
        if query_evidence.trim().is_empty() {
            return Err("a candidate cannot become a finding without real query evidence (D-1)");
        }
        Ok(ConfirmedFinding { description: self.description, query_evidence: query_evidence.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The mapping table (§1) is exhaustive and fixed.
    #[test]
    fn mapping_table_is_exhaustive() {
        assert_eq!(mapped_quantity(VisualPhenomenon::WalkSpeed), SealedQuantity::EdgeWeight);
        assert_eq!(mapped_quantity(VisualPhenomenon::JitterAmplitude), SealedQuantity::Epsilon);
        assert_eq!(mapped_quantity(VisualPhenomenon::MembraneThickness), SealedQuantity::Tau);
        assert_eq!(mapped_quantity(VisualPhenomenon::ParticleColour), SealedQuantity::StateTrichotomy);
        assert_eq!(mapped_quantity(VisualPhenomenon::OrganelleGravity), SealedQuantity::ConceptCentrality);
    }

    // D-2 — every visual cites its number: mismatched provenance is refused.
    #[test]
    fn d2_provenance_must_match_mapping_law() {
        let wrong = Provenance {
            quantity: SealedQuantity::Tau, // JitterAmplitude requires Epsilon, not Tau
            eav_key: "tiamat.tau".into(),
            lamassu_run_id: "run-1".into(),
        };
        assert!(VisualProperty::new(VisualPhenomenon::JitterAmplitude, 0.3, wrong).is_err());

        let right = Provenance {
            quantity: SealedQuantity::Epsilon,
            eav_key: "eav.epsilon".into(),
            lamassu_run_id: "run-1".into(),
        };
        assert!(VisualProperty::new(VisualPhenomenon::JitterAmplitude, 0.3, right).is_ok());
    }

    // D-1 — morphology proposes; algebra proves: empty evidence refused,
    // real evidence promotes a candidate to a finding.
    #[test]
    fn d1_candidate_requires_real_evidence_to_confirm() {
        let candidate = MorphologyCandidate::new("hidden dependency between Pharmacy and Laboratory");
        let refused = candidate.clone().confirm_by_query("");
        assert!(refused.is_err());
        let confirmed = candidate.confirm_by_query("ORBIT PROVE BETTI(1) IS 2 AT SCOPE TRIBE").unwrap();
        assert!(!confirmed.query_evidence.is_empty());
    }
}
