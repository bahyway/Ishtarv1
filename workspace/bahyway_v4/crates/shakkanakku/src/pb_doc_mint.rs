//! Playbook DOCUMENTATION corpus-scan minting — PB-270 catalogs that a
//! playbook FILE exists (`pb_mint`, into EnkiMDB, tribe
//! `pb_mint::PB_TRIBE_ID`); this module catalogs what it actually
//! *documents* (each playbook's leading `#`-comment header block) as a
//! real, versioned Document in EnkiDDB, tribe `enkiddb::PB_DOCS_TRIBE_ID`
//! — the Architect's own explicit ask: "save its documentation in the
//! EnkiDDB pb_docs_schema... query its content from HeptaScript for
//! versioning and release comparison."
//!
//! Unlike `docpulse`'s deliberate one-shot "promote this document now"
//! action (which re-mints unconditionally, since a human just chose to
//! promote), this runs automatically on every Corpus scan, so it must be
//! quiet when nothing changed: a durable JSONL registry (keyed by
//! playbook filename) stores the header text's own sha256 alongside the
//! minted Identity-Kaki. A playbook not yet in the registry gets a fresh
//! mint; a playbook whose header sha256 changed since last seen gets a
//! fresh mint THAT SUPERSEDES the old one (ADR-014's law, applied here);
//! a playbook whose header is byte-identical to last scan is skipped
//! entirely -- no new Identity, no supersession noise, matching
//! `pb_mint`/`crate_mint`/`tablet_mint`'s own "quiet unless something
//! real changed" convention.

use bahyway_core::TribeId;
use enkidb_kaki::{IdentityKaki, Kaki, KakiMinter};
use enkiddb::DocumentParser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct RegistryLine {
    file: String,
    sha256: String,
    kaki_hex: String,
    minted_at: String,
}

fn sha256_hex(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    hex::encode(h.finalize())
}

/// Scans `<repo_root>/playbooks/*.yml`, extracts each real playbook's
/// header-comment documentation, and mints/supersedes it in EnkiDDB when
/// new or changed since the last scan. Returns human-readable log lines
/// for the caller to forward as `RunnerEvent::Log` — empty when every
/// playbook's documentation is unchanged (the common case on most
/// Corpus runs).
pub fn mint_or_supersede_pb_docs(
    repo_root: &Path,
    registry_path: &Path,
    enkiddb_output_root: &Path,
) -> Vec<String> {
    let mut log = Vec::new();

    let playbooks_dir = repo_root.join("playbooks");
    let Ok(entries) = std::fs::read_dir(&playbooks_dir) else {
        log.push(format!("⚠ playbook doc corpus scan failed: cannot read {}", playbooks_dir.display()));
        return log;
    };
    let mut files: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    files.sort();

    // path (registry key = filename) -> (sha256, kaki_bytes), last write
    // wins on a duplicate key -- registry is append-only, this file's
    // own most recent line about a given playbook is the current truth.
    let previous: HashMap<String, (String, [u8; 16])> = std::fs::read_to_string(registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .filter_map(|r| {
            let bytes = hex::decode(&r.kaki_hex).ok()?;
            let arr: [u8; 16] = bytes.try_into().ok()?;
            Some((r.file, (r.sha256, arr)))
        })
        .collect();

    type ChangedEntry = (String, enkiddb::DocumentStructure, String, Option<(String, [u8; 16])>);
    let mut changed: Vec<ChangedEntry> = Vec::new();
    for path in &files {
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let Some(file) = path.file_name().map(|f| f.to_string_lossy().to_string()) else { continue };
        let Ok(text) = std::fs::read_to_string(path) else { continue };
        let scan = enkiddb::scan_document(text.as_bytes());
        if !scan.clean {
            log.push(format!("⚠ skipped {file}: {}", scan.detail));
            continue;
        }
        let Some(structure) = DocumentParser::parse_playbook_header(&text) else { continue };
        // Hash the extracted header text, not the whole file -- a
        // playbook's TASKS changing without its documented intent
        // changing is not a new documentation version.
        let header_text = format!("{}\n{}", structure.title, structure.body.iter().map(|b| b.content.as_str()).collect::<Vec<_>>().join("\n"));
        let sha = sha256_hex(&header_text);

        match previous.get(&file) {
            Some((old_sha, _)) if old_sha == &sha => continue, // unchanged, quiet
            Some((old_sha, old_kaki_bytes)) => changed.push((file, structure, sha, Some((old_sha.clone(), *old_kaki_bytes)))),
            None => changed.push((file, structure, sha, None)),
        }
    }
    if changed.is_empty() {
        return log;
    }

    let minter = KakiMinter::new(TribeId::from_u16(enkiddb::PB_DOCS_TRIBE_ID));
    let mut write_node = enkiddb::WriteNode::new(minter, 64);
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();

    if let Some(p) = registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new().create(true).append(true).open(registry_path).ok();

    for (i, (file, structure, sha, old)) in changed.iter().enumerate() {
        let epoch = i as u32 + 1;
        let kaki = write_node.ingest_document(structure, epoch);

        if let Some((old_sha, old_kaki_bytes)) = old {
            if let Ok(old_raw) = Kaki::from_bytes(*old_kaki_bytes) {
                if let Ok(old_kaki) = IdentityKaki::try_from_kaki(old_raw) {
                    let reason = format!("playbook header content changed (sha256 {}…→{}…)", &old_sha[..12], &sha[..12]);
                    write_node.supersede_document(old_kaki, kaki, &reason, epoch);
                    log.push(format!("↻ superseded prior doc-version of {file}: {old_kaki} → {kaki} ({reason})"));
                }
            }
        } else {
            log.push(format!("𒁾 minted playbook doc {file} → {kaki}"));
        }

        if let Some(f) = &mut registry_file {
            let line = RegistryLine {
                file: file.clone(),
                sha256: sha.clone(),
                kaki_hex: hex::encode(kaki.bytes()),
                minted_at: chrono::Utc::now().to_rfc3339(),
            };
            if let Ok(json) = serde_json::to_string(&line) {
                let _ = writeln!(f, "{json}");
            }
        }
    }

    match enkiddb::materialize_version(&write_node, enkiddb_output_root, &stamp) {
        Ok((generation, stats)) => log.push(format!(
            "𒁾 EnkiDDB Tigris generation {stamp} materialized: {} entities → {}",
            stats.entities,
            generation.entities_path.display()
        )),
        Err(e) => log.push(format!("⚠ EnkiDDB materialize failed: {e}")),
    }

    log
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_pb_doc_mint_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("playbooks")).unwrap();
        dir
    }

    fn write_pb(dir: &Path, name: &str, why: &str) {
        std::fs::write(
            dir.join("playbooks").join(name),
            format!(
                "# =====\n# PB-901 -- Scratch test playbook\n#\n# WHY THIS EXISTS: {why}\n# =====\n- name: \"scratch\"\n  hosts: localhost\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn mints_a_real_new_pb_doc_and_records_it() {
        let dir = scratch_dir("mint_new");
        write_pb(&dir, "playbook_901_scratch.yml", "a real first reason");
        let registry = dir.join("chronicle/pb_doc_kaki_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let log = mint_or_supersede_pb_docs(&dir, &registry, &enkiddb_root);
        assert!(log.iter().any(|l| l.contains("minted playbook doc")), "log: {log:?}");
        assert!(std::fs::read_to_string(&registry).unwrap().contains("playbook_901_scratch.yml"));
    }

    #[test]
    fn unchanged_header_is_quiet_on_second_scan() {
        let dir = scratch_dir("unchanged");
        write_pb(&dir, "playbook_902_scratch.yml", "stable reason");
        let registry = dir.join("chronicle/pb_doc_kaki_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let first = mint_or_supersede_pb_docs(&dir, &registry, &enkiddb_root);
        assert!(!first.is_empty());
        let second = mint_or_supersede_pb_docs(&dir, &registry, &enkiddb_root);
        assert!(second.is_empty(), "unchanged header must mint nothing: {second:?}");
    }

    #[test]
    fn changed_header_mints_a_new_version_and_supersedes_the_old_one() {
        let dir = scratch_dir("changed");
        write_pb(&dir, "playbook_903_scratch.yml", "the original reason");
        let registry = dir.join("chronicle/pb_doc_kaki_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let first = mint_or_supersede_pb_docs(&dir, &registry, &enkiddb_root);
        assert!(first.iter().any(|l| l.contains("minted playbook doc")));

        write_pb(&dir, "playbook_903_scratch.yml", "a REVISED reason after the fact");
        let second = mint_or_supersede_pb_docs(&dir, &registry, &enkiddb_root);
        assert!(second.iter().any(|l| l.contains("superseded prior doc-version")), "log: {second:?}");
    }
}
