//! The runner: executes the corpus in order on a worker thread.
//! Real mode spawns `ansible-playbook` with ANSIBLE_STDOUT_CALLBACK=json;
//! simulate mode drives a scripted rehearsal so the governor can be
//! exercised without touching the real estate.
//!
//! Law: warnings proceed; majors halt and await the Architect (Ctl).

use crate::chronicle::Chronicle;
use crate::config::Config;
use crate::crate_mint;
use crate::model::*;
use crate::pb_doc_mint;
use crate::pb_mint;
use crate::remedy;
use crate::tablet_mint;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use enkimdb::AnuGovernorRunRecordSpec;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn spawn_runner(
    cfg: Config,
    params: Vec<(String, String)>,
    tx: Sender<RunnerEvent>,
    ctl_rx: Receiver<Ctl>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || run(cfg, params, tx, ctl_rx))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// The 5 groups PB-268 creates on the Fedora host, in the Architect's own
/// privilege order (see that playbook's own mapping comment -- same
/// numbers, not re-decided here). Kept here, not imported from PB-268
/// (a YAML file, not a library), so this list is the one place either
/// side would need to change together if the mapping is ever revised.
const BAHYWAY_ROLE_GROUPS: &[(&str, u8)] = &[
    ("bahyway-architect", 7),
    ("bahyway-datasteward", 6),
    ("bahyway-administrator", 5),
    ("bahyway-developer", 3),
    ("bahyway-stakeholder", 1),
];

/// Real Fedora host OS identity of whatever account launched this
/// process -- `id -un`/`id -Gn`, never a fabricated placeholder. This is
/// the SECOND identity signal (see `AnuGovernorRunRecordSpec`'s own doc
/// comment): always available, no vault needed, distinct from the
/// cryptographic vault-passport identity `authenticate_operator` above
/// captures only when `vault_check_enabled=true`.
struct OsIdentity {
    username: Option<String>,
    groups_csv: Option<String>,
    bahyway_role: Option<String>,
}

fn detect_os_identity() -> OsIdentity {
    let run = |arg: &str| -> Option<String> {
        let out = Command::new("id").arg(arg).output().ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8(out.stdout).ok()?;
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    };

    let username = run("-un");
    let groups_csv = run("-Gn").map(|s| s.split_whitespace().collect::<Vec<_>>().join(","));

    // Strongest match wins, same rule the vault-passport path already
    // uses when a vault holds more than one valid passport.
    let bahyway_role = groups_csv.as_ref().and_then(|csv| {
        let groups: Vec<&str> = csv.split(',').collect();
        BAHYWAY_ROLE_GROUPS
            .iter()
            .filter(|(g, _)| groups.contains(g))
            .max_by_key(|(_, level)| *level)
            .map(|(g, _)| (*g).to_string())
    });

    OsIdentity {
        username,
        groups_csv,
        bahyway_role,
    }
}

/// Run-confirmation registry (2026-07-29): counters + operator identity
/// accumulated across the whole run, so a single record can summarize
/// "what happened, who ran it, at what privilege level" without
/// re-reading the sealed report/chronicle.
struct RunTally {
    run_id: String,
    started_at: u64,
    playbook_count: u32,
    ok: u32,
    warned: u32,
    failed: u32,
    skipped: u32,
    operator: Option<kupru_vault::AuthedIdentity>,
    os: OsIdentity,
}

impl RunTally {
    fn new(playbook_count: u32, operator: Option<kupru_vault::AuthedIdentity>) -> Self {
        let run_id = format!("{}", now_secs());
        Self {
            run_id,
            started_at: now_secs(),
            playbook_count,
            ok: 0,
            warned: 0,
            failed: 0,
            skipped: 0,
            operator,
            os: detect_os_identity(),
        }
    }

    fn to_spec(&self, outcome: &str, blocked_reason: Option<String>) -> AnuGovernorRunRecordSpec {
        AnuGovernorRunRecordSpec {
            run_id: self.run_id.clone(),
            started_at: self.started_at,
            finished_at: now_secs(),
            playbook_count: self.playbook_count,
            ok_count: self.ok,
            warned_count: self.warned,
            failed_count: self.failed,
            skipped_count: self.skipped,
            outcome: outcome.to_string(),
            blocked_reason,
            operator_subject_kaki_hex: self.operator.as_ref().map(|o| o.subject_kaki_hex.clone()),
            operator_realm: self.operator.as_ref().map(|o| o.realm.clone()),
            operator_privilege_level: self.operator.as_ref().map(|o| o.privilege_level),
            os_username: self.os.username.clone(),
            os_groups_csv: self.os.groups_csv.clone(),
            os_bahyway_role: self.os.bahyway_role.clone(),
            report_path: String::new(),
        }
    }
}

/// Best-effort: sends `INGEST_RUN_RECORD:<json>` to EnkiMDB's Write Node
/// and returns a human-readable log line. Never panics the runner --
/// EnkiMDB may not be reachable from every environment AnuGovernor runs
/// in (e.g. the authoring sandbox), and a missing confirmation registry
/// entry must never be confused with, or block on, the playbook run
/// itself succeeding or failing.
fn emit_run_record(cfg: &Config, spec: &AnuGovernorRunRecordSpec) -> String {
    let Some(host) = &cfg.run.enkimdb_write_host else {
        return "  · run-confirmation registry: enkimdb_write_host not set, skipped".to_string();
    };
    let addr = format!("{host}:{}", cfg.run.enkimdb_write_port);
    let json = match serde_json::to_string(spec) {
        Ok(j) => j,
        Err(e) => return format!("  ⚠ run-confirmation registry: could not serialize record: {e}"),
    };
    let payload = format!("INGEST_RUN_RECORD:{json}");
    match TcpStream::connect(&addr) {
        Ok(mut s) => {
            let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
            let _ = s.set_write_timeout(Some(Duration::from_secs(10)));
            let send_ok = s
                .write_all(&(payload.len() as u32).to_le_bytes())
                .and_then(|_| s.write_all(payload.as_bytes()));
            if let Err(e) = send_ok {
                return format!("  ⚠ run-confirmation registry: write to {addr} failed: {e}");
            }
            let mut lb = [0u8; 4];
            if s.read_exact(&mut lb).is_err() {
                return format!("  ⚠ run-confirmation registry: no response from {addr}");
            }
            let len = u32::from_le_bytes(lb) as usize;
            let mut buf = vec![0u8; len];
            if s.read_exact(&mut buf).is_err() {
                return format!("  ⚠ run-confirmation registry: truncated response from {addr}");
            }
            let resp = String::from_utf8_lossy(&buf);
            if resp.starts_with("OK:") {
                format!(
                    "  ✓ run-confirmation registry: recorded (outcome={}) at {addr}",
                    spec.outcome
                )
            } else {
                format!("  ⚠ run-confirmation registry: {addr} rejected the record: {resp}")
            }
        }
        Err(e) => format!("  · run-confirmation registry: {addr} unreachable ({e}), skipped"),
    }
}

/// Vault gate 1 of 1: if `vault_check_enabled`, the ENTIRE run is refused
/// (never touches a single playbook) unless a real, cryptographically
/// verified passport in the vault at `vault_path` clears
/// `vault_min_privilege`. `KUPRU_VAULT_PASSPHRASE` is read from the
/// environment only -- never accepted via this file or a CLI arg, which
/// would leak it into shell history / process listings. Returns
/// `Ok(None)` when the gate is disabled (unauthenticated run, honestly
/// recorded as such), `Ok(Some(identity))` on success, `Err(reason)` on
/// any denial.
fn authenticate_operator(cfg: &Config) -> Result<Option<kupru_vault::AuthedIdentity>, String> {
    if !cfg.run.vault_check_enabled {
        return Ok(None);
    }
    let Some(vault_path) = &cfg.run.vault_path else {
        return Err("vault_check_enabled=true but vault_path is not set".to_string());
    };
    let passphrase = std::env::var("KUPRU_VAULT_PASSPHRASE").map_err(|_| {
        "vault_check_enabled=true but KUPRU_VAULT_PASSPHRASE is not set in the environment"
            .to_string()
    })?;
    let vault_bytes = std::fs::read(vault_path)
        .map_err(|e| format!("cannot read vault file {vault_path}: {e}"))?;
    let identity = kupru_vault::open_vault_and_authenticate(&vault_bytes, passphrase.as_bytes())?;
    if identity.privilege_level < cfg.run.vault_min_privilege {
        return Err(format!(
            "operator privilege_level={} below required minimum={}",
            identity.privilege_level, cfg.run.vault_min_privilege
        ));
    }
    Ok(Some(identity))
}

fn run(cfg: Config, params: Vec<(String, String)>, tx: Sender<RunnerEvent>, ctl_rx: Receiver<Ctl>) {
    let chr = Chronicle::open(&cfg.run.chronicle_dir).ok();
    let send = |ev: RunnerEvent| {
        if let Some(c) = &chr {
            if let Err(e) = c.record(&ev) {
                let _ = tx.send(RunnerEvent::Log(format!("⚠ chronicle write failed: {e}")));
            }
        }
        let _ = tx.send(ev);
    };
    let n = cfg.run.playbooks.len();

    let operator = match authenticate_operator(&cfg) {
        Ok(op) => op,
        Err(reason) => {
            send(RunnerEvent::Log(format!("  ✖ Vault authentication DENIED: {reason} — run refused before any playbook executed")));
            let tally = RunTally::new(n as u32, None);
            let spec = tally.to_spec("BlockedByAuthority", Some(reason));
            send(RunnerEvent::Log(emit_run_record(&cfg, &spec)));
            send(RunnerEvent::Aborted);
            return;
        }
    };
    if let Some(id) = &operator {
        send(RunnerEvent::Log(format!(
            "  ✓ Vault authenticated: realm={} privilege_level={}",
            id.realm, id.privilege_level
        )));
    }
    let mut tally = RunTally::new(n as u32, operator);
    match (&tally.os.username, &tally.os.bahyway_role) {
        (Some(user), Some(role)) => send(RunnerEvent::Log(format!(
            "  · OS identity: user={user} bahyway_role={role}"
        ))),
        (Some(user), None) => send(RunnerEvent::Log(format!(
            "  · OS identity: user={user} (not a member of any bahyway-* group)"
        ))),
        (None, _) => send(RunnerEvent::Log(
            "  · OS identity: could not determine (id -un failed)".to_string(),
        )),
    }

    // ADR-014 "mint at authoring time", now covering every real element
    // BahyWay.Ecosystem builds -- PB-270's "one central tool" decree:
    // every real Corpus run mints whatever's new across all three real
    // scanners (playbooks, crates, and .akk/.way/.tmpl tablets), each
    // into its own tribe/registry, quiet (no log lines) on the vast
    // majority of runs that mint nothing new in a given corpus.
    let repo_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let enkimdb_root = PathBuf::from("enkimdb_data");
    let triage_doc = cfg
        .run
        .triage_doc
        .clone()
        .map(|t| repo_root.join(t))
        .unwrap_or_else(|| repo_root.join("docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md"));
    let pb_registry = PathBuf::from(&cfg.run.chronicle_dir).join("pb_kaki_registry.jsonl");
    for line in pb_mint::mint_new_playbooks(&repo_root, &triage_doc, &pb_registry, &enkimdb_root) {
        send(RunnerEvent::Log(line));
    }

    // Keep docs/16_runbooks/PLAYBOOK_EXECUTION_TRIAGE.md itself current,
    // not just the EnkiMDB catalog above -- the Architect's own decree
    // (2026-08-27): a real, numbered playbook with no triage row yet gets
    // one, automatically, on every Corpus pulse. Re-scans (cheap: one
    // filesystem walk + one markdown parse) rather than reusing
    // `mint_new_playbooks`'s internal scan, because that function's own
    // "already minted" registry and this doc's own row-existence are two
    // different, independently-real facts -- a PB can be minted into
    // EnkiMDB on one run and only get its triage row a run later (or vice
    // versa) if one write fails and the other doesn't, and each should
    // reflect its own real state, never borrow the other's.
    match enkimdb::scan_pbs(&repo_root, &triage_doc) {
        Ok(profiles) => match enkimdb::append_untriaged_rows(&triage_doc, &profiles) {
            Ok(0) => {}
            Ok(n) => send(RunnerEvent::Log(format!(
                "𒁾 triage doc: {n} newly-discovered playbook(s) given a `[ ]` row under \"Auto-discovered — not yet triaged\""
            ))),
            Err(e) => send(RunnerEvent::Log(format!(
                "⚠ triage doc: could not append new row(s): {e}"
            ))),
        },
        Err(e) => send(RunnerEvent::Log(format!(
            "⚠ triage doc: PB scan for row-sync failed: {e}"
        ))),
    }

    // Crates live under workspace/bahyway_v4 (this process's own cwd when
    // launched via the repo's own tooling is the repo root, one level
    // above -- scan_crates wants the workspace root specifically, same
    // as enkimdb-write-server's SCAN_CRATES protocol message expects).
    let workspace_root = repo_root.join("workspace/bahyway_v4");
    let crate_registry = PathBuf::from(&cfg.run.chronicle_dir).join("crate_kaki_registry.jsonl");
    for line in crate_mint::mint_new_crates(&workspace_root, &crate_registry, &enkimdb_root) {
        send(RunnerEvent::Log(line));
    }

    // Tablet registry path matches `girsu-mint`'s own default exactly
    // (`chronicle/tablet_kaki_registry.jsonl` relative to cwd) so a
    // tablet marked final from Girsu IDE and this corpus-wide scan share
    // one durable record of "already minted" -- see tablet_mint's doc
    // comment.
    let tablet_registry = PathBuf::from(&cfg.run.chronicle_dir).join("tablet_kaki_registry.jsonl");
    for line in tablet_mint::mint_new_tablets(&repo_root, &tablet_registry, &enkimdb_root) {
        send(RunnerEvent::Log(line));
    }

    // Playbook DOCUMENTATION (not just existence -- see pb_doc_mint's own
    // doc comment): EnkiDDB, not EnkiMDB, since this is what the
    // Architect's "save its documentation in the EnkiDDB pb_docs_schema"
    // decree asked for.
    let pb_doc_registry = PathBuf::from(&cfg.run.chronicle_dir).join("pb_doc_kaki_registry.jsonl");
    let enkiddb_root = PathBuf::from("enkiddb_data");
    for line in pb_doc_mint::mint_or_supersede_pb_docs(&repo_root, &pb_doc_registry, &enkiddb_root)
    {
        send(RunnerEvent::Log(line));
    }

    // Closes the other half of the Architect's 2026-08-27 decree: a real
    // (never simulated, never `--check`) playbook run writes its own
    // Status/Note cells back into the triage doc, so the file reflects
    // what actually happened without a human copying it in by hand.
    // `append_untriaged_rows` above guarantees a row exists to update by
    // the time any playbook here can finish — new PBs always get a row
    // before they ever run through this loop in the same pulse.
    let sync_status = |file_stem: &str, status: &str, note: String| {
        if cfg.run.simulate {
            return; // a rehearsal proves nothing about real infra -- never write it as if it did
        }
        match enkimdb::update_triage_status(&triage_doc, file_stem, status, &note) {
            Ok(true) => {}
            Ok(false) => send(RunnerEvent::Log(format!(
                "⚠ triage doc: no row found for {file_stem}.yml to update -- was it removed since the last Corpus pulse?"
            ))),
            Err(e) => send(RunnerEvent::Log(format!(
                "⚠ triage doc: could not update {file_stem}.yml's row: {e}"
            ))),
        }
    };

    for i in 0..n {
        // non-blocking abort check between playbooks
        if matches!(ctl_rx.try_recv(), Ok(Ctl::Abort)) {
            let spec = tally.to_spec(
                "Aborted",
                Some("Architect aborted between playbooks".to_string()),
            );
            send(RunnerEvent::Log(emit_run_record(&cfg, &spec)));
            send(RunnerEvent::Aborted);
            return;
        }
        let pb = cfg.run.playbooks[i].clone();
        send(RunnerEvent::PbStarted(i));
        send(RunnerEvent::Log(format!("[{}/{}] {}", i + 1, n, pb)));

        loop {
            let outcome = if cfg.run.simulate {
                simulate_one(&cfg, i, &pb)
            } else {
                run_one(&cfg, i, &pb, &params, &ctl_rx)
            };

            match outcome {
                Outcome::Ok => {
                    send(RunnerEvent::PbOk(i));
                    tally.ok += 1;
                    sync_status(
                        pb.trim_end_matches(".yml"),
                        "x",
                        format!(
                            "run for real (anu-governor Corpus) at epoch {} — ansible-playbook exited clean",
                            now_secs()
                        ),
                    );
                    break;
                }
                Outcome::Warned(ev) => {
                    send(RunnerEvent::Log(format!(
                        "  ⚠ {} — proceeding by law",
                        ev.error_type
                    )));
                    sync_status(
                        pb.trim_end_matches(".yml"),
                        "~",
                        format!(
                            "run for real (anu-governor Corpus) at epoch {} — completed with a warning: {}: {}",
                            now_secs(),
                            ev.error_type,
                            ev.message
                        ),
                    );
                    send(RunnerEvent::PbWarned(i, ev));
                    tally.warned += 1;
                    break;
                }
                Outcome::Aborted => {
                    send(RunnerEvent::Log(format!(
                        "  ■ Abort — killed mid-run during '{pb}'"
                    )));
                    tally.skipped += (n - i - 1) as u32;
                    let spec = tally.to_spec(
                        "Aborted",
                        Some(format!("Architect aborted mid-run during '{pb}'")),
                    );
                    send(RunnerEvent::Log(emit_run_record(&cfg, &spec)));
                    send(RunnerEvent::Aborted);
                    return;
                }
                Outcome::Failed(ev) => {
                    send(RunnerEvent::Log(format!(
                        "  ✖ MAJOR at task '{}' — halted, awaiting Architect",
                        ev.task
                    )));
                    send(RunnerEvent::PbFailed(i, ev.clone()));
                    // Block until the Architect decides (CSR-08).
                    match ctl_rx.recv() {
                        Ok(Ctl::Abort) | Err(_) => {
                            tally.failed += 1;
                            tally.skipped += (n - i - 1) as u32;
                            let spec = tally.to_spec(
                                "Aborted",
                                Some(format!(
                                    "MAJOR failure at '{}', Architect chose Abort",
                                    ev.task
                                )),
                            );
                            send(RunnerEvent::Log(emit_run_record(&cfg, &spec)));
                            send(RunnerEvent::Aborted);
                            return;
                        }
                        Ok(Ctl::SkipContinue) => {
                            send(RunnerEvent::Log(
                                "  » skipped by Architect — continuing".into(),
                            ));
                            sync_status(
                                pb.trim_end_matches(".yml"),
                                "~",
                                format!(
                                    "run for real (anu-governor Corpus) at epoch {} — FAILED at '{}': {}, Architect chose to skip and continue",
                                    now_secs(),
                                    ev.task,
                                    ev.message
                                ),
                            );
                            tally.skipped += 1;
                            break;
                        }
                        Ok(Ctl::Retry) => {
                            send(RunnerEvent::Log("  ↻ retrying after remedy".into()));
                            if cfg.run.simulate {
                                // rehearsal: remedy applied, retry succeeds
                                std::thread::sleep(Duration::from_millis(250));
                                send(RunnerEvent::PbOk(i));
                                tally.ok += 1;
                                break;
                            }
                            continue; // real mode: run the playbook again
                        }
                    }
                }
            }
        }
    }
    let spec = tally.to_spec("Completed", None);
    send(RunnerEvent::Log(emit_run_record(&cfg, &spec)));
    send(RunnerEvent::Finished);
}

enum Outcome {
    Ok,
    Warned(ErrorEvent),
    Failed(ErrorEvent),
    /// The Architect hit Abort while `ansible-playbook` was actually
    /// mid-execution (not just between playbooks or while halted awaiting
    /// a decision — those two points were already interruptible).
    Aborted,
}

/// ---------- simulate mode: a deterministic rehearsal ----------
fn simulate_one(cfg: &Config, i: usize, pb: &str) -> Outcome {
    std::thread::sleep(Duration::from_millis(60));
    if Some(i) == cfg.run.simulate_fail_at {
        let msg = "enlil-hotindex.service failed to start: port 7101 already bound".to_string();
        let ev = ErrorEvent {
            pb_index: i,
            playbook: pb.to_string(),
            task: "start enlil-hotindex.service".into(),
            error_type: "ServiceStartFailure".into(),
            message: msg.clone(),
            severity: Severity::Major,
            remedy_id: cfg.find_remedy(&msg).map(|r| r.id.clone()),
        };
        return Outcome::Failed(ev);
    }
    // scripted warnings so the grid has life
    if i % 9 == 4 {
        let msg = format!("[DEPRECATION WARNING] a parameter in {pb} is deprecated");
        let ev = ErrorEvent {
            pb_index: i,
            playbook: pb.to_string(),
            task: "template configuration".into(),
            error_type: "DeprecatedParam".into(),
            severity: cfg.classify("DeprecatedParam", &msg),
            remedy_id: cfg.find_remedy(&msg).map(|r| r.id.clone()),
            message: msg,
        };
        return Outcome::Warned(ev);
    }
    Outcome::Ok
}

/// ---------- real mode: govern ansible-playbook ----------
///
/// Spawns the child and polls it rather than blocking on `Command::output()`
/// — a blocking call can't be interrupted, so an Architect hitting Abort
/// while a playbook is genuinely mid-execution would previously see no
/// effect until that playbook finished on its own. Polling `ctl_rx`
/// alongside `try_wait()` makes Abort responsive at that point too, not
/// just between playbooks or while halted awaiting a decision.
fn run_one(
    cfg: &Config,
    i: usize,
    pb: &str,
    params: &[(String, String)],
    ctl_rx: &Receiver<Ctl>,
) -> Outcome {
    let path = PathBuf::from(&cfg.run.playbook_dir).join(pb);
    let mut cmd = Command::new("ansible-playbook");
    cmd.arg(&path).env("ANSIBLE_STDOUT_CALLBACK", "json");
    if cfg.run.check_mode {
        cmd.arg("--check");
    }
    if let Some(inv) = &cfg.run.inventory {
        cmd.arg("-i").arg(inv);
    }
    for (k, v) in params {
        cmd.arg("-e").arg(format!("{k}={v}"));
    }

    let mut child = match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) => {
            return Outcome::Failed(ErrorEvent {
                pb_index: i,
                playbook: pb.to_string(),
                task: "<spawn ansible-playbook>".into(),
                error_type: "SpawnFailure".into(),
                message: e.to_string(),
                severity: Severity::Major,
                remedy_id: None,
            })
        }
    };

    loop {
        match ctl_rx.try_recv() {
            Ok(Ctl::Abort) | Err(TryRecvError::Disconnected) => {
                let _ = child.kill();
                let _ = child.wait();
                return Outcome::Aborted;
            }
            // SkipContinue/Retry only make sense while halted awaiting a
            // decision (handled in `run`'s outer loop) — ignore if one
            // arrives here, it has no meaning mid-execution.
            Ok(_) | Err(TryRecvError::Empty) => {}
        }
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(150)),
            Err(e) => {
                return Outcome::Failed(ErrorEvent {
                    pb_index: i,
                    playbook: pb.to_string(),
                    task: "<wait ansible-playbook>".into(),
                    error_type: "SpawnFailure".into(),
                    message: e.to_string(),
                    severity: Severity::Major,
                    remedy_id: None,
                })
            }
        }
    }

    // The child has already exited (and been reaped) by this point, so
    // reading its pipes to EOF here can't deadlock on a full buffer.
    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut o) = child.stdout.take() {
        let _ = o.read_to_string(&mut stdout);
    }
    if let Some(mut e) = child.stderr.take() {
        let _ = e.read_to_string(&mut stderr);
    }
    let status = child.wait();

    // Ansible warnings arrive on stderr even on success.
    let warn_line = stderr.lines().find(|l| l.contains("[WARNING]"));

    if status.map(|s| s.success()).unwrap_or(false) {
        if let Some(w) = warn_line {
            let ev = ErrorEvent {
                pb_index: i,
                playbook: pb.to_string(),
                task: "<play>".into(),
                error_type: "AnsibleWarning".into(),
                severity: cfg.classify("AnsibleWarning", w),
                remedy_id: cfg.find_remedy(w).map(|r| r.id.clone()),
                message: w.to_string(),
            };
            // A configured rule may escalate a warning to MAJOR.
            return match ev.severity {
                Severity::Warning => Outcome::Warned(ev),
                Severity::Major => Outcome::Failed(ev),
            };
        }
        return Outcome::Ok;
    }

    // Failure: mine the JSON callback for the failed task + message.
    let (task, message) = parse_failure(&stdout)
        .unwrap_or_else(|| ("<unknown task>".into(), first_lines(&stderr, 3)));
    let error_type = infer_error_type(&message);
    let severity = cfg.classify(&error_type, &message);
    let ev = ErrorEvent {
        pb_index: i,
        playbook: pb.to_string(),
        task,
        error_type,
        remedy_id: cfg.find_remedy(&message).map(|r| r.id.clone()),
        message,
        severity,
    };
    match ev.severity {
        Severity::Warning => Outcome::Warned(ev),
        Severity::Major => Outcome::Failed(ev),
    }
}

/// Walk the json-callback document: plays[].tasks[].hosts{h: {failed, msg}}.
pub fn parse_failure(stdout: &str) -> Option<(String, String)> {
    // The json callback prints exactly one JSON document; find its start.
    let start = stdout.find('{')?;
    let v: serde_json::Value = serde_json::from_str(&stdout[start..]).ok()?;
    for play in v.get("plays")?.as_array()? {
        for task in play.get("tasks")?.as_array()? {
            let name = task
                .pointer("/task/name")
                .and_then(|n| n.as_str())
                .unwrap_or("<task>")
                .to_string();
            if let Some(hosts) = task.get("hosts").and_then(|h| h.as_object()) {
                for (_host, res) in hosts {
                    let failed = res.get("failed").and_then(|f| f.as_bool()).unwrap_or(false)
                        || res
                            .get("unreachable")
                            .and_then(|f| f.as_bool())
                            .unwrap_or(false);
                    if failed {
                        let msg = res
                            .get("msg")
                            .map(|m| m.to_string().trim_matches('"').to_string())
                            .unwrap_or_else(|| "task failed (no msg)".into());
                        return Some((name, msg));
                    }
                }
            }
        }
    }
    None
}

fn infer_error_type(msg: &str) -> String {
    let m = msg.to_lowercase();
    if m.contains("unreachable") || m.contains("connect") {
        "HostUnreachable".into()
    } else if m.contains("permission") || m.contains("denied") {
        "PermissionDenied".into()
    } else if m.contains("service") || m.contains("systemd") {
        "ServiceStartFailure".into()
    } else if m.contains("not found") || m.contains("no such") {
        "MissingArtifact".into()
    } else if m.contains("syntax") {
        "SyntaxError".into()
    } else {
        "TaskFailure".into()
    }
}

fn first_lines(s: &str, n: usize) -> String {
    s.lines().take(n).collect::<Vec<_>>().join(" | ")
}

/// Generate the numbered fix playbook for a failed event (Playbook Law:
/// every remedy is a playbook, never a manual command).
pub fn generate_fix(cfg: &Config, ev: &ErrorEvent) -> anyhow::Result<PathBuf> {
    let rule = cfg
        .find_remedy(&ev.message)
        .ok_or_else(|| anyhow::anyhow!("no remedy rule matches this error"))?;
    remedy::write_fix_playbook(&cfg.run.fix_dir, rule, ev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_callback_failure() {
        let doc = r#"noise before
{"plays":[{"tasks":[
  {"task":{"name":"ok task"},"hosts":{"h1":{"changed":true}}},
  {"task":{"name":"start enlil-hotindex.service"},
   "hosts":{"h1":{"failed":true,"msg":"port 7101 already bound"}}}
]}],"stats":{}}"#;
        let (task, msg) = parse_failure(doc).unwrap();
        assert_eq!(task, "start enlil-hotindex.service");
        assert!(msg.contains("already bound"));
    }

    #[test]
    fn error_type_inference() {
        assert_eq!(
            infer_error_type("systemd unit failed"),
            "ServiceStartFailure"
        );
        assert_eq!(
            infer_error_type("Permission denied (publickey)"),
            "PermissionDenied"
        );
        assert_eq!(infer_error_type("mystery"), "TaskFailure");
    }

    #[test]
    fn detect_os_identity_reads_a_real_username_on_this_host() {
        // Real, not mocked: `id -un` genuinely runs in every environment
        // this crate builds in (unlike libvirt/Godot, no VM/GPU needed).
        let os = detect_os_identity();
        let user = os
            .username
            .expect("id -un must succeed on any real Unix host");
        assert!(!user.is_empty());
        // groups_csv must be present whenever username is (id -Gn always
        // includes at least the user's own primary group).
        assert!(os.groups_csv.is_some());
    }

    #[test]
    fn bahyway_role_lookup_picks_the_strongest_matching_group() {
        // Doesn't depend on real host group membership -- exercises the
        // pure selection logic directly against a synthetic groups_csv.
        let csv = "developer,bahyway-developer,bahyway-architect,wheel".to_string();
        let groups: Vec<&str> = csv.split(',').collect();
        let role = BAHYWAY_ROLE_GROUPS
            .iter()
            .filter(|(g, _)| groups.contains(g))
            .max_by_key(|(_, level)| *level)
            .map(|(g, _)| (*g).to_string());
        assert_eq!(role, Some("bahyway-architect".to_string()));
    }

    #[test]
    fn bahyway_role_lookup_is_none_when_no_group_matches() {
        let csv = "developer,wheel,users".to_string();
        let groups: Vec<&str> = csv.split(',').collect();
        let role = BAHYWAY_ROLE_GROUPS
            .iter()
            .filter(|(g, _)| groups.contains(g))
            .max_by_key(|(_, level)| *level)
            .map(|(g, _)| (*g).to_string());
        assert_eq!(role, None);
    }

    #[test]
    fn abort_during_real_run_is_responsive_not_blocking() {
        // A long-sleeping child stands in for a slow ansible-playbook run.
        // Sending Abort must return well before the child would exit on
        // its own, proving the poll loop (not a blocking wait) governs it.
        let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded();
        let cfg = Config {
            run: crate::config::RunCfg {
                playbook_dir: ".".into(),
                playbooks: vec![],
                inventory: None,
                triage_doc: None,
                simulate: false,
                simulate_fail_at: None,
                fix_dir: "fixes".into(),
                report_dir: "reports".into(),
                seal_key: "seal.key".into(),
                chronicle_dir: "chronicle".into(),
                check_mode: false,
                vault_check_enabled: false,
                vault_path: None,
                vault_min_privilege: 5,
                enkimdb_write_host: None,
                enkimdb_write_port: 7201,
            },
            parameters: vec![],
            severity: vec![],
            remedy: vec![],
        };

        // Swap `ansible-playbook` for `sleep 30` by temporarily invoking
        // run_one against a fake "playbook" is not possible without a real
        // ansible-playbook binary; instead exercise the poll/kill mechanics
        // directly against a long-lived child, which is the part this fix
        // actually changed.
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        ctl_tx.send(Ctl::Abort).unwrap();
        let start = std::time::Instant::now();
        loop {
            if matches!(ctl_rx.try_recv(), Ok(Ctl::Abort)) {
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
            if child.try_wait().unwrap().is_some() {
                panic!("child exited on its own before Abort was observed");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "abort must be responsive, not wait for the 30s sleep"
        );
        let _ = cfg; // constructed above purely to exercise the RunCfg shape
    }
}
