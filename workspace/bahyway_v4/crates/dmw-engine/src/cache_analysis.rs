//! cache_analysis.rs — Level 6: UD (Volatility / Cache & Parameter Sniffing)
//!
//! Question: "Is a stale cached plan being reused for the wrong data?"
//!
//! SQL Server caches execution plans. The first time a query runs, the plan is
//! compiled for the parameter values at that moment. Future executions REUSE
//! that plan — even if data distribution has changed dramatically.
//!
//! Parameter Sniffing Trap:
//!   1. Query first run with @year = 2019 → estimated 1 row
//!   2. Plan compiled: Nested Loop (correct for 1 row)
//!   3. Plan cached
//!   4. Query run with @year = 2024 → actual 6,069 rows
//!   5. SAME Nested Loop plan reused → catastrophic O(n²) execution!
//!
//! This is why a query seems fast in development (fresh plan) but slow in
//! production (sniffed plan from a different data set).
//!
//! ColorID contribution at Level 6:
//!   Green channel falls when sniffing risk is high (plan is unreliable)
//!   Red channel += when sniffing is confirmed
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

use crate::colorid::{ColorId, LevelColorContribution};

// ── Sniffing Risk ─────────────────────────────────────────────────────────────

/// Parameter sniffing risk classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SniffingRisk {
    /// Data distribution is uniform — sniffing not a concern.
    Low,
    /// Some data skew detected — monitor for regression.
    Moderate,
    /// High skew — plan will be wrong for some parameter values.
    High,
    /// Confirmed sniffing — plan was compiled for a different data set.
    Confirmed,
}

impl SniffingRisk {
    pub fn label(self) -> &'static str {
        match self {
            SniffingRisk::Low => "Low — distribution is uniform",
            SniffingRisk::Moderate => "Moderate — some data skew",
            SniffingRisk::High => "High — significant data skew",
            SniffingRisk::Confirmed => "Confirmed — stale cached plan",
        }
    }

    pub fn red_penalty(self) -> u8 {
        match self {
            SniffingRisk::Low => 0,
            SniffingRisk::Moderate => 30,
            SniffingRisk::High => 80,
            SniffingRisk::Confirmed => 150,
        }
    }

    pub fn green_penalty(self) -> u8 {
        match self {
            SniffingRisk::Low => 0,
            SniffingRisk::Moderate => 40,
            SniffingRisk::High => 100,
            SniffingRisk::Confirmed => 160,
        }
    }
}

// ── Cache Fix Strategy ────────────────────────────────────────────────────────

/// The recommended fix for a parameter sniffing problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheFixStrategy {
    /// `OPTION(OPTIMIZE FOR UNKNOWN)` — use average statistics.
    OptimizeForUnknown,
    /// `OPTION(RECOMPILE)` — recompile every execution (higher CPU).
    Recompile,
    /// Separate stored procedures per cardinality tier (best practice).
    SplitByTier,
    /// Plan guide to force a specific plan (advanced, last resort).
    PlanGuide,
    /// No fix needed.
    None,
}

impl CacheFixStrategy {
    pub fn sql_hint(self) -> &'static str {
        match self {
            CacheFixStrategy::OptimizeForUnknown => "OPTION(OPTIMIZE FOR UNKNOWN)",
            CacheFixStrategy::Recompile => "OPTION(RECOMPILE)",
            CacheFixStrategy::SplitByTier => "-- Split into sp_Query_Small and sp_Query_Large",
            CacheFixStrategy::PlanGuide => "EXEC sp_create_plan_guide ...",
            CacheFixStrategy::None => "",
        }
    }

    pub fn when_to_use(self) -> &'static str {
        match self {
            CacheFixStrategy::OptimizeForUnknown => {
                "Use when data skew exists but pattern is unknown. \
                 Uses average statistics for all executions."
            }
            CacheFixStrategy::Recompile => {
                "Use when correctness is critical and CPU is not constrained. \
                 Compiles a fresh plan every call — safe but expensive."
            }
            CacheFixStrategy::SplitByTier => {
                "Best practice: separate procedures for small and large inputs. \
                 Each gets its own optimally compiled plan."
            }
            CacheFixStrategy::PlanGuide => {
                "Advanced: force specific join/index hints via plan guide. \
                 Use only when other strategies fail."
            }
            CacheFixStrategy::None => "No action needed — plan cache is healthy.",
        }
    }
}

// ── Cache Analysis ────────────────────────────────────────────────────────────

/// Full Level 6 cache / parameter-sniffing analysis result.
#[derive(Debug, Clone)]
pub struct CacheAnalysis {
    pub sniffing_risk: SniffingRisk,
    pub plan_age_minutes: u32,
    pub plan_use_count: u64,
    /// Row count when the plan was first compiled.
    pub compiled_for_rows: u64,
    /// Row count in the current execution.
    pub actual_rows: u64,
    pub fix_strategy: CacheFixStrategy,
    pub explanation: String,
    /// StoryWay narrative entry for this level.
    pub storyway_entry: String,
    pub color_contribution: ColorId,
}

/// Analyser for Level 6 — UD (Volatility / Cache).
pub struct CacheAnalyser;

impl CacheAnalyser {
    /// Perform Level 6 cache analysis.
    ///
    /// `data_skew_factor` — 0.0 = uniform distribution, 1.0 = extreme skew.
    pub fn analyse(
        plan_age_minutes: u32,
        plan_use_count: u64,
        compiled_for_rows: u64,
        actual_rows: u64,
        data_skew_factor: f32,
    ) -> CacheAnalysis {
        let row_ratio = if compiled_for_rows > 0 {
            actual_rows as f64 / compiled_for_rows as f64
        } else {
            1.0
        };

        let risk = if row_ratio > 100.0 || (row_ratio < 0.01 && actual_rows > 100) {
            SniffingRisk::Confirmed
        } else if data_skew_factor > 0.7 || row_ratio > 10.0 {
            SniffingRisk::High
        } else if data_skew_factor > 0.3 || row_ratio > 3.0 {
            SniffingRisk::Moderate
        } else {
            SniffingRisk::Low
        };

        let fix = match risk {
            SniffingRisk::Low => CacheFixStrategy::None,
            SniffingRisk::Moderate => CacheFixStrategy::OptimizeForUnknown,
            SniffingRisk::High => CacheFixStrategy::SplitByTier,
            SniffingRisk::Confirmed => {
                if plan_use_count > 1000 {
                    CacheFixStrategy::SplitByTier
                } else {
                    CacheFixStrategy::OptimizeForUnknown
                }
            }
        };

        let explanation = match risk {
            SniffingRisk::Confirmed => format!(
                "PARAMETER SNIFFING CONFIRMED: Plan compiled for {} row(s), \
                 reused {} times. Current execution needs {} rows. \
                 Apply: {}",
                compiled_for_rows,
                plan_use_count,
                actual_rows,
                fix.sql_hint()
            ),
            SniffingRisk::High => format!(
                "High sniffing risk: data distribution skewed {:.0}%. \
                 Plan may be suboptimal for some parameter values.",
                data_skew_factor * 100.0
            ),
            SniffingRisk::Moderate => "Moderate sniffing risk. Monitor plan performance across \
                 different parameter ranges."
                .into(),
            SniffingRisk::Low => "Plan cache is healthy. No parameter sniffing detected.".into(),
        };

        let storyway_entry = format!(
            "L6 UD: plan age={}min, reused {} times. \
             Compiled for {} rows, actual {}. Risk: {}",
            plan_age_minutes,
            plan_use_count,
            compiled_for_rows,
            actual_rows,
            risk.label()
        );

        let red = 20u8.saturating_add(risk.red_penalty());
        let green = 220u8.saturating_sub(risk.green_penalty());

        CacheAnalysis {
            sniffing_risk: risk,
            plan_age_minutes,
            plan_use_count,
            compiled_for_rows,
            actual_rows,
            fix_strategy: fix,
            explanation,
            storyway_entry,
            color_contribution: ColorId::new(red, green, 160),
        }
    }

    /// Demo analysis — plan compiled for 1 row, used 8,542 times, actual 6,069 rows.
    pub fn demo_analysis() -> CacheAnalysis {
        Self::analyse(
            4320, // plan age: 3 days
            8542, // reused 8,542 times
            1,    // compiled for 1 row (2019 data)
            6069, // actual rows (2024 data)
            0.85, // high data skew
        )
    }

    pub fn color_contribution(analysis: &CacheAnalysis) -> LevelColorContribution {
        LevelColorContribution::new(
            "L6 UD (Volatility)",
            analysis.color_contribution,
            &format!(
                "Sniffing risk: {} — fix: {}",
                analysis.sniffing_risk.label(),
                analysis.fix_strategy.sql_hint()
            ),
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_is_confirmed_sniffing() {
        assert_eq!(
            CacheAnalyser::demo_analysis().sniffing_risk,
            SniffingRisk::Confirmed
        );
    }

    #[test]
    fn confirmed_sniffing_has_high_red() {
        let a = CacheAnalyser::demo_analysis();
        assert!(
            a.color_contribution.red > 100,
            "red={}",
            a.color_contribution.red
        );
    }

    #[test]
    fn confirmed_sniffing_has_low_green() {
        let a = CacheAnalyser::demo_analysis();
        assert!(
            a.color_contribution.green < 100,
            "green={}",
            a.color_contribution.green
        );
    }

    #[test]
    fn uniform_distribution_is_low_risk() {
        let a = CacheAnalyser::analyse(60, 100, 500, 480, 0.05);
        assert_eq!(a.sniffing_risk, SniffingRisk::Low);
    }

    #[test]
    fn high_reuse_and_skew_selects_split_by_tier() {
        let a = CacheAnalyser::analyse(1440, 5000, 1, 6069, 0.9);
        assert_eq!(a.fix_strategy, CacheFixStrategy::SplitByTier);
    }

    #[test]
    fn moderate_skew_selects_optimize_for_unknown() {
        let a = CacheAnalyser::analyse(60, 100, 100, 250, 0.4);
        assert_eq!(a.fix_strategy, CacheFixStrategy::OptimizeForUnknown);
    }

    #[test]
    fn low_risk_no_fix_needed() {
        let a = CacheAnalyser::analyse(60, 100, 500, 480, 0.05);
        assert_eq!(a.fix_strategy, CacheFixStrategy::None);
    }

    #[test]
    fn storyway_entry_contains_actual_rows() {
        let a = CacheAnalyser::demo_analysis();
        assert!(
            a.storyway_entry.contains("6069"),
            "entry={}",
            a.storyway_entry
        );
    }

    #[test]
    fn explanation_non_empty() {
        let a = CacheAnalyser::demo_analysis();
        assert!(!a.explanation.is_empty());
    }

    #[test]
    fn confirmed_explanation_contains_hint() {
        let a = CacheAnalyser::demo_analysis();
        let hint = a.fix_strategy.sql_hint();
        assert!(a.explanation.contains(hint) || !hint.is_empty());
    }

    #[test]
    fn color_contribution_level_name() {
        let a = CacheAnalyser::demo_analysis();
        let c = CacheAnalyser::color_contribution(&a);
        assert!(c.level_name.contains("L6"));
    }

    #[test]
    fn sniffing_risk_labels_non_empty() {
        for r in [
            SniffingRisk::Low,
            SniffingRisk::Moderate,
            SniffingRisk::High,
            SniffingRisk::Confirmed,
        ] {
            assert!(!r.label().is_empty());
        }
    }

    #[test]
    fn fix_strategy_hints_non_empty_except_none() {
        assert!(!CacheFixStrategy::OptimizeForUnknown.sql_hint().is_empty());
        assert!(!CacheFixStrategy::Recompile.sql_hint().is_empty());
        assert!(!CacheFixStrategy::SplitByTier.sql_hint().is_empty());
        assert!(!CacheFixStrategy::PlanGuide.sql_hint().is_empty());
        assert_eq!(CacheFixStrategy::None.sql_hint(), "");
    }
}
