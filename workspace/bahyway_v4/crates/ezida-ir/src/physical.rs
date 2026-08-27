//! Physical query plan — bound to the 5 sovereign LSM indexes.
//!
//! The 5 composable indexes over the 7D EAV tuple (E,A,V,T,S,Z,M):
//!   (E,A,T)  — entity-attribute timeline       → point / range lookups
//!   (A,V,T)  — attribute-value timeline         → predicate pushdown
//!   (V,A)    — value inversion                  → reverse lookup
//!   (S,T)    — shard timeline                   → shard-local scans
//!   (Z,T)    — proof-context timeline           → zk circuit selection
//!
//! Proof cost dominates: `proof * 3.0` (see cost.rs).

#![forbid(unsafe_code)]

use crate::expr::Expr;

/// Which of the 5 sovereign LSM indexes to hit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexKind {
    /// (E, A, T) — entity-attribute timeline.
    EntityAttrTime,
    /// (A, V, T) — attribute-value timeline.
    AttrValueTime,
    /// (V, A) — value inversion.
    ValueAttr,
    /// (S, T) — shard timeline.
    ShardTime,
    /// (Z, T) — proof-context timeline.
    ProofCtxTime,
}

/// An inclusive byte-range key bound for LSM scans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRange {
    pub start: Vec<u8>,
    pub end: Vec<u8>,
}

impl KeyRange {
    pub fn full() -> Self {
        Self {
            start: vec![],
            end: vec![0xFF; 32],
        }
    }

    pub fn point(key: Vec<u8>) -> Self {
        let end = key.clone();
        Self { start: key, end }
    }
}

/// A node in the physical execution plan.
#[derive(Debug, Clone)]
pub enum PhysicalPlan {
    /// Scan an LSM index over the given key range.
    IndexScan {
        index: IndexKind,
        key_range: KeyRange,
    },
    /// Sequential full scan (no index available).
    FullScan,
    /// Filter rows produced by the child plan.
    Filter {
        input: Box<PhysicalPlan>,
        predicate: Expr,
    },
    /// Project a subset of fields from child output.
    Projection {
        input: Box<PhysicalPlan>,
        fields: Vec<String>,
    },
    /// Nested-loop join (used when hash join is not applicable).
    NestedLoopJoin {
        outer: Box<PhysicalPlan>,
        inner: Box<PhysicalPlan>,
        on: Expr,
    },
    /// Limit: stop after emitting `count` rows.
    Limit {
        input: Box<PhysicalPlan>,
        count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Expr, Field};

    #[test]
    fn test_index_scan_entity_attr_time() {
        let plan = PhysicalPlan::IndexScan {
            index: IndexKind::EntityAttrTime,
            key_range: KeyRange::full(),
        };
        assert!(matches!(
            plan,
            PhysicalPlan::IndexScan {
                index: IndexKind::EntityAttrTime,
                ..
            }
        ));
    }

    #[test]
    fn test_key_range_point_eq_start_end() {
        let kr = KeyRange::point(vec![1, 2, 3]);
        assert_eq!(kr.start, kr.end);
    }

    #[test]
    fn test_all_five_index_kinds_distinct() {
        let kinds = [
            IndexKind::EntityAttrTime,
            IndexKind::AttrValueTime,
            IndexKind::ValueAttr,
            IndexKind::ShardTime,
            IndexKind::ProofCtxTime,
        ];
        for i in 0..kinds.len() {
            for j in 0..kinds.len() {
                if i == j {
                    assert_eq!(kinds[i], kinds[j]);
                } else {
                    assert_ne!(kinds[i], kinds[j]);
                }
            }
        }
    }

    #[test]
    fn test_filter_wraps_full_scan() {
        let plan = PhysicalPlan::Filter {
            input: Box::new(PhysicalPlan::FullScan),
            predicate: Expr::Eq(
                Box::new(Expr::Field(Field::Shard)),
                Box::new(Expr::LitU64(0)),
            ),
        };
        assert!(matches!(plan, PhysicalPlan::Filter { .. }));
    }
}
