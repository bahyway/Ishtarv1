//! pb_story — merge the 4 append-only registries (`pb_catalog`,
//! `gate_review`, `domain_review`, `pb_dependency_review`) into one real,
//! timestamped chronicle per playbook KAKI -- StoryEngine's "birth till
//! now" narrative, entirely derived from real facts already durably
//! recorded on disk, never a simulated Epoch/VGCA/B11 template like the
//! uploaded ETL-dashboard prototypes' StoryEngine.
//!
//! A gate/domain reclassification keeps every prior approval as its own
//! event too (`gate_events_for`/`domain_events_for` both return every
//! match, not just the current one) -- a real story, not just current
//! state.

use crate::domain_review;
use crate::gate_review;
use crate::pb_catalog;
use crate::pb_dependency_review;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct StoryEvent {
    pub ts: String,
    pub kind: String,
    pub title: String,
    pub detail: String,
}

/// The real chronicle for `kaki_hex`, oldest event first. Empty (not an
/// error) if this kaki isn't in any of the 4 registries at all.
pub fn playbook_story_events(
    pb_registry_path: &Path,
    gate_registry_path: &Path,
    domain_registry_path: &Path,
    dependency_registry_path: &Path,
    kaki_hex: &str,
) -> Vec<StoryEvent> {
    let mut events: Vec<StoryEvent> = Vec::new();

    if let Some((minted_at, source_path)) = pb_catalog::catalog_event_for(pb_registry_path, kaki_hex) {
        events.push(StoryEvent {
            ts: minted_at,
            kind: "CATALOGUED".to_string(),
            title: "Catalogued".to_string(),
            detail: format!("First catalogued from {source_path}"),
        });
    }

    for (gate_name, approved_at) in gate_review::gate_events_for(gate_registry_path, kaki_hex) {
        events.push(StoryEvent {
            ts: approved_at,
            kind: "GATE_APPROVED".to_string(),
            title: "Gate classified".to_string(),
            detail: format!("Classified into the {gate_name} gate"),
        });
    }

    for (gate_name, domain_name, approved_at) in domain_review::domain_events_for(domain_registry_path, kaki_hex) {
        events.push(StoryEvent {
            ts: approved_at,
            kind: "DOMAIN_APPROVED".to_string(),
            title: "Domain classified".to_string(),
            detail: format!("Classified into {domain_name} (within {gate_name})"),
        });
    }

    let dep_events = pb_dependency_review::dependency_events_for(dependency_registry_path, kaki_hex);
    if !dep_events.is_empty() {
        let titles: HashMap<String, String> = pb_catalog::list_cataloged_playbooks(pb_registry_path)
            .into_iter()
            .map(|pb| (pb.kaki_hex, pb.title))
            .collect();
        for (other_hex, approved_at, is_source) in dep_events {
            let other_title = titles.get(&other_hex).cloned().unwrap_or_else(|| other_hex.clone());
            let (kind, title, detail) = if is_source {
                ("DEPENDS_ON_APPROVED", "Dependency confirmed", format!("Confirmed depends-on {other_title}"))
            } else {
                ("DEPENDED_ON_BY_APPROVED", "Dependent confirmed", format!("Confirmed depended-on-by {other_title}"))
            };
            events.push(StoryEvent { ts: approved_at, kind: kind.to_string(), title: title.to_string(), detail });
        }
    }

    events.sort_by(|a, b| a.ts.cmp(&b.ts));
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_review::apply_domain_approvals;
    use crate::gate_review::{apply_gate_approvals, parse_kaki_hex};
    use crate::pb_catalog::{catalog_playbooks, list_cataloged_playbooks, CatalogLocation};
    use crate::pb_dependency_review::apply_dependency_approvals;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;
    use std::path::PathBuf;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_pb_story_test_{name}"));
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

    #[test]
    fn a_real_chronicle_merges_all_4_registries_in_timestamp_order() {
        let dir = scratch_dir("full_chronicle");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        write_pb(&loc, "playbook_980_first.yml");
        write_pb(&loc, "playbook_981_second.yml");

        let pb_registry = dir.join("pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        catalog_playbooks(&[CatalogLocation { name: "loc".into(), root: loc }], &pb_registry, &enkiddb_root);

        let catalogued = list_cataloged_playbooks(&pb_registry);
        let first = catalogued.iter().find(|p| p.source_path.contains("980")).unwrap();
        let second = catalogued.iter().find(|p| p.source_path.contains("981")).unwrap();
        let first_kaki = parse_kaki_hex(&first.kaki_hex).unwrap();
        let second_kaki = parse_kaki_hex(&second.kaki_hex).unwrap();

        let gate_registry = dir.join("pb_gate_registry.jsonl");
        let domain_registry = dir.join("pb_domain_registry.jsonl");
        let dependency_registry = dir.join("pb_dependency_registry.jsonl");

        let mut wn = enkiddb::WriteNode::new(KakiMinter::new(TribeId::from_u16(enkiddb::PLAYBOOK_CATALOG_TRIBE_ID)), 64);
        apply_gate_approvals(&mut wn, &[(first_kaki, "APSU".to_string())], &gate_registry, 1);
        apply_domain_approvals(
            &mut wn,
            &[(first_kaki, "APSU".to_string(), "Data Files".to_string())],
            &domain_registry,
            2,
        );
        apply_dependency_approvals(
            &mut wn,
            &[(first_kaki, first.title.clone(), second_kaki, second.title.clone())],
            &dependency_registry,
            3,
        );

        let story = playbook_story_events(&pb_registry, &gate_registry, &domain_registry, &dependency_registry, &first.kaki_hex);
        let kinds: Vec<&str> = story.iter().map(|e| e.kind.as_str()).collect();
        assert_eq!(
            kinds,
            vec!["CATALOGUED", "GATE_APPROVED", "DOMAIN_APPROVED", "DEPENDS_ON_APPROVED"],
            "chronicle must contain all 4 real event kinds, oldest first: {story:?}"
        );
        assert!(story[3].detail.contains(&second.title), "the dependency event must name the real other playbook: {story:?}");

        // The SECOND playbook's own chronicle must show it from the OTHER
        // side -- a real "who depends on me" fact, not the same event
        // duplicated identically.
        let second_story =
            playbook_story_events(&pb_registry, &gate_registry, &domain_registry, &dependency_registry, &second.kaki_hex);
        assert!(
            second_story.iter().any(|e| e.kind == "DEPENDED_ON_BY_APPROVED" && e.detail.contains(&first.title)),
            "second playbook's chronicle must show the real depended-on-by event: {second_story:?}"
        );
    }
}
