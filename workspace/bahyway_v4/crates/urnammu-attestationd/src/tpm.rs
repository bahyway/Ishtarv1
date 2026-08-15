//! TPM2 PCR state reader.
//! Shells out to tpm2_pcrread (tpm2-tools) and parses the hex digest.
//! PCRs measured: 0 (firmware), 4 (boot loader), 7 (Secure Boot policy).
#![forbid(unsafe_code)]
use std::process::Command;

pub const PCR_BANKS: &[&str] = &["sha256:0,4,7"];

/// Read current PCR state via tpm2_pcrread.
/// Returns raw byte concatenation of PCR 0 + 4 + 7.
pub fn read_pcr_state() -> Result<Vec<u8>, String> {
    let output = Command::new("tpm2_pcrread")
        .args(PCR_BANKS)
        .output()
        .map_err(|e| format!("tpm2_pcrread exec failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "tpm2_pcrread exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    parse_pcr_output(&output.stdout)
}

/// Parse tpm2_pcrread YAML output, extract hex digests for PCR 0, 4, 7.
fn parse_pcr_output(raw: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(raw).map_err(|e| format!("tpm2_pcrread output not UTF-8: {e}"))?;
    let mut combined = Vec::with_capacity(96);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("0 :") || trimmed.starts_with("4 :") || trimmed.starts_with("7 :") {
            if let Some(hex_str) = trimmed.split("0x").nth(1) {
                let bytes = decode_hex(hex_str.trim())?;
                combined.extend_from_slice(&bytes);
            }
        }
    }
    if combined.is_empty() {
        return Err("No PCR values found in tpm2_pcrread output".to_string());
    }
    Ok(combined)
}

/// Minimal hex decoder — avoids pulling in the `hex` crate for one function.
fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err(format!("PCR hex decode error: odd-length string '{s}'"));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("PCR hex decode error: {e}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_pcr_line() {
        let raw = b"  0 : 0xAABBCCDD\n";
        let result = parse_pcr_output(raw).unwrap();
        assert_eq!(result, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn empty_output_is_error() {
        assert!(parse_pcr_output(b"").is_err());
    }

    #[test]
    fn decode_hex_round_trip() {
        assert_eq!(decode_hex("aabb").unwrap(), vec![0xAA, 0xBB]);
        assert!(decode_hex("abc").is_err());
    }
}
