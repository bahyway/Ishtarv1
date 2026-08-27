//! disk_journal — binary serialization and crash-safe deserialization of JournalEntry.
//!
//! On-disk layout per entry:
//!   [0..40]  fixed header  (event_kaki[16] + target_kaki[16] + epoch[4]
//!                           + eav_count[2] + CRC-16[2])
//!   [40..]   EAV triples   for each triple:
//!              attr_hash[4] + value_len[1] + value[value_len]
//!   [last]   COMMIT_MARKER (0xCA) — written by AppendWriter::append_committed
//!
//! An entry without a trailing COMMIT_MARKER is treated as a partial write and
//! discarded — the journal is truncated at the last fully-committed entry.

use std::path::Path;

use bahyway_core::Result;
use bahyway_crc::crc16;
use enkidb_journal::entry::{EavTriple, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki};
use enkidb_storage::COMMIT_MARKER;

// ── Serialise ─────────────────────────────────────────────────────────────────

/// Serialise a `JournalEntry` into bytes ready to be passed to
/// `AppendWriter::append_committed` (which appends the COMMIT_MARKER itself).
pub fn serialize_entry(entry: &JournalEntry) -> Vec<u8> {
    let mut out = Vec::with_capacity(40 + entry.eav.len() * 6);
    out.extend_from_slice(&entry.header_bytes());
    for triple in &entry.eav {
        let vlen = triple.value.len().min(255) as u8;
        out.extend_from_slice(&triple.attr_hash.to_be_bytes());
        out.push(vlen);
        out.extend_from_slice(&triple.value[..vlen as usize]);
    }
    out
}

// ── Deserialise ───────────────────────────────────────────────────────────────

/// Load every fully-committed `JournalEntry` from a journal file.
/// Partially-written entries (missing COMMIT_MARKER) are silently skipped —
/// this is the crash-recovery path (§11.5).
pub fn load_entries(path: &Path) -> Result<Vec<JournalEntry>> {
    let data = std::fs::read(path)?;
    let mut entries = Vec::new();
    let mut pos = 0usize;

    while pos < data.len() {
        match parse_one(&data, pos) {
            Some((entry, next_pos)) => {
                entries.push(entry);
                pos = next_pos;
            }
            None => break, // partial write at EOF — stop here
        }
    }
    Ok(entries)
}

/// Try to parse one entry starting at `pos`.  Returns (entry, next_pos) or None.
fn parse_one(data: &[u8], pos: usize) -> Option<(JournalEntry, usize)> {
    // Need at least the 40-byte header
    if pos + 40 > data.len() {
        return None;
    }

    let mut header = [0u8; 40];
    header.copy_from_slice(&data[pos..pos + 40]);

    // Validate header CRC
    let stored_cs = u16::from_be_bytes([header[38], header[39]]);
    let computed_cs = crc16(&header[0..38]);
    if stored_cs != computed_cs {
        return None;
    }

    let eav_count = u16::from_be_bytes([header[36], header[37]]) as usize;
    let mut cursor = pos + 40;

    // Parse EAV triples
    let mut eav = Vec::with_capacity(eav_count);
    for _ in 0..eav_count {
        if cursor + 5 > data.len() {
            return None;
        }
        let attr_hash = u32::from_be_bytes(data[cursor..cursor + 4].try_into().ok()?);
        let vlen = data[cursor + 4] as usize;
        cursor += 5;
        if cursor + vlen > data.len() {
            return None;
        }
        let value = data[cursor..cursor + vlen].to_vec();
        cursor += vlen;
        eav.push(EavTriple::new(attr_hash, value));
    }

    // Check commit marker
    if cursor >= data.len() || data[cursor] != COMMIT_MARKER {
        return None;
    }
    cursor += 1;

    // Reconstruct KAKIs
    let event_kaki =
        EventKaki::try_from_kaki(Kaki::from_bytes(header[0..16].try_into().ok()?).ok()?).ok()?;
    let target_kaki =
        IdentityKaki::try_from_kaki(Kaki::from_bytes(header[16..32].try_into().ok()?).ok()?)
            .ok()?;
    let epoch = u32::from_be_bytes(header[32..36].try_into().ok()?);

    Some((
        JournalEntry::new(event_kaki, target_kaki, epoch, eav),
        cursor,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{KakiMinter, KakiRole};

    fn sample_entry() -> JournalEntry {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let ik = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        JournalEntry::new(
            ek,
            ik,
            42,
            vec![
                EavTriple::new(0x1A4B, b"Golden".to_vec()),
                EavTriple::new(0x4E89, b"\xFF\xFF".to_vec()),
            ],
        )
    }

    #[test]
    fn round_trip_single_entry() {
        let entry = sample_entry();
        let mut bytes = serialize_entry(&entry);
        bytes.push(COMMIT_MARKER); // simulate AppendWriter

        let parsed = parse_one(&bytes, 0).expect("should parse");
        assert_eq!(parsed.0.epoch, 42);
        assert_eq!(parsed.0.eav.len(), 2);
        assert_eq!(parsed.0.eav[0].attr_hash, 0x1A4B);
        assert_eq!(parsed.0.eav[1].attr_hash, 0x4E89);
        assert_eq!(parsed.1, bytes.len());
    }

    #[test]
    fn partial_write_returns_none() {
        let entry = sample_entry();
        let bytes = serialize_entry(&entry);
        // No COMMIT_MARKER — simulates crash mid-write
        assert!(parse_one(&bytes, 0).is_none());
    }

    #[test]
    fn corrupted_header_crc_returns_none() {
        let entry = sample_entry();
        let mut bytes = serialize_entry(&entry);
        bytes.push(COMMIT_MARKER);
        bytes[39] ^= 0xFF; // corrupt CRC
        assert!(parse_one(&bytes, 0).is_none());
    }

    #[test]
    fn empty_eav_round_trips() {
        let m = KakiMinter::new(TribeId::from_u16(0x0002));
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let ik = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let e = JournalEntry::new(ek, ik, 0, vec![]);
        let mut b = serialize_entry(&e);
        b.push(COMMIT_MARKER);
        let (parsed, _) = parse_one(&b, 0).unwrap();
        assert_eq!(parsed.eav.len(), 0);
    }
}
