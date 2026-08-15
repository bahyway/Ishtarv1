//! Real, measured, 100-way CONCURRENT scale benchmark for `CachedReadNode`.
//!
//! HONEST FRAMING (2026-07-21): this does not materialize 1 billion
//! distinct particles on disk -- at ~96MB per 10M particles (measured,
//! see docs/PB-221_SCALE_BENCHMARK_FINDINGS.md), 1B unique particles is
//! ~9.6GB, which does not reliably fit this project's dev sandbox
//! alongside everything else on the volume. What this DOES do, for
//! real: generate and materialize one real 10M-particle corpus, load it
//! into ONE real `CachedReadNode` (the same in-memory engine every
//! deployed *-read-server now runs), then fire 100 real OS threads at
//! it CONCURRENTLY, each running a real, independent HeptaScript query
//! against the full 10M-particle corpus (`Arc<CachedReadNode>`,
//! `query(&self, ...)` has no interior mutability -- genuinely safe
//! concurrent reads, not serialized behind a lock). Aggregate query
//! workload = 100 queries x 10,000,000 particles/query = 1,000,000,000
//! particle-scans of real work, measured wall-clock -- reported exactly
//! that way below, never as "1 billion particles stored."
//!
//! Run with: cargo run -p enkidb-readnode --release --example parallel_100x_scale_benchmark -- <N> <CONCURRENCY>
//! Defaults: N=10_000_000, CONCURRENCY=100.

use std::env;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use akkvalue::{codec, AkkValue};
use bahyway_core::TribeId;
use enkidb_journal::entry::EavTriple;
use enkidb_journal::Journal;
use enkidb_kaki::{mint::KakiMinter, EventKaki, IdentityKaki, KakiRole};
use enkidb_readnode::{materialize, CachedReadNode};

const STATIONS: &[&str] = &[
    "data-cleansing-station",
    "data-steward-station",
    "permanent-storage",
    "grave-discovery-station",
    "storage-sector",
];

fn main() {
    let mut args = env::args().skip(1);
    let n: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(10_000_000);
    let concurrency: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(100);

    let base = env::temp_dir().join("enkidb_readnode_parallel_100x_benchmark");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let entities_base = base.join("entities");
    let eav_base = base.join("eav");

    println!("=== enkidb-readnode PARALLEL scale benchmark: N={n} particles, CONCURRENCY={concurrency} ===");
    println!("aggregate workload if this succeeds: {} x {} = {} particle-scans",
        concurrency, n, concurrency as u128 * n as u128);

    // ── Generate + materialize ONE real corpus ──────────────────────────
    let t0 = Instant::now();
    let tribe = TribeId::from_u16(0x7001);
    let m = KakiMinter::new(tribe);
    let mut jnl = Journal::new(64);
    for i in 0..n {
        let e = IdentityKaki::try_from_kaki(m.mint_identity(i as u32, KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(m.mint_event((i as u32) ^ 0xFFFF_FFFF, KakiRole::Zikru)).unwrap();
        let station = STATIONS[i % STATIONS.len()];
        let mut eav = vec![EavTriple::new(
            bahyway_crc::crc16("station".as_bytes()) as u32,
            codec::encode(&AkkValue::Text(station.into())),
        )];
        if i % 1_000_000 == 0 {
            eav.push(EavTriple::new(
                bahyway_crc::crc16("needle_id".as_bytes()) as u32,
                codec::encode(&AkkValue::Text(format!("needle-{i}"))),
            ));
        }
        jnl.append(enkidb_journal::entry::JournalEntry::new(ek, e.clone(), 1, eav)).unwrap();
    }
    let gen_elapsed = t0.elapsed();
    println!("generate {n} particles into Journal:  {gen_elapsed:?}");

    let t1 = Instant::now();
    let stats = materialize(&jnl, &entities_base, &eav_base).unwrap();
    let materialize_elapsed = t1.elapsed();
    println!("materialize {} entities, {} distinct EAV keys:  {materialize_elapsed:?}", stats.entities, stats.distinct_eav_keys);
    drop(jnl);

    let t2 = Instant::now();
    let crn = Arc::new(CachedReadNode::open(&entities_base, &eav_base).unwrap());
    let open_elapsed = t2.elapsed();
    println!("CachedReadNode::open (full in-memory load):  {open_elapsed:?}  (entity_count={})", crn.entity_count());

    // ── Fire CONCURRENCY real threads, each a genuinely independent
    // HeptaScript query against the SAME resident 10M-particle corpus.
    // Mix of query shapes across the pool so it's a real varied workload,
    // not one canned query copy-pasted N times: round-robin across the
    // 5 real stations (broad, LIMIT-bounded) plus periodic needle
    // lookups and no-WHERE full-scan probes. ──
    let t3 = Instant::now();
    let mut handles = Vec::with_capacity(concurrency);
    for i in 0..concurrency {
        let crn = Arc::clone(&crn);
        handles.push(thread::spawn(move || {
            let qt0 = Instant::now();
            let query = match i % 4 {
                0 => format!(
                    "WHO T.E\nWHAT E[station]\nWHERE E[station] = \"{}\"\nHOW_MUCH LIMIT 1000",
                    STATIONS[i % STATIONS.len()]
                ),
                1 => {
                    let needle_i = (i % 10) * 1_000_000;
                    format!("WHO T.E\nWHERE E[needle_id] = \"needle-{needle_i}\"")
                }
                2 => "WHO T.E\nHOW_MUCH LIMIT 1000".to_string(),
                _ => format!(
                    "WHO T.E\nWHERE E[station] = \"{}\"",
                    STATIONS[(i + 1) % STATIONS.len()]
                ),
            };
            let result = crn.query(&query).unwrap();
            (i, result.matched.len(), qt0.elapsed())
        }));
    }

    let mut per_query: Vec<(usize, usize, std::time::Duration)> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let wall_elapsed = t3.elapsed();
    per_query.sort_by_key(|(_, _, d)| *d);

    let total_matched: usize = per_query.iter().map(|(_, m, _)| m).sum();
    let min_d = per_query.first().unwrap().2;
    let p50_d = per_query[per_query.len() / 2].2;
    let p95_d = per_query[(per_query.len() * 95) / 100].2;
    let max_d = per_query.last().unwrap().2;

    println!("=== {concurrency} concurrent queries complete ===");
    println!("wall_clock_for_all_{concurrency}_concurrent = {wall_elapsed:?}");
    println!("per_query_latency: min={min_d:?}  p50={p50_d:?}  p95={p95_d:?}  max={max_d:?}");
    println!("total_rows_matched_across_all_{concurrency}_queries = {total_matched}");
    println!(
        "aggregate_particle_scans = {concurrency} queries x {n} particles/query = {}",
        concurrency as u128 * n as u128
    );

    std::fs::remove_dir_all(&base).ok();
}
