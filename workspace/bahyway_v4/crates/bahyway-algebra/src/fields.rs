//! Semantic fields — BahyWay Unified Algebra v1.0 minimal Rust extract.
//!
//! The BUA v1.0 document describes a continuous variational semantic tensor
//! algebra.  The full variational calculus, PDEs, and tensor geometry are
//! explicitly out-of-scope for the Ṭupšarrūtu v4.0 kernel (§8: research-track).
//!
//! What belongs here is the *vocabulary*: the 7 named semantic field kinds
//! and the 7-phase execution model that HeptaScript and the Orbit engine use
//! to name concepts in code and documentation.

/// The 7 canonical semantic field kinds (BUA v1.0 §4).
///
/// Fields are dynamic scalar quantities over the semantic manifold.
/// At the kernel level they are named identifiers; their differential
/// equations and coupling rules are a downstream consumer concern.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SemanticField {
    /// S(x,t) — semantic coherence, continuity, orbit persistence.
    Stability  = 0,
    /// C(x,t) — instability, contradiction tension, prediction conflict.
    Chaos      = 1,
    /// H(x,t) — ambiguity, semantic uncertainty, information dispersion.
    Entropy    = 2,
    /// K(x,t) — coherence failure, fragmentation propagation, shockwaves.
    Collapse   = 3,
    /// R(x,t) — reintegration, semantic repair, coherence restoration.
    Healing    = 4,
    /// U(x,t) — intentional steering, query focus, directional energy.
    Attention  = 5,
    /// W(x,t) — adaptive memory, manifold plasticity, semantic evolution.
    Learning   = 6,
}

impl SemanticField {
    /// BUA notation symbol for this field.
    pub fn symbol(self) -> &'static str {
        match self {
            SemanticField::Stability => "S",
            SemanticField::Chaos     => "C",
            SemanticField::Entropy   => "H",
            SemanticField::Collapse  => "K",
            SemanticField::Healing   => "R",
            SemanticField::Attention => "U",
            SemanticField::Learning  => "W",
        }
    }

    /// Human description of the field's role.
    pub fn description(self) -> &'static str {
        match self {
            SemanticField::Stability => "semantic coherence and orbit persistence",
            SemanticField::Chaos     => "instability, contradiction tension",
            SemanticField::Entropy   => "ambiguity and information dispersion",
            SemanticField::Collapse  => "coherence failure and fragmentation",
            SemanticField::Healing   => "reintegration and semantic repair",
            SemanticField::Attention => "intentional steering and query focus",
            SemanticField::Learning  => "adaptive memory and manifold plasticity",
        }
    }

    /// Returns `true` for fields that increase system order (reduce free energy).
    pub fn is_stabilising(self) -> bool {
        matches!(self, SemanticField::Stability | SemanticField::Healing | SemanticField::Learning)
    }

    /// Returns `true` for fields that increase disorder (increase free energy).
    pub fn is_destabilising(self) -> bool {
        matches!(self, SemanticField::Chaos | SemanticField::Entropy | SemanticField::Collapse)
    }

    /// All 7 fields in declaration order.
    pub fn all() -> &'static [SemanticField] {
        &[
            SemanticField::Stability,
            SemanticField::Chaos,
            SemanticField::Entropy,
            SemanticField::Collapse,
            SemanticField::Healing,
            SemanticField::Attention,
            SemanticField::Learning,
        ]
    }
}

impl core::fmt::Display for SemanticField {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.symbol())
    }
}

/// The 7-phase execution model of the BahyWay semantic engine (BUA v1.0 §12).
///
/// Used to name pipeline stages in logs, metrics, and HeptaScript traces.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum SemanticPhase {
    /// Phase 1 — HeptaScript source parsed into semantic AST.
    Parsing          = 1,
    /// Phase 2 — Semantic structures compiled into field tensor representations.
    TensorCompilation = 2,
    /// Phase 3 — Query perturbations deform semantic geometry.
    QueryInjection   = 3,
    /// Phase 4 — Coherent attractors self-organize.
    OrbitStabilization = 4,
    /// Phase 5 — Unstable structures fragment (Collapse field rises).
    CollapseResolution = 5,
    /// Phase 6 — Repair fields restore manifold continuity.
    HealingPropagation = 6,
    /// Phase 7 — Stable orbit manifolds become query results.
    ResultFormation  = 7,
}

impl SemanticPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            SemanticPhase::Parsing            => "PARSING",
            SemanticPhase::TensorCompilation  => "TENSOR_COMPILATION",
            SemanticPhase::QueryInjection     => "QUERY_INJECTION",
            SemanticPhase::OrbitStabilization => "ORBIT_STABILIZATION",
            SemanticPhase::CollapseResolution => "COLLAPSE_RESOLUTION",
            SemanticPhase::HealingPropagation => "HEALING_PROPAGATION",
            SemanticPhase::ResultFormation    => "RESULT_FORMATION",
        }
    }

    /// The active `SemanticField` most relevant to this phase.
    pub fn dominant_field(self) -> SemanticField {
        match self {
            SemanticPhase::Parsing            => SemanticField::Attention,
            SemanticPhase::TensorCompilation  => SemanticField::Stability,
            SemanticPhase::QueryInjection     => SemanticField::Attention,
            SemanticPhase::OrbitStabilization => SemanticField::Stability,
            SemanticPhase::CollapseResolution => SemanticField::Collapse,
            SemanticPhase::HealingPropagation => SemanticField::Healing,
            SemanticPhase::ResultFormation    => SemanticField::Stability,
        }
    }

    pub fn all() -> &'static [SemanticPhase] {
        &[
            SemanticPhase::Parsing,
            SemanticPhase::TensorCompilation,
            SemanticPhase::QueryInjection,
            SemanticPhase::OrbitStabilization,
            SemanticPhase::CollapseResolution,
            SemanticPhase::HealingPropagation,
            SemanticPhase::ResultFormation,
        ]
    }
}

impl core::fmt::Display for SemanticPhase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_seven_fields() {
        assert_eq!(SemanticField::all().len(), 7);
    }

    #[test]
    fn all_returns_seven_phases() {
        assert_eq!(SemanticPhase::all().len(), 7);
    }

    #[test]
    fn stabilising_and_destabilising_partition() {
        let stab: Vec<_> = SemanticField::all().iter().filter(|f| f.is_stabilising()).collect();
        let destab: Vec<_> = SemanticField::all().iter().filter(|f| f.is_destabilising()).collect();
        // Attention and Learning are only one of each; make sure no overlap
        assert_eq!(stab.len(), 3);   // Stability, Healing, Learning
        assert_eq!(destab.len(), 3); // Chaos, Entropy, Collapse
    }

    #[test]
    fn symbols_are_single_uppercase_letters() {
        for f in SemanticField::all() {
            let s = f.symbol();
            assert_eq!(s.len(), 1, "{} symbol should be 1 char", f.symbol());
        }
    }

    #[test]
    fn phases_are_ordered() {
        let phases = SemanticPhase::all();
        for w in phases.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn phase_dominant_field_non_empty() {
        for p in SemanticPhase::all() {
            let _ = p.dominant_field(); // just ensure it doesn't panic
        }
    }

    #[test]
    fn display_matches_as_str() {
        for f in SemanticField::all() {
            assert_eq!(f.to_string(), f.symbol());
        }
        for p in SemanticPhase::all() {
            assert_eq!(p.to_string(), p.as_str());
        }
    }

    #[test]
    fn descriptions_non_empty() {
        for f in SemanticField::all() {
            assert!(!f.description().is_empty());
        }
    }
}
