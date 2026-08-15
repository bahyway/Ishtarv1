//! PB corpus-scan minting — ADR-014's "mint at authoring time" for
//! playbooks. Each real Corpus-tab run scans `playbooks/` for any real,
//! numbered PB file that doesn't have a minted Identity-Kaki yet
//! (tracked via a durable JSONL registry — a fresh in-process
//! `enkimdb::WriteNode` has no memory of a previous run's mints, so this
//! is what makes "already minted" a real, persistent fact rather than
//! re-derived guesswork) and mints it for real via
//! `enkimdb::WriteNode::ingest_pb`. Never re-mints an already-registered
//! (file) pair — two different files CAN legitimately share one PB
//! number (see `enkimdb::pb`'s own doc comment), so the registry keys on
//! the file, never the bare number.

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

/// Fixed tribe id for BahyWay's own playbook corpus, distinct from
/// `enkiddb::DOCS_TRIBE_ID`'s documentation tribe — single source of
/// truth for every caller that mints `pb.*` particles.
pub const PB_TRIBE_ID: u16 = 0x7161;

#[derive(Serialize, Deserialize)]
struct RegistryLine {
    file: String,
    number: u32,
    kaki_hex: String,
    minted_at: String,
}

/// Scans `repo_root/playbooks` + `triage_doc`, mints a real Identity-Kaki
/// for any real, numbered PB file not already present in `registry_path`
/// (a durable, append-only JSONL ledger — never rewritten, matching this
/// crate's other JSONL ledgers, e.g. `docpulse.rs`'s ingest manifest),
/// and materializes a real, HeptaScript-queryable Euphrates generation
/// when at least one new PB was minted. Returns human-readable log lines
/// for the caller to forward as `RunnerEvent::Log` — empty when there's
/// nothing new to mint (quiet, not an error; this runs on every Corpus
/// pulse, most of which mint nothing).
pub fn mint_new_playbooks(
    repo_root: &Path,
    triage_doc: &Path,
    registry_path: &Path,
    enkimdb_output_root: &Path,
) -> Vec<String> {
    let mut log = Vec::new();

    let scanned = match enkimdb::scan_pbs(repo_root, triage_doc) {
        Ok(v) => v,
        Err(e) => {
            log.push(format!("⚠ PB corpus scan failed: {e}"));
            return log;
        }
    };

    let already: HashSet<String> = std::fs::read_to_string(registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .map(|r| r.file)
        .collect();

    let new_pbs: Vec<_> = scanned.into_iter().filter(|p| !already.contains(&p.file)).collect();
    if new_pbs.is_empty() {
        return log;
    }

    let minter = KakiMinter::new(TribeId::from_u16(PB_TRIBE_ID));
    let mut write_node = enkimdb::WriteNode::new(minter, 64);
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();

    if let Some(p) = registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new().create(true).append(true).open(registry_path).ok();

    for (i, pb) in new_pbs.iter().enumerate() {
        let kaki = write_node.ingest_pb(pb, i as u32 + 1);
        log.push(format!("𒁾 minted PB-{} ({}) → {kaki}", pb.number, pb.file));
        if let Some(f) = &mut registry_file {
            let line = RegistryLine {
                file: pb.file.clone(),
                number: pb.number,
                kaki_hex: hex::encode(kaki.bytes()),
                minted_at: chrono::Utc::now().to_rfc3339(),
            };
            if let Ok(json) = serde_json::to_string(&line) {
                let _ = writeln!(f, "{json}");
            }
        }
    }

    match enkimdb::materialize_version(&write_node, enkimdb_output_root, &stamp) {
        Ok((generation, stats)) => log.push(format!(
            "𒁾 EnkiMDB Euphrates generation {stamp} materialized: {} entities → {}",
            stats.entities,
            generation.entities_path.display()
        )),
        Err(e) => log.push(format!("⚠ EnkiMDB materialize failed: {e}")),
    }

    log
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_pb_mint_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("playbooks")).unwrap();
        dir
    }

    #[test]
    fn mints_a_real_new_pb_and_records_it_in_the_registry() {
        let dir = scratch_dir("mint_new");
        std::fs::write(dir.join("playbooks/playbook_900_test_scratch.yml"), "- hosts: localhost\n").unwrap();
        let triage = dir.join("docs_triage_missing.md"); // deliberately absent
        let registry = dir.join("chronicle/pb_kaki_registry.jsonl");
        let enkimdb_root = dir.join("enkimdb_data");

        let log = mint_new_playbooks(&dir, &triage, &registry, &enkimdb_root);
        assert!(log.iter().any(|l| l.contains("minted PB-900")), "expected a mint log line, got: {log:?}");

        let registry_text = std::fs::read_to_string(&registry).unwrap();
        // PbProfile.file is the file STEM (scan_playbooks strips the
        // extension, matching ArtifactProfile.name's own convention) --
        // not the full ".yml" filename.
        assert!(registry_text.contains("playbook_900_test_scratch"));
        assert!(registry_text.contains("\"number\":900"));
    }

    #[test]
    fn does_not_re_mint_a_pb_already_in_the_registry() {
        let dir = scratch_dir("no_remint");
        std::fs::write(dir.join("playbooks/playbook_901_test_scratch.yml"), "- hosts: localhost\n").unwrap();
        let triage = dir.join("docs_triage_missing.md");
        let registry = dir.join("chronicle/pb_kaki_registry.jsonl");
        let enkimdb_root = dir.join("enkimdb_data");

        let first = mint_new_playbooks(&dir, &triage, &registry, &enkimdb_root);
        assert!(!first.is_empty(), "first run must mint the new PB");

        let second = mint_new_playbooks(&dir, &triage, &registry, &enkimdb_root);
        assert!(second.is_empty(), "second run must find nothing new to mint: {second:?}");
    }
}
