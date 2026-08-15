//! HeptaScript v2.0 W5H2 Abstract Syntax Tree.
//!
//! A HeptaQuery is the compiled representation of a W5H2 source query.
//! It is produced by the parser and consumed by the executor / dispatcher.
//!
//! ## v2.0 additions
//! Eight new optional clauses extend the v1 HeptaQuery:
//!
//! | Clause   | Purpose                                                    |
//! |----------|------------------------------------------------------------|
//! | NODE     | Target one or more EnkiDB types (7001–7007, NARUDU, ENKI_PATTERN) |
//! | ACROSS   | Cross-BIGRING federation (by client name or ALL)           |
//! | TIER     | Filter by storage tier (Hot/Warm/Cold/Crystallized)        |
//! | STATE    | Filter by lifecycle state (Emerging/Stable/Canonical/Deprecated) |
//! | NASH     | Nash equilibrium constraint (SCORE op f | BREAKING)        |
//! | PATTERN  | ENKI-PATTERN query (type, confidence, max constituents)    |
//! | LINEAGE  | Causality traversal depth (DEPTH n | FULL)                |
//! | GATE     | ETL station routing (DataSteward | DataCleansing | ALL …) |
//! | SATAMU   | Governance override (REQUIRED | BYPASS "reason")           |
//! | ORBITAL  | Orbital (intra-day cycle) time range                       |

// ── Shared value and operator types ─────────────────────────────────────────

/// Comparison operator used in WHERE, WHY, NASH, and PATTERN conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Eq, Neq, Gt, Lt, Gte, Lte,
}

/// A literal value used in a condition comparison.
#[derive(Debug, Clone, PartialEq)]
pub enum HeptaValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

/// Logical combinator between consecutive conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Combinator {
    And,
    Or,
}

// ── HeptaQuery ────────────────────────────────────────────────────────────────

/// The Sovereign Operation this query performs — `crate::operations::
/// Operation`, reused rather than duplicated (see that type's own doc
/// comment for the full story). An optional leading keyword on a query,
/// defaulting to `Operation::Orbit` (the read/scan every query performed
/// before this verb syntax existed) when omitted — every query written
/// before this existed continues to parse and execute identically.
///
/// As realized here (`engine.rs::execute_over_histories`), all five stay
/// strictly read-only: none mutates the journal — Triple-O's Write Node /
/// Read Node split is never bypassed by a query verb. Where a verb's name
/// plausibly suggests an external side effect (EMIT birthing a particle,
/// SYNC replicating, WITNESS attesting via a second party), it instead
/// means the *retrieval act* of that concept, or the stored particle
/// representing it:
/// - `Orbit`   — the plain W5H2 read/scan (identical to today's default).
/// - `Emit`    — ORBIT with an implicit `hist.event = "BIRTH"` condition
///   ANDed onto WHERE: "show me what was emitted (born) matching this
///   filter." Every document/section/concept-mention particle already
///   carries `hist.event = "BIRTH"` (`DocumentEmitter`), so this reuses
///   real, already-written data — no new particle kind, no mutation.
/// - `Prove`   — "verify this particle's history": same WHO/WHERE/WHY
///   candidate resolution as ORBIT, but each matched entity's WHAT
///   projection is reported once per epoch in its real journal history
///   (oldest to newest), not just its current last-write-wins state.
/// - `Sync`    — "compare our orbit states": same result set as ORBIT,
///   plus an order-independent CRC32 fingerprint folded over the matched
///   entities' KAKIs, so two SYNC calls (e.g. one against a Write Node's
///   Journal, one against a Read Node's materialized generation) can be
///   diffed by the caller to detect replication drift without comparing
///   full row content.
/// - `Witness` — "I attest to this state": same result set as ORBIT,
///   plus a SHA3-256 digest over the matched entities' KAKIs *and* their
///   full projected content, so any later re-run that reproduces the
///   identical digest has proven byte-identical results. A single-node
///   content attestation (a verifiable checksum), not multi-party
///   consensus — no signing key or second witness is involved.
pub use crate::operations::Operation as QueryVerb;

/// A complete parsed HeptaScript v2.0 W5H2 query.
#[derive(Debug, Clone)]
pub struct HeptaQuery {
    /// The Sovereign Operation this query performs. Defaults to `Orbit`.
    pub verb: QueryVerb,

    // v2.0 routing clauses (optional, parsed before WHO)
    pub node:     Option<NodeClause>,
    pub across:   Option<AcrossClause>,

    // v1 core clauses
    pub who:      WhoClause,
    pub what:     Option<WhatClause>,
    pub r#where:  Vec<WhereCondition>,
    pub when:     Option<WhenClause>,
    pub why:      Vec<WhyCondition>,
    pub how:      Option<HowClause>,
    pub how_much: Option<HowMuchClause>,

    // v2.0 filter / intelligence clauses (optional, parsed after WHY)
    pub tier:     Option<TierClause>,
    pub state:    Option<StateClause>,
    pub nash:     Option<NashClause>,
    pub pattern:  Vec<PatternCondition>,
    pub lineage:  Option<LineageClause>,
    pub gate:     Option<GateClause>,
    pub satamu:   Option<SatamuClause>,
    pub orbital:  Option<OrbitalClause>,

    // v2.1 aggregate clauses (optional, parsed after HOW_MUCH)
    pub measure:  Option<MeasureClause>,
    pub gravity:  Option<GravityClause>,

    // ── ŠUMU-UKIN routing ────────────────────────────────────────────────────
    /// Tribe IDs for ŠUMU-UKIN fan-out routing. Empty = all tribes.
    pub tribes: Vec<String>,

    // ── Production execution hints ────────────────────────────────────────────
    /// ANCHOR — which index drives primary candidate generation.
    pub anchor:         AnchorStrategy,
    /// STREAM — emit results via callback instead of collecting into a Vec.
    pub stream:         bool,
    /// DERIVE_STATION — station name to derive from journal at query time (no index).
    pub derive_station: Option<String>,
    /// ABORT_SCAN — safety valve: stop after evaluating this many candidates.
    pub abort_scan:     Option<u64>,
    /// FILTER_ORDER — explicit predicate pipeline; empty = use engine default order.
    pub filter_order:   Vec<FilterStage>,
}

// ── v2.0: NODE clause ─────────────────────────────────────────────────────────

/// NODE clause — specifies which EnkiDB type(s) to query.
/// `NODE ALL` means all seven types + NARUDU + ENKI_PATTERN.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeClause {
    pub targets: Vec<DbTarget>,
}

/// One EnkiDB node type target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbTarget {
    All,
    EnkiDb,       // 7001 — operational OLTP
    EnkiDw,       // 7002 — data warehouse
    EnkiSdb,      // 7003 — security
    EnkiOdb,      // 7004 — operational between-gate
    EnkiQdb,      // 7005 — quantum probability
    EnkiMdb,      // 7006 — master data
    EnkiDdb,      // 7007 — client documentation
    Narudu,       // pattern event bus
    EnkiPattern,  // ENKI-PATTERN registry
}

impl DbTarget {
    /// Well-known TCP port for this database type (0 = event bus / registry, no fixed port).
    pub fn port(&self) -> Option<u16> {
        match self {
            Self::EnkiDb      => Some(7001),
            Self::EnkiDw      => Some(7002),
            Self::EnkiSdb     => Some(7003),
            Self::EnkiOdb     => Some(7004),
            Self::EnkiQdb     => Some(7005),
            Self::EnkiMdb     => Some(7006),
            Self::EnkiDdb     => Some(7007),
            Self::Narudu      => None,
            Self::EnkiPattern => None,
            Self::All         => None,
        }
    }
}

// ── v2.0: ACROSS clause ───────────────────────────────────────────────────────

/// ACROSS clause — cross-BIGRING federation.
#[derive(Debug, Clone, PartialEq)]
pub enum AcrossClause {
    /// `ACROSS BIGRING ClientName` — one named client's BIGRING.
    Bigring(String),
    /// `ACROSS ALL` — broadcast to all connected BIGRINGs.
    All,
}

// ── v2.0: TIER clause ────────────────────────────────────────────────────────

/// TIER clause — storage tier filter for ENKI-PATTERN queries.
#[derive(Debug, Clone, PartialEq)]
pub struct TierClause {
    pub tiers: Vec<StorageTier>,
}

/// Storage tier lifecycle based on age in orbitals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageTier {
    Hot,          // 0–2 orbitals
    Warm,         // 3–29 orbitals
    Cold,         // 30–364 orbitals
    Crystallized, // 365+ orbitals
}

// ── v2.0: STATE clause ───────────────────────────────────────────────────────

/// STATE clause — pattern / particle lifecycle filter.
#[derive(Debug, Clone, PartialEq)]
pub struct StateClause {
    pub states: Vec<LifecycleState>,
}

/// Pattern lifecycle phase (mirrors PatternLifecycle in enkidb-kaki).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleState {
    Emerging,   // confidence ≥ 0.60
    Stable,     // confidence ≥ 0.85
    Canonical,  // promoted by AI Council
    Deprecated, // replaced or superseded
}

// ── v2.0: NASH clause ────────────────────────────────────────────────────────

/// NASH clause — Nash equilibrium constraint.
#[derive(Debug, Clone, PartialEq)]
pub enum NashClause {
    /// `NASH SCORE op value` — numeric threshold filter.
    Score { op: Op, threshold: f64 },
    /// `NASH BREAKING` — only particles where equilibrium is breaking (score > 0.85).
    Breaking,
}

// ── v2.0: PATTERN clause ─────────────────────────────────────────────────────

/// One PATTERN condition (multiple allowed, all ANDed together).
#[derive(Debug, Clone, PartialEq)]
pub enum PatternCondition {
    /// `PATTERN TYPE CrowdFlow` — pattern type filter.
    Type(PatternKind),
    /// `PATTERN CONFIDENCE >= 0.85` — minimum confidence filter.
    Confidence { op: Op, value: f64 },
    /// `PATTERN MAX_CONSTITUENTS <= 126` — constituent count filter.
    MaxConstituents { op: Op, value: u32 },
}

/// Pattern domain type (mirrors PatternType in enkidb-kaki).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternKind {
    CrowdFlow,
    AviationCorridor,
    AviationHolding,
    IndoorHallway,
    IndoorRoom,
    IndoorTransition,
    WaterFlow,
    Custom,
}

// ── v2.0: LINEAGE clause ─────────────────────────────────────────────────────

/// LINEAGE clause — causality chain traversal depth.
#[derive(Debug, Clone, PartialEq)]
pub enum LineageClause {
    /// `LINEAGE DEPTH n` — follow CrossTribe-KAKIs up to n hops.
    Depth(u32),
    /// `LINEAGE FULL` — follow until no more CrossTribe-KAKIs (bounded by 64 hops internally).
    Full,
}

// ── v2.0: GATE clause ────────────────────────────────────────────────────────

/// GATE clause — route results through one or more ETL stations.
#[derive(Debug, Clone, PartialEq)]
pub struct GateClause {
    pub gates: Vec<GateTarget>,
}

/// An ETL station / gate target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateTarget {
    /// Named station (e.g. "DataSteward", "DataCleansing", "DataStructure").
    Named(String),
    /// ALL — pass through every registered station in order.
    All,
}

// ── v2.0: SATAMU clause ──────────────────────────────────────────────────────

/// SATAMU clause — Šàtamu governance override.
#[derive(Debug, Clone, PartialEq)]
pub enum SatamuClause {
    /// `SATAMU REQUIRED` — query must be approved by the Šàtamu chain before results are released.
    Required,
    /// `SATAMU BYPASS "reason"` — override Šàtamu with an audit-logged reason.
    Bypass(String),
}

// ── v2.0: ORBITAL clause ─────────────────────────────────────────────────────

/// ORBITAL clause — intra-day cycle time range.
#[derive(Debug, Clone, PartialEq)]
pub enum OrbitalClause {
    /// `ORBITAL start .. end` — inclusive orbital range.
    Range { start: u64, end: u64 },
    /// `ORBITAL NOW` — current orbital only.
    Now,
}

// ── v1 types (unchanged) ─────────────────────────────────────────────────────

/// WHO clause — defines the entity variable scope.
#[derive(Debug, Clone)]
pub struct WhoClause {
    pub primary:  EntityBinding,
    pub bound_to: Vec<EntityBinding>,
}

/// An entity variable binding: `TribeName.VarName` (e.g. `Citizens.E`).
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBinding {
    pub tribe: String,
    pub var:   String,
}

/// WHAT clause — attribute projection.
#[derive(Debug, Clone)]
pub struct WhatClause {
    pub projections: Vec<Projection>,
}

/// One projection: `VarName[attr1, attr2]` or `VarName[*]` (empty = all).
#[derive(Debug, Clone)]
pub struct Projection {
    pub var:   String,
    pub attrs: Vec<String>,
}

/// One condition in the WHERE clause.
#[derive(Debug, Clone)]
pub struct WhereCondition {
    pub combinator: Combinator,
    pub var:  String,
    pub attr: String,
    pub test: ConditionTest,
}

/// The test applied to the stored EAV value.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionTest {
    Cmp { op: Op, val: HeptaValue },
    Exists,
    NotExists,
}

/// WHEN clause — temporal constraint on journal entries.
#[derive(Debug, Clone, PartialEq)]
pub enum WhenClause {
    AtEpoch(EpochRef),
    BeforeEpoch(u64),
    AfterEpoch(u64),
}

/// Epoch reference: absolute number or the sentinel NOW.
#[derive(Debug, Clone, PartialEq)]
pub enum EpochRef {
    Abs(u64),
    Now,
}

/// WHY clause — lane, quality, or existence constraints.
#[derive(Debug, Clone)]
pub enum WhyCondition {
    Lane          { combinator: Combinator, op: Op, val: LaneValue },
    QualityByte   { combinator: Combinator, op: Op, val: u8 },
    AttrExists    { combinator: Combinator, var: String, attr: String },
    AttrNotExists { combinator: Combinator, var: String, attr: String },
}

/// Sovereign lane classification value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaneValue {
    Gold,
    Silver,
    White,
    Gray,
    Red,
    Black,
}

/// HOW clause — result ordering.
#[derive(Debug, Clone)]
pub struct HowClause {
    pub var:  String,
    pub attr: String,
    pub dir:  SortDir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

/// HOW_MUCH clause — result cardinality.
#[derive(Debug, Clone, PartialEq)]
pub enum HowMuchClause {
    Limit(usize),
    Top { n: usize, var: String, attr: String, dir: SortDir },
}

// ── v2.1: MEASURE / GRAVITY (Anti-SQL aggregate clauses) ─────────────────────
//
// Not new sovereign verbs -- `Operation` (operations.rs) is a deliberately
// closed set of exactly five (ORBIT/EMIT/PROVE/SYNC/WITNESS), asserted by
// its own `exactly_five_operations` test. These are ordinary clauses, same
// footing as HOW_MUCH, that combine with any of the five (almost always the
// default ORBIT).
//
// Real, correctness-checked designs (not the naive versions first proposed):
//   DENSE      -- a real O(N) tally, NEVER wedge-product/pseudoscalar
//                 magnitude. bahyway-algebra's own `wedge_of_dependent_is_zero`
//                 test proves wedging more vectors than the algebra's
//                 dimension (7, for Cl(7)) collapses to zero -- that
//                 approach silently breaks for any real Tribe.
//   FLUX       -- component-wise multivector sum (`Multivector::add`
//                 fold). Reduces to ordinary SUM for scalar attributes.
//   ROTOR_MEAN -- closed-form circular mean (atan2 of Σsin/Σcos) for
//                 rotor-valued attributes (this codebase's `Rotor` is a
//                 simplified single bivector-plane rotor, so the true mean
//                 is one pass, not the iterative Fréchet-mean optimization
//                 a general multi-plane rotor would need); plain arithmetic
//                 mean for scalar attributes.
//   GRAVITY    -- partition into bands (numeric bucket width) or exact-match
//                 groups. Exact-match mode REQUIRES max_groups (a mandatory
//                 safety cap): grouping by a near-unique attribute can grow
//                 the grouping structure toward O(N) memory, same risk any
//                 GROUP BY has in any database. Gate G4 (bahyway-z3) refuses
//                 a GRAVITY clause missing this cap at design time.

/// MEASURE clause — one aggregate value computed over the matched set.
#[derive(Debug, Clone, PartialEq)]
pub enum MeasureClause {
    /// `MEASURE DENSE` — real count of matched particles.
    Dense,
    /// `MEASURE FLUX VarName[attr]` — component-wise sum of the attribute.
    Flux { var: String, attr: String },
    /// `MEASURE ROTOR_MEAN VarName[attr]` — circular/arithmetic mean of the attribute.
    RotorMean { var: String, attr: String },
}

/// GRAVITY clause — partition the matched set into groups.
#[derive(Debug, Clone, PartialEq)]
pub struct GravityClause {
    pub var:  String,
    pub attr: String,
    pub mode: GravityMode,
}

/// How GRAVITY partitions the matched set.
#[derive(Debug, Clone, PartialEq)]
pub enum GravityMode {
    /// `BAND width` — numeric bucket partition; group count is inherently
    /// bounded by (attribute range / width), safe at any N.
    Band(f64),
    /// `MAX_GROUPS n` — exact-value partition, capped: refuses (at query
    /// execution, and earlier at Gate G4 design time) once more than `n`
    /// distinct groups would be produced, rather than growing unbounded.
    ExactMatchCapped(usize),
}

// ── Production execution hints (1B+ particles, <1 second) ────────────────────

/// ANCHOR clause — selects the primary index strategy for candidate generation.
///
/// Choosing the right anchor eliminates O(n) scans against 1B+ particles.
/// `Auto` lets the engine inspect query predicates and pick the best strategy.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AnchorStrategy {
    /// Engine auto-selects based on available predicates and index sizes.
    #[default]
    Auto,
    /// Drive from SurrogateMap (KAKI→u32) + NatiruIndex orbital pruning.
    SurrogateTime,
    /// Drive from lifecycle-state bitmap first, then derive station from journal.
    StateStation,
    /// Drive from HeptaShellIndex (E7 7D spatial lattice) as primary candidate source.
    E7First,
    /// Full sequential journal scan — use only when no index can narrow the set.
    FullScan,
}

/// FILTER_ORDER clause — explicit ordering of the predicate pipeline.
///
/// The engine applies filter stages left-to-right; each stage narrows the
/// candidate set passed to the next.  Default order if omitted:
/// `[SurrogateRange, OrbitalRange, State, DeriveStation, Lane, QualityByte, EavAttr, E7Region]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterStage {
    /// Binary-search SurrogateMap to resolve KAKI predicates to surrogate IDs.
    SurrogateRange,
    /// NatiruIndex orbital-bucket pruning (eliminates non-temporal surrogates).
    OrbitalRange,
    /// Lifecycle-state bitmap filter (Emerging / Stable / Canonical / Deprecated).
    State,
    /// Derive BeeMDM station from journal at query time (no extra index required).
    DeriveStation,
    /// Lane classification filter (GOLD / SILVER / WHITE / GRAY / RED / BLACK).
    Lane,
    /// Quality-byte threshold filter.
    QualityByte,
    /// EavExactIndex attribute-value lookup (Xor8 fast negative + binary search).
    EavAttr,
    /// HeptaShellIndex E7 spatial region filter (126 kissing-number neighbours).
    E7Region,
}

impl HeptaQuery {
    /// Append a non-default filter stage to the explicit pipeline.
    ///
    /// Called by the parser when a FILTER_ORDER clause is encountered.
    pub fn push_filter_stage(&mut self, stage: FilterStage) {
        self.filter_order.push(stage);
    }
}
