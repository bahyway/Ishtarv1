//! WriteNode — EnkiDDB's write path: real, append-only WAL.
//!
//! Mirrors EnkiDB's own CQRS split (port 7001's write node) exactly,
//! rather than inventing a new persistence scheme for documents. A
//! `DocumentEmitter` produces `enkidb_particles::Particle` rows;
//! `WriteNode` converts each into an `EavTriple` (via the real
//! `enkidb_ingest::bridge::particle_to_eav_triple`, the same bridge
//! EnkiDB's own ingest path uses) and appends one `JournalEntry` per
//! document to a real `enkidb_journal::Journal`. This Journal *is* the
//! WAL — `enkidb-readnode::materialize` reads it to build the Read
//! Node's Data Files (see `readnode.rs` in this crate).
//!
//! WriteNode never serves queries. Per ADR-012's Data Files Law, that
//! is the Read Node's job exclusively.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_ingest::bridge::particle_to_eav_triple;
use enkidb_journal::entry::JournalEntry;
use enkidb_journal::{EventCause, Journal};
use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
use enkidb_particles::Particle;

use crate::authorship::{check_authorship, TeamAllowlist};
use crate::concepts::ConceptRegistry;
use crate::document::DocumentStructure;
use crate::emitter::DocumentEmitter;
use crate::exposure::{ConceptGraph, ExposureReport};
use crate::ingest::{collect_markdown_files, infer_collection, IngestError};
use crate::links::discover_referenced_paths;
use crate::orbit::DocOrbit;
use crate::parser::DocumentParser;

/// EnkiDDB's write node: mints documents, journals their particles as one
/// EAV-bearing Event-Kaki per document. The Journal held here is the real
/// WAL — every document ingested is durable in it before this call returns.
pub struct WriteNode {
    journal: Journal,
    minter: KakiMinter,
    concept_graph: ConceptGraph,
}

impl WriteNode {
    /// `shard_count` matches `Journal::new`'s own partitioning parameter —
    /// see `enkidb-journal` for what it controls.
    pub fn new(minter: KakiMinter, shard_count: u16) -> Self {
        WriteNode {
            journal: Journal::new(shard_count),
            minter,
            concept_graph: ConceptGraph::new(),
        }
    }

    /// Ingest one parsed document: mint its Identity-Kaki, emit its
    /// particles, journal them as a single Event-Kaki-bearing entry at
    /// `epoch`. Returns the document's Identity-Kaki so the caller can
    /// look it up later (e.g. through the Read Node).
    pub fn ingest_document(&mut self, structure: &DocumentStructure, epoch: u32) -> IdentityKaki {
        let emitter = DocumentEmitter::new(&self.minter);
        let (doc_kaki, particles) = emitter.emit_document(structure);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(JournalEntry::new(event_kaki, doc_kaki, epoch, eav))
            .expect("append to an in-memory Journal is infallible in this shard config");

        doc_kaki
    }

    /// Records that `old` (an already-minted document Identity-Kaki, from
    /// an earlier `ingest_document`/`ingest_document_from_path` call --
    /// possibly on a different `WriteNode`/run) has been superseded by
    /// `new`, with `reason` explaining why. An APPEND on `old`'s own
    /// existing identity -- never a new mint, never a delete (ADR-006).
    /// This is the mechanism ADR-014 Decision 2 promised: "why did v4.5
    /// replace v4.4" becomes a queryable `hist.reason` on `old`, not lost
    /// history. Does not journal or validate `new` itself -- `new` must
    /// already exist (this only records the relationship between two
    /// already-real identities).
    pub fn supersede_document(
        &mut self,
        old: IdentityKaki,
        new: IdentityKaki,
        reason: &str,
        epoch: u32,
    ) {
        let emitter = DocumentEmitter::new(&self.minter);
        let particles = emitter.emit_supersession(old, new, reason);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, old, epoch, eav)
                    .with_event_cause(EventCause::DocumentSuperseded),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");
    }

    /// Emit a neutral cross-reference between two already-minted documents
    /// -- e.g. "found in location X" or "same PB number as Y, unreconciled"
    /// -- WITHOUT implying either side replaces or supersedes the other, as
    /// its OWN dedicated edge entity rather than a particle written onto
    /// `source`'s existing identity.
    ///
    /// FIXED (superseded an earlier version of this method that wrote
    /// `link.target`/`link.description` directly onto `source`'s own
    /// entity, one call per edge): a `source` with more than one outgoing
    /// edge -- exactly `anu_governor::pb_catalog`'s case, one location
    /// linked to every playbook found there -- silently lost all but the
    /// last-journaled edge the moment it was read back through the Read
    /// Node. Both `ReadNode::query` and `CachedReadNode::query` fold a
    /// matched entity's history into a single-slot `(attr_hash -> value)`
    /// map before projecting (`heptascript::engine::apply_entry_to_map`),
    /// so repeated `link.target`/`link.description` triples on one entity
    /// collapse to whichever was journaled last. `WriteNode::
    /// link_discovered_dependencies` hit and fixed the identical bug for
    /// citation edges earlier by minting one child entity per edge; this
    /// method applies the same fix here, generalized for any two already-
    /// minted documents rather than just a batch's own cross-references.
    ///
    /// Also carries `link.source_title`/`link.target_title` (plain Text,
    /// alongside the real `link.source`/`link.target` KakiPk values) so a
    /// caller can find every edge FROM a known title with a plain
    /// `WHERE E[link.source_title] = "..."` scan. This is not a
    /// convenience -- HeptaScript's WHERE clause has no way to compare a
    /// KakiPk attribute against a literal at all
    /// (`heptascript::engine::compare_val` has no `AkkValue::KakiPk` arm,
    /// falling through to `_ => false`), so the real `link.source`/
    /// `link.target` fields alone could never be queried back this way --
    /// only their Text-typed title twins can.
    ///
    /// Does not validate `target` -- `target` must already exist. Tagged
    /// `EventCause::DocumentCrossReferenced`, same as before.
    pub fn mint_link_edge(
        &mut self,
        source: IdentityKaki,
        source_title: &str,
        target: IdentityKaki,
        target_title: &str,
        description: &str,
        epoch: u32,
    ) -> IdentityKaki {
        let edge_kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Zikru))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let emitter = DocumentEmitter::new(&self.minter);
        let mut particles: Vec<Particle> =
            emitter.emit_link(edge_kaki, target, description).to_vec();
        particles.push(Particle::base(
            edge_kaki,
            DocOrbit::Link.attr("source"),
            AkkValue::KakiPk(*source.bytes()),
            now_secs(),
        ));
        particles.push(Particle::base(
            edge_kaki,
            DocOrbit::Meta.attr("title"),
            AkkValue::Text(format!("{source_title} § {description}")),
            now_secs(),
        ));
        particles.push(Particle::base(
            edge_kaki,
            DocOrbit::Link.attr("source_title"),
            AkkValue::Text(source_title.to_string()),
            now_secs(),
        ));
        particles.push(Particle::base(
            edge_kaki,
            DocOrbit::Link.attr("target_title"),
            AkkValue::Text(target_title.to_string()),
            now_secs(),
        ));
        particles.push(Particle::base(
            edge_kaki,
            DocOrbit::Hist.attr("event"),
            AkkValue::Text("BIRTH".to_string()),
            now_secs(),
        ));

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, edge_kaki, epoch, eav)
                    .with_event_cause(EventCause::DocumentCrossReferenced),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        edge_kaki
    }

    /// Mint a bare, title-only marker document -- no headers/body/code,
    /// just `meta.title` plus a `meta.collection` tag. Used for nodes that
    /// exist to be linked TO rather than to carry real document content of
    /// their own (e.g. `anu_governor::pb_catalog`'s one marker per source
    /// location, so a location becomes a real, navigable graph node in
    /// Graph Explorer rather than a plain string attribute).
    pub fn mint_marker(&mut self, title: &str, collection: &str, epoch: u32) -> IdentityKaki {
        let structure = DocumentStructure {
            title: title.to_string(),
            ..Default::default()
        };
        let emitter = DocumentEmitter::new(&self.minter);
        let (doc_kaki, mut particles) = emitter.emit_document(&structure);
        particles.push(emitter.emit_collection(doc_kaki, collection));
        Self::journal_document_and_sections(
            &mut self.journal,
            &emitter,
            doc_kaki,
            particles,
            &structure,
            epoch,
        )
        .0
    }

    /// Tag an already-minted document with its current `bahyway_core::
    /// hepta_gate::HeptaGate` classification (`gate_akkadian_name`, e.g.
    /// "ADAD") -- an APPEND on `doc`'s own existing identity, never a new
    /// mint. Unlike `mint_link_edge`, a single `meta.gate` particle per
    /// entity is the correct shape here: a document has exactly ONE
    /// current gate classification at a time (re-tagging is a real
    /// reclassification, not a second simultaneous fact), so
    /// last-write-wins (`heptascript::engine::apply_entry_to_map`) is the
    /// desired semantics, not a bug to work around.
    pub fn tag_gate(&mut self, doc: IdentityKaki, gate_akkadian_name: &str, epoch: u32) {
        let particles = [Particle::base(
            doc,
            DocOrbit::Meta.attr("gate"),
            AkkValue::Text(gate_akkadian_name.to_string()),
            now_secs(),
        )];
        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");
        self.journal
            .append(JournalEntry::new(event_kaki, doc, epoch, eav))
            .expect("append to an in-memory Journal is infallible in this shard config");
    }

    /// Tag an already-minted document with its current sub-classification
    /// WITHIN its gate (`domain_name`, e.g. "Extraction" under ADAD) --
    /// same shape and same reasoning as `tag_gate`: a single `meta.domain`
    /// particle per entity, last-write-wins is the correct semantics for
    /// a single current classification. The domain taxonomy itself
    /// (7 names per gate) is `anu_governor`'s own concern, not a sealed
    /// ecosystem-wide concept like `HeptaGate` -- this method only ever
    /// writes whatever domain name string it's given.
    pub fn tag_domain(&mut self, doc: IdentityKaki, domain_name: &str, epoch: u32) {
        let particles = [Particle::base(
            doc,
            DocOrbit::Meta.attr("domain"),
            AkkValue::Text(domain_name.to_string()),
            now_secs(),
        )];
        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");
        self.journal
            .append(JournalEntry::new(event_kaki, doc, epoch, eav))
            .expect("append to an in-memory Journal is infallible in this shard config");
    }

    /// Ingest one document the "best existing way": mint the parent
    /// document exactly as `ingest_document` does, tag it with a
    /// `meta.collection` particle (the document-KIND taxonomy, orthogonal
    /// to `DocOrbit`), then mint and journal one child section per
    /// `DocumentStructure::sections()` boundary -- each carrying its own
    /// `body.summary` (RAG key) and `body.text` (RAG value, the section's
    /// full untouched text), linked back to the parent document. Every
    /// section is journaled as its own entry, so `enkiddb::rag::RagIndex`
    /// can scan the Journal directly without a separate section registry.
    /// Returns the parent document's Identity-Kaki plus the real total
    /// particle (EAV triple) count journaled for it (document + every
    /// section) -- the server's single-document wire command surfaces this
    /// so a caller can report actual EAV volume, not a file count.
    pub fn ingest_document_categorized(
        &mut self,
        structure: &DocumentStructure,
        epoch: u32,
        collection: &str,
    ) -> (IdentityKaki, usize) {
        let emitter = DocumentEmitter::new(&self.minter);
        let (doc_kaki, mut particles) = emitter.emit_document(structure);
        particles.push(emitter.emit_collection(doc_kaki, collection));
        Self::journal_document_and_sections(
            &mut self.journal,
            &emitter,
            doc_kaki,
            particles,
            structure,
            epoch,
        )
    }

    /// Re-journal a document's FULL particle set (title/headers/body/code/
    /// sections, same shape `ingest_document_categorized` produces) under
    /// an ALREADY-KNOWN `doc_kaki`, instead of minting a fresh identity.
    ///
    /// For a caller that deliberately reuses one stable identity for
    /// content it has seen before (content-hash dedup, e.g.
    /// `anu_governor::pb_catalog`) -- see `DocumentEmitter::emit_document_for`'s
    /// own doc comment for why skipping this step silently loses that
    /// document's content the moment a NEW generation is materialized and
    /// promoted, even though its identity and any fresh links to it still
    /// exist.
    pub fn reingest_document_categorized(
        &mut self,
        doc_kaki: IdentityKaki,
        structure: &DocumentStructure,
        epoch: u32,
        collection: &str,
    ) -> IdentityKaki {
        let emitter = DocumentEmitter::new(&self.minter);
        let mut particles = emitter.emit_document_for(doc_kaki, structure);
        particles.push(emitter.emit_collection(doc_kaki, collection));
        Self::journal_document_and_sections(
            &mut self.journal,
            &emitter,
            doc_kaki,
            particles,
            structure,
            epoch,
        )
        .0
    }

    /// Ingest a document exactly as `ingest_document_categorized` does,
    /// plus scan its full text (title, headers, body, code) against
    /// `registry` for internal-concept mentions, journaling each hit as
    /// its own linked entity (`DocumentEmitter::emit_concept_mentions`).
    /// This is what feeds `exposure_preview` — an access-grant exposure
    /// preview needs to know what a collection's documents actually
    /// mention, and that can only come from documents ingested through
    /// this method (older calls to `ingest_document_categorized`/
    /// `ingest_document_from_path` are unaffected and unscanned).
    pub fn ingest_document_with_concepts(
        &mut self,
        structure: &DocumentStructure,
        epoch: u32,
        collection: &str,
        registry: &ConceptRegistry,
    ) -> IdentityKaki {
        let emitter = DocumentEmitter::new(&self.minter);
        let (doc_kaki, mut particles) = emitter.emit_document(structure);
        particles.push(emitter.emit_collection(doc_kaki, collection));
        let (doc_kaki, _doc_and_section_particle_count) = Self::journal_document_and_sections(
            &mut self.journal,
            &emitter,
            doc_kaki,
            particles,
            structure,
            epoch,
        );

        let mentions = registry.scan_mentions(&document_full_text(structure));
        for (mention_kaki, mention_particles) in emitter.emit_concept_mentions(doc_kaki, &mentions)
        {
            let eav = mention_particles
                .iter()
                .map(particle_to_eav_triple)
                .collect();
            let event = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
                .expect("KakiMinter::event always mints kaki_type=Event");
            self.journal
                .append(JournalEntry::new(event, mention_kaki, epoch, eav))
                .expect("append to an in-memory Journal is infallible in this shard config");
        }

        self.concept_graph
            .record_document(doc_kaki, &structure.title, collection, &mentions);
        doc_kaki
    }

    /// The exposure preview for one `meta.collection`: which documents
    /// (ingested via `ingest_document_with_concepts`) it contains, and
    /// which other internal concepts they mention — what granting a
    /// stakeholder group access to it would actually reveal.
    pub fn exposure_preview(&self, collection: &str) -> ExposureReport {
        self.concept_graph.exposure_preview(collection)
    }

    /// Every collection tag any concept-scanned document has been tagged
    /// with so far.
    pub fn known_collections(&self) -> Vec<String> {
        self.concept_graph.known_collections()
    }

    /// Read one Markdown file straight off disk, categorize it by
    /// [`crate::ingest::infer_collection`] on its path, and journal it plus
    /// every one of its sections -- the single-file version of
    /// [`WriteNode::ingest_directory_categorized`]. Adds a `meta.source_path`
    /// particle (matching `ingest::ingest_directory`'s convention) so every
    /// journaled document stays traceable to the file it came from.
    pub fn ingest_document_from_path(
        &mut self,
        path: &Path,
        epoch: u32,
    ) -> Result<IdentityKaki, IngestError> {
        let text = fs::read_to_string(path)?;

        let scan = crate::security::scan_document(text.as_bytes());
        if !scan.clean {
            return Err(IngestError::SecurityRejected {
                path: path.to_path_buf(),
                detail: scan.detail.to_string(),
            });
        }

        let structure = DocumentParser::parse_markdown(&text);
        let collection = infer_collection(path);

        let emitter = DocumentEmitter::new(&self.minter);
        let (doc_kaki, mut particles) = emitter.emit_document(&structure);
        particles.push(emitter.emit_collection(doc_kaki, &collection));
        particles.push(Particle::base(
            doc_kaki,
            DocOrbit::Meta.attr("source_path"),
            AkkValue::Text(path.display().to_string()),
            now_secs(),
        ));

        Ok(Self::journal_document_and_sections(
            &mut self.journal,
            &emitter,
            doc_kaki,
            particles,
            &structure,
            epoch,
        )
        .0)
    }

    /// Walk `root` for every `.md` file (recursively, same walk
    /// `ingest::ingest_directory` uses) and journal each one, categorized
    /// and chunked, via [`WriteNode::ingest_document_from_path`]. This is
    /// the practical "upload my documentation" entry point: after this
    /// call, `readnode::materialize_version` can turn the result into a
    /// queryable Tigris generation, and `rag::RagIndex::build(wn.journal())`
    /// can search it. Each file gets its own epoch (`start_epoch + i`) so
    /// documents stay distinguishable by journal partition/order.
    pub fn ingest_directory_categorized(
        &mut self,
        root: &Path,
        start_epoch: u32,
    ) -> Result<Vec<IdentityKaki>, IngestError> {
        let mut paths = Vec::new();
        collect_markdown_files(root, &mut paths)?;
        paths
            .into_iter()
            .enumerate()
            .map(|(i, path)| self.ingest_document_from_path(&path, start_epoch + i as u32))
            .collect()
    }

    /// Same as [`WriteNode::ingest_directory_categorized`], but gated by
    /// the Architect's authorship policy: every file's last git commit
    /// author must be on `allowlist` (see [`crate::authorship`]), checked
    /// *before* any file is journaled. Fails closed and aborts the whole
    /// call at the first unauthorized or undeterminable-author file
    /// (`IngestError::UnauthorizedCreator`) rather than silently skipping
    /// just that one -- a directory containing one unexpected source
    /// should not quietly ingest the rest around it. Callers that want a
    /// per-file report before committing to this call should scan +
    /// check each path themselves first (`crate::authorship::check_authorship`
    /// over `crate::ingest::scan_markdown_directory`'s output) -- that is
    /// exactly what `enkiddb-cli`'s "categorize" stage does.
    pub fn ingest_directory_categorized_checked(
        &mut self,
        root: &Path,
        start_epoch: u32,
        allowlist: &TeamAllowlist,
    ) -> Result<Vec<IdentityKaki>, IngestError> {
        let mut paths = Vec::new();
        collect_markdown_files(root, &mut paths)?;

        for path in &paths {
            let check = check_authorship(path, allowlist);
            if !check.authorized {
                return Err(IngestError::UnauthorizedCreator {
                    path: path.clone(),
                    author: check.author,
                });
            }
        }

        let mut kakis = Vec::with_capacity(paths.len());
        let mut by_stem: HashMap<String, IdentityKaki> = HashMap::new();
        let mut texts: Vec<(IdentityKaki, String)> = Vec::with_capacity(paths.len());

        for (i, path) in paths.iter().enumerate() {
            let kaki = self.ingest_document_from_path(path, start_epoch + i as u32)?;
            kakis.push(kaki);
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                by_stem.insert(stem.to_string(), kaki);
            }
            if let Ok(text) = fs::read_to_string(path) {
                texts.push((kaki, text));
            }
        }

        self.link_discovered_dependencies(&texts, &by_stem, start_epoch + paths.len() as u32);

        Ok(kakis)
    }

    /// Second pass over a just-ingested batch: scan each document's raw
    /// text for path-shaped references to other files
    /// ([`crate::links::discover_referenced_paths`]), resolve each
    /// reference against the OTHER documents this same call just
    /// ingested (matched by file stem -- markdown link targets like
    /// `./scripts/deploy.sh` rarely match a `PathBuf` exactly, but the
    /// bare file name reliably does), and journal one child edge-entity
    /// per match, carrying `link.target` (the cited document),
    /// `link.source` (the citing document), and `link.description` =
    /// "depends-on".
    ///
    /// FIXED: this used to write `link.target`/`link.description`
    /// directly onto the *citing* document's own entity (one
    /// `DocumentEmitter::emit_link` per match, same entity every time).
    /// A document citing more than one other document then silently lost
    /// all but the last-journaled citation the moment it was read back
    /// through the Read Node: both `ReadNode::query` and
    /// `CachedReadNode::query` fold a matched entity's history into a
    /// single-slot `(attr_hash -> value)` map before projecting
    /// (`heptascript::engine::apply_entry_to_map`), so repeated
    /// `link.target` triples on one entity collapse to whichever one was
    /// journaled last -- the raw bytes survived in `entities.data`, but
    /// were never queryable as more than one edge. Nothing in this crate
    /// queried `depends-on` edges through the Read Node yet (only
    /// `wn.journal().all_entries()` directly, in this file's own tests),
    /// so this was a real, latent, previously-undetected data-loss bug at
    /// the exact write it now no longer collapses. Minting one child
    /// entity per edge (the same pattern `emit_sections`'s "section-of"
    /// and `emit_concept_mentions`'s "mentioned-in" edges already use)
    /// sidesteps the collision entirely: every edge is its own entity, so
    /// no attribute name is ever written twice on the same entity.
    /// Unresolved references (the referenced file wasn't ingested in this
    /// batch, or doesn't exist) are silently skipped -- this is a
    /// best-effort cross-reference, not a validation gate; a broken link
    /// in someone's prose should never block ingestion of real content.
    fn link_discovered_dependencies(
        &mut self,
        texts: &[(IdentityKaki, String)],
        by_stem: &HashMap<String, IdentityKaki>,
        epoch: u32,
    ) {
        let emitter = DocumentEmitter::new(&self.minter);
        for (doc_kaki, text) in texts {
            for referenced_path in discover_referenced_paths(text) {
                let Some(stem) = Path::new(&referenced_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                else {
                    continue;
                };
                let Some(&target_kaki) = by_stem.get(stem) else {
                    continue;
                };
                if target_kaki == *doc_kaki {
                    continue;
                }

                let edge_kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Zikru))
                    .expect("KakiMinter::identity always mints kaki_type=Identity");
                let mut particles: Vec<Particle> = emitter
                    .emit_link(edge_kaki, target_kaki, "depends-on")
                    .to_vec();
                particles.push(Particle::base(
                    edge_kaki,
                    DocOrbit::Link.attr("source"),
                    AkkValue::KakiPk(*doc_kaki.bytes()),
                    now_secs(),
                ));
                particles.push(Particle::base(
                    edge_kaki,
                    DocOrbit::Hist.attr("event"),
                    AkkValue::Text("BIRTH".to_string()),
                    now_secs(),
                ));

                let eav = particles.iter().map(particle_to_eav_triple).collect();
                let event = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
                    .expect("KakiMinter::event always mints kaki_type=Event");
                self.journal
                    .append(JournalEntry::new(event, edge_kaki, epoch, eav))
                    .expect("append to an in-memory Journal is infallible in this shard config");
            }
        }
    }

    /// Shared tail of every ingestion path: journal the parent document's
    /// already-built particles, then mint + journal one entry per section.
    /// Takes `journal`/`emitter` as explicit params (rather than `&mut
    /// self`) so callers can hold `emitter`'s immutable borrow of
    /// `self.minter` alive across the call while mutating `self.journal`
    /// -- a disjoint-field borrow the compiler can't see through `&mut self`.
    ///
    /// Returns the document's Identity-Kaki plus the REAL total particle
    /// (EAV triple) count journaled for it -- the parent document's own
    /// particles plus every section's, not a file/document count. This is
    /// what lets a caller report actual EAV volume instead of assuming
    /// "1 file = 1 particle" (that assumption undercounts real ingestion
    /// by 1-2 orders of magnitude once a document's sections are counted).
    fn journal_document_and_sections(
        journal: &mut Journal,
        emitter: &DocumentEmitter,
        doc_kaki: IdentityKaki,
        particles: Vec<Particle>,
        structure: &DocumentStructure,
        epoch: u32,
    ) -> (IdentityKaki, usize) {
        let mut particle_count = particles.len();
        let doc_eav = particles.iter().map(particle_to_eav_triple).collect();
        let doc_event = EventKaki::try_from_kaki(emitter.minter().event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");
        journal
            .append(JournalEntry::new(doc_event, doc_kaki, epoch, doc_eav))
            .expect("append to an in-memory Journal is infallible in this shard config");

        for (section_kaki, section_particles) in
            emitter.emit_sections(doc_kaki, &structure.title, structure)
        {
            particle_count += section_particles.len();
            let section_eav = section_particles
                .iter()
                .map(particle_to_eav_triple)
                .collect();
            let section_event = EventKaki::try_from_kaki(emitter.minter().event(KakiRole::Zikru))
                .expect("KakiMinter::event always mints kaki_type=Event");
            journal
                .append(JournalEntry::new(
                    section_event,
                    section_kaki,
                    epoch,
                    section_eav,
                ))
                .expect("append to an in-memory Journal is infallible in this shard config");
        }

        (doc_kaki, particle_count)
    }

    /// The underlying Journal — read-only access, for materializing into
    /// the Read Node's Data Files (`readnode::materialize_now`) or for
    /// direct full-scan queries the Read Node can't serve (historical
    /// `WHEN`, `OR` conditions — see `enkidb-readnode::ReadNodeError`).
    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    pub fn document_count(&self) -> usize {
        self.journal.all_entries().count()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// The document's title, every header's text, every paragraph, and every
/// code block's source, concatenated — the full text surface
/// `ConceptRegistry::scan_mentions` scans for internal-concept mentions.
/// Built directly from `DocumentStructure`'s public fields rather than
/// via `DocumentEmitter`'s private per-section text helper, since this
/// needs the whole document at once, not section by section.
fn document_full_text(structure: &DocumentStructure) -> String {
    let mut out = structure.title.clone();
    for h in &structure.headers {
        out.push('\n');
        out.push_str(&h.text);
    }
    for b in &structure.body {
        out.push('\n');
        out.push_str(&b.content);
    }
    for c in &structure.code_blocks {
        out.push('\n');
        out.push_str(&c.code);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concepts::{ConceptKind, ConceptRegistry};
    use crate::parser::DocumentParser;
    use bahyway_core::TribeId;

    fn write_node() -> WriteNode {
        WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF07)), 64)
    }

    #[test]
    fn ingest_document_with_concepts_populates_the_exposure_preview() {
        let mut wn = write_node();
        let registry = ConceptRegistry::new()
            .with_known_gates()
            .extend_crates(["enkiddb".to_string()]);

        // G1=APSU and G7=ENLIL per the sealed bahyway_core::hepta_gate::
        // HeptaGate ruling (GATE-1, 2026-06-18/07-07).
        let g1_doc = DocumentParser::parse_markdown(
            "# Gate G1 Runbook\n\nThe Apsu gate is the identity/security intake, handled by enkiddb.\n",
        );
        let g7_doc = DocumentParser::parse_markdown(
            "# Gate G7 Runbook\n\nThe Enlil gate governs SLA sign-off.\n",
        );
        let unrelated =
            DocumentParser::parse_markdown("# Glossary\n\nJust background text, no gates here.\n");

        wn.ingest_document_with_concepts(&g1_doc, 1, "gates", &registry);
        wn.ingest_document_with_concepts(&g7_doc, 2, "gates", &registry);
        wn.ingest_document_with_concepts(&unrelated, 3, "glossary", &registry);

        assert_eq!(
            wn.known_collections(),
            vec!["gates".to_string(), "glossary".to_string()]
        );

        let report = wn.exposure_preview("gates");
        assert_eq!(report.document_count, 2);
        let names: Vec<&str> = report
            .revealed_concepts
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert!(names.contains(&"APSU"));
        assert!(names.contains(&"ENLIL"));
        assert!(names.contains(&"enkiddb"));
        assert!(report
            .revealed_concepts
            .iter()
            .any(|c| c.kind == ConceptKind::Gate));
        assert!(report.narrate().contains("2 document"));
    }

    #[test]
    fn concept_mentions_are_journaled_as_their_own_linked_entries() {
        let mut wn = write_node();
        let registry = ConceptRegistry::new().with_known_gates();
        let doc = DocumentParser::parse_markdown("# Doc\n\nMentions the Enlil gate once.\n");

        let doc_kaki = wn.ingest_document_with_concepts(&doc, 1, "gates", &registry);

        // Document entry + 1 section entry + 1 concept-mention entity = 3.
        assert_eq!(wn.document_count(), 3);
        let doc_history = wn.journal().read_particle_history(&doc_kaki);
        assert_eq!(
            doc_history.len(),
            1,
            "the parent document itself is still one entry"
        );
    }

    #[test]
    fn ingest_document_journals_a_real_entry() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown("# Title\n\nA paragraph.\n");
        let kaki = wn.ingest_document(&doc, 1);

        assert_eq!(wn.document_count(), 1);
        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history.len(), 1);
        assert!(
            !history[0].eav.is_empty(),
            "the journaled entry must carry the document's EAV triples"
        );
    }

    #[test]
    fn supersede_document_appends_to_the_old_identitys_own_history_without_minting_a_new_one() {
        let mut wn = write_node();
        let v44 = DocumentParser::parse_markdown("# v4.4\n");
        let v45 = DocumentParser::parse_markdown("# v4.5\n");
        let kaki_v44 = wn.ingest_document(&v44, 1);
        let kaki_v45 = wn.ingest_document(&v45, 2);
        assert_eq!(
            wn.document_count(),
            2,
            "two real documents, no supersession yet"
        );

        wn.supersede_document(
            kaki_v44,
            kaki_v45,
            "v4.5 adds the KAKI-minting stage v4.4 lacked",
            3,
        );

        // Supersession is an APPEND on the OLD identity, not a new mint --
        // document_count (distinct entries) grows to 3 (v4.4's own BIRTH
        // entry plus this new SUPERSEDED entry, v4.5's BIRTH entry), but
        // no THIRD identity exists: read_particle_history(kaki_v44) now
        // shows two entries under the SAME identity.
        assert_eq!(wn.document_count(), 3);
        let v44_history = wn.journal().read_particle_history(&kaki_v44);
        assert_eq!(
            v44_history.len(),
            2,
            "v4.4's own identity now has a BIRTH entry and a SUPERSEDED entry"
        );

        let has_superseded_event = v44_history.iter().flat_map(|e| e.eav.iter()).any(|t| {
            t.attr_hash == enkidb_ingest::bridge::attr_hash("hist.event")
                && matches!(
                    enkidb_ingest::bridge::eav_triple_to_value(t),
                    AkkValue::Text(ref s) if s == "SUPERSEDED"
                )
        });
        assert!(
            has_superseded_event,
            "the new entry must carry hist.event = SUPERSEDED"
        );

        let has_reason = v44_history.iter().flat_map(|e| e.eav.iter()).any(|t| {
            t.attr_hash == enkidb_ingest::bridge::attr_hash("hist.reason")
                && matches!(
                    enkidb_ingest::bridge::eav_triple_to_value(t),
                    AkkValue::Text(ref s) if s == "v4.5 adds the KAKI-minting stage v4.4 lacked"
                )
        });
        assert!(
            has_reason,
            "the WHY must be queryable, not just the fact that a supersession happened"
        );

        let has_link_to_new = v44_history.iter().flat_map(|e| e.eav.iter()).any(|t| {
            t.attr_hash == enkidb_ingest::bridge::attr_hash("hist.superseded_by")
                && matches!(
                    enkidb_ingest::bridge::eav_triple_to_value(t),
                    AkkValue::KakiPk(bytes) if bytes == *kaki_v45.bytes()
                )
        });
        assert!(
            has_link_to_new,
            "must link forward to the NEW document's identity"
        );
    }

    #[test]
    fn tag_gate_writes_a_queryable_meta_gate_particle_on_the_documents_own_identity() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown("# A Playbook\n");
        let kaki = wn.ingest_document(&doc, 1);

        wn.tag_gate(kaki, "ADAD", 2);

        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(
            history.len(),
            2,
            "BIRTH entry plus the new gate-tag entry, same identity"
        );
        let has_gate_tag = history.iter().flat_map(|e| e.eav.iter()).any(|t| {
            t.attr_hash == enkidb_ingest::bridge::attr_hash("meta.gate")
                && matches!(
                    enkidb_ingest::bridge::eav_triple_to_value(t),
                    AkkValue::Text(ref s) if s == "ADAD"
                )
        });
        assert!(
            has_gate_tag,
            "meta.gate must carry the gate's Akkadian name as plain Text"
        );
    }

    #[test]
    fn tag_domain_writes_a_queryable_meta_domain_particle_on_the_documents_own_identity() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown("# A Playbook\n");
        let kaki = wn.ingest_document(&doc, 1);

        wn.tag_domain(kaki, "Extraction", 2);

        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(
            history.len(),
            2,
            "BIRTH entry plus the new domain-tag entry, same identity"
        );
        let has_domain_tag = history.iter().flat_map(|e| e.eav.iter()).any(|t| {
            t.attr_hash == enkidb_ingest::bridge::attr_hash("meta.domain")
                && matches!(
                    enkidb_ingest::bridge::eav_triple_to_value(t),
                    AkkValue::Text(ref s) if s == "Extraction"
                )
        });
        assert!(
            has_domain_tag,
            "meta.domain must carry the domain name as plain Text"
        );
    }

    #[test]
    fn reingest_document_categorized_journals_the_same_content_under_the_same_identity() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown("# Stable Title\n\nSame body every time.\n");
        let (kaki, _particle_count) =
            wn.ingest_document_categorized(&doc, 1, "playbook-record-candidate");

        let kaki2 = wn.reingest_document_categorized(kaki, &doc, 2, "playbook-record-candidate");
        assert_eq!(
            kaki, kaki2,
            "must reuse the SAME identity, never mint a new one"
        );

        // Two BIRTH-shaped entries now exist under the one identity (epoch
        // 1 and epoch 2), each independently carrying the real title --
        // proving the content was actually re-journaled, not just skipped.
        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history.len(), 2);
        for entry in &history {
            let has_title = entry.eav.iter().any(|t| {
                t.attr_hash == enkidb_ingest::bridge::attr_hash("meta.title")
                    && matches!(
                        enkidb_ingest::bridge::eav_triple_to_value(t),
                        AkkValue::Text(ref s) if s == "Stable Title"
                    )
            });
            assert!(
                has_title,
                "every re-journaled entry must carry the real title, not just a bare append"
            );
        }
    }

    #[test]
    fn mint_link_edge_gives_each_edge_its_own_entity_so_multiple_edges_from_one_source_all_survive()
    {
        let mut wn = write_node();
        let location = DocumentParser::parse_markdown("# Location\n");
        let loc_kaki = wn.ingest_document(&location, 1);
        let pb_a = DocumentParser::parse_markdown("# PB A\n");
        let pb_b = DocumentParser::parse_markdown("# PB B\n");
        let kaki_a = wn.ingest_document(&pb_a, 2);
        let kaki_b = wn.ingest_document(&pb_b, 3);

        wn.mint_link_edge(loc_kaki, "Location", kaki_a, "PB A", "contains:a", 4);
        wn.mint_link_edge(loc_kaki, "Location", kaki_b, "PB B", "contains:b", 5);

        // Two DISTINCT edge entities, each carrying its own target --
        // neither collapses into the other the way writing both directly
        // onto `loc_kaki`'s own identity would have (see this method's
        // own doc comment and `apply_entry_to_map`'s last-write-wins fold).
        let target_titles: Vec<String> = wn
            .journal()
            .all_entries()
            .filter_map(|e| {
                e.eav
                    .iter()
                    .find(|t| t.attr_hash == enkidb_ingest::bridge::attr_hash("link.target_title"))
                    .map(|t| match enkidb_ingest::bridge::eav_triple_to_value(t) {
                        AkkValue::Text(s) => s,
                        _ => String::new(),
                    })
            })
            .collect();
        assert_eq!(
            target_titles.len(),
            2,
            "both edges must survive as distinct entities: {target_titles:?}"
        );
        assert!(target_titles.contains(&"PB A".to_string()));
        assert!(target_titles.contains(&"PB B".to_string()));
    }

    #[test]
    fn mint_link_edge_carries_a_text_source_title_for_reverse_lookup() {
        let mut wn = write_node();
        let doc_a = DocumentParser::parse_markdown("# A\n");
        let doc_b = DocumentParser::parse_markdown("# B\n");
        let kaki_a = wn.ingest_document(&doc_a, 1);
        let kaki_b = wn.ingest_document(&doc_b, 2);

        let edge_kaki = wn.mint_link_edge(kaki_a, "A", kaki_b, "B", "contains:b", 3);

        let history = wn.journal().read_particle_history(&edge_kaki);
        assert_eq!(history.len(), 1);
        let has_source_title = history[0].eav.iter().any(|t| {
            t.attr_hash == enkidb_ingest::bridge::attr_hash("link.source_title")
                && matches!(
                    enkidb_ingest::bridge::eav_triple_to_value(t),
                    AkkValue::Text(ref s) if s == "A"
                )
        });
        assert!(
            has_source_title,
            "must carry link.source_title as plain Text -- KakiPk attrs can't be compared in a WHERE clause \
             (heptascript::engine::compare_val has no AkkValue::KakiPk arm)"
        );
    }

    #[test]
    fn ingesting_two_documents_mints_two_distinct_identities() {
        let mut wn = write_node();
        let a = DocumentParser::parse_markdown("# A\n");
        let b = DocumentParser::parse_markdown("# B\n");
        let kaki_a = wn.ingest_document(&a, 1);
        let kaki_b = wn.ingest_document(&b, 2);

        assert_ne!(kaki_a.bytes(), kaki_b.bytes());
        assert_eq!(wn.document_count(), 2);
    }

    #[test]
    fn eav_triples_survive_the_particle_to_journal_bridge() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown("# Title\n\nBody text.\n");
        let kaki = wn.ingest_document(&doc, 5);

        let history = wn
            .journal()
            .read_particle_history(&IdentityKaki::try_from_kaki(*kaki.kaki()).unwrap());
        assert_eq!(history[0].epoch, 5);
        // meta.title + body.paragraph + body.order + hist.event = 4 triples
        assert_eq!(history[0].eav.len(), 4);
    }

    #[test]
    fn ingest_document_categorized_journals_parent_and_every_section() {
        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown(
            "# Guide\n\n## Alpha\n\nAlpha body text here.\n\n## Beta\n\nBeta body text here.\n",
        );
        let (parent, particle_count) = wn.ingest_document_categorized(&doc, 1, "component");

        // Parent entry carries the meta.collection triple.
        let parent_history = wn.journal().read_particle_history(&parent);
        assert_eq!(parent_history.len(), 1);
        assert!(!parent_history[0].eav.is_empty());

        // Two sections (Alpha, Beta) were also journaled as their own
        // entries -- three entries total (parent + 2 sections).
        assert_eq!(wn.journal().entry_count(), 3);

        // particle_count is the REAL sum of every EAV triple journaled for
        // this document -- parent's + both sections', not an entry count.
        // The journal is fresh for this test (write_node()), so summing
        // every entry's EAV triples covers exactly this one ingest call.
        let real_total: usize = wn.journal().all_entries().map(|e| e.eav.len()).sum();
        assert_eq!(
            particle_count, real_total,
            "reported particle_count must equal the real sum of EAV triples \
             across the parent + every section entry"
        );
        assert!(real_total > 0);
    }

    #[test]
    fn section_entries_carry_summary_and_full_text_distinct_from_parent() {
        use enkidb_ingest::bridge::{attr_hash, eav_triple_to_value};

        let mut wn = write_node();
        let doc = DocumentParser::parse_markdown("# Guide\n\n## Only Section\n\nSome body text.\n");
        let (parent, _particle_count) = wn.ingest_document_categorized(&doc, 1, "component");

        let summary_hash = attr_hash("body.summary");
        let text_hash = attr_hash("body.text");

        // The parent entry has neither -- those are section-only triples.
        let parent_history = wn.journal().read_particle_history(&parent);
        assert!(!parent_history[0]
            .eav
            .iter()
            .any(|t| t.attr_hash == summary_hash));

        // Some other journal entry (the section) carries both, and its
        // body.text contains the original paragraph verbatim.
        let section_entry = wn
            .journal()
            .all_entries()
            .find(|e| e.eav.iter().any(|t| t.attr_hash == text_hash))
            .expect("a section entry must carry body.text");
        let text_triple = section_entry
            .eav
            .iter()
            .find(|t| t.attr_hash == text_hash)
            .unwrap();
        match eav_triple_to_value(text_triple) {
            akkvalue::AkkValue::Text(t) => assert!(t.contains("Some body text.")),
            _ => panic!("expected Text value"),
        }
    }

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_writenode_ingest_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn ingest_document_from_path_journals_the_file_categorized_and_chunked() {
        let dir = scratch_dir("from_path");
        let file = dir.join("widget.md");
        std::fs::write(&file, "# Widget\n\n## Usage\n\nCall it like this.\n").unwrap();

        let mut wn = write_node();
        let kaki = wn.ingest_document_from_path(&file, 1).unwrap();

        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history.len(), 1);
        assert!(history[0]
            .eav
            .iter()
            .any(|t| t.attr_hash == enkidb_ingest::bridge::attr_hash("meta.source_path")));
        assert!(history[0]
            .eav
            .iter()
            .any(|t| t.attr_hash == enkidb_ingest::bridge::attr_hash("meta.collection")));
        // parent + one section (Usage)
        assert_eq!(wn.journal().entry_count(), 2);
    }

    #[test]
    fn ingest_document_from_path_rejects_a_malware_signature_and_journals_nothing() {
        let dir = scratch_dir("security_reject");
        let file = dir.join("dropper.md");
        std::fs::write(&file, "# Doc\n\nEICAR-STANDARD-ANTIVIRUS-TEST-FILE\n").unwrap();

        let mut wn = write_node();
        let err = wn.ingest_document_from_path(&file, 1).unwrap_err();

        assert!(matches!(err, IngestError::SecurityRejected { .. }));
        assert_eq!(wn.journal().entry_count(), 0);
    }

    #[test]
    fn ingest_directory_categorized_journals_every_markdown_file_found() {
        let dir = scratch_dir("directory");
        std::fs::create_dir_all(dir.join("docs/components")).unwrap();
        std::fs::write(
            dir.join("docs/components/alpha.md"),
            "# Alpha\n\n## One\n\nBody.\n",
        )
        .unwrap();
        std::fs::write(dir.join("root.md"), "# Root\n\nJust a paragraph.\n").unwrap();
        std::fs::write(dir.join("ignore.txt"), "not markdown").unwrap();

        let mut wn = write_node();
        let kakis = wn.ingest_directory_categorized(&dir, 1).unwrap();

        assert_eq!(kakis.len(), 2, "two .md files, ignore.txt skipped");
        // alpha.md: parent + 1 section. root.md: parent + 1 preamble section (no headers).
        assert_eq!(wn.journal().entry_count(), 4);
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

    fn git_scratch_dir(name: &str, author_email: &str) -> std::path::PathBuf {
        let dir = scratch_dir(name);
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", author_email]);
        git(&dir, &["config", "user.name", "Test Author"]);
        dir
    }

    fn git_commit_all(dir: &std::path::Path) {
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "test commit"]);
    }

    #[test]
    fn ingest_directory_categorized_checked_ingests_when_every_author_is_allowed() {
        let dir = git_scratch_dir("checked_ok", "bahaa.fadam@gmail.com");
        std::fs::write(dir.join("root.md"), "# Root\n\nBody.\n").unwrap();
        git_commit_all(&dir);

        let mut wn = write_node();
        let allow = crate::authorship::TeamAllowlist::seed();
        let kakis = wn
            .ingest_directory_categorized_checked(&dir, 1, &allow)
            .unwrap();

        assert_eq!(kakis.len(), 1, "one file -> one document identity");
        // parent + 1 preamble section (no headers in "Body."), matching
        // ingest_directory_categorized_journals_every_markdown_file_found's
        // established root.md shape.
        assert_eq!(wn.document_count(), 2);
    }

    #[test]
    fn ingest_directory_categorized_checked_rejects_an_unauthorized_author_and_ingests_nothing() {
        let dir = git_scratch_dir("checked_reject", "outsider@example.com");
        std::fs::write(dir.join("root.md"), "# Root\n\nBody.\n").unwrap();
        git_commit_all(&dir);

        let mut wn = write_node();
        let allow = crate::authorship::TeamAllowlist::seed();
        let err = wn
            .ingest_directory_categorized_checked(&dir, 1, &allow)
            .unwrap_err();

        match err {
            IngestError::UnauthorizedCreator { author, .. } => {
                assert_eq!(author.as_deref(), Some("outsider@example.com"));
            }
            _ => panic!("expected UnauthorizedCreator"),
        }
        assert_eq!(
            wn.document_count(),
            0,
            "nothing journaled -- the whole batch is gated, not just the bad file"
        );
    }

    #[test]
    fn ingest_directory_categorized_checked_links_a_discovered_dependency_reference() {
        let dir = git_scratch_dir("checked_links", "bahaa.fadam@gmail.com");
        std::fs::write(
            dir.join("guide.md"),
            "# Guide\n\nSee [the deploy script](./deploy.sh.md) for details.\n",
        )
        .unwrap();
        // Real .md files only (collect_markdown_files only walks .md) --
        // name the "referenced script" with a .md extension so it's both
        // a real ingestable document AND a valid discover_referenced_paths
        // match, proving the two features compose without inventing a
        // second file-type ingest path just for this test.
        std::fs::write(
            dir.join("deploy.sh.md"),
            "# Deploy Script Notes\n\nRun steps here.\n",
        )
        .unwrap();
        git_commit_all(&dir);

        let mut wn = write_node();
        let allow = crate::authorship::TeamAllowlist::seed();
        let kakis = wn
            .ingest_directory_categorized_checked(&dir, 1, &allow)
            .unwrap();
        assert_eq!(kakis.len(), 2);

        // guide.md's own preamble section also mints a "section-of" link
        // back to its parent -- search specifically for the "depends-on"
        // description, not just any link entry, so this doesn't pass for
        // the wrong reason.
        let has_depends_on_link = wn.journal().all_entries().any(|e| {
            e.eav.iter().any(|t| {
                t.attr_hash == enkidb_ingest::bridge::attr_hash("link.description")
                    && matches!(
                        enkidb_ingest::bridge::eav_triple_to_value(t),
                        akkvalue::AkkValue::Text(ref s) if s == "depends-on"
                    )
            })
        });
        assert!(
            has_depends_on_link,
            "a depends-on link entry must be journaled"
        );
    }

    #[test]
    fn ingest_directory_categorized_checked_skips_an_unresolvable_reference_silently() {
        let dir = git_scratch_dir("checked_links_unresolved", "bahaa.fadam@gmail.com");
        std::fs::write(
            dir.join("guide.md"),
            "# Guide\n\nSee `nonexistent.py` for the script.\n",
        )
        .unwrap();
        git_commit_all(&dir);

        let mut wn = write_node();
        let allow = crate::authorship::TeamAllowlist::seed();
        let kakis = wn
            .ingest_directory_categorized_checked(&dir, 1, &allow)
            .unwrap();

        assert_eq!(kakis.len(), 1);
        // guide.md's own preamble section still mints its own "section-of"
        // link to its parent -- assert there is no "depends-on" link
        // specifically, not that there are zero link entries at all.
        let has_depends_on_link = wn.journal().all_entries().any(|e| {
            e.eav.iter().any(|t| {
                t.attr_hash == enkidb_ingest::bridge::attr_hash("link.description")
                    && matches!(
                        enkidb_ingest::bridge::eav_triple_to_value(t),
                        akkvalue::AkkValue::Text(ref s) if s == "depends-on"
                    )
            })
        });
        assert!(
            !has_depends_on_link,
            "a reference to a file that wasn't ingested must not produce a depends-on link"
        );
    }
}
