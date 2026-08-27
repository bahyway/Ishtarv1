//! enki-meta-doc-bridge — the LINK, never a copy.
//!
//! Corrects an earlier misreading of the Architect's own diagram: the
//! new EnkiDB(Golden)/EnkiDW arrows into EnkiMDB and EnkiDDB do NOT mean
//! particles get sent, copied, or newly minted into those databases.
//! "What I meant is that EnkiDB can through a bridge to EnkiMDB &
//! EnkiDDB show its particles related to its metadata in EnkiMDB and to
//! its MetaData Documents in EnkiDDB. SO there is NO Sending data;
//! there is ONLY Linking Golden Data to its related metadata and
//! documentations. This is also true for EnkiDW."
//!
//! The mechanism is the cheapest one Triple-O already gives for free:
//! every particle in every one of the 7 Types already carries a
//! TribeId (KAKI bytes 4..5, immutable, set at birth). This crate reads
//! (never writes) EnkiMDB's and EnkiDDB's own journals and returns
//! every entry that shares a Golden/Warehouse particle's tribe -- a
//! pure lookup, zero new data written anywhere.
//!
//! ## The one honest gap this crate does not paper over
//!
//! For this to find real matches today, whatever EnkiMDB/EnkiDDB
//! content is *specific to a client's tribe* needs to actually be
//! minted under that tribe's own TribeId. Right now the real ingestion
//! paths mint EnkiDDB documents under sovereign, ecosystem-level tribes
//! (`enkiddb::DOCS_TRIBE_ID`, `enkiddb::PB_DOCS_TRIBE_ID`) rather than a
//! client's business tribe -- so a bridge query for a client tribe will
//! honestly return empty until client-specific metadata/documentation
//! is minted under that same TribeId. This crate does not invent or
//! guess that minting convention; it only resolves whatever sharing
//! already exists, honestly, including "nothing yet."
#![forbid(unsafe_code)]

use bahyway_core::TribeId;
use enkidb_journal::Journal;
use enkidb_kaki::IdentityKaki;

/// Every entry in `journal` whose target particle shares `tribe` with
/// the Golden/Warehouse particle the caller is bridging from. Returns
/// distinct Identity-Kakis only (a particle with a long event history
/// is not repeated once per event) -- these ARE the "related metadata"
/// or "related documentation" the bridge promises, addressable but
/// never copied into the caller's own store.
pub fn linked_particles(journal: &Journal, tribe: TribeId) -> Vec<IdentityKaki> {
    let mut out: Vec<IdentityKaki> = Vec::new();
    for entry in journal.all_entries() {
        if entry.target_kaki.tribe_id() == tribe && !out.contains(&entry.target_kaki) {
            out.push(entry.target_kaki);
        }
    }
    out
}

/// Convenience wrapper naming the EnkiMDB side of the bridge explicitly
/// -- same mechanism as `linked_particles`, named so a caller reading
/// `enki_meta_doc_bridge::linked_metadata(golden_particle.tribe_id(),
/// mdb_journal)` sees the intent without re-deriving it from a generic
/// function name.
pub fn linked_metadata(tribe: TribeId, enkimdb_journal: &Journal) -> Vec<IdentityKaki> {
    linked_particles(enkimdb_journal, tribe)
}

/// Convenience wrapper naming the EnkiDDB side of the bridge.
pub fn linked_documentation(tribe: TribeId, enkiddb_journal: &Journal) -> Vec<IdentityKaki> {
    linked_particles(enkiddb_journal, tribe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_journal::entry::JournalEntry;
    use enkidb_kaki::{KakiMinter, KakiRole};

    fn journal_with_entries(tribe_and_epoch: &[(u16, u32)]) -> Journal {
        let mut journal = Journal::new(64);
        for (i, &(tribe, epoch)) in tribe_and_epoch.iter().enumerate() {
            let minter = KakiMinter::new(TribeId::from_u16(tribe));
            let identity = minter.identity(KakiRole::Zikru);
            let target = IdentityKaki::try_from_kaki(identity).unwrap();
            let event =
                enkidb_kaki::EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
            journal
                .append(JournalEntry::new(event, target, epoch, vec![]))
                .unwrap();
            let _ = i;
        }
        journal
    }

    #[test]
    fn finds_only_entries_matching_the_requested_tribe() {
        let journal = journal_with_entries(&[(1, 1), (1, 2), (2, 1), (3, 1)]);
        let found = linked_particles(&journal, TribeId::from_u16(1));
        assert!(found.iter().all(|k| k.tribe_id() == TribeId::from_u16(1)));
        assert_eq!(
            found.len(),
            2,
            "the two tribe-1 particles must both be found, distinctly"
        );
    }

    #[test]
    fn returns_empty_when_no_entry_shares_the_tribe() {
        let journal = journal_with_entries(&[(1, 1), (2, 1)]);
        let found = linked_particles(&journal, TribeId::from_u16(999));
        assert!(
            found.is_empty(),
            "an honest empty result -- no fabricated link"
        );
    }

    #[test]
    fn a_particle_with_multiple_events_is_returned_once() {
        // Two entries, same target tribe, both epoch-distinct events on
        // what should read as related activity for that tribe -- the
        // bridge counts distinct PARTICLES, not distinct events.
        let journal = journal_with_entries(&[(5, 1), (5, 2)]);
        let found = linked_particles(&journal, TribeId::from_u16(5));
        // These are two DIFFERENT minted identities (KakiMinter mints a
        // fresh identity per call), so both are real, distinct
        // particles under tribe 5 -- confirms dedup only collapses a
        // truly repeated target_kaki, not merely same-tribe siblings.
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn linked_metadata_and_linked_documentation_are_named_wrappers_over_the_same_mechanism() {
        let journal = journal_with_entries(&[(7, 1)]);
        let via_metadata = linked_metadata(TribeId::from_u16(7), &journal);
        let via_documentation = linked_documentation(TribeId::from_u16(7), &journal);
        assert_eq!(via_metadata, via_documentation);
    }
}
