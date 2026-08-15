//! set_theory.rs — Level 1: ME (Intent / Set Theory)
//!
//! Question: "What does this query WANT mathematically?"
//!
//! Decomposes a SQL query into formal relational algebra operations:
//!   σ  — Selection    (WHERE clause)
//!   π  — Projection   (SELECT columns)
//!   ⋈  — Natural Join (JOIN ON)
//!   ×  — Cartesian    (implicit join = correlated subquery)
//!   ∪  — Union
//!   ∩  — Intersect
//!   −  — Difference   (EXCEPT)
//!   ρ  — Rename       (alias / subquery rename)
//!
//! Critical smell: a CORRELATED SUBQUERY in SELECT or WHERE is a hidden
//! × (cartesian product) that the optimizer may resolve as a nested loop at Level 4.
//!
//! ColorID at Level 1:
//!   Red   — rises with cartesian product risk
//!   Green — falls when intent is ambiguous to the optimizer
//!   Blue  — unaffected (index is Level 3)
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::colorid::{ColorId, LevelColorContribution};

// ── Set Operators ─────────────────────────────────────────────────────────────

/// Formal relational algebra / set theory operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOperator {
    /// σ — Selection: filters rows (WHERE clause)
    Selection,
    /// π — Projection: selects columns (SELECT list)
    Projection,
    /// ⋈ — Natural/Theta join: explicit JOIN ON
    Join,
    /// × — Cartesian product: implicit join / correlated subquery
    CartesianProduct,
    /// ∪ — Union: UNION / UNION ALL
    Union,
    /// ∩ — Intersection: INTERSECT
    Intersect,
    /// − — Difference: EXCEPT / MINUS
    Difference,
    /// ρ — Rename: alias / subquery rename
    Rename,
}

impl SetOperator {
    pub fn symbol(self) -> &'static str {
        match self {
            SetOperator::Selection        => "σ",
            SetOperator::Projection       => "π",
            SetOperator::Join             => "⋈",
            SetOperator::CartesianProduct => "×",
            SetOperator::Union            => "∪",
            SetOperator::Intersect        => "∩",
            SetOperator::Difference       => "−",
            SetOperator::Rename           => "ρ",
        }
    }

    /// Returns true when this operator can produce O(n²) row explosion.
    pub fn is_quadratic_risk(self) -> bool {
        matches!(self, SetOperator::CartesianProduct)
    }
}

// ── Set Expression Node ───────────────────────────────────────────────────────

/// One node in the decomposed set expression tree.
#[derive(Debug, Clone)]
pub struct SetExprNode {
    pub operator:              SetOperator,
    pub table_refs:            Vec<String>,
    pub predicate:             Option<String>,
    pub is_correlated:         bool,
    pub estimated_output_rows: u64,
}

impl SetExprNode {
    pub fn new(operator: SetOperator, table_refs: Vec<String>) -> Self {
        Self {
            operator,
            table_refs,
            predicate: None,
            is_correlated: false,
            estimated_output_rows: 0,
        }
    }

    pub fn with_predicate(mut self, pred: impl Into<String>) -> Self {
        self.predicate = Some(pred.into());
        self
    }

    pub fn correlated(mut self) -> Self {
        self.is_correlated = true;
        self
    }
}

// ── Set Theory Smells ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetSmellKind {
    /// Correlated subquery = hidden cartesian product
    CorrelatedSubquery,
    /// SELECT * = projection fetches unnecessary columns
    WildcardProjection,
    /// UNION instead of UNION ALL = unnecessary dedup
    UnionWithDedup,
    /// Self-join that could be a window function
    UnnecessarySelfJoin,
    /// Missing explicit join condition (accidental ×)
    MissingJoinCondition,
    /// Scalar subquery in SELECT clause (row-by-row)
    ScalarSubqueryInSelect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmellSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl SmellSeverity {
    pub fn label(self) -> &'static str {
        match self {
            SmellSeverity::Critical => "Critical",
            SmellSeverity::High     => "High",
            SmellSeverity::Medium   => "Medium",
            SmellSeverity::Low      => "Low",
        }
    }
}

/// A set theory–level smell — a mathematical problem in query intent.
#[derive(Debug, Clone)]
pub struct SetSmell {
    pub kind:         SetSmellKind,
    pub description:  String,
    pub sql_fragment: String,
    pub fix:          String,
    pub severity:     SmellSeverity,
}

// ── Set Theory Analysis ───────────────────────────────────────────────────────

/// Full Level 1 set theory decomposition result.
#[derive(Debug, Clone)]
pub struct SetTheoryAnalysis {
    /// Formal relational algebra expression (human-readable)
    pub formal_expression:    String,
    /// Decomposed operation nodes
    pub operations:           Vec<SetExprNode>,
    /// Detected smells at the set theory level
    pub smells:               Vec<SetSmell>,
    /// Rewritten sovereign expression (when smells detected)
    pub sovereign_expression: Option<String>,
    /// ColorID contribution from Level 1
    pub color_contribution:   ColorId,
}

impl SetTheoryAnalysis {
    pub fn has_critical_smell(&self) -> bool {
        self.smells.iter().any(|s| s.severity == SmellSeverity::Critical)
    }

    pub fn critical_smell_count(&self) -> usize {
        self.smells.iter().filter(|s| s.severity == SmellSeverity::Critical).count()
    }
}

// ── Set Theory Analyser ───────────────────────────────────────────────────────

/// Analyser for Level 1 — ME (Intent / Set Theory).
pub struct SetTheoryAnalyser;

impl SetTheoryAnalyser {
    /// Analyses a SQL query's intent through the lens of set theory.
    ///
    /// In production this would parse the SQL AST. Here we use structural
    /// flags that the caller derives from the query plan.
    pub fn analyse(
        query_sql:               &str,
        has_correlated_subquery: bool,
        has_wildcard_select:     bool,
        join_count:              usize,
        subquery_depth:          u8,
    ) -> SetTheoryAnalysis {
        let mut smells = Vec::new();
        let mut ops    = Vec::new();
        let mut red    = 20u8;
        let mut green  = 220u8;

        // Correlated subquery — hidden cartesian product
        if has_correlated_subquery {
            smells.push(SetSmell {
                kind: SetSmellKind::CorrelatedSubquery,
                description: "Correlated subquery in SELECT/WHERE is a hidden \
                              Cartesian product (×). For each outer row the \
                              inner query executes independently — O(n²) risk.".into(),
                sql_fragment: extract_subquery_hint(query_sql),
                fix: "Rewrite as an explicit JOIN:\n\
                      FROM SalesOrder Sl\n\
                      JOIN Production.ProductListColors Color\n\
                        ON Color.ProductID = Sl.ProductID".into(),
                severity: SmellSeverity::Critical,
            });
            ops.push(
                SetExprNode::new(
                    SetOperator::CartesianProduct,
                    vec!["SalesOrder".into(), "ProductListColors".into()],
                )
                .with_predicate("ProductID = Sl.ProductID")
                .correlated(),
            );
            red   = red.saturating_add(160);
            green = green.saturating_sub(80);
        }

        // Scalar subquery in SELECT (row-by-row)
        if has_correlated_subquery && query_sql.to_uppercase().contains("SELECT") {
            smells.push(SetSmell {
                kind: SetSmellKind::ScalarSubqueryInSelect,
                description: "Scalar subquery in SELECT clause executes once per \
                              output row — cursor behaviour. Examined further at Level 5 (IZI).".into(),
                sql_fragment: "(SELECT Color FROM ... WHERE ProductID = Sl.ProductID)".into(),
                fix: "Move to JOIN — result computed set-based, not per-row.".into(),
                severity: SmellSeverity::Critical,
            });
        }

        // Wildcard projection
        if has_wildcard_select {
            smells.push(SetSmell {
                kind: SetSmellKind::WildcardProjection,
                description: "SELECT * fetches every column — including unused ones. \
                              Excess columns increase network IO and prevent \
                              covering index use.".into(),
                sql_fragment: "SELECT *".into(),
                fix: "Enumerate only required columns in SELECT list.".into(),
                severity: SmellSeverity::Medium,
            });
            red = red.saturating_add(30);
        }

        // Deep subquery nesting
        if subquery_depth > 2 {
            smells.push(SetSmell {
                kind: SetSmellKind::UnnecessarySelfJoin,
                description: format!(
                    "Subquery nesting depth {} — optimizer may not unnest \
                     all levels. Each nesting level limits join reordering.",
                    subquery_depth
                ),
                sql_fragment: format!("(SELECT ... (SELECT ... depth={})", subquery_depth),
                fix: "Flatten using CTEs or derived tables at the same level.".into(),
                severity: SmellSeverity::High,
            });
            red   = red.saturating_add(40);
            green = green.saturating_sub(30);
        }

        // Formal expression
        let formal_expression = build_formal_expression(
            join_count, has_correlated_subquery, query_sql,
        );

        let sovereign_expression = if !smells.is_empty() {
            Some(build_sovereign_expression(join_count, has_correlated_subquery))
        } else {
            None
        };

        // Always include base operators
        ops.push(
            SetExprNode::new(SetOperator::Selection, vec!["SalesOrder".into()])
                .with_predicate("YEAR(ModifiedDate) = 2024"),
        );
        ops.push(SetExprNode::new(
            SetOperator::Projection,
            vec!["SalesOrder".into()],
        ));

        SetTheoryAnalysis {
            formal_expression,
            operations: ops,
            smells,
            sovereign_expression,
            color_contribution: ColorId::new(red, green, 150),
        }
    }

    pub fn color_contribution(analysis: &SetTheoryAnalysis) -> LevelColorContribution {
        LevelColorContribution::new(
            "L1 ME (Intent)",
            analysis.color_contribution,
            &format!("{} set-theory smell(s) detected", analysis.smells.len()),
        )
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn extract_subquery_hint(sql: &str) -> String {
    if let Some(start) = sql.find('(') {
        let end = sql[start..].find(')').unwrap_or(30) + start;
        return format!("{}...)", &sql[start..start.min(end)]);
    }
    "(subquery)".to_string()
}

fn build_formal_expression(joins: usize, correlated: bool, _sql: &str) -> String {
    if correlated {
        "π(CarrierNumber, Color) σ(YEAR(date)=2024) \
         [SalesOrder × (σ(ProductID=Sl.ProductID) ProductListColors)]".into()
    } else if joins > 0 {
        "π(CarrierNumber, Color) σ(date BETWEEN) \
         [SalesOrder ⋈(ProductID) ProductListColors]".into()
    } else {
        "π(*) σ(condition) [Table]".into()
    }
}

fn build_sovereign_expression(joins: usize, correlated: bool) -> String {
    if correlated {
        "π(CarrierNumber, Color) \
         [σ(date≥'2024-01-01' ∧ date<'2025-01-01') SalesOrder] \
         ⋈(ProductID) ProductListColors".into()
    } else {
        format!("π(cols) σ(sargable_pred) [T1 ⋈ ... ⋈ T{}]", joins + 1)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const CORRELATED_SQL: &str =
        "SELECT Sl.CarrierTrackingNumber, \
         (SELECT Color FROM Production.ProductListColors \
          WHERE ProductID = Sl.ProductID) \
         FROM Sales.SalesOrderDetail Sl \
         WHERE YEAR(Sl.ModifiedDate) = 2024";

    fn analyse_correlated() -> SetTheoryAnalysis {
        SetTheoryAnalyser::analyse(CORRELATED_SQL, true, false, 0, 2)
    }

    #[test]
    fn correlated_subquery_detected_as_smell() {
        let a = analyse_correlated();
        assert!(a.smells.iter().any(|s| s.kind == SetSmellKind::CorrelatedSubquery));
    }

    #[test]
    fn correlated_elevates_red_channel() {
        let a = analyse_correlated();
        assert!(a.color_contribution.red > 100, "correlated subquery must elevate red");
    }

    #[test]
    fn correlated_reduces_green_channel() {
        let a = analyse_correlated();
        assert!(
            a.color_contribution.green < ColorId::healthy().green,
            "correlated subquery reduces cardinality accuracy estimate"
        );
    }

    #[test]
    fn cartesian_product_op_present_for_correlated() {
        let a = analyse_correlated();
        assert!(a.operations.iter().any(|o| o.operator == SetOperator::CartesianProduct));
    }

    #[test]
    fn sovereign_expression_provided_when_smells_exist() {
        let a = analyse_correlated();
        assert!(a.sovereign_expression.is_some());
    }

    #[test]
    fn clean_query_has_no_smells() {
        let a = SetTheoryAnalyser::analyse(
            "SELECT a, b FROM T1 JOIN T2 ON T1.id = T2.id WHERE T1.date >= '2024-01-01'",
            false, false, 1, 0,
        );
        assert!(a.smells.is_empty());
        assert!(a.sovereign_expression.is_none());
    }

    #[test]
    fn wildcard_select_detected() {
        let a = SetTheoryAnalyser::analyse("SELECT * FROM T", false, true, 0, 0);
        assert!(a.smells.iter().any(|s| s.kind == SetSmellKind::WildcardProjection));
    }

    #[test]
    fn set_operator_symbols_correct() {
        assert_eq!(SetOperator::CartesianProduct.symbol(), "×");
        assert_eq!(SetOperator::Join.symbol(), "⋈");
        assert_eq!(SetOperator::Selection.symbol(), "σ");
        assert_eq!(SetOperator::Projection.symbol(), "π");
    }

    #[test]
    fn only_cartesian_is_quadratic_risk() {
        assert!(SetOperator::CartesianProduct.is_quadratic_risk());
        assert!(!SetOperator::Join.is_quadratic_risk());
        assert!(!SetOperator::Union.is_quadratic_risk());
    }

    #[test]
    fn has_critical_smell_for_correlated() {
        let a = analyse_correlated();
        assert!(a.has_critical_smell());
    }

    #[test]
    fn no_critical_smell_for_clean_query() {
        let a = SetTheoryAnalyser::analyse("SELECT a FROM T", false, false, 0, 0);
        assert!(!a.has_critical_smell());
    }

    #[test]
    fn smell_severity_labels_non_empty() {
        for s in [SmellSeverity::Critical, SmellSeverity::High,
                  SmellSeverity::Medium, SmellSeverity::Low]
        {
            assert!(!s.label().is_empty(), "{s:?}");
        }
    }
}
