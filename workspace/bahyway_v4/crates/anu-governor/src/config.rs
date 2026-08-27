//! anu-governor.toml — the governor's decree: which playbooks, in which order,
//! with which parameters, and the severity + remedy knowledge base.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub run: RunCfg,
    #[serde(default)]
    pub parameters: Vec<Param>,
    #[serde(default)]
    pub severity: Vec<SeverityRule>,
    #[serde(default)]
    pub remedy: Vec<RemedyRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCfg {
    /// Directory holding the numbered playbooks.
    pub playbook_dir: String,
    /// Ordered corpus (relative to playbook_dir). Ignored at load time when
    /// `triage_doc` is set — see that field's doc comment.
    pub playbooks: Vec<String>,
    /// Optional inventory path passed as `-i`.
    #[serde(default)]
    pub inventory: Option<String>,
    /// If set, `Config::load` overwrites `playbooks` above by parsing this
    /// markdown file's own `| # | File | Status | Note |` tables (relative
    /// to the directory this decree file lives in) — the exact document,
    /// and equivalent parsing logic (awk there, this module here), that
    /// `playbook_IsimudEngine.yml`'s own PLAN phase reads. Rows whose
    /// Status column is `SKIP` are dropped; everything else is kept in
    /// document order, duplicates (two files sharing one PB number) and
    /// all. The point: one hand-maintained playbook list (the triage doc)
    /// instead of two that can silently drift apart.
    #[serde(default)]
    pub triage_doc: Option<String>,
    /// Simulate mode: no ansible spawned; a scripted run drives the UI.
    #[serde(default)]
    pub simulate: bool,
    /// In simulate mode, which index raises a MAJOR failure (for rehearsal).
    #[serde(default)]
    pub simulate_fail_at: Option<usize>,
    /// Where generated fix playbooks are written.
    #[serde(default = "default_fix_dir")]
    pub fix_dir: String,
    /// Where sealed reports are written.
    #[serde(default = "default_report_dir")]
    pub report_dir: String,
    /// Ed25519 seal key file (created on first use).
    #[serde(default = "default_seal_key")]
    pub seal_key: String,
    /// Append-only JSONL chronicle directory (StoryEngine-ready).
    #[serde(default = "default_chronicle_dir")]
    pub chronicle_dir: String,
    /// Real mode: pass --check to ansible-playbook (dry-run).
    #[serde(default)]
    pub check_mode: bool,

    // ── Run-confirmation registry (2026-07-29) ──────────────────────────
    //
    // Off by default -- same rationale as PB-265's own `vault_check_enabled`:
    // most sandbox/dev runs have no real Sargon vault set up yet, and this
    // playbook engine must stay usable without one. When enabled, the
    // ENTIRE run (every playbook in the corpus) is gated on a real,
    // cryptographically verified passport clearing `vault_min_privilege`
    // BEFORE anything executes -- CSR-08 in the inverse direction:
    // automation doesn't touch real infra without a human-ratified
    // identity. `KUPRU_VAULT_PASSPHRASE` is read from the environment,
    // never from this file (would defeat the point of a passphrase).
    #[serde(default)]
    pub vault_check_enabled: bool,
    #[serde(default)]
    pub vault_path: Option<String>,
    /// 5 = Administrator tier, same default already used by PB-265 and
    /// `kupru_vault::namespace_authz::RESTRICTED_NAMESPACE_MIN_PRIVILEGE`.
    #[serde(default = "default_vault_min_privilege")]
    pub vault_min_privilege: u8,
    /// EnkiMDB Write Node host:port to emit the run-confirmation record
    /// to after each run. Absent = best-effort skipped (logged, not a
    /// hard failure) -- EnkiMDB may not be reachable from every
    /// environment AnuGovernor runs in (e.g. the authoring sandbox).
    #[serde(default)]
    pub enkimdb_write_host: Option<String>,
    #[serde(default = "default_enkimdb_write_port")]
    pub enkimdb_write_port: u16,
}

fn default_vault_min_privilege() -> u8 {
    5
}
fn default_enkimdb_write_port() -> u16 {
    7201
}

fn default_fix_dir() -> String {
    "fixes".into()
}
fn default_report_dir() -> String {
    "reports".into()
}
fn default_seal_key() -> String {
    "akkadian_seal.key".into()
}
fn default_chronicle_dir() -> String {
    "chronicle".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub key: String,
    pub value: String,
}

/// If a failure message contains `pattern`, classify with `severity`.
/// Unmatched failures default to MAJOR — caution is law.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityRule {
    pub pattern: String,
    pub severity: String, // "major" | "warning"
    #[serde(default)]
    pub error_type: Option<String>,
}

/// Remedy knowledge base entry: your debugging history as a permanent asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemedyRule {
    pub id: String,
    pub pattern: String,
    pub diagnosis: String,
    pub solution: String,
    /// Fix playbook template. Placeholders: {playbook} {task} {message}
    pub fix_template: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read config {}", path.display()))?;
        let mut cfg: Config = toml::from_str(&text).context("config parse error")?;

        if let Some(triage) = cfg.run.triage_doc.clone() {
            let base = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let triage_path = base.join(&triage);
            cfg.run.playbooks = playbooks_from_triage(&triage_path)
                .with_context(|| format!("cannot parse triage doc {}", triage_path.display()))?;
        }

        Ok(cfg)
    }

    pub fn classify(&self, error_type: &str, message: &str) -> Severity {
        for r in &self.severity {
            let hay = format!("{error_type} {message}");
            if hay.to_lowercase().contains(&r.pattern.to_lowercase()) {
                return if r.severity.eq_ignore_ascii_case("warning") {
                    Severity::Warning
                } else {
                    Severity::Major
                };
            }
        }
        Severity::Major
    }

    pub fn find_remedy(&self, message: &str) -> Option<&RemedyRule> {
        self.remedy
            .iter()
            .find(|r| message.to_lowercase().contains(&r.pattern.to_lowercase()))
    }
}

/// Parses a playbook-execution-triage markdown doc's own tables into an
/// ordered file list, skipping `SKIP` rows. Mirrors
/// `playbook_IsimudEngine.yml`'s PARSE step exactly:
/// `awk '/^\| *[0-9]+ *\|/ { ... }'` splitting each row on `|`, taking
/// field 2 as the PB number, field 3 as the file, field 4 as the status,
/// trimming whitespace and backticks, keeping only rows whose file ends
/// in `.yml`.
pub fn playbooks_from_triage(path: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read triage doc {}", path.display()))?;
    let mut out = Vec::new();
    for line in text.lines() {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.split('|').collect();
        if cells.len() < 4 {
            continue;
        }
        let num = cells[1].trim();
        if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let file = cells[2].trim().trim_matches('`');
        let status = cells[3].trim().trim_matches('`');
        if file.ends_with(".yml") && status != "SKIP" {
            out.push(file.to_string());
        }
    }
    Ok(out)
}

use crate::model::Severity;

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"
[run]
playbook_dir = "playbooks"
playbooks = ["01_a.yml", "02_b.yml"]
simulate = true
simulate_fail_at = 1

[[parameters]]
key = "env"
value = "production"

[[severity]]
pattern = "deprecated"
severity = "warning"
error_type = "DeprecatedParam"

[[remedy]]
id = "R-041"
pattern = "already bound"
diagnosis = "orphan process holds the port"
solution = "stop orphan, add pre-start guard"
fix_template = "# fix for {playbook} / {task}\n"
"##;

    #[test]
    fn parses_and_classifies() {
        let cfg: Config = toml::from_str(SAMPLE).unwrap();
        assert_eq!(cfg.run.playbooks.len(), 2);
        assert_eq!(cfg.parameters[0].key, "env");
        assert_eq!(
            cfg.classify("x", "param deprecated since 2.14"),
            Severity::Warning
        );
        // unmatched failures are MAJOR by law
        assert_eq!(cfg.classify("Boom", "unknown explosion"), Severity::Major);
        assert!(cfg.find_remedy("port 7101 already bound").is_some());
        assert!(cfg.find_remedy("nothing matches").is_none());
    }

    #[test]
    fn parses_triage_doc_skipping_skip_rows_and_keeping_duplicates() {
        let doc = "\
## Phase 0 — Foundation

**Declaration:** test phase.

| # | File | Status | Note |
|---|------|--------|------|
| 90 | `playbook_90_a.yml` | `[~]` | run 2026-07-25 |
| 90 | `playbook_90_b.yml` | `[ ]` | different content, same number |
| 117 | `playbook_117_gap.yml` | `SKIP` | gap-diagnostic only |
| 117 | `playbook_117_v2.yml` | `[ ]` | run this one |
";
        let dir = std::env::temp_dir().join("shk_triage_test");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("triage.md");
        std::fs::write(&p, doc).unwrap();
        let list = playbooks_from_triage(&p).unwrap();
        assert_eq!(
            list,
            vec![
                "playbook_90_a.yml".to_string(),
                "playbook_90_b.yml".to_string(),
                "playbook_117_v2.yml".to_string(),
            ]
        );
    }
}
