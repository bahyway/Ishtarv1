//! enkiddb-write-server — EnkiDDB (Tigris) Write Node, sovereign TCP server.
//!
//! Wraps `enkiddb::WriteNode` (real `enkidb_journal::Journal` WAL) behind
//! the exact same wire protocol `enkidb-query-server` already uses
//! (length-prefixed u32 LE frame, one thread per connection, no tokio, no
//! HTTP, no external wire-format crate) so both servers are operable the
//! same way.
//!
//! ## Protocol
//! One request frame in, one response frame out, per connection:
//!   - `FLUSH`                        -> force-materialize now, respond
//!                                       `OK:FLUSHED:<entity_count>`
//!   - `<collection>\n<markdown...>`  -> ingest one document (categorized
//!                                       + chunked into RAG sections, per
//!                                       `WriteNode::ingest_document_categorized`),
//!                                       respond `OK:<kaki_hex>:<section_count>`.
//!                                       Gated first by a pre-parse security
//!                                       scan (`enkiddb::scan_document`,
//!                                       PB-206 — Musarû's byte-signature
//!                                       scanner, the real engine behind
//!                                       the "Nergal" visualization panel)
//!                                       rejecting known malware/webshell/
//!                                       dropper byte patterns with
//!                                       `ERR:security: <detail>` before
//!                                       the body is ever parsed.
//!   - `INGEST_DIR:<path>`            -> bulk-load every `.md` file under
//!                                       `<path>` (categorized + chunked,
//!                                       per `WriteNode::ingest_directory_categorized_checked`),
//!                                       respond `OK:INGESTED:<count>`.
//!                                       Gated by the same git-based
//!                                       team-authorship check
//!                                       `enkiddb-cli` enforces
//!                                       (`enkiddb::authorship`) — a file
//!                                       whose last commit author isn't
//!                                       on the allowlist aborts the
//!                                       whole call, responding
//!                                       `ERR:ingest_dir: <reason>`. The
//!                                       allowlist comes from
//!                                       `ENKIDDB_TEAM_ALLOWLIST` if set,
//!                                       else a built-in seed
//!                                       (`TeamAllowlist::from_env_or_seed`).
//!                                       Each file is also security-scanned
//!                                       (same PB-206 gate as above) inside
//!                                       `WriteNode::ingest_document_from_path`
//!                                       -- a hit aborts the whole call the
//!                                       same way an unauthorized author does.
//!   - anything else malformed        -> `ERR:<message>`
//!
//! ## Durability contract (v1, stated plainly)
//! The Journal itself is in-memory only -- this process does not persist
//! or replay a WAL across restarts (unlike `enkidb-persist::PersistedDb`,
//! which the original `enkidb-query-server` uses). Durability boundary is
//! the periodic materialize: every `FLUSH_EVERY` documents (default 10)
//! or an explicit `FLUSH` request, the current Journal state is
//! re-materialized fresh into `DATA_DIR/current/{entities,eav}` -- a full
//! wipe-and-rebuild each time, not an incremental append, because
//! `enkidb_datafile::DataFileWriter` never truncates (see
//! `enkiddb::readnode::materialize_now`'s own doc comment). Documents
//! ingested since the last flush are lost if the container restarts
//! before the next one. This is a real, known v1 limitation, not hidden:
//! a future iteration should adopt `enkidb-replication`'s KAKI-sealed
//! append-only log instead of full re-materialization for durability.
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
use enkiddb::{readnode, DocumentParser, TeamAllowlist, WriteNode};

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 120;

fn bind_addr() -> String {
    // read=7007 (canonical, see enkiddb-read-server), write=read+10 --
    // the same convention every other EnkiDB type's write server uses
    // (enkidb-write-server=7011, enkidw-write-server=7012, etc).
    // Corrected 2026-07-31 from the stray 7101 default.
    env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7017".to_string())
}
fn data_dir() -> PathBuf {
    PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "/data".to_string()))
}
fn tribe_id() -> u16 {
    env::var("TRIBE_ID")
        .ok()
        .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x7160)
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
    eprintln!("𒁾 enkiddb-write-server — EnkiDDB (Tigris) Write Node");

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
    eprintln!("  data_dir  = {}", data_dir.display());
    eprintln!("  flush_every = {flush_every} documents");
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
    if src.trim().is_empty() {
        return send(&mut stream, "ERR:empty request");
    }

    if src.trim() == "FLUSH" {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        return match materialize_fresh(&st.write_node, data_dir) {
            Ok(stats) => {
                st.since_flush = 0;
                send(&mut stream, &format!("OK:FLUSHED:{}", stats.entities))
            }
            Err(e) => send(&mut stream, &format!("ERR:materialize: {e}")),
        };
    }

    if let Some(root) = src.trim().strip_prefix("INGEST_DIR:") {
        let allowlist = TeamAllowlist::from_env_or_seed();
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        st.epoch += 1;
        let start_epoch = st.epoch;
        return match st.write_node.ingest_directory_categorized_checked(Path::new(root), start_epoch, &allowlist) {
            Ok(kakis) => {
                let count = kakis.len();
                if count > 1 {
                    st.epoch += count as u32 - 1;
                }
                st.since_flush += count as u32;
                let should_flush = st.since_flush >= flush_every;
                if should_flush {
                    match materialize_fresh(&st.write_node, data_dir) {
                        Ok(_) => st.since_flush = 0,
                        Err(e) => eprintln!("[auto-flush] {e}"),
                    }
                }
                eprintln!(
                    "[ingest_dir] root={root} count={count}{}",
                    if should_flush { " [auto-flushed]" } else { "" }
                );
                send(&mut stream, &format!("OK:INGESTED:{count}"))
            }
            Err(e) => send(&mut stream, &format!("ERR:ingest_dir: {e}")),
        };
    }

    let mut parts = src.splitn(2, '\n');
    let collection = parts.next().unwrap_or("general").trim().to_string();
    let body = parts.next().unwrap_or("");
    if body.trim().is_empty() {
        return send(&mut stream, "ERR:missing document body after collection line");
    }

    let scan = enkiddb::scan_document(body.as_bytes());
    if !scan.clean {
        eprintln!("[security] rejected collection={collection}: {}", scan.detail);
        return send(&mut stream, &format!("ERR:security: {}", scan.detail));
    }

    let structure = DocumentParser::parse_markdown(body);
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    st.epoch += 1;
    let epoch = st.epoch;
    let section_count = structure.sections().len();
    let kaki = st.write_node.ingest_document_categorized(&structure, epoch, &collection);
    st.since_flush += 1;

    let should_flush = st.since_flush >= flush_every;
    if should_flush {
        match materialize_fresh(&st.write_node, data_dir) {
            Ok(_) => st.since_flush = 0,
            Err(e) => eprintln!("[auto-flush] {e}"),
        }
    }

    let hex: String = kaki.bytes().iter().map(|b| format!("{b:02x}")).collect();
    eprintln!(
        "[ingest] collection={collection} sections={section_count} epoch={epoch} kaki={hex}{}",
        if should_flush { " [auto-flushed]" } else { "" }
    );
    send(&mut stream, &format!("OK:{hex}:{section_count}"))
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
