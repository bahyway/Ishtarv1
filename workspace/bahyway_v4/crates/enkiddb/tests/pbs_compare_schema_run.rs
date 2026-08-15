//! Real, KAKI-sealed ingestion of the same 3 Architect-requested documents
//! (All PBs Roadmap, BahyWay.Ecosystem v4.0 Manual, BahyWay.Ecosystem v4
//! Full Glossary) as `preparing_bare_metal_pbs_run.rs`, but re-run under a
//! second, distinct collection tag: "Pbs_compare_schema" -- the Architect's
//! own explicit ask, after the 3 documents were updated to also document
//! the Shala4 Gate Orbits / playbook dependency-discovery feature, to save
//! that documentation "with its own Kaki Identity ... as ...
//! Pbs_compare_schema." Named for what the underlying feature actually
//! does: compare each catalogued playbook's own text against every other's
//! to discover real `depends-on` relationships (`pb_dependency_review.rs`).
//!
//! Uses the exact same `WriteNode::ingest_document_categorized` path
//! `preparing_bare_metal_pbs_run.rs` already proved works for these 3
//! documents -- a distinct collection tag mints a distinct set of
//! Identity-Kakis (KAKI is never shared across collections), so this is a
//! real, independent mint, not a re-read of the earlier one.

use std::fs;
use std::path::Path;

use bahyway_core::TribeId;
use enkidb_ingest::bridge::{attr_hash, eav_triple_to_value};
use enkidb_kaki::KakiMinter;
use enkiddb::parser::DocumentParser;
use enkiddb::WriteNode;
use akkvalue::AkkValue;

const COLLECTION: &str = "Pbs_compare_schema";

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap()
}

#[test]
fn the_3_updated_architect_documents_mint_as_real_enkiddb_particles_under_pbs_compare_schema() {
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
        // Prove the feature this schema documents is actually documented,
        // not just that the file happens to mint -- each of the 3 files
        // must carry real content about the dependency-discovery build.
        assert!(
            text.contains("depends-on") || text.contains("Pbs_compare_schema") || text.contains("Gate Orbits"),
            "{label} must document the Gate Orbits / dependency-discovery feature"
        );

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
            "{label} must be tagged under the Pbs_compare_schema schema"
        );
    }

    // Every one of the 3 documents got a DISTINCT real Identity-Kaki --
    // sovereign per-document identity, never shared/aliased.
    let mut kakis: Vec<_> = minted.iter().map(|(k, _)| k.bytes()).collect();
    kakis.sort();
    kakis.dedup();
    assert_eq!(kakis.len(), 3, "each document must mint its own distinct Identity-Kaki");

    // Materialize as a real, queryable EnkiDDB Tigris generation on disk.
    let root = std::env::temp_dir().join("enkiddb_pbs_compare_schema_run");
    let _ = fs::remove_dir_all(&root);
    let (generation, stats) = enkiddb::materialize_version(&write_node, &root, "pbs-compare-1")
        .expect("materializing the Pbs_compare_schema generation must succeed");
    // `entities` counts the 3 parent documents PLUS every child section
    // `ingest_document_categorized` mints per DocumentStructure::sections()
    // boundary -- so it must be at least 3, not exactly 3.
    assert!(stats.entities >= 3, "the materialized generation must contain at least the 3 minted documents, got {}", stats.entities);
    let real_data_file = generation.entities_path.with_extension("data");
    assert!(real_data_file.exists(), "the entities Tigris .data file must be real and on disk at {}", real_data_file.display());
}
