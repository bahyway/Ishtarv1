//! Crate corpus-scan minting — the second leg of making Shakkanakku the
//! one central tool where every element of BahyWay.Ecosystem gets a real
//! KAKIv4.0 Identity, alongside `pb_mint`'s playbooks. Scans
//! `<repo_root>/workspace/bahyway_v4/crates/*/Cargo.toml` via the real
//! `enkimdb::scan_crates`, mints any workspace crate not already in a
//! durable JSONL registry (keyed by manifest path — stable across runs,
//! matching `pb_mint`'s file-keyed convention) via
//! `enkimdb::WriteNode::ingest_artifact`, and materializes a fresh
//! Euphrates generation when at least one new crate was minted.
//!
//! Uses `enkimdb::ARTIFACT_TRIBE_ID` — the same tribe `enkimdb-write-
//! server`'s own crate/playbook scan already mints under, so a crate
//! catalogued by either tool shares one identity space.

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct RegistryLine {
    path: String,
    name: String,
    kaki_hex: String,
    minted_at: String,
}

/// Scans `<workspace_root>/crates/*/Cargo.toml`, mints a real Identity-
/// Kaki for any crate not already in `registry_path`, and materializes a
/// real Euphrates generation when at least one new crate was minted.
/// Returns human-readable log lines for the caller to forward as
/// `RunnerEvent::Log` — empty when there's nothing new (quiet, not an
/// error; most Corpus runs mint no new crates since the workspace's crate
/// list rarely changes).
pub fn mint_new_crates(
    workspace_root: &Path,
    registry_path: &Path,
    enkimdb_output_root: &Path,
) -> Vec<String> {
    let mut log = Vec::new();

    let scanned = match enkimdb::scan_crates(workspace_root) {
        Ok(v) => v,
        Err(e) => {
            log.push(format!("⚠ crate corpus scan failed: {e}"));
            return log;
        }
    };

    let already: HashSet<String> = std::fs::read_to_string(registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .map(|r| r.path)
        .collect();

    let new_crates: Vec<_> = scanned.into_iter().filter(|c| !already.contains(&c.path)).collect();
    if new_crates.is_empty() {
        return log;
    }

    let minter = KakiMinter::new(TribeId::from_u16(enkimdb::ARTIFACT_TRIBE_ID));
    let mut write_node = enkimdb::WriteNode::new(minter, 64);
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();

    if let Some(p) = registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new().create(true).append(true).open(registry_path).ok();

    for (i, artifact) in new_crates.iter().enumerate() {
        let kaki = write_node.ingest_artifact(artifact, i as u32 + 1);
        log.push(format!("𒁾 minted crate {} → {kaki}", artifact.name));
        if let Some(f) = &mut registry_file {
            let line = RegistryLine {
                path: artifact.path.clone(),
                name: artifact.name.clone(),
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
        let dir = std::env::temp_dir().join(format!("shk_crate_mint_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates/scratch_crate")).unwrap();
        std::fs::write(
            dir.join("crates/scratch_crate/Cargo.toml"),
            "[package]\nname = \"scratch_crate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn mints_a_real_new_crate_and_records_it_in_the_registry() {
        let dir = scratch_dir("mint_new");
        let registry = dir.join("chronicle/crate_kaki_registry.jsonl");
        let enkimdb_root = dir.join("enkimdb_data");

        let log = mint_new_crates(&dir, &registry, &enkimdb_root);
        assert!(log.iter().any(|l| l.contains("scratch_crate")), "expected a mint log line, got: {log:?}");

        let registry_text = std::fs::read_to_string(&registry).unwrap();
        assert!(registry_text.contains("scratch_crate"));
    }

    #[test]
    fn does_not_re_mint_a_crate_already_in_the_registry() {
        let dir = scratch_dir("no_remint");
        let registry = dir.join("chronicle/crate_kaki_registry.jsonl");
        let enkimdb_root = dir.join("enkimdb_data");

        let first = mint_new_crates(&dir, &registry, &enkimdb_root);
        assert!(!first.is_empty(), "first run must mint the new crate");

        let second = mint_new_crates(&dir, &registry, &enkimdb_root);
        assert!(second.is_empty(), "second run must find nothing new to mint: {second:?}");
    }
}
