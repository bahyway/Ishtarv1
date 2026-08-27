//! Tablet corpus-scan minting — the third leg of making AnuGovernor the
//! one central tool where every element of BahyWay.Ecosystem gets a real
//! KAKIv4.0 Identity, alongside `pb_mint`'s playbooks and `crate_mint`'s
//! crates. Recursively finds every real `.akk`/`.way`/`.tmpl` file under
//! `repo_root` (skipping `.git`/`target`/`node_modules` — build and VCS
//! internals, never tablets), mints any not already in `registry_path`
//! via `enkimdb::profile_tablet` + `enkimdb::WriteNode::ingest_tablet`,
//! and materializes a fresh Euphrates generation per kind that minted
//! something new.
//!
//! Uses the *same* JSONL registry shape (and, by convention, the same
//! path — `<chronicle_dir>/tablet_kaki_registry.jsonl` from the repo
//! root) as `bin/girsu-mint`, the per-file CLI Girsu IDE's "mark as
//! final" task invokes: a tablet marked final from inside Girsu is
//! already in this registry, so AnuGovernor's corpus-wide pass will not
//! re-mint it, and vice versa — one shared source of truth for "is this
//! tablet's Identity already minted," regardless of which of the two
//! tools noticed it first.
//!
//! Each `TabletKind` mints under its own fixed tribe (`TabletKind::
//! tribe_id`), so — unlike `pb_mint`/`crate_mint`, which mint one kind of
//! thing under one tribe — new files are grouped by kind first and each
//! group gets its own `KakiMinter`/`WriteNode`/materialize pass; a mixed
//! batch of new `.akk` and `.way` files produces two log lines and two
//! Euphrates generations, not one of each mixed together under the wrong
//! tribe.

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use enkimdb::{TabletKind, TabletProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules"];

#[derive(Serialize, Deserialize)]
struct RegistryLine {
    path: String,
    kind: String,
    kaki_hex: String,
    sha256: String,
    minted_at: String,
}

/// Recursively collect every real `.akk`/`.way`/`.tmpl` file under `root`,
/// skipping VCS/build directories. Best-effort: an unreadable
/// subdirectory is skipped, not fatal to the whole scan (matches
/// `scan_crates`/`scan_playbooks`'s own "real files only, no invented
/// entries" ethos, extended to "one bad directory doesn't block the
/// rest").
fn find_tablets(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return out;
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            if SKIP_DIRS.iter().any(|d| name == std::ffi::OsStr::new(d)) {
                continue;
            }
            out.extend(find_tablets(&path));
        } else if TabletKind::from_path(&path).is_some() {
            out.push(path);
        }
    }
    out
}

/// Scans `repo_root` for every real tablet not already in `registry_path`,
/// mints each (grouped by kind, one Identity per tablet), and materializes
/// one Euphrates generation per kind that minted something new. Returns
/// human-readable log lines for the caller to forward as
/// `RunnerEvent::Log` — empty when there's nothing new (quiet on the vast
/// majority of runs, same as `pb_mint`/`crate_mint`).
pub fn mint_new_tablets(
    repo_root: &Path,
    registry_path: &Path,
    enkimdb_output_root: &Path,
) -> Vec<String> {
    let mut log = Vec::new();

    let already: HashSet<String> = std::fs::read_to_string(registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .map(|r| r.path)
        .collect();

    let new_tablets: Vec<TabletProfile> = find_tablets(repo_root)
        .into_iter()
        .filter_map(|p| std::fs::canonicalize(&p).ok())
        .filter(|p| !already.contains(&p.display().to_string()))
        .filter_map(|p| enkimdb::profile_tablet(&p).ok())
        .collect();
    if new_tablets.is_empty() {
        return log;
    }

    if let Some(p) = registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(registry_path)
        .ok();

    for kind in [TabletKind::Akk, TabletKind::Way, TabletKind::Tmpl] {
        let batch: Vec<&TabletProfile> = new_tablets.iter().filter(|p| p.kind == kind).collect();
        if batch.is_empty() {
            continue;
        }

        let minter = KakiMinter::new(TribeId::from_u16(kind.tribe_id()));
        let mut write_node = enkimdb::WriteNode::new(minter, 64);
        let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();

        for (i, profile) in batch.iter().enumerate() {
            let kaki = write_node.ingest_tablet(profile, i as u32 + 1);
            log.push(format!(
                "𒁾 minted {} tablet {} → {kaki}",
                kind.as_str(),
                profile.path
            ));
            if let Some(f) = &mut registry_file {
                let line = RegistryLine {
                    path: profile.path.clone(),
                    kind: kind.as_str().to_string(),
                    kaki_hex: hex::encode(kaki.bytes()),
                    sha256: profile.sha256.clone(),
                    minted_at: chrono::Utc::now().to_rfc3339(),
                };
                if let Ok(json) = serde_json::to_string(&line) {
                    let _ = writeln!(f, "{json}");
                }
            }
        }

        match enkimdb::materialize_version(
            &write_node,
            enkimdb_output_root,
            &format!("{stamp}_{}", kind.as_str()),
        ) {
            Ok((generation, stats)) => log.push(format!(
                "𒁾 EnkiMDB Euphrates generation {stamp}_{} materialized: {} entities → {}",
                kind.as_str(),
                stats.entities,
                generation.entities_path.display()
            )),
            Err(e) => log.push(format!(
                "⚠ EnkiMDB materialize failed for {} tablets: {e}",
                kind.as_str()
            )),
        }
    }

    log
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_tablet_mint_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn mints_a_real_new_akk_tablet_and_records_it() {
        let dir = scratch_dir("mint_akk");
        std::fs::write(dir.join("sample.akk"), "tribe demo {}\n").unwrap();
        let registry = dir.join("chronicle/tablet_kaki_registry.jsonl");
        let enkimdb_root = dir.join("enkimdb_data");

        let log = mint_new_tablets(&dir, &registry, &enkimdb_root);
        assert!(
            log.iter().any(|l| l.contains("akk tablet")),
            "expected an akk mint log line, got: {log:?}"
        );
        assert!(std::fs::read_to_string(&registry)
            .unwrap()
            .contains("sample.akk"));
    }

    #[test]
    fn does_not_re_mint_a_tablet_already_in_the_registry() {
        let dir = scratch_dir("no_remint");
        std::fs::write(dir.join("policy.way"), "some way content\n").unwrap();
        let registry = dir.join("chronicle/tablet_kaki_registry.jsonl");
        let enkimdb_root = dir.join("enkimdb_data");

        let first = mint_new_tablets(&dir, &registry, &enkimdb_root);
        assert!(!first.is_empty(), "first run must mint the new tablet");

        let second = mint_new_tablets(&dir, &registry, &enkimdb_root);
        assert!(
            second.is_empty(),
            "second run must find nothing new to mint: {second:?}"
        );
    }

    #[test]
    fn skips_git_target_and_node_modules_directories() {
        let dir = scratch_dir("skip_dirs");
        std::fs::create_dir_all(dir.join("target/debug")).unwrap();
        std::fs::write(dir.join("target/debug/leftover.akk"), "tribe x {}\n").unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".git/HEAD.akk"), "tribe y {}\n").unwrap();
        std::fs::write(dir.join("real.akk"), "tribe z {}\n").unwrap();

        let found = find_tablets(&dir);
        assert_eq!(found.len(), 1, "expected only real.akk, found: {found:?}");
        assert!(found[0].ends_with("real.akk"));
    }

    #[test]
    fn mixed_kind_batch_mints_both_kinds_under_their_own_tribe() {
        let dir = scratch_dir("mixed_kinds");
        std::fs::write(dir.join("a.akk"), "tribe demo {}\n").unwrap();
        std::fs::write(dir.join("b.way"), "some way content\n").unwrap();
        let registry = dir.join("chronicle/tablet_kaki_registry.jsonl");
        let enkimdb_root = dir.join("enkimdb_data");

        let log = mint_new_tablets(&dir, &registry, &enkimdb_root);
        assert!(log.iter().any(|l| l.contains("akk tablet")));
        assert!(log.iter().any(|l| l.contains("way tablet")));
    }
}
