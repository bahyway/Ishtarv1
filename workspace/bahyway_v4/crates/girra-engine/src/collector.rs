//! Native host collector -- parses /proc directly with std only. Full
//! sovereignty: no sysinfo crate, no procfs crate.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

/// Returns (idle, total) jiffies from /proc/stat's first cpu line.
pub fn cpu_jiffies() -> Option<(u64, u64)> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;
    let vals: Vec<u64> = line.split_whitespace().skip(1).filter_map(|v| v.parse().ok()).collect();
    if vals.len() < 5 {
        return None;
    }
    let idle = vals[3] + vals.get(4).copied().unwrap_or(0);
    let total: u64 = vals.iter().sum();
    Some((idle, total))
}

/// CPU load fraction between two jiffy snapshots.
pub fn cpu_load(prev: (u64, u64), cur: (u64, u64)) -> f64 {
    let dt = cur.1.saturating_sub(prev.1);
    if dt == 0 {
        return 0.0;
    }
    let didle = cur.0.saturating_sub(prev.0);
    1.0 - (didle as f64 / dt as f64)
}

/// Memory used fraction from /proc/meminfo (MemAvailable-based).
pub fn mem_used_fraction() -> Option<f64> {
    let info = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0u64;
    let mut avail = 0u64;
    for line in info.lines() {
        let mut it = line.split_whitespace();
        match it.next()? {
            "MemTotal:" => total = it.next()?.parse().ok()?,
            "MemAvailable:" => avail = it.next()?.parse().ok()?,
            _ => {}
        }
    }
    if total == 0 {
        None
    } else {
        Some(1.0 - avail as f64 / total as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_load_full_idle_is_zero() {
        assert_eq!(cpu_load((0, 100), (100, 200)), 0.0);
    }

    #[test]
    fn cpu_load_no_idle_is_one() {
        assert_eq!(cpu_load((0, 0), (0, 100)), 1.0);
    }

    #[test]
    fn cpu_load_no_elapsed_time_is_safe_zero() {
        assert_eq!(cpu_load((10, 100), (10, 100)), 0.0);
    }

    #[test]
    fn real_proc_stat_is_readable_on_linux() {
        // Sanity check against this sandbox's real /proc -- honest, not
        // mocked: fails loudly if the parser can't read the real file.
        assert!(cpu_jiffies().is_some());
    }

    #[test]
    fn real_proc_meminfo_is_readable_on_linux() {
        let frac = mem_used_fraction();
        assert!(frac.is_some());
        let frac = frac.unwrap();
        assert!((0.0..=1.0).contains(&frac), "frac={frac}");
    }
}
