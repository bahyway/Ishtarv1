//! exposure — the access-grant exposure preview.
//!
//! Granted 2026-07-19: before the Architect grants a stakeholder group
//! access to a `meta.collection` (e.g. documentation about a BeeMDM
//! gate), this answers "what sequence of impact would this reveal" —
//! which documents live in that collection, and which OTHER internal
//! concepts (gates, crates, sovereign names) they mention, per
//! `crate::concepts::ConceptRegistry::scan_mentions`.
//!
//! Deliberately built as a plain in-memory index maintained by
//! `WriteNode` at ingest time, over the same typed `Particle`/
//! `ConceptEntry` data the emitter already produces — NOT as a
//! HeptaScript query. `link.target` particles carry an `AkkValue::KakiPk`
//! value, and HeptaScript's `WHERE` comparison (`heptascript::engine::
//! compare_val`) has no case for comparing a stored `KakiPk` against a
//! literal — querying "which entities link to KAKI X" is not something
//! the query layer supports today. Rather than bolt that support on
//! speculatively, this index sidesteps it entirely: it already has the
//! typed data in hand at the moment `WriteNode::ingest_document_with_concepts`
//! calls it, before anything is bridged into raw Journal EAV bytes.

use enkidb_kaki::IdentityKaki;

use crate::concepts::ConceptEntry;

/// A real, in-memory index of "document → concepts it mentions," built
/// incrementally as documents are ingested via
/// [`crate::writenode::WriteNode::ingest_document_with_concepts`].
#[derive(Debug, Clone, Default)]
pub struct ConceptGraph {
    documents: Vec<DocumentEntry>,
}

#[derive(Debug, Clone)]
struct DocumentEntry {
    kaki: [u8; 16],
    title: String,
    collection: String,
    mentions: Vec<ConceptEntry>,
}

impl ConceptGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one ingested document's collection tag and the concepts its
    /// text mentioned. Idempotent per KAKI is NOT enforced here — a
    /// document re-ingested under a new epoch is recorded again,
    /// consistent with this Ecosystem's insert-only Journal model; the
    /// exposure preview de-duplicates by KAKI when reading back out.
    pub fn record_document(
        &mut self,
        kaki: IdentityKaki,
        title: &str,
        collection: &str,
        mentions: &[ConceptEntry],
    ) {
        self.documents.push(DocumentEntry {
            kaki: *kaki.bytes(),
            title: title.to_string(),
            collection: collection.to_string(),
            mentions: mentions.to_vec(),
        });
    }

    /// The exposure preview: every distinct document tagged with
    /// `collection`, plus the full, deduplicated set of concepts they
    /// mention — exactly what granting a stakeholder group access to
    /// this collection would reveal.
    pub fn exposure_preview(&self, collection: &str) -> ExposureReport {
        let mut seen_kakis: Vec<[u8; 16]> = Vec::new();
        let mut documents: Vec<String> = Vec::new();
        let mut concepts: Vec<ConceptEntry> = Vec::new();

        for doc in &self.documents {
            if doc.collection != collection || seen_kakis.contains(&doc.kaki) {
                continue;
            }
            seen_kakis.push(doc.kaki);
            documents.push(doc.title.clone());
            for mention in &doc.mentions {
                if !concepts.contains(mention) {
                    concepts.push(mention.clone());
                }
            }
        }
        documents.sort();
        concepts.sort_by(|a, b| a.name.cmp(&b.name));

        ExposureReport {
            collection: collection.to_string(),
            document_count: documents.len(),
            documents,
            revealed_concepts: concepts,
        }
    }

    /// Every distinct collection tag seen so far, sorted — lets a caller
    /// enumerate what's even grantable before previewing any one of them.
    pub fn known_collections(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .documents
            .iter()
            .map(|d| d.collection.clone())
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

/// What granting access to one `meta.collection` would reveal.
#[derive(Debug, Clone, PartialEq)]
pub struct ExposureReport {
    pub collection: String,
    pub document_count: usize,
    pub documents: Vec<String>,
    pub revealed_concepts: Vec<ConceptEntry>,
}

impl ExposureReport {
    /// A human-readable summary — what NISABA would actually say when
    /// asked "what happens if I grant this."
    pub fn narrate(&self) -> String {
        if self.document_count == 0 {
            return format!(
                "Collection \"{}\" has no documents yet — nothing would be revealed.",
                self.collection
            );
        }
        if self.revealed_concepts.is_empty() {
            return format!(
                "Collection \"{}\": {} document(s), mentioning no other tracked internal concepts.",
                self.collection, self.document_count
            );
        }
        let names: Vec<String> = self
            .revealed_concepts
            .iter()
            .map(|c| format!("{} ({})", c.name, c.kind.as_str()))
            .collect();
        format!(
            "Collection \"{}\": {} document(s). Granting access would also reveal {} internal concept(s): {}.",
            self.collection,
            self.document_count,
            self.revealed_concepts.len(),
            names.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concepts::ConceptKind;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;

    fn kaki(seed: u32) -> IdentityKaki {
        let minter = KakiMinter::new(TribeId::from_u16(0xFF20));
        IdentityKaki::try_from_kaki(minter.mint_identity(seed, enkidb_kaki::KakiRole::Zikru))
            .unwrap()
    }

    fn concept(name: &str, kind: ConceptKind) -> ConceptEntry {
        ConceptEntry {
            name: name.to_string(),
            kind,
        }
    }

    #[test]
    fn empty_graph_has_no_known_collections() {
        let graph = ConceptGraph::new();
        assert!(graph.known_collections().is_empty());
    }

    #[test]
    fn exposure_preview_lists_documents_and_deduplicated_concepts() {
        let mut graph = ConceptGraph::new();
        graph.record_document(
            kaki(1),
            "Gate G1 Runbook",
            "gates",
            &[
                concept("APSU", ConceptKind::Gate),
                concept("enkiddb", ConceptKind::Crate),
            ],
        );
        graph.record_document(
            kaki(2),
            "Gate G7 Runbook",
            "gates",
            &[
                concept("ENLIL", ConceptKind::Gate),
                concept("enkiddb", ConceptKind::Crate),
            ],
        );
        graph.record_document(
            kaki(3),
            "Unrelated Glossary Entry",
            "glossary",
            &[concept("Tigris", ConceptKind::SovereignName)],
        );

        let report = graph.exposure_preview("gates");
        assert_eq!(report.document_count, 2);
        assert_eq!(
            report.documents,
            vec!["Gate G1 Runbook".to_string(), "Gate G7 Runbook".to_string()]
        );
        // enkiddb mentioned by both documents but only reported once.
        assert_eq!(report.revealed_concepts.len(), 3);
        let names: Vec<&str> = report
            .revealed_concepts
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert!(names.contains(&"APSU"));
        assert!(names.contains(&"ENLIL"));
        assert!(names.contains(&"enkiddb"));
        assert!(
            !names.contains(&"Tigris"),
            "glossary-collection concepts must not leak into the gates preview"
        );
    }

    #[test]
    fn unknown_collection_yields_empty_report_not_an_error() {
        let graph = ConceptGraph::new();
        let report = graph.exposure_preview("nonexistent");
        assert_eq!(report.document_count, 0);
        assert!(report.documents.is_empty());
        assert!(report.revealed_concepts.is_empty());
    }

    #[test]
    fn same_kaki_recorded_twice_is_not_double_counted() {
        let mut graph = ConceptGraph::new();
        let doc = kaki(9);
        graph.record_document(
            doc,
            "Re-ingested Doc",
            "gates",
            &[concept("APSU", ConceptKind::Gate)],
        );
        graph.record_document(
            doc,
            "Re-ingested Doc",
            "gates",
            &[concept("APSU", ConceptKind::Gate)],
        );
        let report = graph.exposure_preview("gates");
        assert_eq!(report.document_count, 1);
    }

    #[test]
    fn known_collections_are_sorted_and_deduplicated() {
        let mut graph = ConceptGraph::new();
        graph.record_document(kaki(1), "A", "gates", &[]);
        graph.record_document(kaki(2), "B", "architecture-reference", &[]);
        graph.record_document(kaki(3), "C", "gates", &[]);
        assert_eq!(
            graph.known_collections(),
            vec!["architecture-reference".to_string(), "gates".to_string()]
        );
    }

    #[test]
    fn narrate_names_every_revealed_concept() {
        let mut graph = ConceptGraph::new();
        graph.record_document(
            kaki(1),
            "Gate Doc",
            "gates",
            &[concept("ENLIL", ConceptKind::Gate)],
        );
        let narration = graph.exposure_preview("gates").narrate();
        assert!(narration.contains("ENLIL (gate)"));
        assert!(narration.contains("1 document"));
    }

    #[test]
    fn narrate_handles_an_empty_collection_honestly() {
        let graph = ConceptGraph::new();
        let narration = graph.exposure_preview("gates").narrate();
        assert!(narration.contains("no documents yet"));
    }
}
