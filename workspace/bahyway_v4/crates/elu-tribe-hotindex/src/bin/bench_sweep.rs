//! BENCH — the sovereign throughput proof for elu-tribe-hotindex.
//! Usage: bench_sweep [num_particles] [threads]

use elu_tribe_hotindex::{validate_batch_parallel, HotIndex};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1_000_000_000);
    let threads: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(
        std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1),
    );

    // Registry with 1,024 live tribes, headroom over any realistic count.
    let table: Vec<(u16, u64)> = (1..=1024u16).map(|t| (t, 0x1000 + t as u64)).collect();
    let idx = HotIndex::from_snapshot(&table, 0xA9_0000_0000_0001, 0xABCD);
    println!(
        "HotIndex footprint: {} KB (validity 64 KB L1-class, nodes 512 KB L2-class)",
        idx.footprint_bytes() / 1024
    );

    print!("allocating {n} particles ({} MB)… ", n * 2 / (1024 * 1024));
    let t0 = Instant::now();
    let mut ids: Vec<u16> = Vec::with_capacity(n);
    for i in 0..n {
        let mut t = (i % 1024) as u16 + 1;
        if i % 10_000_000 == 5_000_000 {
            t = 60_000; // an id no epoch has blessed
        }
        ids.push(t);
    }
    println!("done in {:.2}s", t0.elapsed().as_secs_f64());

    // Warm one pass, then measure.
    let _ = validate_batch_parallel(&idx, &ids[..n.min(1 << 20)], threads);
    let t1 = Instant::now();
    let rep = validate_batch_parallel(&idx, &ids, threads);
    let dt = t1.elapsed().as_secs_f64();

    println!(
        "swept {} particles in {:.4}s on {} thread(s) → {:.2} B particles/s",
        rep.checked, dt, threads, rep.checked as f64 / dt / 1e9
    );
    println!(
        "valid={} invalid={} first_invalid={:?} epoch={:016x}",
        rep.valid, rep.invalid, rep.first_invalid, rep.epoch
    );
    println!(
        "LAW: 1B under 1s → {}",
        if rep.checked as f64 / dt >= 1e9 { "HOLDS on this hardware" } else { "NOT met on this hardware" }
    );
}
