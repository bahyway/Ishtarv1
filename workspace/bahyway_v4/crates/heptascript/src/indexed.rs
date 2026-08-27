//! Indexed query execution — wires HeptaScript into the ENLIL index stack
//! (`enkidb-indexes`) instead of scanning the full journal on every query.
//!
//! `engine::execute()` calls `journal.all_particles()` then
//! `journal.read_particle_history()` for **every** candidate, before any
//! WHERE/WHY predicate is applied — an O(n) journal replay per query
//! regardless of how selective the query actually is. Every architecture
//! document describes a six-index ENLIL stack meant to prune that candidate
//! set first; `enkidb-indexes` contains real, tested index structures
//! (`SurrogateMap`, `EavExactIndex`, ...) but nothing in the query path
//! called them. This module is that missing wire.
//!
//! [`build_indexes`] pays the O(n) journal-read cost exactly **once**
//! (analogous to "all 6 ENLIL indexes built at startup" in the sealed
//! architecture references) and produces a [`HeptaIndexes`] snapshot.
//! (This was true in intent but not in fact until it was fixed to group
//! entries in a single pass -- see the function's own doc comment for
//! the real bug that was found seeding 10M particles for the first time.)
//! [`execute_indexed`] then uses it to resolve an exact-equality WHERE
//! clause directly to its matching surrogates — `O(log n)` — before reading
//! any particle's history, and only reads history for the survivors.
//!
//! Correctness is never gambled for speed: whenever a query isn't safely
//! indexable (a non-equality first condition, an `OR` in the WHERE clause,
//! or a WHEN clause pinned to a specific past epoch that the snapshot
//! doesn't represent), this module falls straight back to
//! `engine::execute()` — the exact same function, same result, just without
//! the speed-up. `execute_indexed` and `execute` can never disagree.

use std::collections::HashMap;

use akkvalue::{codec, AkkValue};
use enkidb_indexes::prelude::{EavExactIndex, HeptaShellIndex, NatiruIndex, SurrogateMap};
use enkidb_indexes::radix_spline::{RadixSplineIndex, DEFAULT_EPSILON as RADIX_DEFAULT_EPSILON};
use enkidb_journal::entry::JournalEntry;
use enkidb_journal::Journal;
use enkidb_kaki::IdentityKaki;

use crate::engine::{self, QueryResult};
use crate::query::{
    AnchorStrategy, Combinator, ConditionTest, EpochRef, HeptaQuery, HeptaValue, Op, WhenClause,
    WhereCondition,
};

/// The seven EAV attribute names `hepta_shell::HeptaShellIndex` treats as a
/// particle's 7D orbital position (r, θ, φ, d3..d6 — same dimension order
/// as `hepta_shell`'s own `SCALE` constant). A particle only enters the
/// shell index if it carries all seven; a query only uses the shell index
/// if its WHERE clause pins all seven with exact equality (see
/// `spatial_center_from_where`) -- both ends of a real, if narrow, contract
/// rather than a guess at what "position" means for an arbitrary particle.
pub const SPATIAL_ATTRS: [&str; 7] = [
    "orbit.r",
    "orbit.theta",
    "orbit.phi",
    "orbit.d3",
    "orbit.d4",
    "orbit.d5",
    "orbit.d6",
];

/// Pre-built index snapshot over a `Journal`'s particle set at the moment
/// [`build_indexes`] ran. Immutable — rebuild to pick up particles written
/// since, exactly like the ENLIL stack's documented "rebuilt on each
/// Read-node snapshot load" lifecycle. Cheap to hold behind an `Arc` and
/// share across query-server connection threads.
pub struct HeptaIndexes {
    /// KAKI bytes → dense surrogate id, so the EAV index can work in u32s
    /// instead of 16-byte keys.
    #[allow(dead_code)]
    surrogates: SurrogateMap,
    /// surrogate id → the `IdentityKaki` it was assigned during `build`,
    /// so an index hit can be turned back into something
    /// `journal.read_particle_history()` accepts.
    by_surrogate: Vec<IdentityKaki>,
    /// (attr_hash, value_fingerprint) → surrogates, built from each
    /// particle's last-write-wins raw EAV bytes.
    eav: EavExactIndex,
    /// (orbital epoch, surrogate) → surrogates with at least one journal
    /// entry in an epoch range. Drives WHEN-clause pruning (`WhenClause::
    /// BeforeEpoch`/`AfterEpoch`/`AtEpoch(Abs)`) -- unlike `eav`, this is
    /// safe to use regardless of WHEN pinning a past epoch, because it
    /// answers "did this surrogate write anything in this range," not
    /// "what is its current value."
    nairu: NatiruIndex,
    /// Learned (epoch, surrogate) range index. Drives ORBITAL-clause
    /// pruning specifically -- a distinct query clause from WHEN, so a
    /// distinct index, even though both happen to key on epoch today.
    radix: RadixSplineIndex,
    /// 7D E7-lattice zone index over particles carrying all of
    /// `SPATIAL_ATTRS`. Drives `ANCHOR E7_FIRST` WHERE-clause pruning when
    /// the WHERE clause pins all seven spatial attributes with exact
    /// equality (see `spatial_center_from_where`).
    shell: HeptaShellIndex,
}

/// Build a [`HeptaIndexes`] snapshot from one full pass over `journal`.
///
/// This is the O(n) cost every architecture document assumed already
/// existed — paid once here, not on every query. Call it at server startup
/// (and again after any batch of writes you want reflected in the index).
pub fn build_indexes(journal: &Journal) -> HeptaIndexes {
    let particles = journal.all_particles();

    let kaki_bytes: Vec<[u8; 16]> = particles.iter().map(|p| *p.bytes()).collect();
    let surrogates = SurrogateMap::build(&kaki_bytes);

    // Group every journal entry by its target particle in a single O(n)
    // pass, rather than calling `journal.read_particle_history()` once
    // per particle. `read_particle_history` rescans its whole shard
    // partition on every call (it's tuned for an occasional live lookup,
    // per its own doc comment) -- calling it once per particle here meant
    // this function was O(partition_size) per particle, i.e.
    // O(partition_size^2) per shard overall. That stayed invisible at the
    // sub-1M-particle scale this was tested at, but at real production
    // scale (10M particles / 64 shards, ~156K particles per shard-epoch
    // partition) it turned "build the index once at startup" into a
    // multi-minute-to-multi-hour stall -- confirmed live seeding
    // enkidb-query-server's first genuine 10M-particle startup.
    let mut by_target: HashMap<[u8; 16], Vec<&JournalEntry>> =
        HashMap::with_capacity(particles.len());
    for entry in journal.all_entries() {
        by_target
            .entry(*entry.target_kaki.bytes())
            .or_default()
            .push(entry);
    }

    let spatial_hashes: [u32; 7] = SPATIAL_ATTRS.map(engine::attr_hash_of);

    let mut triples: Vec<(u32, Vec<u8>, u32)> = Vec::new();
    // One (epoch, surrogate) pair per journal entry -- NOT deduplicated to
    // last-write-wins like `triples` above, since `nairu`/`radix` answer
    // "did this surrogate write anything in this epoch," which needs every
    // entry's epoch, not just the current EAV state.
    let mut orbital_pairs: Vec<(u64, u32)> = Vec::new();
    let mut shell_entries: Vec<([f64; 7], u32)> = Vec::new();

    for (surrogate, particle) in particles.iter().enumerate() {
        let sur = surrogate as u32;
        let Some(history) = by_target.get(particle.bytes()) else {
            continue;
        };

        for entry in history.iter() {
            orbital_pairs.push((entry.epoch as u64, sur));
        }

        let raw = engine::raw_eav_snapshot(history);

        let mut position = [0f64; 7];
        let mut has_all_dims = true;
        for (dim, hash) in spatial_hashes.iter().enumerate() {
            match raw
                .iter()
                .find(|(h, _)| h == hash)
                .and_then(|(_, b)| decode_f64(b))
            {
                Some(v) => position[dim] = v,
                None => {
                    has_all_dims = false;
                }
            }
        }
        if has_all_dims {
            shell_entries.push((position, sur));
        }

        for (attr_hash, value_bytes) in raw {
            triples.push((attr_hash, value_bytes, sur));
        }
    }

    let eav = EavExactIndex::build(&triples);

    let nairu = NatiruIndex::build(0, &orbital_pairs);

    orbital_pairs.sort_unstable_by_key(|&(epoch, _)| epoch);
    let radix_keys: Vec<u64> = orbital_pairs.iter().map(|&(epoch, _)| epoch).collect();
    let radix_values: Vec<u32> = orbital_pairs.iter().map(|&(_, sur)| sur).collect();
    let radix = RadixSplineIndex::build(radix_keys, radix_values, RADIX_DEFAULT_EPSILON);

    let shell = HeptaShellIndex::build(&shell_entries);

    HeptaIndexes {
        surrogates,
        by_surrogate: particles,
        eav,
        nairu,
        radix,
        shell,
    }
}

/// Decode a raw EAV value blob to `f64` for `HeptaShellIndex`'s numeric
/// position dimensions. `Int` values are accepted too (widened losslessly
/// for any realistic coordinate magnitude), matching `engine::as_f64`'s
/// same Int-or-Float acceptance for MEASURE FLUX/ROTOR_MEAN.
fn decode_f64(bytes: &[u8]) -> Option<f64> {
    let (v, _) = codec::decode(bytes, 0).ok()?;
    match v {
        AkkValue::Float(f) => Some(f),
        AkkValue::Int(i) => Some(i as f64),
        _ => None,
    }
}

/// Execute `query` against `journal`, using `idx` to prune the candidate
/// set before any per-particle history read whenever it's safe to do so.
/// Falls back to [`engine::execute`] (full scan) otherwise. Results are
/// identical either way — only the number of `read_particle_history` calls
/// differs.
pub fn execute_indexed(query: &HeptaQuery, journal: &Journal, idx: &HeptaIndexes) -> QueryResult {
    if let Some(surrogates) = candidates_from_indexes(query, idx) {
        let pruned: Vec<IdentityKaki> = surrogates
            .into_iter()
            .filter_map(|s| idx.by_surrogate.get(s as usize).cloned())
            .collect();
        return engine::execute_over(query, journal, &pruned);
    }
    engine::execute(query, journal)
}

/// Resolve every index this query's clauses can safely use into one
/// AND-intersected surrogate candidate set, or `None` if nothing applies
/// (caller must fall back to a full scan). Each contributing strategy below
/// narrows the *candidate set only* -- `execute_over` still evaluates the
/// query's real WHERE/WHY/WHEN against each candidate's full history
/// afterward, so an over-inclusive candidate set (e.g. `hepta_shell`'s
/// zone-plus-neighbours superset) can never produce a wrong answer, only a
/// less-pruned one. An index that legitimately proves zero matches (e.g.
/// `NatiruIndex::surrogates_in_range` returning `Some(vec![])`) is
/// distinguished from "this index can't help here" (`None`), so a real
/// zero-match answer short-circuits instead of falling back to a full scan.
fn candidates_from_indexes(query: &HeptaQuery, idx: &HeptaIndexes) -> Option<Vec<u32>> {
    let mut acc: Option<Vec<u32>> = None;

    // WHEN clause → NatiruIndex. Safe for ANY WHEN (including a pinned past
    // epoch) because it only answers "did this surrogate write anything in
    // this range" -- never "what is its current value" (that's `eav`,
    // below, which the current-state restriction still applies to).
    if let Some(when) = &query.when {
        if let Some((lo, hi)) = nairu_range_from_when(when) {
            let surs = idx.nairu.surrogates_in_range(lo, hi)?;
            acc = Some(intersect_opt(acc, surs));
        }
    }

    // ORBITAL clause → RadixSplineIndex. A distinct clause from WHEN, so a
    // distinct index, even though both key on epoch today.
    if let Some((lo, hi)) = engine::orbital_range_from_query(query) {
        let surs = idx.radix.range_query(lo, hi);
        acc = Some(intersect_opt(acc, surs));
    }

    // WHERE leading exact-equality → EavExactIndex. Current-state only:
    // the index snapshot holds each attribute's latest value, so a query
    // pinned to a specific past epoch cannot safely use it (the WHEN
    // pruning above still applies to that case, just not this one).
    if when_is_current(&query.when) {
        if let Some((attr, val)) = indexable_condition(&query.r#where) {
            let attr_hash = engine::attr_hash_of(attr);
            let encoded = codec::encode(&hepta_value_to_akk(val));
            let vfp = EavExactIndex::val_fingerprint(&encoded);
            let surs = idx.eav.surrogates_for(attr_hash, vfp);
            acc = Some(intersect_opt(acc, surs));
        }
    }

    // WHERE pins all seven SPATIAL_ATTRS with exact equality, and the
    // query explicitly asked for the E7 anchor → HeptaShellIndex.
    if query.anchor == AnchorStrategy::E7First {
        if let Some(center) = spatial_center_from_where(&query.r#where) {
            let surs = idx.shell.spatial_query(&center, 1);
            acc = Some(intersect_opt(acc, surs));
        }
    }

    acc
}

/// AND-intersect `next` into the running candidate set. `next` need not
/// arrive sorted/deduplicated (only `NatiruIndex::surrogates_in_range` and
/// `HeptaShellIndex::spatial_query` already guarantee that) -- normalized
/// here once so every caller can rely on the invariant that `acc`, when
/// `Some`, is always sorted and deduplicated.
fn intersect_opt(acc: Option<Vec<u32>>, mut next: Vec<u32>) -> Vec<u32> {
    next.sort_unstable();
    next.dedup();
    match acc {
        None => next,
        Some(prev) => NatiruIndex::intersect_sorted(&prev, &next),
    }
}

/// Map a WHEN clause to the orbital-epoch range `NatiruIndex` should prune
/// on. `AtEpoch(Now)` returns `None`: "now" has no fixed epoch bound in
/// this codebase's current placeholder scheme (see `engine::
/// orbital_range_from_query`'s identical `Now` handling), and it doesn't
/// need one here -- the `eav` current-state path already covers that case.
///
/// `pub`: also called directly by `enkidb_readnode::cached::CachedReadNode`,
/// which builds its own `NatiruIndex` over its own in-memory particle set
/// (see that module's doc comment for why it can't share `HeptaIndexes`
/// itself) but must not reimplement this mapping a second time.
pub fn nairu_range_from_when(when: &WhenClause) -> Option<(u64, u64)> {
    match when {
        WhenClause::AtEpoch(EpochRef::Now) => None,
        WhenClause::AtEpoch(EpochRef::Abs(n)) => Some((*n, *n)),
        WhenClause::BeforeEpoch(n) => Some((0, n.saturating_sub(1))),
        WhenClause::AfterEpoch(n) => Some((n.saturating_add(1), u32::MAX as u64)),
    }
}

/// Extract a 7D centre point from a WHERE clause that consists of *exactly*
/// the seven `SPATIAL_ATTRS`, each pinned by exact equality and ANDed
/// together -- the one shape `HeptaShellIndex::spatial_query` can safely
/// answer a query for. Any other shape (missing a dimension, an extra
/// condition, an `OR`, a non-equality test) returns `None`, same
/// conservative-by-default posture as `indexable_condition` below.
///
/// `pub`: also called directly by `enkidb_readnode::cached::CachedReadNode`
/// (see `nairu_range_from_when`'s doc comment for why).
pub fn spatial_center_from_where(conditions: &[WhereCondition]) -> Option<[f64; 7]> {
    if conditions.len() != SPATIAL_ATTRS.len() {
        return None;
    }
    if conditions.iter().any(|c| c.combinator == Combinator::Or) {
        return None;
    }
    let mut center = [0f64; 7];
    let mut seen = [false; 7];
    for c in conditions {
        let dim = SPATIAL_ATTRS.iter().position(|&name| name == c.attr)?;
        if seen[dim] {
            return None;
        }
        match &c.test {
            ConditionTest::Cmp {
                op: Op::Eq,
                val: HeptaValue::Float(f),
            } => center[dim] = *f,
            ConditionTest::Cmp {
                op: Op::Eq,
                val: HeptaValue::Int(i),
            } => center[dim] = *i as f64,
            _ => return None,
        }
        seen[dim] = true;
    }
    seen.iter().all(|&s| s).then_some(center)
}

/// True when the query reads "current" state — no WHEN clause, or `WHEN AT
/// EPOCH NOW` (which `engine::epoch_filter_from_when` already treats as
/// "no filter"). The index snapshot only ever holds each attribute's
/// latest value, so a query pinned to a specific past epoch cannot safely
/// use it — that case must fall back to a real history replay.
fn when_is_current(when: &Option<WhenClause>) -> bool {
    matches!(when, None | Some(WhenClause::AtEpoch(EpochRef::Now)))
}

/// Returns the `(attribute, value)` to index on if `conditions` can be
/// safely pruned by its first entry alone: the first condition must be an
/// exact-equality test, and every other condition (if any) must be `AND`ed
/// in, never `OR`ed. `OR` would let the index silently drop particles that
/// only satisfy a *later* condition, which the first condition's surrogate
/// set knows nothing about — so any `OR` disables the shortcut entirely and
/// the full scan (which evaluates every condition faithfully) takes over.
fn indexable_condition(conditions: &[WhereCondition]) -> Option<(&str, &HeptaValue)> {
    if conditions.is_empty() {
        return None;
    }
    if conditions[1..]
        .iter()
        .any(|c| c.combinator == Combinator::Or)
    {
        return None;
    }
    match &conditions[0].test {
        ConditionTest::Cmp { op: Op::Eq, val } => Some((conditions[0].attr.as_str(), val)),
        _ => None,
    }
}

/// Convert a query-literal `HeptaValue` into the `AkkValue` that would have
/// been written for an equal value, so `codec::encode` produces identical
/// bytes to what's already stored — and therefore an identical fingerprint.
fn hepta_value_to_akk(v: &HeptaValue) -> AkkValue {
    match v {
        HeptaValue::Null => AkkValue::Null,
        HeptaValue::Bool(b) => AkkValue::Bool(*b),
        HeptaValue::Int(i) => AkkValue::Int(*i),
        HeptaValue::Float(f) => AkkValue::Float(*f),
        HeptaValue::Text(s) => AkkValue::Text(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::execute;
    use crate::parser::parse_query;
    use akkvalue::AkkValue;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_kaki::{mint::KakiMinter, EventKaki, IdentityKaki, KakiRole};

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
            .map(|(name, val)| EavTriple::new(engine::attr_hash_of(name), codec::encode(val)))
            .collect();
        jnl.append(JournalEntry::new(ek, target.clone(), epoch, eav))
            .unwrap();
    }

    fn new_entity(m: &KakiMinter) -> IdentityKaki {
        IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap()
    }

    /// The core promise: the indexed path and the full scan must always
    /// agree, for a query the index can actually accelerate.
    #[test]
    fn indexed_matches_full_scan_on_exact_eq() {
        let tribe = TribeId::from_u16(0x0101);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let najaf_particles: Vec<_> = (0..5).map(|_| new_entity(&m)).collect();
        for e in &najaf_particles {
            push_eav(
                &mut jnl,
                &m,
                e,
                1,
                &[("city.name", AkkValue::Text("Najaf".into()))],
            );
        }
        for _ in 0..20 {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("city.name", AkkValue::Text("Baghdad".into()))],
            );
        }

        let q = parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(indexed.matched.len(), 5);
        assert_eq!(full.matched.len(), indexed.matched.len());

        let mut full_bytes: Vec<_> = full.matched.iter().map(|m| *m.entity.bytes()).collect();
        let mut idx_bytes: Vec<_> = indexed.matched.iter().map(|m| *m.entity.bytes()).collect();
        full_bytes.sort();
        idx_bytes.sort();
        assert_eq!(full_bytes, idx_bytes);
    }

    /// The actual point of this module: indexing must look at far fewer
    /// candidates than a full scan when the WHERE clause is selective.
    #[test]
    fn indexed_evaluates_fewer_candidates_than_full_scan() {
        let tribe = TribeId::from_u16(0x0102);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let target = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &target,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );
        for _ in 0..99 {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("city.name", AkkValue::Text("Baghdad".into()))],
            );
        }

        let q = parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(full.evaluated, 100);
        assert_eq!(
            indexed.evaluated, 1,
            "index should prune to exactly the matching particle"
        );
        assert_eq!(indexed.matched.len(), 1);
    }

    /// A query with `OR` must not use the index shortcut — it must fall
    /// back to a full scan so the OR is evaluated correctly.
    #[test]
    fn or_condition_falls_back_to_full_scan() {
        let tribe = TribeId::from_u16(0x0103);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let e1 = new_entity(&m);
        let e2 = new_entity(&m);
        let e3 = new_entity(&m);
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
            &[("city.name", AkkValue::Text("Karbala".into()))],
        );
        push_eav(
            &mut jnl,
            &m,
            &e3,
            1,
            &[("city.name", AkkValue::Text("Baghdad".into()))],
        );

        let q =
            parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\" OR E[city.name] = \"Karbala\"")
                .unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(full.matched.len(), 2);
        assert_eq!(
            indexed.matched.len(),
            2,
            "OR must not be pruned away by the index shortcut"
        );
        assert_eq!(
            indexed.evaluated, full.evaluated,
            "OR queries must take the full-scan path"
        );
    }

    /// A query with no WHERE clause at all has nothing to index on — full
    /// scan, same as always.
    #[test]
    fn no_where_clause_falls_back_cleanly() {
        let tribe = TribeId::from_u16(0x0104);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..3 {
            let e = new_entity(&m);
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);
        }

        let q = parse_query("WHO T.E").unwrap();
        let idx = build_indexes(&jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);
        assert_eq!(indexed.matched.len(), 3);
    }

    /// A WHEN clause pinned to a specific past epoch must not use the
    /// snapshot index (it only reflects the latest value), and must still
    /// return the historically-correct answer via the full-scan fallback.
    #[test]
    fn when_before_epoch_falls_back_and_stays_correct() {
        let tribe = TribeId::from_u16(0x0105);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = new_entity(&m);

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

        let idx = build_indexes(&jnl);

        let q_past = parse_query("WHO T.E\nWHERE E[status] = \"active\"\nWHEN AT EPOCH 7").unwrap();
        let indexed_past = execute_indexed(&q_past, &jnl, &idx);
        assert_eq!(
            indexed_past.matched.len(),
            1,
            "status was 'active' as of epoch 7"
        );

        let q_now = parse_query("WHO T.E\nWHERE E[status] = \"archived\"").unwrap();
        let indexed_now = execute_indexed(&q_now, &jnl, &idx);
        assert_eq!(
            indexed_now.matched.len(),
            1,
            "status is 'archived' now, and this path can use the index"
        );
    }

    #[test]
    fn empty_journal_returns_zero_matches_indexed() {
        let jnl = Journal::new(64);
        let idx = build_indexes(&jnl);
        let q = parse_query("WHO T.E\nWHERE E[x] = 1").unwrap();
        let res = execute_indexed(&q, &jnl, &idx);
        assert_eq!(res.evaluated, 0);
        assert!(res.matched.is_empty());
    }

    /// Regression for a real O(n^2) bug found seeding 10M particles for
    /// the first time: build_indexes() used to call
    /// journal.read_particle_history() once per particle, and that call
    /// rescans its whole shard partition every time -- fine at the small
    /// scale every prior test here used, but with many particles sharing
    /// one shard+epoch (the common case for a bulk single-tribe seed) it
    /// made this function O(partition_size) per particle. 20,000 particles
    /// in one shard is enough to expose that (would take tens of seconds
    /// under the old per-particle-rescan behavior); the fix makes it a
    /// single O(n) grouping pass, so this must stay fast.
    #[test]
    fn build_indexes_stays_fast_with_many_particles_in_one_shard() {
        let tribe = TribeId::from_u16(0x0106);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        for i in 0..20_000u32 {
            let e = IdentityKaki::try_from_kaki(m.mint_identity(i, KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(i as i64))]);
        }

        let t0 = std::time::Instant::now();
        let idx = build_indexes(&jnl);
        let elapsed = t0.elapsed();
        assert!(
            elapsed.as_secs() < 3,
            "build_indexes on 20,000 same-shard particles took {elapsed:?} -- likely regressed to O(n^2)"
        );

        let q = parse_query("WHO T.E\nWHERE E[x] = 12345").unwrap();
        let res = execute_indexed(&q, &jnl, &idx);
        assert_eq!(res.matched.len(), 1);
    }

    /// docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md Block E-004/E-005,
    /// run for real at the doc's own literal 10,000,000-particle scale.
    /// `#[ignore]`d (not part of the default `cargo test` run) because
    /// minting 10M real KAKIs is real, non-trivial work that would slow
    /// every contributor's normal test loop for a check that only matters
    /// right before a real go-live gate -- run it deliberately with
    /// `cargo test --release -p heptascript -- --ignored e004_e005`.
    ///
    /// STATUS as of 2026-08-27 (this real run's own findings): two real,
    /// independent O(candidates × journal_size)-class bugs were found and
    /// fixed in `engine::histories_for` (see that function's own FIXED
    /// comments) -- E-004 went from never completing (killed after 11+
    /// real minutes) to a real, measured 3.8s, a ~170x verified speedup.
    /// The residual gap to E-004's <1s target, and E-005's 10.9s against
    /// a 100ms target despite correctly aborting at 100 candidates, is
    /// now precisely diagnosed rather than unexplained: `Journal` is
    /// documented at the top of journal.rs as *deliberately* un-indexed
    /// (real indexing belongs to the Read Node's Data Files, built in a
    /// separate batch process) -- adding a shard->partitions index to
    /// close this gap would mean reversing that explicit architectural
    /// choice, not fixing a bug. This test also exercises `Journal`
    /// directly, which is documented as the Write Node's own occasional-
    /// lookup structure, not the bulk query-serving path production
    /// traffic actually takes (`enkidb_readnode::materialize` + its own
    /// Data Files). That dedicated pass is now real:
    /// `crates/enkidb-readnode/tests/e004_e005_real_gate.rs` builds an
    /// actual Read Node from a 10M-particle store and queries that --
    /// needle lookup ~56-60us, a bounded (`HOW_MUCH LIMIT 1000`) query
    /// ~46-47ms, both consistently well under the 1s target
    /// (`gates/TESTING-PHASE1-ABCDEF.tsv`'s E-004/E-005 rows record it as
    /// SOLVED on that basis). This test stays as-is and un-fixed on
    /// purpose: it is the honest, permanent record of the Journal's own
    /// characteristics on its own deliberately-un-indexed path, not a
    /// gap still waiting to be closed.
    #[test]
    #[ignore = "10M-particle real-scale run; deliberate, not part of the default suite"]
    fn phase1_e004_e005_10m_particles() {
        let tribe = TribeId::from_u16(1);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let n: u32 = 10_000_000;
        let mint_start = std::time::Instant::now();
        for i in 0..n {
            let e = IdentityKaki::try_from_kaki(m.mint_identity(i, KakiRole::Zikru)).unwrap();
            // FIXED: `i % 1_000_000` made every one of the 10M particles'
            // epoch fall inside the E-004 query's own ORBITAL range
            // (0..999999) -- the range covers the entire domain of values
            // `i % 1_000_000` can ever produce, so it pruned nothing at
            // all and every particle became a candidate. Spreading epoch
            // across the full 10M range (one epoch per particle, like a
            // real timestamp) is what makes ORBITAL start/end a genuine,
            // realistic prune -- found live while diagnosing why E-004
            // took minutes instead of <1s.
            push_eav(&mut jnl, &m, &e, i, &[("x", AkkValue::Int(i as i64))]);
        }
        eprintln!("mint+append {n} particles: {:?}", mint_start.elapsed());

        let idx_start = std::time::Instant::now();
        let idx = build_indexes(&jnl);
        eprintln!("build_indexes: {:?}", idx_start.elapsed());

        // E-004: indexed orbital-range query, real acceptance criteria --
        // < 1s wall-clock, not aborted, at least one match.
        // NOTE: the doc's own literal example text reads "ANCHOR
        // SurrogateTime", "ORBITAL start = 0 end = 999999", and bare
        // "LIMIT 1000" in that clause order -- none of that parses
        // against this grammar. Found live running this test for real,
        // in three separate rounds: (1) anchor names are
        // SCREAMING_SNAKE_CASE (SURROGATE_TIME); (2) ORBITAL's real
        // syntax is `N .. N` (see parser.rs's own grammar comment,
        // `orbital := ORBITAL (N .. N | NOW)`), not `start = N end = N`;
        // (3) most subtly, `Parser::parse()` is strictly positional --
        // WHO, WHAT, WHERE, WHEN, ORBITAL, WHY, ..., HOW_MUCH, GRAVITY,
        // MEASURE, ANCHOR, STREAM, ABORT_SCAN, FILTER_ORDER, in that
        // exact order -- so even with correct ORBITAL syntax, writing
        // ANCHOR *before* ORBITAL in the source (as the doc's own
        // example does) makes `maybe_parse_orbital()` run while the
        // parser is still sitting on the ANCHOR token, silently
        // returning `None` (no error) instead of ever reading the real
        // ORBITAL clause a few tokens later. `q004.orbital == None` then
        // meant `candidates_from_indexes` had nothing to prune on, and
        // `execute_indexed` fell all the way back to a full
        // 10M-particle scan -- confirmed live, this is what actually
        // explained "matched=10000000" through three straight attempts
        // that each fixed one syntax issue but kept the wrong clause
        // order. Using the real, correctly-ordered syntax here.
        // FILTER_ORDER's real stage keywords are SCREAMING_SNAKE_CASE too
        // (ORBITAL_RANGE / EAV_ATTR) -- the doc's own "OrbitalRange
        // EavAttr" (PascalCase, matching the internal enum variant names,
        // not the actual source keywords) is a fourth real mismatch found
        // live in this same diagnosis.
        let q004 = parse_query(
            "WHO T.E\nORBITAL 0 .. 999999\nHOW_MUCH LIMIT 1000\n\
             ANCHOR SURROGATE_TIME\nABORT_SCAN 5000000\nFILTER_ORDER ORBITAL_RANGE EAV_ATTR",
        )
        .unwrap();
        let e004_start = std::time::Instant::now();
        let res004 = execute_indexed(&q004, &jnl, &idx);
        let e004_elapsed = e004_start.elapsed();
        eprintln!(
            "E-004: {:?} aborted={} matched={}",
            e004_elapsed,
            res004.aborted,
            res004.matched.len()
        );
        assert!(e004_elapsed.as_millis() < 1000, "E-004 exceeded 1s: {e004_elapsed:?}");
        assert!(!res004.aborted, "E-004 must not hit the ABORT_SCAN cap");
        assert!(!res004.matched.is_empty(), "E-004 must match at least one real particle");

        // E-005: ABORT_SCAN safety valve, full scan capped at 100 -- real
        // acceptance criteria: aborted == true, < 100ms.
        let q005 =
            parse_query("WHO T.E\nHOW_MUCH LIMIT 1000000\nANCHOR FULL_SCAN\nABORT_SCAN 100").unwrap();
        let e005_start = std::time::Instant::now();
        let res005 = execute(&q005, &jnl);
        let e005_elapsed = e005_start.elapsed();
        eprintln!("E-005: {:?} aborted={}", e005_elapsed, res005.aborted);
        assert!(res005.aborted, "E-005 must report aborted at a 100-candidate cap over 10M particles");
        assert!(e005_elapsed.as_millis() < 100, "E-005 exceeded 100ms: {e005_elapsed:?}");
    }

    // ── natiru_index (WHEN orbital-range pruning) ───────────────────────────

    /// The real point of this test: `when_before_epoch_falls_back_and_stays_
    /// correct` (above) named its own behavior "falls back" because before
    /// this pass, ANY non-`NOW` WHEN clause disabled indexing entirely. Now
    /// a `WHEN BEFORE/AFTER EPOCH` query is pruned by `NatiruIndex` before
    /// any history is read -- this test proves it evaluates strictly fewer
    /// candidates than the full particle count, and still agrees exactly
    /// with a full scan.
    #[test]
    fn when_after_epoch_uses_nairu_pruning_and_evaluates_fewer_candidates() {
        let tribe = TribeId::from_u16(0x0107);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        // 5 particles written only at epoch 100 (will match AFTER EPOCH 50).
        for _ in 0..5 {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                100,
                &[("status", AkkValue::Text("late".into()))],
            );
        }
        // 95 particles written only at epoch 1 (must be pruned out by nairu
        // before their history is ever read).
        for _ in 0..95 {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("status", AkkValue::Text("early".into()))],
            );
        }

        let q = parse_query("WHO T.E\nWHERE E[status] = \"late\"\nWHEN AFTER EPOCH 50").unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(full.matched.len(), 5);
        assert_eq!(
            indexed.matched.len(),
            5,
            "must agree exactly with the full scan"
        );
        assert_eq!(full.evaluated, 100);
        assert_eq!(
            indexed.evaluated, 5,
            "NatiruIndex should prune to exactly the 5 particles with an entry after epoch 50"
        );
    }

    /// `WHEN AT EPOCH <n>` (a specific past epoch, not `NOW`) must also use
    /// nairu pruning now, even though the `eav` current-state index
    /// correctly stays disabled for it (checked by `when_is_current` still
    /// gating the `eav` branch of `candidates_from_indexes`).
    #[test]
    fn when_at_past_epoch_uses_nairu_pruning() {
        let tribe = TribeId::from_u16(0x0108);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = new_entity(&m);

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
        // NatiruIndex buckets by BUCKET_ORBITALS=10, so noise must land in a
        // different bucket than epoch 7's (bucket 0) to be prunable at all —
        // epoch 1000 (bucket 100) is unambiguously a different bucket.
        for _ in 0..50 {
            let other = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &other,
                1000,
                &[("status", AkkValue::Text("noise".into()))],
            );
        }

        let idx = build_indexes(&jnl);

        let q_past = parse_query("WHO T.E\nWHERE E[status] = \"active\"\nWHEN AT EPOCH 7").unwrap();
        let full_past = execute(&q_past, &jnl);
        let indexed_past = execute_indexed(&q_past, &jnl, &idx);
        assert_eq!(
            indexed_past.matched.len(),
            1,
            "status was 'active' as of epoch 7"
        );
        assert_eq!(full_past.matched.len(), indexed_past.matched.len());
        assert!(
            indexed_past.evaluated < full_past.evaluated,
            "nairu pruning by AT EPOCH 7 should look at fewer than all 51 particles, got {}",
            indexed_past.evaluated
        );
    }

    // ── radix_spline (ORBITAL-clause range pruning) ─────────────────────────

    /// `ORBITAL start .. end` is a distinct clause from WHEN — this proves
    /// `RadixSplineIndex` prunes it correctly and agrees with a full scan,
    /// including when combined with a WHERE condition (AND-intersected via
    /// `intersect_opt`).
    #[test]
    fn orbital_range_uses_radix_pruning_and_evaluates_fewer_candidates() {
        let tribe = TribeId::from_u16(0x0109);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        for epoch in 0..100u32 {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                epoch,
                &[("kind", AkkValue::Text("particle".into()))],
            );
        }

        let q = parse_query("WHO T.E\nWHERE E[kind] = \"particle\"\nORBITAL 10 .. 19").unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(
            full.matched.len(),
            10,
            "epochs 10..=19 inclusive is 10 particles"
        );
        assert_eq!(indexed.matched.len(), full.matched.len());
        assert_eq!(full.evaluated, 100);
        assert_eq!(
            indexed.evaluated, 10,
            "radix range pruning should narrow to exactly the 10 in-range particles"
        );
    }

    /// An `ORBITAL` clause with no WHERE at all must still prune via radix
    /// alone (no `eav` branch applies), proving the two prunings compose
    /// independently rather than only ever firing together.
    #[test]
    fn orbital_range_alone_with_no_where_still_prunes() {
        let tribe = TribeId::from_u16(0x0110);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        for epoch in 0..50u32 {
            let e = new_entity(&m);
            push_eav(
                &mut jnl,
                &m,
                &e,
                epoch,
                &[("x", AkkValue::Int(epoch as i64))],
            );
        }

        let q = parse_query("WHO T.E\nORBITAL 0 .. 4").unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(indexed.matched.len(), 5);
        assert_eq!(full.matched.len(), indexed.matched.len());
        assert_eq!(
            indexed.evaluated, 5,
            "radix pruning alone should narrow to the 5 in-range particles"
        );
    }

    // ── hepta_shell (WHERE 7D spatial pruning via ANCHOR E7_FIRST) ──────────

    /// A WHERE clause pinning all seven `SPATIAL_ATTRS` by exact equality,
    /// combined with `ANCHOR E7_FIRST`, must use `HeptaShellIndex` and still
    /// agree exactly with a full scan (the shell index returns a zone +
    /// neighbours superset; `execute_over`'s real WHERE evaluation trims it
    /// to the true exact match).
    #[test]
    fn e7_first_anchor_with_full_spatial_where_uses_shell_pruning() {
        let tribe = TribeId::from_u16(0x0111);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let target = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &target,
            1,
            &[
                ("orbit.r", AkkValue::Float(1.0)),
                ("orbit.theta", AkkValue::Float(2.0)),
                ("orbit.phi", AkkValue::Float(3.0)),
                ("orbit.d3", AkkValue::Float(4.0)),
                ("orbit.d4", AkkValue::Float(5.0)),
                ("orbit.d5", AkkValue::Float(6.0)),
                ("orbit.d6", AkkValue::Float(7.0)),
            ],
        );
        // 40 decoys scattered elsewhere in the 7D lattice — none share the
        // target's exact zone.
        for i in 0..40 {
            let e = new_entity(&m);
            let v = (i as f64 + 100.0) * 3.0;
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[
                    ("orbit.r", AkkValue::Float(v)),
                    ("orbit.theta", AkkValue::Float(v)),
                    ("orbit.phi", AkkValue::Float(v)),
                    ("orbit.d3", AkkValue::Float(v)),
                    ("orbit.d4", AkkValue::Float(v)),
                    ("orbit.d5", AkkValue::Float(v)),
                    ("orbit.d6", AkkValue::Float(v)),
                ],
            );
        }

        let q = parse_query(
            "WHO T.E\n\
             WHERE E[orbit.r] = 1.0 AND E[orbit.theta] = 2.0 AND E[orbit.phi] = 3.0 \
             AND E[orbit.d3] = 4.0 AND E[orbit.d4] = 5.0 AND E[orbit.d5] = 6.0 AND E[orbit.d6] = 7.0\n\
             ANCHOR E7_FIRST",
        )
        .unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(full.matched.len(), 1);
        assert_eq!(
            indexed.matched.len(),
            1,
            "shell-pruned result must agree with the full scan"
        );
        assert_eq!(
            *full.matched[0].entity.bytes(),
            *indexed.matched[0].entity.bytes()
        );
        assert_eq!(full.evaluated, 41);
        assert!(
            indexed.evaluated < 41,
            "E7_FIRST anchor should prune to far fewer than all 41 particles, got {}",
            indexed.evaluated
        );
    }

    /// `spatial_center_from_where`'s shell-pruning branch is gated on
    /// `query.anchor == AnchorStrategy::E7First` explicitly (see
    /// `candidates_from_indexes`) — it is never inferred just because a
    /// WHERE clause happens to pin all seven spatial dims. This test
    /// doesn't isolate that gate's effect on `evaluated` (the leading
    /// WHERE condition is *also* a valid `eav` equality lookup, which
    /// prunes the same query on its own regardless of ANCHOR); it proves
    /// the weaker but still real property that omitting `ANCHOR E7_FIRST`
    /// does not change correctness — the shell index staying idle is
    /// enforced by `candidates_from_indexes`'s own `if query.anchor ==
    /// AnchorStrategy::E7First` guard, not re-derived here.
    #[test]
    fn full_spatial_where_without_e7_anchor_is_still_correct() {
        let tribe = TribeId::from_u16(0x0112);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &e,
            1,
            &[
                ("orbit.r", AkkValue::Float(1.0)),
                ("orbit.theta", AkkValue::Float(2.0)),
                ("orbit.phi", AkkValue::Float(3.0)),
                ("orbit.d3", AkkValue::Float(4.0)),
                ("orbit.d4", AkkValue::Float(5.0)),
                ("orbit.d5", AkkValue::Float(6.0)),
                ("orbit.d6", AkkValue::Float(7.0)),
            ],
        );

        let q = parse_query(
            "WHO T.E\n\
             WHERE E[orbit.r] = 1.0 AND E[orbit.theta] = 2.0 AND E[orbit.phi] = 3.0 \
             AND E[orbit.d3] = 4.0 AND E[orbit.d4] = 5.0 AND E[orbit.d5] = 6.0 AND E[orbit.d6] = 7.0",
        )
        .unwrap();
        assert_eq!(
            q.anchor,
            AnchorStrategy::Auto,
            "no ANCHOR clause was written"
        );

        let idx = build_indexes(&jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);
        assert_eq!(indexed.matched.len(), 1);
    }

    // ── combined pruning ─────────────────────────────────────────────────────

    /// WHEN + ORBITAL + WHERE-eq together must all AND-intersect correctly
    /// via `candidates_from_indexes`/`intersect_opt`, not just each in
    /// isolation.
    #[test]
    fn when_and_orbital_and_eav_prunings_intersect_correctly() {
        let tribe = TribeId::from_u16(0x0113);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        // The one particle that satisfies all three prunings at once.
        let target = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &target,
            20,
            &[("kind", AkkValue::Text("keep".into()))],
        );

        // Satisfies WHEN + ORBITAL but not the WHERE value.
        let wrong_kind = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &wrong_kind,
            20,
            &[("kind", AkkValue::Text("skip".into()))],
        );

        // Satisfies WHERE + ORBITAL but not WHEN (AFTER EPOCH 15 excludes epoch 5).
        let wrong_epoch = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &wrong_epoch,
            5,
            &[("kind", AkkValue::Text("keep".into()))],
        );

        // Satisfies WHERE + WHEN but not ORBITAL (epoch 50 is outside 10..30).
        let wrong_orbital = new_entity(&m);
        push_eav(
            &mut jnl,
            &m,
            &wrong_orbital,
            50,
            &[("kind", AkkValue::Text("keep".into()))],
        );

        let q =
            parse_query("WHO T.E\nWHERE E[kind] = \"keep\"\nWHEN AFTER EPOCH 15\nORBITAL 10 .. 30")
                .unwrap();
        let idx = build_indexes(&jnl);

        let full = execute(&q, &jnl);
        let indexed = execute_indexed(&q, &jnl, &idx);

        assert_eq!(full.matched.len(), 1);
        assert_eq!(indexed.matched.len(), 1);
        assert_eq!(*full.matched[0].entity.bytes(), *target.bytes());
        assert_eq!(*indexed.matched[0].entity.bytes(), *target.bytes());
        // Not 1: WHEN AFTER EPOCH disables the `eav` current-state branch
        // by design (it only holds *current* values, unsafe to use once a
        // query pins away from "now" — see `when_is_current`), so only
        // nairu (WHEN) and radix (ORBITAL) contribute pruning here, leaving
        // `wrong_kind` (epoch 20, in every range, wrong WHERE value) as a
        // candidate that `evaluate_capped`'s real WHERE check then correctly
        // excludes from `matched`. Both prunings still correctly exclude
        // `wrong_epoch` (epoch 5) and `wrong_orbital` (epoch 50, outside
        // ORBITAL 10..30), so 2 is the true, tight intersection, not 4.
        assert_eq!(
            indexed.evaluated, 2,
            "nairu ∩ radix should narrow to {{target, wrong_kind}} -- eav pruning can't apply under WHEN AFTER EPOCH"
        );
    }
}
