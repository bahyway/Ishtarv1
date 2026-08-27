//! TopicGraph — citation-affinity clustering over EnkiDDB's real corpus.
//!
//! ## What this is
//! A document graph is already sitting in EnkiDDB, unused for this
//! purpose: `meta.collection` (assigned per-document at ingest time by
//! `crate::ingest::infer_collection`, e.g. "components", "gates",
//! "marduk", "decisions-adr") already partitions the corpus into topics,
//! and `WriteNode::link_discovered_dependencies` already journals a real
//! `depends-on` citation edge every time one document's markdown links to
//! another that was ingested alongside it. This module reads both back
//! out through a Read Node and answers two questions per document:
//!   - which topic does it cite into the most (`TopicPlacement::home_topic`)?
//!   - does it cite substantially into more than one topic, making it a
//!     bridge between them (`TopicPlacement::bridges_to`)?
//!
//! ## What this deliberately is NOT (honest scope)
//! This module classifies documents against a topic partition that
//! already exists (`meta.collection`) — it does not discover topics from
//! scratch. Unsupervised topic/community discovery over the citation
//! graph is real, separate follow-on work (a natural fit for k-means or
//! a graph-clustering pass elsewhere in this workspace), not something
//! this module fakes by pretending `meta.collection` was computed instead
//! of assigned.
//!
//! `radius`/`affinity` below are ordinal ranking numbers — "cites more
//! into this topic" ranks lower `radius` — not a physical distance or a
//! simulated dynamical system. Nothing here evolves a document's
//! position over time; every call recomputes fresh from the Read Node's
//! current Data Files. Citation timestamps exist in the Journal
//! (`hist.event`'s particle carries one, same as every other particle)
//! but this module does not read them — comparing two `TopicGraph`s built
//! from Data Files at two different points in time would show drift, but
//! nothing here does that comparison for you; it is unbuilt, not hidden.
//! Likewise, detecting a citation to a document that was never ingested
//! ("documentation debt") is a real, useful, and NOT implemented
//! extension — `WriteNode::link_discovered_dependencies` only ever
//! records a `depends-on` edge for references it could resolve, so an
//! unresolvable reference already leaves no trace to detect here.
//!
//! ## Bug fixed alongside this module
//! `link_discovered_dependencies` used to write a citing document's
//! `link.target`/`link.description` directly onto that document's own
//! entity. A document citing more than one other document then silently
//! lost all but the last-journaled citation the moment it was read back
//! through a Read Node: `ReadNode`/`CachedReadNode` fold an entity's
//! history into a single-slot `(attr_hash -> value)` map before
//! projecting, so repeated `link.target` triples on one entity collapse
//! to whichever was journaled last. This module is the first real reader
//! of `depends-on` edges through a Read Node, so it would have been
//! silently wrong against real multi-citation documents; the fix (mint
//! one child edge-entity per citation, mirroring `emit_sections`'s
//! "section-of" and `emit_concept_mentions`'s "mentioned-in" edges) lives
//! in `writenode.rs::link_discovered_dependencies`, proven by
//! `multiple_citations_from_one_document_all_survive_through_read_node`
//! below.

use std::collections::HashMap;

use akkvalue::AkkValue;
use enkidb_kaki::{IdentityKaki, Kaki};
use enkidb_readnode::readnode::{ReadNode, ReadNodeError};
use enkidb_readnode::CachedReadNode;
use heptascript::ParseError;

/// Default share of a document's total citations that must land in a
/// non-home topic before that topic is reported as a bridge. Tunable —
/// pass a different value to [`TopicGraph::placement_with_threshold`],
/// or treat this as the default `placement` uses.
pub const DEFAULT_BRIDGE_THRESHOLD: f32 = 0.30;

const BIRTH_QUERY: &str = "WHO T.E\nWHAT E[meta.collection, meta.title, link.target, link.source, link.description]\nWHERE E[hist.event] = \"BIRTH\"";

/// One document's placement relative to the real citation graph: its
/// strongest topic, how dominant that topic is among its citations, an
/// ordinal proximity ranking, and any other topics it bridges to.
#[derive(Debug, Clone, PartialEq)]
pub struct TopicPlacement {
    pub home_topic: String,
    /// citations landing in `home_topic` / total real citations this
    /// document is the source of. 1.0 means every citation lands in the
    /// same topic.
    pub affinity: f32,
    /// `1 / (1 + citations into home_topic)` — smaller means more
    /// citations into that topic, i.e. more central to it. An ordinal
    /// ranking device only; see module doc comment.
    pub radius: f32,
    /// other topics this document also cites into at or above the
    /// bridge threshold.
    pub bridges_to: Vec<String>,
}

struct DocRecord {
    topic: Option<String>,
    title: Option<String>,
    /// every real "depends-on" edge this document is the source of,
    /// resolved to its target's topic (`None` if the target's own
    /// `meta.collection` couldn't be resolved).
    cites_into: Vec<Option<String>>,
}

/// One document's topic tag, title (if resolvable), and placement — the
/// shape [`TopicGraph::documents`] enumerates for a caller that wants a
/// full listing (e.g. a CLI report), not a single lookup.
#[derive(Debug, Clone)]
pub struct DocumentTopic {
    pub kaki: IdentityKaki,
    pub title: Option<String>,
    pub topic: String,
    pub placement: Option<TopicPlacement>,
}

/// A built citation-affinity graph over one Read Node snapshot. Rebuild
/// after new documents/citations are ingested and materialized — same
/// contract as `rag::RagIndex`.
pub struct TopicGraph {
    docs: HashMap<[u8; 16], DocRecord>,
    by_topic: HashMap<String, Vec<[u8; 16]>>,
}

impl TopicGraph {
    /// Build from a `ReadNode`'s materialized Data Files (disk-per-query
    /// engine). See [`TopicGraph::build_from_cached`] for the in-memory
    /// engine `enkiddb-read-server` actually runs.
    pub fn build_from_readnode(read_node: &mut ReadNode) -> Result<Self, ReadNodeError> {
        let result = read_node.query(BIRTH_QUERY)?;
        Ok(Self::from_rows(
            result.matched.into_iter().map(|m| (m.entity, m.projected)),
        ))
    }

    /// Same as [`TopicGraph::build_from_readnode`], built from a
    /// `CachedReadNode` — the zero-per-query-disk-I/O engine
    /// `enkiddb-read-server` runs, mirroring `rag::RagIndex::build_from_cached`.
    pub fn build_from_cached(crn: &CachedReadNode) -> Result<Self, ParseError> {
        let result = crn.query(BIRTH_QUERY)?;
        Ok(Self::from_rows(
            result.matched.into_iter().map(|m| (m.entity, m.projected)),
        ))
    }

    fn from_rows(rows: impl Iterator<Item = (IdentityKaki, Vec<(String, AkkValue)>)>) -> Self {
        let mut topic_by_doc: HashMap<[u8; 16], String> = HashMap::new();
        let mut title_by_doc: HashMap<[u8; 16], String> = HashMap::new();
        // (source, target, is_depends_on)
        let mut edges: Vec<([u8; 16], [u8; 16])> = Vec::new();
        let mut all_doc_kakis: Vec<[u8; 16]> = Vec::new();

        for (entity, projected) in rows {
            let entity_bytes = *entity.bytes();
            let mut collection: Option<String> = None;
            let mut title: Option<String> = None;
            let mut target: Option<[u8; 16]> = None;
            let mut source: Option<[u8; 16]> = None;
            let mut is_depends_on = false;

            for (attr, val) in &projected {
                match (attr.as_str(), val) {
                    ("meta.collection", AkkValue::Text(s)) => collection = Some(s.clone()),
                    ("meta.title", AkkValue::Text(s)) => title = Some(s.clone()),
                    ("link.target", AkkValue::KakiPk(bytes)) => target = Some(*bytes),
                    ("link.source", AkkValue::KakiPk(bytes)) => source = Some(*bytes),
                    ("link.description", AkkValue::Text(s)) if s == "depends-on" => {
                        is_depends_on = true
                    }
                    _ => {}
                }
            }

            if let Some(collection) = collection {
                topic_by_doc.insert(entity_bytes, collection);
                all_doc_kakis.push(entity_bytes);
                if let Some(title) = title {
                    title_by_doc.insert(entity_bytes, title);
                }
            }
            if is_depends_on {
                if let (Some(source), Some(target)) = (source, target) {
                    edges.push((source, target));
                }
            }
        }

        let mut docs: HashMap<[u8; 16], DocRecord> = HashMap::new();
        let mut by_topic: HashMap<String, Vec<[u8; 16]>> = HashMap::new();

        for doc_kaki in &all_doc_kakis {
            let topic = topic_by_doc.get(doc_kaki).cloned();
            let title = title_by_doc.get(doc_kaki).cloned();
            let cites_into: Vec<Option<String>> = edges
                .iter()
                .filter(|(src, _)| src == doc_kaki)
                .map(|(_, dst)| topic_by_doc.get(dst).cloned())
                .collect();

            if let Some(t) = &topic {
                by_topic.entry(t.clone()).or_default().push(*doc_kaki);
            }
            docs.insert(
                *doc_kaki,
                DocRecord {
                    topic,
                    title,
                    cites_into,
                },
            );
        }

        TopicGraph { docs, by_topic }
    }

    /// [`TopicGraph::placement`] with a custom bridge-detection threshold
    /// instead of [`DEFAULT_BRIDGE_THRESHOLD`].
    pub fn placement_with_threshold(
        &self,
        doc: &IdentityKaki,
        bridge_threshold: f32,
    ) -> Option<TopicPlacement> {
        let record = self.docs.get(doc.bytes())?;
        let resolved: Vec<&String> = record
            .cites_into
            .iter()
            .filter_map(|t| t.as_ref())
            .collect();
        let total = resolved.len();
        if total == 0 {
            return None;
        }

        let mut counts: HashMap<&str, usize> = HashMap::new();
        for topic in &resolved {
            *counts.entry(topic.as_str()).or_insert(0) += 1;
        }
        let mut scored: Vec<(&str, usize)> = counts.into_iter().collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

        let (home_topic, home_count) = scored[0];
        let affinity = home_count as f32 / total as f32;
        let radius = 1.0 / (1.0 + home_count as f32);
        let bridges_to: Vec<String> = scored
            .iter()
            .skip(1)
            .filter(|(_, count)| *count as f32 / total as f32 >= bridge_threshold)
            .map(|(topic, _)| topic.to_string())
            .collect();

        Some(TopicPlacement {
            home_topic: home_topic.to_string(),
            affinity,
            radius,
            bridges_to,
        })
    }

    /// A document's placement using [`DEFAULT_BRIDGE_THRESHOLD`]. `None`
    /// if the document has no resolvable citations — deliberately not
    /// defaulted to a placement; an uncited (or only-externally-citing)
    /// document is honestly unplaced, not guessed at.
    pub fn placement(&self, doc: &IdentityKaki) -> Option<TopicPlacement> {
        self.placement_with_threshold(doc, DEFAULT_BRIDGE_THRESHOLD)
    }

    /// The `k` documents most similar to `doc` by home-topic proximity:
    /// same home topic first (ranked by `|radius difference|`, closest
    /// first), with documents that bridge back to `doc`'s own home topic
    /// surfaced ahead of same-radius same-topic neighbors. Empty if `doc`
    /// has no placement.
    pub fn related(&self, doc: &IdentityKaki, k: usize) -> Vec<IdentityKaki> {
        let Some(anchor) = self.placement(doc) else {
            return vec![];
        };
        let mut scored: Vec<(bool, f32, [u8; 16])> = Vec::new();

        for (other_bytes, record) in &self.docs {
            if other_bytes == doc.bytes() {
                continue;
            }
            let Some(other_topic) = &record.topic else {
                continue;
            };
            let Some(other) = self.placement_by_bytes(*other_bytes) else {
                continue;
            };

            let same_home = *other_topic == anchor.home_topic;
            let is_bridge = other.bridges_to.contains(&anchor.home_topic)
                || anchor.bridges_to.contains(other_topic);
            if !same_home && !is_bridge {
                continue;
            }
            scored.push((
                !is_bridge,
                (other.radius - anchor.radius).abs(),
                *other_bytes,
            ));
        }

        scored.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        scored
            .into_iter()
            .take(k)
            .filter_map(|(_, _, bytes)| {
                IdentityKaki::try_from_kaki(Kaki::from_bytes(bytes).ok()?).ok()
            })
            .collect()
    }

    fn placement_by_bytes(&self, bytes: [u8; 16]) -> Option<TopicPlacement> {
        let kaki = IdentityKaki::try_from_kaki(Kaki::from_bytes(bytes).ok()?).ok()?;
        self.placement(&kaki)
    }

    /// Every distinct topic (`meta.collection` value) with at least one
    /// document, sorted for stable output.
    pub fn topics(&self) -> Vec<&str> {
        let mut t: Vec<&str> = self.by_topic.keys().map(|s| s.as_str()).collect();
        t.sort_unstable();
        t
    }

    /// Every document in the graph with its topic, title (if resolvable),
    /// and placement (`None` when the document has no resolvable
    /// citations — see [`TopicGraph::placement`]'s own doc comment).
    /// Sorted by topic then title for stable report output.
    pub fn documents(&self) -> Vec<DocumentTopic> {
        let mut out: Vec<DocumentTopic> = self
            .docs
            .iter()
            .filter_map(|(bytes, record)| {
                let topic = record.topic.clone()?;
                let kaki = IdentityKaki::try_from_kaki(Kaki::from_bytes(*bytes).ok()?).ok()?;
                let placement = self.placement(&kaki);
                Some(DocumentTopic {
                    kaki,
                    title: record.title.clone(),
                    topic,
                    placement,
                })
            })
            .collect();
        out.sort_by(|a, b| a.topic.cmp(&b.topic).then_with(|| a.title.cmp(&b.title)));
        out
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writenode::WriteNode;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;

    fn write_node() -> WriteNode {
        WriteNode::new(KakiMinter::new(TribeId::from_u16(0xD6D6)), 64)
    }

    fn tmp_base(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_topics_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("x")
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    fn git_scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_topics_ingest_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "bahaa.fadam@gmail.com"]);
        git(&dir, &["config", "user.name", "Test Author"]);
        dir
    }

    fn git_commit_all(dir: &std::path::Path) {
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "test commit"]);
    }

    #[test]
    fn multiple_citations_from_one_document_all_survive_through_read_node() {
        // The regression proof for the writenode.rs fix: a document that
        // cites THREE others must show all three as real edges once read
        // back through a Read Node, not just the last-journaled one.
        let dir = git_scratch_dir("multi_cite");
        std::fs::write(
            dir.join("hub.md"),
            "# Hub\n\nSee [a](./a.md), [b](./b.md), and [c](./c.md).\n",
        )
        .unwrap();
        std::fs::write(dir.join("a.md"), "# A\n\nBody A.\n").unwrap();
        std::fs::write(dir.join("b.md"), "# B\n\nBody B.\n").unwrap();
        std::fs::write(dir.join("c.md"), "# C\n\nBody C.\n").unwrap();
        git_commit_all(&dir);

        let mut wn = write_node();
        let allow = crate::authorship::TeamAllowlist::seed();
        let kakis = wn
            .ingest_directory_categorized_checked(&dir, 1, &allow)
            .unwrap();
        assert_eq!(kakis.len(), 4, "hub.md + a.md + b.md + c.md");

        let base = tmp_base("multi_cite");
        crate::readnode::materialize_now(
            &wn,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let result = rn.query(BIRTH_QUERY).unwrap();
        let depends_on_edges = result
            .matched
            .iter()
            .filter(|m| {
                m.projected.iter().any(|(attr, val)| {
                    attr == "link.description"
                        && matches!(val, AkkValue::Text(s) if s == "depends-on")
                })
            })
            .count();
        assert_eq!(
            depends_on_edges, 3,
            "all three depends-on edges from hub.md must be independently queryable"
        );
    }

    #[test]
    fn bridge_detected_when_citations_span_two_topics() {
        // gate_a lives under docs/gates/ and cites two docs/gates/ docs
        // plus one docs/components/ doc -- meta.collection is inferred
        // from the docs/<subfolder>/ path (crate::ingest::infer_collection),
        // exactly the real convention PB-208's full-corpus ingest uses.
        // Expected: home = gates, bridge = components (1/3 >= 0.30).
        let dir = git_scratch_dir("bridge");
        std::fs::create_dir_all(dir.join("docs/gates")).unwrap();
        std::fs::create_dir_all(dir.join("docs/components")).unwrap();
        std::fs::write(
            dir.join("docs/gates/gate_a.md"),
            "# Gate A\n\nSee [gate b](./gate_b.md), [gate c](./gate_c.md), and [comp x](../components/comp_x.md).\n",
        )
        .unwrap();
        std::fs::write(dir.join("docs/gates/gate_b.md"), "# Gate B\n\nBody.\n").unwrap();
        std::fs::write(dir.join("docs/gates/gate_c.md"), "# Gate C\n\nBody.\n").unwrap();
        std::fs::write(dir.join("docs/components/comp_x.md"), "# Comp X\n\nBody.\n").unwrap();
        git_commit_all(&dir);

        let mut wn = write_node();
        let allow = crate::authorship::TeamAllowlist::seed();
        let kakis = wn
            .ingest_directory_categorized_checked(&dir, 1, &allow)
            .unwrap();
        assert_eq!(kakis.len(), 4);

        let base = tmp_base("bridge");
        crate::readnode::materialize_now(
            &wn,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let graph = TopicGraph::build_from_readnode(&mut rn).unwrap();

        assert_eq!(graph.topics(), vec!["components", "gates"]);

        // Find gate_a's own placement by trying every ingested kaki --
        // ingest_directory_categorized_checked doesn't hand back a
        // path->kaki map, only the list, so identify it by outcome
        // (exactly one of the four should be a bridge).
        let placements: Vec<_> = kakis.iter().filter_map(|k| graph.placement(k)).collect();
        let bridges: Vec<_> = placements
            .iter()
            .filter(|p| !p.bridges_to.is_empty())
            .collect();
        assert_eq!(
            bridges.len(),
            1,
            "exactly one document (gate_a) should bridge"
        );
        assert_eq!(bridges[0].home_topic, "gates");
        assert_eq!(bridges[0].bridges_to, vec!["components".to_string()]);
    }

    #[test]
    fn related_surfaces_same_topic_neighbors() {
        let dir = git_scratch_dir("related");
        std::fs::create_dir_all(dir.join("docs/gates")).unwrap();
        std::fs::write(dir.join("docs/gates/g1.md"), "# G1\n\nSee [g2](./g2.md).\n").unwrap();
        std::fs::write(dir.join("docs/gates/g2.md"), "# G2\n\nSee [g1](./g1.md).\n").unwrap();
        std::fs::write(dir.join("docs/gates/g3.md"), "# G3\n\nSee [g1](./g1.md).\n").unwrap();
        git_commit_all(&dir);

        let mut wn = write_node();
        let allow = crate::authorship::TeamAllowlist::seed();
        let kakis = wn
            .ingest_directory_categorized_checked(&dir, 1, &allow)
            .unwrap();
        assert_eq!(kakis.len(), 3);

        let base = tmp_base("related");
        crate::readnode::materialize_now(
            &wn,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        let graph = TopicGraph::build_from_readnode(&mut rn).unwrap();

        // g1 and g3 both cite into "gates" exclusively -- g3 should find
        // g1 as a related neighbor (g2 also qualifies; both share the
        // only topic in this fixture).
        let g3 = kakis.iter().find(|k| graph.placement(k).is_some()).copied();
        assert!(
            g3.is_some(),
            "at least one document has a resolvable placement"
        );
        let related = graph.related(&g3.unwrap(), 5);
        assert!(!related.is_empty(), "same-topic neighbors should be found");
    }
}
