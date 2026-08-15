//! enkidb-seed — bulk particle seeder for enkidb-query-server's PersistedDb
//! store, built to exercise E-004/E-005 (docs/17_troubleshooting/TESTING_PLAYBOOK_PHASE1.md) at
//! real 10M-particle scale. Writes directly through the same
//! `enkidb_persist::persisted_db::PersistedDb` API and on-disk layout
//! (`{data_dir}/tribe-{tribe_id:04x}/journal.bin`) enkidb-query-server reads
//! at startup — not the incompatible enkidb-readnode/Data-Files stack
//! `scale_benchmark.rs` targets, and not enkidb-write-server's separate
//! `DATA_DIR/current` format either. Three parallel storage stacks exist in
//! this workspace; this tool writes the one the real query server serves.
//!
//! Every particle gets one EAV attribute, `seed_idx` (its ordinal), so
//! WHERE clauses have something real to filter on. Every `--shard-every`'th
//! particle additionally gets `shard` = `--shard-value` (default `"e004"`)
//! — a small, exact-match-indexable subset that a selective HeptaScript
//! query like `WHERE E[shard] = "e004"` can prune to directly via
//! `heptascript::indexed::HeptaIndexes`'s `EavExactIndex`, without touching
//! the other 9,999,000 particles. Attribute hashing uses the exact same
//! `bahyway_crc::crc16` scheme `heptascript::engine::attr_hash` uses
//! internally, so triples written here resolve correctly under a real
//! HeptaScript WHERE clause.
#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use akkvalue::{codec, AkkValue};
use bahyway_core::TribeId;
use enkidb_journal::entry::EavTriple;
use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
use enkidb_persist::persisted_db::PersistedDb;
use enkidb_storage::FsyncPolicy;

/// Mirrors the private `heptascript::engine::attr_hash` scheme exactly —
/// `crc16(name) as u32` — so an EAV triple written here and a HeptaScript
/// `WHERE E[name] = ...` condition hash the same attribute name identically.
fn attr_hash(name: &str) -> u32 {
    bahyway_crc::crc16(name.as_bytes()) as u32
}

struct Args {
    data_dir: PathBuf,
    tribe_id: u16,
    count: u64,
    shard_every: u64,
    shard_value: String,
    progress_every: u64,
}

fn parse_args() -> Args {
    // Matches enkidb-query-server's own `ENKIDB_QUERY_DATA_DIR` override, so
    // a test harness can point both the seeder and the server at the same
    // scratch directory without passing --data-dir to each separately.
    let mut data_dir = PathBuf::from(
        env::var("ENKIDB_QUERY_DATA_DIR").unwrap_or_else(|_| "/home/bahyway/enkidb/data".to_string()),
    );
    let mut tribe_id: u16 = 0x0001;
    let mut count: u64 = 10_000_000;
    let mut shard_every: u64 = 10_000;
    let mut shard_value = "e004".to_string();
    let mut progress_every: u64 = 200_000;

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = PathBuf::from(it.next().unwrap_or_else(|| fail("--data-dir needs a value")))
            }
            "--tribe" => {
                tribe_id = it
                    .next()
                    .unwrap_or_else(|| fail("--tribe needs a value"))
                    .parse()
                    .unwrap_or_else(|_| fail("--tribe must be a u16"))
            }
            "--count" => {
                count = it
                    .next()
                    .unwrap_or_else(|| fail("--count needs a value"))
                    .parse()
                    .unwrap_or_else(|_| fail("--count must be an integer"))
            }
            "--shard-every" => {
                shard_every = it
                    .next()
                    .unwrap_or_else(|| fail("--shard-every needs a value"))
                    .parse()
                    .unwrap_or_else(|_| fail("--shard-every must be an integer"))
            }
            "--shard-value" => {
                shard_value = it.next().unwrap_or_else(|| fail("--shard-value needs a value"))
            }
            "--progress-every" => {
                progress_every = it
                    .next()
                    .unwrap_or_else(|| fail("--progress-every needs a value"))
                    .parse()
                    .unwrap_or_else(|_| fail("--progress-every must be an integer"))
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => fail(&format!("unknown argument: {other}")),
        };
    }

    if shard_every == 0 {
        fail("--shard-every must be >= 1");
    }

    Args { data_dir, tribe_id, count, shard_every, shard_value, progress_every }
}

fn fail(msg: &str) -> ! {
    eprintln!("enkidb-seed: {msg}");
    print_usage();
    std::process::exit(1);
}

fn print_usage() {
    eprintln!(
        "enkidb-seed [--data-dir PATH] [--tribe N] [--count N] [--shard-every N] \
         [--shard-value STR] [--progress-every N]\n\
         Defaults match enkidb-query-server: --data-dir /home/bahyway/enkidb/data \
         --tribe 1 --count 10000000 --shard-every 10000 --shard-value e004"
    );
}

fn main() {
    let args = parse_args();

    println!(
        "𒁾 enkidb-seed — writing {} particles to {} (tribe {:#06x}), 1 in {} tagged shard={:?}",
        args.count,
        args.data_dir.display(),
        args.tribe_id,
        args.shard_every,
        args.shard_value
    );

    let tribe_id = TribeId::from_u16(args.tribe_id);
    let m = KakiMinter::new(tribe_id);

    // Batched fsync: real durability (a crash mid-seed doesn't lose
    // everything already committed) without paying a full fsync per
    // particle, which would make a 10M-particle seed impractically slow.
    let mut db = PersistedDb::open(
        &args.data_dir,
        tribe_id,
        FsyncPolicy::Batched { window: Duration::from_millis(250) },
    )
    .unwrap_or_else(|e| {
        eprintln!("FATAL: open {}: {e}", args.data_dir.display());
        std::process::exit(1);
    });

    let seed_idx_hash = attr_hash("seed_idx");
    let shard_hash = attr_hash("shard");
    let shard_value_encoded = codec::encode(&AkkValue::Text(args.shard_value.clone()));

    let t0 = Instant::now();
    let mut shard_count: u64 = 0;

    for i in 0..args.count {
        let identity = IdentityKaki::try_from_kaki(m.mint_identity(i as u32, KakiRole::Zikru))
            .unwrap_or_else(|e| {
                eprintln!("FATAL: mint_identity at i={i}: {e}");
                std::process::exit(1);
            });
        db.register_particle(&identity).unwrap_or_else(|e| {
            eprintln!("FATAL: register_particle at i={i}: {e}");
            std::process::exit(1);
        });

        let mut eav = vec![EavTriple::new(seed_idx_hash, codec::encode(&AkkValue::Int(i as i64)))];
        if i % args.shard_every == 0 {
            eav.push(EavTriple::new(shard_hash, shard_value_encoded.clone()));
            shard_count += 1;
        }

        let event = EventKaki::try_from_kaki(m.mint_event(i as u32, KakiRole::Zikru))
            .unwrap_or_else(|e| {
                eprintln!("FATAL: mint_event at i={i}: {e}");
                std::process::exit(1);
            });
        db.commit(event, identity, 1, eav).unwrap_or_else(|e| {
            eprintln!("FATAL: commit at i={i}: {e}");
            std::process::exit(1);
        });

        if args.progress_every > 0 && (i + 1) % args.progress_every == 0 {
            let elapsed = t0.elapsed().as_secs_f64();
            let rate = (i + 1) as f64 / elapsed.max(0.001);
            println!(
                "  {} / {} written ({rate:.0}/s, {elapsed:.1}s elapsed, {shard_count} shard-tagged so far)",
                i + 1,
                args.count
            );
        }
    }

    db.sync().unwrap_or_else(|e| {
        eprintln!("FATAL: final sync: {e}");
        std::process::exit(1);
    });

    let elapsed = t0.elapsed().as_secs_f64();
    println!(
        "✓ done: {} particles written in {elapsed:.1}s ({:.0}/s), {shard_count} tagged shard={:?} for selective queries",
        args.count,
        args.count as f64 / elapsed.max(0.001),
        args.shard_value
    );
    println!("  entries_written this run: {}", db.stats().entries_written);
}
