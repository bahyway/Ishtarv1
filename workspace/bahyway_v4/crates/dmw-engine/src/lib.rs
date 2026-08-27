//! dmw-engine — DeepMotionWay Sovereign SQL Advisor
//!
//! Analyses RDBMS query execution plans and storage fragmentation in pure Rust.
//! No database connection required.  Plans are described as data structures;
//! the engine identifies bottlenecks, guides through the 7-Level Journey,
//! and lets The Oracle produce ranked APPLY-ROTATION recommendations.
//!
//! # Quick start
//!
//! ```rust
//! use dmw_engine::{DmwAnalyzer, plan::sales_order_plan};
//!
//! let plan    = sales_order_plan();
//! let report  = DmwAnalyzer::analyze(&plan);
//!
//! println!("Alignment:  {:.1}%", report.alignment_pct);
//! println!("State:      {}",     report.state.label());
//! println!("Problems:   {}",     report.bottlenecks.len());
//! println!("Top fix:    {}",     report.recommendations.first().map(|r| r.title.as_str()).unwrap_or("-"));
//! ```

pub mod bottleneck;
pub mod cache_analysis;
pub mod cardinality;
pub mod colorid;
pub mod cpu_analysis;
pub mod data_simulator;
pub mod execution_plan;
pub mod fragmentation;
pub mod gray_rot;
pub mod heptagon;
pub mod index_health;
pub mod io_analysis;
pub mod journey;
pub mod level_result;
pub mod narrative;
pub mod oracle;
pub mod particle;
pub mod plan;
pub mod sargability;
pub mod set_theory;

pub use bottleneck::{Bottleneck, BottleneckDetector, BottleneckKind, Severity};
pub use cache_analysis::{CacheAnalyser, CacheAnalysis, CacheFixStrategy, SniffingRisk};
pub use cardinality::{
    CardinalityAnalyser, CardinalityAnalysis, CardinalityError, StatisticsHealth,
};
pub use colorid::{ColorId, ColorIdBuilder, LevelColorContribution};
pub use cpu_analysis::{CpuAnalyser, CpuAnalysis, CpuPattern, CpuSmell};
pub use data_simulator::{DataSimulator, ExecutionMetrics, RowStatus, SalesOrderRow};
pub use execution_plan::{PlanComparator, PlanGraph, PlanNode as GraphNode, PlanOperator};
pub use fragmentation::{FragmentationMap, FragmentationState, StorageFragment};
pub use gray_rot::{GrayRotLevel, GrayRotReport};
pub use heptagon::{HeptaDimension, HeptaWeights};
pub use index_health::{
    IndexCoverageStatus, IndexHealthAnalyser, IndexHealthAnalysis, IndexRecommendation,
};
pub use io_analysis::{IoAnalyser, IoAnalysis, IoJoinType, JoinCostMetrics};
pub use journey::{DmwState, JourneyLevelId, LevelScore, SevenLevelJourney};
pub use level_result::{AnalysisJourney, LevelResult};
pub use narrative::{gate_summary, Narrative, NarrativeBuilder};
pub use oracle::{Recommendation, RotationType, TheOracle};
pub use particle::LeakType;
pub use particle::{MotionState, QueryParticle};
pub use plan::{JoinType, OpKind, PlanNode, QueryPlan, ScanType};
pub use sargability::{
    NonSargableKind, PredicateAnalysis, SargabilityAnalyser, SargabilityAnalysis,
};
pub use set_theory::{
    SetExprNode, SetOperator, SetSmell, SetSmellKind, SetTheoryAnalyser, SetTheoryAnalysis,
    SmellSeverity,
};

/// DMW Engine version.
pub const DMW_ENGINE_VERSION: &str = "4.0.0";

/// Number of heptagram analysis levels.
pub const DMW_JOURNEY_LEVELS: usize = 7;

/// Alignment threshold for Golden state.
pub const GOLDEN_THRESHOLD: f32 = 80.0;

/// Alignment threshold below which state becomes Critical.
pub const CRITICAL_THRESHOLD: f32 = 40.0;

/// Full analysis report — single call to `DmwAnalyzer::analyze` returns this.
pub struct DmwReport {
    pub query_id: String,
    pub bottlenecks: Vec<Bottleneck>,
    pub journey: SevenLevelJourney,
    pub recommendations: Vec<Recommendation>,
    pub alignment_pct: f32,
    pub state: DmwState,
    /// Projected alignment if all recommendations are applied.
    pub projected_pct: f32,
}

impl DmwReport {
    pub fn is_golden(&self) -> bool {
        self.state == DmwState::Golden
    }
    pub fn is_critical(&self) -> bool {
        self.state == DmwState::Critical
    }
    pub fn problem_count(&self) -> usize {
        self.bottlenecks.len()
    }

    pub fn top_recommendation(&self) -> Option<&Recommendation> {
        TheOracle::top_recommendation(&self.recommendations)
    }

    /// One-line technical debug string — useful for logs and IDE status lines.
    pub fn technical_summary(&self) -> String {
        let weights = heptagon::HeptaWeights::from_journey(&self.journey);
        format!(
            "Query: {} | {:.1}% ({}) | Bottlenecks: {} | Oracle fixes: {} | \
             Projected: {:.1}% | {}",
            self.query_id,
            self.alignment_pct,
            self.state.label(),
            self.bottlenecks.len(),
            self.recommendations.len(),
            self.projected_pct,
            weights,
        )
    }
}

/// Entry point: combines bottleneck detection, journey analysis, and Oracle advice.
pub struct DmwAnalyzer;

impl DmwAnalyzer {
    pub fn analyze(plan: &QueryPlan) -> DmwReport {
        let bottlenecks = BottleneckDetector::detect(plan);
        let journey = SevenLevelJourney::analyze(plan, &bottlenecks);
        let recommendations = TheOracle::advise(plan, &bottlenecks, &journey);
        let alignment_pct = journey.alignment_score();
        let state = journey.state();
        let projected_pct = TheOracle::projected_alignment(alignment_pct, &recommendations);

        DmwReport {
            query_id: plan.query_id.clone(),
            bottlenecks,
            journey,
            recommendations,
            alignment_pct,
            state,
            projected_pct,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plan::{sales_order_plan, simple_nested_loop_plan};

    #[test]
    fn version_non_empty() {
        assert!(!DMW_ENGINE_VERSION.is_empty());
    }
    #[test]
    fn journey_levels_is_seven() {
        assert_eq!(DMW_JOURNEY_LEVELS, 7);
    }
    #[test]
    fn golden_above_critical() {
        assert!(GOLDEN_THRESHOLD > CRITICAL_THRESHOLD);
    }
    #[test]
    fn sales_order_report_structure() {
        let plan = sales_order_plan();
        let report = DmwAnalyzer::analyze(&plan);
        assert_eq!(report.query_id, "SalesOrder Correlated Subquery");
        assert!(report.alignment_pct >= 0.0 && report.alignment_pct <= 100.0);
        assert!(report.projected_pct >= report.alignment_pct);
    }
    #[test]
    fn report_problem_count_matches_bottlenecks() {
        let plan = simple_nested_loop_plan();
        let report = DmwAnalyzer::analyze(&plan);
        assert_eq!(report.problem_count(), report.bottlenecks.len());
    }
    #[test]
    fn simple_plan_not_golden() {
        let plan = simple_nested_loop_plan();
        let report = DmwAnalyzer::analyze(&plan);
        assert!(
            !report.is_golden(),
            "NestedLoop + TableScan cannot be Golden"
        );
    }
}
