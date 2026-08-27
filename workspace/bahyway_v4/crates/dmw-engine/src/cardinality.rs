//! cardinality.rs — Level 2: GU (Mass / Cardinality)
//!
//! Question: "How many rows does SQL think vs. reality?"
//!
//! Wrong cardinality estimates are the #1 silent killer of query performance.
//! The optimizer CHOOSES the join algorithm based on its row estimates:
//!
//!   Estimated 1 row   → Nested Loop (cheap for small sets)
//!   Actual 6,069 rows → Nested Loop now does 6,069² = 37M ops!
//!
//! The gap between estimated and actual rows is the "Cardinality Lie."
//! Caused by: outdated statistics, non-sargable predicates, correlated columns,
//! ascending key patterns.
//!
//! ColorID contribution at Level 2:
//!   Green channel = cardinality accuracy ratio
//!   Red channel   += penalty when estimate is >10× off
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

use crate::colorid::{ColorId, LevelColorContribution};

// ── Cardinality Error Classification ─────────────────────────────────────────

/// How far off the optimizer's row estimate is from reality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalityError {
    /// Estimate within 10× of actual — optimizer is well-informed.
    Accurate,
    /// Estimate off by 10×–100× — optimizer may choose the wrong plan.
    Moderate,
    /// Estimate off by 100×–1000× — wrong plan almost certain.
    Severe,
    /// Estimate off by >1000× — catastrophic plan choice.
    Catastrophic,
}

impl CardinalityError {
    pub fn from_ratio(estimated: u64, actual: u64) -> Self {
        if actual == 0 || estimated == 0 {
            return CardinalityError::Accurate;
        }
        let ratio = (actual as f64 / estimated as f64).max(estimated as f64 / actual as f64); // always ≥ 1.0
        match ratio as u64 {
            0..=10 => CardinalityError::Accurate,
            11..=100 => CardinalityError::Moderate,
            101..=1000 => CardinalityError::Severe,
            _ => CardinalityError::Catastrophic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CardinalityError::Accurate => "Accurate",
            CardinalityError::Moderate => "Moderate skew",
            CardinalityError::Severe => "Severe skew",
            CardinalityError::Catastrophic => "Catastrophic skew",
        }
    }

    /// Red channel penalty contributed by this cardinality error.
    pub fn red_penalty(self) -> u8 {
        match self {
            CardinalityError::Accurate => 0,
            CardinalityError::Moderate => 40,
            CardinalityError::Severe => 120,
            CardinalityError::Catastrophic => 200,
        }
    }

    /// Green channel value representing cardinality accuracy.
    pub fn green_value(self) -> u8 {
        match self {
            CardinalityError::Accurate => 220,
            CardinalityError::Moderate => 160,
            CardinalityError::Severe => 80,
            CardinalityError::Catastrophic => 20,
        }
    }
}

// ── Statistics Health ─────────────────────────────────────────────────────────

/// Health of SQL Server statistics for one table.
#[derive(Debug, Clone)]
pub struct StatisticsHealth {
    pub table_name: String,
    /// Days since the last statistics update.
    pub stats_age_days: u32,
    /// Approximate row modifications since the last update (%).
    pub modification_pct: f32,
    /// Whether an ascending key pattern is detected (stats always trail reality).
    pub has_ascending_key: bool,
}

impl StatisticsHealth {
    pub fn is_stale(&self) -> bool {
        self.stats_age_days > 30 || self.modification_pct > 20.0
    }

    pub fn staleness_label(&self) -> String {
        if self.stats_age_days > 365 {
            format!("{} days old — severely outdated", self.stats_age_days)
        } else if self.stats_age_days > 90 {
            format!("{} days old — outdated", self.stats_age_days)
        } else if self.stats_age_days > 30 {
            format!("{} days old — stale", self.stats_age_days)
        } else {
            format!("{} days old — fresh", self.stats_age_days)
        }
    }

    /// The T-SQL command to refresh statistics on this table.
    pub fn fix_script(&self) -> String {
        format!("UPDATE STATISTICS [{}] WITH FULLSCAN;", self.table_name)
    }
}

// ── Cardinality Analysis ──────────────────────────────────────────────────────

/// Full Level 2 cardinality analysis result.
#[derive(Debug, Clone)]
pub struct CardinalityAnalysis {
    pub estimated_rows: u64,
    pub actual_rows: u64,
    pub error_class: CardinalityError,
    /// The exact deviation ratio (always ≥ 1.0).
    pub error_ratio: f64,
    pub stats_health: Vec<StatisticsHealth>,
    pub recommendation: String,
    pub color_contribution: ColorId,
}

impl CardinalityAnalysis {
    /// The Akkadi AOL rule name for the required fix.
    pub fn akkadi_rule(&self) -> &'static str {
        match self.error_class {
            CardinalityError::Accurate => "stats_ok",
            CardinalityError::Moderate => "stats_update_auto",
            CardinalityError::Severe => "stats_update_fullscan",
            CardinalityError::Catastrophic => "stats_rebuild_all",
        }
    }
}

/// Analyser for Level 2 — GU (Mass / Cardinality).
pub struct CardinalityAnalyser;

impl CardinalityAnalyser {
    /// Perform Level 2 cardinality analysis.
    ///
    /// `tables` — a slice of `(table_name, age_days, modification_pct)` tuples.
    pub fn analyse(
        estimated_rows: u64,
        actual_rows: u64,
        tables: &[(&str, u32, f32)],
        has_ascending_key: bool,
    ) -> CardinalityAnalysis {
        let error_class = CardinalityError::from_ratio(estimated_rows, actual_rows);

        let ratio = if actual_rows > 0 && estimated_rows > 0 {
            (actual_rows as f64 / estimated_rows as f64)
                .max(estimated_rows as f64 / actual_rows as f64)
        } else {
            1.0
        };

        let stats_health: Vec<StatisticsHealth> = tables
            .iter()
            .map(|(name, age, mod_pct)| StatisticsHealth {
                table_name: name.to_string(),
                stats_age_days: *age,
                modification_pct: *mod_pct,
                has_ascending_key,
            })
            .collect();

        let stale_count = stats_health.iter().filter(|s| s.is_stale()).count();

        let recommendation = match error_class {
            CardinalityError::Accurate => "Statistics are accurate. No action needed.".into(),
            CardinalityError::Moderate => format!(
                "Statistics moderately stale ({stale_count} tables). Run UPDATE STATISTICS."
            ),
            CardinalityError::Severe => format!(
                "Statistics severely outdated ({stale_count} tables). \
                 Run UPDATE STATISTICS WITH FULLSCAN for accurate histograms."
            ),
            CardinalityError::Catastrophic => format!(
                "CRITICAL: {ratio:.0}x cardinality error. Optimizer is choosing the WRONG join \
                 algorithm. Run UPDATE STATISTICS WITH FULLSCAN on all {stale_count} tables \
                 immediately. Consider rebuilding indexes to auto-update stats."
            ),
        };

        let red = 20u8.saturating_add(error_class.red_penalty());
        let green = error_class.green_value();

        CardinalityAnalysis {
            estimated_rows,
            actual_rows,
            error_class,
            error_ratio: ratio,
            stats_health,
            recommendation,
            color_contribution: ColorId::new(red, green, 150),
        }
    }

    pub fn color_contribution(analysis: &CardinalityAnalysis) -> LevelColorContribution {
        LevelColorContribution::new(
            "L2 GU (Mass)",
            analysis.color_contribution,
            &format!(
                "Cardinality error: {} ({:.0}x)",
                analysis.error_class.label(),
                analysis.error_ratio
            ),
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accurate_within_10x() {
        assert_eq!(
            CardinalityError::from_ratio(100, 105),
            CardinalityError::Accurate
        );
        assert_eq!(
            CardinalityError::from_ratio(100, 10),
            CardinalityError::Accurate
        );
    }

    #[test]
    fn moderate_at_50x() {
        assert_eq!(
            CardinalityError::from_ratio(1, 50),
            CardinalityError::Moderate
        );
    }

    #[test]
    fn severe_at_200x() {
        assert_eq!(
            CardinalityError::from_ratio(1, 200),
            CardinalityError::Severe
        );
    }

    #[test]
    fn catastrophic_at_1000x() {
        assert_eq!(
            CardinalityError::from_ratio(1, 6069),
            CardinalityError::Catastrophic
        );
    }

    #[test]
    fn demo_query_is_catastrophic() {
        // Optimizer estimated 1 row, actual was 6,069 — the classic DMW demo case.
        let a = CardinalityAnalyser::analyse(1, 6069, &[("SalesOrderDetail", 730, 45.0)], false);
        assert_eq!(a.error_class, CardinalityError::Catastrophic);
    }

    #[test]
    fn catastrophic_color_has_high_red() {
        let a = CardinalityAnalyser::analyse(1, 6069, &[("T", 730, 45.0)], false);
        assert!(
            a.color_contribution.red > 150,
            "red={}",
            a.color_contribution.red
        );
    }

    #[test]
    fn catastrophic_color_has_low_green() {
        let a = CardinalityAnalyser::analyse(1, 6069, &[("T", 730, 45.0)], false);
        assert!(
            a.color_contribution.green < 50,
            "green={}",
            a.color_contribution.green
        );
    }

    #[test]
    fn accurate_color_has_low_red() {
        let a = CardinalityAnalyser::analyse(100, 95, &[("T", 5, 2.0)], false);
        assert!(
            a.color_contribution.red < 50,
            "red={}",
            a.color_contribution.red
        );
    }

    #[test]
    fn stale_statistics_detected() {
        let s = StatisticsHealth {
            table_name: "T".into(),
            stats_age_days: 730,
            modification_pct: 45.0,
            has_ascending_key: false,
        };
        assert!(s.is_stale());
    }

    #[test]
    fn fresh_statistics_not_stale() {
        let s = StatisticsHealth {
            table_name: "T".into(),
            stats_age_days: 2,
            modification_pct: 1.0,
            has_ascending_key: false,
        };
        assert!(!s.is_stale());
    }

    #[test]
    fn fix_script_contains_table_name() {
        let s = StatisticsHealth {
            table_name: "SalesOrder".into(),
            stats_age_days: 100,
            modification_pct: 30.0,
            has_ascending_key: false,
        };
        assert!(s.fix_script().contains("SalesOrder"));
    }

    #[test]
    fn akkadi_rule_catastrophic() {
        let a = CardinalityAnalyser::analyse(1, 6069, &[("T", 100, 50.0)], false);
        assert_eq!(a.akkadi_rule(), "stats_rebuild_all");
    }

    #[test]
    fn akkadi_rule_accurate() {
        let a = CardinalityAnalyser::analyse(100, 98, &[("T", 5, 1.0)], false);
        assert_eq!(a.akkadi_rule(), "stats_ok");
    }

    #[test]
    fn recommendation_non_empty() {
        let a = CardinalityAnalyser::analyse(1, 6069, &[("T", 730, 45.0)], false);
        assert!(!a.recommendation.is_empty());
    }

    #[test]
    fn color_contribution_level_name() {
        let a = CardinalityAnalyser::analyse(100, 95, &[("T", 5, 1.0)], false);
        let c = CardinalityAnalyser::color_contribution(&a);
        assert!(c.level_name.contains("L2"));
    }

    #[test]
    fn error_ratio_always_gte_one() {
        let a = CardinalityAnalyser::analyse(6069, 1, &[], false);
        assert!(a.error_ratio >= 1.0, "ratio={}", a.error_ratio);
    }
}
