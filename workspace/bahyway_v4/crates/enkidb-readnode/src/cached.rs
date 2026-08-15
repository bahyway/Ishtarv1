//! CachedReadNode — an in-memory-resident Read Node.
//!
//! [`ReadNode`](crate::readnode::ReadNode) never loads a Data File into
//! memory: every query pays a fresh O(log n) disk read per matched
//! entity. That is the right tradeoff for instant-startup, low-RAM
//! deployments, but it means a query's *wall-clock* cost still scales
//! with how many rows it touches — real I/O, no amount of indexing
//! removes it.
//!
//! `CachedReadNode` makes the other tradeoff: [`CachedReadNode::open`]
//! pays a real O(n) cost once — reading the entire Data File in two bulk
//! reads via [`enkidb_datafile::DataFileReader::iter_all_raw`], not one
//! seek per record — and holds every entity's full decoded history plus
//! every EAV posting list in RAM. After that, every query (indexed or
//! not) touches zero disk I/O. This is the real "put pages in memory and
//! run the indexes on them" design: build once, query forever from RAM,
//! reload on the same schedule Data Files already sync on.
//!
//! Because everything is resident, this also closes a real capability
//! gap `ReadNode` has by design: a query that isn't a single indexed
//! equality condition (a second WHERE condition, an `OR`, no WHERE at
//! all) doesn't need `ReadNodeError::RequiresWriteNode` here — the full
//! entity set is already in memory, so an unindexed query is just a full
//! in-memory scan via the same `heptascript::execute_over_histories`
//! every other path uses, still with zero disk I/O.
//!
//! ## Surrogate keys, not raw KAKI, for the internal index
//!
//! First version of this module kept `entities` as a `HashMap<[u8; 16],
//! _>` (raw KAKI bytes as the key) and posting lists as `Vec<[u8; 16]>`.
//! Measured live at 10M entities: a `HOW_MUCH LIMIT 1000` query against
//! that shape took 135ms — 1000 lookups into a many-GB `HashMap`, each
//! one a genuinely random probe (a posting list's KAKI order has no
//! relationship to the hashmap's internal bucket layout), so almost
//! every lookup was a real cache/TLB miss. The *un*indexed full-scan
//! fallback, which walks the hashmap's own `.values()` in its native
//! bucket order, took 1.27ms on the identical dataset — same node count,
//! 100x apart, purely from access pattern.
//!
//! `enkidb_indexes::SurrogateMap` (already real, already tested, already
//! part of this codebase's index stack — see `crates/enkidb-indexes/src/
//! surrogate.rs`) resolves a KAKI to a dense `u32` via sorted-array
//! binary search over a single ~20-bytes/entry contiguous allocation —
//! far more cache-friendly per probe than a scattered hashmap even
//! though it's still O(log n). The payoff is what happens *after* that:
//! `by_surrogate[id]` is then a direct array index, O(1), one memory
//! access, no hashing, no probing at all — matching the full scan's own
//! access pattern for the indexed path too. Posting lists store `u32`
//! surrogate ids (4 bytes) instead of 16-byte KAKIs as a direct
//! consequence, a further 4x reduction in posting-list memory traffic.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use akkvalue::{codec, AkkValue};
use enkidb_datafile::DataFileReader;
use enkidb_indexes::prelude::{EavExactIndex, HeptaShellIndex, NatiruIndex, SurrogateMap};
use enkidb_indexes::radix_spline::{RadixSplineIndex, DEFAULT_EPSILON as RADIX_DEFAULT_EPSILON};
use enkidb_journal::entry::JournalEntry;
use enkidb_kaki::{IdentityKaki, Kaki};
use heptascript::{
    execute_over_histories, nairu_range_from_when, orbital_range_from_query, parse_query,
    spatial_center_from_where, AnchorStrategy, Combinator, ConditionTest, EpochRef, HeptaQuery,
    HeptaValue, Op, ParseError, QueryResult, WhenClause, WhereCondition, SPATIAL_ATTRS,
};

use crate::materialize::{decode_history, decode_posting_list, pack_eav_key};

#[derive(Debug)]
pub enum CachedReadNodeError {
    Io(io::Error),
    CorruptRecord,
}

impl From<io::Error> for CachedReadNodeError {
    fn from(e: io::Error) -> Self {
        CachedReadNodeError::Io(e)
    }
}

pub struct CachedReadNode {
    /// KAKI -> dense surrogate u32, sorted-array binary search.
    surrogates: SurrogateMap,
    /// surrogate id -> this entity's identity + full history. Direct
    /// array index once a surrogate is resolved — the whole point.
    by_surrogate: Vec<(IdentityKaki, Vec<JournalEntry>)>,
    /// packed (attr_hash, value_fingerprint) -> matching surrogate ids.
    eav_postings: HashMap<[u8; 16], Vec<u32>>,
    /// WHEN-clause orbital-range pruning (`NatiruIndex`) — same Anu Index
    /// Stack module `heptascript::indexed::HeptaIndexes` uses, built here a
    /// second time because `CachedReadNode` is a distinct in-memory index
    /// over Data-File-materialized entities, not the Journal-backed
    /// `HeptaIndexes` (see this module's own doc comment for why). Real
    /// EnkiDDB/EnkiMDB traffic goes through this struct
    /// (`enkiddb`/`enkimdb`'s `lib.rs`), so wiring only `HeptaIndexes`
    /// would leave the deployed read-servers unaccelerated.
    nairu: NatiruIndex,
    /// ORBITAL-clause range pruning (`RadixSplineIndex`).
    radix: RadixSplineIndex,
    /// `ANCHOR E7_FIRST` WHERE-clause spatial pruning (`HeptaShellIndex`).
    shell: HeptaShellIndex,
}

impl CachedReadNode {
    /// Load an entire Data File pair into memory. O(n) in the number of
    /// entities and postings — the one-time cost this whole module
    /// exists to pay exactly once, not per query. Call again on the same
    /// schedule a live server already reloads Data Files on (see
    /// `enkiddb-read-server`'s `RELOAD_SECS`).
    pub fn open(
        entities_base: impl AsRef<Path>,
        eav_index_base: impl AsRef<Path>,
    ) -> Result<Self, CachedReadNodeError> {
        let mut entities_reader = DataFileReader::open(entities_base)?;
        let mut eav_reader = DataFileReader::open(eav_index_base)?;

        // Decode every entity in one pass, keeping the raw KAKI bytes
        // list in the exact same order as `by_surrogate` so
        // `SurrogateMap::build` (which assigns surrogate `i` to
        // `kaki_bytes[i]`) lines up with it by construction.
        let raw = entities_reader.iter_all_raw()?;
        let mut kaki_bytes: Vec<[u8; 16]> = Vec::with_capacity(raw.len());
        let mut by_surrogate: Vec<(IdentityKaki, Vec<JournalEntry>)> = Vec::with_capacity(raw.len());
        for (bytes, blob) in raw {
            let identity = IdentityKaki::try_from_kaki(
                Kaki::from_bytes(bytes).map_err(|_| CachedReadNodeError::CorruptRecord)?,
            )
            .map_err(|_| CachedReadNodeError::CorruptRecord)?;
            let history = decode_history(&identity, &blob).ok_or(CachedReadNodeError::CorruptRecord)?;
            kaki_bytes.push(bytes);
            by_surrogate.push((identity, history));
        }
        let surrogates = SurrogateMap::build(&kaki_bytes);

        let mut eav_postings = HashMap::new();
        for (key, blob) in eav_reader.iter_all_raw()? {
            let ids: Vec<u32> = decode_posting_list(&blob)
                .iter()
                .filter_map(|k| surrogates.resolve(k))
                .collect();
            eav_postings.insert(key, ids);
        }

        // One more O(n) pass over the already-resident `by_surrogate` (no
        // extra disk I/O -- everything here is already in RAM) to build the
        // three Anu Index Stack members `heptascript::indexed::HeptaIndexes`
        // also builds, for the reasons this struct's own doc comment gives.
        let spatial_hashes: [u32; 7] = SPATIAL_ATTRS.map(|name| bahyway_crc::crc16(name.as_bytes()) as u32);
        let mut orbital_pairs: Vec<(u64, u32)> = Vec::new();
        let mut shell_entries: Vec<([f64; 7], u32)> = Vec::new();

        for (sur, (_, history)) in by_surrogate.iter().enumerate() {
            let sur = sur as u32;
            for entry in history {
                orbital_pairs.push((entry.epoch as u64, sur));
            }

            let mut position = [0f64; 7];
            let mut has_all_dims = true;
            for (dim, hash) in spatial_hashes.iter().enumerate() {
                let latest = history
                    .iter()
                    .filter(|e| e.eav.iter().any(|t| t.attr_hash == *hash))
                    .max_by_key(|e| e.epoch)
                    .and_then(|e| e.eav.iter().find(|t| t.attr_hash == *hash))
                    .and_then(|t| decode_f64(&t.value));
                match latest {
                    Some(v) => position[dim] = v,
                    None => { has_all_dims = false; }
                }
            }
            if has_all_dims {
                shell_entries.push((position, sur));
            }
        }

        let nairu = NatiruIndex::build(0, &orbital_pairs);

        orbital_pairs.sort_unstable_by_key(|&(epoch, _)| epoch);
        let radix_keys: Vec<u64> = orbital_pairs.iter().map(|&(epoch, _)| epoch).collect();
        let radix_values: Vec<u32> = orbital_pairs.iter().map(|&(_, sur)| sur).collect();
        let radix = RadixSplineIndex::build(radix_keys, radix_values, RADIX_DEFAULT_EPSILON);

        let shell = HeptaShellIndex::build(&shell_entries);

        Ok(Self { surrogates, by_surrogate, eav_postings, nairu, radix, shell })
    }

    /// Number of entities resident in memory.
    pub fn entity_count(&self) -> usize {
        self.by_surrogate.len()
    }

    /// Parse and execute a HeptaScript W5H2 query entirely from memory.
    /// Unlike [`ReadNode::query`](crate::readnode::ReadNode::query),
    /// every query shape is served — there is no `RequiresWriteNode`
    /// case, because the full entity set is already resident.
    pub fn query(&self, source: &str) -> Result<QueryResult, ParseError> {
        let q = parse_query(source)?;
        Ok(self.query_parsed(&q))
    }

    fn query_parsed(&self, q: &HeptaQuery) -> QueryResult {
        // WHEN / ORBITAL / ANCHOR E7_FIRST pruning -- the same Anu Index
        // Stack members `heptascript::indexed::candidates_from_indexes`
        // wires into `HeptaIndexes`, wired here a second time because
        // `CachedReadNode` is EnkiDDB/EnkiMDB's real deployed read path
        // (see this module's own doc comment) and does not go through
        // `HeptaIndexes` at all. Only entered when at least one of those
        // clauses is present -- every existing single-eav-condition query
        // (the overwhelming common case, exercised by every test above
        // this change) has `when`/`orbital` both `None` and `anchor ==
        // Auto`, so it skips this branch entirely and hits the unchanged
        // fast path below, byte-for-byte the same as before this wiring.
        if q.when.is_some() || q.orbital.is_some() || q.anchor == AnchorStrategy::E7First {
            if let Some(surs) = self.candidates_from_indexes(q) {
                let histories: Vec<(IdentityKaki, Vec<JournalEntry>)> = surs
                    .into_iter()
                    .filter_map(|id| self.by_surrogate.get(id as usize))
                    .cloned()
                    .collect();
                return execute_over_histories(q, &histories);
            }
        }

        if when_is_current(&q.when) {
            if let Some((attr, val)) = indexable_condition(&q.r#where) {
                let attr_hash = bahyway_crc::crc16(attr.as_bytes()) as u32;
                let encoded = akkvalue::codec::encode(&hepta_value_to_akk(val));
                let vfp = EavExactIndex::val_fingerprint(&encoded);
                let key = pack_eav_key(attr_hash, vfp);

                // Same bug class PB-221 fixed in `ReadNode` -- building
                // the full matched Vec before `execute_over_histories`
                // ever gets a chance to apply HOW_MUCH LIMIT defeats the
                // limit's whole point. When the query's entire filter is
                // this one indexed equality condition (see
                // `safe_early_limit`'s doc for exactly why that's the
                // safe case), the posting list itself already *is* the
                // complete answer set, so truncating it before cloning
                // is correct, not just fast. Each surrogate id resolves
                // to its entity via a direct array index
                // (`by_surrogate[id]`) -- see this module's own doc
                // comment for why that beats a KAKI-keyed hashmap probe
                // at scale.
                let ids_ref = self.eav_postings.get(&key);
                let take_n = safe_early_limit(q);
                let histories: Vec<(IdentityKaki, Vec<JournalEntry>)> = match ids_ref {
                    Some(ids) => {
                        let iter = ids.iter().filter_map(|&id| self.by_surrogate.get(id as usize));
                        match take_n {
                            Some(n) => iter.take(n).cloned().collect(),
                            None => iter.cloned().collect(),
                        }
                    }
                    None => Vec::new(),
                };

                return execute_over_histories(q, &histories);
            }
        }

        // Full in-memory scan -- still zero disk I/O, just no index
        // pruning. Real capability `ReadNode` doesn't have at all.
        //
        // Same early-truncation logic applies when there's no WHERE
        // clause at all: every resident entity trivially "matches" (no
        // filter to fail), so any N of them, taken before cloning, is a
        // valid, complete LIMIT-N answer. `by_surrogate` is a plain
        // `Vec`, so this is already a pure sequential scan -- no hashing
        // involved on this path even before the surrogate-key change.
        let all: Vec<(IdentityKaki, Vec<JournalEntry>)> = match safe_full_scan_limit(q) {
            Some(n) => self.by_surrogate.iter().take(n).cloned().collect(),
            None => self.by_surrogate.clone(),
        };
        execute_over_histories(q, &all)
    }

    /// Resolve a raw KAKI to its dense surrogate id, for callers that
    /// need the surrogate-key mapping directly (e.g. a future binary
    /// wire protocol that addresses particles by surrogate rather than
    /// the full 16-byte KAKI within one connection's lifetime).
    pub fn resolve_surrogate(&self, kaki: &[u8; 16]) -> Option<u32> {
        self.surrogates.resolve(kaki)
    }

    /// AND-intersect every applicable index into one candidate surrogate
    /// set, mirroring `heptascript::indexed::candidates_from_indexes`'s
    /// same algorithm (see that function's doc comment for why an
    /// over-inclusive candidate set can never produce a wrong answer, only
    /// a less-pruned one — `execute_over_histories` still evaluates the
    /// real WHERE/WHY/ORBITAL/WHEN against each candidate's full history
    /// afterward). `None` means no applicable index strategy was found.
    fn candidates_from_indexes(&self, q: &HeptaQuery) -> Option<Vec<u32>> {
        let mut acc: Option<Vec<u32>> = None;

        if let Some(when) = &q.when {
            if let Some((lo, hi)) = nairu_range_from_when(when) {
                let surs = self.nairu.surrogates_in_range(lo, hi)?;
                acc = Some(intersect_opt(acc, surs));
            }
        }

        if let Some((lo, hi)) = orbital_range_from_query(q) {
            let surs = self.radix.range_query(lo, hi);
            acc = Some(intersect_opt(acc, surs));
        }

        if when_is_current(&q.when) {
            if let Some((attr, val)) = indexable_condition(&q.r#where) {
                let attr_hash = bahyway_crc::crc16(attr.as_bytes()) as u32;
                let encoded = codec::encode(&hepta_value_to_akk(val));
                let vfp = EavExactIndex::val_fingerprint(&encoded);
                let key = pack_eav_key(attr_hash, vfp);
                let surs = self.eav_postings.get(&key).cloned().unwrap_or_default();
                acc = Some(intersect_opt(acc, surs));
            }
        }

        if q.anchor == AnchorStrategy::E7First {
            if let Some(center) = spatial_center_from_where(&q.r#where) {
                let surs = self.shell.spatial_query(&center, 1);
                acc = Some(intersect_opt(acc, surs));
            }
        }

        acc
    }
}

/// AND-intersect `next` into the running candidate set — see
/// `heptascript::indexed`'s identical helper of the same name.
fn intersect_opt(acc: Option<Vec<u32>>, mut next: Vec<u32>) -> Vec<u32> {
    next.sort_unstable();
    next.dedup();
    match acc {
        None => next,
        Some(prev) => NatiruIndex::intersect_sorted(&prev, &next),
    }
}

/// `Some(n)` exactly when the indexed posting list is the query's
/// complete, correct answer set (see `query_parsed`'s call site), so
/// pre-truncating it to `n` entries before cloning is safe.
fn safe_early_limit(q: &HeptaQuery) -> Option<usize> {
    let limit = match &q.how_much {
        Some(heptascript::HowMuchClause::Limit(n)) => *n,
        _ => return None,
    };
    if q.how.is_some() || !q.why.is_empty() || q.r#where.len() != 1 {
        return None;
    }
    // See `enkidb_readnode::readnode::safe_early_limit`'s identical guard:
    // ORBITAL is now a real filter (`heptascript::engine::evaluate_capped`
    // applies it), so pre-truncating the posting list before that filter
    // runs could silently undercount.
    if q.orbital.is_some() {
        return None;
    }
    if q.verb != heptascript::QueryVerb::Orbit {
        return None;
    }
    // A MEASURE/GRAVITY aggregate must see every matching particle, not
    // just the first `limit` of them -- same reason
    // `execute_over_histories` itself disables its own early-exit for
    // these clauses. Pre-truncating the posting list here, one layer
    // above that function, would silently undercount past the same
    // early-exit bug PB-221 already fixed once for plain HOW_MUCH LIMIT.
    if q.measure.is_some() || q.gravity.is_some() {
        return None;
    }
    Some(limit)
}

/// `Some(n)` exactly when the full-scan fallback's entire candidate set
/// is already the correct answer: no WHERE clause to filter on (so
/// nothing can be excluded), no WHY, no HOW sort, plain ORBIT, plain
/// LIMIT. Any N residents, taken before cloning, is then a valid,
/// complete LIMIT-N answer.
fn safe_full_scan_limit(q: &HeptaQuery) -> Option<usize> {
    let limit = match &q.how_much {
        Some(heptascript::HowMuchClause::Limit(n)) => *n,
        _ => return None,
    };
    if !q.r#where.is_empty() || q.how.is_some() || !q.why.is_empty() {
        return None;
    }
    // Same ORBITAL guard as `safe_early_limit` above: taking the first
    // `limit` residents before ORBITAL filtering runs could skip past
    // true in-range matches later in `by_surrogate`'s order.
    if q.orbital.is_some() {
        return None;
    }
    if q.verb != heptascript::QueryVerb::Orbit {
        return None;
    }
    // See `safe_early_limit`'s identical guard: a MEASURE/GRAVITY
    // aggregate needs the complete resident set, not just the first
    // `limit` entities.
    if q.measure.is_some() || q.gravity.is_some() {
        return None;
    }
    Some(limit)
}

fn when_is_current(when: &Option<WhenClause>) -> bool {
    matches!(when, None | Some(WhenClause::AtEpoch(EpochRef::Now)))
}

fn indexable_condition(conditions: &[WhereCondition]) -> Option<(&str, &HeptaValue)> {
    if conditions.is_empty() {
        return None;
    }
    if conditions[1..].iter().any(|c| c.combinator == Combinator::Or) {
        return None;
    }
    match &conditions[0].test {
        ConditionTest::Cmp { op: Op::Eq, val } => Some((conditions[0].attr.as_str(), val)),
        _ => None,
    }
}

/// Decode a raw EAV value blob to `f64` for `HeptaShellIndex`'s numeric
/// position dimensions. Mirrors `heptascript::indexed`'s private
/// `decode_f64` exactly (not exported from that crate, so duplicated here
/// rather than made `pub` for one non-generic helper).
fn decode_f64(bytes: &[u8]) -> Option<f64> {
    let (v, _) = codec::decode(bytes, 0).ok()?;
    match v {
        AkkValue::Float(f) => Some(f),
        AkkValue::Int(i)   => Some(i as f64),
        _ => None,
    }
}

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
    use crate::materialize::materialize;
    use akkvalue::codec;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::EavTriple;
    use enkidb_journal::Journal;
    use enkidb_kaki::{mint::KakiMinter, EventKaki, KakiRole};
    use std::fs;
    use std::path::PathBuf;

    fn tmp_base(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkidb_readnode_cached_{name}"));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        dir.join("base")
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
            .map(|(name, val)| EavTriple::new(bahyway_crc::crc16(name.as_bytes()) as u32, codec::encode(val)))
            .collect();
        jnl.append(JournalEntry::new(ek, target.clone(), epoch, eav)).unwrap();
    }

    /// The core promise: identical to a full scan, for the queries a
    /// disk-based ReadNode could already serve.
    #[test]
    fn cached_matches_full_scan_on_exact_eq() {
        let tribe = TribeId::from_u16(0x0401);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let najaf: Vec<_> = (0..5)
            .map(|_| IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap())
            .collect();
        for e in &najaf {
            push_eav(&mut jnl, &m, e, 1, &[("city.name", AkkValue::Text("Najaf".into()))]);
        }
        for _ in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 1, &[("city.name", AkkValue::Text("Baghdad".into()))]);
        }

        let base = tmp_base("matches_full_scan");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let full = heptascript::execute(&parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap(), &jnl);
        let via_crn = crn.query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap();

        assert_eq!(via_crn.matched.len(), 5);
        let mut full_bytes: Vec<_> = full.matched.iter().map(|m| *m.entity.bytes()).collect();
        let mut crn_bytes: Vec<_> = via_crn.matched.iter().map(|m| *m.entity.bytes()).collect();
        full_bytes.sort();
        crn_bytes.sort();
        assert_eq!(full_bytes, crn_bytes);
    }

    /// The real capability gap this closes: a query `ReadNode` would
    /// reject (a second WHERE condition) must still work here, and
    /// still agree with a real full scan.
    #[test]
    fn cached_serves_queries_readnode_would_reject() {
        let tribe = TribeId::from_u16(0x0402);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        for i in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            let age = if i % 2 == 0 { 20 } else { 40 };
            push_eav(
                &mut jnl, &m, &e, 1,
                &[("city.name", AkkValue::Text("Najaf".into())), ("age", AkkValue::Int(age))],
            );
        }

        let base = tmp_base("serves_rejected_queries");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let q = "WHO T.E\nWHERE E[city.name] = \"Najaf\" AND E[age] > 30";
        let full = heptascript::execute(&parse_query(q).unwrap(), &jnl);
        let via_crn = crn.query(q).unwrap();
        assert_eq!(via_crn.matched.len(), 10);
        assert_eq!(full.matched.len(), via_crn.matched.len());

        // No WHERE at all -- ReadNode::query would return RequiresWriteNode.
        let no_where = crn.query("WHO T.E").unwrap();
        assert_eq!(no_where.matched.len(), 20);

        // OR -- same story.
        let or_q = "WHO T.E\nWHERE E[age] = 20 OR E[age] = 40";
        let or_result = crn.query(or_q).unwrap();
        assert_eq!(or_result.matched.len(), 20);
    }

    #[test]
    fn cached_unknown_value_returns_zero_matches() {
        let tribe = TribeId::from_u16(0x0403);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        push_eav(&mut jnl, &m, &e, 1, &[("city.name", AkkValue::Text("Najaf".into()))]);

        let base = tmp_base("unknown_value");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = crn.query("WHO T.E\nWHERE E[city.name] = \"Karbala\"").unwrap();
        assert!(res.matched.is_empty());
    }

    #[test]
    fn cached_how_much_limit_still_honored_and_correct_with_how_sort() {
        let tribe = TribeId::from_u16(0x0404);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for rank in 0..50 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl, &m, &e, 1,
                &[("city.name", AkkValue::Text("Najaf".into())), ("rank", AkkValue::Int(rank))],
            );
        }

        let base = tmp_base("limit_with_how_sort");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = crn
            .query("WHO T.E\nWHAT E[rank]\nWHERE E[city.name] = \"Najaf\"\nHOW BY E[rank] DESC\nHOW_MUCH LIMIT 3")
            .unwrap();
        assert_eq!(res.matched.len(), 3);
        let ranks: Vec<i64> = res
            .matched
            .iter()
            .map(|m| match m.projected.iter().find(|(k, _)| k == "rank").map(|(_, v)| v) {
                Some(AkkValue::Int(n)) => *n,
                other => panic!("expected rank to be projected as Int, got {other:?}"),
            })
            .collect();
        assert_eq!(ranks, vec![49, 48, 47]);
    }

    #[test]
    fn cached_limit_truncates_indexed_match_correctly() {
        let tribe = TribeId::from_u16(0x0405);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..500 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 1, &[("city.name", AkkValue::Text("Najaf".into()))]);
        }

        let base = tmp_base("limit_truncates_indexed");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = crn.query("WHO T.E\nWHERE E[city.name] = \"Najaf\"\nHOW_MUCH LIMIT 10").unwrap();
        assert_eq!(res.matched.len(), 10, "500 true matches exist; LIMIT 10 must still return exactly 10");
    }

    #[test]
    fn cached_limit_truncates_full_scan_correctly() {
        let tribe = TribeId::from_u16(0x0406);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..500 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);
        }

        let base = tmp_base("limit_truncates_full_scan");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = crn.query("WHO T.E\nHOW_MUCH LIMIT 10").unwrap();
        assert_eq!(res.matched.len(), 10, "500 entities exist with no filter; LIMIT 10 must still return exactly 10");
    }

    #[test]
    fn cached_full_scan_limit_does_not_short_circuit_with_a_where_clause() {
        // A WHERE clause means not every resident entity trivially
        // matches -- `safe_full_scan_limit` must decline (only the
        // indexed fast path may truncate a WHERE-filtered query), so
        // this must fall through to the (correct, if slower) full-scan
        // filter-then-limit path and still find the true 3 matches.
        let tribe = TribeId::from_u16(0x0407);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for i in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            let status = if i < 3 { "active" } else { "inactive" };
            push_eav(&mut jnl, &m, &e, 1, &[("status", AkkValue::Text(status.into()))]);
        }

        let base = tmp_base("full_scan_limit_with_where");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        // OR disables the indexed fast path -- exercises the full-scan
        // branch with a real WHERE clause present.
        let res = crn
            .query("WHO T.E\nWHERE E[status] = \"active\" OR E[status] = \"active\"\nHOW_MUCH LIMIT 10")
            .unwrap();
        assert_eq!(res.matched.len(), 3, "only 3 true matches exist; must not overcount or undercount");
    }

    #[test]
    fn cached_measure_dense_ignores_how_much_limit_on_the_indexed_fast_path() {
        // Regression: `safe_early_limit` would otherwise pre-truncate the
        // indexed posting list to 10 entries *before* `execute_over_
        // histories` ever runs, silently reporting DENSE(10) instead of
        // the true count of 500 -- the same early-exit bug class PB-221
        // already fixed once for plain row results, resurfacing one
        // layer up for the new aggregate clauses.
        let tribe = TribeId::from_u16(0x0408);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..500 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 1, &[("city.name", AkkValue::Text("Najaf".into()))]);
        }

        let base = tmp_base("measure_ignores_indexed_limit");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = crn
            .query("WHO T.E\nWHERE E[city.name] = \"Najaf\"\nHOW_MUCH LIMIT 10\nMEASURE DENSE")
            .unwrap();
        assert_eq!(res.matched.len(), 10, "returned rows still honor LIMIT");
        assert_eq!(
            res.measured,
            Some(heptascript::MeasureValue::Dense(500)),
            "the aggregate must reflect all 500 true matches, not the LIMIT-truncated posting list"
        );
    }

    #[test]
    fn cached_measure_dense_ignores_how_much_limit_on_the_full_scan_path() {
        let tribe = TribeId::from_u16(0x0409);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..500 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 1, &[("x", AkkValue::Int(1))]);
        }

        let base = tmp_base("measure_ignores_full_scan_limit");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = crn.query("WHO T.E\nHOW_MUCH LIMIT 10\nMEASURE DENSE").unwrap();
        assert_eq!(res.matched.len(), 10);
        assert_eq!(res.measured, Some(heptascript::MeasureValue::Dense(500)));
    }

    // ── Anu Index Stack wiring: nairu / radix / hepta_shell ─────────────────
    //
    // These prove the same three previously-unwired indexes
    // (`heptascript::indexed`'s own tests prove them at the `HeptaIndexes`
    // level) are real and correct through `CachedReadNode` -- the struct
    // EnkiDDB's and EnkiMDB's actual deployed read-servers query (see
    // `enkiddb::lib.rs`/`enkimdb::lib.rs`), not just the in-memory Journal
    // path `HeptaIndexes` accelerates.

    #[test]
    fn cached_when_after_epoch_uses_nairu_pruning() {
        let tribe = TribeId::from_u16(0x0410);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        for _ in 0..5 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 100, &[("status", AkkValue::Text("late".into()))]);
        }
        for _ in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, 1, &[("status", AkkValue::Text("early".into()))]);
        }

        let base = tmp_base("nairu_when_after_epoch");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let full = heptascript::execute(
            &parse_query("WHO T.E\nWHERE E[status] = \"late\"\nWHEN AFTER EPOCH 50").unwrap(),
            &jnl,
        );
        let via_crn = crn.query("WHO T.E\nWHERE E[status] = \"late\"\nWHEN AFTER EPOCH 50").unwrap();

        assert_eq!(full.matched.len(), 5);
        assert_eq!(via_crn.matched.len(), 5, "must agree with a full journal scan");
    }

    #[test]
    fn cached_orbital_range_uses_radix_pruning() {
        let tribe = TribeId::from_u16(0x0411);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        for epoch in 0..50u32 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, epoch, &[("kind", AkkValue::Text("particle".into()))]);
        }

        let base = tmp_base("radix_orbital_range");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let full = heptascript::execute(
            &parse_query("WHO T.E\nWHERE E[kind] = \"particle\"\nORBITAL 10 .. 19").unwrap(),
            &jnl,
        );
        let via_crn = crn
            .query("WHO T.E\nWHERE E[kind] = \"particle\"\nORBITAL 10 .. 19")
            .unwrap();

        assert_eq!(full.matched.len(), 10, "epochs 10..=19 inclusive is 10 particles");
        assert_eq!(via_crn.matched.len(), 10, "must agree with a full journal scan");
    }

    #[test]
    fn cached_orbital_range_with_limit_does_not_undercount() {
        // Regression for the correctness gap fixed alongside this wiring:
        // `safe_early_limit`/`safe_full_scan_limit` must decline once
        // ORBITAL is present (both now guard on `q.orbital.is_some()`), or
        // the old code would pre-truncate the candidate list before
        // ORBITAL filtering ever ran, silently returning fewer than the
        // true LIMIT-worth of in-range matches.
        let tribe = TribeId::from_u16(0x0412);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        for epoch in 0..50u32 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(&mut jnl, &m, &e, epoch, &[("kind", AkkValue::Text("particle".into()))]);
        }

        let base = tmp_base("radix_orbital_with_limit");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = crn
            .query("WHO T.E\nWHERE E[kind] = \"particle\"\nORBITAL 10 .. 19\nHOW_MUCH LIMIT 100")
            .unwrap();
        assert_eq!(res.matched.len(), 10, "only 10 true in-range matches exist; LIMIT 100 must not undercount");
    }

    #[test]
    fn cached_e7_first_anchor_uses_shell_pruning() {
        let tribe = TribeId::from_u16(0x0413);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let target = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        push_eav(
            &mut jnl, &m, &target, 1,
            &[
                ("orbit.r", AkkValue::Float(1.0)), ("orbit.theta", AkkValue::Float(2.0)),
                ("orbit.phi", AkkValue::Float(3.0)), ("orbit.d3", AkkValue::Float(4.0)),
                ("orbit.d4", AkkValue::Float(5.0)), ("orbit.d5", AkkValue::Float(6.0)),
                ("orbit.d6", AkkValue::Float(7.0)),
            ],
        );
        for i in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            let v = (i as f64 + 100.0) * 3.0;
            push_eav(
                &mut jnl, &m, &e, 1,
                &[
                    ("orbit.r", AkkValue::Float(v)), ("orbit.theta", AkkValue::Float(v)),
                    ("orbit.phi", AkkValue::Float(v)), ("orbit.d3", AkkValue::Float(v)),
                    ("orbit.d4", AkkValue::Float(v)), ("orbit.d5", AkkValue::Float(v)),
                    ("orbit.d6", AkkValue::Float(v)),
                ],
            );
        }

        let base = tmp_base("shell_e7_first_anchor");
        materialize(&jnl, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let q = "WHO T.E\n\
                 WHERE E[orbit.r] = 1.0 AND E[orbit.theta] = 2.0 AND E[orbit.phi] = 3.0 \
                 AND E[orbit.d3] = 4.0 AND E[orbit.d4] = 5.0 AND E[orbit.d5] = 6.0 AND E[orbit.d6] = 7.0\n\
                 ANCHOR E7_FIRST";
        let full = heptascript::execute(&parse_query(q).unwrap(), &jnl);
        let via_crn = crn.query(q).unwrap();

        assert_eq!(full.matched.len(), 1);
        assert_eq!(via_crn.matched.len(), 1, "must agree with a full journal scan");
        assert_eq!(*full.matched[0].entity.bytes(), *via_crn.matched[0].entity.bytes());
    }
}
