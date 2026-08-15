//! Shakkanakku's browser-facing governor. Same engine as the native egui
//! face (`shakkanakku::{config,runner,remedy,report,chronicle}`, unchanged)
//! behind a small hand-rolled HTTP server, so the Architect drives it from
//! any browser instead of a native window — the browser owns resize and
//! layout reflow, which a decoration-less, GPU-less VDI compositor could
//! not be made to do reliably (see git history on this file's sibling
//! `app.rs` for that saga).
//!
//! This is not a toy plaintext localhost server: every connection is real
//! TLS (`shakkanakku::web_tls`), every mutating action requires a real
//! kupru passport session (`shakkanakku::web_auth` — Ed25519-signed,
//! HMAC-cookied, privilege-gated: viewing needs any valid passport,
//! governing the run needs an Architect-privilege one), and login attempts
//! are rate-limited per source IP. One process, one Architect, one
//! Mutex<State>: this is not a public service, so hand-rolled HTTP/1.1
//! (GET/POST, Connection: close, no keep-alive) is deliberately the right
//! amount of machinery for the *protocol* — the *security* underneath it
//! is not hand-waved.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use bahyway_core::hepta_gate::HeptaGate;
use serde_json::{json, Value};
use shakkanakku::docpulse::{self, DocPulseCfg};
use shakkanakku::domain_review;
use shakkanakku::gate_review;
use shakkanakku::model::*;
use shakkanakku::pb_dependency_review;
use shakkanakku::resource_check::{self, ResourceCheckResult, Thresholds};
use shakkanakku::web_auth::{self, AuthedIdentity, LoginDefender, SessionStore};
use shakkanakku::{config::Config, report, runner, web_tls};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const INDEX_HTML: &str = include_str!("web_assets/index.html");
const APP_CSS: &str = include_str!("web_assets/app.css");
const APP_JS: &str = include_str!("web_assets/app.js");
const LOGIN_HTML: &str = include_str!("web_assets/login.html");
const LOGIN_JS: &str = include_str!("web_assets/login.js");
// Shala4 · Gate Orbits: its own ES module (see that file's own header
// comment for why it's not folded into APP_JS), plus Three.js r160 +
// OrbitControls vendored locally rather than fetched from a CDN at
// runtime -- this server never depends on outside network reachability
// for anything else it serves, and a governor tool shouldn't start here.
const GATE_ORBITS_JS: &str = include_str!("web_assets/gate_orbits.js");
const VENDOR_THREE_JS: &str = include_str!("web_assets/vendor/three.module.js");
const VENDOR_ORBIT_CONTROLS_JS: &str = include_str!("web_assets/vendor/OrbitControls.js");

const SESSION_COOKIE: &str = "shk_session";
const SESSION_MAX_AGE_SECS: u64 = 8 * 3600;

struct ServerState {
    cfg_path: String,
    cfg: Option<Config>,
    params: Vec<(String, String)>,
    phase: RunPhase,
    statuses: Vec<PbStatus>,
    events: Vec<ErrorEvent>,
    halt_event: Option<usize>,
    log: Vec<String>,
    fixes: Vec<PathBuf>,
    started: Option<chrono::DateTime<chrono::Utc>>,
    finished: Option<chrono::DateTime<chrono::Utc>>,
    report_path: Option<PathBuf>,
    status_line: String,
    rx: Option<crossbeam_channel::Receiver<RunnerEvent>>,
    ctl_tx: Option<crossbeam_channel::Sender<Ctl>>,

    // ── Tab 1: Resource Check (2026-08-01, 3-tab dashboard) ──
    // The last check's result and when it ran -- Tabs 2/3's start
    // actions are gated on this being recent AND green, server-side,
    // not just a client-side disabled button.
    last_resource_check: Option<ResourceCheckResult>,
    last_resource_check_at: Option<chrono::DateTime<chrono::Utc>>,

    // ── Tab 3: Hala (2026-08-01; sovereign name as of Phase 2 reconciliation, 2026-08-15) ──
    // Mirrors app.rs's own dp_* fields (the native GUI's Hala tab)
    // so this is a third driver of the exact same docpulse engine, not a
    // reimplementation -- see docpulse::spawn_docpulse.
    dp_phase: RunPhase,
    dp_stage_statuses: Vec<PbStatus>,
    dp_events: Vec<ErrorEvent>,
    dp_halt_event: Option<usize>,
    dp_log: Vec<String>,
    dp_started: Option<chrono::DateTime<chrono::Utc>>,
    dp_finished: Option<chrono::DateTime<chrono::Utc>>,
    dp_rx: Option<crossbeam_channel::Receiver<RunnerEvent>>,
    dp_ctl_tx: Option<crossbeam_channel::Sender<Ctl>>,
}

/// How stale a resource check may be before Tabs 2/3's start actions
/// refuse it and require a fresh one -- a green check from an hour ago
/// says nothing about the host's state right now.
const RESOURCE_CHECK_MAX_AGE_SECS: i64 = 300;

impl ServerState {
    fn new() -> Self {
        let mut s = Self {
            cfg_path: "shakkanakku.toml".into(),
            cfg: None,
            params: vec![],
            phase: RunPhase::Idle,
            statuses: vec![],
            events: vec![],
            halt_event: None,
            log: vec![],
            fixes: vec![],
            started: None,
            finished: None,
            report_path: None,
            status_line: "load a config to begin".into(),
            rx: None,
            ctl_tx: None,
            last_resource_check: None,
            last_resource_check_at: None,
            dp_phase: RunPhase::Idle,
            dp_stage_statuses: vec![PbStatus::Pending; docpulse::STAGE_NAMES.len()],
            dp_events: vec![],
            dp_halt_event: None,
            dp_log: vec![],
            dp_started: None,
            dp_finished: None,
            dp_rx: None,
            dp_ctl_tx: None,
        };
        s.load_config();
        s
    }

    /// Tab 1: run the native resource check now, store the result +
    /// timestamp (this is what Tabs 2/3's start actions gate on).
    fn run_resource_check(&mut self) -> &ResourceCheckResult {
        let result = resource_check::check(&Thresholds::default());
        self.last_resource_check = Some(result);
        self.last_resource_check_at = Some(chrono::Utc::now());
        self.last_resource_check.as_ref().unwrap()
    }

    /// The real gate: recent AND green. Used before Tab 2's `/api/run`
    /// and Tab 3's `/api/docpulse/start` both accept a start request --
    /// enforced here, not just reflected in a disabled button client-side.
    fn resource_check_permits_start(&self) -> Result<(), String> {
        let Some(result) = &self.last_resource_check else {
            return Err("no resource check has been run yet -- run Tab 1's check first".into());
        };
        let Some(at) = self.last_resource_check_at else {
            return Err("no resource check has been run yet -- run Tab 1's check first".into());
        };
        let age_secs = (chrono::Utc::now() - at).num_seconds();
        if age_secs > RESOURCE_CHECK_MAX_AGE_SECS {
            return Err(format!(
                "last resource check is {age_secs}s old (max {RESOURCE_CHECK_MAX_AGE_SECS}s) -- run it again"
            ));
        }
        if !result.green {
            return Err(format!("last resource check is RED: {}", result.warnings.join("; ")));
        }
        Ok(())
    }

    fn resource_check_json(&self) -> Value {
        match (&self.last_resource_check, self.last_resource_check_at) {
            (Some(r), Some(at)) => json!({
                "checked": true,
                "checked_at": at.to_rfc3339(),
                "green": r.green,
                "available_ram_gb": r.available_ram_gb,
                "swap_used_pct": r.swap_used_pct,
                "load_per_core": r.load_per_core,
                "warnings": r.warnings,
            }),
            _ => json!({"checked": false}),
        }
    }

    // ── Tab 3: Hala ─────────────────────────────────────────────────

    fn start_docpulse(&mut self, cfg: DocPulseCfg) {
        self.dp_stage_statuses = vec![PbStatus::Pending; docpulse::STAGE_NAMES.len()];
        self.dp_events.clear();
        self.dp_halt_event = None;
        self.dp_log.clear();
        self.dp_started = Some(chrono::Utc::now());
        self.dp_finished = None;
        self.dp_phase = RunPhase::Running;

        let (tx, rx) = crossbeam_channel::unbounded();
        let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded();
        docpulse::spawn_docpulse(cfg, tx, ctl_rx);
        self.dp_rx = Some(rx);
        self.dp_ctl_tx = Some(ctl_tx);
    }

    fn pump_docpulse_events(&mut self) {
        let Some(rx) = &self.dp_rx else { return };
        let drained: Vec<RunnerEvent> = rx.try_iter().collect();
        for ev in drained {
            match ev {
                RunnerEvent::PbStarted(i) => {
                    if let Some(s) = self.dp_stage_statuses.get_mut(i) {
                        *s = PbStatus::Running;
                    }
                }
                RunnerEvent::PbOk(i) => {
                    if let Some(s) = self.dp_stage_statuses.get_mut(i) {
                        if matches!(self.dp_phase, RunPhase::Halted(h) if h == i) {
                            self.dp_phase = RunPhase::Running;
                        }
                        *s = PbStatus::Ok;
                    }
                }
                RunnerEvent::PbWarned(i, e) => {
                    if let Some(s) = self.dp_stage_statuses.get_mut(i) {
                        *s = PbStatus::Warned;
                    }
                    self.dp_events.push(e);
                }
                RunnerEvent::PbFailed(i, e) => {
                    if let Some(s) = self.dp_stage_statuses.get_mut(i) {
                        *s = PbStatus::Failed;
                    }
                    self.dp_events.push(e);
                    self.dp_halt_event = Some(self.dp_events.len() - 1);
                    self.dp_phase = RunPhase::Halted(i);
                }
                RunnerEvent::Log(l) => self.dp_log.push(l),
                RunnerEvent::Finished => {
                    self.dp_phase = RunPhase::Finished;
                    self.dp_finished = Some(chrono::Utc::now());
                }
                RunnerEvent::Aborted => {
                    self.dp_phase = RunPhase::Aborted;
                    self.dp_finished = Some(chrono::Utc::now());
                }
            }
        }
    }

    fn docpulse_json(&self) -> Value {
        let (phase, halted_stage) = match self.dp_phase {
            RunPhase::Idle => ("idle", None),
            RunPhase::Running => ("running", None),
            RunPhase::Halted(i) => ("halted", Some(i)),
            RunPhase::Finished => ("finished", None),
            RunPhase::Aborted => ("aborted", None),
        };
        let events: Vec<Value> = self
            .dp_events
            .iter()
            .map(|ev| {
                json!({
                    "pb_index": ev.pb_index, "playbook": ev.playbook, "task": ev.task,
                    "error_type": ev.error_type, "message": ev.message, "severity": ev.severity.to_string(),
                })
            })
            .collect();
        json!({
            "phase": phase,
            "halted_stage": halted_stage,
            "can_start": matches!(self.dp_phase, RunPhase::Idle | RunPhase::Finished | RunPhase::Aborted),
            "can_abort": matches!(self.dp_phase, RunPhase::Running | RunPhase::Halted(_)),
            "stages": docpulse::STAGE_NAMES,
            "statuses": self.dp_stage_statuses,
            "events": events,
            "halt_event": self.dp_halt_event,
            "log_tail": self.dp_log.iter().rev().take(20).rev().collect::<Vec<_>>(),
        })
    }

    fn load_config(&mut self) {
        match Config::load(std::path::Path::new(&self.cfg_path)) {
            Ok(cfg) => {
                self.params = cfg.parameters.iter().map(|p| (p.key.clone(), p.value.clone())).collect();
                self.statuses = vec![PbStatus::Pending; cfg.run.playbooks.len()];
                self.status_line = format!(
                    "config ✓ {} playbooks resolved{}",
                    cfg.run.playbooks.len(),
                    if cfg.run.simulate { " · SIMULATE mode" } else { "" }
                );
                self.cfg = Some(cfg);
            }
            Err(e) => self.status_line = format!("config error: {e}"),
        }
    }

    fn start_run(&mut self) {
        let Some(cfg) = self.cfg.clone() else { return };
        self.statuses = vec![PbStatus::Pending; cfg.run.playbooks.len()];
        self.events.clear();
        self.fixes.clear();
        self.halt_event = None;
        self.log.clear();
        self.report_path = None;
        self.started = Some(chrono::Utc::now());
        self.finished = None;
        self.phase = RunPhase::Running;

        let (tx, rx) = crossbeam_channel::unbounded();
        let (ctl_tx, ctl_rx) = crossbeam_channel::unbounded();
        runner::spawn_runner(cfg, self.params.clone(), tx, ctl_rx);
        self.rx = Some(rx);
        self.ctl_tx = Some(ctl_tx);
    }

    fn pump_events(&mut self) {
        let Some(rx) = &self.rx else { return };
        let drained: Vec<RunnerEvent> = rx.try_iter().collect();
        for ev in drained {
            match ev {
                RunnerEvent::PbStarted(i) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        *s = PbStatus::Running;
                    }
                }
                RunnerEvent::PbOk(i) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        if matches!(self.phase, RunPhase::Halted(h) if h == i) {
                            self.phase = RunPhase::Running;
                        }
                        *s = PbStatus::Ok;
                    }
                }
                RunnerEvent::PbWarned(i, e) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        *s = PbStatus::Warned;
                    }
                    self.events.push(e);
                }
                RunnerEvent::PbFailed(i, e) => {
                    if let Some(s) = self.statuses.get_mut(i) {
                        *s = PbStatus::Failed;
                    }
                    self.events.push(e);
                    self.halt_event = Some(self.events.len() - 1);
                    self.phase = RunPhase::Halted(i);
                }
                RunnerEvent::Log(l) => self.log.push(l),
                RunnerEvent::Finished => {
                    self.phase = RunPhase::Finished;
                    self.finished = Some(chrono::Utc::now());
                }
                RunnerEvent::Aborted => {
                    self.phase = RunPhase::Aborted;
                    self.finished = Some(chrono::Utc::now());
                }
            }
        }
    }

    fn generate_report(&mut self) {
        let Some(cfg) = &self.cfg else { return };
        let run = report::RunSummary {
            started: self.started.unwrap_or_else(chrono::Utc::now),
            finished: self.finished.unwrap_or_else(chrono::Utc::now),
            playbooks: &cfg.run.playbooks,
            statuses: &self.statuses,
            events: &self.events,
            parameters: &self.params,
            fixes_generated: &self.fixes,
        };
        match report::generate(&cfg.run.report_dir, &cfg.run.seal_key, &run) {
            Ok(p) => {
                self.status_line = format!("𒁾 sealed report: {}", p.display());
                self.report_path = Some(p);
            }
            Err(e) => self.status_line = format!("report error: {e}"),
        }
    }

    /// JSON snapshot the browser polls — deliberately includes everything
    /// the frontend needs to redraw from scratch each poll (this is a
    /// governor for a human, not a high-frequency feed; simplicity over
    /// incremental diffing).
    fn to_json(&self) -> Value {
        let (phase, halted_pb) = match self.phase {
            RunPhase::Idle => ("idle", None),
            RunPhase::Running => ("running", None),
            RunPhase::Halted(i) => ("halted", Some(i)),
            RunPhase::Finished => ("finished", None),
            RunPhase::Aborted => ("aborted", None),
        };
        let playbooks: Vec<&String> = self.cfg.as_ref().map(|c| c.run.playbooks.iter().collect()).unwrap_or_default();
        let ok = self.statuses.iter().filter(|s| **s == PbStatus::Ok).count();
        let warnings = self.statuses.iter().filter(|s| **s == PbStatus::Warned).count();
        let errors = self.statuses.iter().filter(|s| **s == PbStatus::Failed).count();
        let done = self.statuses.iter().filter(|s| !matches!(s, PbStatus::Pending | PbStatus::Running)).count();

        let events: Vec<Value> = self.events.iter().map(|ev| {
            let remedy = self.cfg.as_ref().and_then(|c| c.find_remedy(&ev.message)).map(|r| json!({
                "id": r.id, "diagnosis": r.diagnosis, "solution": r.solution,
            }));
            json!({
                "pb_index": ev.pb_index,
                "playbook": ev.playbook,
                "task": ev.task,
                "error_type": ev.error_type,
                "message": ev.message,
                "severity": ev.severity.to_string(),
                "remedy": remedy,
            })
        }).collect();

        json!({
            "cfg_path": self.cfg_path,
            "cfg_loaded": self.cfg.is_some(),
            "simulate": self.cfg.as_ref().map(|c| c.run.simulate).unwrap_or(false),
            "status_line": self.status_line,
            "phase": phase,
            "halted_pb": halted_pb,
            "can_run": self.cfg.is_some() && matches!(self.phase, RunPhase::Idle | RunPhase::Finished | RunPhase::Aborted),
            "can_abort": matches!(self.phase, RunPhase::Running | RunPhase::Halted(_)),
            "params": self.params.iter().map(|(k, v)| json!({"key": k, "value": v})).collect::<Vec<_>>(),
            "playbooks": playbooks,
            "statuses": self.statuses,
            "events": events,
            "halt_event": self.halt_event,
            "counts": {"ok": ok, "warnings": warnings, "errors": errors, "done": done, "total": self.statuses.len()},
            "log_tail": self.log.iter().rev().take(20).rev().collect::<Vec<_>>(),
            "report_path": self.report_path.as_ref().map(|p| p.display().to_string()),
        })
    }
}

struct AppCtx {
    state: Arc<Mutex<ServerState>>,
    sessions: Arc<SessionStore>,
    defender: Arc<LoginDefender>,
}

fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("SHAKKANAKKU_WEB_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8763);
    // Loopback by default -- deliberate. A network-reachable governor for
    // real infrastructure with no auth in front of it is not a "resize
    // problem", it's an open door; auth now stands in front of it, but
    // loopback stays the default anyway (defense in depth, and it's the
    // only network posture that needs zero configuration to be correct).
    // Set SHAKKANAKKU_WEB_BIND=0.0.0.0 (or a specific LAN address)
    // explicitly to serve beyond this host.
    let bind: String = std::env::var("SHAKKANAKKU_WEB_BIND").unwrap_or_else(|_| "127.0.0.1".into());
    let addr = format!("{bind}:{port}");
    let listener = TcpListener::bind(&addr)?;

    let repo_root = std::path::Path::new(".");
    let tls_config =
        web_tls::load_or_generate_config(&repo_root.join("secrets/shakkanakku_web_cert.pem"), &repo_root.join("secrets/shakkanakku_web_key.pem"))?;

    let ctx = Arc::new(AppCtx {
        state: Arc::new(Mutex::new(ServerState::new())),
        sessions: Arc::new(SessionStore::new()),
        defender: Arc::new(LoginDefender::new()),
    });

    println!("𒑐 Shakkanakku web governor listening on https://{addr}/ (Ctrl-C to stop)");
    println!("   self-signed certificate — your browser will warn once; trust it explicitly, then it's remembered.");
    println!("   log in with a Sargon-vault file (sargon_vault.dat) + its passphrase to unlock the Architect view.");
    if bind != "127.0.0.1" && bind != "localhost" && bind != "::1" {
        println!(
            "   ⚠ bound to {bind}, not loopback — reachable from the network. \
             Every action beyond viewing still requires an Architect-privilege \
             passport session, but confirm this is intentional."
        );
    }

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let Ok(peer_addr) = stream.peer_addr() else { continue };
        let tls_config = Arc::clone(&tls_config);
        let ctx = Arc::clone(&ctx);
        std::thread::spawn(move || {
            let conn = match rustls::ServerConnection::new(tls_config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("TLS setup error: {e}");
                    return;
                }
            };
            let tls_stream = rustls::StreamOwned::new(conn, stream);
            if let Err(e) = handle_connection(tls_stream, peer_addr.ip(), &ctx) {
                eprintln!("connection error: {e}");
            }
        });
    }
    Ok(())
}

type TlsStream = rustls::StreamOwned<rustls::ServerConnection, TcpStream>;

fn handle_connection(mut stream: TlsStream, peer_ip: IpAddr, ctx: &AppCtx) -> std::io::Result<()> {
    let mut request_line = String::new();
    let mut content_length: usize = 0;
    let mut cookie_header: Option<String> = None;
    let mut body = Vec::new();
    {
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut request_line)?;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 || line == "\r\n" {
                break;
            }
            let lower = line.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                content_length = v.trim().parse().unwrap_or(0);
            } else if lower.starts_with("cookie:") {
                cookie_header = Some(line[7..].trim().to_string());
            }
        }
        if content_length > 0 {
            body = vec![0u8; content_length];
            reader.read_exact(&mut body)?;
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();
    let (path_only, query) = path.split_once('?').unwrap_or((&path, ""));
    let session_cookie = cookie_header.as_deref().and_then(extract_session_cookie);

    let (status, content_type, headers, resp_body) = route(&method, path_only, query, &body, peer_ip, session_cookie, ctx);
    write_response(&mut stream, status, content_type, &headers, &resp_body)
}

fn extract_session_cookie(cookie_header: &str) -> Option<String> {
    cookie_header.split(';').find_map(|kv| {
        let (k, v) = kv.trim().split_once('=')?;
        (k == SESSION_COOKIE).then(|| v.to_string())
    })
}

fn write_response(
    stream: &mut TlsStream,
    status: u16,
    content_type: &str,
    extra_headers: &[(String, String)],
    body: &[u8],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        429 => "Too Many Requests",
        501 => "Not Implemented",
        _ => "Internal Server Error",
    };
    let mut header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (k, v) in extra_headers {
        header.push_str(&format!("{k}: {v}\r\n"));
    }
    header.push_str("\r\n");
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

type Resp = (u16, &'static str, Vec<(String, String)>, Vec<u8>);

fn json_ok(v: Value) -> Resp {
    (200, "application/json; charset=utf-8", vec![], serde_json::to_vec(&v).unwrap_or_default())
}

fn json_resp(status: u16, v: Value) -> Resp {
    (status, "application/json; charset=utf-8", vec![], serde_json::to_vec(&v).unwrap_or_default())
}

fn text(status: u16, msg: impl Into<String>) -> Resp {
    (status, "text/plain; charset=utf-8", vec![], msg.into().into_bytes())
}

fn unauthorized() -> Resp {
    json_resp(401, json!({"error": "not authenticated — log in with a Sargon vault + passphrase"}))
}

fn forbidden(msg: &str) -> Resp {
    json_resp(403, json!({"error": msg}))
}

fn too_many_requests(retry_after: Duration) -> Resp {
    let secs = retry_after.as_secs().max(1);
    let mut resp = json_resp(429, json!({"error": format!("too many failed login attempts — try again in {secs}s")}));
    resp.2.push(("Retry-After".into(), secs.to_string()));
    resp
}

fn session_cookie_header(value: &str) -> (String, String) {
    ("Set-Cookie".into(), format!("{SESSION_COOKIE}={value}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={SESSION_MAX_AGE_SECS}"))
}

fn clear_cookie_header() -> (String, String) {
    ("Set-Cookie".into(), format!("{SESSION_COOKIE}=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0"))
}

fn identity_json(id: &AuthedIdentity) -> Value {
    json!({
        "realm": id.realm,
        "privilege_level": id.privilege_level,
        "is_architect": id.is_architect(),
        "passport_id": id.passport_id,
    })
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
        (k == key).then(|| url_decode(v))
    })
}

/// Minimal `application/x-www-form-urlencoded` decode — sufficient for the
/// plain ascii tokens (`md`/`html`/`docx`) this server's own query params
/// ever carry.
fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                if let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(byte as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

/// Serves the sealed report (last one generated this session) in the
/// requested export format. The canonical, AkkadianSeal-signed artifact is
/// always the `.md` file `generate_report` writes to `report_dir` — these
/// are read-only views/conversions of that same content, not separate
/// sources of truth.
fn export_report(format: &str, state: &Arc<Mutex<ServerState>>) -> Resp {
    let report_path = { state.lock().unwrap().report_path.clone() };
    let Some(path) = report_path else {
        return text(400, "no report generated yet — click \"Generate final report\" first");
    };
    let md = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return text(500, format!("cannot read report: {e}")),
    };
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("shakkanakku_report").to_string();

    match format {
        "md" => (
            200,
            "text/markdown; charset=utf-8",
            vec![("X-Suggested-Filename".into(), format!("{stem}.md"))],
            md.into_bytes(),
        ),
        "html" => {
            let html = report::to_html(&md, "Shakkanakku — Infrastructure Deployment Report");
            (
                200,
                "text/html; charset=utf-8",
                vec![("X-Suggested-Filename".into(), format!("{stem}.html"))],
                html.into_bytes(),
            )
        }
        "docx" => {
            let tmp = std::env::temp_dir().join(format!("{stem}_{}.docx", std::process::id()));
            match std::process::Command::new("pandoc").arg(&path).arg("-o").arg(&tmp).output() {
                Ok(o) if o.status.success() => match std::fs::read(&tmp) {
                    Ok(bytes) => {
                        let _ = std::fs::remove_file(&tmp);
                        (
                            200,
                            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                            vec![("X-Suggested-Filename".into(), format!("{stem}.docx"))],
                            bytes,
                        )
                    }
                    Err(e) => text(500, format!("pandoc ran but its output was unreadable: {e}")),
                },
                Ok(o) => text(501, format!("pandoc failed: {}", String::from_utf8_lossy(&o.stderr))),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => text(
                    501,
                    "Word export needs pandoc, which isn't installed on this host. \
                     Install it (e.g. `sudo dnf install pandoc`) and try again — \
                     Markdown, HTML, and PDF (via the browser's print dialog) don't need it.",
                ),
                Err(e) => text(500, format!("cannot run pandoc: {e}")),
            }
        }
        _ => text(400, "unknown format — use md, html, or docx"),
    }
}

// ── Shala4: Gate Orbits ───────────────────────────────────────────────────
//
// Playbook-to-HeptaGate classification is suggest-then-approve only (see
// `shakkanakku::gate_review`'s own doc comment) -- nothing in this section
// ever writes a `meta.gate` tag on its own, only ever the exact pairs an
// explicit `/api/gates/approve` POST body names.
//
// Same default paths `playbook_289_multi_location_playbook_catalog.yml`/
// `playbook_290_publish_and_serve_enkiddb_read_node.yml` already
// established (`~/Forge/pb_catalog_registry.jsonl`, `~/Forge/enkiddb_local`),
// overridable via env vars -- same convention `enkiddb-read-server`'s own
// `DATA_DIR` already uses.

fn pb_catalog_registry_path() -> PathBuf {
    std::env::var("PB_CATALOG_REGISTRY").map(PathBuf::from).unwrap_or_else(|_| home_dir().join("Forge/pb_catalog_registry.jsonl"))
}

fn gate_registry_path() -> PathBuf {
    std::env::var("PB_GATE_REGISTRY").map(PathBuf::from).unwrap_or_else(|_| home_dir().join("Forge/pb_gate_registry.jsonl"))
}

fn enkiddb_output_root() -> PathBuf {
    std::env::var("PB_CATALOG_ENKIDDB_ROOT").map(PathBuf::from).unwrap_or_else(|_| home_dir().join("Forge/enkiddb_local"))
}

fn home_dir() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."))
}

/// Opens the `current` generation -- the same symlink
/// `playbook_290_publish_and_serve_enkiddb_read_node.yml` (or this
/// server's own `approve_gates`) last promoted -- as a fresh Read Node.
/// `None` if nothing has ever been published there yet: a real, expected
/// state (a fresh catalog run before its first PB-290/approve), not an
/// error to surface as one.
fn open_current_read_node() -> Option<enkiddb::ReadNode> {
    let current = enkiddb_output_root().join("current");
    enkiddb::ReadNode::open(current.join("entities"), current.join("eav")).ok()
}

fn gate_json(gate: HeptaGate, playbook_count: usize) -> Value {
    json!({
        "gate_number": gate.gate_number(),
        "akkadian_name": gate.akkadian_name(),
        "sector": gate.sector(),
        "pipeline_function": gate.pipeline_function(),
        "playbook_count": playbook_count,
    })
}

/// How many entities currently carry `meta.gate = <gate's Akkadian name>`
/// in `read_node`. `None` only on a genuine query failure (malformed
/// generation) -- an empty match is `Some(0)`, not `None`.
fn gate_count(read_node: &mut enkiddb::ReadNode, gate_name: &str) -> Option<usize> {
    let escaped = gate_name.replace('"', "\\\"");
    let query = format!("WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.gate] = \"{escaped}\"");
    read_node.query(&query).ok().map(|r| r.matched.len())
}

fn gates_summary_json() -> Value {
    let total_playbooks =
        std::fs::read_to_string(pb_catalog_registry_path()).unwrap_or_default().lines().filter(|l| !l.trim().is_empty()).count();

    let mut read_node = open_current_read_node();
    let mut classified = 0usize;
    let gates: Vec<Value> = HeptaGate::all()
        .into_iter()
        .map(|gate| {
            let count = read_node.as_mut().and_then(|rn| gate_count(rn, gate.akkadian_name())).unwrap_or(0);
            classified += count;
            gate_json(gate, count)
        })
        .collect();

    // Registry-based, same as `total_playbooks` above, not a live EnkiDDB
    // query -- so this stays accurate even if `current` hasn't been
    // (re)published yet, and makes "0 approved so far" visibly distinct
    // from "current isn't published" (see `current_published` below)
    // rather than both silently looking like an empty dashboard.
    let approved_dependency_count = pb_dependency_review::approved_pairs(&dependency_registry_path()).len();

    json!({
        "gates": gates,
        "total_playbooks": total_playbooks,
        // Not a HeptaScript NOT_EXISTS query -- ReadNode's indexable_condition
        // only serves a leading exact-equality WHERE, so arithmetic against
        // the catalog registry's own total is what's actually queryable here.
        "unclassified_count": total_playbooks.saturating_sub(classified),
        "approved_dependency_count": approved_dependency_count,
        "current_published": read_node.is_some(),
    })
}

/// The two numerically-adjacent gates' summaries (gate_number ± 1, clamped
/// at the ends -- not cyclic) -- the "upper/lower layer" reference the
/// Architect asked a gate's playbook list to carry.
fn adjacent_gate_json(read_node: &mut enkiddb::ReadNode, gate: HeptaGate) -> Option<Value> {
    let neighbor = HeptaGate::from_gate_number(gate.gate_number())?;
    let count = gate_count(read_node, neighbor.akkadian_name()).unwrap_or(0);
    Some(gate_json(neighbor, count))
}

fn gate_playbooks_resp(gate_name: &str) -> Resp {
    let Some(gate) = HeptaGate::from_akkadian_name(gate_name) else {
        return text(400, format!("unknown gate '{gate_name}' -- expected one of the 7 real HeptaGate Akkadian names"));
    };
    let Some(mut read_node) = open_current_read_node() else {
        return json_ok(json!({
            "gate": gate_json(gate, 0),
            "playbooks": [],
            "lower": Value::Null,
            "upper": Value::Null,
            "note": "no EnkiDDB generation has been published yet -- run playbook_290 (or approve a gate suggestion here) first",
        }));
    };

    let count = gate_count(&mut read_node, gate.akkadian_name()).unwrap_or(0);
    let escaped = gate.akkadian_name().replace('"', "\\\"");
    let query = format!("WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.gate] = \"{escaped}\"");
    let titles: Vec<String> = match read_node.query(&query) {
        Ok(r) => r
            .matched
            .iter()
            .filter_map(|m| {
                m.projected.iter().find(|(k, _)| k == "meta.title").and_then(|(_, v)| match v {
                    akkvalue::AkkValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
            })
            .collect(),
        Err(_) => vec![],
    };

    let lower_num = gate.gate_number().checked_sub(1).and_then(HeptaGate::from_gate_number);
    let upper_num = gate.gate_number().checked_add(1).and_then(HeptaGate::from_gate_number);
    let lower = lower_num.and_then(|g| adjacent_gate_json(&mut read_node, g));
    let upper = upper_num.and_then(|g| adjacent_gate_json(&mut read_node, g));

    json_ok(json!({
        "gate": gate_json(gate, count),
        "playbooks": titles,
        "lower": lower,
        "upper": upper,
    }))
}

/// Points `root/current` at `generation`'s directory, exactly matching
/// `playbook_290_publish_and_serve_enkiddb_read_node.yml`'s own
/// remove-then-relink steps -- run here so approving a gate is visible
/// immediately in Shala without a separate terminal round-trip. This does
/// not spawn any new shell/git command (the Architect's standing rule
/// about wrapping those in ansible PBs run from Shala) -- it's in-process
/// Rust logic inside the already-launched `shakkanakku_web` binary,
/// exactly like every other `/api/*` handler in this file.
fn promote_current(root: &Path, generation: &enkiddb::Generation) -> std::io::Result<()> {
    let current = root.join("current");
    let target = generation.entities_path.parent().expect("entities_path always has a parent directory").to_path_buf();
    if current.symlink_metadata().is_ok() {
        std::fs::remove_file(&current).or_else(|_| std::fs::remove_dir_all(&current))?;
    }
    std::os::unix::fs::symlink(&target, &current)
}

/// Thin wrapper over `shakkanakku::pb_catalog_rebuild::rebuild_full_write_node`
/// (a path-parameterized, fully unit-tested lib function) supplying this
/// server's own env-resolved registry paths -- see that module's doc
/// comment for why every `approve_*` handler needs this instead of an
/// empty `WriteNode::new()`.
fn rebuild_full_write_node() -> (enkiddb::WriteNode, u32, HashMap<String, (enkidb_kaki::IdentityKaki, String)>) {
    shakkanakku::pb_catalog_rebuild::rebuild_full_write_node(
        &pb_catalog_registry_path(),
        &gate_registry_path(),
        &domain_registry_path(),
        &dependency_registry_path(),
    )
}

/// Shared tail for every handler that mutates then republishes `current`:
/// materialize `write_node`, promote it, and only THEN call `extra_fn` to
/// fold any extra fields (e.g. `approve_gates`' own `summary`) into the
/// success JSON body alongside `ok`/`log` -- `extra_fn` runs after
/// `promote_current` succeeds specifically so a summary it computes (by
/// re-opening `current`) reflects the generation just promoted, not
/// whatever `current` pointed to a moment before.
fn materialize_and_promote(
    write_node: &enkiddb::WriteNode,
    log: Vec<String>,
    extra_fn: impl FnOnce() -> serde_json::Map<String, Value>,
) -> Resp {
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();
    let root = enkiddb_output_root();
    let generation = match enkiddb::materialize_version(write_node, &root, &stamp) {
        Ok((generation, _stats)) => generation,
        Err(e) => return json_resp(500, json!({ "ok": false, "log": log, "error": format!("materialize failed: {e}") })),
    };
    if let Err(e) = promote_current(&root, &generation) {
        return json_resp(
            500,
            json!({ "ok": false, "log": log, "error": format!("materialized, but could not promote as current: {e}") }),
        );
    }
    let mut extra = extra_fn();
    extra.insert("ok".to_string(), json!(true));
    extra.insert("log".to_string(), json!(log));
    json_ok(Value::Object(extra))
}

fn refresh_current_resp() -> Resp {
    let (write_node, _epoch, by_kaki_hex) = rebuild_full_write_node();
    let log = vec![format!("𒁾 republished current from {} catalogued playbook(s), all approved tags/edges replayed", by_kaki_hex.len())];
    materialize_and_promote(&write_node, log, serde_json::Map::new)
}

fn approve_gates(body: &[u8]) -> Resp {
    let approvals_req: Vec<Value> = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return text(400, "malformed request body -- expected a JSON array of {pb_kaki_hex, gate_akkadian_name}"),
    };
    if approvals_req.is_empty() {
        return text(400, "no approvals in request body");
    }

    let mut approvals = Vec::with_capacity(approvals_req.len());
    for item in &approvals_req {
        let Some(kaki_hex) = item.get("pb_kaki_hex").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs pb_kaki_hex");
        };
        let Some(gate_name) = item.get("gate_akkadian_name").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs gate_akkadian_name");
        };
        let Some(gate) = HeptaGate::from_akkadian_name(gate_name) else {
            return text(400, format!("unknown gate '{gate_name}'"));
        };
        let Some(kaki) = gate_review::parse_kaki_hex(kaki_hex) else {
            return text(400, format!("malformed pb_kaki_hex '{kaki_hex}'"));
        };
        approvals.push((kaki, gate.akkadian_name().to_string()));
    }

    let (mut write_node, epoch, _by_kaki_hex) = rebuild_full_write_node();
    let log = gate_review::apply_gate_approvals(&mut write_node, &approvals, &gate_registry_path(), epoch);

    materialize_and_promote(&write_node, log, || {
        let mut extra = serde_json::Map::new();
        extra.insert("summary".to_string(), gates_summary_json());
        extra
    })
}

// ── Shala4: domains within a gate ──────────────────────────────────────

fn domain_registry_path() -> PathBuf {
    std::env::var("PB_DOMAIN_REGISTRY").map(PathBuf::from).unwrap_or_else(|_| home_dir().join("Forge/pb_domain_registry.jsonl"))
}

fn domain_json(gate: HeptaGate, domain_name: &str, playbook_count: usize) -> Value {
    json!({ "gate_akkadian_name": gate.akkadian_name(), "domain_name": domain_name, "playbook_count": playbook_count })
}

/// How many entities currently carry BOTH `meta.domain = domain_name` AND
/// `meta.gate = gate`'s Akkadian name. The AND matters: a handful of
/// domain names (e.g. "Encryption", "Auditing") deliberately repeat
/// across two different gates in `domain_review::DOMAIN_NAMES` -- without
/// the gate condition too, those would double-count. `meta.domain` is the
/// leading (indexed) WHERE condition; `meta.gate` is evaluated as a
/// second AND condition over that same candidate set (heptascript's
/// engine evaluates every WHERE condition, not just the indexed first
/// one -- see `heptascript::engine::eval_where`).
fn domain_count(read_node: &mut enkiddb::ReadNode, gate: HeptaGate, domain_name: &str) -> Option<usize> {
    let d = domain_name.replace('"', "\\\"");
    let g = gate.akkadian_name().replace('"', "\\\"");
    let query = format!("WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.domain] = \"{d}\" AND E[meta.gate] = \"{g}\"");
    read_node.query(&query).ok().map(|r| r.matched.len())
}

fn gate_domains_summary_resp(gate_name: &str) -> Resp {
    let Some(gate) = HeptaGate::from_akkadian_name(gate_name) else {
        return text(400, format!("unknown gate '{gate_name}'"));
    };
    let Some(domains) = domain_review::domains_for_gate(gate.akkadian_name()) else {
        return text(500, format!("no domain taxonomy defined for gate '{gate_name}'"));
    };
    let Some(mut read_node) = open_current_read_node() else {
        return json_ok(json!({
            "gate_akkadian_name": gate.akkadian_name(),
            "domains": domains.iter().map(|d| domain_json(gate, d, 0)).collect::<Vec<_>>(),
            "gate_total": 0,
            "unclassified_in_gate": 0,
            "note": "no EnkiDDB generation has been published yet",
        }));
    };

    let gate_total = gate_count(&mut read_node, gate.akkadian_name()).unwrap_or(0);
    let mut classified = 0usize;
    let domain_list: Vec<Value> = domains
        .iter()
        .map(|d| {
            let c = domain_count(&mut read_node, gate, d).unwrap_or(0);
            classified += c;
            domain_json(gate, d, c)
        })
        .collect();

    json_ok(json!({
        "gate_akkadian_name": gate.akkadian_name(),
        "domains": domain_list,
        "gate_total": gate_total,
        "unclassified_in_gate": gate_total.saturating_sub(classified),
    }))
}

fn domain_playbooks_resp(gate_name: &str, domain_name: &str) -> Resp {
    let Some(gate) = HeptaGate::from_akkadian_name(gate_name) else {
        return text(400, format!("unknown gate '{gate_name}'"));
    };
    let Some(mut read_node) = open_current_read_node() else {
        return json_ok(json!({
            "gate_akkadian_name": gate.akkadian_name(), "domain_name": domain_name, "playbooks": [],
            "note": "no EnkiDDB generation has been published yet",
        }));
    };
    let d = domain_name.replace('"', "\\\"");
    let g = gate.akkadian_name().replace('"', "\\\"");
    let query = format!("WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.domain] = \"{d}\" AND E[meta.gate] = \"{g}\"");
    let playbooks: Vec<Value> = match read_node.query(&query) {
        Ok(r) => r
            .matched
            .iter()
            .filter_map(|m| {
                m.projected.iter().find(|(k, _)| k == "meta.title").and_then(|(_, v)| match v {
                    akkvalue::AkkValue::Text(title) => Some(json!({
                        "title": title,
                        "kaki_hex": hex::encode(m.entity.bytes()),
                    })),
                    _ => None,
                })
            })
            .collect(),
        Err(_) => vec![],
    };
    json_ok(json!({ "gate_akkadian_name": gate.akkadian_name(), "domain_name": domain_name, "playbooks": playbooks }))
}

fn scan_domain_suggestions_resp(gate_name: &str) -> Resp {
    if HeptaGate::from_akkadian_name(gate_name).is_none() {
        return text(400, format!("unknown gate '{gate_name}'"));
    }
    let suggestions =
        domain_review::scan_domain_suggestions(&pb_catalog_registry_path(), &gate_registry_path(), &domain_registry_path(), gate_name);
    json_ok(json!({ "suggestions": suggestions }))
}

fn approve_domains(gate_name: &str, body: &[u8]) -> Resp {
    let Some(gate) = HeptaGate::from_akkadian_name(gate_name) else {
        return text(400, format!("unknown gate '{gate_name}'"));
    };
    let Some(valid_domains) = domain_review::domains_for_gate(gate.akkadian_name()) else {
        return text(500, format!("no domain taxonomy defined for gate '{gate_name}'"));
    };

    let approvals_req: Vec<Value> = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return text(400, "malformed request body -- expected a JSON array of {pb_kaki_hex, domain_name}"),
    };
    if approvals_req.is_empty() {
        return text(400, "no approvals in request body");
    }

    let mut approvals = Vec::with_capacity(approvals_req.len());
    for item in &approvals_req {
        let Some(kaki_hex) = item.get("pb_kaki_hex").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs pb_kaki_hex");
        };
        let Some(domain_name) = item.get("domain_name").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs domain_name");
        };
        if !valid_domains.iter().any(|d| *d == domain_name) {
            return text(400, format!("'{domain_name}' is not one of {gate_name}'s 7 domain names"));
        }
        let Some(kaki) = gate_review::parse_kaki_hex(kaki_hex) else {
            return text(400, format!("malformed pb_kaki_hex '{kaki_hex}'"));
        };
        approvals.push((kaki, gate.akkadian_name().to_string(), domain_name.to_string()));
    }

    let (mut write_node, epoch, _by_kaki_hex) = rebuild_full_write_node();
    let log = domain_review::apply_domain_approvals(&mut write_node, &approvals, &domain_registry_path(), epoch);

    materialize_and_promote(&write_node, log, serde_json::Map::new)
}

fn akk_value_to_json(v: &akkvalue::AkkValue) -> Value {
    match v {
        akkvalue::AkkValue::Text(s) => json!(s),
        akkvalue::AkkValue::Int(i) => json!(i),
        akkvalue::AkkValue::Float(f) => json!(f),
        akkvalue::AkkValue::Bool(b) => json!(b),
        akkvalue::AkkValue::Null => Value::Null,
        other => json!(other.to_string()),
    }
}

/// Thin wrapper over `shakkanakku::pb_story::playbook_story_events` (a
/// path-parameterized, fully unit-tested lib function) supplying this
/// server's own env-resolved registry paths.
fn playbook_story_json(kaki_hex: &str) -> Vec<shakkanakku::pb_story::StoryEvent> {
    shakkanakku::pb_story::playbook_story_events(
        &pb_catalog_registry_path(),
        &gate_registry_path(),
        &domain_registry_path(),
        &dependency_registry_path(),
        kaki_hex,
    )
}

/// StoryEngine detail for one playbook, looked up by its exact
/// `meta.title` (query-param `title`) -- everything the click-a-cube
/// panel needs: its real KAKI, and every attribute it currently carries
/// (title, collection, gate/domain tags if approved, the
/// `[playbook-catalog] source_path=... content_sha256=...` body marker
/// `pb_catalog::catalog_playbooks` always writes, and its `hist.event`).
/// Real provenance already in EnkiDDB -- nothing simulated or invented
/// for display, unlike the original BIGRING/ETL-dashboard prototypes'
/// synthetic StoryEngine event templates.
fn playbook_detail_resp(title: &str) -> Resp {
    if title.is_empty() {
        return text(400, "missing title query param");
    }
    let Some(mut read_node) = open_current_read_node() else {
        return text(404, "no EnkiDDB generation has been published yet");
    };
    let escaped = title.replace('"', "\\\"");
    let query = format!(
        "WHO T.E\nWHAT E[meta.title, meta.collection, meta.gate, meta.domain, body.paragraph, hist.event]\nWHERE E[meta.title] = \"{escaped}\""
    );
    match read_node.query(&query) {
        Ok(r) if !r.matched.is_empty() => {
            let m = &r.matched[0];
            let mut attrs = serde_json::Map::new();
            for (k, v) in &m.projected {
                attrs.insert(k.clone(), akk_value_to_json(v));
            }
            let depends_on = linked_titles(&mut read_node, "link.source_title", title, "link.target_title");
            let depended_on_by = linked_titles(&mut read_node, "link.target_title", title, "link.source_title");
            let kaki_hex = hex::encode(m.entity.bytes());
            let story = playbook_story_json(&kaki_hex);
            let why = playbook_why_text(&mut read_node, m.entity);
            json_ok(json!({
                "kaki_hex": kaki_hex,
                "title": title,
                "attrs": attrs,
                "why": why,
                "depends_on": depends_on,
                "depended_on_by": depended_on_by,
                "story": story,
            }))
        }
        Ok(_) => text(404, "no entity with that exact title"),
        Err(e) => text(500, format!("query failed: {e:?}")),
    }
}

/// The playbook's real, full header prose ("WHY THIS EXISTS: ...",
/// "WHAT:", "WHERE:", etc.) -- FIXED, real gap: `body.paragraph` on the
/// parent entity is a single EAV scalar (last-write-wins), and
/// `pb_catalog::catalog_playbooks` appends its own `[playbook-catalog]
/// source_path=...` bookkeeping line as the LAST body paragraph, so that
/// internal marker -- not the real WHY text -- was the one
/// `body.paragraph` ever actually returned.
///
/// The real content survives intact one level down, in the document's
/// child sections -- but NOT as one bare section whose title equals the
/// parent's own title: `DocumentParser::parse_playbook_header` recognizes
/// "PREFIX: text" lines (`WHY THIS EXISTS:`, `WHAT:`, `WHERE:`, `LAW:`,
/// ...) as real headers, so a typical W5H2-style playbook mints ONE
/// section PER such prefix, each titled `"{parent title} § {PREFIX}"` --
/// there's no single fixed title to guess. Matching by title is a dead
/// end here; matching by the section's real `link.target` KakiPk bytes
/// against `parent_kaki` (done in Rust, since HeptaScript's WHERE clause
/// has no KakiPk-literal comparison at all -- `heptascript::engine::
/// compare_val` has no `AkkValue::KakiPk` arm) is what actually finds
/// every section belonging to this specific playbook, regardless of how
/// many W5H2 headers it has. `meta.section_order` puts them back in the
/// document's own original order.
fn playbook_why_text(read_node: &mut enkiddb::ReadNode, parent_kaki: enkidb_kaki::IdentityKaki) -> Option<String> {
    let parent_bytes = *parent_kaki.bytes();
    let query = "WHO T.E\nWHAT E[link.target, body.text, meta.section_order]\nWHERE E[link.description] = \"section-of\"";
    let r = read_node.query(query).ok()?;

    let mut sections: Vec<(i64, String)> = Vec::new();
    for m in &r.matched {
        let mut target_bytes: Option<[u8; 16]> = None;
        let mut text: Option<String> = None;
        let mut order = 0i64;
        for (k, v) in &m.projected {
            match (k.as_str(), v) {
                ("link.target", akkvalue::AkkValue::KakiPk(b)) => target_bytes = Some(*b),
                ("body.text", akkvalue::AkkValue::Text(t)) => text = Some(t.clone()),
                ("meta.section_order", akkvalue::AkkValue::Int(i)) => order = *i,
                _ => {}
            }
        }
        if target_bytes == Some(parent_bytes) {
            if let Some(t) = text {
                sections.push((order, t));
            }
        }
    }
    sections.sort_by_key(|(order, _)| *order);

    let combined: String = sections
        .into_iter()
        .map(|(_, t)| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    // Strip the catalog's own bookkeeping line -- an Architect asking
    // "why does this exist" wants the playbook's own stated reason, not
    // pb_catalog's internal source_path/sha256 tracking noise mixed in.
    let cleaned: String = combined
        .lines()
        .filter(|l| !l.trim_start().starts_with("[playbook-catalog]"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

/// Every OTHER playbook title reachable from `title` through a real
/// `depends-on` edge (`WriteNode::mint_link_edge`, minted by
/// `pb_dependency_review::apply_dependency_approvals`) -- `match_attr`/
/// `want_attr` swap to walk the edge forward (depends-on) or backward
/// (depended-on-by) without two copies of this query.
fn linked_titles(read_node: &mut enkiddb::ReadNode, match_attr: &str, title: &str, want_attr: &str) -> Vec<String> {
    let escaped = title.replace('"', "\\\"");
    let query = format!(
        "WHO T.E\nWHAT E[{want_attr}]\nWHERE E[{match_attr}] = \"{escaped}\" AND E[link.description] = \"depends-on\""
    );
    match read_node.query(&query) {
        Ok(r) => r
            .matched
            .iter()
            .filter_map(|m| {
                m.projected.iter().find(|(k, _)| k == want_attr).and_then(|(_, v)| match v {
                    akkvalue::AkkValue::Text(s) => Some(s.clone()),
                    _ => None,
                })
            })
            .collect(),
        Err(_) => vec![],
    }
}

fn dependency_registry_path() -> PathBuf {
    std::env::var("PB_DEPENDENCY_REGISTRY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join("Forge/pb_dependency_registry.jsonl"))
}

fn scan_dependency_suggestions_resp() -> Resp {
    let suggestions =
        pb_dependency_review::scan_dependency_suggestions(&pb_catalog_registry_path(), &dependency_registry_path());
    json_ok(json!({ "suggestions": suggestions }))
}

fn approve_dependencies(body: &[u8]) -> Resp {
    let approvals_req: Vec<Value> = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => {
            return text(400, "malformed request body -- expected a JSON array of {source_kaki_hex, target_kaki_hex}")
        }
    };
    if approvals_req.is_empty() {
        return text(400, "no approvals in request body");
    }

    let mut approvals = Vec::with_capacity(approvals_req.len());
    for item in &approvals_req {
        let Some(source_hex) = item.get("source_kaki_hex").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs source_kaki_hex");
        };
        let Some(target_hex) = item.get("target_kaki_hex").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs target_kaki_hex");
        };
        let Some(source_title) = item.get("source_title").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs source_title");
        };
        let Some(target_title) = item.get("target_title").and_then(|v| v.as_str()) else {
            return text(400, "each approval needs target_title");
        };
        let Some(source_kaki) = gate_review::parse_kaki_hex(source_hex) else {
            return text(400, format!("malformed source_kaki_hex '{source_hex}'"));
        };
        let Some(target_kaki) = gate_review::parse_kaki_hex(target_hex) else {
            return text(400, format!("malformed target_kaki_hex '{target_hex}'"));
        };
        approvals.push((source_kaki, source_title.to_string(), target_kaki, target_title.to_string()));
    }

    let (mut write_node, epoch, _by_kaki_hex) = rebuild_full_write_node();
    let log =
        pb_dependency_review::apply_dependency_approvals(&mut write_node, &approvals, &dependency_registry_path(), epoch);

    materialize_and_promote(&write_node, log, serde_json::Map::new)
}

/// Free-text search across every catalogued playbook -- title or source
/// path, case-insensitive substring -- answering the real gap a search-
/// less gate/domain browser has: "what if I don't know which of the 7
/// gates something is even in yet" (the common case right after a fresh
/// catalog run, before anything has been classified at all). Reads
/// straight from `pb_catalog`'s own registry (`list_cataloged_playbooks`)
/// rather than an EnkiDDB query -- HeptaScript's WHERE clause has no
/// substring/"contains" operator (`heptascript::engine::Op` is only
/// Eq/Neq/Gt/Lt/Gte/Lte), so an exact-match query could never serve this
/// anyway. Cross-references the gate/domain approval registries so a
/// result shows its current classification (or `null` if still
/// unclassified) without a second round trip.
fn search_playbooks_resp(query_str: &str) -> Resp {
    let q = query_str.trim().to_lowercase();
    if q.is_empty() {
        return json_ok(json!({ "results": [], "total_scanned": 0 }));
    }

    let all = shakkanakku::pb_catalog::list_cataloged_playbooks(&pb_catalog_registry_path());
    let gates = gate_review::approved_gates(&gate_registry_path());
    let domains = domain_review::approved_domains(&domain_registry_path());

    let results: Vec<Value> = all
        .iter()
        .filter_map(|p| {
            // Checked in this order so a title/path hit is reported over
            // a content hit even when both are true -- "matched in the
            // body text" is the more useful thing to tell an Architect
            // when the title itself DIDN'T already make it obvious.
            let match_kind = if p.title.to_lowercase().contains(&q) {
                "title"
            } else if p.source_path.to_lowercase().contains(&q) {
                "path"
            } else if p.full_text.to_lowercase().contains(&q) {
                "content"
            } else {
                return None;
            };
            Some(json!({
                "title": p.title,
                "kaki_hex": p.kaki_hex,
                "source_path": p.source_path,
                "match_kind": match_kind,
                "gate": gates.get(&p.kaki_hex),
                "domain": domains.get(&p.kaki_hex),
            }))
        })
        .take(50)
        .collect();

    json_ok(json!({ "results": results, "total_scanned": all.len() }))
}

/// What a route requires. Enforced centrally in `route`, not left to each
/// handler to remember — a handler simply never runs unless the caller
/// already cleared this bar.
enum AuthReq {
    Public,
    /// Any valid, unexpired passport session — viewing the run, exporting
    /// the already-sealed report.
    AnyPassport,
    /// Only a privilege_level 7 (Gilgamesh-minted) passport session —
    /// everything that starts, stops, or steers real execution. This is
    /// what CSR-08's "awaiting the Architect" now means literally, not as
    /// a UI label.
    Architect,
}

fn auth_requirement(method: &str, path: &str) -> AuthReq {
    match (method, path) {
        ("GET", "/") | ("GET", "/app.css") | ("GET", "/app.js") => AuthReq::Public,
        ("GET", "/login") | ("GET", "/login.js") => AuthReq::Public,
        // Static assets only -- same tier as app.css/app.js. The real gate
        // is on the /api/gates/* data underneath them, not on the script
        // files themselves.
        ("GET", "/gate_orbits.js") | ("GET", "/vendor/three.module.js") | ("GET", "/vendor/OrbitControls.js") => AuthReq::Public,
        ("POST", "/api/login") => AuthReq::Public,
        ("GET", "/api/state") | ("GET", "/api/session") | ("GET", "/api/report/export") => AuthReq::AnyPassport,
        ("POST", "/api/logout") => AuthReq::AnyPassport,
        // FIXED 2026-08-02 (Architect's own instruction): Tab 1 (Box
        // Resources) is now Public -- ANY stakeholder, signed in or not,
        // can see host resource utilization, no passport required. This
        // was the Architect's own explicit access-model decision: Shala's
        // first tab is meant to be freely visible; only Tabs 2/3
        // (Shakkanakku, Hala) -- and every action anywhere -- stay
        // behind a passport. Tab 3's status poll stays AnyPassport (unlike
        // Tab 1) since viewing Hala's own run state is part of the
        // gated experience, not the free one.
        ("GET", "/api/resource_check") => AuthReq::Public,
        ("GET", "/api/docpulse/state") => AuthReq::AnyPassport,
        // Shala4 · Gate Orbits: FIXED 2026-08-02 (Architect's own
        // instruction, same access-model reasoning as Tab 1's own
        // resource-check exception above) -- viewing the playbook catalog
        // by gate/domain is free to everyone, no passport required, same
        // as Box Resources. Playbook browsing isn't sensitive
        // infrastructure state; only a real governance action (suggesting
        // or approving a gate/domain classification) needs a passport at
        // all, and Architect-privilege specifically, via the default
        // `/api/` rule below.
        ("GET", "/api/gates/summary") => AuthReq::Public,
        ("GET", p) if p.starts_with("/api/gates/") && p.ends_with("/playbooks") => AuthReq::Public,
        ("GET", p) if p.starts_with("/api/gates/") && p.ends_with("/domains/summary") => AuthReq::Public,
        ("GET", "/api/playbook_detail") => AuthReq::Public,
        ("GET", "/api/playbooks/search") => AuthReq::Public,
        _ if path.starts_with("/api/") => AuthReq::Architect,
        _ => AuthReq::Public, // unmatched paths 404 downstream regardless
    }
}

fn route(method: &str, path: &str, query: &str, body: &[u8], peer_ip: IpAddr, cookie: Option<String>, ctx: &AppCtx) -> Resp {
    let identity = cookie.as_deref().and_then(|c| ctx.sessions.verify(c));

    match auth_requirement(method, path) {
        AuthReq::Public => {}
        AuthReq::AnyPassport => {
            if identity.is_none() {
                return unauthorized();
            }
        }
        AuthReq::Architect => match &identity {
            Some(id) if id.is_architect() => {}
            Some(_) => return forbidden("this action needs an Architect-privilege (Gilgamesh-minted) passport, not a gardener one"),
            None => return unauthorized(),
        },
    }

    match (method, path) {
        ("GET", "/") => (200, "text/html; charset=utf-8", vec![], INDEX_HTML.as_bytes().to_vec()),
        ("GET", "/app.css") => (200, "text/css; charset=utf-8", vec![], APP_CSS.as_bytes().to_vec()),
        ("GET", "/app.js") => (200, "application/javascript; charset=utf-8", vec![], APP_JS.as_bytes().to_vec()),
        ("GET", "/login") => (200, "text/html; charset=utf-8", vec![], LOGIN_HTML.as_bytes().to_vec()),
        ("GET", "/login.js") => (200, "application/javascript; charset=utf-8", vec![], LOGIN_JS.as_bytes().to_vec()),
        ("GET", "/gate_orbits.js") => (200, "application/javascript; charset=utf-8", vec![], GATE_ORBITS_JS.as_bytes().to_vec()),
        ("GET", "/vendor/three.module.js") => (200, "application/javascript; charset=utf-8", vec![], VENDOR_THREE_JS.as_bytes().to_vec()),
        ("GET", "/vendor/OrbitControls.js") => {
            (200, "application/javascript; charset=utf-8", vec![], VENDOR_ORBIT_CONTROLS_JS.as_bytes().to_vec())
        }

        ("POST", "/api/login") => {
            if let Err(remaining) = ctx.defender.check(peer_ip) {
                return too_many_requests(remaining);
            }
            let req: Value = match serde_json::from_slice(body) {
                Ok(v) => v,
                Err(_) => return text(400, "malformed request body"),
            };
            let vault_b64 = req.get("vault_b64").and_then(|v| v.as_str()).unwrap_or("");
            let vault_path = req.get("vault_path").and_then(|v| v.as_str()).unwrap_or("").trim();
            let passphrase = req.get("passphrase").and_then(|v| v.as_str()).unwrap_or("");
            // FIXED 2026-08-02: the browser's native file picker can fail to
            // open at all on some desktops (xdg-desktop-portal/Flatpak
            // sandboxing issues, confirmed live on the Architect's own
            // Fedora Workstation 44 host -- Tab-focus worked, Space/Enter
            // did nothing, no dialog ever appeared). Since Shala only ever
            // serves 127.0.0.1 and already runs with the invoking user's
            // own filesystem permissions, letting the SERVER read a local
            // path directly is not a new privilege boundary -- it's the
            // same trust level this process already has. `vault_path` is
            // tried first when present and non-empty; `vault_b64` (the
            // original browser-upload path) still works unchanged for
            // anyone whose file picker works fine.
            let vault_bytes = if !vault_path.is_empty() {
                match std::fs::read(vault_path) {
                    Ok(b) => b,
                    Err(e) => {
                        ctx.defender.record_failure(peer_ip);
                        return text(400, format!("could not read vault_path '{vault_path}': {e}"));
                    }
                }
            } else {
                match B64.decode(vault_b64) {
                    Ok(b) => b,
                    Err(_) => {
                        ctx.defender.record_failure(peer_ip);
                        return text(400, "vault file was not valid base64 — did the browser upload finish?");
                    }
                }
            };
            match web_auth::open_vault_and_authenticate(&vault_bytes, passphrase.as_bytes()) {
                Ok(id) => {
                    ctx.defender.record_success(peer_ip);
                    let summary = identity_json(&id);
                    let cookie_val = ctx.sessions.mint(id);
                    (200, "application/json; charset=utf-8", vec![session_cookie_header(&cookie_val)], serde_json::to_vec(&summary).unwrap())
                }
                Err(e) => {
                    ctx.defender.record_failure(peer_ip);
                    text(401, e.to_string())
                }
            }
        }
        ("POST", "/api/logout") => {
            if let Some(c) = &cookie {
                ctx.sessions.revoke(c);
            }
            (200, "application/json; charset=utf-8", vec![clear_cookie_header()], b"{}".to_vec())
        }
        ("GET", "/api/session") => json_ok(identity_json(&identity.expect("AuthReq::AnyPassport already checked"))),

        ("GET", "/api/report/export") => {
            let format = query_param(query, "format").unwrap_or_else(|| "md".to_string());
            export_report(&format, &ctx.state)
        }
        ("GET", "/api/state") => {
            let mut st = ctx.state.lock().unwrap();
            st.pump_events();
            json_ok(st.to_json())
        }
        ("POST", "/api/load") => {
            let req: Value = serde_json::from_slice(body).unwrap_or(json!({}));
            let mut st = ctx.state.lock().unwrap();
            if let Some(p) = req.get("cfg_path").and_then(|v| v.as_str()) {
                st.cfg_path = p.to_string();
            }
            st.load_config();
            json_ok(st.to_json())
        }
        ("POST", "/api/params/add") => {
            let req: Value = serde_json::from_slice(body).unwrap_or(json!({}));
            let mut st = ctx.state.lock().unwrap();
            let key = req.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let value = req.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if !key.is_empty() {
                st.params.push((key, value));
            }
            json_ok(st.to_json())
        }
        ("POST", "/api/params/remove") => {
            let req: Value = serde_json::from_slice(body).unwrap_or(json!({}));
            let mut st = ctx.state.lock().unwrap();
            if let Some(i) = req.get("index").and_then(|v| v.as_u64()) {
                let i = i as usize;
                if i < st.params.len() {
                    st.params.remove(i);
                }
            }
            json_ok(st.to_json())
        }
        ("POST", "/api/run") => {
            let mut st = ctx.state.lock().unwrap();
            if let Err(reason) = st.resource_check_permits_start() {
                return forbidden(&format!("Tab 1 resource check does not permit starting a run: {reason}"));
            }
            if st.cfg.is_some() && matches!(st.phase, RunPhase::Idle | RunPhase::Finished | RunPhase::Aborted) {
                st.start_run();
            }
            json_ok(st.to_json())
        }
        ("POST", "/api/abort") => {
            let mut st = ctx.state.lock().unwrap();
            if let Some(t) = &st.ctl_tx {
                let _ = t.send(Ctl::Abort);
            }
            st.pump_events();
            json_ok(st.to_json())
        }
        ("POST", "/api/skip") => {
            let mut st = ctx.state.lock().unwrap();
            if let RunPhase::Halted(i) = st.phase {
                if let Some(s) = st.statuses.get_mut(i) {
                    *s = PbStatus::Skipped;
                }
                st.phase = RunPhase::Running;
                if let Some(t) = &st.ctl_tx {
                    let _ = t.send(Ctl::SkipContinue);
                }
            }
            json_ok(st.to_json())
        }
        ("POST", "/api/retry") => {
            let mut st = ctx.state.lock().unwrap();
            if let (Some(cfg), Some(sel)) = (st.cfg.clone(), st.halt_event) {
                if let Some(ev) = st.events.get(sel).cloned() {
                    match runner::generate_fix(&cfg, &ev) {
                        Ok(p) => {
                            st.log.push(format!("𒁾 generated fix playbook: {}", p.display()));
                            st.fixes.push(p);
                            if let Some(t) = &st.ctl_tx {
                                let _ = t.send(Ctl::Retry);
                            }
                        }
                        Err(e) => st.log.push(format!("remedy error: {e}")),
                    }
                }
            }
            json_ok(st.to_json())
        }
        ("POST", "/api/generate_fix") => {
            let req: Value = serde_json::from_slice(body).unwrap_or(json!({}));
            let mut st = ctx.state.lock().unwrap();
            if let Some(pb_index) = req.get("pb_index").and_then(|v| v.as_u64()) {
                let pb_index = pb_index as usize;
                if let (Some(cfg), Some(ev)) =
                    (st.cfg.clone(), st.events.iter().find(|e| e.pb_index == pb_index).cloned())
                {
                    match runner::generate_fix(&cfg, &ev) {
                        Ok(p) => {
                            st.log.push(format!("𒁾 fix written: {}", p.display()));
                            st.fixes.push(p);
                        }
                        Err(e) => st.log.push(format!("remedy error: {e}")),
                    }
                }
            }
            json_ok(st.to_json())
        }
        ("POST", "/api/report") => {
            let mut st = ctx.state.lock().unwrap();
            if matches!(st.phase, RunPhase::Finished | RunPhase::Aborted) {
                st.generate_report();
            }
            json_ok(st.to_json())
        }

        // ── Tab 1: Resource Check ────────────────────────────────────
        ("GET", "/api/resource_check") => {
            let mut st = ctx.state.lock().unwrap();
            st.run_resource_check();
            json_ok(st.resource_check_json())
        }

        // ── Tab 3: Hala ────────────────────────────────────────────────
        ("GET", "/api/docpulse/state") => {
            let mut st = ctx.state.lock().unwrap();
            st.pump_docpulse_events();
            json_ok(st.docpulse_json())
        }
        ("POST", "/api/docpulse/start") => {
            let mut st = ctx.state.lock().unwrap();
            if let Err(reason) = st.resource_check_permits_start() {
                return forbidden(&format!("Tab 1 resource check does not permit starting a pulse: {reason}"));
            }
            if !matches!(st.dp_phase, RunPhase::Idle | RunPhase::Finished | RunPhase::Aborted) {
                return forbidden("a pulse is already running or halted -- abort it first");
            }
            let req: Value = match serde_json::from_slice(body) {
                Ok(v) => v,
                Err(_) => return text(400, "malformed request body"),
            };
            let s = |k: &str| req.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let docs_tribe = req.get("docs_tribe").and_then(|v| v.as_str()).unwrap_or("docs");
            let docs_tribe_id = match docs_tribe {
                "docs" => enkiddb::DOCS_TRIBE_ID,
                "architect_docs" => enkiddb::ARCHITECT_DOCS_TRIBE_ID,
                other => return text(400, &format!("unknown docs_tribe '{other}' -- expected \"docs\" or \"architect_docs\"")),
            };
            let cfg = DocPulseCfg {
                repo_path: s("repo"),
                message: s("message"),
                limit_mb: req.get("limit_mb").and_then(|v| v.as_u64()).unwrap_or(90),
                archive_dir: s("archive"),
                ingest_manifest_dir: s("ingest_manifest_dir"),
                chronicle_dir: {
                    let c = s("chronicle_dir");
                    if c.is_empty() { "hala_chronicle".into() } else { c }
                },
                enkiddb_output_root: s("enkiddb_root"),
                official_repo_path: s("official_repo"),
                official_repo_subdir: s("official_repo_subdir"),
                official_repo_branch: s("official_repo_branch"),
                auto_push_to_official_repo: req.get("push_to_official_repo").and_then(|v| v.as_bool()).unwrap_or(false),
                docs_tribe_id,
            };
            if cfg.repo_path.is_empty() || cfg.archive_dir.is_empty() || cfg.enkiddb_output_root.is_empty() {
                return text(400, "repo, archive, and enkiddb_root are all required");
            }
            st.start_docpulse(cfg);
            json_ok(st.docpulse_json())
        }
        ("POST", "/api/docpulse/abort") => {
            let mut st = ctx.state.lock().unwrap();
            if let Some(t) = &st.dp_ctl_tx {
                let _ = t.send(Ctl::Abort);
            }
            st.pump_docpulse_events();
            json_ok(st.docpulse_json())
        }
        ("POST", "/api/docpulse/retry") => {
            let st = ctx.state.lock().unwrap();
            if let RunPhase::Halted(_) = st.dp_phase {
                if let Some(t) = &st.dp_ctl_tx {
                    let _ = t.send(Ctl::Retry);
                }
            }
            json_ok(st.docpulse_json())
        }
        ("POST", "/api/docpulse/skip") => {
            let st = ctx.state.lock().unwrap();
            if let RunPhase::Halted(_) = st.dp_phase {
                if let Some(t) = &st.dp_ctl_tx {
                    let _ = t.send(Ctl::SkipContinue);
                }
            }
            json_ok(st.docpulse_json())
        }

        // ── Shala4: Gate Orbits ──────────────────────────────────────
        ("GET", "/api/gates/summary") => json_ok(gates_summary_json()),

        // Checked BEFORE the general "/api/gates/<gate>/playbooks" arm below
        // (which would otherwise also match this same "...ends_with
        // /playbooks" shape) -- this one specifically excludes it via
        // `!p.contains("/domains/")` so match order can't silently pick
        // the wrong arm as new gate/domains/* routes are added.
        ("GET", p) if p.starts_with("/api/gates/") && p.ends_with("/playbooks") && !p.contains("/domains/") => {
            let name = p.trim_start_matches("/api/gates/").trim_end_matches("/playbooks");
            gate_playbooks_resp(&url_decode(name))
        }

        ("POST", "/api/gates/scan_suggestions") => {
            let suggestions = gate_review::scan_gate_suggestions(&pb_catalog_registry_path(), &gate_registry_path());
            json_ok(json!({ "suggestions": suggestions }))
        }

        // ── Shala4: domains within a gate ─────────────────────────────
        ("GET", p) if p.starts_with("/api/gates/") && p.ends_with("/domains/summary") => {
            let gate = p.trim_start_matches("/api/gates/").trim_end_matches("/domains/summary");
            gate_domains_summary_resp(&url_decode(gate))
        }
        ("GET", p) if p.starts_with("/api/gates/") && p.contains("/domains/") && p.ends_with("/playbooks") => {
            let rest = p.trim_start_matches("/api/gates/").trim_end_matches("/playbooks");
            match rest.split_once("/domains/") {
                Some((gate, domain)) => domain_playbooks_resp(&url_decode(gate), &url_decode(domain)),
                None => text(404, "not found"),
            }
        }
        ("POST", p) if p.starts_with("/api/gates/") && p.ends_with("/domains/scan_suggestions") => {
            let gate = p.trim_start_matches("/api/gates/").trim_end_matches("/domains/scan_suggestions");
            scan_domain_suggestions_resp(&url_decode(gate))
        }
        ("POST", p) if p.starts_with("/api/gates/") && p.ends_with("/domains/approve") => {
            let gate = p.trim_start_matches("/api/gates/").trim_end_matches("/domains/approve").to_string();
            approve_domains(&url_decode(&gate), body)
        }

        ("GET", "/api/playbook_detail") => playbook_detail_resp(&query_param(query, "title").unwrap_or_default()),
        ("GET", "/api/playbooks/search") => search_playbooks_resp(&query_param(query, "q").unwrap_or_default()),
        ("POST", "/api/dependencies/scan_suggestions") => scan_dependency_suggestions_resp(),
        ("POST", "/api/dependencies/approve") => approve_dependencies(body),
        ("POST", "/api/refresh_current") => refresh_current_resp(),

        ("POST", "/api/gates/approve") => approve_gates(body),

        _ => text(404, "not found"),
    }
}
