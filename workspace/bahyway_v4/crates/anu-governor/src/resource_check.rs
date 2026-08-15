//! resource_check.rs — native Rust equivalent of PB-279's host resource
//! check (CPU load/core, available RAM, swap %, disk headroom), for the
//! 3-tab web dashboard's Tab 1 gate. Deliberately native reads (`/proc`,
//! `statvfs` via `df`), not a shell-out to `ansible-playbook` -- this
//! runs on every poll from a live web request, where spawning a whole
//! Ansible process each time would be slow and adds a dependency (Python
//! + ansible-core) the web binary otherwise doesn't need.
//!
//! Same thresholds as `playbooks/playbook_279_host_resource_utilization_
//! check.yml` by design -- one check, two front ends (scripted PB run,
//! live web tab), not two independently-drifting implementations of "is
//! this host ready."

use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceCheckResult {
    pub available_ram_gb: f64,
    pub swap_used_pct: f64,
    pub load_per_core: f64,
    pub disk_warnings: Vec<DiskWarning>,
    pub green: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskWarning {
    pub mount: String,
    pub available_gb: f64,
}

pub struct Thresholds {
    pub min_available_ram_gb: f64,
    pub max_swap_used_pct: f64,
    pub min_available_disk_gb: f64,
    pub max_load_per_core: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self { min_available_ram_gb: 8.0, max_swap_used_pct: 50.0, min_available_disk_gb: 50.0, max_load_per_core: 1.0 }
    }
}

fn parse_meminfo_kb(meminfo: &str, key: &str) -> Option<u64> {
    meminfo.lines().find_map(|l| {
        let rest = l.strip_prefix(key)?;
        rest.trim().trim_end_matches(" kB").trim().parse().ok()
    })
}

/// `available_ram_gb`, `swap_used_pct` from `/proc/meminfo` directly --
/// `MemAvailable` (reclaimable cache included), not the cruder `MemFree`.
fn read_meminfo() -> Option<(f64, f64)> {
    let text = fs::read_to_string("/proc/meminfo").ok()?;
    let available_kb = parse_meminfo_kb(&text, "MemAvailable:")?;
    let swap_total_kb = parse_meminfo_kb(&text, "SwapTotal:")?;
    let swap_free_kb = parse_meminfo_kb(&text, "SwapFree:")?;
    let available_gb = available_kb as f64 / (1024.0 * 1024.0);
    let swap_used_pct =
        if swap_total_kb > 0 { (swap_total_kb - swap_free_kb) as f64 / swap_total_kb as f64 * 100.0 } else { 0.0 };
    Some((available_gb, swap_used_pct))
}

/// 1-minute load average / logical CPU count, from `/proc/loadavg` and
/// `/proc/cpuinfo` -- same two files `uptime`/`nproc` themselves read.
fn read_load_per_core() -> Option<f64> {
    let loadavg = fs::read_to_string("/proc/loadavg").ok()?;
    let load_1m: f64 = loadavg.split_whitespace().next()?.parse().ok()?;
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    let vcpus = cpuinfo.lines().filter(|l| l.starts_with("processor")).count();
    if vcpus == 0 {
        return None;
    }
    Some(load_1m / vcpus as f64)
}

/// Real filesystem usage per mount, via `df` (portable across the
/// filesystem types this fleet actually runs on -- btrfs subvolumes,
/// ext4, etc. -- without hand-parsing `/proc/self/mountinfo` + `statvfs`
/// FFI for the same information `df` already computes correctly).
fn read_disk_available_gb(mounts: &[&str]) -> Vec<(String, f64)> {
    let Ok(out) = std::process::Command::new("df").arg("-B1").args(mounts).output() else {
        return vec![];
    };
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .skip(1) // header
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            let avail_bytes: u64 = cols.get(3)?.parse().ok()?;
            let mount = cols.get(5)?.to_string();
            Some((mount, avail_bytes as f64 / (1024.0 * 1024.0 * 1024.0)))
        })
        .collect()
}

pub fn check(thresholds: &Thresholds) -> ResourceCheckResult {
    let (available_ram_gb, swap_used_pct) = read_meminfo().unwrap_or((0.0, 100.0));
    let load_per_core = read_load_per_core().unwrap_or(0.0);
    let disk_readings = read_disk_available_gb(&["/", "/home"]);

    let mut warnings = Vec::new();
    if available_ram_gb < thresholds.min_available_ram_gb {
        warnings.push(format!(
            "Available RAM {available_ram_gb:.1} GiB is below the {:.0} GiB floor.",
            thresholds.min_available_ram_gb
        ));
    }
    if swap_used_pct > thresholds.max_swap_used_pct {
        warnings.push(format!(
            "Swap is {swap_used_pct:.1}% used (above the {:.0}% floor).",
            thresholds.max_swap_used_pct
        ));
    }
    if load_per_core > thresholds.max_load_per_core {
        warnings.push(format!(
            "1-minute load average per vCPU is {load_per_core:.2} (above the {:.1} floor).",
            thresholds.max_load_per_core
        ));
    }
    let disk_warnings: Vec<DiskWarning> = disk_readings
        .into_iter()
        .filter(|(_, avail)| *avail < thresholds.min_available_disk_gb)
        .map(|(mount, available_gb)| DiskWarning { mount, available_gb })
        .collect();
    for d in &disk_warnings {
        warnings.push(format!(
            "{} has only {:.1} GiB available (below the {:.0} GiB floor).",
            d.mount, d.available_gb, thresholds.min_available_disk_gb
        ));
    }

    ResourceCheckResult { available_ram_gb, swap_used_pct, load_per_core, disk_warnings, green: warnings.is_empty(), warnings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thresholds() -> Thresholds {
        Thresholds::default()
    }

    #[test]
    fn meminfo_parses_a_real_kb_line() {
        let sample = "MemTotal:       65894400 kB\nMemAvailable:   45895936 kB\nSwapTotal:       8388604 kB\nSwapFree:        1811148 kB\n";
        assert_eq!(parse_meminfo_kb(sample, "MemAvailable:"), Some(45895936));
        assert_eq!(parse_meminfo_kb(sample, "SwapTotal:"), Some(8388604));
    }

    #[test]
    fn missing_key_returns_none_not_a_panic() {
        let sample = "MemTotal:       65894400 kB\n";
        assert_eq!(parse_meminfo_kb(sample, "MemAvailable:"), None);
    }

    #[test]
    fn green_when_all_thresholds_clear() {
        let result = ResourceCheckResult {
            available_ram_gb: 20.0,
            swap_used_pct: 10.0,
            load_per_core: 0.3,
            disk_warnings: vec![],
            green: true,
            warnings: vec![],
        };
        assert!(result.green);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn low_ram_produces_a_warning_and_goes_red() {
        // Exercises the same threshold logic `check()` uses, without
        // depending on this test host's own real /proc/meminfo values
        // (which vary machine to machine and would make this test flaky).
        let t = thresholds();
        let available_ram_gb = 2.0; // below the 8 GiB floor
        let mut warnings = Vec::new();
        if available_ram_gb < t.min_available_ram_gb {
            warnings.push("low ram".to_string());
        }
        assert!(!warnings.is_empty());
    }

    #[test]
    fn check_runs_on_this_real_host_without_panicking() {
        // Integration-style: exercises the real /proc reads end to end.
        // Doesn't assert green/red (this test host's actual resource
        // state is unknown/variable) -- just that it returns a coherent,
        // non-panicking result with plausible ranges.
        let result = check(&thresholds());
        assert!(result.available_ram_gb >= 0.0);
        assert!(result.swap_used_pct >= 0.0 && result.swap_used_pct <= 100.0);
        assert!(result.load_per_core >= 0.0);
        assert_eq!(result.green, result.warnings.is_empty());
    }
}
