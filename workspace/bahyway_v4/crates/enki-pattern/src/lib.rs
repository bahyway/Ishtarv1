#![forbid(unsafe_code)]
//! # ENKI-PATTERN — Shared Pattern Registry
//!
//! **`PatternRegistry` (registry.rs) is SUPERSEDED as of 2026-07-31,
//! per the Architect's law: "no particle stays or evaluates outside the
//! 7 Types EnkiDB."** This crate's own registry was a separate physical
//! store — genuinely non-compliant, and (confirmed at the time of the
//! migration) had zero real callers anywhere in the workspace regardless.
//! Real patterns now mint as EnkiMDB particles: see
//! `enkimdb::pattern_emitter::PatternEmitter` /
//! `enkimdb::writenode::WriteNode::ingest_pattern`. `registry.rs` and
//! `PatternRegistry` are kept here, unmodified, per this ecosystem's own
//! no-delete law — read `enkimdb`'s version for anything new.
//!
//! `tier.rs` (`StorageTier::from_age`) is NOT superseded — it's pure,
//! stateless classification logic, reused as-is by
//! `enkimdb::pattern::PatternProfile::tier()` rather than duplicated.
//! `narudu.rs`/`nash.rs` were not touched by this migration; whether they
//! need the same treatment is a separate, not-yet-answered question.
//!
//! Patterns were previously stored across four tiers based on age and
//! access frequency (this table describes `PatternRegistry`'s own,
//! now-superseded design):
//!
//! | Tier         | Age (orbitals) | Max patterns | Access    |
//! |--------------|---------------|--------------|-----------|
//! | Hot          | 0–3           | unlimited    | <1ms      |
//! | Warm         | 3–30          | unlimited    | <10ms     |
//! | Cold         | 30–365        | unlimited    | <1s       |
//! | Crystallized | 365+          | unlimited    | minutes   |
//!
//! ## NARUDU-PATTERN events (NARUDU-001):
//! Emitted on every lifecycle transition: Published, NashShift, Deprecated,
//! SimulationComplete, StewardDecision.

pub mod narudu;
pub mod nash;
pub mod registry;
pub mod tier;

pub use narudu::{NaruduEvent, NaruduEventKind};
pub use nash::NashState;
pub use registry::PatternRegistry;
pub use tier::{PatternRecord, StorageTier};
