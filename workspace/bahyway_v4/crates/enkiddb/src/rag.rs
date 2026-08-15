//! RagIndex — the "Key-Value Document Mapping" search layer over EnkiDDB.
//!
//! The Architect's ask, verbatim: match a query against condensed
//! *summaries* (the Key) rather than the noisy full text, then fetch the
//! matching chunk's untouched full text (the Value) once a summary hits.
//!
//! This module is that mechanism, built directly on what already exists
//! rather than inventing new storage:
//!   - The Key layer is `adapa_recall::RecallIndex` (real TF-IDF + cosine
//!     retrieval, `crates/adapa-recall`) built over every `body.summary`
//!     particle already journaled by `WriteNode::ingest_document_categorized`
//!     -- one entry per document section (`emitter::emit_sections`).
//!   - The Value layer is the section's own `body.text` particle, fetched
//!     straight back out of the same `Journal` by the section's
//!     Identity-Kaki -- the full, untouched chunk, never summarized away.
//!
//! Scanning the Journal directly (not through the Read Node) is
//! deliberate: `EavTriple::attr_hash` is a CRC-16 of the attribute name
//! (see `enkidb_ingest::bridge::attr_hash`), so filtering by hash needs no
//! Data Files materialization step -- this index can be rebuilt straight
//! from the Write Node the moment new documents land.
//!
//! HONEST SCOPE: summaries are heuristic (title + a text preview), not
//! LLM-generated -- see `emitter::heuristic_summary`'s doc comment. When
//! `enkidullm-codex` exists, richer summaries can be substituted without
//! changing this module's shape: it only cares that `body.summary` holds
//! searchable text and `body.text` holds the verbatim payload.

use std::collections::HashMap;

use akkvalue::AkkValue;
use enkidb_ingest::bridge::{attr_hash, eav_triple_to_value};
use enkidb_journal::Journal;
use enkidb_kaki::IdentityKaki;
use enkidb_readnode::readnode::{ReadNode, ReadNodeError};
use enkidb_readnode::CachedReadNode;
use heptascript::ParseError;

use crate::orbit::DocOrbit;
use adapa_recall::{Document, DocId, RecallIndex};

/// One search hit: the matched section's Identity-Kaki and its cosine
/// similarity score against the query (see `RecallIndex::query`).
#[derive(Debug, Clone)]
pub struct RagHit {
    pub section: IdentityKaki,
    pub score: f32,
}

/// A built RAG search index over a `Journal`'s `body.summary` particles.
/// Rebuild (via [`RagIndex::build`]) after new documents are ingested --
/// same batch-rebuild contract as `RecallIndex` itself and as
/// `readnode::materialize_now`.
pub struct RagIndex {
    recall: RecallIndex,
    section_by_id: HashMap<DocId, IdentityKaki>,
    /// Populated only by [`RagIndex::build_from_readnode`] -- the full
    /// `body.text` value for each indexed section, captured at build time
    /// so [`RagIndex::fetch_value_from_readnode`] never needs a second
    /// round trip through the (expensive, unindexed) "match everything"
    /// query `build_from_readnode` itself has to run. Empty when built via
    /// [`RagIndex::build`] -- that path uses [`RagIndex::fetch_value`]
    /// instead, which reads straight from the Journal on demand.
    text_cache: HashMap<DocId, String>,
}

impl RagIndex {
    /// Scan every entry in `journal` for a `body.summary` triple and index
    /// it. Entries without one (e.g. a parent document entry, which has no
    /// `body.summary` of its own -- only its child sections do) are
    /// skipped, not an error.
    pub fn build(journal: &Journal) -> Self {
        let summary_hash = attr_hash(&DocOrbit::Body.attr("summary"));
        let mut docs = Vec::new();
        let mut section_by_id = HashMap::new();

        for entry in journal.all_entries() {
            let Some(triple) = entry.eav.iter().find(|t| t.attr_hash == summary_hash) else {
                continue;
            };
            if let AkkValue::Text(summary) = eav_triple_to_value(triple) {
                let id = kaki_hex(&entry.target_kaki);
                docs.push(Document::new(id.clone(), summary));
                section_by_id.insert(id, entry.target_kaki);
            }
        }

        RagIndex { recall: RecallIndex::build(docs), section_by_id, text_cache: HashMap::new() }
    }

    /// Rank sections by topical relevance to `query`'s condensed summary
    /// (the Key layer). Use [`RagIndex::fetch_value`] on each hit's
    /// `section` to retrieve the full, untouched chunk text (the Value).
    pub fn search(&self, query: &str, top_k: usize) -> Vec<RagHit> {
        self.recall
            .query(query, top_k)
            .into_iter()
            .filter_map(|(id, score)| self.section_by_id.get(&id).map(|k| RagHit { section: *k, score }))
            .collect()
    }

    /// Fetch a section's full, untouched `body.text` payload straight out
    /// of `journal` by its Identity-Kaki -- the Value half of the
    /// summary-to-chunk mapping. `None` if `section` carries no
    /// `body.text` (e.g. it isn't a section entry at all).
    pub fn fetch_value(journal: &Journal, section: &IdentityKaki) -> Option<String> {
        let text_hash = attr_hash(&DocOrbit::Body.attr("text"));
        journal
            .read_particle_history(section)
            .into_iter()
            .flat_map(|entry| entry.eav.iter())
            .find(|t| t.attr_hash == text_hash)
            .and_then(|t| match eav_triple_to_value(t) {
                AkkValue::Text(s) => Some(s),
                _ => None,
            })
    }

    /// Build the index from a `ReadNode`'s materialized Data Files instead
    /// of a live `Journal` -- the entry point for a read-server process
    /// that never has (and per ADR-012's Data Files Law, must never have)
    /// access to the Write Node's Journal.
    ///
    /// `ReadNode::query` only serves single-exact-equality lookups (see
    /// its own doc comment) -- there is no "give me every entity" query.
    /// This uses `hist.event = "BIRTH"`, the one attribute *every* entity
    /// this crate's `DocumentEmitter` mints carries (both parent documents
    /// and sections), as a universal, indexable marker to enumerate
    /// everything, then keeps only the entities that also carry
    /// `body.summary` (i.e. sections, not parent documents).
    ///
    /// Projects `body.summary`/`body.text` by name, not `WHAT E[*]`:
    /// materialized Data Files only ever stored `attr_hash` (a CRC-16), not
    /// the original attribute string (see `enkidb_ingest::bridge::attr_hash`
    /// -- the hash has no reverse map anywhere in this codebase), so a `*`
    /// wildcard projection can only hand back hash-keyed placeholder names
    /// (`heptascript::engine::project_what`'s own comment says as much).
    /// Naming the attributes explicitly sidesteps that entirely --
    /// `project_what` resolves a named projection by hashing the *request*
    /// and returns it labeled with the name that was asked for.
    pub fn build_from_readnode(read_node: &mut ReadNode) -> Result<Self, ReadNodeError> {
        let result = read_node.query(
            "WHO T.E\nWHAT E[body.summary, body.text]\nWHERE E[hist.event] = \"BIRTH\"",
        )?;

        let mut docs = Vec::new();
        let mut section_by_id = HashMap::new();
        let mut text_cache = HashMap::new();

        for m in &result.matched {
            let mut summary = None;
            let mut text = None;
            for (attr, val) in &m.projected {
                let AkkValue::Text(s) = val else { continue };
                match attr.as_str() {
                    "body.summary" => summary = Some(s.clone()),
                    "body.text" => text = Some(s.clone()),
                    _ => {}
                }
            }
            if let Some(summary) = summary {
                let id = kaki_hex(&m.entity);
                docs.push(Document::new(id.clone(), summary));
                section_by_id.insert(id.clone(), m.entity);
                if let Some(text) = text {
                    text_cache.insert(id, text);
                }
            }
        }

        Ok(RagIndex { recall: RecallIndex::build(docs), section_by_id, text_cache })
    }

    /// Same as [`RagIndex::build_from_readnode`], built from a
    /// [`CachedReadNode`] instead of a disk-per-query [`ReadNode`] --
    /// the in-memory engine `enkiddb-read-server` moved to so EnkiDDB
    /// gets the same zero-per-query-disk-I/O treatment `enkidb-read-
    /// server` (core EnkiDB) already has. `CachedReadNode::query` never
    /// fails on I/O (everything is already resident), so the only error
    /// case left is a malformed query string -- which is a hardcoded
    /// literal here, not user input, so this can't realistically fail,
    /// but still returns `Result` rather than panicking on principle.
    pub fn build_from_cached(crn: &CachedReadNode) -> Result<Self, ParseError> {
        let result = crn.query("WHO T.E\nWHAT E[body.summary, body.text]\nWHERE E[hist.event] = \"BIRTH\"")?;

        let mut docs = Vec::new();
        let mut section_by_id = HashMap::new();
        let mut text_cache = HashMap::new();

        for m in &result.matched {
            let mut summary = None;
            let mut text = None;
            for (attr, val) in &m.projected {
                let AkkValue::Text(s) = val else { continue };
                match attr.as_str() {
                    "body.summary" => summary = Some(s.clone()),
                    "body.text" => text = Some(s.clone()),
                    _ => {}
                }
            }
            if let Some(summary) = summary {
                let id = kaki_hex(&m.entity);
                docs.push(Document::new(id.clone(), summary));
                section_by_id.insert(id.clone(), m.entity.clone());
                if let Some(text) = text {
                    text_cache.insert(id, text);
                }
            }
        }

        Ok(RagIndex { recall: RecallIndex::build(docs), section_by_id, text_cache })
    }

    /// Fetch a section's full, untouched `body.text` payload -- the
    /// `ReadNode`-backed counterpart to [`RagIndex::fetch_value`]. Reads
    /// from the cache captured at [`RagIndex::build_from_readnode`] time
    /// rather than re-querying: `ReadNode::query` has no "fetch by KAKI"
    /// point lookup (HeptaScript's WHERE matches on attribute values, not
    /// entity identity), so the only way to find one section's row is the
    /// same unindexed `hist.event = "BIRTH"` scan `build_from_readnode`
    /// already paid for once -- no reason to pay for it again per fetch.
    pub fn fetch_value_from_readnode(&self, section: &IdentityKaki) -> Option<String> {
        self.text_cache.get(&kaki_hex(section)).cloned()
    }

    pub fn len(&self) -> usize {
        self.recall.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recall.is_empty()
    }
}

fn kaki_hex(kaki: &IdentityKaki) -> String {
    kaki.bytes().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::DocumentParser;
    use crate::writenode::WriteNode;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;

    fn write_node() -> WriteNode {
        WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF0D)), 64)
    }

    #[test]
    fn search_finds_the_topically_relevant_section() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown(
            "# Reference\n\n## KAKI Identity\n\nKAKI is the sovereign 16-byte identity used everywhere in EnkiDB.\n\n## Weather\n\nThe weather today is cloudy with a chance of rain.\n",
        );
        wn.ingest_document_categorized(&doc, 1, "component");

        let index = RagIndex::build(wn.journal());
        assert_eq!(index.len(), 2, "one entry per section");

        let hits = index.search("KAKI sovereign identity", 5);
        assert!(!hits.is_empty());

        let value = RagIndex::fetch_value(wn.journal(), &hits[0].section).unwrap();
        assert!(value.contains("KAKI is the sovereign 16-byte identity"));
    }

    #[test]
    fn fetch_value_returns_the_full_untouched_text_not_the_summary() {
        let mut wn = write_node();
        let long_body = "sovereign ".repeat(100);
        let md = format!("# T\n\n## Section\n\n{long_body}\n");
        let doc = DocumentParser::parse_markdown(&md);
        wn.ingest_document_categorized(&doc, 1, "component");

        let index = RagIndex::build(wn.journal());
        let hits = index.search("sovereign", 1);
        assert_eq!(hits.len(), 1);

        let value = RagIndex::fetch_value(wn.journal(), &hits[0].section).unwrap();
        assert_eq!(value.trim(), format!("Section\n{}", long_body.trim()).trim());
        assert!(value.len() > 200, "the value must be the full text, not the truncated summary");
    }

    #[test]
    fn parent_document_entries_are_not_indexed_only_sections_are() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown("# Standalone Title\n\n## Only\n\nBody.\n");
        wn.ingest_document_categorized(&doc, 1, "component");

        let index = RagIndex::build(wn.journal());
        // Only the one section has a body.summary -- the parent entry does not.
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn empty_journal_yields_an_empty_index() {
        let wn = write_node();
        let index = RagIndex::build(wn.journal());
        assert!(index.is_empty());
        assert!(index.search("anything", 5).is_empty());
    }

    fn tmp_base(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_rag_readnode_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        dir.join("base")
    }

    #[test]
    fn build_from_readnode_matches_journal_based_search_after_materializing() {
        // The read-server process never has the Journal -- only Data Files.
        // Prove build_from_readnode(), running purely against materialized
        // Data Files via ReadNode::query, finds the same relevant section
        // and returns the same full text as the Journal-backed path does.
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown(
            "# Reference\n\n## KAKI Identity\n\nKAKI is the sovereign 16-byte identity used everywhere in EnkiDB.\n\n## Weather\n\nThe weather today is cloudy with a chance of rain.\n",
        );
        wn.ingest_document_categorized(&doc, 1, "component");

        let base = tmp_base("build_from_readnode");
        crate::readnode::materialize_now(&wn, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let mut rn = ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let index = RagIndex::build_from_readnode(&mut rn).unwrap();
        assert_eq!(index.len(), 2, "one entry per section, parent doc excluded");

        let hits = index.search("KAKI sovereign identity", 5);
        assert!(!hits.is_empty());
        let value = index.fetch_value_from_readnode(&hits[0].section).unwrap();
        assert!(value.contains("KAKI is the sovereign 16-byte identity"));
    }

    #[test]
    fn build_from_cached_matches_journal_based_search_after_materializing() {
        // Same promise as build_from_readnode's own test, for the
        // in-memory engine enkiddb-read-server actually runs now.
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown(
            "# Reference\n\n## KAKI Identity\n\nKAKI is the sovereign 16-byte identity used everywhere in EnkiDB.\n\n## Weather\n\nThe weather today is cloudy with a chance of rain.\n",
        );
        wn.ingest_document_categorized(&doc, 1, "component");

        let base = tmp_base("build_from_cached");
        crate::readnode::materialize_now(&wn, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let index = RagIndex::build_from_cached(&crn).unwrap();
        assert_eq!(index.len(), 2, "one entry per section, parent doc excluded");

        let hits = index.search("KAKI sovereign identity", 5);
        assert!(!hits.is_empty());
        let value = index.fetch_value_from_readnode(&hits[0].section).unwrap();
        assert!(value.contains("KAKI is the sovereign 16-byte identity"));
    }

    #[test]
    fn build_from_cached_on_empty_data_files_is_empty_not_an_error() {
        let wn = write_node();
        let base = tmp_base("empty_cached");
        crate::readnode::materialize_now(&wn, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let crn = CachedReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let index = RagIndex::build_from_cached(&crn).unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn build_from_readnode_on_empty_data_files_is_empty_not_an_error() {
        let wn = write_node();
        let base = tmp_base("empty_readnode");
        crate::readnode::materialize_now(&wn, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let mut rn = ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let index = RagIndex::build_from_readnode(&mut rn).unwrap();
        assert!(index.is_empty());
    }
}
