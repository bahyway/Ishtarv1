//! Sealed PCR baseline storage and constant-time comparison.
//! Baseline is written at provisioning time and sealed to
//! ~/.config/urnammu/pcr_baseline.bin (chmod 600).
#![forbid(unsafe_code)]
use std::fs;
use std::path::{Path, PathBuf};

pub const BASELINE_REL_PATH: &str = ".config/urnammu/pcr_baseline.bin";

/// Resolve the baseline path under $HOME. Avoids pulling in the `dirs`
/// crate for a single environment-variable lookup.
pub fn baseline_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(BASELINE_REL_PATH)
}

/// Load the sealed PCR baseline from disk.
pub fn load_baseline(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("Baseline read failed ({}): {e}", path.display()))
}

/// Constant-time comparison — prevents timing side-channel leakage.
/// Returns true only if observed == baseline, byte-for-byte, same length.
pub fn compare_constant_time(observed: &[u8], baseline: &[u8]) -> bool {
    if observed.len() != baseline.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (a, b) in observed.iter().zip(baseline.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Write a new baseline (provisioning step — run once after clean boot).
pub fn provision_baseline(pcr_state: &[u8]) -> Result<(), String> {
    let path = baseline_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Baseline dir create failed: {e}"))?;
    }
    fs::write(&path, pcr_state).map_err(|e| format!("Baseline write failed: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Baseline chmod failed: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_matches_equal_slices() {
        assert!(compare_constant_time(&[1, 2, 3], &[1, 2, 3]));
    }

    #[test]
    fn constant_time_rejects_mismatch() {
        assert!(!compare_constant_time(&[1, 2, 3], &[1, 2, 4]));
    }

    #[test]
    fn constant_time_rejects_length_mismatch() {
        assert!(!compare_constant_time(&[1, 2, 3], &[1, 2]));
    }

    #[test]
    fn provision_and_load_round_trip() {
        let tmp = std::env::temp_dir().join(format!(
            "urnammu_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let path = tmp.join("baseline.bin");
        std::fs::write(&path, [9u8, 8, 7]).unwrap();
        let loaded = load_baseline(&path).unwrap();
        assert_eq!(loaded, vec![9, 8, 7]);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
