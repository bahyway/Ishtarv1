//! enkidb-write-server — EnkiDB (core operational particle store) Write
//! Node, sovereign TCP server. The core type 7001 EAV particle store never
//! had a deployed CQRS pair (unlike EnkiDDB/EnkiMDB) — only the old,
//! undeployed `enkidb-query-server` existed for it, on a different wire
//! protocol. This mirrors `enkiddb-write-server`'s conventions (length-
//! prefixed frame, one thread per connection, materialize-on-flush).
//!
//! ## Protocol
//! One request frame in, one response frame out, per connection:
//!   - `SEED:<n>`   -> replace the current Journal with `n` freshly
//!                     generated synthetic particles (round-robin across
//!                     5 real station names, one globally-unique "needle"
//!                     particle at the midpoint — the exact generator
//!                     `enkidb-readnode`'s `scale_benchmark` example uses),
//!                     then materialize to `DATA_DIR/current`. Responds
//!                     `OK:SEEDED:<n>:<generate_ms>:<materialize_ms>`.
//!                     This is a scale/demo tool, not a general-purpose
//!                     ingest API: real particle ingestion belongs to the
//!                     ETL station chain (BeeMDM), which is out of this
//!                     server's scope.
//!   - `FLUSH`      -> re-materialize the current Journal state now.
//!                     Responds `OK:FLUSHED:<entity_count>`.
//!   - anything else -> `ERR:<message>`
//!
//! ## Durability
//! Same v1 boundary as `enkiddb-write-server`: the Journal is in-memory
//! only for this process; `DATA_DIR/current` is the durable, materialized
//! state a paired `enkidb-read-server` actually serves from.
#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use akkvalue::{codec, AkkValue};
use bahyway_core::TribeId;
use enkidb_journal::entry::EavTriple;
use enkidb_journal::Journal;
use enkidb_kaki::{mint::KakiMinter, EventKaki, IdentityKaki, KakiRole};

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 600; // SEED at scale can take real time to materialize.

const STATIONS: &[&str] = &[
    "data-cleansing-station",
    "data-steward-station",
    "permanent-storage",
    "grave-discovery-station",
    "storage-sector",
];

fn bind_addr() -> String {
    // 7001 is already spoken for as EnkiDB's client-facing query port
    // (see enkidb-read-server's own comment) -- 7011 is a plain,
    // non-colliding choice for the write-only SEED/FLUSH side, which no
    // existing client dials today.
    env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7011".to_string())
}
fn data_dir() -> PathBuf {
    PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "/data".to_string()))
}
fn tribe_id() -> u16 {
    env::var("TRIBE_ID")
        .ok()
        .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x7001)
}

struct SharedState {
    journal: Journal,
    minter: KakiMinter,
}

fn main() {
    eprintln!("𒁾 enkidb-write-server — EnkiDB (core particle store) Write Node");

    let minter = KakiMinter::new(TribeId::from_u16(tribe_id()));
    let state = Mutex::new(SharedState { journal: Journal::new(64), minter });
    let data_dir = data_dir();

    let addr = bind_addr();
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("FATAL: bind {addr}: {e}");
        std::process::exit(1);
    });
    eprintln!("  data_dir = {}", data_dir.display());
    eprintln!("𒁾 Listening on {addr}");

    std::thread::scope(|scope| {
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let peer = s.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "?".into());
                    let state_ref = &state;
                    let data_dir_ref = &data_dir;
                    scope.spawn(move || {
                        if let Err(e) = handle(s, state_ref, data_dir_ref) {
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
    let trimmed = src.trim();

    if trimmed == "FLUSH" {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        return match materialize_fresh(&st.journal, data_dir) {
            Ok(stats) => send(&mut stream, &format!("OK:FLUSHED:{}", stats.entities)),
            Err(e) => send(&mut stream, &format!("ERR:materialize: {e}")),
        };
    }

    if let Some(rest) = trimmed.strip_prefix("SEED:") {
        let n: usize = match rest.trim().parse() {
            Ok(n) if n > 0 => n,
            _ => return send(&mut stream, "ERR:SEED:<n> requires a positive integer"),
        };
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());

        let t0 = Instant::now();
        st.journal = Journal::new(64);
        let SharedState { journal, minter } = &mut *st;
        seed_journal(journal, minter, n);
        let generate_ms = t0.elapsed().as_millis();

        let t1 = Instant::now();
        return match materialize_fresh(&st.journal, data_dir) {
            Ok(stats) => {
                let materialize_ms = t1.elapsed().as_millis();
                eprintln!(
                    "[seed] n={n} entities={} generate_ms={generate_ms} materialize_ms={materialize_ms}",
                    stats.entities
                );
                send(&mut stream, &format!("OK:SEEDED:{n}:{generate_ms}:{materialize_ms}"))
            }
            Err(e) => send(&mut stream, &format!("ERR:materialize: {e}")),
        };
    }

    send(&mut stream, "ERR:unrecognized request -- use SEED:<n> or FLUSH")
}

/// The exact generator `enkidb-readnode`'s `scale_benchmark` example
/// uses: N particles round-robin across 5 real station names, one
/// entity at the midpoint carrying a globally-unique `needle_id` so a
/// single-row point-lookup query has a real target to find.
fn seed_journal(journal: &mut Journal, minter: &KakiMinter, n: usize) {
    for i in 0..n {
        let e = IdentityKaki::try_from_kaki(minter.mint_identity(i as u32, KakiRole::Zikru)).unwrap();
        let ek =
            EventKaki::try_from_kaki(minter.mint_event((i as u32) ^ 0xFFFF_FFFF, KakiRole::Zikru)).unwrap();
        let station = STATIONS[i % STATIONS.len()];
        let mut eav = vec![EavTriple::new(
            bahyway_crc::crc16("station".as_bytes()) as u32,
            codec::encode(&AkkValue::Text(station.into())),
        )];
        if i == n / 2 {
            eav.push(EavTriple::new(
                bahyway_crc::crc16("needle_id".as_bytes()) as u32,
                codec::encode(&AkkValue::Text("needle-particle-42".into())),
            ));
        }
        journal
            .append(enkidb_journal::entry::JournalEntry::new(ek, e, 1, eav))
            .unwrap();
    }
}

fn materialize_fresh(journal: &Journal, data_dir: &Path) -> io::Result<enkidb_readnode::MaterializeStats> {
    let current = data_dir.join("current");
    let _ = fs::remove_dir_all(&current);
    fs::create_dir_all(&current)?;
    enkidb_readnode::materialize(journal, current.join("entities"), current.join("eav"))
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
