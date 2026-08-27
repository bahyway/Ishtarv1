//! enkidb-read-server — EnkiDB (core operational particle store) Read
//! Node, sovereign TCP server. Wraps `enkidb_readnode::CachedReadNode`
//! (the whole Data File loaded into RAM once at open/reload, zero disk
//! I/O per query — see that module's own doc comment for the real,
//! measured numbers behind why this replaced the disk-per-query
//! `ReadNode`) behind a real, hand-rolled **binary** wire protocol — no
//! JSON, ever, per the Architect's standing law for EnkiDB Types
//! communications. See [`encode_success`]/[`decode_response`]-shaped
//! comment below for the exact frame layout.
//!
//! ## Where its data comes from
//! This server never writes — it only opens `DATA_DIR/current/{entities,eav}`
//! read-only. `enkidb-write-server`'s `SEED:<n>`/`FLUSH` commands (or a
//! real Write Node sync step) are what populate that path — the same
//! CQRS split ADR-012 already establishes elsewhere.
//!
//! ## Live reload
//! A background thread re-opens `DATA_DIR/current` every `RELOAD_SECS`
//! (default 30) so newly-materialized data becomes visible without a
//! restart — the same full in-memory rebuild `CachedReadNode::open` pays
//! at startup, paid again on this schedule. If the path doesn't exist
//! yet, the server starts up anyway and answers an error frame until the
//! first successful reload.
//!
//! ## Concurrency
//! State is held behind an `RwLock`, not a `Mutex`: queries only ever
//! read the loaded snapshot, so concurrent connections can run queries
//! against it in parallel, genuinely on separate OS threads (one thread
//! per connection) — only the (rare) reload swap needs the exclusive
//! write lock, and it holds it only for the instant it swaps the `Arc`
//! in, not for the O(n) rebuild itself (built into a fresh value first,
//! then swapped in).
//!
//! ## Protocol
//! One request frame in, one response frame out (the same length-
//! prefixed-u32-LE + trailing 0-terminator framing every server in this
//! fleet already uses):
//! ```text
//!   - QUERY:<heptascript source> -> a binary response frame:
//!       [1 byte tag]
//!         0x00 success, followed by:
//!           [u32 LE row_count]
//!           for each row: [16B kaki] [u16 LE attr_count] {attrs} [u16 LE history_count] {history_snapshots}
//!             attr = [u8 name_len][name bytes][u8 type_tag][value bytes]
//!               type_tag: 0=Null 1=Bool(1B) 2=Int(8B LE i64) 3=Float(8B LE f64)
//!                         4=Text(u32 LE len + bytes) 5=KakiPk(16B)
//!             history snapshot = [u32 LE epoch][u16 LE attr_count]{attrs}
//!           [u8 trailer_tag] 0=none 1=sync_fingerprint 2=witness_digest,
//!             followed by [u8 len][bytes] (hex-string bytes) when non-zero
//!         0x01 error, followed by [u32 LE len][utf8 message bytes]
//!   - TRIBES:<heptascript source> -> same shape as QUERY:, answered
//!       from the real per-tribe particle-count corpus materialize()
//!       writes alongside the main one (2026-07-21, the Architect's "PU"
//!       idea) -- tribe.id/tribe.particle_count/meta.kind attrs.
//!   - anything else -> the same 0x01 error frame shape
//! ```
#![forbid(unsafe_code)]

use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use akkvalue::AkkValue;
use enkidb_readnode::CachedReadNode;

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 120;

fn bind_addr() -> String {
    // 7001 is the port every existing client already dials for EnkiDB
    // queries (`enki_engines.gd`'s TABLE, `heptascript::query::DbTarget
    // ::port()`, and the old undeployed `enkidb-query-server`) -- kept
    // here so this real CQRS read side is a drop-in replacement for
    // that, not a new port clients would need to learn.
    env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7001".to_string())
}
fn data_dir() -> PathBuf {
    PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "/data".to_string()))
}
fn reload_secs() -> u64 {
    env::var("RELOAD_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

struct Live {
    read_node: CachedReadNode,
    tribes: Option<CachedReadNode>,
}

fn try_load(data_dir: &std::path::Path) -> Option<Live> {
    let current = data_dir.join("current");
    let entities = current.join("entities");
    let eav = current.join("eav");
    let read_node = CachedReadNode::open(&entities, &eav).ok()?;
    let tribes = enkidb_readnode::open_tribe_summaries(&entities, &eav);
    Some(Live { read_node, tribes })
}

fn main() {
    eprintln!(
        "𒁾 enkidb-read-server — EnkiDB (core particle store) Read Node [in-memory, binary wire]"
    );

    let data_dir = data_dir();
    let reload_secs = reload_secs();
    let state: Arc<RwLock<Option<Arc<Live>>>> =
        Arc::new(RwLock::new(try_load(&data_dir).map(Arc::new)));

    {
        let ready = state.read().unwrap().is_some();
        eprintln!("  data_dir = {}", data_dir.display());
        eprintln!(
            "  initial state: {}",
            if ready { "loaded" } else { "not ready yet" }
        );
    }

    {
        let state = Arc::clone(&state);
        let data_dir = data_dir.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(reload_secs));
            if let Some(live) = try_load(&data_dir) {
                let entities = live.read_node.entity_count();
                let tribes = live.tribes.as_ref().map(|t| t.entity_count());
                // Build fully off to the side, then swap the Arc in under
                // the write lock only for the instant of the swap itself
                // -- readers already holding a cloned Arc from before the
                // swap keep querying the old snapshot to completion, no
                // reader is ever blocked on a reload's O(n) rebuild.
                *state.write().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(live));
                eprintln!("[reload] entities={entities} tribes={tribes:?}");
            }
        });
    }

    let addr = bind_addr();
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("FATAL: bind {addr}: {e}");
        std::process::exit(1);
    });
    eprintln!("𒁾 Listening on {addr}");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let peer = s
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| "?".into());
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(e) = handle(s, &state) {
                        eprintln!("[{peer}] {e}");
                    }
                });
            }
            Err(e) => eprintln!("[accept] {e}"),
        }
    }
}

fn handle(mut stream: TcpStream, state: &RwLock<Option<Arc<Live>>>) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))?;
    stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT)))?;

    let src = read_frame(&mut stream)?;

    // Clone the Arc under the read lock, then release the lock
    // immediately -- the query itself runs against the cloned Arc with
    // no lock held at all, so concurrent queries never contend with each
    // other, only (briefly) with a reload swap.
    let live = state.read().unwrap_or_else(|e| e.into_inner()).clone();
    let Some(live) = live else {
        return send_error(&mut stream, "not ready -- Data Files not seeded/synced yet");
    };

    if let Some(query) = src.strip_prefix("QUERY:") {
        return match live.read_node.query(query) {
            Ok(result) => send_success(&mut stream, &result),
            Err(e) => send_error(&mut stream, &format!("query: {e:?}")),
        };
    }

    if let Some(query) = src.strip_prefix("TRIBES:") {
        let Some(tribes) = &live.tribes else {
            return send_error(&mut stream, "tribe-summary corpus not available -- data predates this feature or hasn't been re-materialized yet");
        };
        return match tribes.query(query) {
            Ok(result) => send_success(&mut stream, &result),
            Err(e) => send_error(&mut stream, &format!("query: {e:?}")),
        };
    }

    send_error(
        &mut stream,
        "unrecognized request -- use QUERY:<heptascript> or TRIBES:<heptascript>",
    )
}

// ── Binary encoding (see module doc comment for the exact frame layout) ──

fn send_success(stream: &mut TcpStream, result: &heptascript::QueryResult) -> io::Result<()> {
    let mut buf = Vec::with_capacity(64 + result.matched.len() * 96);
    buf.push(0x00u8); // success tag
    encode_rows(&mut buf, &result.matched);
    match result.verb {
        heptascript::QueryVerb::Sync => {
            buf.push(1u8);
            encode_short_string(&mut buf, result.state_fingerprint.as_deref().unwrap_or(""));
        }
        heptascript::QueryVerb::Witness => {
            buf.push(2u8);
            encode_short_string(&mut buf, result.witness_digest.as_deref().unwrap_or(""));
        }
        _ => buf.push(0u8),
    }
    encode_aggregate(&mut buf, result);
    send_frame(stream, &buf)
}

fn send_error(stream: &mut TcpStream, msg: &str) -> io::Result<()> {
    let mut buf = Vec::with_capacity(msg.len() + 5);
    buf.push(0x01u8); // error tag
    let bytes = msg.as_bytes();
    buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(bytes);
    send_frame(stream, &buf)
}

fn encode_rows(buf: &mut Vec<u8>, matched: &[heptascript::MatchedEntity]) {
    buf.extend_from_slice(&(matched.len() as u32).to_le_bytes());
    for m in matched {
        buf.extend_from_slice(m.entity.bytes());
        encode_attrs(buf, &m.projected);
        buf.extend_from_slice(&(m.history.len() as u16).to_le_bytes());
        for snap in &m.history {
            buf.extend_from_slice(&snap.epoch.to_le_bytes());
            encode_attrs(buf, &snap.attrs);
        }
    }
}

fn encode_attrs(buf: &mut Vec<u8>, attrs: &[(String, AkkValue)]) {
    buf.extend_from_slice(&(attrs.len() as u16).to_le_bytes());
    for (name, val) in attrs {
        let name_bytes = &name.as_bytes()[..name.len().min(255)];
        buf.push(name_bytes.len() as u8);
        buf.extend_from_slice(name_bytes);
        encode_value(buf, val);
    }
}

fn encode_value(buf: &mut Vec<u8>, v: &AkkValue) {
    match v {
        AkkValue::Null => buf.push(0),
        AkkValue::Bool(b) => {
            buf.push(1);
            buf.push(*b as u8);
        }
        AkkValue::Int(n) => {
            buf.push(2);
            buf.extend_from_slice(&n.to_le_bytes());
        }
        AkkValue::Float(f) => {
            buf.push(3);
            buf.extend_from_slice(&f.to_le_bytes());
        }
        AkkValue::Text(s) => {
            buf.push(4);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        AkkValue::KakiPk(pk) => {
            buf.push(5);
            let mut fixed = [0u8; 16];
            let n = pk.len().min(16);
            fixed[..n].copy_from_slice(&pk[..n]);
            buf.extend_from_slice(&fixed);
        }
        other => {
            // Any AkkValue variant with no dedicated wire tag falls back
            // to its Debug text, same honesty as the old JSON path's own
            // `_ => format!("{v:?}")` catch-all -- never silently drops
            // a value, just represents it as text (tag 4).
            buf.push(4);
            let s = format!("{other:?}");
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
    }
}

// ── MEASURE / GRAVITY aggregate trailer ──────────────────────────────────
//
// Appended after the existing verb trailer (sync_fingerprint/witness_digest)
// since MEASURE/GRAVITY can co-occur with any of the five sovereign verbs,
// not just SYNC/WITNESS -- a separate tag byte, not a new case folded into
// the verb-trailer match above.
//   `[u8 aggregate_tag]` 0=none 1=measured 2=grouped
//     1: `[measure_value]` (see below)
//     2: `[u8 capped][u32 LE group_count]` then per group:
//          `[gravity_key][u64 LE count][measure_value]`
//   measure_value = `[u8 kind][payload]`
//     kind 0=Dense(u64 LE) 1=Flux(f64 LE) 2=RotorMean(f64 LE)
//   gravity_key = `[u8 key_tag][payload]`
//     tag 0=Band(i64 LE) 1=Exact(u32 LE len + utf8 bytes) 2=Missing 3=Overflow

fn encode_aggregate(buf: &mut Vec<u8>, result: &heptascript::QueryResult) {
    match (&result.measured, &result.grouped) {
        (Some(m), None) => {
            buf.push(1u8);
            encode_measure_value(buf, m);
        }
        (None, Some(g)) => {
            buf.push(2u8);
            buf.push(g.capped as u8);
            buf.extend_from_slice(&(g.groups.len() as u32).to_le_bytes());
            for group in &g.groups {
                encode_gravity_key(buf, &group.key);
                buf.extend_from_slice(&(group.count as u64).to_le_bytes());
                encode_measure_value(buf, &group.measure);
            }
        }
        _ => buf.push(0u8),
    }
}

fn encode_measure_value(buf: &mut Vec<u8>, m: &heptascript::MeasureValue) {
    match m {
        // u64 LE, not u32 -- DENSE is exactly the aggregate meant to keep
        // working past the row counts this wire format otherwise bounds
        // (matched.len() as u32 elsewhere), so its own count must not
        // silently wrap at 4.29B when a Tribe scales past that.
        heptascript::MeasureValue::Dense(n) => {
            buf.push(0u8);
            buf.extend_from_slice(&(*n as u64).to_le_bytes());
        }
        heptascript::MeasureValue::Flux(f) => {
            buf.push(1u8);
            buf.extend_from_slice(&f.to_le_bytes());
        }
        heptascript::MeasureValue::RotorMean(f) => {
            buf.push(2u8);
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
}

fn encode_gravity_key(buf: &mut Vec<u8>, key: &heptascript::GravityKey) {
    match key {
        heptascript::GravityKey::Band(b) => {
            buf.push(0u8);
            buf.extend_from_slice(&b.to_le_bytes());
        }
        heptascript::GravityKey::Exact(s) => {
            buf.push(1u8);
            let bytes = s.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        heptascript::GravityKey::Missing => buf.push(2u8),
        heptascript::GravityKey::Overflow => buf.push(3u8),
    }
}

fn encode_short_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = &s.as_bytes()[..s.len().min(255)];
    buf.push(bytes.len() as u8);
    buf.extend_from_slice(bytes);
}

fn send_frame(stream: &mut TcpStream, payload: &[u8]) -> io::Result<()> {
    stream.write_all(&(payload.len() as u32).to_le_bytes())?;
    stream.write_all(payload)?;
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
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame too large: {len}"),
        ));
    }
    let mut buf = vec![0u8; len as usize];
    s.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
