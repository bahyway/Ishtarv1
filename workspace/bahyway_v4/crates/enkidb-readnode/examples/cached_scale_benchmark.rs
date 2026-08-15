//! Real, measured, end-to-end scale benchmark for `CachedReadNode` — the
//! in-memory-resident engine (see `crate::cached`'s own doc comment for
//! why it exists). Same generator, same station distribution, same
//! needle particle as `scale_benchmark.rs`, so its numbers are a direct,
//! apples-to-apples comparison against the disk-based `ReadNode`.
//!
//! Run with: cargo run -p enkidb-readnode --release --example cached_scale_benchmark -- <N>

use std::env;
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
    let n: usize = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let base = env::temp_dir().join("enkidb_readnode_cached_scale_benchmark");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let entities_base = base.join("entities");
    let eav_base = base.join("eav");

    println!("=== enkidb-readnode CACHED scale benchmark: N = {n} particles ===");

    let t0 = Instant::now();
    let tribe = TribeId::from_u16(0x7001);
    let m = KakiMinter::new(tribe);
    let mut jnl = Journal::new(64);
    let mut needle: Option<IdentityKaki> = None;

    for i in 0..n {
        let e = IdentityKaki::try_from_kaki(m.mint_identity(i as u32, KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(m.mint_event((i as u32) ^ 0xFFFF_FFFF, KakiRole::Zikru)).unwrap();
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
        jnl.append(enkidb_journal::entry::JournalEntry::new(ek, e.clone(), 1, eav)).unwrap();
        if i == n / 2 {
            needle = Some(e);
        }
    }
    let needle = needle.expect("n must be > 0");
    let gen_elapsed = t0.elapsed();
    println!("generate {n} particles into Journal:  {gen_elapsed:?}");

    let t1 = Instant::now();
    let stats = materialize(&jnl, &entities_base, &eav_base).unwrap();
    let materialize_elapsed = t1.elapsed();
    println!(
        "materialize {} entities, {} distinct EAV keys:  {materialize_elapsed:?}",
        stats.entities, stats.distinct_eav_keys
    );
    drop(jnl);

    // ── Open: the real cost this engine pays that ReadNode doesn't --
    // reading the ENTIRE Data File into RAM. Measured explicitly because
    // it's the one honest tradeoff of this design.
    let t2 = Instant::now();
    let crn = CachedReadNode::open(&entities_base, &eav_base).unwrap();
    let open_elapsed = t2.elapsed();
    println!(
        "CachedReadNode::open (full in-memory load):  {open_elapsed:?}  (entity_count={})",
        crn.entity_count()
    );

    // ── Query 1: unbounded broad match (~1/5 of the dataset). Zero disk
    // I/O now -- this is the number that was 1.95s/26.4s on ReadNode. ──
    let t3 = Instant::now();
    let broad = crn.query("WHO T.E\nWHERE E[station] = \"data-cleansing-station\"").unwrap();
    let broad_elapsed = t3.elapsed();
    println!(
        "query WHERE station = 'data-cleansing-station' (UNBOUNDED)  ->  {} matched  in {broad_elapsed:?}",
        broad.matched.len()
    );

    // ── Query 2: same broad match, HOW_MUCH LIMIT-bounded. ──
    let t3b = Instant::now();
    let limited = crn
        .query("WHO T.E\nWHAT E[station]\nWHERE E[station] = \"data-cleansing-station\"\nHOW_MUCH LIMIT 1000")
        .unwrap();
    let limited_elapsed = t3b.elapsed();
    println!(
        "query WHERE station = 'data-cleansing-station' HOW_MUCH LIMIT 1000  ->  {} matched  in {limited_elapsed:?}",
        limited.matched.len()
    );

    // ── Query 3: single-entity needle-in-haystack. ──
    let t4 = Instant::now();
    let needle_result = crn.query("WHO T.E\nWHERE E[needle_id] = \"needle-particle-42\"").unwrap();
    let needle_elapsed = t4.elapsed();
    let found_needle = needle_result.matched.iter().any(|me| *me.entity.bytes() == *needle.bytes());
    println!(
        "query WHERE needle_id = 'needle-particle-42' (1 match out of {n})  in {needle_elapsed:?}  needle_found={found_needle}"
    );

    // ── Query 4: NO WHERE clause at all -- ReadNode cannot serve this
    // (RequiresWriteNode); CachedReadNode does, via a full in-memory
    // scan. Real new capability, worth measuring honestly (its cost is
    // the full-scan clone cost documented in cached.rs). ──
    let t5 = Instant::now();
    let unfiltered = crn.query("WHO T.E\nHOW_MUCH LIMIT 1000").unwrap();
    let unfiltered_elapsed = t5.elapsed();
    println!(
        "query WHO T.E (no WHERE) HOW_MUCH LIMIT 1000  ->  {} matched  in {unfiltered_elapsed:?}",
        unfiltered.matched.len()
    );

    println!("=== summary ===");
    println!(
        "N={n}  generate={gen_elapsed:?}  materialize={materialize_elapsed:?}  cached_open={open_elapsed:?}  \
broad_unbounded({}_matches)={broad_elapsed:?}  broad_limit_1000({}_matches)={limited_elapsed:?}  \
needle_query(1_match)={needle_elapsed:?}  no_where_limit_1000={unfiltered_elapsed:?}",
        broad.matched.len(),
        limited.matched.len(),
    );

    std::fs::remove_dir_all(&base).ok();
}
