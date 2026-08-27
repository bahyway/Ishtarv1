//! enkisdb-write-server — EnkiSDB Stage pipeline daemon (§12.1-12.3).
//!
//! This one process owns the whole real staging pipeline, not just EnkiSDB:
//!
//!   LandingZone --SdbPipeline--> SdbStore (Pending)
//!                                    |  (every SWEEP_INTERVAL_TICKS)
//!                              ValidationSweep
//!                                 pass |        | fail
//!                                     v          v
//!                             OdbStore        QdbStore
//!                          .drain_from_sdb  .drain_from_sdb
//!                             (EnkiODB)       (EnkiQDB)
//!
//! It is colocated on purpose, not split into three write processes: the
//! real domain code in `enkiodb::OdbStore::drain_from_sdb` and
//! `enkiqdb::QdbStore::drain_from_sdb` both take `&enkisdb::SdbStore` by
//! reference — an in-process borrow, not a network call. That is the
//! actual, already-designed shape of this pipeline stage, so this server
//! honors it rather than inventing a synthetic cross-process protocol for
//! three stores that were built to share one process. Each of the three
//! stages still gets its own real `enkidb_journal::Journal`, its own
//! materialized Data File pair, and its own independent Read Node (see
//! `enkisdb-read-server`, `enkiodb-read-server`, `enkiqdb-read-server`) --
//! the CQRS split (ADR-012) is per-*type*, not per-process.
//!
//! EnkiDW is NOT part of this pipeline: its own module doc says it is fed
//! by a separate LandingZone + `PersistedDb`, not by EnkiODB retirement
//! (`OdbStore::retire` only journals an `ArchiveMove` event locally; it
//! does not push into a DW store). See `enkidw-write-server`.
//!
//! ## Control protocol
//! Plain length-prefixed text frames (same convention as every other
//! *-write-server in this workspace) -- no JSON, no HTTP:
//! ```text
//!   - STATS -> OK:STATS:tick=<t>:sdb_pending=<n>:sdb_promoted=<n>:sdb_quarantined=<n>:odb_active=<n>:qdb_total=<n>
//!   - SWEEP -> force a ValidationSweep + drain right now (regardless of
//!              cadence), responds OK:SWEPT:promoted=<n>:quarantined=<n>:odb_moved=<n>:qdb_moved=<n>
//!   - FLUSH -> force-materialize all three Journals now, responds
//!              OK:FLUSHED:sdb=<n>:odb=<n>:qdb=<n>
//!   - anything else -> ERR:<message>
//! ```
//!
//! ## Background loop
//! A daemon thread polls the landing zone every `POLL_SECS` (default 5),
//! advances a real wall-clock tick counter (1 tick == 1 elapsed second, so
//! the sovereign default `SWEEP_INTERVAL_TICKS=900` really does mean 15
//! minutes), runs the sweep + drains when due, and re-materializes all
//! three Journals every `MATERIALIZE_EVERY_SECS` (default 30) so the three
//! Read Nodes see fresh data without a restart.
#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use bahyway_core::TribeId;
use enkidb_journal::{EventCause, Journal};
use enkidb_kaki::KakiMinter;
use enkiodb::OdbStore;
use enkiqdb::QdbStore;
use enkisdb::{SdbPipeline, SdbStore, ValidationSweep};

const MAX_FRAME: u32 = 16 * 1024 * 1024;
const READ_TIMEOUT: u64 = 30;
const WRITE_TIMEOUT: u64 = 120;

fn bind_addr() -> String {
    env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:7013".to_string())
}
fn landing_dir() -> PathBuf {
    PathBuf::from(env::var("LANDING_DIR").unwrap_or_else(|_| "/data/sdb/landing".to_string()))
}
fn sdb_data_dir() -> PathBuf {
    PathBuf::from(env::var("SDB_DATA_DIR").unwrap_or_else(|_| "/data/sdb".to_string()))
}
fn odb_data_dir() -> PathBuf {
    PathBuf::from(env::var("ODB_DATA_DIR").unwrap_or_else(|_| "/data/odb".to_string()))
}
fn qdb_data_dir() -> PathBuf {
    PathBuf::from(env::var("QDB_DATA_DIR").unwrap_or_else(|_| "/data/qdb".to_string()))
}
fn tribe_id() -> u16 {
    env::var("TRIBE_ID")
        .ok()
        .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x7300)
}
fn poll_secs() -> u64 {
    env::var("POLL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}
fn sweep_interval_ticks() -> u64 {
    env::var("SWEEP_INTERVAL_TICKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(enkisdb::DEFAULT_SWEEP_INTERVAL_TICKS)
}
fn materialize_every_secs() -> u64 {
    env::var("MATERIALIZE_EVERY_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

struct SharedState {
    tribe: TribeId,
    tick: u64,
    minter: KakiMinter,
    pipeline: SdbPipeline,
    sdb_store: SdbStore,
    sdb_journal: Journal,
    sweep: ValidationSweep,
    odb_store: OdbStore,
    odb_journal: Journal,
    qdb_store: QdbStore,
    qdb_journal: Journal,
}

impl SharedState {
    fn sweep_and_drain(&mut self) -> (usize, usize, usize, usize) {
        let result = self.sweep.run(
            &mut self.sdb_store,
            &mut self.sdb_journal,
            &self.minter,
            self.tribe,
            self.tick,
        );
        let odb_moved = self.odb_store.drain_from_sdb(
            &self.sdb_store,
            &mut self.odb_journal,
            &self.minter,
            self.tick,
        );
        let qdb_moved = self.qdb_store.drain_from_sdb(
            &self.sdb_store,
            EventCause::QuarantineMove,
            &mut self.qdb_journal,
            &self.minter,
            self.tick,
        );
        (result.promoted, result.quarantined, odb_moved, qdb_moved)
    }

    fn materialize_all(
        &self,
        sdb_dir: &Path,
        odb_dir: &Path,
        qdb_dir: &Path,
    ) -> io::Result<(usize, usize, usize)> {
        let sdb = materialize_fresh(&self.sdb_journal, sdb_dir)?;
        let odb = materialize_fresh(&self.odb_journal, odb_dir)?;
        let qdb = materialize_fresh(&self.qdb_journal, qdb_dir)?;
        Ok((sdb.entities, odb.entities, qdb.entities))
    }
}

fn main() {
    eprintln!("𒁾 enkisdb-write-server — EnkiSDB/EnkiODB/EnkiQDB staging pipeline daemon");

    let tribe = TribeId::from_u16(tribe_id());
    let landing = landing_dir();
    let sdb_dir = sdb_data_dir();
    let odb_dir = odb_data_dir();
    let qdb_dir = qdb_data_dir();
    let poll_secs = poll_secs();
    let materialize_every_secs = materialize_every_secs();

    let pipeline = SdbPipeline::open(&landing, tribe).unwrap_or_else(|e| {
        eprintln!("FATAL: cannot open landing zone {}: {e}", landing.display());
        std::process::exit(1);
    });

    let state = Mutex::new(SharedState {
        tribe,
        tick: 0,
        minter: KakiMinter::new(tribe),
        pipeline,
        sdb_store: SdbStore::new(),
        sdb_journal: Journal::new(64),
        sweep: ValidationSweep::new(sweep_interval_ticks()),
        odb_store: OdbStore::new(),
        odb_journal: Journal::new(64),
        qdb_store: QdbStore::new(),
        qdb_journal: Journal::new(64),
    });

    eprintln!("  landing_dir = {}", landing.display());
    eprintln!("  sdb_data_dir = {}", sdb_dir.display());
    eprintln!("  odb_data_dir = {}", odb_dir.display());
    eprintln!("  qdb_data_dir = {}", qdb_dir.display());
    eprintln!(
        "  tribe = {:#06x}  poll_secs = {poll_secs}  sweep_interval_ticks = {}",
        tribe.as_u16(),
        sweep_interval_ticks()
    );

    std::thread::scope(|scope| {
        // ── Background daemon: poll landing zone, sweep, materialize ──
        {
            let state = &state;
            let sdb_dir = sdb_dir.clone();
            let odb_dir = odb_dir.clone();
            let qdb_dir = qdb_dir.clone();
            scope.spawn(move || {
                let mut since_materialize = Instant::now();
                let mut last_staged = 0usize;
                loop {
                    std::thread::sleep(Duration::from_secs(poll_secs));
                    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                    st.tick += poll_secs;

                    let SharedState { pipeline, sdb_store, sdb_journal, .. } = &mut *st;
                    let ingest_stats = pipeline.run_once(sdb_store, sdb_journal).clone();
                    for alert in pipeline.take_alerts() {
                        eprintln!("[sdb-pipeline] {}", alert.message);
                    }
                    // records_staged is a lifetime-cumulative counter (see
                    // SdbPipelineStats), not a per-call delta -- only log
                    // when it actually grew, or every idle tick would
                    // reprint the same total forever.
                    if ingest_stats.records_staged > last_staged {
                        eprintln!("[sdb-pipeline] staged={} (+{})", ingest_stats.records_staged, ingest_stats.records_staged - last_staged);
                        last_staged = ingest_stats.records_staged;
                    }

                    if st.sweep.is_due(st.tick) {
                        let (promoted, quarantined, odb_moved, qdb_moved) = st.sweep_and_drain();
                        eprintln!(
                            "[sweep] tick={} promoted={promoted} quarantined={quarantined} odb_moved={odb_moved} qdb_moved={qdb_moved}",
                            st.tick
                        );
                    }

                    if since_materialize.elapsed() >= Duration::from_secs(materialize_every_secs) {
                        match st.materialize_all(&sdb_dir, &odb_dir, &qdb_dir) {
                            Ok((s, o, q)) => eprintln!("[materialize] sdb={s} odb={o} qdb={q}"),
                            Err(e) => eprintln!("[materialize] error: {e}"),
                        }
                        since_materialize = Instant::now();
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
                    let peer = s
                        .peer_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| "?".into());
                    let state_ref = &state;
                    let sdb_dir = sdb_dir.clone();
                    let odb_dir = odb_dir.clone();
                    let qdb_dir = qdb_dir.clone();
                    scope.spawn(move || {
                        if let Err(e) = handle(s, state_ref, &sdb_dir, &odb_dir, &qdb_dir) {
                            eprintln!("[{peer}] {e}");
                        }
                    });
                }
                Err(e) => eprintln!("[accept] {e}"),
            }
        }
    });
}

fn handle(
    mut stream: TcpStream,
    state: &Mutex<SharedState>,
    sdb_dir: &Path,
    odb_dir: &Path,
    qdb_dir: &Path,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT)))?;
    stream.set_write_timeout(Some(Duration::from_secs(WRITE_TIMEOUT)))?;

    let src = read_frame(&mut stream)?;
    match src.trim() {
        "STATS" => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            let sdb_stats = st.sdb_store.stats();
            send(&mut stream, &format!(
                "OK:STATS:tick={}:sdb_pending={}:sdb_promoted={}:sdb_quarantined={}:odb_active={}:qdb_total={}",
                st.tick, sdb_stats.pending_count(), sdb_stats.total_promoted, sdb_stats.total_quarantined,
                st.odb_store.stats().active_count, st.qdb_store.len(),
            ))
        }
        "SWEEP" => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let (promoted, quarantined, odb_moved, qdb_moved) = st.sweep_and_drain();
            send(&mut stream, &format!(
                "OK:SWEPT:promoted={promoted}:quarantined={quarantined}:odb_moved={odb_moved}:qdb_moved={qdb_moved}"
            ))
        }
        "FLUSH" => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st.materialize_all(sdb_dir, odb_dir, qdb_dir) {
                Ok((s, o, q)) => send(&mut stream, &format!("OK:FLUSHED:sdb={s}:odb={o}:qdb={q}")),
                Err(e) => send(&mut stream, &format!("ERR:materialize: {e}")),
            }
        }
        _ => send(
            &mut stream,
            "ERR:unrecognized request -- use STATS, SWEEP, or FLUSH",
        ),
    }
}

fn materialize_fresh(
    journal: &Journal,
    data_dir: &Path,
) -> io::Result<enkidb_readnode::MaterializeStats> {
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
