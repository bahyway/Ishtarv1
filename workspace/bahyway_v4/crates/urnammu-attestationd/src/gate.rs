//! Boot handoff gate — writes a sentinel file that the display manager
//! reads before starting the session.
//! PASS writes /run/urnammu_attestation_result = "PASS"
//! FAIL writes "FAIL" and the display manager service fails.
#![forbid(unsafe_code)]
use std::fs;

pub const SENTINEL_PATH: &str = "/run/urnammu_attestation_result";

pub fn gate_boot_handoff(attestation_passed: bool) -> Result<(), String> {
    gate_boot_handoff_to(attestation_passed, SENTINEL_PATH)
}

fn gate_boot_handoff_to(attestation_passed: bool, path: &str) -> Result<(), String> {
    let result = if attestation_passed {
        "PASS\n"
    } else {
        "FAIL\n"
    };
    fs::write(path, result).map_err(|e| format!("Gate sentinel write failed ({path}): {e}"))?;
    if !attestation_passed {
        eprintln!("[urnammu] BOOT GATE: attestation FAILED — sentinel written FAIL, display manager will not start");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_writes_pass_sentinel() {
        let tmp = std::env::temp_dir().join("urnammu_gate_pass_test");
        gate_boot_handoff_to(true, tmp.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(&tmp).unwrap(), "PASS\n");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn fail_writes_fail_sentinel() {
        let tmp = std::env::temp_dir().join("urnammu_gate_fail_test");
        gate_boot_handoff_to(false, tmp.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(&tmp).unwrap(), "FAIL\n");
        let _ = std::fs::remove_file(&tmp);
    }
}
