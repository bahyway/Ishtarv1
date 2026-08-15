//! pb_dependency_review — suggest, never auto-decide, which OTHER
//! catalogued playbooks a playbook's own text references (a real
//! "depends-on" candidate), mirroring `gate_review`/`domain_review`'s
//! exact suggest-then-approve shape.
//!
//! `enkiddb::links::discover_referenced_paths` (the mechanism
//! `WriteNode::link_discovered_dependencies` already uses for markdown
//! docs) only matches a Markdown `[label](path)` link or a `` `path` ``
//! backtick span -- real playbook cross-references never look like
//! that. They show up as a bare path in a `# ansible-playbook
//! playbooks/playbook_284_launch_shala_dashboard.yml` usage comment, or
//! plain prose like "see PB-284" / "requires playbook_284". This module
//! scans for those shapes instead: a whole-word mention of another
//! catalogued playbook's exact file stem, or its "PB-<n>"/"PB<n>"/
//! "playbook_<n>" number forms.

use crate::pb_catalog::{guess_pb_number, list_cataloged_playbooks, CatalogedPlaybook};
use enkiddb::WriteNode;
use enkidb_kaki::IdentityKaki;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

/// One real, un-approved "source's text mentions target" candidate --
/// a raw suggestion, never a ruling. `direction` is always
/// source-depends-on-target from the scanner's point of view; the
/// Architect decides whether that's actually true (a mention isn't
/// proof of a real dependency).
#[derive(Debug, Clone, Serialize)]
pub struct DependencySuggestion {
    pub source_kaki_hex: String,
    pub source_title: String,
    pub target_kaki_hex: String,
    pub target_title: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct DependencyRegistryLine {
    source_kaki_hex: String,
    target_kaki_hex: String,
    approved_at: String,
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn contains_word(haystack_lower: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return false;
    }
    let hbytes = haystack_lower.as_bytes();
    for (idx, _) in haystack_lower.match_indices(needle_lower) {
        let before_ok = idx == 0 || !is_word_byte(hbytes[idx - 1]);
        let after_idx = idx + needle_lower.len();
        let after_ok = after_idx >= hbytes.len() || !is_word_byte(hbytes[after_idx]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn stem_of(source_path: &str) -> String {
    PathBuf::from(source_path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
}

/// Every real, distinct way `target`'s identity could plausibly appear
/// in another playbook's prose -- its exact file stem (highest
/// precision), plus its number in "PB-<n>", "PB<n>", and "playbook_<n>"
/// forms when a number is guessable at all (`guess_pb_number`'s own
/// hint, not an identity).
fn mention_needles(target: &CatalogedPlaybook) -> Vec<String> {
    let stem = stem_of(&target.source_path);
    let mut needles = vec![stem.to_lowercase()];
    if let Some(n) = guess_pb_number(&stem) {
        needles.push(format!("pb-{n}"));
        needles.push(format!("pb{n}"));
        needles.push(format!("playbook_{n}"));
    }
    needles
}

/// Every (source_kaki_hex, target_kaki_hex) pair an Architect has already
/// approved as a real `depends-on` edge -- read by `scan_dependency_
/// suggestions` (so it never re-suggests one) and by `shakkanakku_web`'s
/// full-catalog rebuild (so every approved edge survives a re-materialized
/// generation, not just the ones minted in whichever WriteNode last
/// promoted `current`).
pub fn approved_pairs(dependency_registry_path: &Path) -> HashSet<(String, String)> {
    std::fs::read_to_string(dependency_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<DependencyRegistryLine>(l).ok())
        .map(|r| (r.source_kaki_hex, r.target_kaki_hex))
        .collect()
}

/// Every real `depends-on` edge `kaki_hex` was ever a party to, on either
/// side, oldest first: `(other_kaki_hex, approved_at, is_source)` where
/// `is_source = true` means `kaki_hex` is the one that depends on
/// `other_kaki_hex`, `false` means `other_kaki_hex` depends on
/// `kaki_hex`. StoryEngine's chronicle needs both directions to tell a
/// real "who does this affect" story, not just "what does this depend on."
pub fn dependency_events_for(dependency_registry_path: &Path, kaki_hex: &str) -> Vec<(String, String, bool)> {
    std::fs::read_to_string(dependency_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<DependencyRegistryLine>(l).ok())
        .filter_map(|r| {
            if r.source_kaki_hex == kaki_hex {
                Some((r.target_kaki_hex, r.approved_at, true))
            } else if r.target_kaki_hex == kaki_hex {
                Some((r.source_kaki_hex, r.approved_at, false))
            } else {
                None
            }
        })
        .collect()
}

/// Scan every catalogued playbook's full text for a whole-word mention
/// of any OTHER catalogued playbook's identity (see `mention_needles`),
/// skipping pairs `dependency_registry_path` already has an approved
/// decision for. O(n²) whole-word scans over the real catalog size
/// (hundreds, not millions, of playbooks) -- a deliberate, occasional
/// action, not a hot path. A pure read: writes nothing, mints nothing.
pub fn scan_dependency_suggestions(
    pb_registry_path: &Path,
    dependency_registry_path: &Path,
) -> Vec<DependencySuggestion> {
    let all = list_cataloged_playbooks(pb_registry_path);
    let already = approved_pairs(dependency_registry_path);

    let mut out = Vec::new();
    for source in &all {
        let lower = source.full_text.to_lowercase();
        for target in &all {
            if target.kaki_hex == source.kaki_hex {
                continue;
            }
            if already.contains(&(source.kaki_hex.clone(), target.kaki_hex.clone())) {
                continue;
            }
            let needles = mention_needles(target);
            if needles.iter().any(|n| contains_word(&lower, n)) {
                out.push(DependencySuggestion {
                    source_kaki_hex: source.kaki_hex.clone(),
                    source_title: source.title.clone(),
                    target_kaki_hex: target.kaki_hex.clone(),
                    target_title: target.title.clone(),
                });
            }
        }
    }
    out
}

/// Write exactly the (source, target) "depends-on" pairs an Architect
/// has explicitly approved -- via `WriteNode::mint_link_edge`, the same
/// dedicated-edge-entity primitive `pb_catalog`'s own found-in/contains
/// edges already use (never a repeated particle on the shared source
/// entity -- see that method's own doc comment for why). Appends each
/// approval to `dependency_registry_path` so a later
/// `scan_dependency_suggestions` call stops re-suggesting it. Caller is
/// responsible for materializing `write_node` afterward.
pub fn apply_dependency_approvals(
    write_node: &mut WriteNode,
    approvals: &[(IdentityKaki, String, IdentityKaki, String)], // (source_kaki, source_title, target_kaki, target_title)
    dependency_registry_path: &Path,
    mut epoch: u32,
) -> Vec<String> {
    let mut log = Vec::new();
    if let Some(p) = dependency_registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new().create(true).append(true).open(dependency_registry_path).ok();

    for (source_kaki, source_title, target_kaki, target_title) in approvals {
        write_node.mint_link_edge(*source_kaki, source_title, *target_kaki, target_title, "depends-on", epoch);
        epoch += 1;

        let line = DependencyRegistryLine {
            source_kaki_hex: hex::encode(source_kaki.bytes()),
            target_kaki_hex: hex::encode(target_kaki.bytes()),
            approved_at: chrono::Utc::now().to_rfc3339(),
        };
        if let Some(f) = &mut registry_file {
            if let Ok(json) = serde_json::to_string(&line) {
                let _ = writeln!(f, "{json}");
            }
        }
        log.push(format!("𒃵 {source_title} --depends-on--> {target_title}"));
    }
    log
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb_catalog::{catalog_playbooks, CatalogLocation};
    use crate::gate_review::parse_kaki_hex;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_pb_dependency_review_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_pb_mentioning(dir: &Path, name: &str, mention: &str) {
        std::fs::write(
            dir.join(name),
            format!("# =====\n# {name} -- Scratch\n#\n# Runs after {mention} completes.\n# =====\n- name: \"scratch\"\n  hosts: localhost\n"),
        )
        .unwrap();
    }

    #[test]
    fn scan_suggests_a_real_mention_of_another_catalogued_playbook() {
        let dir = scratch_dir("suggest");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        write_pb_mentioning(&loc, "playbook_960_base.yml", "nothing else yet");
        write_pb_mentioning(&loc, "playbook_961_followup.yml", "playbook_960_base");

        let pb_registry = dir.join("pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        catalog_playbooks(&[CatalogLocation { name: "loc".into(), root: loc }], &pb_registry, &enkiddb_root);

        let dep_registry = dir.join("pb_dependency_registry.jsonl");
        let suggestions = scan_dependency_suggestions(&pb_registry, &dep_registry);

        assert_eq!(suggestions.len(), 1, "{suggestions:?}");
        assert!(suggestions[0].source_title.contains("961"), "{suggestions:?}");
        assert!(suggestions[0].target_title.contains("960"), "{suggestions:?}");
    }

    #[test]
    fn an_approved_pair_is_never_suggested_again() {
        let dir = scratch_dir("no_resuggest");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        write_pb_mentioning(&loc, "playbook_962_base.yml", "nothing else yet");
        write_pb_mentioning(&loc, "playbook_963_followup.yml", "playbook_962_base");

        let pb_registry = dir.join("pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        catalog_playbooks(&[CatalogLocation { name: "loc".into(), root: loc }], &pb_registry, &enkiddb_root);

        let dep_registry = dir.join("pb_dependency_registry.jsonl");
        let first = scan_dependency_suggestions(&pb_registry, &dep_registry);
        assert_eq!(first.len(), 1);
        let source = parse_kaki_hex(&first[0].source_kaki_hex).unwrap();
        let target = parse_kaki_hex(&first[0].target_kaki_hex).unwrap();

        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(enkiddb::PLAYBOOK_CATALOG_TRIBE_ID)), 64);
        let log = apply_dependency_approvals(
            &mut wn,
            &[(source, first[0].source_title.clone(), target, first[0].target_title.clone())],
            &dep_registry,
            1,
        );
        assert!(log.iter().any(|l| l.contains("depends-on")), "{log:?}");

        let second = scan_dependency_suggestions(&pb_registry, &dep_registry);
        assert!(second.is_empty(), "an approved pair must not be re-suggested: {second:?}");
    }
}
