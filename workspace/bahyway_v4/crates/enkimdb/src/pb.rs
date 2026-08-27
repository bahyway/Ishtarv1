//! PB scanning — real, numbered playbooks as a scannable profile,
//! cross-referenced against `docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md`'s own
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
            Some(PbProfile {
                number,
                file: a.name,
                path: a.path,
                status,
                summary,
            })
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

/// Real, epoch-seconds stamp for an auto-written row's Note cell -- no
/// human-readable calendar date, deliberately: this crate carries no
/// date-formatting dependency, and epoch seconds still lets anyone
/// reading a row confirm exactly when it was written, honestly, without
/// pulling one in just for cosmetics.
fn epoch_stamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

const AUTO_SECTION_HEADER: &str = "## Auto-discovered — not yet triaged";

/// Appends one `[ ]` row per real playbook that has no triage row yet
/// (`PbProfile.status.is_none()` -- i.e. `scan_pbs` found the file on
/// disk but `parse_triage_rows` found no matching File cell for it),
/// under a dedicated auto-managed section this function creates once and
/// reuses on every later call. **Never edits or removes a row that
/// already exists** -- not one this function wrote earlier, and not one
/// a human wrote by hand; a PB this function already gave a row to is
/// never re-appended, because by the time it's called again `scan_pbs`
/// will have re-parsed that same row and given it `status: Some(_)`.
///
/// Returns how many rows were appended (0 is the common case -- most
/// Corpus pulses mint nothing new). A missing `triage_doc` is created
/// fresh with just the auto section, matching `scan_pbs`'s own "a
/// missing doc is not an error" stance.
pub fn append_untriaged_rows(
    triage_doc: &Path,
    profiles: &[PbProfile],
) -> std::io::Result<usize> {
    let new_rows: Vec<&PbProfile> = profiles.iter().filter(|p| p.status.is_none()).collect();
    if new_rows.is_empty() {
        return Ok(0);
    }

    let existing = fs::read_to_string(triage_doc).unwrap_or_default();
    let mut out = existing;

    if !out.contains(AUTO_SECTION_HEADER) {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
        out.push_str(AUTO_SECTION_HEADER);
        out.push_str(
            "\n\nRows below are appended automatically (anu-governor's Corpus scan, \
`enkimdb::pb::append_untriaged_rows`) whenever a real, numbered playbook has \
no row anywhere above -- move a row up into its real phase section by hand \
once it's actually been triaged. This section never edits or deletes a row \
once written; only a human promotes one out of it.\n\n",
        );
        out.push_str("| # | File | Status | Note |\n|---|------|--------|------|\n");
    }

    for p in &new_rows {
        out.push_str(&format!(
            "| {} | `{}.yml` | `[ ]` | auto-added at epoch {} — real file on disk, no triage row yet |\n",
            p.number,
            p.file,
            epoch_stamp(),
        ));
    }

    if let Some(parent) = triage_doc.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(triage_doc, out)?;
    Ok(new_rows.len())
}

/// Rewrites one existing row's Status and Note cells in place after a
/// **real** playbook run (never call this after a `--check`/simulate
/// run -- a status cell means "this ran for real" everywhere else in the
/// doc, and this function has no way to tell a real run from a simulated
/// one itself; that judgment belongs to the caller). Matches by the File
/// cell, the same identity `parse_triage_rows` uses, so two files sharing
/// one PB number each keep their own row. Every other line in the
/// document -- every other row, every hand-written paragraph -- passes
/// through byte-for-byte unchanged.
///
/// `status` is the bare marker (`x`, `~`, or a `SKIP`-shaped string),
/// wrapped in the doc's own backtick-bracket convention automatically.
/// Returns `Ok(true)` if a row for `file` (bare stem, no `.yml`) was
/// found and rewritten, `Ok(false)` if no row exists yet for it (the
/// caller should have run `append_untriaged_rows` first).
pub fn update_triage_status(
    triage_doc: &Path,
    file: &str,
    status: &str,
    note: &str,
) -> std::io::Result<bool> {
    let existing = fs::read_to_string(triage_doc)?;
    let target_filename = format!("{file}.yml");
    let mut found = false;

    let rewritten: Vec<String> = existing
        .lines()
        .map(|line| {
            if found || !line.trim_start().starts_with('|') {
                return line.to_string();
            }
            let cells: Vec<&str> = line.splitn(5, '|').collect();
            if cells.len() < 4 || cells[1].trim().parse::<u32>().is_err() {
                return line.to_string();
            }
            let filename = cells[2].trim().trim_matches('`');
            if filename != target_filename {
                return line.to_string();
            }
            found = true;
            let status_cell = if status.starts_with('[') {
                format!("`{status}`")
            } else {
                format!("`[{status}]`")
            };
            format!("| {} | {} | {} | {} |", cells[1].trim(), cells[2].trim(), status_cell, note)
        })
        .collect();

    if !found {
        return Ok(false);
    }
    fs::write(triage_doc, rewritten.join("\n") + "\n")?;
    Ok(true)
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
        let (_, summary) = rows
            .get("playbook_42_x.yml")
            .cloned()
            .expect("row 42 must parse");
        assert!(
            summary.unwrap().contains("A\\|B"),
            "the note's own pipe must survive intact"
        );
    }

    #[test]
    fn two_files_sharing_one_pb_number_keep_their_own_distinct_status_and_note() {
        // The exact scenario docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md's own
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
            .parent()
            .unwrap() // crates/
            .parent()
            .unwrap() // workspace/bahyway_v4/
            .parent()
            .unwrap() // workspace/
            .parent()
            .unwrap() // repo root
            .to_path_buf();
        let triage = repo_root.join("docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md");
        let found =
            scan_pbs(&repo_root, &triage).expect("scan_pbs should succeed against the real repo");
        assert!(
            found.iter().any(|p| p.number == 269),
            "expected to find real PB-269 on disk"
        );
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
        assert_eq!(
            files.len(),
            before,
            "scan_pbs must not scan the same real file twice"
        );
    }

    fn scratch_doc(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("enkimdb_pb_triage_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        dir.join("TRIAGE.md")
    }

    #[test]
    fn append_untriaged_rows_adds_a_row_for_a_pb_with_no_status_yet() {
        let doc = scratch_doc("append_new");
        fs::write(&doc, "# existing doc\n\n| 90 | `playbook_90_a.yml` | `[x]` | already triaged |\n").unwrap();

        let profiles = vec![
            PbProfile { number: 90, file: "playbook_90_a".into(), path: String::new(), status: Some("[x]".into()), summary: Some("already triaged".into()) },
            PbProfile { number: 700, file: "playbook_700_new_thing".into(), path: String::new(), status: None, summary: None },
        ];
        let added = append_untriaged_rows(&doc, &profiles).unwrap();
        assert_eq!(added, 1, "only the untriaged PB-700 should get a new row");

        let text = fs::read_to_string(&doc).unwrap();
        assert!(text.contains("already triaged"), "the existing row must survive untouched");
        assert!(text.contains(AUTO_SECTION_HEADER));
        assert!(text.contains("playbook_700_new_thing.yml"));
        assert!(text.contains("| 700 |"));

        let rows = parse_triage_rows(&text);
        let (status, _) = rows.get("playbook_700_new_thing.yml").cloned().unwrap();
        assert_eq!(status.as_deref(), Some("[ ]"), "an auto-added row must start unrun");
    }

    #[test]
    fn append_untriaged_rows_is_idempotent_once_a_row_exists() {
        let doc = scratch_doc("idempotent");
        let profiles = vec![PbProfile {
            number: 701,
            file: "playbook_701_x".into(),
            path: String::new(),
            status: None,
            summary: None,
        }];
        assert_eq!(append_untriaged_rows(&doc, &profiles).unwrap(), 1);

        // A second Corpus pulse re-scans; scan_pbs would now find this
        // exact row (status: Some("[ ]")) and hand back status:Some, not
        // None -- simulate that real re-scan result here directly.
        let rescanned = vec![PbProfile {
            number: 701,
            file: "playbook_701_x".into(),
            path: String::new(),
            status: Some("[ ]".into()),
            summary: Some("auto-added at epoch 0 — real file on disk, no triage row yet".into()),
        }];
        assert_eq!(
            append_untriaged_rows(&doc, &rescanned).unwrap(),
            0,
            "a PB already given a row must never be re-appended"
        );
        let text = fs::read_to_string(&doc).unwrap();
        assert_eq!(text.matches("playbook_701_x.yml").count(), 1);
    }

    #[test]
    fn append_untriaged_rows_creates_a_missing_doc_fresh() {
        let doc = scratch_doc("missing_doc").parent().unwrap().join("does_not_exist_yet.md");
        let profiles = vec![PbProfile {
            number: 702,
            file: "playbook_702_x".into(),
            path: String::new(),
            status: None,
            summary: None,
        }];
        assert_eq!(append_untriaged_rows(&doc, &profiles).unwrap(), 1);
        assert!(doc.exists());
        assert!(fs::read_to_string(&doc).unwrap().contains("playbook_702_x.yml"));
    }

    #[test]
    fn update_triage_status_rewrites_only_the_matching_row() {
        let doc = scratch_doc("update_status");
        fs::write(
            &doc,
            "\
# doc
| 90 | `playbook_90_a.yml` | `[ ]` | not run yet |
| 91 | `playbook_91_b.yml` | `[ ]` | not run yet |
",
        )
        .unwrap();

        let updated = update_triage_status(
            &doc,
            "playbook_90_a",
            "x",
            "run for real 2026-08-28, ok=7 changed=3 failed=0",
        )
        .unwrap();
        assert!(updated);

        let text = fs::read_to_string(&doc).unwrap();
        let rows = parse_triage_rows(&text);
        let (s90, n90) = rows.get("playbook_90_a.yml").cloned().unwrap();
        assert_eq!(s90.as_deref(), Some("[x]"));
        assert_eq!(n90.as_deref(), Some("run for real 2026-08-28, ok=7 changed=3 failed=0"));

        // The untouched row must survive byte-for-byte.
        let (s91, n91) = rows.get("playbook_91_b.yml").cloned().unwrap();
        assert_eq!(s91.as_deref(), Some("[ ]"));
        assert_eq!(n91.as_deref(), Some("not run yet"));
    }

    #[test]
    fn update_triage_status_returns_false_when_no_row_exists() {
        let doc = scratch_doc("update_missing");
        fs::write(&doc, "# doc\n").unwrap();
        let updated = update_triage_status(&doc, "playbook_999_ghost", "x", "irrelevant").unwrap();
        assert!(!updated, "there is no row to update, so this must report false, not fabricate one");
    }

    #[test]
    fn update_triage_status_accepts_a_prebracketed_status_too() {
        let doc = scratch_doc("update_bracketed");
        fs::write(&doc, "| 5 | `playbook_5_x.yml` | `[ ]` | pending |\n").unwrap();
        assert!(update_triage_status(&doc, "playbook_5_x", "[~]", "partial run").unwrap());
        let rows = parse_triage_rows(&fs::read_to_string(&doc).unwrap());
        let (status, _) = rows.get("playbook_5_x.yml").cloned().unwrap();
        assert_eq!(status.as_deref(), Some("[~]"));
    }
}
