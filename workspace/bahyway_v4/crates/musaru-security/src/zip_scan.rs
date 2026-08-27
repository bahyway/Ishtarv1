//! ZIP malware byte scan — pre-extraction sovereign security check.
//!
//! Applied to raw ZIP bytes BEFORE extraction so that no harmful content
//! ever touches the SDB file system.  Pure pattern matching; no third-party
//! AV library.  Signatures are byte-level patterns known to represent
//! malware probes or embedded executables.

/// Canonical malware byte signatures checked by Musarû.
/// New signatures must be appended (never reordered) to keep logs stable.
pub const MALWARE_SIGS: &[&[u8]] = &[
    b"EICAR-STANDARD-ANTIVIRUS-TEST-FILE", // 0 — EICAR test vector
    b"X5O!P%@AP",                          // 1 — EICAR prefix (partial match)
    b"\x4D\x5A\x90\x00",                   // 2 — PE MZ header embedded in ZIP
    b"<?php eval(",                        // 3 — PHP webshell probe
    b"<script>document.write(unescape(",   // 4 — JS dropper probe
];

/// Outcome of a ZIP pre-extraction scan.
#[derive(Debug, Clone)]
pub struct ZipScanResult {
    /// True if any malware signature was matched.
    pub malware_detected: bool,
    /// Index into MALWARE_SIGS of the first matched signature (if any).
    pub matched_sig_index: Option<usize>,
    /// Human-readable description of what was found.
    pub detail: &'static str,
}

impl ZipScanResult {
    pub fn clean() -> Self {
        ZipScanResult {
            malware_detected: false,
            matched_sig_index: None,
            detail: "clean",
        }
    }

    pub fn hit(idx: usize) -> Self {
        ZipScanResult {
            malware_detected: true,
            matched_sig_index: Some(idx),
            detail: MALWARE_DETAIL[idx.min(MALWARE_DETAIL.len() - 1)],
        }
    }
}

const MALWARE_DETAIL: &[&str] = &[
    "EICAR standard antivirus test file",
    "EICAR prefix detected",
    "PE executable (MZ header) embedded in ZIP",
    "PHP eval webshell probe",
    "JavaScript dropper probe",
];

/// Scan raw ZIP bytes for known malware signatures.
/// Returns on first match; call `scan_all` for exhaustive reporting.
pub fn scan(data: &[u8]) -> ZipScanResult {
    for (idx, sig) in MALWARE_SIGS.iter().enumerate() {
        if data.windows(sig.len()).any(|w| w == *sig) {
            return ZipScanResult::hit(idx);
        }
    }
    ZipScanResult::clean()
}

/// Scan and return results for ALL matched signatures.
pub fn scan_all(data: &[u8]) -> Vec<ZipScanResult> {
    MALWARE_SIGS
        .iter()
        .enumerate()
        .filter(|(_, sig)| data.windows(sig.len()).any(|w| w == **sig))
        .map(|(idx, _)| ZipScanResult::hit(idx))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_data_passes() {
        let result = scan(b"name,epoch,state\nAli,1,Golden\n");
        assert!(!result.malware_detected);
        assert_eq!(result.matched_sig_index, None);
    }

    #[test]
    fn eicar_detected() {
        let data = b"some prefix EICAR-STANDARD-ANTIVIRUS-TEST-FILE some suffix";
        let r = scan(data);
        assert!(r.malware_detected);
        assert_eq!(r.matched_sig_index, Some(0));
    }

    #[test]
    fn pe_mz_header_detected() {
        let mut data = vec![0u8; 16];
        data[4] = 0x4D;
        data[5] = 0x5A;
        data[6] = 0x90;
        data[7] = 0x00;
        let r = scan(&data);
        assert!(r.malware_detected);
        assert_eq!(r.matched_sig_index, Some(2));
    }

    #[test]
    fn scan_all_finds_multiple_sigs() {
        let mut data = b"EICAR-STANDARD-ANTIVIRUS-TEST-FILE".to_vec();
        data.extend_from_slice(b"\x4D\x5A\x90\x00");
        let results = scan_all(&data);
        assert!(results.len() >= 2);
        assert!(results.iter().all(|r| r.malware_detected));
    }

    #[test]
    fn empty_data_is_clean() {
        assert!(!scan(b"").malware_detected);
    }
}
