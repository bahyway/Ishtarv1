//! Materialize a Journal's current EAV state into two Data Files:
//! - entity history store: entity KAKI -> that entity's raw journal history
//!   (enough to reconstruct real `JournalEntry` objects for evaluation)
//! - EAV posting index: (attr_hash, value_fingerprint) -> entity KAKI list,
//!   for pruning an exact-equality WHERE clause without touching history.
//!
//! This pays the same O(n) journal-read cost `heptascript::build_indexes`
//! already pays once — the difference is the result is *persisted*, so a
//! Read Node opened later (`ReadNode::open`) never re-pays it: opening is
//! O(1), and a query only reads the entities it actually needs.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use akkvalue::{codec, AkkValue};
use bahyway_core::TribeId;
use enkidb_datafile::DataFileWriter;
use enkidb_indexes::prelude::EavExactIndex;
use enkidb_journal::entry::{EavTriple, JournalEntry};
use enkidb_journal::Journal;
use enkidb_kaki::mint::KakiMinter;
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki, KakiRole};

/// Stats from one materialization pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct MaterializeStats {
    pub entities: usize,
    pub distinct_eav_keys: usize,
    /// Number of distinct tribes seen -- one `tribe_summary` entity is
    /// materialized per tribe into the SEPARATE tribe-summary Data Files
    /// (see `tribe_summary_paths`'s doc comment for why these are kept
    /// out of the main entity/EAV files).
    pub tribes: usize,
}

/// Sentinel `meta.kind` value every tribe-summary entity carries.
pub const TRIBE_SUMMARY_KIND: &str = "tribe_summary";

/// Fixed uuid_hash used for every tribe-summary entity this pass mints.
/// Harmless to reuse across tribes: `tribe_id` (bytes[4..6]) already makes
/// each tribe's summary KAKI distinct from every other tribe's, and the
/// summary entity is discovered by querying its real attrs (`tribe.id`,
/// `meta.kind`), never by assuming a fixed KAKI address -- re-materializing
/// mints a fresh KAKI for the same tribe every time (birth timestamp
/// differs), which is fine since nothing depends on address stability here.
const TRIBE_SUMMARY_UUID_HASH: u32 = 0x5452_4942; // ASCII "TRIB"

/// Derives the sibling Data File paths a tribe-summary corpus is written
/// to, from the SAME base paths the real particle corpus uses -- e.g.
/// `.../entities` -> `.../entities_tribes`, `.../eav` -> `.../eav_tribes`.
/// Purely additive (no caller of `materialize()` needs new parameters),
/// and no caller of `materialize()` needs to change at all -- deliberately
/// kept as a SEPARATE Data File pair, not appended into the real
/// `entities`/`eav_index` files: an earlier version of this feature did
/// exactly that and broke real invariants two different callers depend
/// on -- `CachedReadNode::entity_count()` no longer matched the real
/// ingested particle count, and a plain WHERE-less `WHO T.E` ("give me
/// every particle") silently returned tribe-summary rows mixed in with
/// real ones (caught by this crate's own
/// `cached_serves_queries_readnode_would_reject` test expecting exactly
/// 20 particles back and getting 21). Keeping tribe summaries in their
/// own small corpus avoids both: the real entity space is untouched, and
/// tribe summaries are still real, ordinary HeptaScript-queryable
/// entities -- just via their own `CachedReadNode`/`ReadNode::open`
/// pointed at `tribe_summary_paths()`, not the main one.
pub fn tribe_summary_paths(
    entities_base: impl AsRef<Path>,
    eav_index_base: impl AsRef<Path>,
) -> (PathBuf, PathBuf) {
    (
        sibling_with_suffix(entities_base.as_ref(), "_tribes"),
        sibling_with_suffix(eav_index_base.as_ref(), "_tribes"),
    )
}

fn sibling_with_suffix(base: &Path, suffix: &str) -> PathBuf {
    let file_name = base.file_name().and_then(|n| n.to_str()).unwrap_or("data");
    base.with_file_name(format!("{file_name}{suffix}"))
}

/// Opens the tribe-summary corpus a real `materialize()` pass wrote
/// alongside the given real particle corpus, as an ordinary
/// `CachedReadNode` -- real HeptaScript queries against it work exactly
/// like queries against any other `CachedReadNode`. Returns `None`
/// (never an error) when the sibling files don't exist yet -- e.g. data
/// materialized before this feature existed, or before the very first
/// materialize pass has run at all -- so a caller (a *-read-server's
/// reload loop) can keep serving the real particle corpus even when the
/// tribe-summary corpus isn't available yet.
pub fn open_tribe_summaries(
    entities_base: impl AsRef<Path>,
    eav_index_base: impl AsRef<Path>,
) -> Option<crate::cached::CachedReadNode> {
    let (tribes_entities, tribes_eav) = tribe_summary_paths(entities_base, eav_index_base);
    crate::cached::CachedReadNode::open(tribes_entities, tribes_eav).ok()
}

/// Materialize every particle currently in `journal` into the Data Files
/// rooted at `entities_base` and `eav_index_base`. One O(n) pass over the
/// journal — the same cost class as `heptascript::build_indexes` — paid
/// once here, not on every Read Node open.
///
/// Groups the journal's entries by target in a single local pass
/// (`journal.all_entries()`, not `read_particle_history()` called once per
/// particle) — the grouping lives only in this function's stack and is
/// discarded when it returns. The Journal itself never carries a
/// persistent secondary index for this: that's Read Node territory
/// (these Data Files), not the Write Node's.
pub fn materialize(
    journal: &Journal,
    entities_base: impl AsRef<Path>,
    eav_index_base: impl AsRef<Path>,
) -> io::Result<MaterializeStats> {
    let mut by_target: HashMap<[u8; 16], Vec<&JournalEntry>> = HashMap::new();
    for entry in journal.all_entries() {
        by_target
            .entry(*entry.target_kaki.bytes())
            .or_default()
            .push(entry);
    }

    // `materialize()` recomputes the FULL current state from `journal` on
    // every call (not an incremental delta) -- but `DataFileWriter::open`
    // always appends. Without resetting first, a second (or Nth) call
    // against the same on-disk files -- e.g. a later `FLUSH` against a
    // Podman volume that survived a container rebuild -- keeps every
    // record any earlier pass ever wrote, merged into the same `.idx` via
    // `compact_index`'s `read_all_records(&self.idx_path)`. A reader that
    // then iterates the whole index (`iter_all_raw`) hits stale records
    // from an earlier, possibly incompatible encoding and fails to decode
    // them -- surfacing as `CachedReadNodeError::CorruptRecord` on the Read
    // Node side, even though this pass's own `by_target.len()` is correct.
    reset_data_file(entities_base.as_ref())?;
    reset_data_file(eav_index_base.as_ref())?;

    let mut entities_writer = DataFileWriter::open(entities_base.as_ref())?;
    let mut postings: HashMap<[u8; 16], Vec<u8>> = HashMap::new();

    // Real per-tribe particle tally -- every target KAKI already self-
    // describes its tribe in bytes[4..6] (see enkidb-kaki's Kaki::tribe_id()),
    // so this costs nothing extra: one HashMap bump per entity, in the same
    // pass that's already iterating `by_target`.
    let mut tribe_counts: HashMap<u16, usize> = HashMap::new();

    for (target_bytes, history) in &by_target {
        let blob = encode_history(history);
        entities_writer.append_raw(*target_bytes, &blob)?;

        for (attr_hash, value_bytes) in last_write_wins(history) {
            let vfp = EavExactIndex::val_fingerprint(&value_bytes);
            let key = pack_eav_key(attr_hash, vfp);
            postings
                .entry(key)
                .or_default()
                .extend_from_slice(target_bytes);
        }

        let tribe_id = u16::from_be_bytes([target_bytes[4], target_bytes[5]]);
        *tribe_counts.entry(tribe_id).or_insert(0) += 1;
    }
    entities_writer.sync()?;
    entities_writer.compact_index()?;

    let mut eav_writer = DataFileWriter::open(eav_index_base.as_ref())?;
    let distinct_eav_keys = postings.len();
    for (key, kaki_list) in postings {
        eav_writer.append_raw(key, &kaki_list)?;
    }
    eav_writer.sync()?;
    eav_writer.compact_index()?;

    let tribes = tribe_counts.len();
    materialize_tribe_summaries(&tribe_counts, entities_base, eav_index_base)?;

    Ok(MaterializeStats {
        entities: by_target.len(),
        distinct_eav_keys,
        tribes,
    })
}

/// Registers the real total particle count per tribe as real, ordinary
/// queryable entities -- one per distinct tribe, attrs `tribe.id` (Int)
/// and `tribe.particle_count` (Int), tagged `meta.kind = "tribe_summary"`
/// so both "give me tribe X's total" (indexed exact match on tribe.id)
/// and "list every tribe's total" (indexed exact match on meta.kind) are
/// cheap, ordinary HeptaScript WHERE queries -- no new verb, no new wire
/// protocol, no live aggregation at query time (which the anti-SQL law
/// forbids); the count is computed once, at materialize time.
///
/// Written into a SEPARATE Data File pair (`tribe_summary_paths`), not
/// appended into the real particle `entities`/`eav_index_base` files --
/// see that function's own doc comment for the real regression this
/// avoided (a plain `WHO T.E` full scan silently returning tribe-summary
/// rows mixed in with real particles).
fn materialize_tribe_summaries(
    tribe_counts: &HashMap<u16, usize>,
    entities_base: impl AsRef<Path>,
    eav_index_base: impl AsRef<Path>,
) -> io::Result<()> {
    let (tribes_entities_path, tribes_eav_path) =
        tribe_summary_paths(entities_base, eav_index_base);
    // Same full-recompute-not-delta reasoning as the real particle corpus
    // above -- see that call site's comment.
    reset_data_file(&tribes_entities_path)?;
    reset_data_file(&tribes_eav_path)?;
    let mut entities_writer = DataFileWriter::open(&tribes_entities_path)?;
    let mut postings: HashMap<[u8; 16], Vec<u8>> = HashMap::new();

    for (tribe_id, count) in tribe_counts {
        let minter = KakiMinter::new(TribeId::from_u16(*tribe_id));
        let summary_identity = IdentityKaki::try_from_kaki(
            minter.mint_identity(TRIBE_SUMMARY_UUID_HASH, KakiRole::Zikru),
        )
        .expect("mint_identity always produces a valid IdentityKaki");
        let summary_event =
            EventKaki::try_from_kaki(minter.mint_event(TRIBE_SUMMARY_UUID_HASH, KakiRole::Zikru))
                .expect("mint_event always produces a valid IdentityKaki");

        let eav = vec![
            EavTriple::new(
                bahyway_crc::crc16("tribe.id".as_bytes()) as u32,
                codec::encode(&AkkValue::Int(*tribe_id as i64)),
            ),
            EavTriple::new(
                bahyway_crc::crc16("tribe.particle_count".as_bytes()) as u32,
                codec::encode(&AkkValue::Int(*count as i64)),
            ),
            EavTriple::new(
                bahyway_crc::crc16("meta.kind".as_bytes()) as u32,
                codec::encode(&AkkValue::Text(TRIBE_SUMMARY_KIND.into())),
            ),
        ];
        let target_bytes = *summary_identity.bytes();
        let entry = JournalEntry::new(summary_event, summary_identity, 1, eav.clone());

        let blob = encode_history(&[&entry]);
        entities_writer.append_raw(target_bytes, &blob)?;

        for triple in &eav {
            let vfp = EavExactIndex::val_fingerprint(&triple.value);
            let key = pack_eav_key(triple.attr_hash, vfp);
            postings
                .entry(key)
                .or_default()
                .extend_from_slice(&target_bytes);
        }
    }
    entities_writer.sync()?;
    entities_writer.compact_index()?;

    let mut eav_writer = DataFileWriter::open(&tribes_eav_path)?;
    for (key, kaki_list) in postings {
        eav_writer.append_raw(key, &kaki_list)?;
    }
    eav_writer.sync()?;
    eav_writer.compact_index()?;

    Ok(())
}

/// Removes a Data File's three on-disk siblings (`{base}.data`,
/// `{base}.idx`, `{base}.idx.staging`) if present, so a full materialize
/// pass starts from a clean slate instead of appending onto whatever an
/// earlier pass left behind. Missing files are not an error (first-ever
/// materialize against a fresh volume).
fn reset_data_file(base: &Path) -> io::Result<()> {
    for ext in ["data", "idx", "idx.staging"] {
        match std::fs::remove_file(base.with_extension(ext)) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// (attr_hash, value_fingerprint) packed into a 16-byte key so a posting
/// list can share `enkidb-datafile`'s KAKI-shaped index instead of a
/// second key type.
pub fn pack_eav_key(attr_hash: u32, val_fp: u32) -> [u8; 16] {
    let mut key = [0u8; 16];
    key[0..4].copy_from_slice(&attr_hash.to_be_bytes());
    key[4..8].copy_from_slice(&val_fp.to_be_bytes());
    key
}

/// Decode a posting-list record (concatenated 16-byte entity KAKIs) back
/// into individual KAKI bytes.
pub fn decode_posting_list(bytes: &[u8]) -> Vec<[u8; 16]> {
    bytes
        .chunks_exact(16)
        .map(|c| {
            c.try_into()
                .expect("chunks_exact(16) always yields 16 bytes")
        })
        .collect()
}

/// Last-write-wins EAV snapshot (raw bytes, sorted by epoch) — mirrors
/// `heptascript::engine::raw_eav_snapshot`'s documented rule exactly, so a
/// posting list built here agrees with what a live full scan would compute.
fn last_write_wins(history: &[&JournalEntry]) -> Vec<(u32, Vec<u8>)> {
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

/// Encode one entity's full raw journal history (event kaki + epoch + EAV
/// triples per entry) so it can be reconstructed into real `JournalEntry`
/// objects later and handed to `heptascript::execute_over` unchanged.
///
/// `Journal::read_particle_history` returns entries in HashMap partition
/// iteration order, not epoch order (its own callers each sort themselves
/// before use) — sorting here makes the persisted blob deterministic
/// regardless of what order the caller's `history` slice arrived in.
fn encode_history(history: &[&JournalEntry]) -> Vec<u8> {
    let mut sorted: Vec<&JournalEntry> = history.to_vec();
    sorted.sort_by_key(|e| e.epoch);

    let mut buf = Vec::new();
    buf.extend_from_slice(&(sorted.len() as u32).to_be_bytes());
    for entry in &sorted {
        buf.extend_from_slice(entry.event_kaki.bytes());
        buf.extend_from_slice(&entry.epoch.to_be_bytes());
        buf.extend_from_slice(&(entry.eav.len() as u16).to_be_bytes());
        for triple in &entry.eav {
            buf.extend_from_slice(&triple.attr_hash.to_be_bytes());
            // u32, not u16: `triple.value` is already a self-describing
            // `akkvalue::codec`-encoded blob (real incident, 2026-08-21: a
            // full document's "full text" EAV value routinely exceeds
            // 64KB in this corpus). A u16 here silently truncates the
            // length prefix while the full value bytes are still written,
            // desyncing every field decoded after it in this entity's
            // history -- surfacing as `CachedReadNodeError::CorruptRecord`
            // on the Read Node for that entity, and only that entity
            // (each entity's history blob has its own outer offset+len
            // in the Data File's index, so this does not cascade to
            // other entities).
            buf.extend_from_slice(&(triple.value.len() as u32).to_be_bytes());
            buf.extend_from_slice(&triple.value);
        }
    }
    buf
}

/// Decode a blob written by `encode_history` back into real `JournalEntry`
/// objects for `target`. Returns `None` on malformed/truncated bytes.
pub fn decode_history(target: &IdentityKaki, bytes: &[u8]) -> Option<Vec<JournalEntry>> {
    let mut pos = 0usize;
    let count = u32::from_be_bytes(bytes.get(pos..pos + 4)?.try_into().ok()?) as usize;
    pos += 4;

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let ek_bytes: [u8; 16] = bytes.get(pos..pos + 16)?.try_into().ok()?;
        pos += 16;
        let event_kaki = EventKaki::try_from_kaki(Kaki::from_bytes(ek_bytes).ok()?).ok()?;

        let epoch = u32::from_be_bytes(bytes.get(pos..pos + 4)?.try_into().ok()?);
        pos += 4;

        let eav_count = u16::from_be_bytes(bytes.get(pos..pos + 2)?.try_into().ok()?) as usize;
        pos += 2;

        let mut eav = Vec::with_capacity(eav_count);
        for _ in 0..eav_count {
            let attr_hash = u32::from_be_bytes(bytes.get(pos..pos + 4)?.try_into().ok()?);
            pos += 4;
            let val_len = u32::from_be_bytes(bytes.get(pos..pos + 4)?.try_into().ok()?) as usize;
            pos += 4;
            let value = bytes.get(pos..pos + val_len)?.to_vec();
            pos += val_len;
            eav.push(EavTriple::new(attr_hash, value));
        }

        entries.push(JournalEntry::new(event_kaki, *target, epoch, eav));
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CachedReadNode;
    use akkvalue::{codec, AkkValue};
    use bahyway_core::TribeId;
    use enkidb_datafile::DataFileReader;
    use enkidb_kaki::{mint::KakiMinter, KakiRole};
    use std::fs;
    use std::path::PathBuf;

    fn tmp_base(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkidb_readnode_materialize_{name}"));
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

    #[test]
    fn history_round_trips_through_encode_decode() {
        let tribe = TribeId::from_u16(0x0201);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();

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
            2,
            &[("status", AkkValue::Text("archived".into()))],
        );

        let history = jnl.read_particle_history(&e);
        let blob = encode_history(&history);
        let decoded = decode_history(&e, &blob).expect("decode must succeed");

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].epoch, 1);
        assert_eq!(decoded[1].epoch, 2);
        assert_eq!(*decoded[0].target_kaki.bytes(), *e.bytes());
    }

    #[test]
    fn materialize_persists_entities_and_eav_postings() {
        let tribe = TribeId::from_u16(0x0202);
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);

        let najaf: Vec<_> = (0..3)
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
        for _ in 0..7 {
            let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m,
                &e,
                1,
                &[("city.name", AkkValue::Text("Baghdad".into()))],
            );
        }

        let base = tmp_base("basic");
        let entities_base = base.with_file_name("entities");
        let eav_base = base.with_file_name("eav");
        let stats = materialize(&jnl, &entities_base, &eav_base).unwrap();
        assert_eq!(stats.entities, 10);

        let attr_hash = bahyway_crc::crc16("city.name".as_bytes()) as u32;
        let vfp = EavExactIndex::val_fingerprint(&codec::encode(&AkkValue::Text("Najaf".into())));
        let key = pack_eav_key(attr_hash, vfp);

        let mut eav_reader = DataFileReader::open(&eav_base).unwrap();
        let posting = eav_reader
            .get_raw(&key)
            .unwrap()
            .expect("posting list must exist");
        let kakis = decode_posting_list(&posting);
        assert_eq!(kakis.len(), 3);

        let mut expected: Vec<[u8; 16]> = najaf.iter().map(|e| *e.bytes()).collect();
        expected.sort();
        let mut actual = kakis;
        actual.sort();
        assert_eq!(actual, expected);

        let mut entities_reader = DataFileReader::open(&entities_base).unwrap();
        for e in &najaf {
            let blob = entities_reader
                .get_raw(e.bytes())
                .unwrap()
                .expect("entity history must exist");
            let history = decode_history(e, &blob).unwrap();
            assert_eq!(history.len(), 1);
        }
    }

    // Real test for the per-tribe particle-count feature (the Architect's
    // own "PU" idea, 2026-07-21): two tribes, different real particle
    // counts, verify the materialized tribe_summary entity for each tribe
    // carries the REAL count -- not by trusting the code, by reading the
    // Data Files back and decoding exactly what a real query would see.
    #[test]
    fn materialize_registers_real_per_tribe_particle_count() {
        let tribe_a = TribeId::from_u16(0x1001);
        let tribe_b = TribeId::from_u16(0x1002);
        let mut jnl = Journal::new(64);

        let m_a = KakiMinter::new(tribe_a);
        for _ in 0..5 {
            let e = IdentityKaki::try_from_kaki(m_a.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m_a,
                &e,
                1,
                &[("kind", AkkValue::Text("alpha".into()))],
            );
        }

        let m_b = KakiMinter::new(tribe_b);
        for _ in 0..12 {
            let e = IdentityKaki::try_from_kaki(m_b.identity(KakiRole::Zikru)).unwrap();
            push_eav(
                &mut jnl,
                &m_b,
                &e,
                1,
                &[("kind", AkkValue::Text("beta".into()))],
            );
        }

        let base = tmp_base("tribe_pu");
        let entities_base = base.with_file_name("entities");
        let eav_base = base.with_file_name("eav");
        let stats = materialize(&jnl, &entities_base, &eav_base).unwrap();

        // 17 real particles, unaffected by the 2 tribe_summary entities --
        // those live in a separate corpus (see tribe_summary_paths's doc
        // comment for why: they must NOT count toward the real particle
        // total or leak into a plain "WHO T.E" full scan of real data).
        assert_eq!(stats.entities, 17);
        assert_eq!(stats.tribes, 2);

        let real_crn = CachedReadNode::open(&entities_base, &eav_base).unwrap();
        assert_eq!(
            real_crn.entity_count(),
            17,
            "tribe summaries must not appear in the real particle corpus"
        );
        let all_real = real_crn.query("WHO T.E").unwrap();
        assert_eq!(
            all_real.matched.len(),
            17,
            "a plain full scan must never see tribe_summary rows"
        );

        let (tribes_entities_base, tribes_eav_base) =
            tribe_summary_paths(&entities_base, &eav_base);
        let mut eav_reader = DataFileReader::open(&tribes_eav_base).unwrap();
        let mut entities_reader = DataFileReader::open(&tribes_entities_base).unwrap();

        for (tribe_id, expected_count) in [(tribe_a.as_u16(), 5i64), (tribe_b.as_u16(), 12i64)] {
            let attr_hash = bahyway_crc::crc16("tribe.id".as_bytes()) as u32;
            let vfp =
                EavExactIndex::val_fingerprint(&codec::encode(&AkkValue::Int(tribe_id as i64)));
            let key = pack_eav_key(attr_hash, vfp);
            let posting = eav_reader
                .get_raw(&key)
                .unwrap()
                .expect("tribe.id posting must exist");
            let kakis = decode_posting_list(&posting);
            assert_eq!(
                kakis.len(),
                1,
                "exactly one tribe_summary entity per tribe.id"
            );

            let summary_kaki =
                IdentityKaki::try_from_kaki(Kaki::from_bytes(kakis[0]).unwrap()).unwrap();
            let blob = entities_reader
                .get_raw(&kakis[0])
                .unwrap()
                .expect("summary entity must exist");
            let history = decode_history(&summary_kaki, &blob).unwrap();
            assert_eq!(history.len(), 1);

            let count_hash = bahyway_crc::crc16("tribe.particle_count".as_bytes()) as u32;
            let count_triple = history[0]
                .eav
                .iter()
                .find(|t| t.attr_hash == count_hash)
                .unwrap();
            let (decoded_count, _) = codec::decode(&count_triple.value, 0).unwrap();
            assert_eq!(decoded_count, AkkValue::Int(expected_count));

            let kind_hash = bahyway_crc::crc16("meta.kind".as_bytes()) as u32;
            let kind_triple = history[0]
                .eav
                .iter()
                .find(|t| t.attr_hash == kind_hash)
                .unwrap();
            let (decoded_kind, _) = codec::decode(&kind_triple.value, 0).unwrap();
            assert_eq!(decoded_kind, AkkValue::Text(TRIBE_SUMMARY_KIND.into()));
        }

        // Real HeptaScript query path, not just direct Data File reads --
        // exactly what a real Read Node opened against the tribe-summary
        // corpus would answer.
        let tribes_crn = CachedReadNode::open(&tribes_entities_base, &tribes_eav_base).unwrap();
        let result = tribes_crn
            .query(&format!(
                "WHO T.E\nWHAT E[tribe.particle_count]\nWHERE E[tribe.id] = {}",
                tribe_b.as_u16()
            ))
            .unwrap();
        assert_eq!(result.matched.len(), 1);
    }

    // Regression test for the real bare-metal incident (2026-08-21): a
    // Write Node's on-disk volume survives across container rebuilds, so
    // `materialize()` (via repeated real `FLUSH` commands) gets called
    // more than once against the SAME `entities_base`/`eav_index_base`
    // files over that volume's lifetime. Before this fix, `DataFileWriter`
    // always appended, so a second full pass left the first pass's records
    // sitting in the `.data`/`.idx` files right alongside the new ones --
    // and `CachedReadNode::open`'s `iter_all_raw` scan hit them too,
    // failing outright the moment any of them didn't decode cleanly
    // (`CachedReadNodeError::CorruptRecord`, surfaced by a real
    // `enkiddb-read-server` reload). A second `materialize()` pass over a
    // SHRUNK journal (entity that existed in pass 1 removed from what
    // pass 2 would produce) must leave the Data Files reflecting ONLY
    // pass 2's state -- not pass 1's leftovers layered underneath it.
    #[test]
    fn second_materialize_pass_does_not_leave_stale_records_from_the_first() {
        let tribe = TribeId::from_u16(0x2001);
        let base = tmp_base("repeated_flush");
        let entities_base = base.with_file_name("entities");
        let eav_base = base.with_file_name("eav");

        let mut jnl_a = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let gone = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let stays = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        push_eav(
            &mut jnl_a,
            &m,
            &gone,
            1,
            &[("city.name", AkkValue::Text("Uruk".into()))],
        );
        push_eav(
            &mut jnl_a,
            &m,
            &stays,
            1,
            &[("city.name", AkkValue::Text("Eridu".into()))],
        );

        let stats_a = materialize(&jnl_a, &entities_base, &eav_base).unwrap();
        assert_eq!(
            stats_a.entities, 2,
            "first FLUSH materializes both entities"
        );

        // Simulate the container-rebuild-preserves-the-volume scenario: a
        // second real FLUSH, against a journal that no longer contains
        // `gone` (e.g. a fresh Write Node process whose in-memory journal
        // was rebuilt from a smaller/different real corpus than whatever
        // produced the first pass on this same persistent volume).
        let mut jnl_b = Journal::new(64);
        push_eav(
            &mut jnl_b,
            &m,
            &stays,
            1,
            &[("city.name", AkkValue::Text("Eridu".into()))],
        );

        let stats_b = materialize(&jnl_b, &entities_base, &eav_base).unwrap();
        assert_eq!(
            stats_b.entities, 1,
            "second FLUSH materializes only what's in THIS pass's journal"
        );

        let crn = CachedReadNode::open(&entities_base, &eav_base).expect(
            "a real Read Node must be able to open Data Files written by a second materialize pass",
        );
        assert_eq!(
            crn.entity_count(),
            1,
            "the first pass's leftover record must not survive a second full pass"
        );

        let all = crn.query("WHO T.E").unwrap();
        assert_eq!(
            all.matched.len(),
            1,
            "a full scan must see only the current pass's real entities"
        );
    }

    // Regression test for the real bare-metal incident (2026-08-21, second
    // half): even after the append-vs-replace fix above, a real 527-document
    // ingestion still hit `CachedReadNodeError::CorruptRecord` on the Read
    // Node. Root cause: `encode_history` wrote each EAV value's length as a
    // `u16`, silently truncating for any value over 64KB while still
    // writing the FULL value bytes -- desyncing `decode_history`'s cursor
    // for every field after it. A real document's full-text EAV value
    // routinely exceeds 64KB. This entity's value is deliberately >64KB to
    // prove the u32 length prefix round-trips it correctly.
    #[test]
    fn history_value_over_64kb_round_trips_without_corrupting_the_record() {
        let tribe = TribeId::from_u16(0x2002);
        let base = tmp_base("oversized_value");
        let entities_base = base.with_file_name("entities");
        let eav_base = base.with_file_name("eav");

        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(tribe);
        let target = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let big_text = "x".repeat(70_000);
        push_eav(
            &mut jnl,
            &m,
            &target,
            1,
            &[("meta.full_text", AkkValue::Text(big_text.clone()))],
        );

        let stats = materialize(&jnl, &entities_base, &eav_base).unwrap();
        assert_eq!(stats.entities, 1);

        let crn = CachedReadNode::open(&entities_base, &eav_base)
            .expect("a >64KB EAV value must not corrupt the record it belongs to");
        assert_eq!(crn.entity_count(), 1);

        let all = crn.query("WHO T.E").unwrap();
        assert_eq!(
            all.matched.len(),
            1,
            "the oversized-value entity must still be found, not silently lost"
        );
    }
}
