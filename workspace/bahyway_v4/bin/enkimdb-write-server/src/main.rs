//! enkimdb-write-server — EnkiMDB (Euphrates) Write Node, sovereign TCP server.
//!
//! Wraps `enkimdb::WriteNode` (real `enkidb_journal::Journal` WAL) behind
//! the exact same wire protocol `enkiddb-write-server`/`enkidb-query-server`
//! already use (length-prefixed u32 LE frame, one thread per connection, no
//! tokio, no HTTP, no external wire-format crate).
//!
//! Unlike EnkiDDB (which ingests one markdown document per request), this
//! server catalogs artifacts already sitting on a mounted volume: point it
//! at a checked-out copy of the workspace and it scans + journals every
//! crate/playbook found there in one call, via the real
//! `enkimdb::scan_crates`/`scan_playbooks` + `WriteNode::ingest_artifact`
//! pipeline — no artifact is invented or hand-typed by this server.
//!
//! ## Protocol
//! One request frame in, one response frame out, per connection:
//!   - `SCAN_CRATES:<workspace_root>`   -> scan `<root>/crates/*/Cargo.toml`,
//!                                         ingest each, respond
//!                                         `OK:INGESTED:<count>`
//!   - `SCAN_PLAYBOOKS:<repo_root>`     -> scan `<root>/playbooks/*.yml`,
//!                                         ingest each, respond
//!                                         `OK:INGESTED:<count>`
//!   - `FLUSH`                          -> force-materialize now, respond
//!                                         `OK:FLUSHED:<entity_count>`
//!   - `INGEST_RUN_RECORD:<json>`       -> parse `<json>` as a real
//!                                         `enkimdb::AnuGovernorRunRecordSpec`,
//!                                         ingest it (role=Zikru,
//!                                         `anu_governor_run.*` namespace),
//!                                         respond `OK:INGESTED:1`
//!                                         (2026-07-29, the run-confirmation
//!                                         registry -- see that type's own
//!                                         doc comment for why)
//!   - anything else malformed         -> `ERR:<message>`
//!
//! `<workspace_root>`/`<repo_root>` are paths inside this container — the
//! Architect's Podman run command must bind-mount the real checkout (e.g.
//! the repo root at `/source:ro,Z`) for `SCAN_CRATES:/source/workspace/
//! bahyway_v4` and `SCAN_PLAYBOOKS:/source` to find anything.
//!
//! ## Durability contract (v1, stated plainly)
//! Same as `enkiddb-write-server`: the Journal is in-memory only. Every
//! `FLUSH` (or the process exiting normally after one) is a full
//! wipe-and-rebuild materialize into `DATA_DIR/current/{entities,eav}` —
//! entries ingested since the last flush are lost on an unclean restart.
//! `enkidb-replication` is the real durability fix, not wired in here.
#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use enkimdb::{readnode, scan_crates, scan_playbooks, AnuGovernorRunRecordSpec, WriteNode};

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 120;

fn bind_addr() -> String {
    // read=7006 (canonical, see enkimdb-read-server), write=read+10 --
    // the same convention every other EnkiDB type's write server uses
    // (enkidb-write-server=7011, enkidw-write-server=7012, etc).
    // Corrected 2026-07-31 from the stray 7201 default.
    env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7016".to_string())
}
fn data_dir() -> PathBuf {
    PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "/data".to_string()))
}
fn tribe_id() -> u16 {
    env::var("TRIBE_ID")
        .ok()
        .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(enkimdb::ARTIFACT_TRIBE_ID)
}
fn flush_every() -> u32 {
    env::var("FLUSH_EVERY").ok().and_then(|s| s.parse().ok()).unwrap_or(10)
}

struct SharedState {
    write_node: WriteNode,
    epoch: u32,
    since_flush: u32,
}

fn main() {
    eprintln!("𒁾 enkimdb-write-server — EnkiMDB (Euphrates) Write Node");

    let minter = KakiMinter::new(TribeId::from_u16(tribe_id()));
    let state = Mutex::new(SharedState {
        write_node: WriteNode::new(minter, 64),
        epoch: 0,
        since_flush: 0,
    });
    let flush_every = flush_every();
    let data_dir = data_dir();
    let request_count = AtomicU32::new(0);

    let addr = bind_addr();
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("FATAL: bind {addr}: {e}");
        std::process::exit(1);
    });
    eprintln!("  data_dir    = {}", data_dir.display());
    eprintln!("  flush_every = {flush_every} ingest calls");
    eprintln!("𒁾 Listening on {addr}");

    std::thread::scope(|scope| {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let peer = s.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".into());
                    request_count.fetch_add(1, Ordering::Relaxed);
                    let state_ref = &state;
                    let data_dir_ref = &data_dir;
                    scope.spawn(move || {
                        if let Err(e) = handle(s, state_ref, data_dir_ref, flush_every) {
                            eprintln!("[{peer}] {e}");
                        }
                    });
                }
                Err(e) => eprintln!("[accept] {e}"),
            }
        }
    });
}

fn handle(mut stream: TcpStream, state: &Mutex<SharedState>, data_dir: &Path, flush_every: u32) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))?;
    stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT)))?;

    let src = read_frame(&mut stream)?;
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return send(&mut stream, "ERR:empty request");
    }

    if trimmed == "FLUSH" {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        return match materialize_fresh(&st.write_node, data_dir) {
            Ok(stats) => {
                st.since_flush = 0;
                send(&mut stream, &format!("OK:FLUSHED:{}", stats.entities))
            }
            Err(e) => send(&mut stream, &format!("ERR:materialize: {e}")),
        };
    }

    if let Some(root) = trimmed.strip_prefix("SCAN_CRATES:") {
        return match scan_crates(Path::new(root)) {
            Ok(profiles) => {
                let count = ingest_all(state, data_dir, flush_every, &profiles);
                send(&mut stream, &format!("OK:INGESTED:{count}"))
            }
            Err(e) => send(&mut stream, &format!("ERR:scan_crates: {e}")),
        };
    }

    if let Some(root) = trimmed.strip_prefix("SCAN_PLAYBOOKS:") {
        return match scan_playbooks(Path::new(root)) {
            Ok(profiles) => {
                let count = ingest_all(state, data_dir, flush_every, &profiles);
                send(&mut stream, &format!("OK:INGESTED:{count}"))
            }
            Err(e) => send(&mut stream, &format!("ERR:scan_playbooks: {e}")),
        };
    }

    if let Some(json) = trimmed.strip_prefix("INGEST_RUN_RECORD:") {
        return match serde_json::from_str::<AnuGovernorRunRecordSpec>(json) {
            Ok(spec) => {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                st.epoch += 1;
                let epoch = st.epoch;
                let kaki = st.write_node.ingest_anu_governor_run_record(&spec, epoch);
                st.since_flush += 1;
                let should_flush = st.since_flush >= flush_every;
                if should_flush {
                    if materialize_fresh(&st.write_node, data_dir).is_ok() {
                        st.since_flush = 0;
                    }
                }
                let hex: String = kaki.bytes().iter().map(|b| format!("{b:02x}")).collect();
                eprintln!("[ingest] run_record run_id={} outcome={} epoch={epoch} kaki={hex}", spec.run_id, spec.outcome);
                send(&mut stream, "OK:INGESTED:1")
            }
            Err(e) => send(&mut stream, &format!("ERR:malformed run record json: {e}")),
        };
    }

    send(&mut stream, "ERR:unrecognized request -- use SCAN_CRATES:<path>, SCAN_PLAYBOOKS:<path>, FLUSH, or INGEST_RUN_RECORD:<json>")
}

fn ingest_all(
    state: &Mutex<SharedState>,
    data_dir: &Path,
    flush_every: u32,
    profiles: &[enkimdb::ArtifactProfile],
) -> usize {
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    for profile in profiles {
        st.epoch += 1;
        let epoch = st.epoch;
        let kaki = st.write_node.ingest_artifact(profile, epoch);
        st.since_flush += 1;
        let hex: String = kaki.bytes().iter().map(|b| format!("{b:02x}")).collect();
        eprintln!("[ingest] kind={} name={} epoch={epoch} kaki={hex}", profile.kind.as_str(), profile.name);
    }

    let should_flush = st.since_flush >= flush_every;
    if should_flush {
        match materialize_fresh(&st.write_node, data_dir) {
            Ok(_) => {
                st.since_flush = 0;
                eprintln!("[auto-flush] materialized");
            }
            Err(e) => eprintln!("[auto-flush] {e}"),
        }
    }

    profiles.len()
}

fn materialize_fresh(write_node: &WriteNode, data_dir: &Path) -> io::Result<readnode::MaterializeStats> {
    let current = data_dir.join("current");
    let _ = fs::remove_dir_all(&current);
    fs::create_dir_all(&current)?;
    readnode::materialize_now(write_node, current.join("entities"), current.join("eav"))
}

fn send(stream: &mut TcpStream, payload: &str) -> io::Result<()> {
    write_frame(stream, payload)?;
    stream.write_all(&0u32.to_le_bytes())?;
    stream.flush()
}

fn read_frame(s: &mut TcpStream) -> io::Result<String> {
    let mut lb = [0u8; 4];
    s.read_exact(&mut lb)?;
    let len = u32::from_le_bytes(lb);
    if len == 0 {
        return Ok(String::new());
    }
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("frame too large: {len}")));
    }
    let mut buf = vec![0u8; len as usize];
    s.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_frame(s: &mut TcpStream, payload: &str) -> io::Result<()> {
    let b = payload.as_bytes();
    s.write_all(&(b.len() as u32).to_le_bytes())?;
    s.write_all(b)?;
    s.flush()
}
