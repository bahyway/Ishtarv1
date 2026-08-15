//! Pre-ingest security scan for markdown documents — the Architect's
//! "NergalEngine tests before EAV ingest" request.
//!
//! There is no single unified "NergalEngine" API anywhere in this
//! workspace (confirmed by direct code research): `dubsar-visualizer`'s
//! `panels::nergal` is a visualization panel only, reading counters
//! written elsewhere; `nergal-av`/`nergal-agent` are named by two ansible
//! playbooks that *generate* those crates on a remote host but neither
//! crate exists in this checkout. The real, working scanner underneath
//! the Nergal panel's data is a differently-named crate, `musaru-security`
//! — its `zip_scan` module does pure byte-signature pattern matching
//! (EICAR test vectors, PE headers, PHP/JS dropper probes), built for ZIP
//! bytes pre-extraction but equally valid against any raw bytes. This
//! module is a thin, enkiddb-scoped wrapper around that primitive, same
//! discipline as `authorship.rs`'s wrapper around `git log`: fail closed,
//! name exactly what was matched, never silently pass ambiguous content.
//!
//! Deliberately scoped to markdown/text bytes for now, not image files —
//! there is no image-decoding dependency anywhere in this workspace, and
//! EnkiDDB's real ingest path only ever reads `.md` files. "Unrecognized
//! hyperspectral image" has no real substrate to check against today; see
//! `crates/wpd-engine` (numeric reflectance-band telemetry, not image
//! files) if that capability is ever actually wanted.

use musaru_security::zip_scan;

/// Result of scanning one document's raw bytes for known malware
/// signatures before it is parsed or journaled.
#[derive(Debug, Clone)]
pub struct SecurityScanResult {
    pub clean: bool,
    pub detail: &'static str,
}

/// Scan raw document bytes for known malware/webshell/dropper signatures
/// (`musaru_security::zip_scan::MALWARE_SIGS`) — the same primitive
/// Musarû uses on ZIP bytes pre-extraction, applied here to markdown text
/// bytes pre-parse. A hit means the document must never reach
/// `DocumentParser` or the Journal.
pub fn scan_document(bytes: &[u8]) -> SecurityScanResult {
    let result = zip_scan::scan(bytes);
    SecurityScanResult { clean: !result.malware_detected, detail: result.detail }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_markdown_passes() {
        let r = scan_document(b"# Widget Guide\n\nBody text about widgets.\n");
        assert!(r.clean);
    }

    #[test]
    fn eicar_signature_is_caught() {
        let r = scan_document(b"# Doc\n\nEICAR-STANDARD-ANTIVIRUS-TEST-FILE\n");
        assert!(!r.clean);
        assert!(r.detail.contains("EICAR"));
    }

    #[test]
    fn php_webshell_probe_is_caught() {
        let r = scan_document(b"# Doc\n\n```\n<?php eval($_GET['x']); ?>\n```\n");
        assert!(!r.clean);
    }

    #[test]
    fn empty_document_is_clean() {
        assert!(scan_document(b"").clean);
    }
}
