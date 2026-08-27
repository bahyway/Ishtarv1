//! Ezida IR — Logical and Physical Intermediate Representation.
//!
//! Pipeline:
//!   HeptaScript AST
//!     → LogicalPlan   (this crate)
//!     → PhysicalPlan  (this crate, via cost optimizer)
//!     → CPU Runtime   (ezida-executor)
//!     → Raft + WAL    (enkidb-raft)
//!     → LSM Storage   (enkidb-storage)

#![forbid(unsafe_code)]

pub mod cost;
pub mod expr;
pub mod logical;
pub mod physical;

pub use cost::{total_cost, Cost};
pub use expr::Expr;
pub use logical::LogicalPlan;
pub use physical::{IndexKind, KeyRange, PhysicalPlan};
