// ============================================================================
// compare-tribe-schema — full schema versioning and comparison engine.
//
// Modules:
//   schema_version    — FieldMeta, FieldType, SchemaVersion (versioned snapshot)
//   field_diff        — FieldChange, FieldDiff (granular per-field changes)
//   schema_comparator — compare_versions() → CompareReport + CompareVerdict
//   dead_verdict      — DeadVerdict + SchemaMismatchAlert (for DataStewardEngine)
//   tribe_completeness— cross-tribe optional attribute completeness ranking
//   compare           — legacy Template-level field set diff (§10.8)
//   pauli_dedup       — hash-indexed Tribe-vs-Tribe Pauli-duplicate dedup
// ============================================================================

pub mod compare;
pub mod dead_verdict;
pub mod field_diff;
pub mod pauli_dedup;
pub mod schema_comparator;
pub mod schema_version;
pub mod tribe_completeness;

// V1→V2 pipeline
pub use dead_verdict::{AlertPriority, DeadVerdict, SchemaMismatchAlert};
pub use field_diff::{FieldChange, FieldDiff};
pub use pauli_dedup::{dedup_tribes, PauliDedupReport, PauliDuplicate};
pub use schema_comparator::{compare_versions, CompareReport, CompareVerdict};
pub use schema_version::{FieldMeta, FieldType, SchemaVersion};
pub use tribe_completeness::{
    completeness_gap, most_organized, rank_tribes, TribeCompleteness, TribeRank,
};

// Legacy template diff
pub use compare::{compare, SchemaDiff};
