//! QueryParticle — the sovereign particle model for a DMW query execution.
//!
//! Every query is a particle traveling through the 7-Gate heptagon.
//! Its `MotionState` describes orbital dynamics derived from alignment:
//!
//!   Laminar  — smooth, stable orbit   (alignment ≥ 80 %, Golden)
//!   Vortex   — turbulent orbit        (alignment 40–79 %, Stable or Turbulent)
//!   DeadZone — critical decay orbit   (alignment <  40 %, Critical)
//!
//! `orbit_radius` is the particle's distance from the heptagon centre:
//!   r = (1.0 - alignment_pct / 100.0) × MAX_ORBIT
//!   A perfect Golden particle has r = 0; a Critical particle approaches r = 100.
//!
//! `particle_state` bridges DMW state into the BahyWay particle lifecycle
//! (Golden / Fuzzy / Dead) so query health participates in the wider Orbit model.

use crate::colorid::ColorId;
use crate::journey::DmwState;
use bahyway_core::ParticleState;

/// Maximum orbit radius — the outer boundary of the heptagon space.
pub const MAX_ORBIT: f32 = 100.0;

/// The specific bottleneck causing a Vortex or DeadZone state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeakType {
    /// Non-sargable predicate — function on column, implicit conversion
    NonSargablePredicate,
    /// Nested loop join at scale — O(n²) complexity
    NestedLoopJoin,
    /// Physical disk fragmentation > 20%
    DiskFragmentation,
    /// Deep recursion — CTE or subquery depth > 5
    DeepRecursion,
    /// Cross join — accidental cartesian product
    CartesianProduct,
    /// Multiple leaks detected simultaneously
    Combined,
}

impl std::fmt::Display for LeakType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeakType::NonSargablePredicate => write!(f, "Non-Sargable Predicate"),
            LeakType::NestedLoopJoin => write!(f, "Nested Loop Join"),
            LeakType::DiskFragmentation => write!(f, "Disk Fragmentation"),
            LeakType::DeepRecursion => write!(f, "Deep Recursion"),
            LeakType::CartesianProduct => write!(f, "Cartesian Product"),
            LeakType::Combined => write!(f, "Combined Leaks"),
        }
    }
}

/// Orbital motion state of a DMW query particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionState {
    /// Smooth, laminar flow — query is performant and well-aligned (Golden).
    Laminar,
    /// Turbulent vortex — significant bottlenecks are deflecting the orbit.
    Vortex,
    /// Critical decay — the particle has entered the dead zone.
    DeadZone,
}

impl MotionState {
    pub fn label(self) -> &'static str {
        match self {
            MotionState::Laminar => "Laminar",
            MotionState::Vortex => "Vortex",
            MotionState::DeadZone => "Dead Zone",
        }
    }

    /// Derive motion from DMW alignment state.
    pub fn from_dmw_state(state: DmwState) -> Self {
        match state {
            DmwState::Golden => MotionState::Laminar,
            DmwState::Stable | DmwState::Turbulent => MotionState::Vortex,
            DmwState::Critical => MotionState::DeadZone,
        }
    }

    /// `true` when the particle is in stable or improving orbit.
    pub fn is_stable(self) -> bool {
        self == MotionState::Laminar
    }
}

/// A query particle — the DMW representation of a single plan execution.
///
/// Combines a ColorId fingerprint, orbital motion, BahyWay lifecycle state,
/// and alignment metrics into a single sovereign particle descriptor.
#[derive(Debug, Clone)]
pub struct QueryParticle {
    /// RGB health fingerprint.
    pub color_id: ColorId,
    /// Orbital motion state.
    pub motion: MotionState,
    /// BahyWay particle lifecycle state derived from DMW alignment.
    pub particle_state: ParticleState,
    /// Alignment score 0–100 %.
    pub alignment_pct: f32,
    /// Distance from the heptagon centre: 0 = perfect orbit, MAX_ORBIT = dead zone.
    pub orbit_radius: f32,
    /// Primary leak type driving a Vortex or DeadZone state (None when Laminar).
    pub leak: Option<LeakType>,
}

impl QueryParticle {
    /// Construct a QueryParticle from pre-computed DMW components.
    pub fn new(color_id: ColorId, dmw_state: DmwState, alignment_pct: f32) -> Self {
        let motion = MotionState::from_dmw_state(dmw_state);
        let particle_state = dmw_state_to_particle_state(dmw_state);
        let orbit_radius = (1.0 - alignment_pct / 100.0).max(0.0) * MAX_ORBIT;
        QueryParticle {
            color_id,
            motion,
            particle_state,
            alignment_pct,
            orbit_radius,
            leak: None,
        }
    }

    /// `true` when the particle holds sovereign orbit — Golden motion and Golden lifecycle.
    pub fn is_sovereign(&self) -> bool {
        self.motion == MotionState::Laminar && self.particle_state == ParticleState::Golden
    }

    /// Human-readable label combining motion state and orbit radius.
    pub fn orbit_label(&self) -> String {
        format!("{} (r = {:.1})", self.motion.label(), self.orbit_radius)
    }

    /// `true` when the particle is in the critical dead zone.
    pub fn is_dead_zone(&self) -> bool {
        self.motion == MotionState::DeadZone
    }
}

/// Map a DMW alignment state to the BahyWay particle lifecycle state.
fn dmw_state_to_particle_state(dmw: DmwState) -> ParticleState {
    match dmw {
        DmwState::Golden => ParticleState::Golden,
        DmwState::Stable => ParticleState::Fuzzy,
        DmwState::Turbulent => ParticleState::Fuzzy,
        DmwState::Critical => ParticleState::Dead,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bottleneck::BottleneckDetector;
    use crate::colorid::ColorId;
    use crate::plan::{
        sales_order_plan, simple_nested_loop_plan, OpKind, PlanNode, QueryPlan, ScanType,
    };

    fn particle_for(plan: &QueryPlan) -> QueryParticle {
        let bns = BottleneckDetector::detect(plan);
        let journey = crate::journey::SevenLevelJourney::analyze(plan, &bns);
        let color = ColorId::from_journey(plan, &journey);
        let state = journey.state();
        let pct = journey.alignment_score();
        QueryParticle::new(color, state, pct)
    }

    #[test]
    fn laminar_for_golden_state() {
        let root = PlanNode::new(
            OpKind::IndexSeek {
                name: "PK_T".into(),
                scan_type: ScanType::Clustered,
            },
            10.0,
            100,
            50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let p = particle_for(&plan);
        assert_eq!(p.motion, MotionState::Laminar);
    }

    #[test]
    fn vortex_for_nested_loop_plan() {
        let p = particle_for(&simple_nested_loop_plan());
        assert_eq!(p.motion, MotionState::Vortex);
    }

    #[test]
    fn orbit_radius_zero_for_perfect_alignment() {
        let root = PlanNode::new(
            OpKind::IndexSeek {
                name: "PK_T".into(),
                scan_type: ScanType::Clustered,
            },
            10.0,
            100,
            50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let p = particle_for(&plan);
        // Golden (≥80%) means radius is small
        assert!(
            p.orbit_radius < MAX_ORBIT * 0.25,
            "orbit_radius={}",
            p.orbit_radius
        );
    }

    #[test]
    fn orbit_radius_large_for_critical_plan() {
        // Construct a worst-case plan: nested loop over large table scans
        let inner = PlanNode::new(OpKind::TableScan { name: "B".into() }, 50.0, 100_000, 5_000);
        let outer = PlanNode::new(OpKind::TableScan { name: "A".into() }, 50.0, 100_000, 5_000);
        let mut root = PlanNode::new(
            OpKind::NestedLoopJoin(crate::plan::JoinType::Inner),
            100.0,
            100_000,
            10_000,
        );
        root.children.push(outer);
        root.children.push(inner);
        let plan = QueryPlan::new("cross", root, 100.0);
        let p = particle_for(&plan);
        // NestedLoop + TableScan → large orbit
        assert!(p.orbit_radius > 20.0, "orbit_radius={}", p.orbit_radius);
    }

    #[test]
    fn is_sovereign_only_when_golden_laminar() {
        let root = PlanNode::new(
            OpKind::IndexSeek {
                name: "PK_T".into(),
                scan_type: ScanType::Clustered,
            },
            10.0,
            100,
            50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let p = particle_for(&plan);
        assert!(p.is_sovereign());
    }

    #[test]
    fn vortex_plan_not_sovereign() {
        let p = particle_for(&simple_nested_loop_plan());
        assert!(!p.is_sovereign());
    }

    #[test]
    fn orbit_label_non_empty() {
        let p = particle_for(&sales_order_plan());
        assert!(!p.orbit_label().is_empty());
    }

    #[test]
    fn orbit_label_contains_radius() {
        let p = particle_for(&sales_order_plan());
        // label format: "Laminar (r = 5.3)" — must contain "r ="
        assert!(p.orbit_label().contains("r ="), "label={}", p.orbit_label());
    }

    #[test]
    fn particle_state_golden_for_golden_motion() {
        let root = PlanNode::new(
            OpKind::IndexSeek {
                name: "PK_T".into(),
                scan_type: ScanType::Clustered,
            },
            10.0,
            100,
            50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let p = particle_for(&plan);
        assert_eq!(p.particle_state, ParticleState::Golden);
    }

    #[test]
    fn particle_state_fuzzy_for_vortex() {
        let p = particle_for(&simple_nested_loop_plan());
        assert_eq!(p.particle_state, ParticleState::Fuzzy);
    }

    #[test]
    fn alignment_pct_in_range() {
        let p = particle_for(&sales_order_plan());
        assert!(p.alignment_pct >= 0.0 && p.alignment_pct <= 100.0);
    }

    #[test]
    fn motion_state_labels_non_empty() {
        for s in [
            MotionState::Laminar,
            MotionState::Vortex,
            MotionState::DeadZone,
        ] {
            assert!(!s.label().is_empty());
        }
    }

    #[test]
    fn is_stable_only_for_laminar() {
        assert!(MotionState::Laminar.is_stable());
        assert!(!MotionState::Vortex.is_stable());
        assert!(!MotionState::DeadZone.is_stable());
    }
}
