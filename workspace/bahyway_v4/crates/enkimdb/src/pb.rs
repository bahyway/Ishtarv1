//! PB scanning — real, numbered playbooks as a scannable profile,
//! cross-referenced against `docs/PLAYBOOK_EXECUTION_TRIAGE.md`'s own
//! Status/Note columns when available. Mirrors `scan.rs`'s own
//! "no bootstrap/demo data" law: every `PbProfile` corresponds to a real
//! file on disk at scan time. The triage doc's filename cell is never
//! trusted for identity -- only for status/note -- so a stale doc row
//! can't misname a real file; the filesystem scan is what decides which
//! PBs exist.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::scan::scan_playbooks;

/// One real, numbered playbook's profile, ready to be minted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PbProfile {
    pub number: u32,
    /// The file STEM (no `.yml` extension), matching
    /// `ArtifactProfile.name`'s own convention — e.g. `playbook_269_x`,
    /// not `playbook_269_x.yml`. This is the identity key any
    /// "already minted" registry should use.
    pub file: String,
    pub path: String,
    /// From the triage doc's own Status column (`[x]`/`[~]`/`[ ]`/`SKIP`),
    /// when that row exists yet -- a PB can be real (numbered, committed)
    /// before it has a triage row.
    pub status: Option<String>,
    /// The triage doc's own Note column text for this PB, when present.
    pub summary: Option<String>,
}

/// Scan `<repo_root>/playbooks/*.yml` for real, numbered playbooks
/// (`playbook_<N>_...yml` -- files with no parseable number, e.g. the
/// orchestrator `playbook_IsimudEngine.yml`, are not part of the numbered
/// corpus and are skipped), cross-referenced against `triage_doc`'s own
/// `| # | File | Status | Note |` table when that path exists (a missing
/// triage doc is not an error -- every PB just gets `status`/`summary`
/// of `None`).
pub fn scan_pbs(repo_root: &Path, triage_doc: &Path) -> std::io::Result<Vec<PbProfile>> {
    let artifacts = scan_playbooks(repo_root)?;
    let triage = fs::read_to_string(triage_doc).unwrap_or_default();
    // Keyed by filename, NOT by bare number: two real files can
    // legitimately share one PB number (the triage doc's own fixture
    // documents this -- playbook_90_a.yml / playbook_90_b.yml, "different
    // content, same number"). Keying by number here would silently
    // misattribute one file's status/note onto the other.
    let rows = parse_triage_rows(&triage);

    let mut out: Vec<PbProfile> = artifacts
        .into_iter()
        .filter_map(|a| {
            let number = parse_pb_number(&a.name)?;
            let filename = format!("{}.yml", a.name);
            let (status, summary) = rows.get(&filename).cloned().unwrap_or((None, None));
            Some(PbProfile { number, file: a.name, path: a.path, status, summary })
        })
        .collect();
    out.sort_by(|a, b| a.number.cmp(&b.number).then_with(|| a.file.cmp(&b.file)));
    Ok(out)
}

/// `playbook_269_retire_....yml` (file stem, extension already stripped
/// by `scan_playbooks`) -> `269`. Files that don't match this shape
/// (no `playbook_` prefix, or a non-numeric second segment like
/// `playbook_IsimudEngine`) return `None`.
fn parse_pb_number(file_stem: &str) -> Option<u32> {
    let rest = file_stem.strip_prefix("playbook_")?;
    rest.split('_').next()?.parse::<u32>().ok()
}

/// Parses `| # | File | Status | Note |`-shaped rows, keyed by the File
/// cell (backtick-quoted filename, e.g. `` `playbook_90_a.yml` ``) rather
/// than the bare number -- the only key that stays correct when two real
/// files legitimately share one PB number. `splitn(5, '|')` keeps a Note
/// cell that itself contains a literal `|` intact (everything after the
/// 3rd real separator lands in one final piece), rather than truncating
/// it the way a plain `split('|')` + `cells[4]` would.
fn parse_triage_rows(text: &str) -> HashMap<String, (Option<String>, Option<String>)> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.splitn(5, '|').collect();
        if cells.len() < 4 {
            continue;
        }
        // Row must actually be a numbered PB row (`| 268 | ... |`), not a
        // header/separator row (`| # | ... |` / `|---|...|`).
        if cells[1].trim().parse::<u32>().is_err() {
            continue;
        }
        let filename = cells[2].trim().trim_matches('`').to_string();
        if filename.is_empty() {
            continue;
        }
        let status = cells[3].trim().trim_matches('`').to_string();
        let summary = cells
            .get(4)
            .map(|s| s.trim().trim_end_matches('|').trim().to_string())
            .filter(|s| !s.is_empty());
        out.insert(filename, (Some(status).filter(|s| !s.is_empty()), summary));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pb_number_from_real_filename_pattern() {
        assert_eq!(
            parse_pb_number("playbook_269_retire_eriduous_vdi_confirm_baremetal_control_node"),
            Some(269)
        );
        assert_eq!(parse_pb_number("playbook_IsimudEngine"), None);
        assert_eq!(parse_pb_number("not_a_playbook"), None);
    }

    #[test]
    fn parses_a_real_triage_row() {
        let doc = "\
| # | File | Status | Note |
|---|------|--------|------|
| 268 | `playbook_268_bahyway_host_privilege_groups.yml` | `[x]` | run for real, groups created |
";
        let rows = parse_triage_rows(doc);
        let (status, summary) = rows
            .get("playbook_268_bahyway_host_privilege_groups.yml")
            .cloned()
            .expect("row 268 must parse");
        assert_eq!(status.as_deref(), Some("[x]"));
        assert_eq!(summary.as_deref(), Some("run for real, groups created"));
    }

    #[test]
    fn keeps_a_literal_pipe_inside_the_note_cell_intact() {
        let doc = "| 42 | `playbook_42_x.yml` | `[~]` | see A\\|B for details |\n";
        let rows = parse_triage_rows(doc);
        let (_, summary) = rows.get("playbook_42_x.yml").cloned().expect("row 42 must parse");
        assert!(summary.unwrap().contains("A\\|B"), "the note's own pipe must survive intact");
    }

    #[test]
    fn two_files_sharing_one_pb_number_keep_their_own_distinct_status_and_note() {
        // The exact scenario docs/PLAYBOOK_EXECUTION_TRIAGE.md's own
        // parser test fixture documents: two different files, same
        // leading number, genuinely different content and status. Keying
        // by number instead of filename would let one overwrite the
        // other in the lookup map.
        let doc = "\
| 90 | `playbook_90_a.yml` | `[~]` | run 2026-07-25 |
| 90 | `playbook_90_b.yml` | `[ ]` | different content, same number |
";
        let rows = parse_triage_rows(doc);
        let (status_a, note_a) = rows.get("playbook_90_a.yml").cloned().unwrap();
        let (status_b, note_b) = rows.get("playbook_90_b.yml").cloned().unwrap();
        assert_eq!(status_a.as_deref(), Some("[~]"));
        assert_eq!(note_a.as_deref(), Some("run 2026-07-25"));
        assert_eq!(status_b.as_deref(), Some("[ ]"));
        assert_eq!(note_b.as_deref(), Some("different content, same number"));
    }

    #[test]
    fn scan_pbs_finds_this_very_repo_s_real_numbered_playbooks() {
        // Repo root is 5 levels up from this crate's manifest dir:
        // workspace/bahyway_v4/crates/enkimdb -> workspace/bahyway_v4 -> workspace -> repo root.
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap() // crates/
            .parent().unwrap() // workspace/bahyway_v4/
            .parent().unwrap() // workspace/
            .parent().unwrap() // repo root
            .to_path_buf();
        let triage = repo_root.join("docs/PLAYBOOK_EXECUTION_TRIAGE.md");
        let found = scan_pbs(&repo_root, &triage).expect("scan_pbs should succeed against the real repo");
        assert!(found.iter().any(|p| p.number == 269), "expected to find real PB-269 on disk");
        // Two real files CAN legitimately share one PB number (the triage
        // doc's own fixture documents this: playbook_90_a.yml /
        // playbook_90_b.yml, "different content, same number") -- identity
        // here is per FILE, never per bare number. What must never
        // duplicate is the (file) path itself: scan_playbooks walks each
        // real file once.
        let mut files: Vec<&str> = found.iter().map(|p| p.file.as_str()).collect();
        let before = files.len();
        files.sort_unstable();
        files.dedup();
        assert_eq!(files.len(), before, "scan_pbs must not scan the same real file twice");
    }
}
