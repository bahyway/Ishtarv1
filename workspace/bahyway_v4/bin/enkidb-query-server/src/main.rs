//! enkidb-query-server — Sovereign TCP query server.
//!
//! Rebuilt against the current index/query stack. The original version
//! (PB-34/92/95/97/98) hand-rolled its own NATIRU/BTree candidate
//! filtering and imported `enkidb_enlil_index` / `hepta_shell_index` /
//! `geo_engine` — none of which exist anymore; that indexing work was
//! consolidated into `enkidb-indexes`.
//!
//! For a while after that consolidation, nothing actually called
//! `enkidb-indexes` from the query path — `heptascript::execute()` did a
//! full `journal.all_particles()` + per-particle `read_particle_history()`
//! scan on every query regardless of selectivity, which was the real,
//! undocumented reason 10,000-particle queries stayed slow after the
//! akk_decode-era bug was already fixed. `heptascript::build_indexes()` is
//! called once at startup below to build a real `enkidb-indexes`-backed
//! snapshot (`SurrogateMap` + `EavExactIndex`), and every query runs through
//! `heptascript::execute_indexed()`, which prunes via that snapshot before
//! reading any particle's history and falls back to the full scan only when
//! a query genuinely can't be pruned safely (see `heptascript::indexed` for
//! exactly which queries qualify). This server is a thin TCP wrapper around
//! that.
//!
//! ## BIGRING bridge
//! When a query's `ACROSS` clause is `BIGRING <name>` or `ALL`, each matched
//! particle's real orbital position (radius/azimuth/altitude, derived from
//! its actual KAKI bytes + B11 ColourID) is computed via
//! `bahyway_algebra::orbital` and included in the response, so a BIGRING
//! renderer (e.g. `bigring.gd` in DubSar Theater) can plot real tribe data
//! instead of procedurally-generated placeholder positions.
#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use akkvalue::AkkValue;
use bahyway_algebra::orbital::{orbital_position, OrbitalPosition};
use bahyway_core::TribeId;
use enkidb_persist::persisted_db::PersistedDb;
use enkidb_storage::FsyncPolicy;
use heptascript::{build_indexes, execute_indexed, parse_query, AcrossClause, HeptaIndexes, MatchedEntity, QueryResult};

const BIND_ADDR: &str = "0.0.0.0:7001";
const DEFAULT_DATA_DIR: &str = "/home/bahyway/enkidb/data";
const TRIBE_ID: u16 = 0x0001;

/// `ENKIDB_QUERY_DATA_DIR`, if set, overrides `DEFAULT_DATA_DIR` — lets a
/// test harness (e.g. playbook_99's E-004/E-005 client) or `enkidb-seed`
/// smoke test point the server at a scratch directory instead of the real
/// production journal, without needing a second copy of this binary.
fn data_dir() -> String {
    std::env::var("ENKIDB_QUERY_DATA_DIR").unwrap_or_else(|_| DEFAULT_DATA_DIR.to_string())
}
const MAX_FRAME: u32 = 16 * 1024 * 1024;
const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 120;

/// BIGRING orbital scale — outer ring radius (visual units, matches
/// bigring.gd / BIGRING_MultiSphere_Orbits.html scale).
const BIGRING_R_MAX: f64 = 100.0;
/// BIGRING orbital scale — max altitude above/below the equatorial plane.
const BIGRING_H_MAX: f64 = 20.0;
/// Neutral quality-distance fallback when a matched particle has no B11
/// ColourID projected (δ=0.5 places it mid-shell, ACTIVE band).
const DEFAULT_DELTA: f64 = 0.5;

fn main() {
    eprintln!("𒁾 enkidb-query-server — heptascript::execute over enkidb-persist journal");

    let tribe_id = TribeId::from_u16(TRIBE_ID);
    let t0 = Instant::now();

    let data_dir = data_dir();
    let db = PersistedDb::open(Path::new(&data_dir), tribe_id, FsyncPolicy::PerCommit)
        .unwrap_or_else(|e| {
            eprintln!("FATAL: {e}");
            std::process::exit(1);
        });

    eprintln!(
        "  ✓ {} entries loaded in {}ms",
        db.stats().entries_replayed,
        t0.elapsed().as_millis()
    );

    // Build the ENLIL-backed index snapshot once, here, at startup — not on
    // every query. This is the fix for the long-standing "100 particles /
    // 5+ seconds, 10K / hours" complaint: prior to this, execute() replayed
    // every particle's full journal history on every single query, because
    // nothing in the query path ever called enkidb-indexes despite it
    // existing. See heptascript::indexed for the full explanation.
    let t1 = Instant::now();
    let indexes = build_indexes(db.db().journal());
    eprintln!("  ✓ ENLIL index snapshot built in {}ms", t1.elapsed().as_millis());

    let shared_db = Arc::new(db);
    let shared_indexes = Arc::new(indexes);

    let listener = TcpListener::bind(BIND_ADDR).unwrap_or_else(|e| {
        eprintln!("FATAL: bind {BIND_ADDR}: {e}");
        std::process::exit(1);
    });
    eprintln!("𒁾 Listening on {BIND_ADDR}");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let peer = s
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| "?".into());
                let db = Arc::clone(&shared_db);
                let indexes = Arc::clone(&shared_indexes);
                thread::spawn(move || {
                    if let Err(e) = handle(s, &db, &indexes) {
                        eprintln!("[{peer}] {e}");
                    }
                });
            }
            Err(e) => eprintln!("[accept] {e}"),
        }
    }
}

fn handle(mut stream: TcpStream, db: &PersistedDb, indexes: &HeptaIndexes) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))?;
    stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT)))?;

    let src = read_frame(&mut stream)?;
    if src.trim().is_empty() {
        return send_err(&mut stream, "empty query");
    }

    let clean: String = src
        .lines()
        .filter(|l| !l.trim().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");

    let query = match parse_query(&clean) {
        Ok(q) => q,
        Err(e) => {
            return send_err(&mut stream, &format!("parse: {}", e.to_string().replace('"', "'")))
        }
    };

    let t0 = Instant::now();
    let result = execute_indexed(&query, db.db().journal(), indexes);
    let elapsed_ms = t0.elapsed().as_millis();
    let bigring = matches!(
        result.plan.across,
        Some(AcrossClause::Bigring(_)) | Some(AcrossClause::All)
    );
    eprintln!(
        "[query] {} matched / {} evaluated in {}ms{}",
        result.matched.len(),
        result.evaluated,
        elapsed_ms,
        if bigring { " [BIGRING]" } else { "" }
    );

    let json = result_to_json(&result, bigring);
    write_frame(&mut stream, &json)?;
    stream.write_all(&0u32.to_le_bytes())?;
    stream.flush()
}

/// Wire response shape: `{"rows": [...], "stats": {"matched": N,
/// "evaluated": N, "aborted": bool}}`. Previously a bare `[...]` array —
/// changed so `ABORT_SCAN`'s `QueryResult.aborted` (see heptascript::engine)
/// is observable over the wire at all, which E-005
/// (docs/TESTING_PLAYBOOK_PHASE1.md) requires. Every Godot caller goes
/// through `enkidb_tcp.gd`'s `execute_query()`, updated in the same pass to
/// unwrap this shape back into the flat `Array` its own callers still expect.
fn result_to_json(result: &QueryResult, bigring: bool) -> String {
    let rows = rows_to_json(result, bigring);
    format!(
        "{{\"rows\":{rows},\"stats\":{{\"matched\":{},\"evaluated\":{},\"aborted\":{}}}}}",
        result.matched.len(),
        result.evaluated,
        result.aborted
    )
}

fn rows_to_json(result: &QueryResult, bigring: bool) -> String {
    let matched = &result.matched;
    let mut out = String::with_capacity(matched.len() * 320);
    out.push('[');
    for (i, m) in matched.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let hex: String = m.entity.0.bytes().iter().map(|b| format!("{b:02x}")).collect();
        out.push_str("{\"kaki\":\"");
        out.push_str(&hex);
        out.push_str("\",\"attrs\":[");
        for (j, (attr, val)) in m.projected.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "[\"{}\",\"{}\"]",
                json_safe(attr, 80),
                json_safe(&fmt_val(val), 200)
            ));
        }
        out.push(']');
        // Only for QueryVerb::Prove -- `m.history` is empty for every other
        // verb (see MatchedEntity's own doc comment), so this key is
        // omitted entirely rather than sent as `"history":[]` noise on
        // every ORBIT/EMIT/SYNC/WITNESS row. One entry per real journal
        // epoch, oldest first, each with that epoch's own WHAT projection
        // -- a Godot client (e.g. a StoryEngine-style drill-down panel)
        // gets a particle's full history in the same response as its
        // current state, no second round-trip needed.
        if !m.history.is_empty() {
            out.push_str(",\"history\":[");
            for (k, snap) in m.history.iter().enumerate() {
                if k > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{{\"epoch\":{},\"attrs\":[", snap.epoch));
                for (j, (attr, val)) in snap.attrs.iter().enumerate() {
                    if j > 0 {
                        out.push(',');
                    }
                    out.push_str(&format!(
                        "[\"{}\",\"{}\"]",
                        json_safe(attr, 80),
                        json_safe(&fmt_val(val), 200)
                    ));
                }
                out.push_str("]}");
            }
            out.push(']');
        }
        if bigring {
            let pos = compute_orbit(m);
            let [x, y, z] = pos.to_cartesian();
            out.push_str(&format!(
                ",\"orbit\":{{\"radius\":{:.4},\"azimuth\":{:.4},\"altitude\":{:.4},\"x\":{:.4},\"y\":{:.4},\"z\":{:.4}}}",
                pos.radius, pos.azimuth, pos.altitude, x, y, z
            ));
        }
        out.push('}');
    }
    out.push(']');
    out
}

/// Compute a matched particle's real BIGRING orbital position.
///
/// δ (quality distance) comes from the B11 ColourID EAV attribute if the
/// query's WHAT clause projected it; falls back to `DEFAULT_DELTA` if not
/// present (the query didn't ask for it, or the particle has none yet).
fn compute_orbit(m: &MatchedEntity) -> OrbitalPosition {
    let delta = m
        .projected
        .iter()
        .find(|(attr, _)| attr == "b11")
        .and_then(|(_, val)| match val {
            AkkValue::Int(n) => Some((*n).clamp(0, 240) as f64 / 240.0),
            AkkValue::Float(f) => Some(f.clamp(0.0, 240.0) / 240.0),
            _ => None,
        })
        .map(|health| 1.0 - health)
        .unwrap_or(DEFAULT_DELTA);

    orbital_position(m.entity.0.bytes(), delta, BIGRING_R_MAX, BIGRING_H_MAX)
}

fn fmt_val(v: &AkkValue) -> String {
    match v {
        AkkValue::Text(s) => s.clone(),
        AkkValue::Int(n) => n.to_string(),
        AkkValue::Float(f) => format!("{f:.4}"),
        AkkValue::Bool(b) => b.to_string(),
        AkkValue::Null => "null".into(),
        other => format!("{other:?}"),
    }
}

fn json_safe(s: &str, max: usize) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(max)
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            o => vec![o],
        })
        .collect()
}

fn send_err(s: &mut TcpStream, msg: &str) -> io::Result<()> {
    write_frame(s, &format!("ERR:{msg}"))?;
    s.write_all(&0u32.to_le_bytes())?;
    s.flush()
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

fn write_frame(s: &mut TcpStream, payload: &str) -> io::Result<()> {
    let b = payload.as_bytes();
    s.write_all(&(b.len() as u32).to_le_bytes())?;
    s.write_all(b)?;
    s.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use akkvalue::codec;
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_journal::Journal;
    use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
    use heptascript::{execute, parse_query};

    fn attr_hash(name: &str) -> u32 {
        bahyway_crc::crc16(name.as_bytes()) as u32
    }

    #[test]
    fn rows_to_json_omits_history_for_a_plain_orbit_query() {
        let tribe = TribeId::from_u16(0x0001);
        let minter = KakiMinter::new(tribe);
        let mut jnl = Journal::new(64);
        let e = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
        jnl.append(JournalEntry::new(
            ek, e, 1,
            vec![EavTriple::new(attr_hash("b11"), codec::encode(&AkkValue::Int(150)))],
        )).unwrap();

        let q = parse_query("WHO T.E\nWHAT E[b11]").unwrap();
        let res = execute(&q, &jnl);
        let json = rows_to_json(&res, false);
        assert!(!json.contains("\"history\""), "ORBIT query must not emit a history key: {json}");
    }

    #[test]
    fn rows_to_json_includes_full_epoch_history_for_a_prove_query() {
        let tribe = TribeId::from_u16(0x0001);
        let minter = KakiMinter::new(tribe);
        let mut jnl = Journal::new(64);
        let e = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();

        for (epoch, b11) in [(1u32, 100i64), (2, 150), (3, 210)] {
            let ek = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
            jnl.append(JournalEntry::new(
                ek, e, epoch,
                vec![EavTriple::new(attr_hash("b11"), codec::encode(&AkkValue::Int(b11)))],
            )).unwrap();
        }

        let q = parse_query("PROVE\nWHO T.E\nWHAT E[b11]").unwrap();
        let res = execute(&q, &jnl);
        let json = rows_to_json(&res, false);
        assert!(json.contains("\"history\":["), "expected a history array: {json}");
        assert!(json.contains("\"epoch\":1"), "{json}");
        assert!(json.contains("\"epoch\":2"), "{json}");
        assert!(json.contains("\"epoch\":3"), "{json}");
        assert!(json.contains("\"b11\""), "{json}");
    }
}
