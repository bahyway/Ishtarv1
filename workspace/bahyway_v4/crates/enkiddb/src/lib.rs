//! enkiddb — EnkiDDB: BahyWay.Ecosystem v4.0 Documentation Database.
//!
//! Documents (and, later, source-code/service catalog entries) are minted
//! as sovereign Identity-Kakis whose content lands as real
//! `enkidb_particles::Particle` EAV rows, namespaced by `DocOrbit`
//! (meta/head/body/code/hist/link) — the same insert-only, KAKI-sealed
//! model used everywhere else in the ecosystem.
//!
//! CQRS split (added 2026-07-12), mirroring EnkiDB's own port-7001
//! write/read node pattern rather than inventing a new one:
//!   - [`writenode::WriteNode`] — the write path. A real
//!     `enkidb_journal::Journal` is the WAL; every ingested document is
//!     durable in it before `ingest_document` returns.
//!   - [`readnode`] — the read path. Wraps the real, pre-existing
//!     `enkidb-readnode` crate (ADR-012): [`readnode::materialize_now`]
//!     performs a one-time O(n) pass over the WriteNode's Journal into
//!     two Data Files; [`readnode::ReadNode`] opens them in O(1) and
//!     answers HeptaScript queries without ever touching the Journal.
//! The two nodes are genuinely decoupled — the Read Node sees nothing
//! until `materialize_now` runs, proven by test.
//!
//! Sovereign name (2026-07-12): EnkiDDB is **Tigris** —
//! [`readnode::SOVEREIGN_NAME`]. [`readnode::materialize_version`] +
//! [`readnode::list_versions`] give every materialization a version tag
//! ("Tigris v4.0", "Tigris v4.1", ...) so DubSar Theater can list, open,
//! and compare multiple documentation generations side by side. This
//! does not rename the crate or the EnkiDDB type — same relationship
//! "GeoEngine" has to `bahyway-algebra`.
//!
//! RAG categorization + summary-to-chunk indexing (added 2026-07-12), the
//! Architect's "Key-Value Document Mapping" ask, built without deferring
//! any part of it to a later phase:
//!   - [`document::Section`] / [`document::DocumentStructure::sections`] —
//!     groups a parsed document's headers/body/code at header boundaries,
//!     so nothing (no paragraph, no code block) is orphaned or dropped.
//!   - [`emitter::DocumentEmitter::emit_collection`] — tags a document with
//!     a `meta.collection` KIND taxonomy (architecture-reference, glossary,
//!     component, playbook-record, ...), orthogonal to `DocOrbit`.
//!   - [`emitter::DocumentEmitter::emit_sections`] — mints one child
//!     Identity-Kaki per section, linked back to its parent document, each
//!     carrying `body.summary` (the searchable Key: header + a bounded
//!     preview) and `body.text` (the Value: the section's full, untouched
//!     text).
//!   - [`writenode::WriteNode::ingest_document_categorized`] — journals a
//!     document plus every one of its sections in one call, so the
//!     Journal alone is enough to rebuild the RAG index.
//!   - [`rag::RagIndex`] — wraps the real `adapa-recall` TF-IDF + cosine
//!     retrieval crate over every `body.summary` in a `Journal`;
//!     [`rag::RagIndex::search`] finds the topically closest chunks,
//!     [`rag::RagIndex::fetch_value`] fetches each hit's full, untouched
//!     `body.text` back out of the same Journal. Summaries are heuristic,
//!     not LLM-generated (`enkidullm-codex` doesn't exist yet) — an honest
//!     v1 that indexes now rather than waiting on that.
#![forbid(unsafe_code)]

/// Fixed tribe id for BahyWay's own documentation corpus, distinct from
/// any tribe used for ingested external/third-party content — single
/// source of truth for every caller that mints documentation Identity-
/// Kakis, whether that's `enkiddb-ingest`'s bulk directory walk or
/// AnuGovernor's Hala tab (prior name Uruinimgina) minting one promoted document at a time
/// (see `crates/anu-governor/src/docpulse.rs`). Both enter the same
/// EnkiDDB documentation corpus, so both use the same tribe. See
/// `enkidb_kaki::KakiMinter::new`'s doc comment on tribe id conventions.
pub const DOCS_TRIBE_ID: u16 = 0x7160;

/// Fixed tribe id for playbook DOCUMENTATION specifically (the header
/// comment block each numbered playbook carries) -- distinct from
/// `DOCS_TRIBE_ID` (markdown docs) and from
/// `anu_governor::pb_mint::PB_TRIBE_ID` (the playbook FILE's own
/// existence/registration identity, in EnkiMDB, not EnkiDDB). Continues
/// the 0x716x sequence (0x7160 docs, 0x7161 PB, 0x7162/63/64 tablets).
/// PB-270's decree: EnkiMDB catalogs that a playbook exists; this tribe
/// is what lets its actual documented CONTENT be versioned and compared
/// release-over-release via `supersede_document`, the same law
/// `DOCS_TRIBE_ID` already uses for markdown docs.
pub const PB_DOCS_TRIBE_ID: u16 = 0x7165;

/// Fixed tribe id for the Architect's own personal daily-work documentation
/// corpus (ideas, discussions, drafts, and complete write-ups alike) --
/// deliberately DISTINCT from `DOCS_TRIBE_ID`. Both corpora are minted
/// through the same Hala pipeline (`crates/anu-governor/src/
/// docpulse.rs`, via `DocPulseCfg::docs_tribe_id`) and both are real,
/// queryable EnkiDDB content -- the separation exists so a HeptaScript
/// `WHERE tribe=...` query (or any RagIndex search) never silently mixes
/// the Architect's own drafts/notes with sealed BahyWay.Ecosystem v4.0
/// documentation. Continues the 0x716x sequence (0x7160 docs, 0x7161 PB,
/// 0x7162/63/64 tablets, 0x7165 PB docs).
pub const ARCHITECT_DOCS_TRIBE_ID: u16 = 0x7166;

/// Fixed tribe id for MULTI-LOCATION playbook catalog candidates --
/// deliberately NOT the sealed corpus (`anu_governor::pb_mint::PB_TRIBE_ID`
/// in EnkiMDB for file registration, `PB_DOCS_TRIBE_ID` above for this
/// repo's own canonical `playbooks/` documentation). Historical/backup/
/// duplicate playbook sightings gathered from every known location (the
/// Architect's DailyWork repo, the retired eriduous-vdi's rsynced mirror,
/// dated external-drive backups, ...) land here first, unreviewed, so a
/// HeptaScript `WHERE tribe=...` query never silently mixes an
/// unreconciled backup copy with the sealed, current corpus -- same
/// separation principle `ARCHITECT_DOCS_TRIBE_ID` already established.
/// See `anu_governor::pb_catalog`. Continues the 0x716x sequence (0x7160
/// docs, 0x7161 PB, 0x7162/63/64 tablets, 0x7165 PB docs, 0x7166
/// architect docs).
pub const PLAYBOOK_CATALOG_TRIBE_ID: u16 = 0x7167;

pub mod authorship;
pub mod concepts;
pub mod document;
pub mod emitter;
pub mod exposure;
pub mod ingest;
pub mod links;
pub mod orbit;
pub mod parser;
pub mod rag;
pub mod readnode;
pub mod security;
pub mod topics;
pub mod writenode;

pub use authorship::{check_authorship, git_author_email, AuthorshipCheck, TeamAllowlist};
pub use concepts::{ConceptEntry, ConceptKind, ConceptRegistry};
pub use document::{BodyElement, BodyType, CodeBlock, DocumentStructure, Header, Section};
pub use emitter::DocumentEmitter;
pub use exposure::{ConceptGraph, ExposureReport};
pub use ingest::{ingest_directory, scan_markdown_directory, IngestError, IngestedDocument};
pub use links::discover_referenced_paths;
pub use orbit::DocOrbit;
pub use parser::DocumentParser;
pub use rag::{RagHit, RagIndex};
pub use readnode::{list_versions, materialize_now, materialize_version, Generation, ReadNode, ReadNodeError, SOVEREIGN_NAME};
pub use security::{scan_document, SecurityScanResult};
pub use writenode::WriteNode;
