//! enkiddb-ingest — CLI runner: upload a folder tree of Markdown docs into
//! EnkiDDB (Tigris) in one pass — walk, categorize, chunk into RAG
//! sections, journal, materialize a queryable generation, demo-search it.
//!
//! Usage: enkiddb-ingest <docs_root> <output_root> <version> [search_query]
//!
//! Example:
//!   enkiddb-ingest ~/BahyWayV4/docs ~/tigris_data 4.0 "KAKI identity"
#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use enkiddb::{materialize_version, RagIndex, WriteNode, DOCS_TRIBE_ID};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <docs_root> <output_root> <version> [search_query]", args[0]);
        eprintln!(r#"Example: {} ~/BahyWayV4/docs ~/tigris_data 4.0 "KAKI identity""#, args[0]);
        std::process::exit(1);
    }
    let docs_root = PathBuf::from(&args[1]);
    let output_root = PathBuf::from(&args[2]);
    let version = &args[3];
    let search_query = args.get(4);

    if !docs_root.is_dir() {
        eprintln!("error: {} is not a directory", docs_root.display());
        std::process::exit(1);
    }

    let minter = KakiMinter::new(TribeId::from_u16(DOCS_TRIBE_ID));
    let mut write_node = WriteNode::new(minter, 64);

    println!("Walking {} for Markdown files...", docs_root.display());
    let doc_kakis = match write_node.ingest_directory_categorized(&docs_root, 1) {
        Ok(kakis) => kakis,
        Err(e) => {
            eprintln!("error walking/ingesting {}: {e:?}", docs_root.display());
            std::process::exit(1);
        }
    };

    let total_entries = write_node.journal().entry_count();
    let section_entries = total_entries - doc_kakis.len();
    println!(
        "Journaled {} document(s), {} section chunk(s) ({} Journal entries total).",
        doc_kakis.len(),
        section_entries,
        total_entries
    );
    if doc_kakis.is_empty() {
        println!("No .md files found under {} -- nothing to materialize.", docs_root.display());
        return;
    }

    println!("Materializing Tigris v{version} to {}...", output_root.display());
    let (generation, stats) = match materialize_version(&write_node, &output_root, version) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("error materializing: {e}");
            std::process::exit(1);
        }
    };
    println!(
        "Tigris v{version} ready: {} entities, {} distinct EAV keys, at {}",
        stats.entities,
        stats.distinct_eav_keys,
        generation.entities_path.display()
    );
    println!("Queryable now via enkiddb::ReadNode::open() / HeptaScript, no re-ingestion needed.");

    let rag = RagIndex::build(write_node.journal());
    println!("RAG index: {} searchable chunk(s) (from body.summary particles).", rag.len());
    println!(
        "NOTE: the RAG index above is built from this run's in-memory Journal; \
         it does not persist to disk. Re-run this binary against the same \
         docs_root to rebuild it (cheap: a plain re-parse + re-index pass, \
         the same batch-rebuild contract materialize_version already uses)."
    );

    if let Some(query) = search_query {
        println!("\nSearch: \"{query}\"");
        let hits = rag.search(query, 5);
        if hits.is_empty() {
            println!("  (no matches)");
        }
        for hit in hits {
            let value = RagIndex::fetch_value(write_node.journal(), &hit.section).unwrap_or_default();
            let preview: String = value.chars().take(120).collect();
            println!("  score={:.3}  {}...", hit.score, preview.trim());
        }
    }
}
