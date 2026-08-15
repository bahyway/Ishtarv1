//! heptascript v2.0 — W5H2 Sovereign Query Language for the EAV particle space.
//!
//! ## Sovereign Operation verb (optional, leads the whole query):
//!   ORBIT    — read/scan (default when no verb keyword is given)
//!   EMIT     — ORBIT filtered to `hist.event = "BIRTH"` particles
//!   PROVE    — ORBIT with each match's full per-epoch history attached
//!   SYNC     — ORBIT plus an order-independent CRC32 state fingerprint
//!   WITNESS  — ORBIT plus a SHA3-256 content digest
//! See `operations::Operation` (this verb's real type) and
//! `engine::execute_over_histories` for what each one actually does —
//! all five stay strictly read-only.
//!
//! ## v1 clauses (7 W5H2 dimensions):
//!   WHO      — entity variable bindings  (TribeName.VarName)
//!   WHAT     — attribute projection      (VarName[attr1, attr2])
//!   WHERE    — EAV-native conditions     (VarName[attr] op value)
//!   WHEN     — temporal constraints      (AT EPOCH N, BEFORE EPOCH N)
//!   WHY      — lane / quality evidence   (LANE != Black, QUALITY_BYTE > 150)
//!   HOW      — ordering methodology      (BY VarName[attr] ASC|DESC)
//!   HOW_MUCH — cardinality               (LIMIT N, TOP N BY …)
//!
//! ## v2.0 clauses (cross-database, pattern intelligence, federation):
//!   NODE     — target DB type(s)         (EnkiMDB | NARUDU | ENKI_PATTERN | ALL)
//!   ACROSS   — cross-BIGRING federation  (BIGRING ClientX | ALL)
//!   TIER     — storage tier filter       (HOT | WARM | COLD | CRYSTALLIZED)
//!   STATE    — lifecycle filter          (EMERGING | STABLE | CANONICAL | DEPRECATED)
//!   NASH     — Nash equilibrium filter   (SCORE < 0.20 | BREAKING)
//!   PATTERN  — ENKI-PATTERN conditions  (TYPE CrowdFlow | CONFIDENCE >= 0.85)
//!   LINEAGE  — causality traversal       (DEPTH 3 | FULL)
//!   GATE     — ETL station routing       (DataSteward, DataCleansing | ALL)
//!   SATAMU   — governance gate           (REQUIRED | BYPASS "reason")
//!   ORBITAL  — orbital time range        (100 .. 500 | NOW)
//!
//! ## Example (full v2.0 query):
//! ```text
//! NODE EnkiMDB | ENKI_PATTERN
//! ACROSS BIGRING ClientAlpha
//! WHO Tribes.E
//! WHAT E[event.type, location.zone]
//! WHERE E[event.type] = "entry"
//! WHEN AFTER EPOCH 1680000000
//! ORBITAL 100 .. 500
//! WHY LANE != Black
//! TIER HOT | WARM
//! STATE CANONICAL
//! NASH SCORE < 0.20
//! PATTERN TYPE CrowdFlow
//! PATTERN CONFIDENCE >= 0.85
//! LINEAGE DEPTH 3
//! GATE DataSteward, DataCleansing
//! SATAMU REQUIRED
//! HOW BY E[location.zone] ASC
//! HOW_MUCH LIMIT 1000
//! ```

pub mod token;
pub mod query;
pub mod parser;
pub mod engine;
pub mod indexed;
pub mod modular_index;
pub mod operations;
pub mod orbit_objects;
pub mod sumuukin;

pub use token::tokenize;
pub use operations::Operation;
pub use query::{
    // Sovereign Operation verb (alias of operations::Operation)
    QueryVerb,
    // v1
    HeptaQuery, WhoClause, EntityBinding,
    WhatClause, Projection,
    WhereCondition, ConditionTest,
    WhenClause, EpochRef,
    WhyCondition, LaneValue,
    HowClause, SortDir,
    HowMuchClause,
    Op, HeptaValue, Combinator,
    // v2.0
    NodeClause, DbTarget,
    AcrossClause,
    TierClause, StorageTier,
    StateClause, LifecycleState,
    NashClause,
    PatternCondition, PatternKind,
    LineageClause,
    GateClause, GateTarget,
    SatamuClause,
    OrbitalClause,
    // v2.1 (Anti-SQL aggregate clauses)
    MeasureClause, GravityClause, GravityMode,
};
pub use parser::{parse_query, ParseError};
pub use engine::{
    execute, execute_over, execute_over_histories, orbital_range_from_query,
    QueryResult, QueryPlan, MatchedEntity, EpochSnapshot,
    MeasureValue, GravityKey, GravityGroup, GravityResult,
};
pub use indexed::{
    build_indexes, execute_indexed, nairu_range_from_when, spatial_center_from_where,
    HeptaIndexes, SPATIAL_ATTRS,
};
// Production execution hints (ANCHOR / FILTER_ORDER) — not re-exported
// above with the rest of `query`'s v2.0/v2.1 clause types because they
// were added later; re-exported here so `AnchorStrategy` (which
// `enkidb_readnode::cached::CachedReadNode` matches on directly) doesn't
// need the full `heptascript::query::` path.
pub use query::{AnchorStrategy, FilterStage};
pub use modular_index::{
    ModularNaviIndex, ResonanceScorerNc2, E2FourierWeights,
    BUCKET_STEP, MAX_BUCKETS, chord_weight, weight_chord,
};
