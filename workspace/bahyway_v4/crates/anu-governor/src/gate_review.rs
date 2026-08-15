//! gate_review — suggest, never auto-decide, which of the 7 real
//! `bahyway_core::hepta_gate::HeptaGate` sectors a catalogued playbook
//! belongs to.
//!
//! Reuses `enkiddb::concepts::ConceptRegistry::with_known_gates()` -- the
//! exact same deterministic whole-word text scanner already wired into
//! `WriteNode::ingest_document_with_concepts` for concept-mention
//! detection elsewhere -- read-only here, as a SUGGESTION source only.
//! Per the Architect's own standing rule ("I do not want you to decide
//! what I need to include"), nothing in this module ever writes a gate
//! tag on its own; `apply_gate_approvals` only ever tags exactly the
//! (playbook, gate) pairs an explicit caller-supplied approval list
//! names -- the same propose -> Architect reviews -> explicit approve ->
//! only then written shape `godot/dubsar-theater/scripts/pdm_tab.gd`'s
//! Logical tab already established for schema-discovery proposals.

use crate::pb_catalog::RegistryLine;
use enkiddb::concepts::{ConceptKind, ConceptRegistry};
use enkiddb::{DocumentParser, WriteNode};
use enkidb_kaki::IdentityKaki;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

/// One playbook the gate registry has no approved classification for yet,
/// plus whichever real HeptaGate names its own text mentions (possibly
/// none, possibly several) -- a raw suggestion, never a ruling.
#[derive(Debug, Clone, Serialize)]
pub struct GateSuggestion {
    pub pb_kaki_hex: String,
    pub title: String,
    pub source_path: String,
    pub suggested_gates: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct GateRegistryLine {
    pb_kaki_hex: String,
    gate_akkadian_name: String,
    approved_at: String,
}

/// Every playbook with an APPROVED gate so far: `pb_kaki_hex ->
/// gate_akkadian_name`, later lines winning on a duplicate kaki (same
/// "later lines win" convention `docpulse`'s own doc-kaki registry
/// already uses) -- e.g. a re-classification recorded as a new appended
/// line rather than an in-place edit. Read by `anu_governor::domain_review`
/// to find which playbooks are eligible for domain suggestions under a
/// given gate; not needed by `pb_catalog` itself.
pub fn approved_gates(gate_registry_path: &Path) -> HashMap<String, String> {
    std::fs::read_to_string(gate_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<GateRegistryLine>(l).ok())
        .map(|r| (r.pb_kaki_hex, r.gate_akkadian_name))
        .collect()
}

/// Every real, distinct `(gate_akkadian_name, approved_at)` an Architect
/// has ever approved for `kaki_hex`, oldest first (the registry is
/// append-only, so file order already is chronological) -- ALL of them,
/// not just the current one, so StoryEngine's chronicle shows a real
/// reclassification as its own event rather than silently overwriting
/// the story of what happened before it.
pub fn gate_events_for(gate_registry_path: &Path, kaki_hex: &str) -> Vec<(String, String)> {
    std::fs::read_to_string(gate_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<GateRegistryLine>(l).ok())
        .filter(|r| r.pb_kaki_hex == kaki_hex)
        .map(|r| (r.gate_akkadian_name, r.approved_at))
        .collect()
}

/// Same "parse a stable kaki_hex back into an `IdentityKaki`" step
/// `pb_catalog::catalog_playbooks` already does for its own cross-run
/// `hash_kakis` rehydration -- shared here since gate review/approval
/// needs the identical conversion for playbooks it never minted itself.
pub fn parse_kaki_hex(kaki_hex: &str) -> Option<IdentityKaki> {
    let bytes = hex::decode(kaki_hex).ok()?;
    let arr: [u8; 16] = bytes.try_into().ok()?;
    let kaki = enkidb_kaki::Kaki::from_bytes(arr).ok()?;
    IdentityKaki::try_from_kaki(kaki).ok()
}

/// Scan every playbook in `pb_registry_path` (`pb_catalog`'s own content
/// registry) that `gate_registry_path` (this module's own approved-
/// assignment log) doesn't already have a recorded gate for, and return a
/// suggestion per one -- ready for an Architect to review, edit, and
/// approve. A pure read: writes nothing, mints nothing.
pub fn scan_gate_suggestions(pb_registry_path: &Path, gate_registry_path: &Path) -> Vec<GateSuggestion> {
    let already_tagged: HashSet<String> = std::fs::read_to_string(gate_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<GateRegistryLine>(l).ok())
        .map(|r| r.pb_kaki_hex)
        .collect();

    let registry = ConceptRegistry::new().with_known_gates();

    std::fs::read_to_string(pb_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .filter(|r| !already_tagged.contains(&r.kaki_hex))
        .filter_map(|r| {
            let path = PathBuf::from(&r.first_seen_path);
            let text = std::fs::read_to_string(&path).ok()?;
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let title = DocumentParser::parse_playbook_header(&text)
                .map(|s| s.title)
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| stem);
            let suggested_gates = registry
                .scan_mentions(&text)
                .into_iter()
                .filter(|c| c.kind == ConceptKind::Gate)
                .map(|c| c.name)
                .collect();
            Some(GateSuggestion { pb_kaki_hex: r.kaki_hex, title, source_path: r.first_seen_path, suggested_gates })
        })
        .collect()
}

/// Write exactly the (playbook, gate) pairs an Architect has explicitly
/// approved -- never more, never inferred. Appends each to
/// `gate_registry_path` so a later `scan_gate_suggestions` call stops
/// re-suggesting it. Returns one log line per approval; caller is
/// responsible for materializing `write_node` afterward (same as
/// `pb_catalog::catalog_playbooks`'s own contract).
pub fn apply_gate_approvals(
    write_node: &mut WriteNode,
    approvals: &[(IdentityKaki, String)],
    gate_registry_path: &Path,
    mut epoch: u32,
) -> Vec<String> {
    let mut log = Vec::new();
    if let Some(p) = gate_registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new().create(true).append(true).open(gate_registry_path).ok();

    for (kaki, gate_name) in approvals {
        write_node.tag_gate(*kaki, gate_name, epoch);
        epoch += 1;

        let line = GateRegistryLine {
            pb_kaki_hex: hex::encode(kaki.bytes()),
            gate_akkadian_name: gate_name.clone(),
            approved_at: chrono::Utc::now().to_rfc3339(),
        };
        if let Some(f) = &mut registry_file {
            if let Ok(json) = serde_json::to_string(&line) {
                let _ = writeln!(f, "{json}");
            }
        }
        log.push(format!("𒃵 tagged {kaki} -> {gate_name}"));
    }
    log
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb_catalog::{catalog_playbooks, CatalogLocation};
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_gate_review_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_gate_mentioning_pb(dir: &Path, name: &str, mention: &str) {
        std::fs::write(
            dir,
            format!("# =====\n# {name} -- Scratch\n#\n# Runs through the {mention} gate.\n# =====\n- name: \"scratch\"\n  hosts: localhost\n").as_bytes(),
        )
        .unwrap();
        let _ = name;
    }

    #[test]
    fn scan_suggests_the_real_gate_a_playbooks_own_text_mentions() {
        let dir = scratch_dir("suggest");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        let pb_path = loc.join("playbook_940_scratch.yml");
        write_gate_mentioning_pb(&pb_path, "PB-940", "ADAD");

        let pb_registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        catalog_playbooks(&[CatalogLocation { name: "loc".into(), root: loc }], &pb_registry, &enkiddb_root);

        let gate_registry = dir.join("chronicle/pb_gate_registry.jsonl");
        let suggestions = scan_gate_suggestions(&pb_registry, &gate_registry);

        assert_eq!(suggestions.len(), 1, "one catalogued playbook, one suggestion: {suggestions:?}");
        assert!(suggestions[0].suggested_gates.contains(&"ADAD".to_string()), "{suggestions:?}");
    }

    #[test]
    fn a_playbook_already_approved_is_never_suggested_again() {
        let dir = scratch_dir("no_resuggest");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        let pb_path = loc.join("playbook_941_scratch.yml");
        write_gate_mentioning_pb(&pb_path, "PB-941", "APSU");

        let pb_registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        catalog_playbooks(&[CatalogLocation { name: "loc".into(), root: loc }], &pb_registry, &enkiddb_root);

        let gate_registry = dir.join("chronicle/pb_gate_registry.jsonl");
        let first = scan_gate_suggestions(&pb_registry, &gate_registry);
        assert_eq!(first.len(), 1);
        let kaki = parse_kaki_hex(&first[0].pb_kaki_hex).unwrap();

        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(enkiddb::PLAYBOOK_CATALOG_TRIBE_ID)), 64);
        let log = apply_gate_approvals(&mut wn, &[(kaki, "APSU".to_string())], &gate_registry, 1);
        assert!(log.iter().any(|l| l.contains("APSU")), "{log:?}");

        let second = scan_gate_suggestions(&pb_registry, &gate_registry);
        assert!(second.is_empty(), "an approved playbook must not be re-suggested: {second:?}");
    }
}
