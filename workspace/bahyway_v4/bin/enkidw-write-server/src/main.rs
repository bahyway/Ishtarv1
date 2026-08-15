//! enkidw-write-server — EnkiDW (Data Warehouse) Write Node, sovereign TCP
//! daemon.
//!
//! Wraps `enkidw::EtlPipeline` (the real LandingZone -> ArchiveEngine ->
//! RecordRouter -> KakiGenerator -> `enkidb_persist::PersistedDb` chain
//! already used by `enkidw-cli`'s `ingest`/`watch` commands): a background
//! thread polls the landing zone every `POLL_SECS` (default 5) and commits
//! every parsed record through `PersistedDb::commit` -- disk-first, crash-
//! safe, replayed on reopen (`enkidb_persist::disk_journal`, its own
//! binary format, never JSON).
//!
//! `PersistedDb`'s own on-disk journal (under `PERSIST_DIR`) is EnkiDW's
//! durability boundary. Separately, on the same cadence as every other
//! *-write-server` in this fleet, this daemon re-materializes
//! `pdb.db().journal()` (a real `enkidb_journal::Journal` -- `EnkiDb::
//! journal()` exposes it directly, so the exact same `enkidb_readnode::
//! materialize` + `CachedReadNode` path every other EnkiDB Type uses
//! applies here too) into `DATA_DIR/current/{entities,eav}` for
//! `enkidw-read-server` to serve HeptaScript queries from, zero disk I/O
//! per query, no JSON on the wire.
//!
//! ## Control protocol
//! Plain length-prefixed text frames (same convention as every other
//! *-write-server` in this workspace):
//!   - `STATS` -> `OK:STATS:files_seen=<n>:records_ingested=<n>:records_skipped=<n>:total_particles=<n>`
//!   - `FLUSH` -> force-materialize now, responds `OK:FLUSHED:<entity_count>`
//!   - anything else -> `ERR:<message>`
#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use bahyway_core::TribeId;
use enkidb_storage::FsyncPolicy;
use enkidw::EtlPipeline;

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 120;

fn bind_addr() -> String {
    env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7012".to_string())
}
fn landing_dir() -> PathBuf {
    PathBuf::from(env::var("LANDING_DIR").unwrap_or_else(|_| "/data/dw/landing".to_string()))
}
fn persist_dir() -> PathBuf {
    PathBuf::from(env::var("PERSIST_DIR").unwrap_or_else(|_| "/data/dw/persist".to_string()))
}
fn data_dir() -> PathBuf {
    PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "/data/dw".to_string()))
}
fn tribe_id() -> u16 {
    env::var("TRIBE_ID")
        .ok()
        .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x7600)
}
fn poll_secs() -> u64 {
    env::var("POLL_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(5)
}
fn materialize_every_secs() -> u64 {
    env::var("MATERIALIZE_EVERY_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30)
}

struct SharedState {
    pipeline: EtlPipeline,
}

fn main() {
    eprintln!("𒁾 enkidw-write-server — EnkiDW (Data Warehouse) Write Node");

    let tribe = TribeId::from_u16(tribe_id());
    let landing = landing_dir();
    let persist = persist_dir();
    let data_dir = data_dir();
    let poll_secs = poll_secs();
    let materialize_every_secs = materialize_every_secs();

    let pipeline = EtlPipeline::open(&landing, &persist, tribe, FsyncPolicy::PerCommit).unwrap_or_else(|e| {
        eprintln!("FATAL: cannot open EtlPipeline (landing={}, persist={}): {e}", landing.display(), persist.display());
        std::process::exit(1);
    });

    let state = Mutex::new(SharedState { pipeline });

    eprintln!("  landing_dir = {}", landing.display());
    eprintln!("  persist_dir = {}", persist.display());
    eprintln!("  data_dir = {}", data_dir.display());
    eprintln!("  tribe = {:#06x}  poll_secs = {poll_secs}", tribe.as_u16());

    std::thread::scope(|scope| {
        // ── Background daemon: poll landing zone, materialize on cadence ──
        {
            let state = &state;
            let data_dir = data_dir.clone();
            scope.spawn(move || {
                let mut last_materialize = std::time::Instant::now();
                let mut last_ingested = 0usize;
                loop {
                    std::thread::sleep(Duration::from_secs(poll_secs));
                    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());

                    // EtlStats is lifetime-cumulative (see EtlPipeline::
                    // run_once -- it only ever `+=`s, never resets), so
                    // STATS below reads it straight off the pipeline
                    // rather than double-accumulating into a second
                    // counter here; this log line only prints on growth.
                    let ingest_stats = st.pipeline.run_once().clone();
                    for alert in st.pipeline.take_alerts() {
                        eprintln!("[dw-pipeline] {}", alert.message);
                    }
                    if ingest_stats.records_ingested > last_ingested {
                        eprintln!("[dw-pipeline] ingested={} (+{})", ingest_stats.records_ingested, ingest_stats.records_ingested - last_ingested);
                        last_ingested = ingest_stats.records_ingested;
                    }

                    if last_materialize.elapsed() >= Duration::from_secs(materialize_every_secs) {
                        match materialize_fresh(&st, &data_dir) {
                            Ok(stats) => eprintln!("[materialize] entities={}", stats.entities),
                            Err(e) => eprintln!("[materialize] error: {e}"),
                        }
                        last_materialize = std::time::Instant::now();
                    }
                }
            });
        }

        // ── TCP control listener ──
        let addr = bind_addr();
        let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
            eprintln!("FATAL: bind {addr}: {e}");
            std::process::exit(1);
        });
        eprintln!("𒁾 Listening on {addr}");

        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let peer = s.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".into());
                    let state_ref = &state;
                    let data_dir = data_dir.clone();
                    scope.spawn(move || {
                        if let Err(e) = handle(s, state_ref, &data_dir) {
                            eprintln!("[{peer}] {e}");
                        }
                    });
                }
                Err(e) => eprintln!("[accept] {e}"),
            }
        }
    });
}

fn handle(mut stream: TcpStream, state: &Mutex<SharedState>, data_dir: &Path) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))?;
    stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT)))?;

    let src = read_frame(&mut stream)?;
    match src.trim() {
        "STATS" => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            let s = st.pipeline.stats();
            let total_particles = st.pipeline.pdb().db().journal().all_particles().len();
            send(&mut stream, &format!(
                "OK:STATS:files_seen={}:records_ingested={}:records_skipped={}:total_particles={}",
                s.files_seen, s.records_ingested, s.records_skipped, total_particles,
            ))
        }
        "FLUSH" => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            match materialize_fresh(&st, data_dir) {
                Ok(stats) => send(&mut stream, &format!("OK:FLUSHED:{}", stats.entities)),
                Err(e) => send(&mut stream, &format!("ERR:materialize: {e}")),
            }
        }
        _ => send(&mut stream, "ERR:unrecognized request -- use STATS or FLUSH"),
    }
}

fn materialize_fresh(st: &SharedState, data_dir: &Path) -> io::Result<enkidb_readnode::MaterializeStats> {
    let current = data_dir.join("current");
    let _ = fs::remove_dir_all(&current);
    fs::create_dir_all(&current)?;
    enkidb_readnode::materialize(st.pipeline.pdb().db().journal(), current.join("entities"), current.join("eav"))
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
