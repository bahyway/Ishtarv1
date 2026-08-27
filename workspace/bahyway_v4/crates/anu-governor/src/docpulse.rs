//! docpulse.rs — 𒄩𒆷 HALA inside the governor (sovereign name as of
//! Phase 2 reconciliation, 2026-08-15; prior name "Uruinimgina"). Binary
//! name `uruinimgina-cli` and config file `uruinimgina.toml` kept
//! unchanged -- see docs/99_index/BAHYWAY_PHASE2_GLOSSARY.md "Hala" entry.
//! Reform (quarantine oversized files OUTSIDE the repo) → commit →
//! blob audit → pulse devVM→main → EnkiDDB ingestion manifest.
//!
//! Pure Rust port of uruinimgina.sh. Git is driven via std::process
//! (same boundary policy as ansible-playbook in runner.rs). Nothing
//! is ever deleted: quarantined files move to the archive with a
//! MANIFEST.txt, and every act is chronicled.
//!
//! Law: the blob audit HALTS the pulse (major) — the Architect
//! decides. Warnings (e.g. nothing to commit) proceed.

use crate::chronicle::Chronicle;
use crate::model::{Ctl, ErrorEvent, RunnerEvent, Severity};
use bahyway_core::TribeId;
use crossbeam_channel::{Receiver, Sender};
use enkidb_kaki::{IdentityKaki, Kaki, KakiMinter};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Stage names for the queue panel — the single place either the runner
/// or the UI needs to change if a stage is ever added, renamed, or reordered.
pub const STAGE_NAMES: [&str; 6] = [
    "Reform (quarantine)",
    "Commit",
    "Blob audit",
    "Pulse devVM→main",
    "EnkiDDB mint + manifest",
    "Official repo landing (gated)",
];

#[derive(Clone)]
pub struct DocPulseCfg {
    pub repo_path: String,           // the input field: any BahyWay repo
    pub message: String,             // commit message
    pub limit_mb: u64,               // quarantine threshold (default 90)
    pub archive_dir: String,         // OUTSIDE the repo
    pub ingest_manifest_dir: String, // where the JSONL audit copy lands
    pub chronicle_dir: String,
    /// Where this run's real EnkiDDB Tigris generation is materialized
    /// locally (`enkiddb::materialize_version`'s `root` argument). Every
    /// qualifying changed file gets a real Identity-Kaki minted via
    /// `enkiddb::WriteNode` here, not just a JSONL precursor record.
    pub enkiddb_output_root: String,
    /// Local clone of the OFFICIAL EnkiDB repo (bahyway/EnkiDB) — the
    /// personal/authoring repo at `repo_path` above is NEVER assumed to
    /// be that repo. Empty string = stage 5 is skipped entirely (opt-in,
    /// not required for the existing devVM->main single-repo flow).
    pub official_repo_path: String,
    /// Path inside the official repo the promoted docs land under, e.g.
    /// "docs/bahyway-v4".
    pub official_repo_subdir: String,
    /// Branch in the official repo this stage commits to — deliberately
    /// never "main"/"master" (checked, refused if so): a human-reviewed
    /// PR off this branch is the real gate into the official repo's
    /// trunk, on top of the push gate below.
    pub official_repo_branch: String,
    /// Off by default. When false, stage 5 copies + commits the promoted
    /// doc(s) into `official_repo_path` on `official_repo_branch` and
    /// STOPS — it never runs `git push` on a repo shared with other
    /// people unless explicitly told to. When true, it pushes that one
    /// branch (never force, never main) so a PR can be opened.
    pub auto_push_to_official_repo: bool,
    /// Which EnkiDDB tribe stage 5 mints into. Was hardcoded to
    /// `enkiddb::DOCS_TRIBE_ID` until 2026-08-01 -- now configurable so a
    /// pulse of the Architect's own personal/daily-work corpus (a
    /// DIFFERENT source repo entirely) can mint under
    /// `enkiddb::ARCHITECT_DOCS_TRIBE_ID` instead, keeping it queryable
    /// (`WHERE tribe=...`) but never mixed with sealed BahyWay.Ecosystem
    /// v4.0 documentation. Callers that want the old, single-corpus
    /// behavior just pass `enkiddb::DOCS_TRIBE_ID` here, unchanged.
    pub docs_tribe_id: u16,
}

/// Expands a leading `~` or `~/...` using `$HOME`. `Command` never invokes a
/// shell, so a literal `~` in a repo/archive path (as shown in the UI
/// blueprint's own example fields) would otherwise be passed to `git`
/// unexpanded and fail immediately with "No such file or directory".
fn expand_home(p: &str) -> String {
    if p == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| p.to_string());
    }
    if let Some(rest) = p.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home.trim_end_matches('/'), rest);
        }
    }
    p.to_string()
}

fn git(repo: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// One line of the doc-supersession registry: the last real Identity-Kaki
/// minted for a given repo-relative path. Read back on every pulse so a
/// document promoted again under the same path can be recognized as a
/// NEW version of something real, not a brand-new, unrelated document —
/// this is what lets stage 4 call `supersede_document` (ADR-014 Decision
/// 2) instead of minting an orphaned, unlinked identity every time.
#[derive(Serialize, Deserialize)]
struct DocKakiRegistryLine {
    path: String,
    kaki_hex: String,
    minted_at: String,
}

/// Loads the current (path -> last minted Kaki bytes) map from a JSONL
/// registry -- later lines win on a duplicate path, so re-reading after
/// an append always reflects the latest mint. A missing file is not an
/// error (first pulse ever against this repo): returns an empty map.
fn load_doc_kaki_registry(path: &Path) -> HashMap<String, [u8; 16]> {
    let mut out = HashMap::new();
    let Ok(text) = std::fs::read_to_string(path) else {
        return out;
    };
    for line in text.lines() {
        let Ok(rec) = serde_json::from_str::<DocKakiRegistryLine>(line) else {
            continue;
        };
        let Ok(raw) = hex::decode(&rec.kaki_hex) else {
            continue;
        };
        let Ok(bytes): Result<[u8; 16], _> = raw.try_into() else {
            continue;
        };
        out.insert(rec.path, bytes);
    }
    out
}

#[derive(Serialize)]
struct IngestRecord<'a> {
    ts: String,
    repo: &'a str,
    commit: &'a str,
    path: &'a str,
    bytes: u64,
    sha256: String,
    kaki_type_hint: &'a str, // "0x02 Event -> document particle precursor"
}

pub fn spawn_docpulse(
    cfg: DocPulseCfg,
    tx: Sender<RunnerEvent>,
    ctl_rx: Receiver<Ctl>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || run(cfg, tx, ctl_rx))
}

fn run(cfg: DocPulseCfg, tx: Sender<RunnerEvent>, ctl_rx: Receiver<Ctl>) {
    let chr = Chronicle::open(&cfg.chronicle_dir).ok();
    let send = |ev: RunnerEvent| {
        if let Some(c) = &chr {
            let _ = c.record(&ev);
        }
        let _ = tx.send(ev);
    };
    let log = |s: String| send(RunnerEvent::Log(s));
    let repo = expand_home(cfg.repo_path.trim_end_matches('/'));
    let archive_dir = expand_home(&cfg.archive_dir);

    // Guard: archive OUTSIDE the repo; branch devVM
    if Path::new(&archive_dir).starts_with(&repo) {
        send(RunnerEvent::PbFailed(
            0,
            major(
                &repo,
                "guard",
                "ArchiveInsideRepo",
                "ARCHIVE_DIR must be outside the repository",
            ),
        ));
        wait_abort(&ctl_rx, &send);
        return;
    }
    match git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        Ok(b) if b.trim() == "devVM" => log("✓ on branch devVM".into()),
        Ok(b) => {
            send(RunnerEvent::PbFailed(
                0,
                major(
                    &repo,
                    "branch guard",
                    "WrongBranch",
                    &format!("expected devVM, on '{}'", b.trim()),
                ),
            ));
            wait_abort(&ctl_rx, &send);
            return;
        }
        Err(e) => {
            send(RunnerEvent::PbFailed(
                0,
                major(&repo, "git", "NotARepo", &e),
            ));
            wait_abort(&ctl_rx, &send);
            return;
        }
    }

    // ── Stage: HALA REFORM ───────────────────────────────────────
    send(RunnerEvent::PbStarted(0));
    log("𒁾 Reform: fencing .gitignore, quarantining oversized files".into());
    fence_gitignore(&repo);
    // Untrack every ALREADY-COMMITTED path matching a fenced pattern, at
    // any depth -- not the old hardcoded `git rm --cached target
    // node_modules`, which only ever matched those two names at the repo
    // ROOT and silently missed anything nested (e.g. `_DailyWorsk/
    // target/...`), leaving it tracked and pushed forever regardless of
    // `.gitignore`.
    if let Ok(listing) = git(&repo, &["ls-files"]) {
        let tracked: Vec<&str> = listing.lines().collect();
        let to_untrack = paths_to_untrack(&tracked);
        if !to_untrack.is_empty() {
            log(format!(
                "𒁾 untracking {} already-committed path(s) matching .gitignore fences",
                to_untrack.len()
            ));
            let mut args = vec!["rm", "-q", "--cached", "--"];
            args.extend(to_untrack.iter().map(|s| s.as_str()));
            let _ = git(&repo, &args);
        }
    }
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let repo_name = Path::new(&repo)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".into());
    let dest = PathBuf::from(&archive_dir).join(format!("{repo_name}_{stamp}"));
    let mut moved = 0usize;

    // tracked + untracked-but-addable
    for list_args in [
        vec!["ls-files"],
        vec!["ls-files", "--others", "--exclude-standard"],
    ] {
        if let Ok(listing) = git(&repo, &list_args) {
            for rel in listing.lines() {
                let full = Path::new(&repo).join(rel);
                let Ok(md) = std::fs::metadata(&full) else {
                    continue;
                };
                if !md.is_file() {
                    continue;
                }
                let mb = md.len() / (1024 * 1024);
                if mb > cfg.limit_mb {
                    let target = dest.join(rel);
                    if let Some(p) = target.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    if std::fs::rename(&full, &target).is_ok() {
                        let _ = git(&repo, &["rm", "-q", "--cached", "--ignore-unmatch", rel]);
                        append(&dest.join("MANIFEST.txt"), &format!("{mb} MB\t{rel}\n"));
                        log(format!("  quarantined: {rel} ({mb} MB)"));
                        moved += 1;
                    }
                }
            }
        }
    }
    log(if moved > 0 {
        format!("✓ reform: {moved} file(s) → {}", dest.display())
    } else {
        "✓ records already clean".into()
    });
    send(RunnerEvent::PbOk(0));

    // ── Stage: COMMIT ────────────────────────────────────────────
    send(RunnerEvent::PbStarted(1));
    let dirty = git(&repo, &["status", "--short"]).unwrap_or_default();
    // Stage 4 must know whether *this run* actually produced a new commit —
    // otherwise it would fall back to diffing whatever the previous, unrelated
    // commit happened to change and mislabel those docs as this pulse's output.
    let mut committed_now = false;
    if dirty.trim().is_empty() {
        send(RunnerEvent::PbWarned(
            1,
            warn_ev(
                &repo,
                "commit",
                "NothingToCommit",
                "working tree clean — proceeding by law",
            ),
        ));
    } else {
        let _ = git(&repo, &["add", "."]);
        match git(&repo, &["commit", "-m", &cfg.message]) {
            Ok(_) => {
                log(format!("✓ committed: {}", cfg.message));
                send(RunnerEvent::PbOk(1));
                committed_now = true;
            }
            Err(e) => {
                send(RunnerEvent::PbFailed(
                    1,
                    major(&repo, "commit", "CommitFailed", &e),
                ));
                if !architect_continues(&ctl_rx, &send) {
                    return;
                }
            }
        }
    }

    // ── Stage: BLOB AUDIT (halts on major) ───────────────────────
    send(RunnerEvent::PbStarted(2));
    let range = if git(&repo, &["rev-parse", "--verify", "origin/devVM"]).is_ok() {
        "origin/devVM..HEAD"
    } else {
        "HEAD"
    };
    match audit_blobs(&repo, range, 95 * 1024 * 1024) {
        Ok(big) if big.is_empty() => {
            log("✓ no oversized blobs in outgoing history".into());
            send(RunnerEvent::PbOk(2));
        }
        Ok(big) => {
            let msg = format!("oversized blobs in outgoing history: {}", big.join("; "));
            send(RunnerEvent::PbFailed(
                2,
                major(&repo, "blob audit", "OversizedBlob", &msg),
            ));
            // HALT — Architect decides (Skip is NOT offered meaningfully here;
            // Retry after manual amend, or Abort).
            if !architect_continues(&ctl_rx, &send) {
                return;
            }
        }
        Err(e) => {
            send(RunnerEvent::PbFailed(
                2,
                major(&repo, "blob audit", "AuditFailed", &e),
            ));
            if !architect_continues(&ctl_rx, &send) {
                return;
            }
        }
    }

    // ── Stage: PULSE devVM → main ────────────────────────────────
    send(RunnerEvent::PbStarted(3));
    for (label, args) in [
        ("push devVM", vec!["push", "origin", "devVM"]),
        ("checkout main", vec!["checkout", "main"]),
        ("pull main", vec!["pull", "origin", "main"]),
        ("merge devVM", vec!["merge", "devVM", "--no-edit"]),
        ("push main", vec!["push", "origin", "main"]),
        ("back to devVM", vec!["checkout", "devVM"]),
    ] {
        match git(&repo, &args) {
            Ok(_) => log(format!("✓ {label}")),
            Err(e) => {
                send(RunnerEvent::PbFailed(
                    3,
                    major(&repo, label, "GitStepFailed", &e),
                ));
                if !architect_continues(&ctl_rx, &send) {
                    return;
                }
            }
        }
    }
    send(RunnerEvent::PbOk(3));

    // ── Stage: ENKIDDB MINT + MANIFEST ───────────────────────────
    // Each qualifying changed document gets a REAL Identity-Kaki minted
    // via enkiddb::WriteNode/DocumentEmitter (the same pipeline
    // enkiddb-ingest's bulk CLI uses, single source of truth:
    // enkiddb::DOCS_TRIBE_ID) and is journaled + materialized as a real,
    // HeptaScript-queryable Tigris generation — not just a JSONL
    // precursor record waiting on a future network API. The JSONL is
    // kept too, as a cheap durable audit copy alongside the real mint.
    send(RunnerEvent::PbStarted(4));
    if !committed_now {
        log("· no new commit this run — nothing to mint".into());
        send(RunnerEvent::PbOk(4));
        if stage_official_repo_landing(&cfg, &repo, &repo_name, &[], &send, &log, &ctl_rx) {
            send(RunnerEvent::Finished);
        }
        return;
    }
    let commit = git(&repo, &["rev-parse", "HEAD"])
        .unwrap_or_default()
        .trim()
        .to_string();
    // `git show --name-only HEAD` (not `diff HEAD~1..HEAD`) also works on a
    // repo's very first commit, where `HEAD~1` doesn't exist and the old
    // `diff` form would silently fail and produce an empty (not erroring)
    // manifest via `unwrap_or_default()`.
    let changed =
        git(&repo, &["show", "--pretty=format:", "--name-only", "HEAD"]).unwrap_or_default();
    let qualifying: Vec<String> = changed
        .lines()
        .filter(|p| !p.trim().is_empty())
        .filter(|p| {
            let l = p.to_lowercase();
            l.ends_with(".md")
                || l.ends_with(".akk")
                || l.ends_with(".svg")
                || l.ends_with(".png")
                || l.ends_with(".toml")
                || l.ends_with(".hepta")
        })
        .map(str::to_string)
        .collect();

    let _ = std::fs::create_dir_all(&cfg.ingest_manifest_dir);
    let manifest_path =
        PathBuf::from(&cfg.ingest_manifest_dir).join(format!("enkiddb_ingest_{stamp}.jsonl"));
    // Which path last minted which Kaki -- read BEFORE this run's own
    // mints so a document promoted again under the same path is
    // recognized as a new version of something real (supersede_document),
    // never confused with this same run's freshly-minted rows below.
    let doc_registry_path = PathBuf::from(&cfg.ingest_manifest_dir).join("doc_kaki_registry.jsonl");
    let previous_kakis = load_doc_kaki_registry(&doc_registry_path);

    let minter = KakiMinter::new(TribeId::from_u16(cfg.docs_tribe_id));
    let mut write_node = enkiddb::WriteNode::new(minter, 64);
    let mut n = 0usize;
    for (i, rel) in qualifying.iter().enumerate() {
        let full = Path::new(&repo).join(rel);
        let Ok(bytes) = std::fs::read(&full) else {
            continue;
        };
        let rec = IngestRecord {
            ts: chrono::Utc::now().to_rfc3339(),
            repo: &repo_name,
            commit: &commit,
            path: rel,
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
            kaki_type_hint: "document-particle-precursor",
        };
        if let Ok(line) = serde_json::to_string(&rec) {
            append(&manifest_path, &format!("{line}\n"));
        }
        match write_node.ingest_document_from_path(&full, i as u32 + 1) {
            Ok(kaki) => {
                log(format!("  𒁾 minted {rel} → {kaki}"));
                n += 1;

                // ADR-014 Decision 2: if this path already had a real,
                // minted identity, this new mint SUPERSEDES it -- an
                // APPEND on the OLD identity recording why, never a
                // delete and never an unlinked, orphaned new particle.
                // cfg.message (the Architect's own commit message for
                // this pulse) is the rationale -- it already describes
                // what changed, for free, with no new UI needed.
                if let Some(old_bytes) = previous_kakis.get(rel) {
                    if let Ok(old_raw) = Kaki::from_bytes(*old_bytes) {
                        if let Ok(old_kaki) = IdentityKaki::try_from_kaki(old_raw) {
                            write_node.supersede_document(
                                old_kaki,
                                kaki,
                                &cfg.message,
                                i as u32 + 1,
                            );
                            log(format!(
                                "  ↻ superseded prior version of {rel}: {old_kaki} → {kaki} ({})",
                                cfg.message
                            ));
                        }
                    }
                }

                let reg_line = DocKakiRegistryLine {
                    path: rel.clone(),
                    kaki_hex: hex::encode(kaki.bytes()),
                    minted_at: chrono::Utc::now().to_rfc3339(),
                };
                if let Ok(json) = serde_json::to_string(&reg_line) {
                    append(&doc_registry_path, &format!("{json}\n"));
                }
            }
            Err(e) => log(format!("  ⚠ could not mint {rel}: {e}")),
        }
    }
    log(format!(
        "𒁾 audit manifest: {} record(s) → {}",
        qualifying.len(),
        manifest_path.display()
    ));

    if write_node.document_count() > 0 {
        match enkiddb::materialize_version(&write_node, &cfg.enkiddb_output_root, &stamp) {
            Ok((generation, stats)) => log(format!(
                "𒁾 EnkiDDB Tigris generation {stamp} materialized: {} entities → {}",
                stats.entities,
                generation.entities_path.display()
            )),
            Err(e) => log(format!("⚠ EnkiDDB materialize failed: {e}")),
        }
    }
    log(format!(
        "✓ {n}/{} document(s) minted as real EnkiDDB particles",
        qualifying.len()
    ));
    send(RunnerEvent::PbOk(4));

    // ── Stage: OFFICIAL REPO LANDING (gated, opt-in) ──────────────
    if stage_official_repo_landing(&cfg, &repo, &repo_name, &qualifying, &send, &log, &ctl_rx) {
        send(RunnerEvent::Finished);
    }
}

/// Copies this run's qualifying changed files into the OFFICIAL EnkiDB
/// repo (never assumed to be `repo`/`repo_path` above — a separate,
/// shared repo) and commits them on `official_repo_branch`. Never pushes
/// unless `auto_push_to_official_repo` is explicitly set, and never
/// targets main/master even then — landing there is a human PR's job.
/// A no-op (logged, not an error) when `official_repo_path` isn't
/// configured, so the existing single-repo devVM->main flow is
/// unaffected for callers that don't use this.
/// Returns `true` if the run should proceed to `RunnerEvent::Finished`
/// normally, `false` if this stage already ended the run itself (the
/// Architect chose Abort on a halt — `Aborted` was already sent, and the
/// caller must not also send `Finished` on top of it).
#[allow(clippy::too_many_arguments)]
fn stage_official_repo_landing(
    cfg: &DocPulseCfg,
    source_repo: &str,
    source_repo_name: &str,
    qualifying: &[String],
    send: &impl Fn(RunnerEvent),
    log: &impl Fn(String),
    ctl_rx: &Receiver<Ctl>,
) -> bool {
    send(RunnerEvent::PbStarted(5));
    if cfg.official_repo_path.trim().is_empty() {
        log("· official repo landing not configured — skipped".into());
        send(RunnerEvent::PbOk(5));
        return true;
    }
    if qualifying.is_empty() {
        log("· nothing to land — no qualifying documents this run".into());
        send(RunnerEvent::PbOk(5));
        return true;
    }
    let official_repo = expand_home(&cfg.official_repo_path);
    let branch = cfg.official_repo_branch.trim();
    if branch.is_empty()
        || branch.eq_ignore_ascii_case("main")
        || branch.eq_ignore_ascii_case("master")
    {
        send(RunnerEvent::PbFailed(5, major(&official_repo, "official repo landing", "UnsafeTargetBranch",
            "official_repo_branch must be set and must not be main/master — land on a review branch, let a PR gate the merge")));
        return architect_continues(ctl_rx, send);
    }

    if git(&official_repo, &["rev-parse", "--verify", branch]).is_ok() {
        if let Err(e) = git(&official_repo, &["checkout", branch]) {
            send(RunnerEvent::PbFailed(
                5,
                major(&official_repo, "checkout", "GitStepFailed", &e),
            ));
            return architect_continues(ctl_rx, send);
        }
    } else if let Err(e) = git(&official_repo, &["checkout", "-b", branch]) {
        send(RunnerEvent::PbFailed(
            5,
            major(&official_repo, "checkout -b", "GitStepFailed", &e),
        ));
        return architect_continues(ctl_rx, send);
    }

    let dest_root = PathBuf::from(&official_repo).join(&cfg.official_repo_subdir);
    let mut landed = 0usize;
    for rel in qualifying {
        let src = Path::new(source_repo).join(rel);
        let Ok(bytes) = std::fs::read(&src) else {
            continue;
        };
        let dest = dest_root.join(rel);
        if let Some(p) = dest.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        if std::fs::write(&dest, &bytes).is_ok() {
            log(format!("  staged: {} → {}", rel, dest.display()));
            landed += 1;
        }
    }
    if landed == 0 {
        log("· nothing actually copied — skipping commit".into());
        send(RunnerEvent::PbOk(5));
        return true;
    }

    let _ = git(&official_repo, &["add", cfg.official_repo_subdir.as_str()]);
    let msg = format!("docs: promote {landed} document(s) via Hala from {source_repo_name}");
    match git(&official_repo, &["commit", "-m", &msg]) {
        Ok(_) => log(format!(
            "✓ committed {landed} document(s) on branch {branch} in {official_repo}"
        )),
        Err(e) => {
            send(RunnerEvent::PbFailed(
                5,
                major(&official_repo, "commit", "CommitFailed", &e),
            ));
            return architect_continues(ctl_rx, send);
        }
    }

    if cfg.auto_push_to_official_repo {
        match git(&official_repo, &["push", "origin", branch]) {
            Ok(_) => log(format!(
                "✓ pushed {branch} to origin — open a PR to merge into main"
            )),
            Err(e) => {
                send(RunnerEvent::PbFailed(
                    5,
                    major(&official_repo, "push", "GitStepFailed", &e),
                ));
                return architect_continues(ctl_rx, send);
            }
        }
    } else {
        log(format!(
            "· staged locally on {branch} in {official_repo} — NOT pushed (auto_push_to_official_repo=false). \
             Push yourself when ready, or open a PR directly from that local branch."
        ));
    }
    send(RunnerEvent::PbOk(5));
    true
}

// ---------- helpers ----------

/// The single source of truth for what Reform fences and untracks --
/// `fence_gitignore` and `paths_to_untrack` both read this, so they can
/// never drift apart the way `.gitignore`'s patterns and the old
/// hardcoded `git rm --cached target node_modules` pathspec did (found
/// live 2026-08-01: that literal pathspec only ever matched a directory
/// named exactly "target" at the REPO ROOT, never a nested one like
/// `_DailyWorsk/target/`, so those files stayed tracked -- and pushed --
/// forever regardless of `.gitignore`).
const FENCED_PATTERNS: [&str; 5] = ["target/", "node_modules/", "_Backups*/", "*.log", ".npm/"];

fn fence_gitignore(repo: &str) {
    let gi = Path::new(repo).join(".gitignore");
    let existing = std::fs::read_to_string(&gi).unwrap_or_default();
    let mut add = String::new();
    for pat in FENCED_PATTERNS {
        if !existing.lines().any(|l| l.trim() == pat) {
            add.push_str(pat);
            add.push('\n');
        }
    }
    if !add.is_empty() {
        append(&gi, &add);
    }
}

/// Whether `path` (a `/`-separated repo-relative path, as `git ls-files`
/// prints it) has any component matching one of `FENCED_PATTERNS`,
/// AT ANY DEPTH -- not just at the repo root. This is what makes
/// `_DailyWorsk/target/debug/deps/...` match the same as a root-level
/// `target/...` would.
fn matches_fenced_pattern(path: &str) -> bool {
    let components: Vec<&str> = path.split('/').collect();
    for pat in FENCED_PATTERNS {
        if let Some(dirname) = pat.strip_suffix('/') {
            if let Some(prefix) = dirname.strip_suffix('*') {
                // e.g. "_Backups*/" -- any component starting with "_Backups"
                if components.iter().any(|c| c.starts_with(prefix)) {
                    return true;
                }
            } else if components.contains(&dirname) {
                // e.g. "target/" / "node_modules/" / ".npm/" -- an exact
                // component match anywhere in the path, not just the root.
                return true;
            }
        } else if let Some(suffix) = pat.strip_prefix('*') {
            // e.g. "*.log"
            if path.ends_with(suffix) {
                return true;
            }
        }
    }
    false
}

/// Every currently-tracked path (from `git ls-files`) that matches a
/// fenced pattern -- what Reform must `git rm --cached` so a file
/// already committed before `.gitignore` existed (or before it was
/// nested somewhere `target/node_modules` alone never caught) actually
/// stops being tracked, instead of silently surviving in the next
/// commit and every commit after it.
fn paths_to_untrack(tracked: &[&str]) -> Vec<String> {
    tracked
        .iter()
        .filter(|p| matches_fenced_pattern(p))
        .map(|p| p.to_string())
        .collect()
}

/// One `git cat-file --batch-check` process, not one `git cat-file -s`
/// spawn per object: `rev-list --objects` over real history can return
/// thousands of entries, and this stage exists specifically to catch
/// bloated histories — the input it's built for is exactly the input
/// that would make one-subprocess-per-object slowest.
fn audit_blobs(repo: &str, range: &str, limit: u64) -> Result<Vec<String>, String> {
    let objects = git(repo, &["rev-list", "--objects", range])?;
    let mut oid_paths: Vec<(String, String)> = Vec::new();
    for line in objects.lines() {
        let mut it = line.splitn(2, ' ');
        let (Some(oid), Some(path)) = (it.next(), it.next()) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        oid_paths.push((oid.to_string(), path.to_string()));
    }
    if oid_paths.is_empty() {
        return Ok(vec![]);
    }

    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "cat-file",
            "--batch-check=%(objectname) %(objecttype) %(objectsize)",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git cat-file --batch-check spawn failed: {e}"))?;

    let mut stdin = child.stdin.take().expect("stdin was piped");
    let oids: Vec<String> = oid_paths.iter().map(|(oid, _)| oid.clone()).collect();
    // Written from a separate thread while stdout is read below: with
    // thousands of objects the input can exceed the pipe buffer, and writing
    // it all before reading stdout would deadlock against git blocking on a
    // full stdout buffer.
    let writer = std::thread::spawn(move || {
        for oid in &oids {
            let _ = writeln!(stdin, "{oid}");
        }
    });

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("stdout was piped")
        .read_to_string(&mut stdout)
        .map_err(|e| format!("reading git cat-file --batch-check output failed: {e}"))?;
    let _ = writer.join();
    let status = child
        .wait()
        .map_err(|e| format!("git cat-file --batch-check wait failed: {e}"))?;
    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut e) = child.stderr.take() {
            let _ = e.read_to_string(&mut stderr);
        }
        return Err(format!(
            "git cat-file --batch-check exited non-zero: {stderr}"
        ));
    }

    let mut sizes: HashMap<&str, u64> = HashMap::new();
    for line in stdout.lines() {
        let mut parts = line.split(' ');
        let (Some(oid), Some(ty), Some(sz)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if ty != "blob" {
            continue;
        }
        if let Ok(bytes) = sz.parse::<u64>() {
            sizes.insert(oid, bytes);
        }
    }

    Ok(oid_paths
        .iter()
        .filter_map(|(oid, path)| {
            sizes
                .get(oid.as_str())
                .filter(|&&bytes| bytes > limit)
                .map(|&bytes| format!("{} ({} MB)", path, bytes / 1_048_576))
        })
        .collect())
}

fn append(path: &Path, text: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(text.as_bytes());
    }
}

/// Canonical SHA-256 (the `sha2` crate is already resolved in this
/// workspace's lockfile via ed25519-dalek's own dependency tree, so this
/// costs nothing new to build) — the `IngestRecord.sha256` field is used
/// downstream for document identity/integrity, so it must be real SHA-256,
/// not a same-length stand-in from a different hash function.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn major(pb: &str, task: &str, ty: &str, msg: &str) -> ErrorEvent {
    ErrorEvent {
        pb_index: 0,
        playbook: pb.into(),
        task: task.into(),
        error_type: ty.into(),
        message: msg.into(),
        severity: Severity::Major,
        remedy_id: None,
    }
}
fn warn_ev(pb: &str, task: &str, ty: &str, msg: &str) -> ErrorEvent {
    ErrorEvent {
        pb_index: 0,
        playbook: pb.into(),
        task: task.into(),
        error_type: ty.into(),
        message: msg.into(),
        severity: Severity::Warning,
        remedy_id: None,
    }
}

fn architect_continues(ctl_rx: &Receiver<Ctl>, send: &impl Fn(RunnerEvent)) -> bool {
    match ctl_rx.recv() {
        Ok(Ctl::SkipContinue) | Ok(Ctl::Retry) => {
            send(RunnerEvent::Log("↻ Architect ruled: continue".into()));
            true
        }
        _ => {
            send(RunnerEvent::Aborted);
            false
        }
    }
}
fn wait_abort(ctl_rx: &Receiver<Ctl>, send: &impl Fn(RunnerEvent)) {
    let _ = ctl_rx.recv();
    send(RunnerEvent::Aborted);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_target_dir_matches_not_just_repo_root() {
        // The exact real-world case that broke live 2026-08-01: a target/
        // directory nested under a differently-named parent, which the
        // old hardcoded `git rm --cached target` pathspec never matched.
        assert!(matches_fenced_pattern(
            "_DailyWorsk/target/debug/deps/dubsar_visualizer-abc123"
        ));
        assert!(matches_fenced_pattern("target/debug/foo"));
        assert!(matches_fenced_pattern("a/b/c/target/x"));
    }

    #[test]
    fn nested_backups_dir_matches() {
        assert!(matches_fenced_pattern(
            "_Backups20260626/dubsar-theater/godot/Godot_v4.7-stable_linux.x86_64"
        ));
        assert!(matches_fenced_pattern(
            "_Backups20260626/dubsar-theater/godot/godot.zip"
        ));
    }

    #[test]
    fn node_modules_and_npm_and_log_still_match() {
        assert!(matches_fenced_pattern(
            "frontend/node_modules/react/index.js"
        ));
        assert!(matches_fenced_pattern("some/.npm/cache"));
        assert!(matches_fenced_pattern("logs/build.log"));
    }

    #[test]
    fn ordinary_source_paths_do_not_match() {
        assert!(!matches_fenced_pattern(
            "docs/19_roadmap/BAHYWAY_ECOSYSTEM_V4_ROADMAP.md"
        ));
        assert!(!matches_fenced_pattern("crates/naming-registry/src/lib.rs"));
        // "targeting.md" contains "target" as a substring but not as a
        // path COMPONENT -- must not false-positive.
        assert!(!matches_fenced_pattern("docs/targeting.md"));
    }

    #[test]
    fn paths_to_untrack_filters_a_realistic_tracked_listing() {
        let tracked = vec![
            "docs/README.md",
            "_DailyWorsk/target/debug/deps/dubsar_visualizer-0a7adf20c7d40abb",
            "_Backups20260626/dubsar-theater/godot/Godot_v4.7-stable_linux.x86_64",
            "crates/foo/src/lib.rs",
            "node_modules/left-pad/index.js",
        ];
        let result = paths_to_untrack(&tracked);
        assert_eq!(result.len(), 3);
        assert!(result.contains(
            &"_DailyWorsk/target/debug/deps/dubsar_visualizer-0a7adf20c7d40abb".to_string()
        ));
        assert!(result.contains(
            &"_Backups20260626/dubsar-theater/godot/Godot_v4.7-stable_linux.x86_64".to_string()
        ));
        assert!(result.contains(&"node_modules/left-pad/index.js".to_string()));
    }

    #[test]
    fn expands_home_tilde_paths() {
        std::env::set_var("HOME", "/home/tester");
        assert_eq!(expand_home("~/Forge/docs"), "/home/tester/Forge/docs");
        assert_eq!(expand_home("~"), "/home/tester");
        assert_eq!(expand_home("/already/absolute"), "/already/absolute");
    }

    #[test]
    fn sha256_matches_known_vector() {
        // Known SHA-256("") test vector — proves this is real SHA-256, not
        // a truncated stand-in from a different hash function.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    fn test_cfg(
        official_repo_path: &str,
        official_repo_branch: &str,
        auto_push: bool,
    ) -> DocPulseCfg {
        DocPulseCfg {
            repo_path: "/tmp/unused_source_repo".into(),
            message: "test".into(),
            limit_mb: 90,
            archive_dir: "/tmp/unused_archive".into(),
            ingest_manifest_dir: "/tmp/unused_manifest".into(),
            chronicle_dir: "/tmp/unused_chronicle".into(),
            enkiddb_output_root: "/tmp/unused_enkiddb".into(),
            official_repo_path: official_repo_path.into(),
            official_repo_subdir: "docs/bahyway-v4".into(),
            official_repo_branch: official_repo_branch.into(),
            auto_push_to_official_repo: auto_push,
            docs_tribe_id: enkiddb::DOCS_TRIBE_ID,
        }
    }

    #[test]
    fn official_repo_landing_is_a_noop_when_not_configured() {
        let cfg = test_cfg("", "docs-intake", false);
        let (tx, rx) = crossbeam_channel::unbounded();
        let (_ctl_tx, ctl_rx) = crossbeam_channel::unbounded::<Ctl>();
        let send = |ev: RunnerEvent| {
            let _ = tx.send(ev);
        };
        let log = |_: String| {};
        let proceed = stage_official_repo_landing(
            &cfg,
            "/tmp/src",
            "src_repo",
            &["a.md".to_string()],
            &send,
            &log,
            &ctl_rx,
        );
        assert!(
            proceed,
            "an unconfigured landing stage must not block Finished"
        );
        let events: Vec<RunnerEvent> = rx.try_iter().collect();
        assert!(matches!(events[0], RunnerEvent::PbStarted(5)));
        assert!(matches!(events[1], RunnerEvent::PbOk(5)));
        assert_eq!(events.len(), 2, "no other events expected");
    }

    #[test]
    fn official_repo_landing_refuses_main_as_the_target_branch() {
        // A repo path that doesn't exist is fine here -- the branch-name
        // guard must fire before any real git command runs against it.
        let cfg = test_cfg("/nonexistent/official/repo", "main", false);
        let (tx, rx) = crossbeam_channel::unbounded();
        let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded::<Ctl>();
        ctl_tx.send(Ctl::Abort).unwrap();
        let send = |ev: RunnerEvent| {
            let _ = tx.send(ev);
        };
        let log = |_: String| {};
        let proceed = stage_official_repo_landing(
            &cfg,
            "/tmp/src",
            "src_repo",
            &["a.md".to_string()],
            &send,
            &log,
            &ctl_rx,
        );
        assert!(
            !proceed,
            "must not proceed to Finished after the Architect chose Abort"
        );
        let events: Vec<RunnerEvent> = rx.try_iter().collect();
        assert!(matches!(events[0], RunnerEvent::PbStarted(5)));
        match &events[1] {
            RunnerEvent::PbFailed(5, ev) => assert_eq!(ev.error_type, "UnsafeTargetBranch"),
            other => panic!("expected PbFailed(5, UnsafeTargetBranch), got {other:?}"),
        }
        assert!(matches!(events[2], RunnerEvent::Aborted));
    }

    #[test]
    fn doc_kaki_registry_missing_file_is_an_empty_map_not_an_error() {
        let dir = std::env::temp_dir().join("shk_docpulse_registry_missing");
        let _ = std::fs::remove_dir_all(&dir);
        let map = load_doc_kaki_registry(&dir.join("never_written.jsonl"));
        assert!(map.is_empty());
    }

    #[test]
    fn doc_kaki_registry_round_trips_and_later_lines_win_on_duplicate_path() {
        let dir = std::env::temp_dir().join("shk_docpulse_registry_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("doc_kaki_registry.jsonl");

        let older = DocKakiRegistryLine {
            path: "docs/guide.md".to_string(),
            kaki_hex: hex::encode([0x11u8; 16]),
            minted_at: "2026-07-29T00:00:00Z".to_string(),
        };
        let newer = DocKakiRegistryLine {
            path: "docs/guide.md".to_string(),
            kaki_hex: hex::encode([0x22u8; 16]),
            minted_at: "2026-07-30T00:00:00Z".to_string(),
        };
        append(
            &path,
            &format!("{}\n", serde_json::to_string(&older).unwrap()),
        );
        append(
            &path,
            &format!("{}\n", serde_json::to_string(&newer).unwrap()),
        );

        let map = load_doc_kaki_registry(&path);
        assert_eq!(
            map.get("docs/guide.md"),
            Some(&[0x22u8; 16]),
            "the later line must win over the earlier one"
        );
    }
}
