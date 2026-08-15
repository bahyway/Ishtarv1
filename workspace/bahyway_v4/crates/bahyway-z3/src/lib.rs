//! bahyway-z3 — dev-time-only HeptaScript WHERE/WHY satisfiability checker.
//!
//! ## Gate G4
//!
//! Sealed in `BC-ENV-001_Enbilulu_Calculus` (2026-07-07): *"Z3 is
//! design-time only (MUMMU/GeoEngine, Gate G4): usable for proving
//! satisfiability of composite Enbi template shapes in DubSar PDM; never
//! present in the shipped EnbiluluEngine binary."* — the same ruling
//! applies here: this crate wraps the real `z3` crate (a real C++ SMT
//! solver underneath), which makes it categorically different from every
//! other crate in this workspace. It exists to be invoked from a
//! separate dev-time tool (a CLI, or DubSar PDM's design-time "validate
//! query" action) — **no shipped sovereign server binary** (`enkiddb-
//! read-server`, `enkiddb-write-server`, `enkimdb-read-server`,
//! `enkimdb-write-server`, ...) may ever depend on this crate.
//!
//! ## What it checks
//!
//! Stakeholders assembling long HeptaScript queries in DubSar's PDM
//! (connecting Tribe Orbits across the CrossTribe-Kaki Journal, per the
//! Architect's own framing) can build a WHERE/WHY clause set that no
//! entity could ever satisfy — e.g. `WHERE E[age] > 10 AND E[age] < 5`.
//! `check_query` compiles those clauses into Z3 constraints and reports
//! whether they are jointly satisfiable, unsatisfiable (with which parts
//! of the query are to blame), or undecided within Z3's default limits.
//!
//! This is scalar constraint satisfiability over one entity's attribute
//! values — not relational algebra. No JOIN/UNION/INTERSECT/EXCEPT/
//! GROUP-BY/CTE/APPLY shape is modeled, needed, or accepted here, per
//! the Architect's Anti-SQL law: HeptaScript's WHERE/WHY conditions are
//! themselves already scalar (an entity's own attributes compared to
//! literals), so satisfiability checking never has to reach for a
//! relational-algebra concept to do its job.

pub mod compile;

pub use compile::{check_query, SatOutcome, ValidationReport};
