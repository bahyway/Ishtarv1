//! index_health.rs — Level 7: URU (Strength / Index Health)
//!
//! Question: "Are indexes covering and physically healthy?"
//!
//! Two distinct problems at Level 7:
//!
//! A. Missing or Non-Covering Index
//!    CREATE INDEX ix_Sales_Date_Cover ON SalesOrder(ModifiedDate)
//!    INCLUDE(CarrierTrackingNumber, ProductID)
//!
//! B. Physical Fragmentation (VaultWay Gray-Rot)
//!    Fragmentation ≥ 20% → ALTER INDEX ALL ON [table] REBUILD WITH (ONLINE = ON)
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::colorid::{ColorId, LevelColorContribution};
use crate::gray_rot::GrayRotReport;

// ── Index Coverage ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexCoverageStatus {
    /// Index perfectly covers all columns needed by the query.
    FullyCovering,
    /// Index covers filter column but requires Key Lookup for SELECT cols.
    PartialCovering,
    /// Index exists but is not used (wrong columns or low selectivity).
    NotUsed,
    /// No useful index exists — table scan or clustered scan.
    Missing,
}

impl IndexCoverageStatus {
    pub fn label(self) -> &'static str {
        match self {
            IndexCoverageStatus::FullyCovering => "Fully covering",
            IndexCoverageStatus::PartialCovering => "Partial (Key Lookup needed)",
            IndexCoverageStatus::NotUsed => "Exists but not used",
            IndexCoverageStatus::Missing => "Missing index",
        }
    }

    pub fn blue_value(self) -> u8 {
        match self {
            IndexCoverageStatus::FullyCovering => 220,
            IndexCoverageStatus::PartialCovering => 140,
            IndexCoverageStatus::NotUsed => 80,
            IndexCoverageStatus::Missing => 20,
        }
    }

    pub fn red_penalty(self) -> u8 {
        match self {
            IndexCoverageStatus::FullyCovering => 0,
            IndexCoverageStatus::PartialCovering => 40,
            IndexCoverageStatus::NotUsed => 80,
            IndexCoverageStatus::Missing => 160,
        }
    }
}

// ── Index Recommendation ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IndexRecommendation {
    pub table_name: String,
    pub current_status: IndexCoverageStatus,
    pub current_index: Option<String>,
    pub recommended_index: String,
    pub create_statement: String,
    pub estimated_benefit: String,
    pub key_columns: Vec<String>,
    pub include_columns: Vec<String>,
    pub akkadi_rule: String,
}

// ── Index Health Analysis ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IndexHealthAnalysis {
    pub recommendations: Vec<IndexRecommendation>,
    pub gray_rot: GrayRotReport,
    pub fragmentation_trend_weekly_pct: f32,
    pub days_until_gray_rot_threshold: Option<u32>,
    pub full_rebuild_script: String,
    pub color_contribution: ColorId,
}

impl IndexHealthAnalysis {
    /// Returns true when immediate index maintenance is required.
    pub fn requires_immediate_action(&self) -> bool {
        self.gray_rot.action_required
            || self
                .recommendations
                .iter()
                .any(|r| r.current_status == IndexCoverageStatus::Missing)
    }

    /// Combined improvement estimate from all index recommendations.
    pub fn total_estimated_improvement(&self) -> String {
        let issues = self.recommendations.len();
        let frag = self.gray_rot.fragmentation_pct;
        if issues == 0 && !self.gray_rot.action_required {
            "No index improvements needed.".into()
        } else {
            format!(
                "{} index recommendation(s) + {:.0}% fragmentation = \
                significant IO reduction expected.",
                issues, frag
            )
        }
    }
}

/// Analyser for Level 7 — URU (Strength / Index Health).
pub struct IndexHealthAnalyser;

impl IndexHealthAnalyser {
    pub fn analyse(
        table_name: &str,
        coverage_status: IndexCoverageStatus,
        fragmentation_pct: f32,
        page_reads: u64,
        filter_columns: &[&str],
        select_columns: &[&str],
        fragmentation_trend: f32, // % per week increase
    ) -> IndexHealthAnalysis {
        let gray_rot = GrayRotReport::from_plan_data(fragmentation_pct, page_reads);

        // Days until 20% threshold (only meaningful when below threshold and trending up)
        let days_until_threshold = if fragmentation_pct < 20.0 && fragmentation_trend > 0.0 {
            let weeks_remaining = (20.0 - fragmentation_pct) / fragmentation_trend;
            Some((weeks_remaining * 7.0) as u32)
        } else {
            None
        };

        // Build recommendation when index is not fully covering
        let mut recommendations = Vec::new();
        if coverage_status != IndexCoverageStatus::FullyCovering {
            let key_cols: Vec<String> = filter_columns.iter().map(|s| s.to_string()).collect();
            let inc_cols: Vec<String> = select_columns.iter().map(|s| s.to_string()).collect();

            let idx_name = format!("ix_{}_Cover", table_name.replace('.', "_"));
            let create_stmt = format!(
                "CREATE NONCLUSTERED INDEX [{}]\nON [{}] ({})\nINCLUDE ({});",
                idx_name,
                table_name,
                key_cols.join(", "),
                inc_cols.join(", "),
            );

            recommendations.push(IndexRecommendation {
                table_name: table_name.to_string(),
                current_status: coverage_status,
                current_index: None,
                recommended_index: idx_name,
                create_statement: create_stmt,
                estimated_benefit: "Eliminates Key Lookup / Table Scan — \
                    reduces logical reads by ~95%."
                    .into(),
                key_columns: key_cols,
                include_columns: inc_cols,
                akkadi_rule: "create_covering_index".into(),
            });
        }

        // Rebuild script
        let rebuild = format!(
            "-- VaultWay Gray-Rot: {:.1}% fragmentation\n\
             -- Threshold: 20% | Action: {}\n\
             ALTER INDEX ALL ON [{}] REBUILD WITH (ONLINE = ON, MAXDOP = 4);",
            fragmentation_pct, gray_rot.recommended_action, table_name
        );

        // ColorID
        let blue = coverage_status.blue_value();
        let red = 20u8
            .saturating_add(coverage_status.red_penalty())
            .saturating_add(if gray_rot.action_required { 60 } else { 0 });
        let green = if gray_rot.action_required {
            120u8
        } else {
            180u8
        };

        IndexHealthAnalysis {
            recommendations,
            gray_rot,
            fragmentation_trend_weekly_pct: fragmentation_trend,
            days_until_gray_rot_threshold: days_until_threshold,
            full_rebuild_script: rebuild,
            color_contribution: ColorId::new(red, green, blue),
        }
    }

    /// Demo analysis for SalesOrder (38% fragmentation, PartialCovering).
    pub fn demo_analysis() -> IndexHealthAnalysis {
        Self::analyse(
            "Sales.SalesOrderDetail",
            IndexCoverageStatus::PartialCovering,
            38.0,   // above 20% Gray-Rot threshold
            12_592, // page reads
            &["ModifiedDate"],
            &["CarrierTrackingNumber", "ProductID", "Color"],
            2.1, // 2.1% fragmentation growth per week
        )
    }

    pub fn color_contribution(analysis: &IndexHealthAnalysis) -> LevelColorContribution {
        LevelColorContribution::new(
            "L7 URU (Strength)",
            analysis.color_contribution,
            &format!(
                "Fragmentation: {:.0}% ({}) | {} recommendation(s)",
                analysis.gray_rot.fragmentation_pct,
                analysis.gray_rot.level,
                analysis.recommendations.len(),
            ),
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_gray_rot_above_threshold() {
        let a = IndexHealthAnalyser::demo_analysis();
        assert!(
            a.gray_rot.action_required,
            "38% fragmentation is above the 20% Gray-Rot threshold"
        );
    }

    #[test]
    fn demo_has_one_recommendation() {
        let a = IndexHealthAnalyser::demo_analysis();
        assert_eq!(a.recommendations.len(), 1);
    }

    #[test]
    fn demo_requires_immediate_action() {
        let a = IndexHealthAnalyser::demo_analysis();
        assert!(a.requires_immediate_action());
    }

    #[test]
    fn already_above_threshold_no_days_prediction() {
        let a = IndexHealthAnalyser::demo_analysis();
        // 38% already above 20% — no countdown
        assert!(a.days_until_gray_rot_threshold.is_none());
    }

    #[test]
    fn below_threshold_predicts_days() {
        let a = IndexHealthAnalyser::analyse(
            "T",
            IndexCoverageStatus::FullyCovering,
            10.0,
            1000,
            &[],
            &[],
            2.0,
        );
        // 10% now, growing 2%/week → reaches 20% in 5 weeks = 35 days
        assert!(a.days_until_gray_rot_threshold.is_some());
        let days = a.days_until_gray_rot_threshold.unwrap();
        assert!(days > 30 && days < 42, "expected ~35 days, got {days}");
    }

    #[test]
    fn missing_index_color_low_blue() {
        let a = IndexHealthAnalyser::analyse(
            "T",
            IndexCoverageStatus::Missing,
            5.0,
            500,
            &["col"],
            &["a", "b"],
            0.5,
        );
        assert!(a.color_contribution.blue < 50);
    }

    #[test]
    fn create_statement_contains_table_name() {
        let a = IndexHealthAnalyser::demo_analysis();
        let rec = &a.recommendations[0];
        assert!(rec.create_statement.contains("SalesOrderDetail"));
    }

    #[test]
    fn fully_covering_has_no_recommendation() {
        let a = IndexHealthAnalyser::analyse(
            "T",
            IndexCoverageStatus::FullyCovering,
            5.0,
            100,
            &[],
            &[],
            0.0,
        );
        assert!(a.recommendations.is_empty());
    }

    #[test]
    fn improvement_summary_for_demo() {
        let a = IndexHealthAnalyser::demo_analysis();
        let s = a.total_estimated_improvement();
        assert!(!s.is_empty());
        assert!(s.contains("recommendation"));
    }
}
