//! Proof-aware cost model for the Ezida query optimizer.
//!
//! Formula (ADR-002):
//!   total = io * 1.0 + cpu * 0.5 + proof * 3.0 + memory * 1.5
//!
//! Proof cost dominates because a larger zk witness = a larger circuit.
//! Minimizing proof cost is the primary objective of physical plan selection.

#![forbid(unsafe_code)]

use crate::physical::{IndexKind, PhysicalPlan};

/// Per-operation cost estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct Cost {
    /// Estimated I/O operations (LSM block reads).
    pub io: f64,
    /// Estimated CPU cycles (normalized).
    pub cpu: f64,
    /// Estimated zk proof cost (witness size proxy).
    pub proof: f64,
    /// Estimated memory pressure (bytes / 1024).
    pub memory: f64,
}

impl Cost {
    pub const ZERO: Self = Self {
        io: 0.0,
        cpu: 0.0,
        proof: 0.0,
        memory: 0.0,
    };

    pub fn add(&self, other: &Self) -> Self {
        Self {
            io: self.io + other.io,
            cpu: self.cpu + other.cpu,
            proof: self.proof + other.proof,
            memory: self.memory + other.memory,
        }
    }
}

/// Total scalar cost — proof term has coefficient 3.0 (ADR-002).
pub fn total_cost(c: &Cost) -> f64 {
    c.io * 1.0 + c.cpu * 0.5 + c.proof * 3.0 + c.memory * 1.5
}

/// Heuristic cost estimates for each physical plan node.
/// Actual cardinality statistics are not yet available (v4.0 stub).
pub fn estimate(plan: &PhysicalPlan) -> Cost {
    match plan {
        PhysicalPlan::IndexScan { index, .. } => {
            // Flat per-index estimates (v4.0 stub — no cardinality stats yet).
            // ProofCtxTime carries the highest proof cost because it selects
            // which zk circuit to instantiate — larger witness = larger circuit.
            let (io, cpu, proof, memory) = match index {
                IndexKind::EntityAttrTime => (1.0, 0.50, 0.5, 0.25),
                IndexKind::AttrValueTime => (1.5, 0.75, 1.0, 0.38),
                IndexKind::ValueAttr => (2.0, 1.00, 1.5, 0.50),
                IndexKind::ShardTime => (2.0, 1.00, 1.5, 0.50),
                IndexKind::ProofCtxTime => (3.0, 1.50, 6.0, 0.75),
            };
            Cost {
                io,
                cpu,
                proof,
                memory,
            }
        }
        PhysicalPlan::FullScan => Cost {
            io: 100.0,
            cpu: 50.0,
            proof: 1.0,
            memory: 16.0,
        },
        PhysicalPlan::Filter { input, .. } => {
            let child = estimate(input);
            child.add(&Cost {
                io: 0.0,
                cpu: 1.0,
                proof: 0.0,
                memory: 0.5,
            })
        }
        PhysicalPlan::Projection { input, fields } => {
            let child = estimate(input);
            let f = fields.len() as f64;
            child.add(&Cost {
                io: 0.0,
                cpu: f * 0.1,
                proof: 0.0,
                memory: f * 0.1,
            })
        }
        PhysicalPlan::NestedLoopJoin { outer, inner, .. } => {
            let oc = estimate(outer);
            let ic = estimate(inner);
            Cost {
                io: oc.io + ic.io * 10.0,
                cpu: oc.cpu + ic.cpu * 10.0,
                proof: oc.proof + ic.proof,
                memory: oc.memory + ic.memory,
            }
        }
        PhysicalPlan::Limit { input, .. } => estimate(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical::{IndexKind, KeyRange, PhysicalPlan};

    #[test]
    fn test_proof_cost_dominates_for_proof_ctx_index() {
        let plan = PhysicalPlan::IndexScan {
            index: IndexKind::ProofCtxTime,
            key_range: KeyRange::full(),
        };
        let c = estimate(&plan);
        // proof * 3.0 should exceed io * 1.0 for ProofCtxTime
        assert!(c.proof * 3.0 > c.io * 1.0);
    }

    #[test]
    fn test_entity_attr_time_cheapest_index() {
        let eat = estimate(&PhysicalPlan::IndexScan {
            index: IndexKind::EntityAttrTime,
            key_range: KeyRange::full(),
        });
        let zkt = estimate(&PhysicalPlan::IndexScan {
            index: IndexKind::ProofCtxTime,
            key_range: KeyRange::full(),
        });
        assert!(total_cost(&eat) < total_cost(&zkt));
    }

    #[test]
    fn test_full_scan_more_expensive_than_index_scan() {
        let idx = estimate(&PhysicalPlan::IndexScan {
            index: IndexKind::EntityAttrTime,
            key_range: KeyRange::full(),
        });
        let full = estimate(&PhysicalPlan::FullScan);
        assert!(total_cost(&full) > total_cost(&idx));
    }

    #[test]
    fn test_cost_add_commutative() {
        let a = Cost {
            io: 1.0,
            cpu: 2.0,
            proof: 3.0,
            memory: 4.0,
        };
        let b = Cost {
            io: 0.5,
            cpu: 1.0,
            proof: 0.5,
            memory: 1.0,
        };
        let ab = a.add(&b);
        let ba = b.add(&a);
        assert_eq!(ab, ba);
    }

    #[test]
    fn test_total_cost_formula() {
        let c = Cost {
            io: 10.0,
            cpu: 10.0,
            proof: 10.0,
            memory: 10.0,
        };
        // 10*1.0 + 10*0.5 + 10*3.0 + 10*1.5 = 10 + 5 + 30 + 15 = 60
        assert!((total_cost(&c) - 60.0_f64).abs() < f64::EPSILON);
    }
}
