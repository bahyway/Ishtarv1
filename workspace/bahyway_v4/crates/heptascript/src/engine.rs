//! HeptaScript v2.0 W5H2 EAV-native executor.
//!
//! ## v1 evaluation pipeline (single-node journal scan):
//!   1. WHO      → enumerate candidate IdentityKakis from journal
//!   2. WHEN     → filter journal entries by epoch boundary
//!   3. WHERE    → evaluate EAV conditions against decoded AkkValues
//!   4. WHY      → apply lane / quality constraints
//!   5. WHAT     → project requested attribute values
//!   6. HOW      → sort the result set
//!   7. HOW_MUCH → truncate to requested cardinality
//!
//! ## v2.0 additions (recorded in QueryPlan, dispatched by caller):
//!   NODE     → target DB type(s) — carried in QueryPlan.node_targets
//!   ACROSS   → cross-BIGRING routing — carried in QueryPlan.across
//!   TIER     → storage tier filter — applied by ENKI-PATTERN dispatcher
//!   STATE    → lifecycle filter — applied by ENKI-PATTERN dispatcher
//!   NASH     → Nash equilibrium check — applied post-fetch
//!   PATTERN  → pattern conditions — routed to enki-pattern crate
//!   LINEAGE  → causality depth — triggers CausalityEngine after fetch
//!   GATE     → ETL station chain — applied by the Eridu scheduler
//!   SATAMU   → governance gate — checked before result release
//!   ORBITAL  → orbital range filter — additional temporal dimension
//!
//! The `execute()` function handles the v1 journal scan. The `QueryPlan`
//! returned alongside the result carries the v2.0 directives so the
//! caller (enkidu-protocol dispatcher) can act on them.

use std::collections::HashMap;

use akkvalue::{codec::decode as codec_decode, AkkValue};
use bahyway_crc::crc16;
use enkidb_journal::entry::JournalEntry;
use enkidb_journal::Journal;
use enkidb_kaki::IdentityKaki;

use crate::query::*;

// ── Public result types ───────────────────────────────────────────────────────

/// One entity that matched all W5H2 conditions, with projected attribute values.
#[derive(Debug, Clone)]
pub struct MatchedEntity {
    pub entity: IdentityKaki,
    /// Projected `(attr_name, value)` pairs from the WHAT clause.
    pub projected: Vec<(String, AkkValue)>,
    /// Populated only for `QueryVerb::Prove` — the same WHAT projection
    /// re-evaluated after each journal entry in this entity's real
    /// history, oldest first, instead of only the final last-write-wins
    /// state `projected` holds. Empty for every other verb.
    pub history: Vec<EpochSnapshot>,
}

/// One epoch's WHAT-projected state, as `QueryVerb::Prove` reports it.
#[derive(Debug, Clone)]
pub struct EpochSnapshot {
    pub epoch: u32,
    pub attrs: Vec<(String, AkkValue)>,
}

/// Result of executing a HeptaScript W5H2 query.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub matched: Vec<MatchedEntity>,
    /// Total candidate entities evaluated before filtering.
    pub evaluated: usize,
    /// v2.0 query plan — carries routing / governance directives for the caller.
    pub plan: QueryPlan,
    /// The Sovereign Operation this result came from — copied straight
    /// from the query, not inferred from which optional fields below are
    /// `Some`, so a caller can tell EMIT/PROVE/SYNC/WITNESS apart from
    /// ORBIT even when `matched` is empty (e.g. a SYNC query that matched
    /// nothing still legitimately reports a fingerprint of the empty set).
    pub verb: QueryVerb,
    /// Populated only for `QueryVerb::Sync` — an order-independent CRC32
    /// fingerprint (hex) folded over every matched entity's KAKI bytes.
    /// Two SYNC calls whose matched *sets* agree produce the same
    /// fingerprint regardless of scan order; a changed fingerprint means
    /// the set of matching particles differs.
    pub state_fingerprint: Option<String>,
    /// Populated only for `QueryVerb::Witness` — a SHA3-256 digest (hex)
    /// over every matched entity's KAKI bytes *and* its full projected
    /// content, domain-separated with `b"HEPT_WITNESS"` (same convention
    /// `enkidb-replication::event` uses for its own frame digests). A
    /// single-node content attestation: reproduces identically only if
    /// the exact same rows with the exact same values come back again.
    pub witness_digest: Option<String>,
    /// Populated only when the query carries a `MEASURE` clause with no
    /// `GRAVITY` partition — the single aggregate value computed over the
    /// *entire* matched set (before `HOW_MUCH` truncates the returned
    /// `matched` rows; an aggregate must reflect every match, not just the
    /// rows the caller chose to display).
    pub measured: Option<MeasureValue>,
    /// Populated only when the query carries a `GRAVITY` clause — the
    /// matched set partitioned into groups, each with its own `MEASURE`
    /// (or a plain count, when no `MEASURE` clause is present).
    pub grouped: Option<GravityResult>,
    /// True if `ABORT_SCAN N` fired: fewer candidates were evaluated than
    /// the full input set because the safety valve capped the scan. See
    /// `execute_over_histories` for where the cap is actually applied —
    /// this mirrors `StreamStats::aborted` (the separate, not-yet-wired-
    /// into-serving streaming evaluator) so `ABORT_SCAN` behaves
    /// identically regardless of which evaluation path a caller reaches.
    pub aborted: bool,
}

/// One computed aggregate value — the result of a `MEASURE` clause,
/// evaluated over one group (`GRAVITY` present) or the whole matched set
/// (`GRAVITY` absent).
#[derive(Debug, Clone, PartialEq)]
pub enum MeasureValue {
    /// `MEASURE DENSE` — a real O(N) tally of matched particles. Never a
    /// wedge-product/pseudoscalar magnitude — `bahyway-algebra`'s own
    /// `wedge_of_dependent_is_zero` test proves that approach collapses to
    /// zero once more vectors than the algebra's dimension are wedged, so
    /// it silently breaks as a count for any real Tribe beyond N=7.
    Dense(usize),
    /// `MEASURE FLUX E[attr]` — scalar sum of the attribute's numeric
    /// value. The EAV store holds only scalar `AkkValue`s (no
    /// multivector-typed attribute exists to sum component-wise), so this
    /// is `Multivector::add`'s fold degenerated to ordinary addition —
    /// exactly SUM, for the values this schema can actually hold.
    Flux(f64),
    /// `MEASURE ROTOR_MEAN E[attr]` — closed-form circular mean,
    /// `atan2(Σsin θ, Σcos θ)`, treating the attribute's numeric value as
    /// an angle in radians. One pass, because this codebase's `Rotor` is a
    /// simplified single bivector-plane rotor, not the general multi-plane
    /// rotor a Fréchet-mean optimization would be needed for.
    RotorMean(f64),
}

/// A single `GRAVITY` partition and its `MEASURE`.
#[derive(Debug, Clone, PartialEq)]
pub struct GravityGroup {
    pub key: GravityKey,
    /// Number of matched particles that fell into this group — always
    /// populated regardless of which `MeasureValue` variant `measure` is,
    /// since group membership is tracked independently of the aggregate.
    pub count: usize,
    pub measure: MeasureValue,
}

/// The group a particle was partitioned into by a `GRAVITY` clause.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GravityKey {
    /// `BAND width` — bucket index `floor(value / width)`. Inherently
    /// bounded by `(attribute range / width)`, so this key space stays
    /// safe at any N without an explicit cap.
    Band(i64),
    /// `MAX_GROUPS n` — canonical text form of the exact attribute value.
    Exact(String),
    /// The `GRAVITY` attribute was absent on this particle.
    Missing,
    /// `MAX_GROUPS n` was reached — every additional *new* distinct value
    /// folds into this single sentinel bucket instead of growing group
    /// memory further. This is the runtime enforcement of the cap Gate G4
    /// checks for at design time: memory stays O(max_groups) regardless of
    /// the attribute's true cardinality.
    Overflow,
}

/// Result of a `GRAVITY` clause: the partitioned groups plus whether the
/// `MAX_GROUPS` cap was actually hit during this run.
#[derive(Debug, Clone, PartialEq)]
pub struct GravityResult {
    pub groups: Vec<GravityGroup>,
    /// True if `MAX_GROUPS` was reached and one or more additional distinct
    /// values were folded into `GravityKey::Overflow` rather than growing
    /// the group set further.
    pub capped: bool,
}

/// v2.0 query plan extracted from the v2.0 clauses.
/// The local `execute()` function performs the journal scan; the plan
/// tells the upstream dispatcher (enkidu-protocol) where else to send
/// the query and what post-processing to apply.
#[derive(Debug, Clone, Default)]
pub struct QueryPlan {
    /// NODE targets — empty means "local node only" (v1 behaviour).
    pub node_targets: Vec<crate::query::DbTarget>,
    /// ACROSS directive.
    pub across: Option<crate::query::AcrossClause>,
    /// TIER filter (ENKI-PATTERN dispatcher).
    pub tiers: Vec<crate::query::StorageTier>,
    /// STATE filter (ENKI-PATTERN dispatcher).
    pub states: Vec<crate::query::LifecycleState>,
    /// NASH constraint (post-fetch filter).
    pub nash: Option<crate::query::NashClause>,
    /// PATTERN conditions (enki-pattern crate).
    pub pattern: Vec<crate::query::PatternCondition>,
    /// LINEAGE depth (CausalityEngine).
    pub lineage: Option<crate::query::LineageClause>,
    /// GATE chain (Eridu scheduler).
    pub gates: Vec<crate::query::GateTarget>,
    /// SATAMU governance gate.
    pub satamu: Option<crate::query::SatamuClause>,
    /// ORBITAL range filter.
    pub orbital: Option<crate::query::OrbitalClause>,
    /// Chosen anchor strategy (resolved at execute time if Auto).
    pub anchor: crate::query::AnchorStrategy,
    /// Station name to derive from journal (no extra index required).
    pub derive_station: Option<String>,
    /// Abort after evaluating this many candidates (safety valve).
    pub abort_scan: Option<u64>,
    /// Explicit filter pipeline order (empty = engine default).
    pub filter_order: Vec<crate::query::FilterStage>,
}

impl QueryPlan {
    fn from_query(q: &HeptaQuery) -> Self {
        Self {
            node_targets: q
                .node
                .as_ref()
                .map(|n| n.targets.clone())
                .unwrap_or_default(),
            across: q.across.clone(),
            tiers: q.tier.as_ref().map(|t| t.tiers.clone()).unwrap_or_default(),
            states: q
                .state
                .as_ref()
                .map(|s| s.states.clone())
                .unwrap_or_default(),
            nash: q.nash.clone(),
            pattern: q.pattern.clone(),
            lineage: q.lineage.clone(),
            gates: q.gate.as_ref().map(|g| g.gates.clone()).unwrap_or_default(),
            satamu: q.satamu.clone(),
            orbital: q.orbital.clone(),
            anchor: q.anchor.clone(),
            derive_station: q.derive_station.clone(),
            abort_scan: q.abort_scan,
            filter_order: q.filter_order.clone(),
        }
    }

    /// Returns true if this query needs to be dispatched beyond the local node.
    pub fn is_federated(&self) -> bool {
        !self.node_targets.is_empty() || self.across.is_some()
    }

    /// Returns true if Šàtamu governance must approve before results are released.
    pub fn satamu_required(&self) -> bool {
        matches!(self.satamu, Some(crate::query::SatamuClause::Required))
    }
}

// ── MEASURE / GRAVITY aggregation ─────────────────────────────────────────────
//
// The index (WHO/WHERE/WHY/anchor pruning, above) determines *which*
// particles this ever touches; this section determines *what* gets
// computed once over that already-bounded set. `Aggregator::observe` is
// called once per particle that survives WHERE/WHY, alongside — not
// instead of — the existing `matched` projection, so aggregate values
// always reflect the *complete* matched set even when `HOW_MUCH` later
// truncates the returned rows.

#[derive(Default)]
struct GroupAccumulator {
    count: usize,
    flux_sum: f64,
    rotor_sin_sum: f64,
    rotor_cos_sum: f64,
    rotor_n: usize,
}

struct Aggregator<'q> {
    measure: Option<&'q MeasureClause>,
    gravity: Option<&'q GravityClause>,
    global: GroupAccumulator,
    groups: std::collections::HashMap<GravityKey, GroupAccumulator>,
    /// Count of genuinely distinct `GravityKey::Exact` groups admitted so
    /// far — tracked incrementally so the `MAX_GROUPS` check stays O(1)
    /// amortized per particle instead of re-scanning `groups` (which would
    /// turn a capped GRAVITY into an O(N × max_groups) scan, the exact
    /// class of blowup the cap exists to prevent).
    exact_groups: usize,
    capped: bool,
}

impl<'q> Aggregator<'q> {
    fn new(measure: Option<&'q MeasureClause>, gravity: Option<&'q GravityClause>) -> Self {
        Aggregator {
            measure,
            gravity,
            global: GroupAccumulator::default(),
            groups: std::collections::HashMap::new(),
            exact_groups: 0,
            capped: false,
        }
    }

    fn observe(&mut self, eav: &EavMap) {
        let measure = self.measure;
        match self.gravity {
            None => Self::accumulate(&mut self.global, measure, eav),
            Some(g) => {
                let key = self.group_key(g, eav);
                let acc = self.groups.entry(key).or_default();
                Self::accumulate(acc, measure, eav);
            }
        }
    }

    fn group_key(&mut self, g: &GravityClause, eav: &EavMap) -> GravityKey {
        match &g.mode {
            GravityMode::Band(width) => match get_val(eav, &g.attr).and_then(as_f64) {
                Some(v) => GravityKey::Band((v / width).floor() as i64),
                None => GravityKey::Missing,
            },
            GravityMode::ExactMatchCapped(max) => match get_val(eav, &g.attr) {
                Some(v) => {
                    let key = GravityKey::Exact(canonical_key(v));
                    if self.groups.contains_key(&key) {
                        key
                    } else if self.exact_groups < *max {
                        self.exact_groups += 1;
                        key
                    } else {
                        self.capped = true;
                        GravityKey::Overflow
                    }
                }
                None => GravityKey::Missing,
            },
        }
    }

    fn accumulate(acc: &mut GroupAccumulator, measure: Option<&MeasureClause>, eav: &EavMap) {
        acc.count += 1;
        match measure {
            None | Some(MeasureClause::Dense) => {}
            Some(MeasureClause::Flux { attr, .. }) => {
                if let Some(v) = get_val(eav, attr).and_then(as_f64) {
                    acc.flux_sum += v;
                }
            }
            Some(MeasureClause::RotorMean { attr, .. }) => {
                if let Some(v) = get_val(eav, attr).and_then(as_f64) {
                    acc.rotor_sin_sum += v.sin();
                    acc.rotor_cos_sum += v.cos();
                    acc.rotor_n += 1;
                }
            }
        }
    }

    fn finalize_one(acc: &GroupAccumulator, measure: Option<&MeasureClause>) -> MeasureValue {
        match measure {
            None | Some(MeasureClause::Dense) => MeasureValue::Dense(acc.count),
            Some(MeasureClause::Flux { .. }) => MeasureValue::Flux(acc.flux_sum),
            Some(MeasureClause::RotorMean { .. }) => {
                if acc.rotor_n == 0 {
                    MeasureValue::RotorMean(0.0)
                } else {
                    MeasureValue::RotorMean(acc.rotor_sin_sum.atan2(acc.rotor_cos_sum))
                }
            }
        }
    }

    /// Ordering key for `GravityGroup` output — bands sort numerically,
    /// exact values sort lexicographically, `Missing`/`Overflow` sort last
    /// — so results are deterministic regardless of `HashMap` iteration
    /// order.
    fn key_order(k: &GravityKey) -> (u8, i64, &str) {
        match k {
            GravityKey::Band(b) => (0, *b, ""),
            GravityKey::Exact(s) => (1, 0, s.as_str()),
            GravityKey::Missing => (2, 0, ""),
            GravityKey::Overflow => (3, 0, ""),
        }
    }

    fn finish(self) -> (Option<MeasureValue>, Option<GravityResult>) {
        if self.gravity.is_none() {
            let measured = self
                .measure
                .map(|_| Self::finalize_one(&self.global, self.measure));
            return (measured, None);
        }
        let measure = self.measure;
        let mut groups: Vec<GravityGroup> = self
            .groups
            .into_iter()
            .map(|(key, acc)| GravityGroup {
                key,
                count: acc.count,
                measure: Self::finalize_one(&acc, measure),
            })
            .collect();
        groups.sort_by(|a, b| Self::key_order(&a.key).cmp(&Self::key_order(&b.key)));
        (
            None,
            Some(GravityResult {
                groups,
                capped: self.capped,
            }),
        )
    }
}

fn as_f64(v: &AkkValue) -> Option<f64> {
    match v {
        AkkValue::Int(n) => Some(*n as f64),
        AkkValue::Float(f) => Some(*f),
        _ => None,
    }
}

/// Canonical text form of an `AkkValue` for use as an exact-match
/// `GravityKey` — two particles with an equal attribute value must hash
/// and compare equal as group keys.
fn canonical_key(v: &AkkValue) -> String {
    match v {
        AkkValue::Text(s) => s.clone(),
        AkkValue::Int(n) => n.to_string(),
        AkkValue::Float(f) => f.to_string(),
        AkkValue::Bool(b) => b.to_string(),
        other => format!("{other:?}"),
    }
}

// ── Stream execution (production: 1B+ particles, <1 second) ──────────────────

/// Statistics returned by `execute_stream`.
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Number of entities passed to the callback.
    pub matched: usize,
    /// Number of candidate entities evaluated (after anchor pre-filter).
    pub evaluated: usize,
    /// True if ABORT_SCAN fired before the full candidate set was evaluated.
    pub aborted: bool,
    /// The resolved query plan (for the caller / dispatcher).
    pub plan: QueryPlan,
}

/// Execute a `HeptaQuery` in streaming mode.
///
/// Instead of accumulating results in a `Vec`, each matching entity is passed
/// to `callback` as it is found.  Return `ControlFlow::Break(())` from the
/// callback to stop early (e.g. once LIMIT is satisfied upstream).
///
/// The function respects:
/// - `ANCHOR` — skip full scan when an index can narrow candidates
/// - `ORBITAL` range — prune by intra-day orbital before EAV evaluation
/// - `ABORT_SCAN N` — safety valve: stop after N candidates regardless
/// - `HOW_MUCH LIMIT` — stop after emitting N matches (when anchor is trivial)
///
/// HOW (sort) is **not** applied in stream mode — the caller is responsible
/// for maintaining a bounded priority queue if ordered output is required.
pub fn execute_stream<F>(query: &HeptaQuery, journal: &Journal, mut callback: F) -> StreamStats
where
    F: FnMut(MatchedEntity) -> core::ops::ControlFlow<()>,
{
    use core::ops::ControlFlow;

    let plan = QueryPlan::from_query(query);
    let epoch_filter = epoch_filter_from_when(query.when.as_ref());
    let abort_limit = query.abort_scan;
    let stream_limit = match &query.how_much {
        Some(HowMuchClause::Limit(n)) => Some(*n),
        _ => None,
    };

    // Orbital range for fast pre-pruning (ORBITAL clause or WHEN).
    let orbital_range = orbital_range_from_query(query);

    let candidates = journal.all_particles();

    let mut evaluated = 0usize;
    let mut matched = 0usize;
    let mut aborted = false;

    'outer: for entity in &candidates {
        // ABORT_SCAN safety valve
        if let Some(limit) = abort_limit {
            if evaluated as u64 >= limit {
                aborted = true;
                break 'outer;
            }
        }

        let history = journal.read_particle_history(entity);
        if history.is_empty() {
            continue;
        }

        // Fast orbital pre-filter: skip if no entry falls in the orbital range.
        if let Some((orb_lo, orb_hi)) = orbital_range {
            let any_in_range = history.iter().any(|e| {
                let orb = e.epoch as u64;
                orb >= orb_lo && orb <= orb_hi
            });
            if !any_in_range {
                evaluated += 1;
                continue;
            }
        }

        evaluated += 1;

        let eav = build_eav_map(&history, &epoch_filter);

        if !eval_where(&query.r#where, &eav) {
            continue;
        }
        if !eval_why(&query.why, &eav) {
            continue;
        }

        let projected = project_what(&query.what, &eav);
        let entity_match = MatchedEntity {
            entity: *entity,
            projected,
            history: Vec::new(),
        };

        matched += 1;
        if callback(entity_match) == ControlFlow::Break(()) {
            break 'outer;
        }

        // LIMIT satisfied — stop early
        if let Some(lim) = stream_limit {
            if matched >= lim {
                break 'outer;
            }
        }
    }

    StreamStats {
        matched,
        evaluated,
        aborted,
        plan,
    }
}

// ── Orbital range helper ──────────────────────────────────────────────────────

/// Extract a (lo, hi) orbital range from the query's ORBITAL clause.
/// Returns None if no orbital filtering is requested.
///
/// `pub`: called from `indexed::candidates_from_indexes` (in-crate) and from
/// `enkidb_readnode::cached::CachedReadNode` (out-of-crate, the EnkiDDB/
/// EnkiMDB read-server's real hot path) so the `ORBITAL NOW` placeholder
/// mapping (documented below) is defined in exactly one place, never
/// reimplemented a second time per consumer.
pub fn orbital_range_from_query(q: &HeptaQuery) -> Option<(u64, u64)> {
    match q.orbital.as_ref()? {
        OrbitalClause::Range { start, end } => Some((*start, *end)),
        OrbitalClause::Now => {
            // Map NOW to a ±5 orbital window around the current moment.
            // In production this would read from a shared atomic clock; here
            // we use a reasonable default that keeps tests deterministic.
            let now: u64 = 0; // placeholder — production binds a real clock
            Some((now.saturating_sub(5), now + 5))
        }
    }
}

/// Execute a `HeptaQuery` against the in-memory journal.
///
/// This always evaluates every particle in the journal (`journal
/// .all_particles()`) — the correct, always-available fallback. Callers
/// that have a pre-built [`crate::indexed::HeptaIndexes`] snapshot should
/// prefer [`crate::indexed::execute_indexed`], which prunes the candidate
/// set via the EAV exact-match index before any per-particle history read
/// whenever the query allows it, and otherwise delegates back to this exact
/// function — so results are always identical, only the candidate-selection
/// cost differs.
pub fn execute(query: &HeptaQuery, journal: &Journal) -> QueryResult {
    // When ABORT_SCAN is set, enumerate at most that many candidates in the
    // first place (`Journal::all_particles_capped`) instead of always
    // paying `all_particles()`'s full O(total distinct particles) scan.
    // `execute_over` already caps the expensive per-candidate history
    // reads, but at real scale (10M+ particles) the candidate-enumeration
    // scan itself dominates -- this is the fix for that, one level
    // earlier. See `Journal::all_particles_capped`'s doc comment.
    if let Some(limit) = query.abort_scan {
        let (candidates, aborted) = journal.all_particles_capped(limit as usize);
        let histories = histories_for(journal, &candidates);
        let evaluated = histories.len();
        return evaluate_capped(query, &histories, evaluated, aborted);
    }
    let candidates = journal.all_particles();
    execute_over(query, journal, &candidates)
}

/// Below this many candidates, per-candidate `Journal::read_particle_
/// history` lookups stay cheap (it's priced for an occasional live
/// lookup, per its own doc comment). At or above it, one grouped O(n)
/// pass over the whole journal is cheaper overall -- the same trade-off
/// `indexed::build_indexes` already makes for its own O(n) journal read
/// (see that function's doc comment for the real stall this exact
/// anti-pattern caused there).
const BULK_HISTORY_THRESHOLD: usize = 1_000;

/// Read every candidate's history from `journal`. Shared by `execute`'s
/// ABORT_SCAN-capped enumeration path and `execute_over`, so both build
/// `(entity, history)` pairs identically.
///
/// FIXED (found live running the real Phase-1 E-004 10M-particle test,
/// docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md): this used to
/// always call `journal.read_particle_history()` once per candidate --
/// each call rescans its whole shard partition, so a large candidate set
/// (post-ORBITAL/EAV pruning, still realistically hundreds of thousands
/// at 10M-particle scale) turned an indexed query meant to finish in
/// under a second into one that hadn't finished after 11+ minutes. This
/// is the exact same O(candidates × shard_size) anti-pattern
/// `indexed::build_indexes` already found and fixed for its own O(n)
/// pass -- that fix was never carried over to the query-execution path
/// until now. Below `BULK_HISTORY_THRESHOLD`, per-candidate lookups are
/// kept (cheap, and avoids paying a full journal scan for a
/// two-candidate query); at or above it, one grouped pass over
/// `journal.all_entries()` builds every candidate's history in a single
/// O(journal_size) pass instead.
fn histories_for(
    journal: &Journal,
    candidates: &[IdentityKaki],
) -> Vec<(IdentityKaki, Vec<JournalEntry>)> {
    if candidates.len() < BULK_HISTORY_THRESHOLD {
        return candidates
            .iter()
            .map(|entity| {
                let history = journal
                    .read_particle_history(entity)
                    .into_iter()
                    .cloned()
                    .collect();
                (*entity, history)
            })
            .collect();
    }

    // FIXED (found live, same real E-004 run): building history for
    // every particle in the whole journal -- not just the real,
    // index-pruned candidate set -- meant a query pruned down to, say,
    // 1M of 10M particles still paid the clone/allocate cost for all
    // 10M. The journal scan itself stays O(journal_size) (an entry could
    // belong to any candidate, so every entry must be looked at once),
    // but only entries whose target is actually in `candidates` are
    // cloned and kept.
    let wanted: std::collections::HashSet<[u8; 16]> = candidates.iter().map(|e| *e.bytes()).collect();
    let mut by_target: HashMap<[u8; 16], Vec<JournalEntry>> = HashMap::with_capacity(wanted.len());
    for entry in journal.all_entries() {
        let target = entry.target_kaki.bytes();
        if wanted.contains(target) {
            by_target.entry(*target).or_default().push(entry.clone());
        }
    }
    candidates
        .iter()
        .map(|entity| {
            let history = by_target.remove(entity.bytes()).unwrap_or_default();
            (*entity, history)
        })
        .collect()
}

/// Shared evaluation core: run `query` against exactly `candidates` (not
/// necessarily every particle in the journal), reading each candidate's
/// history from `journal`. `execute()` calls this with the full particle
/// set; `indexed::execute_indexed()` calls it with an index-pruned subset.
/// Extracted so the two callers can never drift in WHERE/WHY/WHAT/HOW/
/// HOW_MUCH semantics — there is exactly one evaluation implementation.
pub fn execute_over(
    query: &HeptaQuery,
    journal: &Journal,
    candidates: &[IdentityKaki],
) -> QueryResult {
    // ABORT_SCAN must cap `candidates` *before* any `read_particle_history`
    // call, not after: at real scale (10M+ particles) the per-particle
    // history read is the expensive part, and building histories for every
    // candidate first (as this used to do, delegating straight to
    // `execute_over_histories`) meant a low ABORT_SCAN only trimmed the
    // post-hoc evaluated count while still paying the full O(n) I/O cost --
    // exactly the cost the safety valve exists to bound. See
    // `abort_scan_cap` / `evaluate_capped`.
    let (eval_cap, aborted) = abort_scan_cap(query.abort_scan, candidates.len());
    let candidates = &candidates[..eval_cap];
    let histories = histories_for(journal, candidates);
    evaluate_capped(query, &histories, eval_cap, aborted)
}

/// Shared evaluation core, decoupled from `Journal`: run `query` against
/// pre-fetched `(entity, history)` pairs directly. [`execute_over`]
/// (Journal-backed, used by [`execute`] and [`crate::indexed::execute_indexed`])
/// and a Read Node's own reconstructed histories (`enkidb_readnode`, which
/// decodes history straight from its Data Files) both go through this —
/// so a Read Node never needs to stuff reconstructed entries into a
/// throwaway `Journal` just to reuse this evaluation logic, and never
/// inherits `Journal::read_particle_history`'s cost profile (tuned for
/// the Write Node's occasional live lookup, not bulk Read Node query
/// serving). There is exactly one evaluation implementation regardless of
/// where the history came from.
pub fn execute_over_histories(
    query: &HeptaQuery,
    histories: &[(IdentityKaki, Vec<JournalEntry>)],
) -> QueryResult {
    // ABORT_SCAN safety valve: cap how many candidates this call will even
    // look at, regardless of how many match. Previously only honored by
    // the separate, not-actually-served `execute_stream` -- every real
    // caller (execute/execute_over/indexed::execute_indexed) went through
    // this function, which ignored `query.abort_scan` entirely, so the
    // clause parsed but never did anything. Fixed here, once, in the one
    // real evaluation implementation, rather than by routing callers
    // through the separate streaming evaluator (which lacks EMIT/MEASURE/
    // GRAVITY/HOW support and would be a regression, not a fix).
    //
    // This only bounds CPU-bound evaluation over `histories`, which the
    // caller has already fetched into memory (e.g. a Read Node's own
    // reconstructed histories) -- it cannot cap I/O that already happened
    // before this function was called. `execute_over` (Journal-backed)
    // applies the same cap earlier, to `candidates`, so the I/O itself is
    // bounded too; see its doc comment.
    let (eval_cap, aborted) = abort_scan_cap(query.abort_scan, histories.len());
    evaluate_capped(query, &histories[..eval_cap], eval_cap, aborted)
}

/// Given the real candidate count `total` and an optional `ABORT_SCAN N`
/// cap, returns `(how many to evaluate, whether that's fewer than `total`)`.
fn abort_scan_cap(abort_scan: Option<u64>, total: usize) -> (usize, bool) {
    match abort_scan {
        Some(limit) if (limit as usize) < total => (limit as usize, true),
        _ => (total, false),
    }
}

/// The one real evaluation implementation: WHERE/WHY/WHAT/HOW/HOW_MUCH/
/// MEASURE/GRAVITY semantics over `histories`. The caller has already
/// trimmed `histories` to the ABORT_SCAN cap (via `abort_scan_cap`) and
/// passes the resulting `evaluated`/`aborted` through verbatim -- this
/// function never re-derives them, so it can't disagree with whichever
/// caller (`execute_over_histories` or `execute_over`) did the capping.
fn evaluate_capped(
    query: &HeptaQuery,
    histories: &[(IdentityKaki, Vec<JournalEntry>)],
    evaluated: usize,
    aborted: bool,
) -> QueryResult {
    let epoch_filter = epoch_filter_from_when(query.when.as_ref());

    // EMIT ANDs an implicit `hist.event = "BIRTH"` condition onto WHERE —
    // "show me what was emitted (born) matching this filter." Built once
    // here, not per-candidate; every other verb evaluates the user's WHERE
    // unchanged. This only ever narrows the candidate set further, so it
    // is safe to apply after a Read Node's own index-based pruning (which
    // still runs against the query's own first WHERE condition, untouched).
    let emit_condition = WhereCondition {
        combinator: Combinator::And,
        var: query.who.primary.var.clone(),
        attr: "hist.event".to_string(),
        test: ConditionTest::Cmp {
            op: Op::Eq,
            val: HeptaValue::Text("BIRTH".to_string()),
        },
    };
    let effective_where: Vec<WhereCondition> = match query.verb {
        QueryVerb::Emit => {
            let mut conds = query.r#where.clone();
            conds.push(emit_condition);
            conds
        }
        _ => query.r#where.clone(),
    };

    // Early-exit limit: stop scanning after N matches for HOW_MUCH LIMIT N.
    // Without this, LIMIT 100 still scans all 80,272 entries before truncating.
    //
    // Only safe when nothing downstream needs to rank the full candidate
    // set first: `HOW BY <attr>` sorts the surviving matches at line
    // ~380, and `HOW_MUCH TOP n BY <attr>` sorts its own candidates
    // internally right before truncating — pre-truncating to N *before*
    // either sort would hand both an arbitrary N-subset (whatever
    // `histories` iteration order produced) to sort among itself, not
    // the true top-N of the complete match set. A plain `HOW_MUCH LIMIT
    // n` with no HOW makes no ordering promise at all, so taking the
    // first N matches encountered is a valid, complete answer.
    // A MEASURE/GRAVITY aggregate must reflect the *complete* matched set,
    // never just the first N encountered — so the early-exit optimization
    // (safe for a plain, unordered `HOW_MUCH LIMIT n`) is disabled whenever
    // either clause is present, same as it already is whenever `HOW` needs
    // the full set to sort.
    let has_aggregate = query.measure.is_some() || query.gravity.is_some();
    let early_limit: usize = if query.how.is_some() || has_aggregate {
        usize::MAX
    } else {
        match &query.how_much {
            Some(HowMuchClause::Limit(n)) => *n,
            Some(HowMuchClause::Top { .. }) => usize::MAX,
            None => usize::MAX,
        }
    };

    let want_history = query.verb == QueryVerb::Prove;

    // ORBITAL range — same "any entry falls in range" semantics
    // `execute_stream`'s pre-filter already uses. Previously this clause
    // was only ever applied by `execute_stream`, which no real caller
    // dispatches through (`execute`/`execute_over`/`execute_over_histories`
    // — the only paths any server actually serves — all go through this
    // function instead) — so `ORBITAL start .. end` parsed but silently
    // matched everything, regardless of range. Fixed here, once, in the
    // one real evaluation implementation, same pattern as the ABORT_SCAN
    // fix above.
    let orbital_range = orbital_range_from_query(query);

    let mut aggregator = Aggregator::new(query.measure.as_ref(), query.gravity.as_ref());

    let mut matched: Vec<MatchedEntity> = histories
        .iter()
        .filter_map(|(entity, history)| {
            if history.is_empty() {
                return None;
            }

            if let Some((lo, hi)) = orbital_range {
                let any_in_range = history.iter().any(|e| {
                    let orb = e.epoch as u64;
                    orb >= lo && orb <= hi
                });
                if !any_in_range {
                    return None;
                }
            }

            let refs: Vec<&JournalEntry> = history.iter().collect();
            let eav = build_eav_map(&refs, &epoch_filter);

            if !eval_where(&effective_where, &eav) {
                return None;
            }
            if !eval_why(&query.why, &eav) {
                return None;
            }

            aggregator.observe(&eav);

            let projected = project_what(&query.what, &eav);
            let history = if want_history {
                epoch_snapshots(&refs, &query.what, &epoch_filter)
            } else {
                Vec::new()
            };
            Some(MatchedEntity {
                entity: *entity,
                projected,
                history,
            })
        })
        .take(early_limit)
        .collect();

    let (measured, grouped) = aggregator.finish();

    // HOW — primary sort
    if let Some(how) = &query.how {
        sort_by_attr(&mut matched, &how.attr, how.dir == SortDir::Desc);
    }

    // HOW_MUCH — cardinality
    match &query.how_much {
        Some(HowMuchClause::Limit(n)) => {
            matched.truncate(*n);
        }
        Some(HowMuchClause::Top { n, attr, dir, .. }) => {
            sort_by_attr(&mut matched, attr, *dir == SortDir::Desc);
            matched.truncate(*n);
        }
        None => {}
    }

    let state_fingerprint = if query.verb == QueryVerb::Sync {
        Some(sync_fingerprint(&matched))
    } else {
        None
    };
    let witness_digest = if query.verb == QueryVerb::Witness {
        Some(witness_digest(&matched))
    } else {
        None
    };

    let plan = QueryPlan::from_query(query);
    QueryResult {
        matched,
        evaluated,
        plan,
        verb: query.verb,
        state_fingerprint,
        witness_digest,
        measured,
        grouped,
        aborted,
    }
}

/// `QueryVerb::Prove`'s per-epoch projection: the same WHAT projection
/// `project_what` computes once for the current last-write-wins state,
/// recomputed after each real journal entry in this entity's history
/// (oldest first, honoring the same `WHEN`-derived `epoch_filter` a plain
/// ORBIT read would). Shares `build_eav_map`'s exact triple-decoding and
/// last-write-wins accumulation via `apply_entry_to_map`, so a `Prove`
/// snapshot at the final epoch always agrees with what ORBIT's `projected`
/// reports for the same entity — one accumulation rule, applied
/// incrementally instead of only once.
fn epoch_snapshots(
    refs: &[&JournalEntry],
    what: &Option<WhatClause>,
    filter: &Option<EpochFilter>,
) -> Vec<EpochSnapshot> {
    let mut sorted: Vec<&JournalEntry> = refs.to_vec();
    sorted.sort_by_key(|e| e.epoch);

    let mut map: EavMap = Vec::new();
    let mut out = Vec::with_capacity(sorted.len());
    for entry in &sorted {
        if let Some(f) = filter {
            if !entry_passes(entry, f) {
                continue;
            }
        }
        apply_entry_to_map(&mut map, entry);
        out.push(EpochSnapshot {
            epoch: entry.epoch,
            attrs: project_what(what, &map),
        });
    }
    out
}

/// `QueryVerb::Sync`'s state fingerprint: XOR-folded CRC32 over every
/// matched entity's KAKI bytes. XOR makes the fold order-independent — two
/// scans that matched the identical *set* of particles agree on this value
/// regardless of what order they were evaluated in, which is exactly the
/// property "compare our orbit states" needs (a byte-for-byte content diff
/// is `QueryVerb::Witness`'s job, not this one).
fn sync_fingerprint(matched: &[MatchedEntity]) -> String {
    let mut acc: u32 = 0;
    for m in matched {
        acc ^= bahyway_crc::crc32(m.entity.bytes());
    }
    format!("{acc:08x}")
}

/// `QueryVerb::Witness`'s content digest: SHA3-256 over every matched
/// entity's KAKI *and* its full projected content, domain-separated with
/// `b"HEPT_WITNESS"` (the same domain-separation convention
/// `enkidb-replication::event` uses for its own frame digests). Sorted by
/// KAKI first so the digest is independent of scan order, same as
/// `sync_fingerprint`. This is a single-node content attestation — a
/// verifiable checksum of exactly what came back — not a cryptographic
/// signature or multi-party consensus; the `{v:?}` value encoding below is
/// an internal canonicalization for hashing purposes only, not a public
/// serialization format any caller should depend on byte-for-byte.
fn witness_digest(matched: &[MatchedEntity]) -> String {
    use sha3::{Digest, Sha3_256};
    let mut rows: Vec<&MatchedEntity> = matched.iter().collect();
    rows.sort_by_key(|m| *m.entity.bytes());

    let mut hasher = Sha3_256::new();
    hasher.update(b"HEPT_WITNESS");
    for m in rows {
        hasher.update(m.entity.bytes());
        for (k, v) in &m.projected {
            hasher.update(k.as_bytes());
            hasher.update(format!("{v:?}").as_bytes());
        }
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ── Epoch filter ──────────────────────────────────────────────────────────────

#[derive(Clone)]
enum EpochFilter {
    AtOrBefore(u64),
    Before(u64),
    After(u64),
}

fn epoch_filter_from_when(when: Option<&WhenClause>) -> Option<EpochFilter> {
    match when? {
        WhenClause::AtEpoch(EpochRef::Now) => None,
        WhenClause::AtEpoch(EpochRef::Abs(n)) => Some(EpochFilter::AtOrBefore(*n)),
        WhenClause::BeforeEpoch(n) => Some(EpochFilter::Before(*n)),
        WhenClause::AfterEpoch(n) => Some(EpochFilter::After(*n)),
    }
}

fn entry_passes(entry: &JournalEntry, filter: &EpochFilter) -> bool {
    let ep = entry.epoch as u64;
    match filter {
        EpochFilter::AtOrBefore(n) => ep <= *n,
        EpochFilter::Before(n) => ep < *n,
        EpochFilter::After(n) => ep > *n,
    }
}

// ── EAV map builder ───────────────────────────────────────────────────────────

/// Flattened EAV: last write wins per attribute hash.
type EavMap = Vec<(u32, AkkValue)>;

fn build_eav_map(history: &[&JournalEntry], filter: &Option<EpochFilter>) -> EavMap {
    let mut sorted: Vec<&JournalEntry> = history.to_vec();
    sorted.sort_by_key(|e| e.epoch);

    let mut map: EavMap = Vec::new();
    for entry in &sorted {
        if let Some(f) = filter {
            if !entry_passes(entry, f) {
                continue;
            }
        }
        apply_entry_to_map(&mut map, entry);
    }
    map
}

/// Fold one journal entry's EAV triples into `map`, last-write-wins per
/// attribute hash. The single accumulation step both `build_eav_map`
/// (one shot, final state) and `epoch_snapshots` (`QueryVerb::Prove`,
/// called once per entry to record the running state as it goes) share —
/// extracted so the two can never disagree on what "last write wins"
/// means at a given point in an entity's history.
fn apply_entry_to_map(map: &mut EavMap, entry: &JournalEntry) {
    for triple in &entry.eav {
        let val = if triple.value.is_empty() {
            AkkValue::Null
        } else {
            codec_decode(&triple.value, 0)
                .map(|(v, _)| v)
                .unwrap_or(AkkValue::Null)
        };
        if let Some(slot) = map.iter_mut().find(|(h, _)| *h == triple.attr_hash) {
            slot.1 = val;
        } else {
            map.push((triple.attr_hash, val));
        }
    }
}

fn attr_hash(name: &str) -> u32 {
    crc16(name.as_bytes()) as u32
}

/// Public-within-crate for `indexed.rs`, which must hash WHERE-clause
/// attribute names the same way `EavTriple::attr_hash` values are produced
/// here, so an index lookup and a full-scan `eval_where` agree on identity.
pub(crate) fn attr_hash_of(name: &str) -> u32 {
    attr_hash(name)
}

/// Last-write-wins EAV snapshot keeping **raw encoded bytes**, not decoded
/// `AkkValue`s — used to build [`crate::indexed::HeptaIndexes`]'s exact-match
/// index, which fingerprints raw bytes. Query-time values must be encoded
/// with the same `akkvalue::codec::encode` before fingerprinting so a stored
/// value and an equal query literal produce the same fingerprint. Mirrors
/// `build_eav_map`'s sort-by-epoch/last-write-wins rule exactly, but skips
/// the decode step since the index never needs the decoded form.
pub(crate) fn raw_eav_snapshot(history: &[&JournalEntry]) -> Vec<(u32, Vec<u8>)> {
    let mut sorted: Vec<&JournalEntry> = history.to_vec();
    sorted.sort_by_key(|e| e.epoch);

    let mut map: Vec<(u32, Vec<u8>)> = Vec::new();
    for entry in &sorted {
        for triple in &entry.eav {
            if let Some(slot) = map.iter_mut().find(|(h, _)| *h == triple.attr_hash) {
                slot.1 = triple.value.clone();
            } else {
                map.push((triple.attr_hash, triple.value.clone()));
            }
        }
    }
    map
}

fn get_val<'m>(map: &'m EavMap, attr: &str) -> Option<&'m AkkValue> {
    let h = attr_hash(attr);
    map.iter().find(|(k, _)| *k == h).map(|(_, v)| v)
}

// ── WHERE evaluation ──────────────────────────────────────────────────────────

fn eval_where(conditions: &[WhereCondition], map: &EavMap) -> bool {
    if conditions.is_empty() {
        return true;
    }
    let mut ok = eval_where_cond(&conditions[0], map);
    for c in &conditions[1..] {
        let this = eval_where_cond(c, map);
        ok = match c.combinator {
            Combinator::And => ok && this,
            Combinator::Or => ok || this,
        };
    }
    ok
}

fn eval_where_cond(c: &WhereCondition, map: &EavMap) -> bool {
    match &c.test {
        ConditionTest::Exists => get_val(map, &c.attr).is_some(),
        ConditionTest::NotExists => get_val(map, &c.attr).is_none(),
        ConditionTest::Cmp { op, val } => match get_val(map, &c.attr) {
            None => false,
            Some(stored) => compare_val(stored, op, val),
        },
    }
}

fn compare_val(stored: &AkkValue, op: &Op, expected: &HeptaValue) -> bool {
    match (stored, expected) {
        (AkkValue::Text(a), HeptaValue::Text(b)) => apply_op_ord(op, a.as_str(), b.as_str()),
        (AkkValue::Int(a), HeptaValue::Int(b)) => apply_op_i64(op, *a, *b),
        (AkkValue::Float(a), HeptaValue::Float(b)) => apply_op_f64(op, *a, *b),
        (AkkValue::Float(a), HeptaValue::Int(b)) => apply_op_f64(op, *a, *b as f64),
        (AkkValue::Int(a), HeptaValue::Float(b)) => apply_op_f64(op, *a as f64, *b),
        (AkkValue::Bool(a), HeptaValue::Bool(b)) => match op {
            Op::Eq => a == b,
            Op::Neq => a != b,
            _ => false,
        },
        (AkkValue::Null, HeptaValue::Null) => matches!(op, Op::Eq),
        _ => false,
    }
}

fn apply_op_ord<T: Ord>(op: &Op, a: T, b: T) -> bool {
    match op {
        Op::Eq => a == b,
        Op::Neq => a != b,
        Op::Gt => a > b,
        Op::Lt => a < b,
        Op::Gte => a >= b,
        Op::Lte => a <= b,
    }
}

fn apply_op_i64(op: &Op, a: i64, b: i64) -> bool {
    apply_op_ord(op, a, b)
}

fn apply_op_f64(op: &Op, a: f64, b: f64) -> bool {
    match op {
        Op::Eq => (a - b).abs() < f64::EPSILON,
        Op::Neq => (a - b).abs() >= f64::EPSILON,
        Op::Gt => a > b,
        Op::Lt => a < b,
        Op::Gte => a >= b,
        Op::Lte => a <= b,
    }
}

// ── WHY evaluation ────────────────────────────────────────────────────────────

fn eval_why(conditions: &[WhyCondition], map: &EavMap) -> bool {
    if conditions.is_empty() {
        return true;
    }
    let mut ok = eval_why_cond(&conditions[0], map);
    for c in &conditions[1..] {
        let this = eval_why_cond(c, map);
        let comb = why_combinator(c);
        ok = match comb {
            Combinator::And => ok && this,
            Combinator::Or => ok || this,
        };
    }
    ok
}

fn why_combinator(c: &WhyCondition) -> &Combinator {
    match c {
        WhyCondition::Lane { combinator, .. } => combinator,
        WhyCondition::QualityByte { combinator, .. } => combinator,
        WhyCondition::AttrExists { combinator, .. } => combinator,
        WhyCondition::AttrNotExists { combinator, .. } => combinator,
    }
}

fn eval_why_cond(c: &WhyCondition, map: &EavMap) -> bool {
    match c {
        WhyCondition::Lane { op, val, .. } => match get_val(map, "lane") {
            Some(AkkValue::Text(s)) => apply_op_ord(op, s.as_str(), lane_str(val)),
            _ => false,
        },
        WhyCondition::QualityByte { op, val, .. } => match get_val(map, "quality.byte") {
            Some(AkkValue::Int(n)) => apply_op_i64(op, *n, *val as i64),
            _ => false,
        },
        WhyCondition::AttrExists { attr, .. } => get_val(map, attr).is_some(),
        WhyCondition::AttrNotExists { attr, .. } => get_val(map, attr).is_none(),
    }
}

fn lane_str(lane: &LaneValue) -> &'static str {
    match lane {
        LaneValue::Gold => "Gold",
        LaneValue::Silver => "Silver",
        LaneValue::White => "White",
        LaneValue::Gray => "Gray",
        LaneValue::Red => "Red",
        LaneValue::Black => "Black",
    }
}

// ── Projection ────────────────────────────────────────────────────────────────

fn project_what(what: &Option<WhatClause>, map: &EavMap) -> Vec<(String, AkkValue)> {
    let what = match what {
        Some(w) => w,
        None => return vec![],
    };
    let mut out = Vec::new();
    for proj in &what.projections {
        if proj.attrs.is_empty() {
            // Star (*) — all attributes; we only have hashes, no reverse map yet.
            // Return all decoded pairs using numeric hash keys as placeholder names.
            for (h, v) in map {
                out.push((format!("#{h:04x}"), v.clone()));
            }
        } else {
            for attr in &proj.attrs {
                if let Some(v) = get_val(map, attr) {
                    out.push((attr.clone(), v.clone()));
                }
            }
        }
    }
    out
}

// ── Sorting ───────────────────────────────────────────────────────────────────

fn sort_by_attr(entities: &mut [MatchedEntity], attr: &str, descending: bool) {
    entities.sort_by(|a, b| {
        let va = a.projected.iter().find(|(n, _)| n == attr).map(|(_, v)| v);
        let vb = b.projected.iter().find(|(n, _)| n == attr).map(|(_, v)| v);
        let ord = cmp_akk(va, vb);
        if descending {
            ord.reverse()
        } else {
            ord
        }
    });
}

fn cmp_akk(a: Option<&AkkValue>, b: Option<&AkkValue>) -> core::cmp::Ordering {
    match (a, b) {
        (None, None) => core::cmp::Ordering::Equal,
        (None, _) => core::cmp::Ordering::Greater,
        (_, None) => core::cmp::Ordering::Less,
        (Some(x), Some(y)) => match (x, y) {
            (AkkValue::Int(i), AkkValue::Int(j)) => i.cmp(j),
            (AkkValue::Float(i), AkkValue::Float(j)) => {
                i.partial_cmp(j).unwrap_or(core::cmp::Ordering::Equal)
            }
            (AkkValue::Text(i), AkkValue::Text(j)) => i.cmp(j),
            (AkkValue::Bool(i), AkkValue::Bool(j)) => i.cmp(j),
            _ => core::cmp::Ordering::Equal,
        },
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_query;
    use akkvalue::codec::encode as codec_encode;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::EavTriple;
    use enkidb_journal::Journal;
    use enkidb_kaki::{mint::KakiMinter, EventKaki, KakiRole};

    fn make_journal(tribe: TribeId) -> (Journal, KakiMinter) {
        (Journal::new(64), KakiMinter::new(tribe))
    }

    fn push_eav(
        jnl: &mut Journal,
        m: &KakiMinter,
        target: &IdentityKaki,
        epoch: u32,
        attrs: &[(&str, AkkValue)],
    ) {
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav: Vec<EavTriple> = attrs
            .iter()
            .map(|(name, val)| EavTriple::new(attr_hash(name), codec_encode(val)))
            .collect();
        use enkidb_journal::entry::JournalEntry;
        jnl.append(JournalEntry::new(ek, target.clone(), epoch, eav))
            .unwrap();
    }

    fn new_entity(m: &KakiMinter) -> IdentityKaki {
        IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap()
    }

    // ── Basic WHERE filter ────────────────────────────────────────────────────

    #[test]
    fn where_string_eq_filters_correctly() {
        let tribe = TribeId::from_u16(0x0001);
        let (mut jnl, m) = make_journal(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[("city.name", AkkValue::Text("Baghdad".into()))],
        );

        let q = parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.evaluated, 2);
        assert_eq!(res.matched.len(), 1);
        assert_eq!(*res.matched[0].entity.bytes(), *e1.bytes());
    }

    #[test]
    fn where_int_lt_filters_correctly() {
        let tribe = TribeId::from_u16(0x0001);
        let (mut jnl, m) = make_journal(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(&mut jnl, &m, &e1, 1, &[("birth.year", AkkValue::Int(1975))]);
        push_eav(&mut jnl, &m, &e2, 1, &[("birth.year", AkkValue::Int(1995))]);

        let q = parse_query("WHO T.E\nWHERE E[birth.year] < 1990").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
    }

    #[test]
    fn where_float_gte() {
        let tribe = TribeId::from_u16(0x0001);
        let (mut jnl, m) = make_journal(tribe);

        let e = new_entity(&m);
        push_eav(&mut jnl, &m, &e, 1, &[("score", AkkValue::Float(9.5))]);

        let q = parse_query("WHO T.E\nWHERE E[score] >= 9.0").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
    }

    #[test]
    fn where_exists() {
        let tribe = TribeId::from_u16(0x0001);
        let (mut jnl, m) = make_journal(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("leak.reported", AkkValue::Bool(true))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[("status", AkkValue::Text("ok".into()))],
        );

        let q = parse_query("WHO T.E\nWHERE E[leak.reported] EXISTS").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
    }

    #[test]
    fn where_not_exists() {
        let tribe = TribeId::from_u16(0x0001);
        let (mut jnl, m) = make_journal(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("archive.date", AkkValue::Int(1000))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[("status", AkkValue::Text("active".into()))],
        );

        let q = parse_query("WHO T.E\nWHERE E[archive.date] NOT EXISTS").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
        assert_eq!(*res.matched[0].entity.bytes(), *e2.bytes());
    }

    #[test]
    fn where_and_both_conditions_must_match() {
        let tribe = TribeId::from_u16(0x0001);
        let (mut jnl, m) = make_journal(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[
                ("city.name", AkkValue::Text("Najaf".into())),
                ("birth.year", AkkValue::Int(1970)),
            ],
        );
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[
                ("city.name", AkkValue::Text("Najaf".into())),
                ("birth.year", AkkValue::Int(2000)),
            ],
        );

        let q = parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\" AND E[birth.year] < 1990")
            .unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
        assert_eq!(*res.matched[0].entity.bytes(), *e1.bytes());
    }

    // ── WHEN clause ───────────────────────────────────────────────────────────

    #[test]
    fn when_at_epoch_excludes_later_entries() {
        let tribe = TribeId::from_u16(0x0002);
        let (mut jnl, m) = make_journal(tribe);
        let e = new_entity(&m);

        // Write at epoch 5 (active) then override at epoch 10 (archived)
        push_eav(
            &mut jnl,
            &m,
            &e,
            5,
            &[("status", AkkValue::Text("active".into()))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e,
            10,
            &[("status", AkkValue::Text("archived".into()))],
        );

        // At epoch 7: should see "active" (epoch 5 entry only)
        let q = parse_query("WHO T.E\nWHERE E[status] = \"active\"\nWHEN AT EPOCH 7").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);

        // At epoch NOW: should see "archived"
        let q2 = parse_query("WHO T.E\nWHERE E[status] = \"archived\"\nWHEN AT EPOCH NOW").unwrap();
        let res2 = execute(&q2, &jnl);
        assert_eq!(res2.matched.len(), 1);
    }

    // ── WHY clause ────────────────────────────────────────────────────────────

    #[test]
    fn why_quality_byte_filter() {
        let tribe = TribeId::from_u16(0x0003);
        let (mut jnl, m) = make_journal(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("quality.byte", AkkValue::Int(200))],
        );
        push_eav(&mut jnl, &m, &e2, 1, &[("quality.byte", AkkValue::Int(50))]);

        let q = parse_query("WHO T.E\nWHY QUALITY_BYTE > 150").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
    }

    #[test]
    fn why_lane_neq_filter() {
        let tribe = TribeId::from_u16(0x0004);
        let (mut jnl, m) = make_journal(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("lane", AkkValue::Text("Gold".into()))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[("lane", AkkValue::Text("Black".into()))],
        );

        let q = parse_query("WHO T.E\nWHY LANE != Black").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
    }

    // ── WHAT clause ───────────────────────────────────────────────────────────

    #[test]
    fn what_projects_named_attrs() {
        let tribe = TribeId::from_u16(0x0005);
        let (mut jnl, m) = make_journal(tribe);
        let e = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e,
            1,
            &[
                ("name.given", AkkValue::Text("Bahaa".into())),
                ("city.name", AkkValue::Text("Najaf".into())),
                ("score", AkkValue::Float(9.5)),
            ],
        );

        let q = parse_query("WHO T.E\nWHAT E[name.given, city.name]").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
        let proj = &res.matched[0].projected;
        assert_eq!(proj.len(), 2);
        assert!(proj
            .iter()
            .any(|(k, v)| k == "name.given" && *v == AkkValue::Text("Bahaa".into())));
        assert!(proj
            .iter()
            .any(|(k, v)| k == "city.name" && *v == AkkValue::Text("Najaf".into())));
    }

    // ── HOW + HOW_MUCH ────────────────────────────────────────────────────────

    #[test]
    fn how_much_limit_truncates() {
        let tribe = TribeId::from_u16(0x0006);
        let (mut jnl, m) = make_journal(tribe);
        for year in [1980, 1985, 1990, 1995] {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("birth.year", AkkValue::Int(year))]);
        }
        let q = parse_query("WHO T.E\nHOW_MUCH LIMIT 2").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.evaluated, 4);
        assert_eq!(res.matched.len(), 2);
    }

    #[test]
    fn empty_journal_returns_zero_matches() {
        let tribe = TribeId::from_u16(0x0007);
        let (jnl, _) = make_journal(tribe);
        let q = parse_query("WHO T.E\nWHERE E[x] = 1").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.evaluated, 0);
        assert!(res.matched.is_empty());
    }

    #[test]
    fn no_conditions_matches_all() {
        let tribe = TribeId::from_u16(0x0008);
        let (mut jnl, m) = make_journal(tribe);
        for _ in 0..3 {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);
        }
        let q = parse_query("WHO T.E").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 3);
    }

    // ── ABORT_SCAN safety valve ──────────────────────────────────────────────
    // Regression coverage for the real gap found while wiring E-005
    // (TESTING_PLAYBOOK_PHASE1.md): `execute_over_histories` is the one real
    // evaluation implementation every caller (execute/execute_over/
    // indexed::execute_indexed) goes through, but it ignored
    // `query.abort_scan` entirely -- the clause parsed successfully and did
    // nothing. `StreamStats.aborted` existed on the separate, never-served
    // `execute_stream` path only.

    #[test]
    fn abort_scan_caps_evaluated_and_reports_aborted_when_below_candidate_count() {
        let tribe = TribeId::from_u16(0x0009);
        let (mut jnl, m) = make_journal(tribe);
        for _ in 0..10 {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);
        }
        let q = parse_query("WHO T.E\nABORT_SCAN 3").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(
            res.evaluated, 3,
            "must stop looking at candidates once the cap is hit"
        );
        assert!(
            res.aborted,
            "cap (3) is below the real candidate count (10) -- must report aborted"
        );
        assert!(
            res.matched.len() <= 3,
            "cannot match more than it evaluated"
        );
    }

    #[test]
    fn abort_scan_does_not_fire_when_limit_is_at_or_above_candidate_count() {
        let tribe = TribeId::from_u16(0x000a);
        let (mut jnl, m) = make_journal(tribe);
        for _ in 0..5 {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);
        }
        let q = parse_query("WHO T.E\nABORT_SCAN 5000000").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(
            res.evaluated, 5,
            "a cap well above the real candidate count must not truncate the scan"
        );
        assert!(!res.aborted);
        assert_eq!(res.matched.len(), 5);
    }

    #[test]
    fn no_abort_scan_clause_never_reports_aborted() {
        let tribe = TribeId::from_u16(0x000b);
        let (mut jnl, m) = make_journal(tribe);
        for _ in 0..4 {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);
        }
        let q = parse_query("WHO T.E").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.evaluated, 4);
        assert!(!res.aborted);
    }

    // ── Sovereign Operation verbs ────────────────────────────────────────────

    #[test]
    fn orbit_is_the_default_verb_and_an_explicit_keyword_is_identical() {
        let tribe = TribeId::from_u16(0x0100);
        let (mut jnl, m) = make_journal(tribe);
        let e = new_entity(&m);
        push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);

        let implicit = parse_query("WHO T.E\nWHAT E[x]").unwrap();
        let explicit = parse_query("ORBIT\nWHO T.E\nWHAT E[x]").unwrap();
        assert_eq!(implicit.verb, QueryVerb::Orbit);
        assert_eq!(explicit.verb, QueryVerb::Orbit);

        let r1 = execute(&implicit, &jnl);
        let r2 = execute(&explicit, &jnl);
        assert_eq!(r1.matched.len(), 1);
        assert_eq!(r2.matched.len(), 1);
        assert_eq!(r1.matched[0].projected, r2.matched[0].projected);
        assert!(r1.state_fingerprint.is_none());
        assert!(r1.witness_digest.is_none());
        assert!(r1.matched[0].history.is_empty());
    }

    #[test]
    fn emit_only_returns_birth_event_particles() {
        let tribe = TribeId::from_u16(0x0101);
        let (mut jnl, m) = make_journal(tribe);

        let born = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &born,
            1,
            &[
                ("meta.title", AkkValue::Text("Born One".into())),
                ("hist.event", AkkValue::Text("BIRTH".into())),
            ],
        );
        let updated_only = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &updated_only,
            1,
            &[("meta.title", AkkValue::Text("Never Birthed".into()))],
        );

        let q = parse_query("EMIT\nWHO T.E\nWHAT E[meta.title]").unwrap();
        assert_eq!(q.verb, QueryVerb::Emit);
        let res = execute(&q, &jnl);
        assert_eq!(
            res.matched.len(),
            1,
            "only the BIRTH-tagged particle should match"
        );
        assert_eq!(*res.matched[0].entity.bytes(), *born.bytes());
    }

    #[test]
    fn emit_combines_with_an_explicit_where_condition() {
        let tribe = TribeId::from_u16(0x0102);
        let (mut jnl, m) = make_journal(tribe);

        let matches_both = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &matches_both,
            1,
            &[
                ("meta.collection", AkkValue::Text("component".into())),
                ("hist.event", AkkValue::Text("BIRTH".into())),
            ],
        );
        let wrong_collection = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &wrong_collection,
            1,
            &[
                ("meta.collection", AkkValue::Text("roadmap".into())),
                ("hist.event", AkkValue::Text("BIRTH".into())),
            ],
        );
        let not_born = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &not_born,
            1,
            &[("meta.collection", AkkValue::Text("component".into()))],
        );

        let q = parse_query(
            "EMIT\nWHO T.E\nWHAT E[meta.collection]\nWHERE E[meta.collection] = \"component\"",
        )
        .unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);
        assert_eq!(*res.matched[0].entity.bytes(), *matches_both.bytes());
    }

    #[test]
    fn prove_reports_full_per_epoch_history_alongside_current_state() {
        let tribe = TribeId::from_u16(0x0103);
        let (mut jnl, m) = make_journal(tribe);
        let e = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e,
            1,
            &[("status", AkkValue::Text("active".into()))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e,
            5,
            &[("status", AkkValue::Text("archived".into()))],
        );

        let q = parse_query("PROVE\nWHO T.E\nWHAT E[status]\nWHERE E[status] EXISTS").unwrap();
        assert_eq!(q.verb, QueryVerb::Prove);
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 1);

        let m0 = &res.matched[0];
        // Current-state projection is unaffected — still last-write-wins.
        assert_eq!(
            m0.projected,
            vec![("status".to_string(), AkkValue::Text("archived".into()))]
        );

        // But the full history is now available too, oldest first.
        assert_eq!(m0.history.len(), 2);
        assert_eq!(m0.history[0].epoch, 1);
        assert_eq!(
            m0.history[0].attrs,
            vec![("status".to_string(), AkkValue::Text("active".into()))]
        );
        assert_eq!(m0.history[1].epoch, 5);
        assert_eq!(
            m0.history[1].attrs,
            vec![("status".to_string(), AkkValue::Text("archived".into()))]
        );
    }

    #[test]
    fn prove_history_is_empty_for_every_other_verb() {
        let tribe = TribeId::from_u16(0x0104);
        let (mut jnl, m) = make_journal(tribe);
        let e = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e,
            1,
            &[("status", AkkValue::Text("active".into()))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e,
            5,
            &[("status", AkkValue::Text("archived".into()))],
        );

        let q = parse_query("WHO T.E\nWHAT E[status]\nWHERE E[status] EXISTS").unwrap();
        let res = execute(&q, &jnl);
        assert!(
            res.matched[0].history.is_empty(),
            "ORBIT must not pay PROVE's per-epoch cost"
        );
    }

    #[test]
    fn sync_fingerprint_is_present_and_order_independent() {
        let tribe = TribeId::from_u16(0x0105);
        let (mut jnl, m) = make_journal(tribe);
        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );

        let q = parse_query("SYNC\nWHO T.E\nWHAT E[city.name]\nWHERE E[city.name] = \"Najaf\"")
            .unwrap();
        assert_eq!(q.verb, QueryVerb::Sync);
        let res_a = execute(&q, &jnl);
        let res_b = execute(&q, &jnl);
        assert_eq!(res_a.matched.len(), 2);
        assert!(res_a.state_fingerprint.is_some());
        assert!(
            res_a.witness_digest.is_none(),
            "SYNC must not also compute WITNESS's digest"
        );
        // Two independent scans of the identical matched set agree exactly,
        // regardless of internal HashMap/journal iteration order.
        assert_eq!(res_a.state_fingerprint, res_b.state_fingerprint);
    }

    #[test]
    fn sync_fingerprint_changes_when_the_matched_set_changes() {
        let tribe = TribeId::from_u16(0x0106);
        let (mut jnl, m) = make_journal(tribe);
        let e1 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );

        let q = parse_query("SYNC\nWHO T.E\nWHAT E[city.name]\nWHERE E[city.name] = \"Najaf\"")
            .unwrap();
        let before = execute(&q, &jnl).state_fingerprint;

        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );
        let after = execute(&q, &jnl).state_fingerprint;

        assert_ne!(
            before, after,
            "adding a matching particle must change the fingerprint"
        );
    }

    #[test]
    fn witness_digest_is_present_deterministic_and_content_sensitive() {
        let tribe = TribeId::from_u16(0x0107);
        let (mut jnl, m) = make_journal(tribe);
        let e = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e,
            1,
            &[("meta.title", AkkValue::Text("Alpha".into()))],
        );

        let q = parse_query("WITNESS\nWHO T.E\nWHAT E[meta.title]\nWHERE E[meta.title] EXISTS")
            .unwrap();
        assert_eq!(q.verb, QueryVerb::Witness);
        let res_a = execute(&q, &jnl);
        let res_b = execute(&q, &jnl);
        assert!(
            res_a.state_fingerprint.is_none(),
            "WITNESS must not also compute SYNC's fingerprint"
        );
        let digest_a = res_a
            .witness_digest
            .expect("witness digest must be present");
        assert_eq!(digest_a.len(), 64, "SHA3-256 hex digest is 64 characters");
        assert_eq!(
            Some(digest_a.clone()),
            res_b.witness_digest,
            "identical content must reproduce identically"
        );

        // Change the underlying content without changing which entity
        // matches -- the digest must move even though the matched SET
        // (what SYNC cares about) would not have.
        push_eav(
            &mut jnl,
            &m,
            &e,
            2,
            &[("meta.title", AkkValue::Text("Alpha Revised".into()))],
        );
        let digest_b = execute(&q, &jnl)
            .witness_digest
            .expect("witness digest must be present");
        assert_ne!(
            digest_a, digest_b,
            "changed content must change the digest even for the same entity"
        );
    }

    // ── MEASURE / GRAVITY (v2.1 aggregate clauses) ───────────────────────────

    #[test]
    fn no_measure_or_gravity_clause_leaves_both_fields_none() {
        let tribe = TribeId::from_u16(0x0200);
        let (mut jnl, m) = make_journal(tribe);
        let e = new_entity(&m);
        push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);

        let q = parse_query("WHO T.E").unwrap();
        let res = execute(&q, &jnl);
        assert!(res.measured.is_none());
        assert!(res.grouped.is_none());
    }

    #[test]
    fn measure_dense_counts_correctly_past_the_algebra_dimension() {
        // 10 particles -- deliberately more than Cl(7)'s 7 dimensions, the
        // point at which a wedge-product-magnitude "count" would collapse
        // to zero. A real tally must not care.
        let tribe = TribeId::from_u16(0x0201);
        let (mut jnl, m) = make_journal(tribe);
        for i in 0..10 {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("n", AkkValue::Int(i))]);
        }

        let q = parse_query("WHO T.E\nMEASURE DENSE").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.measured, Some(MeasureValue::Dense(10)));
        assert!(res.grouped.is_none());
    }

    #[test]
    fn measure_dense_reflects_full_matched_set_even_under_how_much_limit() {
        // HOW_MUCH LIMIT bounds the *returned rows*, never the aggregate --
        // otherwise DENSE would just report back whatever LIMIT was asked
        // for, which defeats the point of counting.
        let tribe = TribeId::from_u16(0x0202);
        let (mut jnl, m) = make_journal(tribe);
        for i in 0..20 {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("n", AkkValue::Int(i))]);
        }

        let q = parse_query("WHO T.E\nHOW_MUCH LIMIT 3\nMEASURE DENSE").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.matched.len(), 3, "returned rows honor LIMIT");
        assert_eq!(
            res.measured,
            Some(MeasureValue::Dense(20)),
            "the aggregate must not"
        );
    }

    #[test]
    fn measure_flux_sums_the_matched_attribute() {
        let tribe = TribeId::from_u16(0x0203);
        let (mut jnl, m) = make_journal(tribe);
        for reading in [10, 20, 30, 40] {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("pressure", AkkValue::Int(reading))]);
        }

        let q = parse_query("WHO T.E\nMEASURE FLUX E[pressure]").unwrap();
        let res = execute(&q, &jnl);
        assert_eq!(res.measured, Some(MeasureValue::Flux(100.0)));
    }

    #[test]
    fn measure_rotor_mean_is_a_true_circular_mean_not_an_arithmetic_one() {
        // Two orientations just past +-pi -- geometrically almost identical
        // (both point "backwards"), but their arithmetic mean is ~0 (a
        // "forwards" direction, the wrong answer). The circular mean must
        // land back near +-pi, proving this isn't secretly `AVG`.
        let tribe = TribeId::from_u16(0x0204);
        let (mut jnl, m) = make_journal(tribe);
        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e1,
            1,
            &[("orientation", AkkValue::Float(3.14))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e2,
            1,
            &[("orientation", AkkValue::Float(-3.14))],
        );

        let q = parse_query("WHO T.E\nMEASURE ROTOR_MEAN E[orientation]").unwrap();
        let res = execute(&q, &jnl);
        let mean = match res.measured {
            Some(MeasureValue::RotorMean(v)) => v,
            other => panic!("expected RotorMean, got {other:?}"),
        };
        assert!(
            mean.abs() > 3.0,
            "circular mean should stay near +-pi, got {mean}"
        );
    }

    #[test]
    fn gravity_band_partitions_into_numeric_buckets() {
        let tribe = TribeId::from_u16(0x0205);
        let (mut jnl, m) = make_journal(tribe);
        // Band width 10 -> buckets [0,10), [10,20), [20,30)
        for reading in [1, 5, 12, 15, 22] {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("pressure", AkkValue::Int(reading))]);
        }

        let q = parse_query("WHO T.E\nGRAVITY E[pressure] BAND 10.0\nMEASURE DENSE").unwrap();
        let res = execute(&q, &jnl);
        let grouped = res.grouped.expect("gravity result present");
        assert!(!grouped.capped);
        assert_eq!(grouped.groups.len(), 3, "{:?}", grouped.groups);
        assert_eq!(grouped.groups[0].key, GravityKey::Band(0));
        assert_eq!(grouped.groups[0].measure, MeasureValue::Dense(2));
        assert_eq!(grouped.groups[1].key, GravityKey::Band(1));
        assert_eq!(grouped.groups[1].measure, MeasureValue::Dense(2));
        assert_eq!(grouped.groups[2].key, GravityKey::Band(2));
        assert_eq!(grouped.groups[2].measure, MeasureValue::Dense(1));
    }

    #[test]
    fn gravity_max_groups_caps_and_reports_capped_true() {
        let tribe = TribeId::from_u16(0x0206);
        let (mut jnl, m) = make_journal(tribe);
        // 5 distinct stations, cap at 2 -- the 3rd distinct value onward
        // must fold into Overflow, never grow group memory past the cap.
        for station in ["A", "B", "C", "D", "E"] {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("station", AkkValue::Text(station.into()))],
            );
        }

        let q = parse_query("WHO T.E\nGRAVITY E[station] MAX_GROUPS 2\nMEASURE DENSE").unwrap();
        let res = execute(&q, &jnl);
        let grouped = res.grouped.expect("gravity result present");
        assert!(
            grouped.capped,
            "cap must have been hit with 5 distinct values against MAX_GROUPS 2"
        );
        // At most 2 real Exact groups + 1 Overflow sentinel -- bounded
        // regardless of the attribute's true 5-value cardinality.
        assert!(grouped.groups.len() <= 3, "{:?}", grouped.groups);
        assert!(grouped.groups.iter().any(|g| g.key == GravityKey::Overflow));
        let total: usize = grouped.groups.iter().map(|g| g.count).sum();
        assert_eq!(
            total, 5,
            "every matched particle must land in some group, none silently dropped"
        );
    }

    #[test]
    fn gravity_without_max_groups_reached_never_sets_capped() {
        let tribe = TribeId::from_u16(0x0207);
        let (mut jnl, m) = make_journal(tribe);
        for station in ["A", "B"] {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("station", AkkValue::Text(station.into()))],
            );
        }

        let q = parse_query("WHO T.E\nGRAVITY E[station] MAX_GROUPS 10\nMEASURE DENSE").unwrap();
        let res = execute(&q, &jnl);
        let grouped = res.grouped.expect("gravity result present");
        assert!(!grouped.capped);
        assert_eq!(grouped.groups.len(), 2);
    }

    #[test]
    fn gravity_group_missing_attribute_particles_are_not_silently_dropped() {
        let tribe = TribeId::from_u16(0x0208);
        let (mut jnl, m) = make_journal(tribe);
        let has_station = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &has_station,
            1,
            &[("station", AkkValue::Text("A".into()))],
        );
        let no_station = new_entity(&m);
        push_eav(&mut jnl, &m, &no_station, 1, &[("other", AkkValue::Int(1))]);

        let q = parse_query("WHO T.E\nGRAVITY E[station] MAX_GROUPS 10\nMEASURE DENSE").unwrap();
        let res = execute(&q, &jnl);
        let grouped = res.grouped.expect("gravity result present");
        assert!(grouped
            .groups
            .iter()
            .any(|g| g.key == GravityKey::Missing && g.count == 1));
        let total: usize = grouped.groups.iter().map(|g| g.count).sum();
        assert_eq!(total, 2);
    }
}
