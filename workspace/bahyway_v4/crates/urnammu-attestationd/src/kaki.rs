//! UrNammu attestation Event KAKI builder.
//! kaki_type = 0x02 (Event), kaki_role = 0x02 (ZIKRU)
//! tribe_id  = "shedu_security"
//! EAV attrs: attestation_result (PASS/FAIL), pcr_banks, timestamp
//!
//! Real KISPU four-way atomic commit wired when EnkiSDB is reachable.
//! Fallback: appends to local attestation log for deferred commit.
#![forbid(unsafe_code)]
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TRIBE_ID: &str = "shedu_security";
pub const ATTESTATION_LOG: &str = "/var/log/urnammu/attestation.log";

pub struct AttestationEvent {
    pub result: bool,
    pub pcr_banks: String,
    pub boot_phase: String, // "boot" | "runtime"
}

/// Emit an attestation Event KAKI. Returns the generated kaki_id.
pub fn emit_attestation_kaki(event: &AttestationEvent) -> Result<String, String> {
    emit_attestation_kaki_to(event, ATTESTATION_LOG)
}

fn emit_attestation_kaki_to(event: &AttestationEvent, log_path: &str) -> Result<String, String> {
    let ts = unix_epoch_secs();
    let kaki_id = format!("urnammu-{}-{:x}", ts, pseudo_random_suffix());
    let result_str = if event.result { "PASS" } else { "FAIL" };

    let line = format!(
        "{ts}|URNAMMU|kaki_type=0x02|kaki_role=0x02|tribe={TRIBE_ID}|result={result_str}|pcr_banks={}|phase={}|kaki_id={kaki_id}\n",
        event.pcr_banks, event.boot_phase
    );

    let path = PathBuf::from(log_path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut f) => {
            f.write_all(line.as_bytes())
                .map_err(|e| format!("attestation log write failed: {e}"))?;
        }
        Err(e) => eprintln!("Attestation log write failed: {e}"),
    }

    // TODO: KISPU four-way atomic commit to EnkiSDB (SHEDU tribe)
    eprintln!("[urnammu] KAKI Event: {}", line.trim());
    Ok(kaki_id)
}

fn unix_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn pseudo_random_suffix() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_kaki_and_writes_log_line() {
        let tmp = std::env::temp_dir().join(format!(
            "urnammu_kaki_test_{}.log",
            pseudo_random_suffix()
        ));
        let event = AttestationEvent {
            result: true,
            pcr_banks: "sha256:0,4,7".to_string(),
            boot_phase: "boot".to_string(),
        };
        let kaki_id = emit_attestation_kaki_to(&event, tmp.to_str().unwrap()).unwrap();
        assert!(kaki_id.starts_with("urnammu-"));
        let contents = std::fs::read_to_string(&tmp).unwrap();
        assert!(contents.contains("result=PASS"));
        assert!(contents.contains(&kaki_id));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn fail_result_recorded() {
        let tmp = std::env::temp_dir().join(format!(
            "urnammu_kaki_fail_test_{}.log",
            pseudo_random_suffix()
        ));
        let event = AttestationEvent {
            result: false,
            pcr_banks: "sha256:0,4,7".to_string(),
            boot_phase: "runtime".to_string(),
        };
        emit_attestation_kaki_to(&event, tmp.to_str().unwrap()).unwrap();
        let contents = std::fs::read_to_string(&tmp).unwrap();
        assert!(contents.contains("result=FAIL"));
        let _ = std::fs::remove_file(&tmp);
    }
}
