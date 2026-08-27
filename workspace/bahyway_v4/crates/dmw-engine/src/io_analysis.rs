//! io_analysis.rs — Level 4: A (Viscosity / IO Friction)
//!
//! Question: "How are the tables joined and how much IO does it cause?"
//!
//! Nested Loop Join  — O(n²): each outer row scans full inner table
//! Hash Match Join   — O(n+m): build hash table once, probe per outer row
//! Merge Join        — O(n+m) when sorted, O(n log n) if sort needed
//! Cross Join        — O(n×m): pure cartesian product
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::colorid::{ColorId, LevelColorContribution};

// ── Join Types (IO-layer enum, separate from plan::JoinType) ─────────────────

/// Physical join algorithm used in Level 4 IO analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoJoinType {
    NestedLoop,
    HashJoin,
    MergeJoin,
    CrossJoin,
}

impl IoJoinType {
    /// Viscosity: 0.0 = frictionless, 1.0 = catastrophic friction.
    pub fn viscosity(self) -> f32 {
        match self {
            IoJoinType::NestedLoop => 0.8,
            IoJoinType::HashJoin => 0.1,
            IoJoinType::MergeJoin => 0.15,
            IoJoinType::CrossJoin => 1.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            IoJoinType::NestedLoop => "NestedLoop",
            IoJoinType::HashJoin => "HashJoin",
            IoJoinType::MergeJoin => "MergeJoin",
            IoJoinType::CrossJoin => "CrossJoin",
        }
    }
}

// ── Join Cost Model ───────────────────────────────────────────────────────────

/// Physical cost metrics for one join operation.
#[derive(Debug, Clone)]
pub struct JoinCostMetrics {
    pub join_type: IoJoinType,
    pub outer_rows: u64,
    pub inner_rows: u64,
    pub logical_reads: u64,
    pub physical_reads: u64,
    pub cpu_operations: u64,
    pub execution_ms: f32,
}

impl JoinCostMetrics {
    /// Compute metrics for a given join type and actual row counts.
    pub fn compute(join_type: IoJoinType, outer_rows: u64, inner_rows: u64) -> Self {
        let (logical, physical, cpu, ms) = match join_type {
            IoJoinType::NestedLoop => {
                // O(n²) — each outer row probes full inner
                let ops = outer_rows.saturating_mul(inner_rows);
                let reads = (outer_rows * 100 + inner_rows * outer_rows / 10).min(1_000_000);
                (reads, reads / 10, ops, ops as f32 / 500_000.0 * 1000.0)
            }
            IoJoinType::HashJoin => {
                // O(n+m) — build + probe once
                let ops = outer_rows + inner_rows;
                let reads = (outer_rows + inner_rows) * 2;
                (reads, 0, ops, ops as f32 / 5_000_000.0 * 1000.0)
            }
            IoJoinType::MergeJoin => {
                // O(n+m) when sorted, O(n log n) if sort needed
                let ops = outer_rows + inner_rows;
                let reads = (outer_rows + inner_rows) * 3;
                (reads, 0, ops, ops as f32 / 4_000_000.0 * 1000.0)
            }
            IoJoinType::CrossJoin => {
                // O(n×m) — pure cartesian product
                let ops = outer_rows.saturating_mul(inner_rows);
                let reads = ops;
                (reads, reads, ops, ops as f32 / 100_000.0 * 1000.0)
            }
        };
        JoinCostMetrics {
            join_type,
            outer_rows,
            inner_rows,
            logical_reads: logical,
            physical_reads: physical,
            cpu_operations: cpu,
            execution_ms: ms.min(99_999.0),
        }
    }

    /// Speedup ratio compared to another join's execution time.
    pub fn speedup_vs(&self, other: &JoinCostMetrics) -> f32 {
        if self.execution_ms > 0.0 {
            other.execution_ms / self.execution_ms
        } else {
            1.0
        }
    }
}

// ── IO Analysis ───────────────────────────────────────────────────────────────

/// Full Level 4 IO Friction analysis result.
#[derive(Debug, Clone)]
pub struct IoAnalysis {
    /// Current (legacy) join cost metrics
    pub legacy_cost: JoinCostMetrics,
    /// Optimal (sovereign) join cost metrics
    pub sovereign_cost: JoinCostMetrics,
    /// The recommended join type
    pub recommended_join: IoJoinType,
    /// Speedup from switching to recommended join
    pub speedup_factor: f32,
    /// Logical read reduction in percent
    pub read_reduction_pct: f32,
    /// Explanation of why the legacy join was chosen
    pub why_legacy_was_chosen: String,
    /// Explanation of why sovereign join is better
    pub why_sovereign_wins: String,
    /// AkkadianAOL rule reference
    pub akkadi_rule: String,
    pub color_contribution: ColorId,
}

/// Analyser for Level 4 — A (Viscosity / IO Friction).
pub struct IoAnalyser;

impl IoAnalyser {
    pub fn analyse(
        legacy_join: IoJoinType,
        outer_rows: u64,
        inner_rows: u64,
        estimated_rows_when_plan_compiled: u64,
    ) -> IoAnalysis {
        let legacy_cost = JoinCostMetrics::compute(legacy_join, outer_rows, inner_rows);

        let recommended_join = recommend_join(outer_rows, inner_rows);
        let sovereign_cost = JoinCostMetrics::compute(recommended_join, outer_rows, inner_rows);

        let speedup_factor = sovereign_cost.speedup_vs(&legacy_cost);
        let read_reduction_pct = if legacy_cost.logical_reads > 0 {
            (1.0 - sovereign_cost.logical_reads as f32 / legacy_cost.logical_reads as f32) * 100.0
        } else {
            0.0
        };

        let why_legacy = match legacy_join {
            IoJoinType::NestedLoop => format!(
                "Optimizer estimated {} row(s) from statistics. \
                 Nested Loop is optimal for tiny outer inputs. \
                 But actual rows were {} — making this O({} × {}) = {:.0} operations.",
                estimated_rows_when_plan_compiled,
                outer_rows,
                outer_rows,
                inner_rows,
                outer_rows as f64 * inner_rows as f64
            ),
            IoJoinType::CrossJoin => "CROSS JOIN has no join condition — \
                accidental cartesian product."
                .into(),
            _ => "Join type may be suboptimal for this data volume.".into(),
        };

        let why_sovereign = match recommended_join {
            IoJoinType::HashJoin => format!(
                "Hash Join reads each table ONCE: {} + {} = {} rows. \
                 Builds a hash table on smaller input, probes once per outer row. \
                 Logical reads: {} vs {} — {:.0}× reduction.",
                outer_rows,
                inner_rows,
                outer_rows + inner_rows,
                sovereign_cost.logical_reads,
                legacy_cost.logical_reads,
                speedup_factor
            ),
            IoJoinType::MergeJoin => "Merge Join on sorted inputs is O(n+m) \
                with minimal memory overhead."
                .into(),
            _ => "Recommended join reduces IO for this data volume.".into(),
        };

        let akkadi_rule = match (legacy_join, recommended_join) {
            (IoJoinType::NestedLoop, IoJoinType::HashJoin) => "loop_to_hash",
            (IoJoinType::CrossJoin, _) => "cross_join_elim",
            _ => "join_reorder",
        }
        .to_string();

        // ColorID: viscosity
        let viscosity = legacy_join.viscosity();
        let red = (viscosity * 220.0) as u8;
        let green = ((1.0 - viscosity) * 200.0 + 30.0) as u8;
        let blue = ((1.0 - viscosity) * 200.0 + 30.0) as u8;

        IoAnalysis {
            legacy_cost,
            sovereign_cost,
            recommended_join,
            speedup_factor,
            read_reduction_pct,
            why_legacy_was_chosen: why_legacy,
            why_sovereign_wins: why_sovereign,
            akkadi_rule,
            color_contribution: ColorId::new(red, green, blue),
        }
    }

    pub fn color_contribution(analysis: &IoAnalysis) -> LevelColorContribution {
        LevelColorContribution::new(
            "L4 A (Viscosity)",
            analysis.color_contribution,
            &format!(
                "{} join — {:.0}× speedup available with {}",
                analysis.legacy_cost.join_type.label(),
                analysis.speedup_factor,
                analysis.recommended_join.label(),
            ),
        )
    }
}

fn recommend_join(outer: u64, inner: u64) -> IoJoinType {
    let total = outer + inner;
    if outer < 100 {
        IoJoinType::NestedLoop // tiny outer — nested loop is fine
    } else if total > 10_000 {
        IoJoinType::HashJoin // large inputs — hash join
    } else {
        IoJoinType::MergeJoin // medium — merge join if sorted
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_loop_catastrophic_at_scale() {
        let m = JoinCostMetrics::compute(IoJoinType::NestedLoop, 6069, 6069);
        assert!(
            m.logical_reads > 100_000,
            "nested loop at 6K×6K should produce >100K logical reads"
        );
    }

    #[test]
    fn hash_join_far_fewer_reads_than_nested_loop() {
        let nl = JoinCostMetrics::compute(IoJoinType::NestedLoop, 6069, 6069);
        let hash = JoinCostMetrics::compute(IoJoinType::HashJoin, 6069, 6069);
        assert!(
            hash.logical_reads < nl.logical_reads / 10,
            "hash join reads should be >10× fewer than nested loop"
        );
    }

    #[test]
    fn demo_scenario_recommends_hash_join() {
        let a = IoAnalyser::analyse(IoJoinType::NestedLoop, 6069, 6069, 1);
        assert_eq!(a.recommended_join, IoJoinType::HashJoin);
    }

    #[test]
    fn speedup_is_substantial_for_demo() {
        let a = IoAnalyser::analyse(IoJoinType::NestedLoop, 6069, 6069, 1);
        assert!(
            a.speedup_factor > 3.0,
            "hash join speedup should be >3× for 6K row join"
        );
    }

    #[test]
    fn read_reduction_pct_over_90() {
        let a = IoAnalyser::analyse(IoJoinType::NestedLoop, 6069, 6069, 1);
        assert!(
            a.read_reduction_pct > 90.0,
            "switching to hash join should reduce reads by >90%"
        );
    }

    #[test]
    fn akkadi_rule_loop_to_hash() {
        let a = IoAnalyser::analyse(IoJoinType::NestedLoop, 6069, 6069, 1);
        assert_eq!(a.akkadi_rule, "loop_to_hash");
    }

    #[test]
    fn nested_loop_color_is_high_red() {
        let a = IoAnalyser::analyse(IoJoinType::NestedLoop, 6069, 6069, 1);
        assert!(a.color_contribution.red > 150);
    }

    #[test]
    fn viscosity_ordering() {
        assert!(IoJoinType::CrossJoin.viscosity() > IoJoinType::NestedLoop.viscosity());
        assert!(IoJoinType::HashJoin.viscosity() < IoJoinType::MergeJoin.viscosity());
        assert!(IoJoinType::HashJoin.viscosity() < 0.5);
    }

    #[test]
    fn tiny_outer_recommends_nested_loop() {
        let a = IoAnalyser::analyse(IoJoinType::HashJoin, 50, 10_000, 50);
        assert_eq!(a.recommended_join, IoJoinType::NestedLoop);
    }

    #[test]
    fn cross_join_akkadi_rule() {
        let a = IoAnalyser::analyse(IoJoinType::CrossJoin, 1000, 1000, 1000);
        assert_eq!(a.akkadi_rule, "cross_join_elim");
    }
}
