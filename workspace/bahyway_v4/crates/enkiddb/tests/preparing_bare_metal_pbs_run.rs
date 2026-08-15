//! Real, KAKI-sealed ingestion of the 3 Architect-requested documents
//! (All PBs Roadmap, BahyWay.Ecosystem v4.0 Manual, BahyWay.Ecosystem v4
//! Full Glossary) into EnkiDDB under the "Preparing_bare_metal_PBs_Run"
//! collection tag -- this is the actual mapping the Architect asked for,
//! not a description of one. Uses the same `WriteNode::
//! ingest_document_categorized` path `docpulse.rs`'s stage 4 (EnkiDDB
//! mint) and `enkiddb-ingest`'s bulk CLI both already use, with a fixed
//! `collection` string instead of `infer_collection`'s path-based
//! inference, since all 3 documents share one Architect-chosen schema
//! regardless of which repo subdirectory they happen to live in.

use std::fs;
use std::path::Path;

use bahyway_core::TribeId;
use enkidb_ingest::bridge::{attr_hash, eav_triple_to_value};
use enkidb_kaki::KakiMinter;
use enkiddb::parser::DocumentParser;
use enkiddb::WriteNode;
use akkvalue::AkkValue;

const COLLECTION: &str = "Preparing_bare_metal_PBs_Run";

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap()
}

#[test]
fn all_three_architect_documents_mint_as_real_enkiddb_particles_under_the_schema() {
    let docs = [
        ("docs/20_meta_engine/ALL_PBS_ROADMAP.md", "All PBs Roadmap"),
        ("docs/20_meta_engine/BAHYWAY_ECOSYSTEM_MANUAL_V4.md", "BahyWay.Ecosystem v4.0 Manual"),
        ("docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md", "BahyWay.Ecosystem v4 Full Glossary"),
    ];

    let minter = KakiMinter::new(TribeId::from_u16(enkiddb::PB_DOCS_TRIBE_ID));
    let mut write_node = WriteNode::new(minter, 64);
    let mut minted = Vec::new();

    for (i, (rel, label)) in docs.iter().enumerate() {
        let full = repo_root().join(rel);
        let text = fs::read_to_string(&full)
            .unwrap_or_else(|e| panic!("{label} must be a real, readable file at {}: {e}", full.display()));
        assert!(!text.trim().is_empty(), "{label} must not be empty");

        let structure = DocumentParser::parse_markdown(&text);
        let doc_kaki = write_node.ingest_document_categorized(&structure, i as u32 + 1, COLLECTION);
        minted.push((doc_kaki, *label));
    }

    assert_eq!(minted.len(), 3, "all 3 Architect-requested documents must mint");

    let collection_hash = attr_hash("meta.collection");
    for (kaki, label) in &minted {
        let history = write_node.journal().read_particle_history(kaki);
        assert!(!history.is_empty(), "{label}'s Identity-Kaki must have real journal history");

        let tagged = history.iter().flat_map(|e| e.eav.iter()).find(|t| t.attr_hash == collection_hash);
        let tagged = tagged.unwrap_or_else(|| panic!("{label} must carry a real meta.collection particle"));
        let value = eav_triple_to_value(tagged);
        assert_eq!(
            value,
            AkkValue::Text(COLLECTION.to_string()),
            "{label} must be tagged under the Preparing_bare_metal_PBs_Run schema"
        );
    }

    // Every one of the 3 documents got a DISTINCT real Identity-Kaki --
    // sovereign per-document identity, never shared/aliased.
    let mut kakis: Vec<_> = minted.iter().map(|(k, _)| k.bytes()).collect();
    kakis.sort();
    kakis.dedup();
    assert_eq!(kakis.len(), 3, "each document must mint its own distinct Identity-Kaki");

    // Materialize as a real, queryable EnkiDDB Tigris generation on disk --
    // the same call docpulse.rs's stage 4 makes, so this is a real mint,
    // not just an in-memory journal.
    let root = std::env::temp_dir().join("enkiddb_preparing_bare_metal_pbs_run");
    let _ = fs::remove_dir_all(&root);
    let (generation, stats) = enkiddb::materialize_version(&write_node, &root, "pbs-run-1")
        .expect("materializing the Preparing_bare_metal_PBs_Run generation must succeed");
    // `entities` counts the 3 parent documents PLUS every child section
    // `ingest_document_categorized` mints per DocumentStructure::sections()
    // boundary -- so it must be at least 3, not exactly 3.
    assert!(stats.entities >= 3, "the materialized generation must contain at least the 3 minted documents, got {}", stats.entities);
    // `entities_path` is DataFileWriter's `base` -- the real file on disk
    // is `{base}.data` (see enkidb-datafile::writer::DataFileWriter::open).
    let real_data_file = generation.entities_path.with_extension("data");
    assert!(real_data_file.exists(), "the entities Tigris .data file must be real and on disk at {}", real_data_file.display());
}
