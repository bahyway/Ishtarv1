//! LAW-HOT-2: the billion-particle sweep.
//!
//! The kernel is a branchless table-lookup accumulation over a columnar
//! tribe_id slice (2 bytes per particle instead of streaming full 16-byte
//! KAKI rows). Chunked so LLVM can unroll/vectorize; correctness never
//! depends on it doing so.

use crate::hot_table::HotIndex;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchReport {
    pub checked: usize,
    pub valid: usize,
    pub invalid: usize,
    /// Index of the first offending particle, when any (for barû triage).
    pub first_invalid: Option<usize>,
    /// LAW-HOT-3: the epoch this sweep was judged against.
    pub epoch: u64,
}

/// Single-thread kernel. Branchless in the hot loop; the first_invalid
/// scan runs only when the count says an offender exists.
pub fn validate_batch(idx: &HotIndex, tribe_ids: &[u16]) -> BatchReport {
    let plane = idx.validity_plane();
    let mut valid: u64 = 0;

    // 64-wide chunks: enough for the optimizer to unroll and, with a
    // gather-capable target, vectorize; remainder handled the same way.
    let mut chunks = tribe_ids.chunks_exact(64);
    for c in &mut chunks {
        let mut acc: u64 = 0;
        for &t in c {
            acc += plane[t as usize] as u64; // branchless accumulate
        }
        valid += acc;
    }
    for &t in chunks.remainder() {
        valid += plane[t as usize] as u64;
    }

    let checked = tribe_ids.len();
    let invalid = checked - valid as usize;
    let first_invalid = if invalid > 0 {
        tribe_ids.iter().position(|&t| plane[t as usize] == 0)
    } else {
        None
    };
    BatchReport { checked, valid: valid as usize, invalid, first_invalid, epoch: idx.epoch }
}

/// std::thread fan-out (sovereign law: stdlib threads, zero async runtime).
pub fn validate_batch_parallel(idx: &HotIndex, tribe_ids: &[u16], threads: usize) -> BatchReport {
    let n = threads.max(1);
    if n == 1 || tribe_ids.len() < 1 << 16 {
        return validate_batch(idx, tribe_ids);
    }
    let chunk = tribe_ids.len().div_ceil(n);
    let mut reports: Vec<BatchReport> = Vec::with_capacity(n);
    thread::scope(|s| {
        let handles: Vec<_> = tribe_ids
            .chunks(chunk)
            .map(|slice| s.spawn(move || validate_batch(idx, slice)))
            .collect();
        for h in handles {
            reports.push(h.join().expect("validator thread"));
        }
    });
    let mut out = BatchReport { checked: 0, valid: 0, invalid: 0, first_invalid: None, epoch: idx.epoch };
    let mut offset = 0usize;
    for r in reports {
        if out.first_invalid.is_none() {
            out.first_invalid = r.first_invalid.map(|i| i + offset);
        }
        out.checked += r.checked;
        out.valid += r.valid;
        out.invalid += r.invalid;
        offset += r.checked;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hot_table::HotIndex;

    fn idx() -> HotIndex {
        let table: Vec<(u16, u64)> = (1..=1000u16).map(|t| (t, 7000 + t as u64)).collect();
        HotIndex::from_snapshot(&table, 0xE7, 0x5EA1)
    }

    #[test]
    fn batch_counts_invalids_exactly_and_finds_first() {
        let i = idx();
        let mut ids: Vec<u16> = (0..200_000).map(|k| (k % 1000) as u16 + 1).collect();
        ids[137] = 60_000; // offender 1
        ids[99_999] = 0; // offender 2 (id 0 never allocated)
        let r = validate_batch(&i, &ids);
        assert_eq!(r.checked, 200_000);
        assert_eq!(r.invalid, 2);
        assert_eq!(r.valid, 199_998);
        assert_eq!(r.first_invalid, Some(137));
        assert_eq!(r.epoch, 0xE7); // LAW-HOT-3: the sweep cites its epoch
    }

    #[test]
    fn parallel_equals_serial_including_first_offender() {
        let i = idx();
        let mut ids: Vec<u16> = (0..3_000_000).map(|k| (k % 1000) as u16 + 1).collect();
        ids[2_500_001] = 60_000;
        let a = validate_batch(&i, &ids);
        for threads in [2, 3, 5, 8] {
            let b = validate_batch_parallel(&i, &ids, threads);
            assert_eq!(a, b, "threads={threads}");
        }
    }

    #[test]
    fn all_valid_and_all_invalid_edges() {
        let i = idx();
        let ok: Vec<u16> = vec![1; 100_001];
        let r = validate_batch(&i, &ok);
        assert_eq!((r.invalid, r.first_invalid), (0, None));
        let bad: Vec<u16> = vec![2000; 63];
        let r = validate_batch(&i, &bad); // remainder-only path (< one chunk)
        assert_eq!((r.valid, r.invalid, r.first_invalid), (0, 63, Some(0)));
    }

    #[test]
    fn empty_sweep_is_lawful() {
        let i = idx();
        let r = validate_batch(&i, &[]);
        assert_eq!((r.checked, r.valid, r.invalid, r.first_invalid), (0, 0, 0, None));
    }
}
