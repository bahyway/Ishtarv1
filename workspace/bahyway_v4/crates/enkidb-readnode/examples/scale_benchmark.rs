//! Real, measured, end-to-end scale benchmark: builds a Journal of N
//! particles distributed across the real station names in this workspace,
//! materializes it into Data Files (the one-time O(n) batch cost), opens a
//! fresh `ReadNode` (O(1) -- no journal replay), then runs actual
//! HeptaScript query strings through `ReadNode::query()` -- the same
//! parser + evaluator the rest of the system uses, not a synthetic
//! point-lookup microbenchmark.
//!
//! Run with: cargo run -p enkidb-readnode --release --example scale_benchmark -- <N>

use std::env;
use std::time::Instant;

use akkvalue::{codec, AkkValue};
use bahyway_core::TribeId;
use enkidb_journal::entry::EavTriple;
use enkidb_journal::Journal;
use enkidb_kaki::{mint::KakiMinter, EventKaki, IdentityKaki, KakiRole};
use enkidb_readnode::{materialize, ReadNode};

const STATIONS: &[&str] = &[
    "data-cleansing-station",
    "data-steward-station",
    "permanent-storage",
    "grave-discovery-station",
    "storage-sector",
];

fn main() {
    let n: usize = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    let base = env::temp_dir().join("enkidb_readnode_scale_benchmark");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let entities_base = base.join("entities");
    let eav_base = base.join("eav");

    println!("=== enkidb-readnode scale benchmark: N = {n} particles ===");

    // ── Generate: N particles, round-robin across 5 real station names,
    // one entity picked as the needle for a highly selective query. ──
    let t0 = Instant::now();
    let tribe = TribeId::from_u16(0x7001);
    let m = KakiMinter::new(tribe);
    let mut jnl = Journal::new(64);
    let mut needle: Option<IdentityKaki> = None;

    for i in 0..n {
        // Deterministic uuid_hash (index-derived), not the random
        // nanosecond-mixed `identity()`/`event()`: at high volume the
        // latter's 32-bit hash hits real birthday-bound collisions (seen
        // directly in this benchmark before this fix -- 100,000 mints
        // already produced a duplicate KAKI). See the writeup for why
        // this is a real, separate finding worth fixing in KakiMinter.
        let e = IdentityKaki::try_from_kaki(m.mint_identity(i as u32, KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(m.mint_event((i as u32) ^ 0xFFFF_FFFF, KakiRole::Zikru))
            .unwrap();
        let station = STATIONS[i % STATIONS.len()];
        let mut eav = vec![EavTriple::new(
            bahyway_crc::crc16("station".as_bytes()) as u32,
            codec::encode(&AkkValue::Text(station.into())),
        )];
        if i == n / 2 {
            // A globally-unique tag so the "needle" query below is a
            // genuine single-row lookup, not a partial match against a
            // broad category -- the demo-relevant case (find *this*
            // particle) as distinct from "list everything in category X".
            eav.push(EavTriple::new(
                bahyway_crc::crc16("needle_id".as_bytes()) as u32,
                codec::encode(&AkkValue::Text("needle-particle-42".into())),
            ));
        }
        jnl.append(enkidb_journal::entry::JournalEntry::new(
            ek,
            e.clone(),
            1,
            eav,
        ))
        .unwrap();
        if i == n / 2 {
            needle = Some(e);
        }
    }
    let needle = needle.expect("n must be > 0");
    let gen_elapsed = t0.elapsed();
    println!("generate {n} particles into Journal:  {gen_elapsed:?}");

    // ── Materialize: the one-time O(n) batch cost, paid once. ──
    let t1 = Instant::now();
    let stats = materialize(&jnl, &entities_base, &eav_base).unwrap();
    let materialize_elapsed = t1.elapsed();
    println!(
        "materialize {} entities, {} distinct EAV keys:  {materialize_elapsed:?}",
        stats.entities, stats.distinct_eav_keys
    );
    drop(jnl); // the Write Node's journal is gone; only Data Files remain.

    // ── Open: must be O(1) -- no replay, regardless of N. ──
    let t2 = Instant::now();
    let mut rn = ReadNode::open(&entities_base, &eav_base).unwrap();
    let open_elapsed = t2.elapsed();
    println!(
        "ReadNode::open (no journal replay):  {open_elapsed:?}  (entity_count={})",
        rn.entity_count()
    );

    // ── Query 1: unbounded broad match (~1/5 of the dataset) -- the
    // honest worst case: HeptaScript has to fetch and materialize every
    // one of those rows' full content, which is I/O bound by the *result
    // size*, not the corpus size. No amount of indexing removes this --
    // it's the same physics as any database asked to return millions of
    // rows. Kept here specifically so this stays measured, not assumed.
    let t3 = Instant::now();
    let broad = rn
        .query("WHO T.E\nWHERE E[station] = \"data-cleansing-station\"")
        .unwrap();
    let broad_elapsed = t3.elapsed();
    println!(
        "query WHERE station = 'data-cleansing-station' (UNBOUNDED)  ->  {} matched  in {broad_elapsed:?}",
        broad.matched.len()
    );

    // ── Query 2: the same broad match, but HOW_MUCH LIMIT-bounded -- the
    // pattern DubSar's Grid/Orbit view actually uses: a fast total match
    // count plus a bounded page of rows to render, not literally every
    // matching row. This is what "show it in <1s" should mean at any N.
    let t3b = Instant::now();
    let limited = rn
        .query("WHO T.E\nWHERE E[station] = \"data-cleansing-station\"\nHOW_MUCH LIMIT 1000")
        .unwrap();
    let limited_elapsed = t3b.elapsed();
    println!(
        "query WHERE station = 'data-cleansing-station' HOW_MUCH LIMIT 1000  ->  {} matched  in {limited_elapsed:?}",
        limited.matched.len()
    );

    // ── Query 3: single-entity needle-in-haystack -- the demo-relevant
    // case (find *this* particle out of N), not a category listing. ──
    let t4 = Instant::now();
    let needle_result = rn
        .query("WHO T.E\nWHERE E[needle_id] = \"needle-particle-42\"")
        .unwrap();
    let needle_elapsed = t4.elapsed();
    let found_needle = needle_result
        .matched
        .iter()
        .any(|me| *me.entity.bytes() == *needle.bytes());
    println!(
        "query WHERE needle_id = 'needle-particle-42' (1 match out of {n})  in {needle_elapsed:?}  needle_found={found_needle}"
    );

    println!("=== summary ===");
    println!(
        "N={n}  generate={gen_elapsed:?}  materialize={materialize_elapsed:?}  open={open_elapsed:?}  \
broad_unbounded({}_matches)={broad_elapsed:?}  broad_limit_1000({}_matches)={limited_elapsed:?}  \
needle_query(1_match)={needle_elapsed:?}",
        broad.matched.len(),
        limited.matched.len(),
    );

    std::fs::remove_dir_all(&base).ok();
}
