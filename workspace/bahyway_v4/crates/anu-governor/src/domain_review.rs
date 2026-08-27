//! domain_review — suggest, never auto-decide, which of 7 sub-domains
//! WITHIN an already-approved `bahyway_core::hepta_gate::HeptaGate` a
//! catalogued playbook belongs to.
//!
//! Unlike `HeptaGate` itself (a sealed, ecosystem-wide ruling), the 49
//! domain names below (7 per gate) are `anu_governor`'s own concern --
//! scoped to browsing/organizing the playbook catalog specifically, not
//! claimed as a sealed ecosystem taxonomy. They exist to give the gate
//! suggestion scanner a real vocabulary to look for; the Architect stays
//! free to redirect any individual suggestion in review, same as
//! `gate_review`'s own suggest-then-approve shape.
//!
//! Only playbooks with an ALREADY-APPROVED gate (`gate_review::
//! approved_gates`) are ever scanned here -- a domain only makes sense
//! once its parent gate is settled.

use crate::gate_review::approved_gates;
use crate::pb_catalog::RegistryLine;
use enkidb_kaki::IdentityKaki;
use enkiddb::{DocumentParser, WriteNode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

/// 7 domain names per gate, in `bahyway_core::hepta_gate::HeptaGate`'s own
/// Akkadian-name order. A reasonable, reviewable starting vocabulary
/// grounded in each gate's own sealed `sector()`/`pipeline_function()` --
/// not a ruling, since nothing here is ever written without an explicit
/// Architect approval.
pub const DOMAIN_NAMES: [(&str, [&str; 7]); 7] = [
    (
        "APSU",
        [
            "Data Files",
            "Journals",
            "Backups",
            "Snapshots",
            "Replication",
            "Retention",
            "Encryption",
        ],
    ),
    (
        "ADAD",
        [
            "Extraction",
            "Transformation",
            "Loading",
            "Validation",
            "Scheduling",
            "Error Handling",
            "Source Connectors",
        ],
    ),
    (
        "SHEDU",
        [
            "Authentication",
            "Authorization",
            "Encryption",
            "Auditing",
            "Threat Detection",
            "Vault Management",
            "Network Hardening",
        ],
    ),
    (
        "MUMMU",
        [
            "Schema Design",
            "Type Systems",
            "Query Engines",
            "Indexing",
            "Optimization",
            "Formal Verification",
            "Data Modeling",
        ],
    ),
    (
        "ENKIDU",
        [
            "Match Dedupe",
            "Steward Review",
            "Model Training",
            "Inference",
            "Prompt Engineering",
            "Agent Orchestration",
            "Feedback Loops",
        ],
    ),
    (
        "DUBSAR",
        [
            "Parsing",
            "Tokenization",
            "Grammar",
            "Localization",
            "Documentation",
            "Naming Conventions",
            "Translation",
        ],
    ),
    (
        "ENLIL",
        [
            "SLA Signoff",
            "Policy",
            "Compliance",
            "Auditing",
            "Change Management",
            "Access Control",
            "Reporting",
        ],
    ),
];

pub fn domains_for_gate(gate_akkadian_name: &str) -> Option<&'static [&'static str; 7]> {
    let upper = gate_akkadian_name.to_ascii_uppercase();
    DOMAIN_NAMES
        .iter()
        .find(|(g, _)| *g == upper)
        .map(|(_, domains)| domains)
}

/// One playbook (already approved into `gate_akkadian_name`) the domain
/// registry has no approved sub-classification for yet, plus whichever of
/// that gate's 7 domain names its own text mentions -- a raw suggestion.
#[derive(Debug, Clone, Serialize)]
pub struct DomainSuggestion {
    pub pb_kaki_hex: String,
    pub title: String,
    pub source_path: String,
    pub suggested_domains: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct DomainRegistryLine {
    pb_kaki_hex: String,
    gate_akkadian_name: String,
    domain_name: String,
    approved_at: String,
}

/// Every playbook with an APPROVED domain so far: `pb_kaki_hex ->
/// domain_name`, later lines winning on a duplicate kaki -- same "later
/// lines win" convention `gate_review::approved_gates` already uses.
/// Read by callers (e.g. search/listing UIs) that want to show a
/// playbook's current domain without a live EnkiDDB query.
pub fn approved_domains(domain_registry_path: &Path) -> std::collections::HashMap<String, String> {
    std::fs::read_to_string(domain_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<DomainRegistryLine>(l).ok())
        .map(|r| (r.pb_kaki_hex, r.domain_name))
        .collect()
}

/// Every real `(gate_akkadian_name, domain_name, approved_at)` an
/// Architect has ever approved for `kaki_hex`, oldest first -- ALL of
/// them, for the same reason `gate_review::gate_events_for` keeps every
/// approval: a real reclassification is its own chronicle event, not a
/// silent overwrite.
pub fn domain_events_for(
    domain_registry_path: &Path,
    kaki_hex: &str,
) -> Vec<(String, String, String)> {
    std::fs::read_to_string(domain_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<DomainRegistryLine>(l).ok())
        .filter(|r| r.pb_kaki_hex == kaki_hex)
        .map(|r| (r.gate_akkadian_name, r.domain_name, r.approved_at))
        .collect()
}

/// Whole-phrase, case-insensitive, boundary-checked match -- `needle` may
/// itself contain spaces (domain names are multi-word, e.g. "Data
/// Files"), so `enkiddb::concepts::ConceptRegistry`'s single-identifier
/// word-boundary scanner doesn't apply here; this is the phrase-shaped
/// equivalent of the same idea.
fn contains_phrase(haystack_lower: &str, needle_lower: &str) -> bool {
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

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// Scan every playbook approved into `gate_akkadian_name` that
/// `domain_registry_path` has no recorded domain for yet, and return a
/// suggestion per one. A pure read: writes nothing, mints nothing.
pub fn scan_domain_suggestions(
    pb_registry_path: &Path,
    gate_registry_path: &Path,
    domain_registry_path: &Path,
    gate_akkadian_name: &str,
) -> Vec<DomainSuggestion> {
    let Some(domains) = domains_for_gate(gate_akkadian_name) else {
        return Vec::new();
    };
    let gate_upper = gate_akkadian_name.to_ascii_uppercase();

    let gates = approved_gates(gate_registry_path);
    let already_domained: HashSet<String> = std::fs::read_to_string(domain_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<DomainRegistryLine>(l).ok())
        .map(|r| r.pb_kaki_hex)
        .collect();

    std::fs::read_to_string(pb_registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .filter(|r| {
            gates.get(&r.kaki_hex).map(|g| g.to_ascii_uppercase()) == Some(gate_upper.clone())
        })
        .filter(|r| !already_domained.contains(&r.kaki_hex))
        .filter_map(|r| {
            let path = PathBuf::from(&r.first_seen_path);
            let text = std::fs::read_to_string(&path).ok()?;
            let lower = text.to_lowercase();
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let title = DocumentParser::parse_playbook_header(&text)
                .map(|s| s.title)
                .filter(|t| !t.trim().is_empty())
                .unwrap_or(stem);
            let suggested_domains = domains
                .iter()
                .filter(|d| contains_phrase(&lower, &d.to_lowercase()))
                .map(|d| d.to_string())
                .collect();
            Some(DomainSuggestion {
                pb_kaki_hex: r.kaki_hex,
                title,
                source_path: r.first_seen_path,
                suggested_domains,
            })
        })
        .collect()
}

/// Write exactly the (playbook, domain) pairs an Architect has explicitly
/// approved. Appends each to `domain_registry_path` so a later
/// `scan_domain_suggestions` call stops re-suggesting it. Caller is
/// responsible for materializing `write_node` afterward, same contract as
/// `gate_review::apply_gate_approvals`.
pub fn apply_domain_approvals(
    write_node: &mut WriteNode,
    approvals: &[(IdentityKaki, String, String)], // (kaki, gate_akkadian_name, domain_name)
    domain_registry_path: &Path,
    mut epoch: u32,
) -> Vec<String> {
    let mut log = Vec::new();
    if let Some(p) = domain_registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(domain_registry_path)
        .ok();

    for (kaki, gate_name, domain_name) in approvals {
        write_node.tag_domain(*kaki, domain_name, epoch);
        epoch += 1;

        let line = DomainRegistryLine {
            pb_kaki_hex: hex::encode(kaki.bytes()),
            gate_akkadian_name: gate_name.clone(),
            domain_name: domain_name.clone(),
            approved_at: chrono::Utc::now().to_rfc3339(),
        };
        if let Some(f) = &mut registry_file {
            if let Ok(json) = serde_json::to_string(&line) {
                let _ = writeln!(f, "{json}");
            }
        }
        log.push(format!("𒃵 tagged {kaki} -> {gate_name} / {domain_name}"));
    }
    log
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_review::{apply_gate_approvals, parse_kaki_hex};
    use crate::pb_catalog::{catalog_playbooks, CatalogLocation};
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_domain_review_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_domain_mentioning_pb(path: &Path, mention: &str) {
        std::fs::write(
            path,
            format!("# =====\n# PB-950 -- Scratch\n#\n# Runs through the ADAD gate, focused on {mention}.\n# =====\n- name: \"scratch\"\n  hosts: localhost\n"),
        )
        .unwrap();
    }

    #[test]
    fn scan_suggests_a_real_domain_a_playbooks_own_text_mentions() {
        let dir = scratch_dir("suggest");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        let pb_path = loc.join("playbook_950_scratch.yml");
        write_domain_mentioning_pb(&pb_path, "Extraction");

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

        let suggestions_before_gate =
            scan_domain_suggestions(&pb_registry, &gate_registry, &domain_registry, "ADAD");
        assert!(
            suggestions_before_gate.is_empty(),
            "a playbook with no approved gate yet must not be domain-suggested"
        );

        let gate_suggestions =
            crate::gate_review::scan_gate_suggestions(&pb_registry, &gate_registry);
        assert_eq!(gate_suggestions.len(), 1);
        let kaki = parse_kaki_hex(&gate_suggestions[0].pb_kaki_hex).unwrap();
        let mut wn = WriteNode::new(
            KakiMinter::new(TribeId::from_u16(enkiddb::PLAYBOOK_CATALOG_TRIBE_ID)),
            64,
        );
        apply_gate_approvals(&mut wn, &[(kaki, "ADAD".to_string())], &gate_registry, 1);

        let suggestions =
            scan_domain_suggestions(&pb_registry, &gate_registry, &domain_registry, "ADAD");
        assert_eq!(suggestions.len(), 1, "{suggestions:?}");
        assert!(
            suggestions[0]
                .suggested_domains
                .contains(&"Extraction".to_string()),
            "{suggestions:?}"
        );
    }
}
