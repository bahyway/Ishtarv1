//! Musarû Emergency Scribe — generates KAKI-stamped emergency tablets.
//!
//! When Shedu detects an imminent node failure, Musarû scribes the context
//! into an EmergencyTablet — an immutable, KAKI-stamped record of the crisis.
//!
//! "Musarû" (Akkadian: "confirmation, inscribed tablet") — here it confirms
//! the failure context for the sovereign audit trail.

/// Severity of an emergency event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmergencyLevel {
    /// D7 integrity degrading — monitoring elevated.
    Advisory,
    /// D7 integrity in FUZZY zone — freeze warning imminent.
    Warning,
    /// D7 integrity in DEAD zone — Quantum Freeze triggered.
    Critical,
}

impl EmergencyLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Advisory => "ADVISORY",
            Self::Warning  => "WARNING",
            Self::Critical => "CRITICAL",
        }
    }
}

/// An immutable, KAKI-stamped emergency record produced by Musarû.
#[derive(Debug, Clone)]
pub struct EmergencyTablet {
    /// Human-readable failure context.
    pub context: String,
    /// Unix-epoch millisecond timestamp.
    pub timestamp_ms: u64,
    /// FNV-1a derived 16-byte KAKI stamp for the tablet.
    pub kaki_stamp: [u8; 16],
    /// Severity level.
    pub severity: EmergencyLevel,
    /// Tablet sequence number (monotonically increasing per session).
    pub sequence: u64,
}

impl EmergencyTablet {
    /// Render the tablet as a human-readable markdown block.
    pub fn to_markdown(&self) -> String {
        format!(
            "## 𒁾 Musarû Emergency Tablet #{}\n\
             - **Severity:** {}\n\
             - **Timestamp:** {} ms\n\
             - **Context:** {}\n\
             - **KAKI stamp:** {}\n",
            self.sequence,
            self.severity.label(),
            self.timestamp_ms,
            self.context,
            hex_kaki(&self.kaki_stamp),
        )
    }
}

/// Scribe an emergency tablet for a node failure event.
///
/// `context`      — human-readable description of the failure
/// `severity`     — emergency level
/// `timestamp_ms` — event timestamp
/// `sequence`     — monotonic sequence number for this session
pub fn emergency_scribe(
    context:      &str,
    severity:     EmergencyLevel,
    timestamp_ms: u64,
    sequence:     u64,
) -> EmergencyTablet {
    let kaki_stamp = derive_tablet_kaki(context, timestamp_ms, sequence);
    EmergencyTablet {
        context: context.to_string(),
        timestamp_ms,
        kaki_stamp,
        severity,
        sequence,
    }
}

/// Derive a 16-byte KAKI stamp from the tablet content.
fn derive_tablet_kaki(context: &str, timestamp_ms: u64, sequence: u64) -> [u8; 16] {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
    const FNV_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

    let mut hash = FNV_BASIS;
    for &b in context.as_bytes() {
        hash ^= b as u64;
        hash  = hash.wrapping_mul(FNV_PRIME);
    }
    for &b in &timestamp_ms.to_le_bytes() {
        hash ^= b as u64;
        hash  = hash.wrapping_mul(FNV_PRIME);
    }
    for &b in &sequence.to_le_bytes() {
        hash ^= b as u64;
        hash  = hash.wrapping_mul(FNV_PRIME);
    }
    let hash2 = hash.wrapping_mul(FNV_PRIME).wrapping_add(0xAB_CDEF_12);

    let mut kaki = [0u8; 16];
    kaki[..8].copy_from_slice(&hash.to_le_bytes());
    kaki[8..].copy_from_slice(&hash2.to_le_bytes());
    kaki[6] = 0x02; // EventKaki — emergency is a sovereign event
    kaki
}

fn hex_kaki(kaki: &[u8; 16]) -> String {
    kaki.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join("")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emergency_scribe_produces_tablet() {
        let t = emergency_scribe("Node 03 imminent collapse", EmergencyLevel::Critical, 9_000, 1);
        assert_eq!(t.severity, EmergencyLevel::Critical);
        assert_eq!(t.sequence, 1);
        assert!(t.context.contains("Node 03"));
    }

    #[test]
    fn kaki_stamp_type_byte_is_event() {
        let t = emergency_scribe("Test context", EmergencyLevel::Warning, 0, 0);
        assert_eq!(t.kaki_stamp[6], 0x02, "tablet KAKI must be EventKaki type");
    }

    #[test]
    fn kaki_stamp_deterministic() {
        let t1 = emergency_scribe("same", EmergencyLevel::Advisory, 100, 1);
        let t2 = emergency_scribe("same", EmergencyLevel::Advisory, 100, 1);
        assert_eq!(t1.kaki_stamp, t2.kaki_stamp);
    }

    #[test]
    fn different_contexts_different_kaki() {
        let t1 = emergency_scribe("context A", EmergencyLevel::Critical, 0, 0);
        let t2 = emergency_scribe("context B", EmergencyLevel::Critical, 0, 0);
        assert_ne!(t1.kaki_stamp, t2.kaki_stamp);
    }

    #[test]
    fn markdown_contains_severity_and_context() {
        let t  = emergency_scribe("GPU thermal critical", EmergencyLevel::Critical, 5_000, 3);
        let md = t.to_markdown();
        assert!(md.contains("CRITICAL"));
        assert!(md.contains("GPU thermal critical"));
        assert!(md.contains("Musarû Emergency Tablet #3"));
    }

    #[test]
    fn severity_ordering() {
        assert!(EmergencyLevel::Critical > EmergencyLevel::Warning);
        assert!(EmergencyLevel::Warning  > EmergencyLevel::Advisory);
    }
}
