//! cpu_analysis.rs — Level 5: IZI (Heat / CPU)
//!
//! Question: "Is CPU doing unnecessary row-by-row work?"
//!
//! The highest-temperature CPU problems in SQL:
//!
//!   1. Scalar subquery in SELECT — executes once per output row.
//!      (SELECT Color FROM Products WHERE ProductID = Sl.ProductID)
//!      Called 6,069 times = 6,069 individual executions!
//!      CPU time: 4,200ms vs. 85ms after JOIN rewrite.
//!
//!   2. Scalar UDFs — same as above but worse: cannot be parallelised.
//!
//!   3. Cursor / WHILE loop — explicit row-by-row processing.
//!
//!   4. Unnecessary ORDER BY without TOP/FETCH — sorts entire result.
//!
//! ColorID contribution at Level 5:
//!   Red channel += for each row-by-row pattern detected
//!   Green channel falls when CPU time estimate is high
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

use crate::colorid::{ColorId, LevelColorContribution};

// ── CPU Pattern ───────────────────────────────────────────────────────────────

/// A row-by-row CPU anti-pattern detected in the query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuPattern {
    /// Scalar subquery in SELECT called once per output row.
    ScalarSubqueryInSelect,
    /// Scalar UDF called once per row — also prevents parallelism.
    ScalarUdf,
    /// Explicit cursor or WHILE loop — row-by-row without set-based rewrite.
    CursorOrWhileLoop,
    /// ORDER BY without TOP/FETCH — sorts entire result unnecessarily.
    UnnecessarySort,
    /// COUNT(*) on a large table without WHERE — full scan for count.
    FullScanCount,
    /// Implicit DISTINCT through GROUP BY with no aggregates.
    ImplicitDistinct,
}

impl CpuPattern {
    pub fn label(self) -> &'static str {
        match self {
            CpuPattern::ScalarSubqueryInSelect => "Scalar subquery in SELECT",
            CpuPattern::ScalarUdf => "Scalar UDF (row-by-row)",
            CpuPattern::CursorOrWhileLoop => "Cursor / WHILE loop",
            CpuPattern::UnnecessarySort => "Unnecessary ORDER BY sort",
            CpuPattern::FullScanCount => "Full scan COUNT(*)",
            CpuPattern::ImplicitDistinct => "Implicit DISTINCT via GROUP BY",
        }
    }

    /// Estimated CPU time multiplier relative to a set-based alternative.
    pub fn cpu_multiplier(self, row_count: u64) -> f32 {
        match self {
            CpuPattern::ScalarSubqueryInSelect => row_count as f32 * 0.02,
            CpuPattern::ScalarUdf => row_count as f32 * 0.05,
            CpuPattern::CursorOrWhileLoop => row_count as f32 * 0.10,
            CpuPattern::UnnecessarySort => (row_count as f32).log2() * 2.0,
            CpuPattern::FullScanCount => row_count as f32 * 0.001,
            CpuPattern::ImplicitDistinct => (row_count as f32).log2(),
        }
    }

    /// Red channel penalty contributed by this pattern.
    pub fn red_penalty(self) -> u8 {
        match self {
            CpuPattern::ScalarSubqueryInSelect => 140,
            CpuPattern::ScalarUdf => 160,
            CpuPattern::CursorOrWhileLoop => 200,
            CpuPattern::UnnecessarySort => 60,
            CpuPattern::FullScanCount => 40,
            CpuPattern::ImplicitDistinct => 50,
        }
    }
}

// ── CPU Smell ─────────────────────────────────────────────────────────────────

/// A single detected CPU anti-pattern instance with fix details.
#[derive(Debug, Clone)]
pub struct CpuSmell {
    pub pattern: CpuPattern,
    pub sql_fragment: String,
    pub executions_per_query: u64,
    pub estimated_cpu_ms: f32,
    pub fix_description: String,
    pub fix_sql: String,
    pub cpu_savings_ms: f32,
    pub akkadi_rule: String,
}

// ── CPU Analysis ──────────────────────────────────────────────────────────────

/// Full Level 5 CPU analysis result.
#[derive(Debug, Clone)]
pub struct CpuAnalysis {
    pub smells: Vec<CpuSmell>,
    pub total_cpu_ms_legacy: f32,
    pub total_cpu_ms_sovereign: f32,
    pub cpu_reduction_pct: f32,
    pub can_parallelise: bool,
    pub color_contribution: ColorId,
}

/// Analyser for Level 5 — IZI (Heat / CPU).
pub struct CpuAnalyser;

impl CpuAnalyser {
    pub fn analyse(
        row_count: u64,
        has_scalar_subquery: bool,
        has_scalar_udf: bool,
        has_cursor: bool,
        has_unnecessary_order_by: bool,
    ) -> CpuAnalysis {
        let mut smells = Vec::new();
        let mut total_ms = 0.0f32;
        let mut red = 20u8;

        if has_scalar_subquery {
            let cpu_ms = CpuPattern::ScalarSubqueryInSelect.cpu_multiplier(row_count);
            smells.push(CpuSmell {
                pattern: CpuPattern::ScalarSubqueryInSelect,
                sql_fragment: "(SELECT Color FROM Products WHERE ProductID = Sl.ProductID)".into(),
                executions_per_query: row_count,
                estimated_cpu_ms: cpu_ms,
                fix_description: "Replace scalar subquery with JOIN. \
                    The join is computed set-based (once for all rows) \
                    instead of row-by-row."
                    .into(),
                fix_sql: "JOIN Production.ProductListColors Color\n    \
                    ON Color.ProductID = Sl.ProductID"
                    .into(),
                cpu_savings_ms: cpu_ms * 0.98,
                akkadi_rule: "scalar_subquery_to_join".into(),
            });
            total_ms += cpu_ms;
            red = red.saturating_add(CpuPattern::ScalarSubqueryInSelect.red_penalty());
        }

        if has_scalar_udf {
            let cpu_ms = CpuPattern::ScalarUdf.cpu_multiplier(row_count);
            smells.push(CpuSmell {
                pattern: CpuPattern::ScalarUdf,
                sql_fragment: "dbo.fn_SomeFunction(col)".into(),
                executions_per_query: row_count,
                estimated_cpu_ms: cpu_ms,
                fix_description: "Replace scalar UDF with inline table-valued \
                    function (iTVF) or computed column. \
                    Scalar UDFs prevent parallelism."
                    .into(),
                fix_sql: "CROSS APPLY dbo.fn_iTVF(col) AS x".into(),
                cpu_savings_ms: cpu_ms * 0.90,
                akkadi_rule: "scalar_udf_to_itvf".into(),
            });
            total_ms += cpu_ms;
            red = red.saturating_add(CpuPattern::ScalarUdf.red_penalty());
        }

        if has_cursor {
            let cpu_ms = CpuPattern::CursorOrWhileLoop.cpu_multiplier(row_count);
            smells.push(CpuSmell {
                pattern: CpuPattern::CursorOrWhileLoop,
                sql_fragment: "DECLARE cursor CURSOR FOR SELECT ...".into(),
                executions_per_query: row_count,
                estimated_cpu_ms: cpu_ms,
                fix_description: "Replace cursor with set-based UPDATE/INSERT. \
                    Cursors operate one row at a time and cannot be parallelised."
                    .into(),
                fix_sql: "UPDATE target SET col = src.val\n\
                    FROM source src WHERE target.id = src.id"
                    .into(),
                cpu_savings_ms: cpu_ms * 0.95,
                akkadi_rule: "cursor_to_set_based".into(),
            });
            total_ms += cpu_ms;
            red = red.saturating_add(CpuPattern::CursorOrWhileLoop.red_penalty());
        }

        if has_unnecessary_order_by {
            let cpu_ms = CpuPattern::UnnecessarySort.cpu_multiplier(row_count);
            smells.push(CpuSmell {
                pattern: CpuPattern::UnnecessarySort,
                sql_fragment: "ORDER BY col -- no TOP or FETCH".into(),
                executions_per_query: 1,
                estimated_cpu_ms: cpu_ms,
                fix_description: "Remove ORDER BY from inner queries / CTEs. \
                    Only sort at the final output when required by the application."
                    .into(),
                fix_sql: "-- Remove ORDER BY from CTE/subquery\n\
                    -- Add only to outermost SELECT if needed"
                    .into(),
                cpu_savings_ms: cpu_ms * 0.85,
                akkadi_rule: "remove_inner_order_by".into(),
            });
            total_ms += cpu_ms;
            red = red.saturating_add(CpuPattern::UnnecessarySort.red_penalty() / 3);
        }

        let total_savings: f32 = smells.iter().map(|s| s.cpu_savings_ms).sum();
        let sovereign_ms = (total_ms - total_savings).max(1.0);
        let reduction_pct = if total_ms > 0.0 {
            (total_savings / total_ms) * 100.0
        } else {
            0.0
        };

        let can_parallelise = !has_scalar_udf && !has_cursor;

        let green: u8 = if reduction_pct > 80.0 {
            50
        } else if reduction_pct > 40.0 {
            120
        } else {
            200
        };

        CpuAnalysis {
            smells,
            total_cpu_ms_legacy: total_ms.max(1.0),
            total_cpu_ms_sovereign: sovereign_ms,
            cpu_reduction_pct: reduction_pct,
            can_parallelise,
            color_contribution: ColorId::new(red, green, 160),
        }
    }

    /// Demo analysis for the classic SalesOrder correlated subquery (6,069 rows).
    pub fn demo_analysis(row_count: u64) -> CpuAnalysis {
        Self::analyse(row_count, true, false, false, false)
    }

    pub fn color_contribution(analysis: &CpuAnalysis) -> LevelColorContribution {
        LevelColorContribution::new(
            "L5 IZI (Heat)",
            analysis.color_contribution,
            &format!(
                "{} CPU pattern(s) — {:.0}ms → {:.0}ms ({:.0}% reduction)",
                analysis.smells.len(),
                analysis.total_cpu_ms_legacy,
                analysis.total_cpu_ms_sovereign,
                analysis.cpu_reduction_pct
            ),
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_subquery_detected() {
        let a = CpuAnalyser::demo_analysis(6069);
        assert!(a
            .smells
            .iter()
            .any(|s| s.pattern == CpuPattern::ScalarSubqueryInSelect));
    }

    #[test]
    fn scalar_subquery_executes_per_row() {
        let a = CpuAnalyser::demo_analysis(6069);
        let smell = a
            .smells
            .iter()
            .find(|s| s.pattern == CpuPattern::ScalarSubqueryInSelect)
            .unwrap();
        assert_eq!(smell.executions_per_query, 6069);
    }

    #[test]
    fn cpu_reduction_over_90_percent() {
        let a = CpuAnalyser::demo_analysis(6069);
        assert!(
            a.cpu_reduction_pct > 90.0,
            "scalar subquery fix must save >90% CPU, got {:.1}%",
            a.cpu_reduction_pct
        );
    }

    #[test]
    fn cursor_prevents_parallelism() {
        let a = CpuAnalyser::analyse(1000, false, false, true, false);
        assert!(!a.can_parallelise);
    }

    #[test]
    fn scalar_udf_prevents_parallelism() {
        let a = CpuAnalyser::analyse(1000, false, true, false, false);
        assert!(!a.can_parallelise);
    }

    #[test]
    fn no_smells_clean_analysis() {
        let a = CpuAnalyser::analyse(1000, false, false, false, false);
        assert!(a.smells.is_empty());
        assert!(a.cpu_reduction_pct < 1.0);
    }

    #[test]
    fn clean_analysis_can_parallelise() {
        let a = CpuAnalyser::analyse(1000, false, false, false, false);
        assert!(a.can_parallelise);
    }

    #[test]
    fn cursor_has_highest_red_penalty() {
        assert!(
            CpuPattern::CursorOrWhileLoop.red_penalty()
                >= CpuPattern::ScalarSubqueryInSelect.red_penalty()
        );
    }

    #[test]
    fn demo_color_high_red() {
        let a = CpuAnalyser::demo_analysis(6069);
        assert!(
            a.color_contribution.red > 100,
            "red={}",
            a.color_contribution.red
        );
    }

    #[test]
    fn multiple_smells_accumulate_red() {
        let single = CpuAnalyser::analyse(1000, true, false, false, false);
        let double = CpuAnalyser::analyse(1000, true, true, false, false);
        assert!(double.color_contribution.red >= single.color_contribution.red);
    }

    #[test]
    fn fix_sql_non_empty_for_all_smells() {
        let a = CpuAnalyser::analyse(1000, true, true, true, true);
        for s in &a.smells {
            assert!(!s.fix_sql.is_empty());
        }
    }

    #[test]
    fn akkadi_rule_non_empty_for_all_smells() {
        let a = CpuAnalyser::analyse(1000, true, true, true, true);
        for s in &a.smells {
            assert!(!s.akkadi_rule.is_empty());
        }
    }

    #[test]
    fn pattern_labels_non_empty() {
        for p in [
            CpuPattern::ScalarSubqueryInSelect,
            CpuPattern::ScalarUdf,
            CpuPattern::CursorOrWhileLoop,
            CpuPattern::UnnecessarySort,
            CpuPattern::FullScanCount,
            CpuPattern::ImplicitDistinct,
        ] {
            assert!(!p.label().is_empty());
        }
    }
}
