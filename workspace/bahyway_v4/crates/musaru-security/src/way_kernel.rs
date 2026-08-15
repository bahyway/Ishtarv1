//! WAY v0.2 kernel types — Maṣṣārūtu (𒈦𒍝𒊒𒌅), the art of the watchman.
//!
//! This module is the minimum Rust surface required by WAY v0.2.
//! It composes onto Ṭupšarrūtu v4.0 via trait extension — no kernel sort
//! is redefined here.
//!
//! Sealed spec dependencies: Ṭupšarrūtu v4.0 (kernel), WAY v0.2 §1–7.

use bahyway_core::lane::Lane;

// ── Metric Constants (§5) ──────────────────────────────────────────────────────

/// W4 cross-tribe affinity gate — propagation requires ≥ this cosine similarity.
pub const ALPHA_CROSS: f32 = 0.85;

/// W2 default capability propagation radius — max topology distance (hops).
pub const TAU_RADIUS: u32 = 3;

/// Orbit stability gate — orbits above this entropy are flagged unstable.
/// Range [0, log₂ 3]; log₂ 3 ≈ 1.585.
pub const EPSILON_UNSTABLE: f32 = 1.20;

/// Runtime rate-limit gate — propagation throttled when pressure ≥ this.
pub const PI_THROTTLE: f32 = 0.80;

/// W2 minimum trust at destination — propagation fails below this value.
pub const TAU_MIN: f32 = 0.10;

/// Default decay per hop (δ = 0.05 → ~45 hops before trust falls below τ_min).
pub const DEFAULT_DECAY: f32 = 0.05;

/// Default sliding window for pressure computation (seconds).
pub const PRESSURE_WINDOW_SECS: u32 = 60;

// ── Capability (§2) ───────────────────────────────────────────────────────────

/// Capability identity — wraps a KAKI-sized 16-byte identifier (C5: caps are particles).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CapabilityId(pub [u8; 16]);

impl CapabilityId {
    pub fn new(bytes: [u8; 16]) -> Self { Self(bytes) }
    pub fn bytes(&self) -> &[u8; 16] { &self.0 }
}

/// Atomic authority unit — bearer of an action over a scope (§2.1).
#[derive(Clone, Debug)]
pub struct Capability {
    /// Capability identity (KAKI-equivalent, C5: this is a particle).
    pub cid: CapabilityId,
    /// The action this capability grants (e.g. "telemetry.read", "vault.decrypt").
    pub action: Action,
    /// The tribe or orbit scope this capability applies to.
    pub scope: Scope,
    /// Maximum topology distance the capability may propagate. `u32::MAX` = ∞.
    pub radius: u32,
    /// Trust attenuation per hop ∈ [0.0, 1.0]. 0.0 = no decay.
    pub decay: f32,
    /// If true, every propagation must invoke `validate` (W7).
    pub requires_validation: bool,
}

impl Capability {
    /// Effective trust at a destination reached via a path of `hops` steps.
    ///
    /// Formula (C4): `trust_at = (1 − decay)^(hops)`.
    /// At origin (hops = 0) this is 1.0 (C3).
    pub fn trust_at_hops(&self, hops: u32) -> f32 {
        if hops == 0 { return 1.0; }
        (1.0 - self.decay).powi(hops as i32)
    }
}

/// Symbolic action name — the algebra commits only to its existence, not a registry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Action(pub String);

impl Action {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Orbit or Tribe scope for a capability.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Capability applies to an entire tribe (identified by u32 ID).
    Tribe(u32),
    /// Capability applies to a specific orbit (identified by 16-byte KAKI).
    Orbit([u8; 16]),
}

/// Ordered sequence of orbit KAKIs a capability traverses (§1.2, sort 𝒫𝒫).
#[derive(Clone, Debug, Default)]
pub struct PropagationPath(pub Vec<[u8; 16]>);

impl PropagationPath {
    pub fn new() -> Self { Self(Vec::new()) }
    pub fn with_orbits(orbits: Vec<[u8; 16]>) -> Self { Self(orbits) }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
    /// Number of hops = len - 1 (hops between adjacent orbits, not the count of orbits).
    pub fn hops(&self) -> u32 { self.0.len().saturating_sub(1) as u32 }
}

// ── ValidationResult (§1.2) ───────────────────────────────────────────────────

/// Result of `validate(c, path)` — W7 pluggable validation (§4.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationResult {
    /// Path is geometrically continuous — propagation may proceed.
    Continuous,
    /// Path has a geometric discontinuity — propagation denied.
    Discontinuous(String),
    /// Insufficient data to determine continuity — propagation denied by default.
    Inconclusive,
}

impl ValidationResult {
    pub fn is_continuous(&self) -> bool { *self == ValidationResult::Continuous }
}

// ── TrustState (§3) ───────────────────────────────────────────────────────────

/// Security posture of a SecurityOrbit — per-orbit, orthogonal to `Lane` (§1.2).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TrustState {
    /// Full capability enforcement; propagation allowed within radius.
    Sealed,
    /// Propagation bounded by radius + decay constraints.
    Bounded,
    /// Admin-only write access; all external propagation blocked.
    Isolated,
    /// Read-only observation; no mutations permitted.
    Observational,
    /// Telemetry-aware; pressure monitoring active.
    Adaptive,
    /// Bidirectional pairing with another orbit (TS3: symmetric and idempotent).
    Entangled,
    /// Time-bound; reverts after expiry.
    Transient,
    /// Auto-quarantined on W4 failure or policy trigger; terminal until admin action.
    Quarantined,
}

impl TrustState {
    /// Returns `true` if this TrustState permits outbound propagation.
    pub fn allows_propagation(self) -> bool {
        !matches!(self, TrustState::Quarantined | TrustState::Isolated | TrustState::Observational)
    }

    /// Returns `true` if this is a terminal state (requires out-of-band admin recovery).
    pub fn is_terminal(self) -> bool {
        self == TrustState::Quarantined
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TrustState::Sealed        => "SEALED",
            TrustState::Bounded       => "BOUNDED",
            TrustState::Isolated      => "ISOLATED",
            TrustState::Observational => "OBSERVATIONAL",
            TrustState::Adaptive      => "ADAPTIVE",
            TrustState::Entangled     => "ENTANGLED",
            TrustState::Transient     => "TRANSIENT",
            TrustState::Quarantined   => "QUARANTINED",
        }
    }
}

impl core::fmt::Display for TrustState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── SealState (§3) ────────────────────────────────────────────────────────────

/// Seal lifecycle of a SecurityOrbit — progresses upward, not downward (except unseal).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SealState {
    /// Default state for new orbits — mutations and capability grants permitted.
    Mutable   = 0,
    /// One step below sealed — write access requires explicit capability.
    Guarded   = 1,
    /// Member set, capability set, and trust state frozen. Only `unseal` can exit.
    Sealed    = 2,
    /// Terminal — cannot be unsealed by any algebra operation (TS4).
    Frozen    = 3,
}

impl SealState {
    /// Returns `true` when the orbit is immutable (Sealed or Frozen).
    pub fn is_immutable(self) -> bool {
        matches!(self, SealState::Sealed | SealState::Frozen)
    }

    /// Returns `true` when no further progression is possible (Frozen is terminal).
    pub fn is_terminal(self) -> bool { self == SealState::Frozen }

    /// Validate a requested SealState transition.  Returns `Ok(target)` if
    /// the transition is legal, or `Err(SecurityError::IllegalTrustTransition)` if not.
    ///
    /// Legal progressions: Mutable → Guarded → Sealed → Frozen.
    /// `unseal` (Sealed → Guarded) is handled by the separate `unseal_transition` fn.
    pub fn guard_transition(self) -> Result<SealState, SecurityError> {
        match self {
            SealState::Mutable => Ok(SealState::Guarded),
            _                  => Err(SecurityError::IllegalTrustTransition),
        }
    }

    pub fn seal_transition(self) -> Result<SealState, SecurityError> {
        match self {
            SealState::Guarded => Ok(SealState::Sealed),
            _                  => Err(SecurityError::IllegalTrustTransition),
        }
    }

    pub fn freeze_transition(self) -> Result<SealState, SecurityError> {
        match self {
            SealState::Sealed => Ok(SealState::Frozen),
            _                 => Err(SecurityError::IllegalTrustTransition),
        }
    }

    /// `unseal` requires Evidence (mirrors A3) and steps back Sealed → Guarded.
    pub fn unseal_transition(self) -> Result<SealState, SecurityError> {
        match self {
            SealState::Sealed => Ok(SealState::Guarded),
            SealState::Frozen => Err(SecurityError::OrbitSealed), // TS4: frozen is terminal
            _                 => Err(SecurityError::IllegalTrustTransition),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SealState::Mutable  => "MUTABLE",
            SealState::Guarded  => "GUARDED",
            SealState::Sealed   => "SEALED",
            SealState::Frozen   => "FROZEN",
        }
    }
}

impl core::fmt::Display for SealState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Error Types (§7) ──────────────────────────────────────────────────────────

/// Errors returned by SecurityOrbit mutating methods.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecurityError {
    /// W1: operation attempted without a required capability.
    CapabilityMissing,
    /// W6: orbit is sealed or frozen — mutation denied.
    OrbitSealed,
    /// W5: TrustState or SealState transition is not allowed by the state machine.
    IllegalTrustTransition,
    /// W6/TS2: `unseal` requires a fresh Evidence event (mirrors Ṭupšarrūtu A3).
    EvidenceRequired,
}

/// Errors returned by `Propagator::propagate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropagationError {
    /// W1: capability not held by source orbit.
    CapabilityMissing,
    /// P5: target orbit explicitly denies this action.
    DenyDeclared,
    /// W2: topology_distance > radius(c).
    RadiusExceeded,
    /// W2: trust_at(c, path) < τ_min.
    TrustDecayBelowFloor,
    /// W4: cross-tribe semantic_affinity < α_cross.
    SemanticAffinityBelowThreshold,
    /// W6/P4: target orbit is sealed or frozen.
    OrbitSealed,
    /// Runtime: pressure ≥ π_throttle.
    PressureExceeded,
    /// W7: `validate` returned Discontinuous or Inconclusive.
    ValidationFailed(String),
}

impl core::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SecurityError::CapabilityMissing       => write!(f, "W1: capability missing"),
            SecurityError::OrbitSealed             => write!(f, "W6: orbit sealed/frozen"),
            SecurityError::IllegalTrustTransition  => write!(f, "W5: illegal trust transition"),
            SecurityError::EvidenceRequired        => write!(f, "W6/TS2: evidence event required"),
        }
    }
}

impl core::fmt::Display for PropagationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PropagationError::CapabilityMissing               => write!(f, "W1: capability missing"),
            PropagationError::DenyDeclared                    => write!(f, "P5: deny declared"),
            PropagationError::RadiusExceeded                  => write!(f, "W2: radius exceeded"),
            PropagationError::TrustDecayBelowFloor            => write!(f, "W2: trust below τ_min"),
            PropagationError::SemanticAffinityBelowThreshold  => write!(f, "W4: affinity below α_cross"),
            PropagationError::OrbitSealed                     => write!(f, "W6: orbit sealed"),
            PropagationError::PressureExceeded                => write!(f, "runtime: pressure exceeded"),
            PropagationError::ValidationFailed(r)             => write!(f, "W7: validation failed — {r}"),
        }
    }
}

// ── Metric Functions (§5) ────────────────────────────────────────────────────

/// C4: geometric trust decay along a propagation path.
///
/// `trust_at = (1 − decay)^hops`
/// At origin (hops = 0) returns 1.0 (C3).
pub fn trust_at(decay: f32, hops: u32) -> f32 {
    if hops == 0 { return 1.0; }
    (1.0_f32 - decay.clamp(0.0, 1.0)).powi(hops as i32)
}

/// Shannon entropy of the lane distribution among orbit members (§5.3).
///
/// Input: counts of White, Black, and Gray particles in the orbit.
/// Range: [0.0, log₂ 3 ≈ 1.585].
/// `0.0` = lane-homogeneous; `≈1.58` = maximally mixed (suspicious).
///
/// Default stability gate: `EPSILON_UNSTABLE = 1.20`.
pub fn orbit_entropy_from_counts(white: usize, black: usize, gray: usize) -> f32 {
    let total = (white + black + gray) as f32;
    if total == 0.0 { return 0.0; }
    let entropy_term = |count: usize| -> f32 {
        if count == 0 { return 0.0; }
        let p = count as f32 / total;
        -p * p.log2()
    };
    entropy_term(white) + entropy_term(black) + entropy_term(gray)
}

/// Compute `orbit_entropy` from a slice of `Lane` values (§5.3).
pub fn orbit_entropy_from_lanes(lanes: &[Lane]) -> f32 {
    let white = lanes.iter().filter(|&&l| l == Lane::White).count();
    let black = lanes.iter().filter(|&&l| l == Lane::Black).count();
    let gray  = lanes.iter().filter(|&&l| l == Lane::Gray).count();
    orbit_entropy_from_counts(white, black, gray)
}

/// Returns `true` if the orbit entropy exceeds the stability threshold (§5.3).
pub fn is_orbit_unstable(entropy: f32) -> bool {
    entropy > EPSILON_UNSTABLE
}

/// Propagation pre-check for W2: returns the appropriate error if either the
/// radius or trust floor constraint fails.
pub fn check_radius_and_trust(
    cap: &Capability,
    distance: u32,
    path_hops: u32,
) -> Result<(), PropagationError> {
    if cap.radius != u32::MAX && distance > cap.radius {
        return Err(PropagationError::RadiusExceeded);
    }
    if trust_at(cap.decay, path_hops) < TAU_MIN {
        return Err(PropagationError::TrustDecayBelowFloor);
    }
    Ok(())
}

// ── Validator Trait (§7) ──────────────────────────────────────────────────────

/// Pluggable validator for W7 — VGCA-Δ continuity is the recommended default.
pub trait Validator {
    fn validate(&self, cap: &Capability, path: &PropagationPath) -> ValidationResult;
}

/// A no-op validator that always returns `Continuous` — for tests and placeholders.
pub struct AlwaysContinuousValidator;

impl Validator for AlwaysContinuousValidator {
    fn validate(&self, _cap: &Capability, _path: &PropagationPath) -> ValidationResult {
        ValidationResult::Continuous
    }
}

/// A validator that always returns `Discontinuous` — for negative test fixtures.
pub struct AlwaysDiscontinuousValidator(pub String);

impl Validator for AlwaysDiscontinuousValidator {
    fn validate(&self, _cap: &Capability, _path: &PropagationPath) -> ValidationResult {
        ValidationResult::Discontinuous(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── trust_at ──────────────────────────────────────────────────────────────

    #[test]
    fn trust_at_origin_is_one() {
        assert_eq!(trust_at(DEFAULT_DECAY, 0), 1.0);
        assert_eq!(trust_at(0.5, 0), 1.0);
    }

    #[test]
    fn trust_at_geometric_decay() {
        let t1 = trust_at(0.1, 1);
        let t2 = trust_at(0.1, 2);
        assert!((t1 - 0.9).abs() < 1e-5);
        assert!((t2 - 0.81).abs() < 1e-5);
    }

    #[test]
    fn trust_at_no_decay() {
        for hops in 0..10 {
            assert_eq!(trust_at(0.0, hops), 1.0);
        }
    }

    #[test]
    fn trust_at_falls_below_tau_min_after_many_hops() {
        // With δ=0.05, trust_at should drop below TAU_MIN eventually
        let hops = 50;
        assert!(trust_at(DEFAULT_DECAY, hops) < TAU_MIN);
    }

    // ── orbit_entropy ─────────────────────────────────────────────────────────

    #[test]
    fn orbit_entropy_homogeneous_is_zero() {
        assert_eq!(orbit_entropy_from_counts(10, 0, 0), 0.0);
        assert_eq!(orbit_entropy_from_counts(0, 10, 0), 0.0);
        assert_eq!(orbit_entropy_from_counts(0, 0, 10), 0.0);
    }

    #[test]
    fn orbit_entropy_uniform_is_max() {
        let e = orbit_entropy_from_counts(10, 10, 10);
        // log₂(3) ≈ 1.585
        assert!((e - 1.585).abs() < 0.01, "expected ~1.585, got {e}");
    }

    #[test]
    fn orbit_entropy_empty_is_zero() {
        assert_eq!(orbit_entropy_from_counts(0, 0, 0), 0.0);
    }

    #[test]
    fn orbit_entropy_from_lanes_matches_counts() {
        let lanes = vec![Lane::White, Lane::White, Lane::Black, Lane::Gray];
        let e_lanes  = orbit_entropy_from_lanes(&lanes);
        let e_counts = orbit_entropy_from_counts(2, 1, 1);
        assert!((e_lanes - e_counts).abs() < 1e-6);
    }

    #[test]
    fn is_orbit_unstable_below_threshold() {
        let e = orbit_entropy_from_counts(10, 0, 0); // 0.0
        assert!(!is_orbit_unstable(e));
    }

    #[test]
    fn is_orbit_unstable_above_threshold() {
        let e = orbit_entropy_from_counts(10, 10, 10); // ~1.585 > 1.20
        assert!(is_orbit_unstable(e));
    }

    // ── SealState transitions ─────────────────────────────────────────────────

    #[test]
    fn seal_state_valid_progression() {
        let s = SealState::Mutable;
        let g = s.guard_transition().unwrap();
        assert_eq!(g, SealState::Guarded);
        let sealed = g.seal_transition().unwrap();
        assert_eq!(sealed, SealState::Sealed);
        let frozen = sealed.freeze_transition().unwrap();
        assert_eq!(frozen, SealState::Frozen);
    }

    #[test]
    fn seal_state_skip_step_fails() {
        assert!(SealState::Mutable.seal_transition().is_err());
        assert!(SealState::Mutable.freeze_transition().is_err());
        assert!(SealState::Guarded.freeze_transition().is_err());
    }

    #[test]
    fn unseal_sealed_to_guarded() {
        let result = SealState::Sealed.unseal_transition().unwrap();
        assert_eq!(result, SealState::Guarded);
    }

    #[test]
    fn unseal_frozen_is_error() {
        assert!(SealState::Frozen.unseal_transition().is_err());
    }

    #[test]
    fn frozen_is_terminal() {
        assert!(SealState::Frozen.is_terminal());
        assert!(!SealState::Sealed.is_terminal());
    }

    #[test]
    fn sealed_and_frozen_are_immutable() {
        assert!(SealState::Sealed.is_immutable());
        assert!(SealState::Frozen.is_immutable());
        assert!(!SealState::Mutable.is_immutable());
        assert!(!SealState::Guarded.is_immutable());
    }

    // ── TrustState ───────────────────────────────────────────────────────────

    #[test]
    fn quarantined_is_terminal() {
        assert!(TrustState::Quarantined.is_terminal());
        assert!(!TrustState::Sealed.is_terminal());
    }

    #[test]
    fn propagation_blocked_when_quarantined() {
        assert!(!TrustState::Quarantined.allows_propagation());
        assert!(!TrustState::Isolated.allows_propagation());
        assert!(!TrustState::Observational.allows_propagation());
        assert!(TrustState::Sealed.allows_propagation());
        assert!(TrustState::Bounded.allows_propagation());
    }

    // ── W2 radius + trust check ───────────────────────────────────────────────

    #[test]
    fn w2_radius_check_pass() {
        let cap = Capability {
            cid: CapabilityId::new([0; 16]),
            action: Action::new("read"),
            scope: Scope::Tribe(1),
            radius: TAU_RADIUS,
            decay: 0.0,
            requires_validation: false,
        };
        assert!(check_radius_and_trust(&cap, 2, 2).is_ok());
    }

    #[test]
    fn w2_radius_exceeded() {
        let cap = Capability {
            cid: CapabilityId::new([0; 16]),
            action: Action::new("read"),
            scope: Scope::Tribe(1),
            radius: 2,
            decay: 0.0,
            requires_validation: false,
        };
        assert_eq!(check_radius_and_trust(&cap, 3, 3), Err(PropagationError::RadiusExceeded));
    }

    #[test]
    fn w2_trust_floor_exceeded() {
        let cap = Capability {
            cid: CapabilityId::new([0; 16]),
            action: Action::new("read"),
            scope: Scope::Tribe(1),
            radius: u32::MAX,
            decay: 0.5,  // heavy decay
            requires_validation: false,
        };
        // After 10 hops with 50% decay, trust = 0.5^10 ≈ 0.001 < 0.10
        assert_eq!(check_radius_and_trust(&cap, 10, 10), Err(PropagationError::TrustDecayBelowFloor));
    }

    // ── Validator trait ───────────────────────────────────────────────────────

    #[test]
    fn always_continuous_validator() {
        let v = AlwaysContinuousValidator;
        let cap = Capability {
            cid: CapabilityId::new([0; 16]),
            action: Action::new("test"),
            scope: Scope::Tribe(1),
            radius: 5, decay: 0.0, requires_validation: true,
        };
        let path = PropagationPath::new();
        assert_eq!(v.validate(&cap, &path), ValidationResult::Continuous);
    }

    #[test]
    fn always_discontinuous_validator() {
        let v = AlwaysDiscontinuousValidator("test-failure".to_string());
        let cap = Capability {
            cid: CapabilityId::new([0; 16]),
            action: Action::new("test"),
            scope: Scope::Tribe(1),
            radius: 5, decay: 0.0, requires_validation: true,
        };
        let path = PropagationPath::new();
        let result = v.validate(&cap, &path);
        assert!(!result.is_continuous());
    }

    // ── Error display ─────────────────────────────────────────────────────────

    #[test]
    fn security_error_display() {
        assert!(SecurityError::CapabilityMissing.to_string().contains("W1"));
        assert!(SecurityError::OrbitSealed.to_string().contains("W6"));
        assert!(SecurityError::IllegalTrustTransition.to_string().contains("W5"));
        assert!(SecurityError::EvidenceRequired.to_string().contains("W6"));
    }

    #[test]
    fn propagation_error_display() {
        assert!(PropagationError::RadiusExceeded.to_string().contains("W2"));
        assert!(PropagationError::SemanticAffinityBelowThreshold.to_string().contains("W4"));
        assert!(PropagationError::ValidationFailed("x".into()).to_string().contains("W7"));
    }

    // ── Constants ─────────────────────────────────────────────────────────────

    #[test]
    fn constants_in_valid_ranges() {
        assert!(ALPHA_CROSS > 0.0 && ALPHA_CROSS <= 1.0);
        assert!(PI_THROTTLE > 0.0 && PI_THROTTLE <= 1.0);
        assert!(TAU_MIN > 0.0 && TAU_MIN <= 1.0);
        assert!(DEFAULT_DECAY >= 0.0 && DEFAULT_DECAY <= 1.0);
        assert!(EPSILON_UNSTABLE < 1.586); // must be below log₂(3)
    }

    #[test]
    fn propagation_path_hops() {
        let path = PropagationPath::with_orbits(vec![[0; 16], [1; 16], [2; 16]]);
        assert_eq!(path.len(), 3);
        assert_eq!(path.hops(), 2);
        assert_eq!(PropagationPath::new().hops(), 0);
    }
}

// ── WAY STIR Pipeline (WAY v0.1 §13) ─────────────────────────────────────────

/// Stages of the WAY Security Topology IR (STIR) compilation pipeline.
///
/// A `.way` file passes through these stages before producing runtime
/// enforcement configuration consumed by Lamassu.
///
/// Pipeline: Way → Ast → Stir → OrbitSecurityGraph → CapabilityPlanner → RuntimeEnforcement
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum WayPipelineStage {
    /// Stage 1: `.way` source file parsed into an abstract syntax tree.
    Way            = 1,
    /// Stage 2: AST produced from `.way` source.
    Ast            = 2,
    /// Stage 3: Security Topology IR — normalized, platform-independent representation.
    Stir           = 3,
    /// Stage 4: STIR lowered to an Orbit Security Graph (capability adjacency).
    OrbitSecurityGraph = 4,
    /// Stage 5: Capability Planner resolves propagation paths and decay.
    CapabilityPlanner  = 5,
    /// Stage 6: Runtime Enforcement configuration emitted to Lamassu.
    RuntimeEnforcement = 6,
}

impl WayPipelineStage {
    pub fn as_str(self) -> &'static str {
        match self {
            WayPipelineStage::Way                => ".way",
            WayPipelineStage::Ast                => "AST",
            WayPipelineStage::Stir               => "STIR",
            WayPipelineStage::OrbitSecurityGraph => "OrbitSecurityGraph",
            WayPipelineStage::CapabilityPlanner  => "CapabilityPlanner",
            WayPipelineStage::RuntimeEnforcement => "RuntimeEnforcement",
        }
    }

    /// All 6 pipeline stages in execution order.
    pub fn all() -> &'static [WayPipelineStage] {
        &[
            WayPipelineStage::Way,
            WayPipelineStage::Ast,
            WayPipelineStage::Stir,
            WayPipelineStage::OrbitSecurityGraph,
            WayPipelineStage::CapabilityPlanner,
            WayPipelineStage::RuntimeEnforcement,
        ]
    }
}

impl core::fmt::Display for WayPipelineStage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod stir_tests {
    use super::*;

    #[test]
    fn pipeline_has_six_stages() {
        assert_eq!(WayPipelineStage::all().len(), 6);
    }

    #[test]
    fn stages_are_ordered() {
        let stages = WayPipelineStage::all();
        for w in stages.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn way_is_first_stage() {
        assert_eq!(WayPipelineStage::all()[0], WayPipelineStage::Way);
    }

    #[test]
    fn runtime_enforcement_is_last() {
        let stages = WayPipelineStage::all();
        assert_eq!(*stages.last().unwrap(), WayPipelineStage::RuntimeEnforcement);
    }

    #[test]
    fn display_correct() {
        assert_eq!(WayPipelineStage::Way.to_string(),  ".way");
        assert_eq!(WayPipelineStage::Stir.to_string(), "STIR");
        assert_eq!(WayPipelineStage::RuntimeEnforcement.to_string(), "RuntimeEnforcement");
    }
}
