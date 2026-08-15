//! sargability.rs — Level 3: SAG (Gravity / Selectivity)
//!
//! Question: "Can the filter USE an index?"
//!
//! SARG = Search ARGument able. A predicate is sargable when the database
//! engine can use it to directly seek into an index.
//! Non-sargable predicates FORCE a full table scan regardless of indexes.
//!
//! Common non-sargable patterns:
//!   1. Function on column:   YEAR(date) = 2024
//!      Fix:                  date >= '2024-01-01' AND date < '2025-01-01'
//!   2. Implicit conversion:  col_int = @nvarchar_param
//!      Fix:                  CAST(@param AS INT) = col_int
//!   3. Leading wildcard:     col LIKE '%value%'
//!      Fix:                  full-text search, or LIKE 'value%'
//!   4. Arithmetic on column: col * 2 = 100
//!      Fix:                  col = 50
//!   5. Negation:             col <> 'value'  (usually unavoidable)
//!
//! ColorID at Level 3:
//!   Blue  = index utilisation potential
//!   Red  += for each non-sargable predicate found
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::colorid::{ColorId, LevelColorContribution};

// ── Sargability Kind ──────────────────────────────────────────────────────────

/// The specific reason a predicate is non-sargable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonSargableKind {
    /// Function wrapping the indexed column: YEAR(col), CAST(col), ISNULL(col)
    FunctionOnColumn,
    /// Implicit type conversion: INT col compared to NVARCHAR param
    ImplicitConversion,
    /// Leading wildcard in LIKE: col LIKE '%value'
    LeadingWildcard,
    /// Arithmetic on column: col * 2 = 100, col + 1 > 5
    ArithmeticOnColumn,
    /// Double negation or NOT IN with NULLs
    NegationTrap,
    /// OR condition spanning multiple columns (can't use single index)
    OrAcrossColumns,
}

impl NonSargableKind {
    pub fn label(self) -> &'static str {
        match self {
            NonSargableKind::FunctionOnColumn   => "Function on column",
            NonSargableKind::ImplicitConversion => "Implicit type conversion",
            NonSargableKind::LeadingWildcard    => "Leading wildcard LIKE",
            NonSargableKind::ArithmeticOnColumn => "Arithmetic on column",
            NonSargableKind::NegationTrap       => "Negation trap",
            NonSargableKind::OrAcrossColumns    => "OR across columns",
        }
    }

    pub fn red_penalty(self) -> u8 {
        match self {
            NonSargableKind::FunctionOnColumn   => 100,
            NonSargableKind::ImplicitConversion => 120,
            NonSargableKind::LeadingWildcard    => 80,
            NonSargableKind::ArithmeticOnColumn => 60,
            NonSargableKind::NegationTrap       => 40,
            NonSargableKind::OrAcrossColumns    => 50,
        }
    }
}

// ── Predicate Analysis ────────────────────────────────────────────────────────

/// Analysis of a single WHERE/ON predicate.
#[derive(Debug, Clone)]
pub struct PredicateAnalysis {
    /// The raw predicate SQL fragment
    pub predicate_sql:     String,
    /// Whether this predicate can use an index
    pub is_sargable:       bool,
    /// Why it is non-sargable (if applicable)
    pub non_sargable_kind: Option<NonSargableKind>,
    /// The sargable rewrite
    pub sargable_rewrite:  Option<String>,
    /// Estimated page reads: non-sargable (scan) vs sargable (seek)
    pub page_reads_scan:   u64,
    pub page_reads_seek:   u64,
}

impl PredicateAnalysis {
    /// Fractional reduction in page reads if this predicate is made sargable.
    pub fn page_read_reduction(&self) -> f32 {
        if self.page_reads_scan == 0 { return 0.0; }
        1.0 - (self.page_reads_seek as f32 / self.page_reads_scan as f32)
    }
}

// ── Sargability Analysis ──────────────────────────────────────────────────────

/// Full Level 3 sargability analysis result.
#[derive(Debug, Clone)]
pub struct SargabilityAnalysis {
    pub predicates:                   Vec<PredicateAnalysis>,
    pub sargable_count:               usize,
    pub non_sargable_count:           usize,
    /// Total page read reduction achievable by fixing all non-sargable predicates (0–100).
    pub potential_read_reduction_pct: f32,
    pub color_contribution:           ColorId,
}

impl SargabilityAnalysis {
    pub fn is_fully_sargable(&self) -> bool {
        self.non_sargable_count == 0
    }

    /// The worst non-sargable predicate (highest scan cost).
    pub fn worst_predicate(&self) -> Option<&PredicateAnalysis> {
        self.predicates.iter()
            .filter(|p| !p.is_sargable)
            .max_by_key(|p| p.page_reads_scan)
    }
}

/// Analyser for Level 3 — SAG (Gravity / Selectivity).
pub struct SargabilityAnalyser;

impl SargabilityAnalyser {
    /// Analyses a list of predicates for sargability and returns Level 3 report.
    pub fn analyse(predicates: Vec<PredicateAnalysis>) -> SargabilityAnalysis {
        let sargable_count     = predicates.iter().filter(|p|  p.is_sargable).count();
        let non_sargable_count = predicates.iter().filter(|p| !p.is_sargable).count();

        // Blue channel: 255 = all sargable, 0 = none sargable
        let blue = if predicates.is_empty() {
            200u8
        } else {
            (sargable_count as f32 / predicates.len() as f32 * 255.0) as u8
        };

        // Red channel: penalty per non-sargable predicate (capped)
        let extra_red: u8 = predicates.iter()
            .filter_map(|p| p.non_sargable_kind)
            .map(|k| k.red_penalty())
            .fold(0u8, |acc, v| acc.saturating_add(v / 2));
        let red = 20u8.saturating_add(extra_red);

        // Potential read reduction
        let total_scan: u64 = predicates.iter().map(|p| p.page_reads_scan).sum();
        let total_seek: u64 = predicates.iter().map(|p| p.page_reads_seek).sum();
        let reduction = if total_scan > 0 {
            1.0 - (total_seek as f32 / total_scan as f32)
        } else {
            0.0
        };

        SargabilityAnalysis {
            sargable_count,
            non_sargable_count,
            predicates,
            potential_read_reduction_pct: reduction * 100.0,
            color_contribution: ColorId::new(red, 180, blue),
        }
    }

    /// Demo analysis for the SalesOrder correlated subquery (Level 3 canonical case).
    pub fn demo_analysis() -> SargabilityAnalysis {
        Self::analyse(vec![
            PredicateAnalysis {
                predicate_sql:     "YEAR(Sl.ModifiedDate) = 2024".into(),
                is_sargable:       false,
                non_sargable_kind: Some(NonSargableKind::FunctionOnColumn),
                sargable_rewrite:  Some(
                    "Sl.ModifiedDate >= '2024-01-01' AND Sl.ModifiedDate < '2025-01-01'".into()),
                page_reads_scan:   12_592,
                page_reads_seek:   47,
            },
            PredicateAnalysis {
                predicate_sql:     "Color.ProductID = Sl.ProductID".into(),
                is_sargable:       true,
                non_sargable_kind: None,
                sargable_rewrite:  None,
                page_reads_scan:   0,
                page_reads_seek:   6,
            },
        ])
    }

    pub fn color_contribution(analysis: &SargabilityAnalysis) -> LevelColorContribution {
        LevelColorContribution::new(
            "L3 SAG (Gravity)",
            analysis.color_contribution,
            &format!(
                "{} non-sargable predicate(s) — {:.0}% read reduction available",
                analysis.non_sargable_count,
                analysis.potential_read_reduction_pct,
            ),
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_has_one_non_sargable_one_sargable() {
        let a = SargabilityAnalyser::demo_analysis();
        assert_eq!(a.non_sargable_count, 1);
        assert_eq!(a.sargable_count,     1);
    }

    #[test]
    fn non_sargable_lowers_blue_channel() {
        let a = SargabilityAnalyser::analyse(vec![
            PredicateAnalysis {
                predicate_sql:     "YEAR(col) = 2024".into(),
                is_sargable:       false,
                non_sargable_kind: Some(NonSargableKind::FunctionOnColumn),
                sargable_rewrite:  None,
                page_reads_scan:   10_000,
                page_reads_seek:   10,
            },
        ]);
        assert!(a.color_contribution.blue < 100, "all non-sargable must lower blue");
    }

    #[test]
    fn fully_sargable_gives_high_blue() {
        let a = SargabilityAnalyser::analyse(vec![
            PredicateAnalysis {
                predicate_sql:     "col = 42".into(),
                is_sargable:       true,
                non_sargable_kind: None,
                sargable_rewrite:  None,
                page_reads_scan:   100,
                page_reads_seek:   1,
            },
        ]);
        assert!(a.color_contribution.blue > 200, "fully sargable must give high blue");
    }

    #[test]
    fn demo_page_read_reduction_over_90() {
        let a = SargabilityAnalyser::demo_analysis();
        assert!(
            a.potential_read_reduction_pct > 90.0,
            "YEAR() fix should reduce reads by >90%"
        );
    }

    #[test]
    fn function_on_column_has_highest_red_penalty() {
        assert!(
            NonSargableKind::FunctionOnColumn.red_penalty() >=
            NonSargableKind::ArithmeticOnColumn.red_penalty()
        );
        assert!(
            NonSargableKind::ImplicitConversion.red_penalty() >=
            NonSargableKind::FunctionOnColumn.red_penalty()
        );
    }

    #[test]
    fn worst_predicate_is_the_table_scan() {
        let a = SargabilityAnalyser::demo_analysis();
        let worst = a.worst_predicate().unwrap();
        assert_eq!(worst.page_reads_scan, 12_592);
    }

    #[test]
    fn empty_predicates_is_fully_sargable() {
        let a = SargabilityAnalyser::analyse(vec![]);
        assert!(a.is_fully_sargable());
    }

    #[test]
    fn labels_non_empty_for_all_kinds() {
        let kinds = [
            NonSargableKind::FunctionOnColumn,
            NonSargableKind::ImplicitConversion,
            NonSargableKind::LeadingWildcard,
            NonSargableKind::ArithmeticOnColumn,
            NonSargableKind::NegationTrap,
            NonSargableKind::OrAcrossColumns,
        ];
        for k in kinds { assert!(!k.label().is_empty(), "{k:?}"); }
    }

    #[test]
    fn page_read_reduction_zero_when_scan_is_zero() {
        let p = PredicateAnalysis {
            predicate_sql:     "col = 1".into(),
            is_sargable:       true,
            non_sargable_kind: None,
            sargable_rewrite:  None,
            page_reads_scan:   0,
            page_reads_seek:   5,
        };
        assert_eq!(p.page_read_reduction(), 0.0);
    }

    #[test]
    fn red_elevated_for_non_sargable() {
        let a = SargabilityAnalyser::analyse(vec![
            PredicateAnalysis {
                predicate_sql:     "CAST(col AS VARCHAR) = @p".into(),
                is_sargable:       false,
                non_sargable_kind: Some(NonSargableKind::ImplicitConversion),
                sargable_rewrite:  None,
                page_reads_scan:   5000, page_reads_seek: 50,
            },
        ]);
        assert!(a.color_contribution.red > 50);
    }
}
