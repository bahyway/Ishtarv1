//! Journal — in-memory append-only event log with partition-aware routing.
//!
//! The Journal accumulates JournalEntries per PartitionKey.
//! Persistence to cold storage is handled by enkidb-storage (AppendWriter).
//! This module is the in-process append buffer and sequential read interface.
//!
//! Deliberately un-indexed: this is the Write Node's structure. Indexing
//! belongs to the Read Node's Data Files (`enkidb-datafile` /
//! `enkidb-readnode`), which are rebuilt/reorganized as a batch process and
//! paged in memory to serve queries. The Journal's only job is a lean,
//! sequential, append-only log — adding a permanent secondary index here
//! (maintained on every `append`) would push Read-Node-shaped cost onto
//! the Write Node's hot path, which is exactly the coupling the Write
//! Node / Read Node split exists to avoid.

use std::collections::HashMap;

use bahyway_core::Result;
use enkidb_kaki::IdentityKaki;

use crate::entry::JournalEntry;
use crate::partition::PartitionKey;

/// In-memory journal buffer.  One partition = one ordered Vec of entries.
/// Production: backed by AppendWriter per partition file; here the buffer is
/// the source of truth for testing and single-node in-process use.
pub struct Journal {
    partitions: HashMap<PartitionKey, Vec<JournalEntry>>,
    shard_count: u16,
}

impl Journal {
    pub fn new(shard_count: u16) -> Self {
        Journal { partitions: HashMap::new(), shard_count }
    }

    /// Append an entry.  Entry is immutable once inserted (no mutation path exists).
    pub fn append(&mut self, entry: JournalEntry) -> Result<()> {
        let pk = PartitionKey::new(
            entry.target_kaki.tribe_id(),
            entry.target_kaki.uuid_hash(),
            entry.epoch,
            self.shard_count,
        );
        self.partitions.entry(pk).or_default().push(entry);
        Ok(())
    }

    /// Read all entries for a given particle in chronological order.
    /// A live/occasional single-entity lookup, not the access pattern for
    /// a bulk pass over every particle — that belongs on the Read-Node
    /// side (see `enkidb_readnode::materialize`, which groups the whole
    /// journal by target in one local pass instead of calling this in a
    /// loop).
    pub fn read_particle_history<'a>(
        &'a self,
        target: &IdentityKaki,
    ) -> Vec<&'a JournalEntry> {
        let tribe_id   = target.tribe_id();
        let uuid_hash  = target.uuid_hash();
        let shard      = (uuid_hash % self.shard_count as u32) as u16;

        // Scan all epochs for this shard — in production, epoch-partitioned files
        // are located by the index; here we scan the in-memory map.
        let target_bytes = *target.bytes();
        self.partitions
            .iter()
            .filter(move |(pk, _)| pk.tribe_id == tribe_id && pk.shard == shard)
            .flat_map(|(_, entries)| entries.iter())
            .filter(move |e| *e.target_kaki.bytes() == target_bytes)
            .collect()
    }

    /// Total entries across all partitions.
    pub fn entry_count(&self) -> usize {
        self.partitions.values().map(|v| v.len()).sum()
    }

    /// Every entry in the journal, in no particular order. Not an index —
    /// no state, computed fresh each call, purely a flatten over what's
    /// already stored (same shape as `entry_count`/`all_particles` above).
    /// For a bulk pass that needs every particle's full history (building
    /// a Read Node's Data Files, for instance), group these by target
    /// locally in the caller rather than calling `read_particle_history`
    /// once per particle — that call is priced for an occasional live
    /// lookup, not for O(n) bulk use.
    pub fn all_entries(&self) -> impl Iterator<Item = &JournalEntry> + '_ {
        self.partitions.values().flatten()
    }

    /// Return one IdentityKaki per distinct particle seen in the journal.
    /// Used by query engines to enumerate the candidate set.
    pub fn all_particles(&self) -> Vec<IdentityKaki> {
        // HashSet dedup, not Vec::contains: a linear-scan-per-insert dedup
        // is O(n^2) over n distinct particles. This is an algorithmic fix
        // inside an existing read-only method -- no persistent state, no
        // change to append()'s behavior -- not the write-path indexing
        // this module's docs above warn against.
        let mut seen: std::collections::HashSet<[u8; 16]> = std::collections::HashSet::new();
        let mut out: Vec<IdentityKaki> = Vec::new();
        for entries in self.partitions.values() {
            for e in entries {
                let bytes = *e.target_kaki.bytes();
                if seen.insert(bytes) {
                    out.push(e.target_kaki.clone());
                }
            }
        }
        out
    }

    /// Same as [`all_particles`](Self::all_particles), but stops enumerating
    /// once `cap` distinct particles have been found instead of always
    /// paying the full O(total entries) scan. Returns `(particles,
    /// truncated)` — `truncated` is true iff at least one more distinct
    /// particle exists beyond the returned set (enumeration was cut short,
    /// not just naturally exhausted at exactly `cap`).
    ///
    /// Exists for `ABORT_SCAN`-capped queries (`heptascript::engine::execute`):
    /// without this, a low `ABORT_SCAN` bounded how many candidates'
    /// histories got read, but every query still paid this method's full
    /// scan first just to build the candidate list — at real scale (10M+
    /// particles) that scan alone blows past the safety valve's own time
    /// budget. This lets the enumeration cost track the cap instead of the
    /// journal's real size.
    pub fn all_particles_capped(&self, cap: usize) -> (Vec<IdentityKaki>, bool) {
        let mut seen: std::collections::HashSet<[u8; 16]> = std::collections::HashSet::new();
        let mut out: Vec<IdentityKaki> = Vec::new();
        let mut truncated = false;
        'outer: for entries in self.partitions.values() {
            for e in entries {
                let bytes = *e.target_kaki.bytes();
                if seen.contains(&bytes) {
                    continue;
                }
                if out.len() >= cap {
                    truncated = true;
                    break 'outer;
                }
                seen.insert(bytes);
                out.push(e.target_kaki.clone());
            }
        }
        (out, truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{EventKaki, IdentityKaki, KakiRole, mint::KakiMinter};
    use bahyway_core::TribeId;
    use crate::entry::EavTriple;

    fn make_entries(minter: &KakiMinter, target: &IdentityKaki, n: usize) -> Vec<JournalEntry> {
        (0..n).map(|i| {
            let ek  = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
            let eav = vec![EavTriple::new(0x0001, format!("event_{}", i).into_bytes())];
            JournalEntry::new(ek, target.clone(), i as u32, eav)
        }).collect()
    }

    #[test]
    fn append_and_read_back() {
        let m      = KakiMinter::new(TribeId::from_u16(0x0001));
        let target = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);

        let entries = make_entries(&m, &target, 5);
        for e in entries {
            jnl.append(e).unwrap();
        }

        let history = jnl.read_particle_history(&target);
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn different_particles_dont_mix() {
        let m  = KakiMinter::new(TribeId::from_u16(0x0002));
        let t1 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let t2 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);

        for e in make_entries(&m, &t1, 3) { jnl.append(e).unwrap(); }
        for e in make_entries(&m, &t2, 2) { jnl.append(e).unwrap(); }

        assert_eq!(jnl.read_particle_history(&t1).len(), 3);
        assert_eq!(jnl.read_particle_history(&t2).len(), 2);
        assert_eq!(jnl.entry_count(), 5);
    }

    #[test]
    fn all_particles_capped_truncates_and_reports_it() {
        let m = KakiMinter::new(TribeId::from_u16(0x0003));
        let mut jnl = Journal::new(64);
        for _ in 0..10 {
            let t = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
            for e in make_entries(&m, &t, 1) {
                jnl.append(e).unwrap();
            }
        }

        let (capped, truncated) = jnl.all_particles_capped(3);
        assert_eq!(capped.len(), 3);
        assert!(truncated, "10 real particles, capped at 3 -- must report truncation");

        let (all, not_truncated) = jnl.all_particles_capped(100);
        assert_eq!(all.len(), 10);
        assert!(!not_truncated, "cap above the real count must not report truncation");

        let mut full = jnl.all_particles();
        let mut capped_full = all;
        full.sort_by_key(|k| *k.bytes());
        capped_full.sort_by_key(|k| *k.bytes());
        assert_eq!(full, capped_full, "capped(above real count) must agree with all_particles()");
    }
}
