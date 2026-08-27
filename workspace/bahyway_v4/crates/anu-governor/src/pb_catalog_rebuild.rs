//! pb_catalog_rebuild — replay the FULL playbook catalog (every
//! catalogued playbook's title/body, under its own stable KAKI) plus
//! every already-approved gate tag, domain tag, and depends-on edge into
//! a fresh `WriteNode`, so materializing and promoting THAT `WriteNode`
//! as `current` produces a complete, self-sufficient generation -- never
//! a tiny delta that would blank out everything a prior generation had.
//!
//! Extracted as its own path-parameterized, fully testable module because
//! `approve_gates`/`approve_domains`/`approve_dependencies`
//! (`bin/anu_governor_web.rs`) used to each build an EMPTY `WriteNode::
//! new()`, mint only their own new tags/edges, and promote THAT as
//! `current` -- discarding the entire previously-catalogued playbook set
//! (every title, body, and prior tag/edge) from `current`'s view the
//! moment the FIRST approval of any kind was ever made: every gate/
//! domain count would read back 0 and every `playbook_detail` lookup
//! would 404, even for playbooks that were genuinely catalogued and even
//! genuinely tagged in an earlier approval round. Exactly the same class
//! of bug Phase 0 fixed in `pb_catalog::catalog_playbooks` itself (every
//! `enkidb-readnode::Generation` must be a complete, independent
//! snapshot) -- this closes the identical gap for every OTHER write path
//! that promotes `current`.

use crate::domain_review;
use crate::gate_review;
use crate::pb_catalog;
use crate::pb_dependency_review;
use bahyway_core::TribeId;
use enkidb_kaki::{IdentityKaki, KakiMinter};
use enkiddb::{DocumentParser, DocumentStructure, WriteNode};
use std::collections::HashMap;
use std::path::Path;

/// Rebuilds a complete `WriteNode` from every registry this catalog
/// maintains. Returns the rebuilt `WriteNode`, the next free epoch to
/// keep minting from, and a `pb_kaki_hex -> (IdentityKaki, title)`
/// lookup so a caller (e.g. a NEW dependency approval that needs a
/// title) doesn't have to re-parse the catalog registry a second time.
pub fn rebuild_full_write_node(
    pb_registry_path: &Path,
    gate_registry_path: &Path,
    domain_registry_path: &Path,
    dependency_registry_path: &Path,
) -> (WriteNode, u32, HashMap<String, (IdentityKaki, String)>) {
    let mut write_node = WriteNode::new(
        KakiMinter::new(TribeId::from_u16(enkiddb::PLAYBOOK_CATALOG_TRIBE_ID)),
        64,
    );
    let mut epoch = 1u32;
    let mut by_kaki_hex: HashMap<String, (IdentityKaki, String)> = HashMap::new();

    for pb in pb_catalog::list_cataloged_playbooks(pb_registry_path) {
        let Some(kaki) = gate_review::parse_kaki_hex(&pb.kaki_hex) else {
            continue;
        };
        let stem = Path::new(&pb.source_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut structure =
            DocumentParser::parse_playbook_header(&pb.full_text).unwrap_or_else(|| {
                DocumentStructure {
                    title: stem.clone(),
                    ..Default::default()
                }
            });
        if structure.title.trim().is_empty() {
            structure.title = stem.clone();
        }
        write_node.reingest_document_categorized(
            kaki,
            &structure,
            epoch,
            "playbook-record-candidate",
        );
        epoch += 1;
        by_kaki_hex.insert(pb.kaki_hex.clone(), (kaki, pb.title.clone()));
    }

    for (kaki_hex, gate_name) in gate_review::approved_gates(gate_registry_path) {
        if let Some((kaki, _)) = by_kaki_hex.get(&kaki_hex) {
            write_node.tag_gate(*kaki, &gate_name, epoch);
            epoch += 1;
        }
    }

    for (kaki_hex, domain_name) in domain_review::approved_domains(domain_registry_path) {
        if let Some((kaki, _)) = by_kaki_hex.get(&kaki_hex) {
            write_node.tag_domain(*kaki, &domain_name, epoch);
            epoch += 1;
        }
    }

    for (source_hex, target_hex) in pb_dependency_review::approved_pairs(dependency_registry_path) {
        if let (Some((source_kaki, source_title)), Some((target_kaki, target_title))) =
            (by_kaki_hex.get(&source_hex), by_kaki_hex.get(&target_hex))
        {
            write_node.mint_link_edge(
                *source_kaki,
                source_title,
                *target_kaki,
                target_title,
                "depends-on",
                epoch,
            );
            epoch += 1;
        }
    }

    (write_node, epoch, by_kaki_hex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_review::apply_gate_approvals;
    use crate::pb_catalog::{catalog_playbooks, CatalogLocation};
    use std::path::PathBuf;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_pb_catalog_rebuild_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_pb(dir: &Path, name: &str) {
        std::fs::write(
            dir.join(name),
            format!(
                "# =====\n# {name} -- Scratch\n#\n# WHY THIS EXISTS: a real reason\n# =====\n- name: \"scratch\"\n  hosts: localhost\n"
            ),
        )
        .unwrap();
    }

    /// The exact regression this module exists to prevent: approving a
    /// SECOND playbook's gate (a fresh rebuild + apply, exactly what a
    /// real second `approve_gates` HTTP call does) must not blank the
    /// FIRST playbook's already-approved gate tag -- or the first
    /// playbook itself -- out of the newly materialized generation.
    #[test]
    fn a_second_approval_round_does_not_lose_the_first_rounds_playbook_or_tag() {
        let dir = scratch_dir("two_rounds");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        write_pb(&loc, "playbook_970_first.yml");
        write_pb(&loc, "playbook_971_second.yml");

        let pb_registry = dir.join("pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        catalog_playbooks(
            &[CatalogLocation {
                name: "loc".into(),
                root: loc,
            }],
            &pb_registry,
            &enkiddb_root,
        );

        let gate_registry = dir.join("pb_gate_registry.jsonl");
        let domain_registry = dir.join("pb_domain_registry.jsonl");
        let dependency_registry = dir.join("pb_dependency_registry.jsonl");

        let catalogued = pb_catalog::list_cataloged_playbooks(&pb_registry);
        let first = catalogued
            .iter()
            .find(|p| p.source_path.contains("970"))
            .unwrap();
        let second = catalogued
            .iter()
            .find(|p| p.source_path.contains("971"))
            .unwrap();
        let first_kaki = gate_review::parse_kaki_hex(&first.kaki_hex).unwrap();
        let second_kaki = gate_review::parse_kaki_hex(&second.kaki_hex).unwrap();

        // Round 1: approve the FIRST playbook's gate.
        let (mut wn1, epoch1, _) = rebuild_full_write_node(
            &pb_registry,
            &gate_registry,
            &domain_registry,
            &dependency_registry,
        );
        apply_gate_approvals(
            &mut wn1,
            &[(first_kaki, "APSU".to_string())],
            &gate_registry,
            epoch1,
        );

        // Round 2: approve the SECOND playbook's gate -- a fresh rebuild,
        // same as a real second `approve_gates` call.
        let (mut wn2, epoch2, by_kaki) = rebuild_full_write_node(
            &pb_registry,
            &gate_registry,
            &domain_registry,
            &dependency_registry,
        );
        apply_gate_approvals(
            &mut wn2,
            &[(second_kaki, "ADAD".to_string())],
            &gate_registry,
            epoch2,
        );

        assert!(
            by_kaki.contains_key(&first.kaki_hex),
            "round 2's rebuild must still know about round 1's playbook"
        );
        assert_eq!(by_kaki.get(&first.kaki_hex).unwrap().1, first.title);

        let root = dir.join("materialized");
        let (generation, _stats) = enkiddb::materialize_version(&wn2, &root, "v2").unwrap();
        let mut read_node =
            enkiddb::ReadNode::open(&generation.entities_path, &generation.eav_path).unwrap();

        let apsu = read_node
            .query("WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.gate] = \"APSU\"")
            .unwrap();
        assert_eq!(
            apsu.matched.len(),
            1,
            "round 1's APSU tag on the first playbook must survive round 2's rebuild+promote: {apsu:?}"
        );

        let adad = read_node
            .query("WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.gate] = \"ADAD\"")
            .unwrap();
        assert_eq!(
            adad.matched.len(),
            1,
            "round 2's own ADAD tag on the second playbook must also be there: {adad:?}"
        );
    }
}
