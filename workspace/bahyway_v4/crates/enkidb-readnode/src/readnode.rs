//! ReadNode — serves HeptaScript queries against Data Files materialized
//! by [`crate::materialize::materialize`], per ADR-012's Data Files Law:
//! a Read Node holds no journal. `ReadNode::open` is O(1) (two file opens
//! + two stat calls, exactly like `enkidb_datafile::DataFileReader::open`);
//!
//! `ReadNode::query` touches only the entities a selective query actually
//! matches, never the whole dataset.
//!
//! Scope: a Read Node answers exactly the queries `heptascript::indexed`
//! can already accelerate in memory — current state (no WHEN, or `WHEN AT
//! EPOCH NOW`), first WHERE condition an exact equality, no `OR`. Anything
//! else (historical WHEN, non-equality or OR'd first condition, no WHERE
//! at all) is out of scope for a point-lookup-shaped disk index and must
//! go to the Write Node's full Journal — [`ReadNodeError::RequiresWriteNode`]
//! says so explicitly rather than silently returning a wrong or partial
//! answer.

use std::io;
use std::path::Path;

use akkvalue::AkkValue;
use enkidb_datafile::DataFileReader;
use enkidb_indexes::prelude::EavExactIndex;
use enkidb_kaki::{IdentityKaki, Kaki};
use heptascript::{
    execute_over_histories, parse_query, Combinator, ConditionTest, EpochRef, HeptaQuery,
    HeptaValue, HowMuchClause, Op, ParseError, QueryResult, QueryVerb, WhenClause, WhereCondition,
};

use crate::materialize::{decode_history, decode_posting_list, pack_eav_key};

#[derive(Debug)]
pub enum ReadNodeError {
    Parse(ParseError),
    Io(io::Error),
    /// The query isn't a current-state, single-equality lookup a Data File
    /// index can serve — it needs the Write Node's full Journal instead.
    RequiresWriteNode(&'static str),
    /// A posting or history record was found but couldn't be decoded —
    /// on-disk corruption or a materializer/reader version mismatch.
    CorruptRecord,
}

impl From<io::Error> for ReadNodeError {
    fn from(e: io::Error) -> Self {
        ReadNodeError::Io(e)
    }
}

pub struct ReadNode {
    entities: DataFileReader,
    eav_index: DataFileReader,
}

impl ReadNode {
    /// Open a Read Node over Data Files written by [`crate::materialize::materialize`].
    /// O(1): opens the four underlying files and reads two index lengths —
    /// never loads the dataset, never replays a journal.
    pub fn open(
        entities_base: impl AsRef<Path>,
        eav_index_base: impl AsRef<Path>,
    ) -> io::Result<Self> {
        Ok(ReadNode {
            entities: DataFileReader::open(entities_base)?,
            eav_index: DataFileReader::open(eav_index_base)?,
        })
    }

    /// Number of entities this Read Node has materialized.
    pub fn entity_count(&self) -> u64 {
        self.entities.record_count()
    }

    /// Parse and execute a HeptaScript W5H2 query. Only current-state,
    /// single-exact-equality queries are served (see module docs); anything
    /// else returns `RequiresWriteNode` rather than a wrong answer.
    pub fn query(&mut self, source: &str) -> Result<QueryResult, ReadNodeError> {
        let q = parse_query(source).map_err(ReadNodeError::Parse)?;
        self.query_parsed(&q)
    }

    fn query_parsed(&mut self, q: &HeptaQuery) -> Result<QueryResult, ReadNodeError> {
        if !when_is_current(&q.when) {
            return Err(ReadNodeError::RequiresWriteNode(
                "WHEN pins a specific past epoch; the Read Node only holds current state",
            ));
        }
        let (attr, val) =
            indexable_condition(&q.r#where).ok_or(ReadNodeError::RequiresWriteNode(
                "WHERE clause has no leading exact-equality condition (or uses OR) to index on",
            ))?;

        let attr_hash = bahyway_crc::crc16(attr.as_bytes()) as u32;
        let encoded = akkvalue::codec::encode(&hepta_value_to_akk(val));
        let vfp = EavExactIndex::val_fingerprint(&encoded);
        let key = pack_eav_key(attr_hash, vfp);

        let posting = self.eav_index.get_raw(&key)?;
        let mut kakis: Vec<[u8; 16]> = posting.map(|b| decode_posting_list(&b)).unwrap_or_default();

        // `execute_over_histories` already early-exits its own scan once
        // HOW_MUCH LIMIT N is satisfied (see its own doc comment) — but
        // that only helps once it's actually running. The real cost here
        // is upstream of it: fetching every matched KAKI's full history
        // blob from disk (an O(log n) binary-search read *each*, paid
        // once per candidate) before `execute_over_histories` ever gets a
        // chance to truncate. For a query matching a large fraction of
        // the corpus, that upstream fetch loop dominates — measured at
        // ~2s for 200K matches out of 1M particles, ~26s for 2M matches
        // out of 10M (a real, confirmed bottleneck, not a hypothetical
        // one). Truncate the KAKI list itself before the fetch loop runs
        // whenever it's provably safe: the index's posting list for this
        // (attr, value) pair already *is* the complete, exact answer set
        // for `WHERE attr = value` alone — so when that is the query's
        // *entire* filter (no more WHERE conditions, no WHY, no EMIT's
        // implicit BIRTH condition, no HOW sort requiring the full
        // candidate set to rank correctly), any N of those KAKIs are N
        // valid, complete matches. Any other query shape leaves `kakis`
        // untouched and keeps today's already-correct, already-tested
        // full-fetch behavior.
        if let Some(limit) = safe_early_limit(q) {
            kakis.truncate(limit);
        }

        // Each matched candidate's history, reconstructed straight from
        // this Read Node's own Data Files — no `Journal` involved at all.
        // Evaluating via `execute_over_histories` (the same core
        // `execute_over` delegates to) guarantees this path can never
        // disagree with a full Write-Node scan, without inheriting
        // `Journal::read_particle_history`'s cost profile, which is tuned
        // for the Write Node's occasional live lookup, not bulk Read Node
        // query serving.
        let mut histories = Vec::with_capacity(kakis.len());
        for kaki_bytes in &kakis {
            let identity = IdentityKaki::try_from_kaki(
                Kaki::from_bytes(*kaki_bytes).map_err(|_| ReadNodeError::CorruptRecord)?,
            )
            .map_err(|_| ReadNodeError::CorruptRecord)?;

            let blob = self
                .entities
                .get_raw(kaki_bytes)?
                .ok_or(ReadNodeError::CorruptRecord)?;
            let history = decode_history(&identity, &blob).ok_or(ReadNodeError::CorruptRecord)?;
            histories.push((identity, history));
        }

        Ok(execute_over_histories(q, &histories))
    }
}

/// `Some(n)` exactly when the indexed posting list *is* the query's
/// complete, correct answer set (see the call site's comment for why),
/// so pre-truncating it to `n` entries before the disk-fetch loop is
/// safe. `None` for every other query shape — including `Top { .. }`,
/// which needs the full candidate set to rank correctly, unlike a plain
/// `Limit`, which makes no ordering promise.
fn safe_early_limit(q: &HeptaQuery) -> Option<usize> {
    let limit = match &q.how_much {
        Some(HowMuchClause::Limit(n)) => *n,
        _ => return None,
    };
    if q.how.is_some() || !q.why.is_empty() || q.r#where.len() != 1 {
        return None;
    }
    // An ORBITAL clause is a real filter now (`heptascript::engine::
    // evaluate_capped` applies it; previously it was silently a no-op
    // everywhere `execute`/`execute_over_histories` actually ran, so this
    // guard wasn't needed). Truncating the posting list to `limit` before
    // `execute_over_histories` ever gets to apply that filter could
    // silently drop true in-range matches past the first `limit` entries
    // in on-disk posting order -- same undercount class the WHY/HOW/
    // second-WHERE guards above already exist to prevent.
    if q.orbital.is_some() {
        return None;
    }
    if q.verb != QueryVerb::Orbit {
        // Emit ANDs in an implicit `hist.event = "BIRTH"` condition
        // (see `operations::Operation`'s own doc comment); every other
        // non-Orbit verb still runs the identical Orbit candidate
        // resolution underneath, so this check only actually needs to
        // exclude Emit, but spelling it as "must be Orbit" keeps the
        // safety proof exact rather than enumerating verbs as they're
        // added.
        return None;
    }
    // A MEASURE/GRAVITY aggregate needs the complete matched set, not
    // just the first `limit` entities -- same guard `enkidb_readnode
    // ::cached::safe_early_limit` carries for the identical reason.
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
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_journal::Journal;
    use enkidb_kaki::{mint::KakiMinter, EventKaki, KakiRole};
    use std::fs;
    use std::path::PathBuf;

    fn tmp_base(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkidb_readnode_readnode_{name}"));
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
            .map(|(name, val)| {
                EavTriple::new(
                    bahyway_crc::crc16(name.as_bytes()) as u32,
                    codec::encode(val),
                )
            })
            .collect();
        jnl.append(JournalEntry::new(ek, target.clone(), epoch, eav))
            .unwrap();
    }

    /// The core promise: for a query the Read Node can serve, its answer
    /// must be identical to `heptascript::execute`'s full scan of the live
    /// journal — same invariant `heptascript::indexed` holds for its
    /// in-memory index shortcut.
    #[test]
    fn readnode_matches_full_scan_on_exact_eq() {
        let tribe = TribeId::from_u16(0x0301);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let najaf: Vec<_> = (0..5)
            .map(|_| IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap())
            .collect();
        for e in &najaf {
            push_eav(
                &mut jnl,
                &m,
                e,
                1,
                &[("city.name", AkkValue::Text("Najaf".into()))],
            );
        }
        for _ in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("city.name", AkkValue::Text("Baghdad".into()))],
            );
        }

        let base = tmp_base("matches_full_scan");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let full = heptascript::execute(
            &parse_query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap(),
            &jnl,
        );
        let via_rn = rn.query("WHO T.E\nWHERE E[city.name] = \"Najaf\"").unwrap();

        assert_eq!(via_rn.matched.len(), 5);
        assert_eq!(full.matched.len(), via_rn.matched.len());

        let mut full_bytes: Vec<_> = full.matched.iter().map(|m| *m.entity.bytes()).collect();
        let mut rn_bytes: Vec<_> = via_rn.matched.iter().map(|m| *m.entity.bytes()).collect();
        full_bytes.sort();
        rn_bytes.sort();
        assert_eq!(full_bytes, rn_bytes);
    }

    #[test]
    fn readnode_unknown_value_returns_zero_matches() {
        let tribe = TribeId::from_u16(0x0302);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        push_eav(
            &mut jnl,
            &m,
            &e,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );

        let base = tmp_base("unknown_value");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn
            .query("WHO T.E\nWHERE E[city.name] = \"Karbala\"")
            .unwrap();
        assert!(res.matched.is_empty());
    }

    #[test]
    fn how_much_limit_truncates_the_fetch_for_a_single_indexed_condition() {
        let tribe = TribeId::from_u16(0x0306);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..500 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("city.name", AkkValue::Text("Najaf".into()))],
            );
        }

        let base = tmp_base("limit_truncates_fetch");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn
            .query("WHO T.E\nWHERE E[city.name] = \"Najaf\"\nHOW_MUCH LIMIT 10")
            .unwrap();
        assert_eq!(
            res.matched.len(),
            10,
            "500 true matches exist; LIMIT 10 must still return exactly 10"
        );
    }

    #[test]
    fn how_much_limit_larger_than_true_match_count_returns_every_match() {
        let tribe = TribeId::from_u16(0x0307);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..5 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("city.name", AkkValue::Text("Najaf".into()))],
            );
        }

        let base = tmp_base("limit_larger_than_matches");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn
            .query("WHO T.E\nWHERE E[city.name] = \"Najaf\"\nHOW_MUCH LIMIT 100")
            .unwrap();
        assert_eq!(
            res.matched.len(),
            5,
            "only 5 true matches exist; truncate() to 100 must be a no-op"
        );
    }

    #[test]
    fn how_much_limit_does_not_short_circuit_with_a_how_sort() {
        // A HOW BY sort needs the *entire* candidate set to rank
        // correctly -- taking the posting list's first N in on-disk
        // order (undefined w.r.t. any attribute) would silently return
        // the wrong top-N. `safe_early_limit` must decline here, and the
        // real top-N (by descending rank) must still come back correct.
        let tribe = TribeId::from_u16(0x0308);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for rank in 0..50 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[
                    ("city.name", AkkValue::Text("Najaf".into())),
                    ("rank", AkkValue::Int(rank)),
                ],
            );
        }

        let base = tmp_base("limit_with_how_sort");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn
            .query(
                "WHO T.E\nWHAT E[rank]\nWHERE E[city.name] = \"Najaf\"\nHOW BY E[rank] DESC\nHOW_MUCH LIMIT 3",
            )
            .unwrap();
        assert_eq!(res.matched.len(), 3);
        let ranks: Vec<i64> = res
            .matched
            .iter()
            .map(|m| {
                match m
                    .projected
                    .iter()
                    .find(|(k, _)| k == "rank")
                    .map(|(_, v)| v)
                {
                    Some(AkkValue::Int(n)) => *n,
                    other => panic!("expected rank to be projected as Int, got {other:?}"),
                }
            })
            .collect();
        assert_eq!(
            ranks,
            vec![49, 48, 47],
            "must be the true top-3 by rank, not the first 3 on disk"
        );
    }

    #[test]
    fn how_much_limit_does_not_short_circuit_with_a_second_where_condition() {
        // A second WHERE condition means the indexed posting list is only
        // a *candidate* set, not the final answer -- truncating it before
        // the second condition is applied could return fewer than the
        // true LIMIT-worth of matches. Half of the "Najaf" entities also
        // carry age > 30; LIMIT 3 must still find 3 real matches, not
        // stop after checking only the first `limit` untested candidates.
        let tribe = TribeId::from_u16(0x0309);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for i in 0..20 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            let age = if i % 2 == 0 { 20 } else { 40 };
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[
                    ("city.name", AkkValue::Text("Najaf".into())),
                    ("age", AkkValue::Int(age)),
                ],
            );
        }

        let base = tmp_base("limit_with_second_where");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn
            .query("WHO T.E\nWHERE E[city.name] = \"Najaf\" AND E[age] > 30\nHOW_MUCH LIMIT 3")
            .unwrap();
        assert_eq!(
            res.matched.len(),
            3,
            "10 true age>30 matches exist; LIMIT 3 must find 3, not undercount"
        );
    }

    #[test]
    fn readnode_rejects_or_condition() {
        let tribe = TribeId::from_u16(0x0303);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        push_eav(
            &mut jnl,
            &m,
            &e,
            1,
            &[("city.name", AkkValue::Text("Najaf".into()))],
        );

        let base = tmp_base("rejects_or");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn.query("WHO T.E\nWHERE E[city.name] = \"Najaf\" OR E[city.name] = \"Karbala\"");
        assert!(matches!(res, Err(ReadNodeError::RequiresWriteNode(_))));
    }

    #[test]
    fn readnode_rejects_historical_when() {
        let tribe = TribeId::from_u16(0x0304);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        push_eav(
            &mut jnl,
            &m,
            &e,
            5,
            &[("status", AkkValue::Text("active".into()))],
        );

        let base = tmp_base("rejects_when");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn.query("WHO T.E\nWHERE E[status] = \"active\"\nWHEN AT EPOCH 3");
        assert!(matches!(res, Err(ReadNodeError::RequiresWriteNode(_))));
    }

    #[test]
    fn open_is_cheap_after_materialize_at_moderate_scale() {
        let tribe = TribeId::from_u16(0x0305);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let target = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        push_eav(
            &mut jnl,
            &m,
            &target,
            1,
            &[("station", AkkValue::Text("cleansing".into()))],
        );
        for _ in 0..999 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("station", AkkValue::Text("steward".into()))],
            );
        }

        let base = tmp_base("open_cheap");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        assert_eq!(rn.entity_count(), 1000);

        let res = rn
            .query("WHO T.E\nWHERE E[station] = \"cleansing\"")
            .unwrap();
        assert_eq!(res.matched.len(), 1);
        assert_eq!(*res.matched[0].entity.bytes(), *target.bytes());
    }

    #[test]
    fn measure_dense_ignores_how_much_limit() {
        // Same early-exit bug class the `how_much_limit_does_not_short_
        // circuit_*` tests above guard against, one layer up: `safe_early_
        // limit` must decline once a MEASURE clause is present, or DENSE
        // would silently report the LIMIT-truncated posting-list size
        // instead of the true match count.
        let tribe = TribeId::from_u16(0x0310);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        for _ in 0..40 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("city.name", AkkValue::Text("Najaf".into()))],
            );
        }

        let base = tmp_base("measure_ignores_limit");
        materialize(
            &jnl,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let res = rn
            .query("WHO T.E\nWHERE E[city.name] = \"Najaf\"\nHOW_MUCH LIMIT 5\nMEASURE DENSE")
            .unwrap();
        assert_eq!(res.matched.len(), 5, "returned rows still honor LIMIT");
        assert_eq!(
            res.measured,
            Some(heptascript::MeasureValue::Dense(40)),
            "the aggregate must reflect all 40 true matches, not the LIMIT-truncated posting list"
        );
    }
}
