//! uruinimgina-cli — headless face for Uruinimgina's Reform & Pulse engine
//! (`anu_governor::docpulse`). Same engine the Uruinimgina tab in the native
//! `anu-governor` app drives (feature "gui") -- this binary drives it from
//! a TOML config and a terminal instead, for hosts that want to run
//! document ingestion without a windowing session (e.g. a headless build
//! on Fedora Workstation44).
//!
//! Usage: `uruinimgina-cli [config.toml]` (default: `uruinimgina.toml`).
//! See `docs/20_meta_engine/URUINIMGINA_EXTERNAL_DOCS.md` for what each
//! field means and the hard requirements (devVM branch, main branch must
//! exist, archive outside repo).
//!
//! CSR-08 (Architect Sovereignty): a HALT (major failure -- blob audit,
//! a failed git step) never auto-resolves. This binary reads a line from
//! stdin at each halt -- "r"/"retry", "s"/"skip" -- anything else,
//! including EOF (e.g. stdin redirected from /dev/null), aborts. The
//! Architect is always the one who decides; the safe default when no one
//! is there to ask is to stop, not to press on.

use anu_governor::docpulse::{spawn_docpulse, DocPulseCfg};
use anu_governor::model::{Ctl, RunnerEvent};
use serde::Deserialize;
use std::io::BufRead;

#[derive(Deserialize)]
struct CliConfig {
    repo: String,
    message: String,
    #[serde(default = "default_limit_mb")]
    limit_mb: u64,
    archive: String,
    ingest_manifest_dir: String,
    #[serde(default = "default_chronicle_dir")]
    chronicle_dir: String,
    enkiddb_root: String,
    #[serde(default)]
    official_repo: String,
    #[serde(default)]
    official_repo_subdir: String,
    #[serde(default)]
    official_repo_branch: String,
    #[serde(default)]
    push_to_official_repo: bool,
    /// Which EnkiDDB tribe this pulse mints into. "docs" (default) =
    /// `enkiddb::DOCS_TRIBE_ID`, the sealed BahyWay.Ecosystem v4.0
    /// documentation corpus. "architect_docs" =
    /// `enkiddb::ARCHITECT_DOCS_TRIBE_ID`, the Architect's own personal
    /// daily-work corpus (ideas/drafts/discussions, a different source
    /// repo entirely) -- kept queryable but never mixed with the sealed
    /// corpus. Any other value is a hard error, not a silent fallback.
    #[serde(default = "default_docs_tribe")]
    docs_tribe: String,
}

fn default_limit_mb() -> u64 {
    90
}
fn default_chronicle_dir() -> String {
    "uruinimgina_chronicle".into()
}
fn default_docs_tribe() -> String {
    "docs".into()
}

fn resolve_tribe_id(name: &str) -> Result<u16, String> {
    match name {
        "docs" => Ok(enkiddb::DOCS_TRIBE_ID),
        "architect_docs" => Ok(enkiddb::ARCHITECT_DOCS_TRIBE_ID),
        other => Err(format!(
            "unknown docs_tribe '{other}' -- expected \"docs\" or \"architect_docs\""
        )),
    }
}

fn build_cfg(cli: CliConfig) -> Result<DocPulseCfg, String> {
    let docs_tribe_id = resolve_tribe_id(&cli.docs_tribe)?;
    Ok(DocPulseCfg {
        repo_path: cli.repo,
        message: cli.message,
        limit_mb: cli.limit_mb,
        archive_dir: cli.archive,
        ingest_manifest_dir: cli.ingest_manifest_dir,
        chronicle_dir: cli.chronicle_dir,
        enkiddb_output_root: cli.enkiddb_root,
        official_repo_path: cli.official_repo,
        official_repo_subdir: cli.official_repo_subdir,
        official_repo_branch: cli.official_repo_branch,
        auto_push_to_official_repo: cli.push_to_official_repo,
        docs_tribe_id,
    })
}

fn load_cfg(path: &str) -> Result<DocPulseCfg, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
    let cli: CliConfig = toml::from_str(&text).map_err(|e| format!("parsing {path}: {e}"))?;
    build_cfg(cli)
}

/// Reads one decision line from stdin at a halt. EOF or anything not
/// recognized as retry/skip is treated as Abort -- the safe default.
fn read_ctl_decision() -> Ctl {
    print!("Uruinimgina halted -- (r)etry / (s)kip-continue / (a)bort? [a] ");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut line = String::new();
    let stdin = std::io::stdin();
    if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
        return Ctl::Abort; // EOF -- no one there to ask
    }
    match line.trim().to_ascii_lowercase().as_str() {
        "r" | "retry" => Ctl::Retry,
        "s" | "skip" => Ctl::SkipContinue,
        _ => Ctl::Abort,
    }
}

fn main() {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "uruinimgina.toml".into());
    let cfg = match load_cfg(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("uruinimgina-cli: {e}");
            std::process::exit(2);
        }
    };

    let (tx, rx) = crossbeam_channel::unbounded();
    let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded();
    let handle = spawn_docpulse(cfg, tx, ctl_rx);

    let mut exit_code = 0;
    while let Ok(ev) = rx.recv() {
        match ev {
            RunnerEvent::Log(s) => println!("{s}"),
            RunnerEvent::PbStarted(_) => {}
            RunnerEvent::PbOk(_) => {}
            RunnerEvent::PbWarned(_, err) => {
                println!("⚠ {}: {}", err.error_type, err.message);
            }
            RunnerEvent::PbFailed(_, err) => {
                println!("✗ HALT [{}] {}: {}", err.error_type, err.task, err.message);
                let decision = read_ctl_decision();
                let _ = ctl_tx.send(decision);
                if decision == Ctl::Abort {
                    exit_code = 1;
                }
            }
            RunnerEvent::Finished => {
                println!("𒁾 Uruinimgina pulse finished.");
            }
            RunnerEvent::Aborted => {
                println!("✗ Uruinimgina pulse aborted.");
                exit_code = 1;
            }
        }
    }
    let _ = handle.join();
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_toml_with_defaults() {
        let toml_text = r#"
            repo = "/home/bfadam/Forge/bahyway_v4_docs"
            message = "daily pulse"
            archive = "/home/bfadam/Forge/uruinimgina_archive"
            ingest_manifest_dir = "/home/bfadam/Forge/uruinimgina_manifests"
            enkiddb_root = "/home/bfadam/Forge/enkiddb_local"
        "#;
        let cli: CliConfig = toml::from_str(toml_text).unwrap();
        let cfg = build_cfg(cli).unwrap();
        assert_eq!(cfg.repo_path, "/home/bfadam/Forge/bahyway_v4_docs");
        assert_eq!(cfg.limit_mb, 90);
        assert_eq!(cfg.chronicle_dir, "uruinimgina_chronicle");
        assert!(
            cfg.official_repo_path.is_empty(),
            "official repo must default empty -- skip landing stage"
        );
        assert!(!cfg.auto_push_to_official_repo);
        assert_eq!(
            cfg.docs_tribe_id,
            enkiddb::DOCS_TRIBE_ID,
            "docs_tribe must default to the sealed corpus, not architect_docs"
        );
    }

    #[test]
    fn parses_full_toml_with_official_repo_landing() {
        let toml_text = r#"
            repo = "/docs"
            message = "m"
            archive = "/archive"
            ingest_manifest_dir = "/manifests"
            enkiddb_root = "/enkiddb"
            official_repo = "/official/EnkiDB"
            official_repo_subdir = "docs/bahyway-v4"
            official_repo_branch = "docpulse-landing"
            push_to_official_repo = true
        "#;
        let cli: CliConfig = toml::from_str(toml_text).unwrap();
        let cfg = build_cfg(cli).unwrap();
        assert_eq!(cfg.official_repo_path, "/official/EnkiDB");
        assert_eq!(cfg.official_repo_branch, "docpulse-landing");
        assert!(cfg.auto_push_to_official_repo);
    }

    #[test]
    fn architect_docs_tribe_mints_into_the_separate_tribe() {
        let toml_text = r#"
            repo = "/home/bfadam/Forge/bahyway_v4"
            message = "architect daily-work pulse"
            archive = "/home/bfadam/Forge/uruinimgina_archive"
            ingest_manifest_dir = "/home/bfadam/Forge/uruinimgina_manifests"
            enkiddb_root = "/home/bfadam/Forge/enkiddb_local"
            docs_tribe = "architect_docs"
        "#;
        let cli: CliConfig = toml::from_str(toml_text).unwrap();
        let cfg = build_cfg(cli).unwrap();
        assert_eq!(cfg.docs_tribe_id, enkiddb::ARCHITECT_DOCS_TRIBE_ID);
        assert_ne!(
            enkiddb::ARCHITECT_DOCS_TRIBE_ID,
            enkiddb::DOCS_TRIBE_ID,
            "architect_docs must never collide with the sealed docs tribe"
        );
    }

    #[test]
    fn unknown_docs_tribe_is_a_hard_error_not_a_silent_default() {
        let toml_text = r#"
            repo = "/docs"
            message = "m"
            archive = "/archive"
            ingest_manifest_dir = "/manifests"
            enkiddb_root = "/enkiddb"
            docs_tribe = "nonsense"
        "#;
        let cli: CliConfig = toml::from_str(toml_text).unwrap();
        let err = match build_cfg(cli) {
            Err(e) => e,
            Ok(_) => panic!("expected an error for an unknown docs_tribe value"),
        };
        assert!(
            err.contains("nonsense"),
            "error must name the bad value: {err}"
        );
    }

    #[test]
    fn missing_required_field_fails_to_parse() {
        let toml_text = r#"
            message = "m"
            archive = "/archive"
        "#;
        let result: Result<CliConfig, _> = toml::from_str(toml_text);
        assert!(
            result.is_err(),
            "missing 'repo' must fail to parse, not default silently"
        );
    }
}
