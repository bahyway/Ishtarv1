//! Logical query plan — the algebraic layer above physical storage.
//! Produced by the HeptaScript → Ezida compiler front-end.

#![forbid(unsafe_code)]

use crate::expr::Expr;

/// A node in the logical query plan tree.
#[derive(Debug, Clone)]
pub enum LogicalPlan {
    /// Full scan by entity identifier.
    ScanEntity { entity: String },
    /// Scan all tuples for a given attribute.
    ScanAttr { attr: String },
    /// Temporal range scan: all tuples with `time_start <= T < time_end`.
    ScanTimeRange { time_start: u64, time_end: u64 },
    /// Shard-local scan.
    ScanShard { shard: u32 },
    /// Filter: keep tuples matching the predicate.
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
    /// Projection: emit only the named fields.
    Project {
        input: Box<LogicalPlan>,
        fields: Vec<String>,
    },
    /// Hash join on entity id (E dimension).
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        on: Expr,
    },
    /// Aggregate: `GROUP BY fields` with `AggregateOp` applied to `value_expr`.
    Aggregate {
        input: Box<LogicalPlan>,
        group_by: Vec<String>,
        op: AggregateOp,
        value_expr: Expr,
    },
    /// Limit: emit at most `count` tuples.
    Limit {
        input: Box<LogicalPlan>,
        count: usize,
    },
}

/// Aggregation operations available in HeptaScript GROUP-BY queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateOp {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Expr, Field};

    #[test]
    fn test_build_filter_over_scan() {
        let plan = LogicalPlan::Filter {
            input: Box::new(LogicalPlan::ScanEntity {
                entity: "sensor:42".into(),
            }),
            predicate: Expr::Gt(
                Box::new(Expr::Field(Field::Time)),
                Box::new(Expr::LitU64(1_000_000)),
            ),
        };
        assert!(matches!(plan, LogicalPlan::Filter { .. }));
    }

    #[test]
    fn test_build_project_over_scan_attr() {
        let plan = LogicalPlan::Project {
            input: Box::new(LogicalPlan::ScanAttr {
                attr: "temperature".into(),
            }),
            fields: vec!["entity".into(), "value".into(), "time".into()],
        };
        assert!(matches!(plan, LogicalPlan::Project { .. }));
    }

    #[test]
    fn test_aggregate_op_variants() {
        assert_eq!(AggregateOp::Count, AggregateOp::Count);
        assert_ne!(AggregateOp::Sum, AggregateOp::Max);
    }
}
