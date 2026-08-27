//! # Force — Sovereign Force Generators
//!
//! Implements the three kinetic force terms that drive particle evolution:
//!
//! ```text
//! df/dt = I_phys + I_mem + I_learn
//! ```
//!
//! Each `ForceGenerator` produces a `Vec7D` force vector that the
//! `SovereignAccumulator` integrates into the particle's velocity
//! and position over one Heptagon analysis cycle (dt).
//!
//! ## Force Semantics
//!
//! Forces in 7D Heptagon space have direction:
//! - **Positive** force on a dimension → particle moves toward healthy orbit
//! - **Negative** force on a dimension → particle moves away from healthy orbit
//!
//! The three BahyWay force types:
//! - [`PhysicalForce`]  — I_phys: vault-engine Gray-Rot, IO friction, storage health
//! - [`MemoryForce`]    — I_mem:  EnkiDB cardinality staleness, plan drift
//! - [`LearningForce`]  — I_learn: Oracle Egg rewrites, AkkadianAOL rule activation
//! - [`ConstantForce`]  — utility: applies a fixed Vec7D regardless of position

use crate::vec7d::Vec7D;

// ── Dimension Enum ──────────────────────────────────────────────────────────

/// The seven sovereign dimensions of the Heptagon analytical engine.
/// Each corresponds to one sealed analysis level in DMW v1.1.0 (116 tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeptaDimension {
    /// ME 𒈨 — Intent / Set Theory
    ME,
    /// GU 𒄖 — Mass / Cardinality
    GU,
    /// SAG 𒊕 — Gravity / Selectivity
    SAG,
    /// A 𒀀 — Viscosity / IO Friction
    A,
    /// IZI 𒉈 — Heat / CPU Burn
    IZI,
    /// UD 𒌓 — Volatility / Cache / Plan Sniffing
    UD,
    /// URU 𒌷 — Strength / Index Health / Gray-Rot
    URU,
}

impl HeptaDimension {
    /// All seven dimensions, in ME..URU order — the sovereign Heptagon
    /// sequence used to key `[bool; 7]`/`[T; 7]` arrays across BahyWay
    /// (e.g. sumer-engine's `Vec7DKey::dimensions`).
    pub const ALL: [HeptaDimension; 7] = [
        HeptaDimension::ME,
        HeptaDimension::GU,
        HeptaDimension::SAG,
        HeptaDimension::A,
        HeptaDimension::IZI,
        HeptaDimension::UD,
        HeptaDimension::URU,
    ];

    /// Position in the sovereign ME..URU order (0..=6).
    pub fn index(&self) -> usize {
        match self {
            HeptaDimension::ME => 0,
            HeptaDimension::GU => 1,
            HeptaDimension::SAG => 2,
            HeptaDimension::A => 3,
            HeptaDimension::IZI => 4,
            HeptaDimension::UD => 5,
            HeptaDimension::URU => 6,
        }
    }

    /// Canonical Sumerian sign name (ME, GU, SAG, A, IZI, UD, URU).
    pub fn name(&self) -> &'static str {
        match self {
            HeptaDimension::ME => "ME",
            HeptaDimension::GU => "GU",
            HeptaDimension::SAG => "SAG",
            HeptaDimension::A => "A",
            HeptaDimension::IZI => "IZI",
            HeptaDimension::UD => "UD",
            HeptaDimension::URU => "URU",
        }
    }
}

// ── ForceGenerator Trait ────────────────────────────────────────────────────

/// A force generator — produces a `Vec7D` force from the particle's
/// current position in 7D Heptagon space.
///
/// Forces are accumulated by `SovereignAccumulator` and integrated
/// into the particle's velocity and position each analysis cycle.
pub trait ForceGenerator: Send + Sync {
    /// Compute the force Vec7D for a particle at the given position.
    fn generate(&self, position: &Vec7D) -> Vec7D;

    /// The primary Heptagon dimension this force acts on.
    fn primary_dimension(&self) -> HeptaDimension;
}

// ── I_phys — Physical Force ─────────────────────────────────────────────────

/// **I_phys** — Physical force from storage health and IO friction.
///
/// Acts primarily on:
/// - `URU` (Index Health / Gray-Rot fragmentation)
/// - `A`   (IO Friction / Join Cost)
///
/// High `gray_rot_pct` → strong negative URU force (particle pulled from healthy orbit).
/// High `io_friction`  → strong negative A force.
pub struct PhysicalForce {
    /// Fragmentation level in [0.0, 1.0] — from vault-engine Gray-Rot report
    /// DMW threshold: 0.20 (20% fragmentation = Gray-Rot onset)
    pub gray_rot_pct: f64,
    /// IO friction level in [0.0, 1.0] — from join topology analysis
    /// 1.0 = nested loop O(n²), 0.0 = ideal hash join O(n)
    pub io_friction: f64,
}

impl PhysicalForce {
    pub fn new(gray_rot_pct: f64, io_friction: f64) -> Self {
        Self {
            gray_rot_pct: gray_rot_pct.clamp(0.0, 1.0),
            io_friction: io_friction.clamp(0.0, 1.0),
        }
    }

    /// Healthy storage — no fragmentation, no IO friction
    pub fn healthy() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Gray-Rot onset — 20% fragmentation threshold
    pub fn gray_rot_onset() -> Self {
        Self::new(0.20, 0.0)
    }
}

impl ForceGenerator for PhysicalForce {
    fn generate(&self, _position: &Vec7D) -> Vec7D {
        Vec7D {
            me: 0.0,
            gu: 0.0,
            sag: 0.0,
            a: -self.io_friction,
            izi: 0.0,
            ud: 0.0,
            uru: -self.gray_rot_pct,
        }
    }

    fn primary_dimension(&self) -> HeptaDimension {
        HeptaDimension::URU
    }
}

// ── I_mem — Memory Force ────────────────────────────────────────────────────

/// **I_mem** — Memory force from cardinality staleness and plan drift.
///
/// Acts primarily on:
/// - `GU`  (Cardinality — statistics staleness)
/// - `UD`  (Cache — parameter sniffing / plan reuse errors)
pub struct MemoryForce {
    /// Statistics staleness in [0.0, 1.0]
    pub staleness: f64,
    /// Parameter sniffing severity in [0.0, 1.0]
    pub plan_drift: f64,
}

impl MemoryForce {
    pub fn new(staleness: f64, plan_drift: f64) -> Self {
        Self {
            staleness: staleness.clamp(0.0, 1.0),
            plan_drift: plan_drift.clamp(0.0, 1.0),
        }
    }

    /// Fresh statistics, no plan drift
    pub fn healthy() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl ForceGenerator for MemoryForce {
    fn generate(&self, _position: &Vec7D) -> Vec7D {
        Vec7D {
            me: 0.0,
            gu: -self.staleness,
            sag: 0.0,
            a: 0.0,
            izi: 0.0,
            ud: -self.plan_drift,
            uru: 0.0,
        }
    }

    fn primary_dimension(&self) -> HeptaDimension {
        HeptaDimension::GU
    }
}

// ── I_learn — Learning Force ────────────────────────────────────────────────

/// **I_learn** — Learning force from Oracle Egg rewrites and AkkadianAOL rules.
///
/// Acts primarily on:
/// - `ME`  (Intent / Set Theory — cartesian product elimination)
/// - `SAG` (Gravity / Selectivity — sargable predicate rewriting)
/// - `IZI` (Heat / CPU — scalar subquery elimination)
///
/// This is the **positive** force — it moves the particle toward healthy orbit.
pub struct LearningForce {
    /// Oracle improvement potential in [0.0, 1.0]
    pub oracle_improvement: f64,
    /// Sargability gain in [0.0, 1.0]
    pub sargability_gain: f64,
    /// CPU reduction potential in [0.0, 1.0]
    pub cpu_reduction: f64,
}

impl LearningForce {
    pub fn new(oracle_improvement: f64, sargability_gain: f64, cpu_reduction: f64) -> Self {
        Self {
            oracle_improvement: oracle_improvement.clamp(0.0, 1.0),
            sargability_gain: sargability_gain.clamp(0.0, 1.0),
            cpu_reduction: cpu_reduction.clamp(0.0, 1.0),
        }
    }

    /// No Oracle improvement possible
    pub fn none() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Full Oracle optimization applied
    pub fn full() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

impl ForceGenerator for LearningForce {
    fn generate(&self, _position: &Vec7D) -> Vec7D {
        Vec7D {
            me: self.oracle_improvement,
            gu: 0.0,
            sag: self.sargability_gain,
            a: 0.0,
            izi: self.cpu_reduction,
            ud: 0.0,
            uru: 0.0,
        }
    }

    fn primary_dimension(&self) -> HeptaDimension {
        HeptaDimension::ME
    }
}

// ── ConstantForce — Utility ─────────────────────────────────────────────────

/// A constant force that applies a fixed `Vec7D` regardless of particle position.
/// Useful for testing and for simple uniform field interactions.
pub struct ConstantForce {
    pub force: Vec7D,
    pub dimension: HeptaDimension,
}

impl ConstantForce {
    pub fn new(force: Vec7D, dimension: HeptaDimension) -> Self {
        Self { force, dimension }
    }
}

impl ForceGenerator for ConstantForce {
    fn generate(&self, _position: &Vec7D) -> Vec7D {
        self.force
    }

    fn primary_dimension(&self) -> HeptaDimension {
        self.dimension
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn physical_force_healthy_produces_zero_force() {
        let f = PhysicalForce::healthy();
        let force = f.generate(&Vec7D::ZERO);
        assert_eq!(force, Vec7D::ZERO);
    }

    #[test]
    fn physical_force_gray_rot_acts_on_uru_dimension() {
        let f = PhysicalForce::new(0.38, 0.0);
        let force = f.generate(&Vec7D::ZERO);
        assert!(approx(force.uru, -0.38), "uru force={}", force.uru);
        assert!(approx(force.a, 0.0));
        assert!(approx(force.me, 0.0));
    }

    #[test]
    fn physical_force_io_friction_acts_on_a_dimension() {
        let f = PhysicalForce::new(0.0, 0.95);
        let force = f.generate(&Vec7D::ZERO);
        assert!(approx(force.a, -0.95), "a force={}", force.a);
        assert!(approx(force.uru, 0.0));
    }

    #[test]
    fn physical_force_clamps_inputs_to_unit_range() {
        let f = PhysicalForce::new(2.0, -1.0);
        assert!(approx(f.gray_rot_pct, 1.0));
        assert!(approx(f.io_friction, 0.0));
    }

    #[test]
    fn physical_force_primary_dimension_is_uru() {
        assert_eq!(
            PhysicalForce::healthy().primary_dimension(),
            HeptaDimension::URU
        );
    }

    #[test]
    fn memory_force_healthy_produces_zero_force() {
        let f = MemoryForce::healthy();
        let force = f.generate(&Vec7D::ZERO);
        assert_eq!(force, Vec7D::ZERO);
    }

    #[test]
    fn memory_force_staleness_acts_on_gu_dimension() {
        let f = MemoryForce::new(0.72, 0.0);
        let force = f.generate(&Vec7D::ZERO);
        assert!(approx(force.gu, -0.72), "gu force={}", force.gu);
        assert!(approx(force.ud, 0.0));
    }

    #[test]
    fn memory_force_plan_drift_acts_on_ud_dimension() {
        let f = MemoryForce::new(0.0, 0.50);
        let force = f.generate(&Vec7D::ZERO);
        assert!(approx(force.ud, -0.50), "ud force={}", force.ud);
        assert!(approx(force.gu, 0.0));
    }

    #[test]
    fn memory_force_primary_dimension_is_gu() {
        assert_eq!(
            MemoryForce::healthy().primary_dimension(),
            HeptaDimension::GU
        );
    }

    #[test]
    fn learning_force_none_produces_zero_force() {
        let f = LearningForce::none();
        let force = f.generate(&Vec7D::ZERO);
        assert_eq!(force, Vec7D::ZERO);
    }

    #[test]
    fn learning_force_oracle_acts_on_me_dimension() {
        let f = LearningForce::new(0.85, 0.0, 0.0);
        let force = f.generate(&Vec7D::ZERO);
        assert!(approx(force.me, 0.85), "me force={}", force.me);
        assert!(approx(force.sag, 0.0));
        assert!(approx(force.izi, 0.0));
    }

    #[test]
    fn learning_force_sargability_acts_on_sag_dimension() {
        let f = LearningForce::new(0.0, 0.60, 0.0);
        let force = f.generate(&Vec7D::ZERO);
        assert!(approx(force.sag, 0.60), "sag force={}", force.sag);
    }

    #[test]
    fn learning_force_cpu_reduction_acts_on_izi_dimension() {
        let f = LearningForce::new(0.0, 0.0, 0.70);
        let force = f.generate(&Vec7D::ZERO);
        assert!(approx(force.izi, 0.70), "izi force={}", force.izi);
    }

    #[test]
    fn learning_force_positive_moves_toward_healthy_orbit() {
        let f = LearningForce::full();
        let force = f.generate(&Vec7D::ZERO);
        assert!(force.me > 0.0);
        assert!(force.sag > 0.0);
        assert!(force.izi > 0.0);
    }

    #[test]
    fn learning_force_primary_dimension_is_me() {
        assert_eq!(
            LearningForce::none().primary_dimension(),
            HeptaDimension::ME
        );
    }

    #[test]
    fn constant_force_returns_exact_force_regardless_of_position() {
        let target = Vec7D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5);
        let f = ConstantForce::new(target, HeptaDimension::ME);
        assert_eq!(f.generate(&Vec7D::ZERO), target);
        assert_eq!(f.generate(&Vec7D::UNIT), target);
    }

    #[test]
    fn constant_force_primary_dimension_matches_constructor() {
        let f = ConstantForce::new(Vec7D::ZERO, HeptaDimension::SAG);
        assert_eq!(f.primary_dimension(), HeptaDimension::SAG);
    }
}
