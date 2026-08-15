//! JournalEntry — one immutable Event-Kaki record in the Journal.
//!
//! Wire format:
//!   [0..16]  event_kaki      — 16-byte Event-Kaki bytes
//!   [16..32] target_kaki     — 16-byte Identity-Kaki of the target particle
//!   [32..36] epoch           — u32 big-endian journal epoch (partition key)
//!   [36..38] eav_count       — u16 big-endian number of EAV triples
//!   [38..40] entry_checksum  — CRC-16/CCITT over [0..38]
//!   [40..]   eav_payload     — eav_count EavTriple records (variable length)

use bahyway_crc::crc16;
use bahyway_core::{BahywayError, Result};
use enkidb_kaki::{EventKaki, IdentityKaki};
use crate::event_cause::EventCause;

// EAV attribute hashes mirrored from story-engine::projection (§4.2).
// Duplicated here so enkidb-journal has no dep on story-engine.
const ATTR_COLOR_ID_SNAPSHOT: u32 = 0x5F3A; // crc16("color_id_snapshot")
const ATTR_COLOR_DRIFT:        u32 = 0x6C2B; // crc16("color_drift")
const ATTR_EVENT_CAUSE:        u32 = 0x7D1C; // crc16("event_cause")
const ATTR_SOURCE_KAKI:        u32 = 0x8E0D; // crc16("source_kaki")

/// One EAV triple — (attribute_name_hash: u32, value_bytes: up to 256 bytes).
#[derive(Debug, Clone)]
pub struct EavTriple {
    /// 32-bit hash of the attribute name (fast lookup; full name in Dictionary).
    pub attr_hash: u32,
    /// Raw attribute value bytes (max 256 bytes, enough for any atomic value).
    pub value:     Vec<u8>,
}

impl EavTriple {
    pub fn new(attr_hash: u32, value: Vec<u8>) -> Self {
        EavTriple { attr_hash, value }
    }
}

/// An immutable Journal entry representing one committed Event-Kaki.
#[derive(Debug, Clone)]
pub struct JournalEntry {
    /// The Event-Kaki that was minted for this state transition.
    pub event_kaki:  EventKaki,
    /// The Identity-Kaki of the particle this event targets.
    pub target_kaki: IdentityKaki,
    /// Journal partition epoch (used for time-based partitioning per §9.1).
    pub epoch:       u32,
    /// The EAV triples carried by this event.
    pub eav:         Vec<EavTriple>,
}

impl JournalEntry {
    pub fn new(
        event_kaki:  EventKaki,
        target_kaki: IdentityKaki,
        epoch:       u32,
        eav:         Vec<EavTriple>,
    ) -> Self {
        JournalEntry { event_kaki, target_kaki, epoch, eav }
    }

    /// Serialize the fixed 40-byte header portion.
    pub fn header_bytes(&self) -> [u8; 40] {
        let mut buf = [0u8; 40];
        buf[0..16].copy_from_slice(self.event_kaki.bytes());
        buf[16..32].copy_from_slice(self.target_kaki.bytes());
        buf[32..36].copy_from_slice(&self.epoch.to_be_bytes());
        let count = self.eav.len().min(u16::MAX as usize) as u16;
        buf[36..38].copy_from_slice(&count.to_be_bytes());
        let cs = crc16(&buf[0..38]);
        buf[38..40].copy_from_slice(&cs.to_be_bytes());
        buf
    }

    /// Validate the header checksum.
    pub fn verify_header(&self) -> bool {
        let h  = self.header_bytes();
        let cs = u16::from_be_bytes([h[38], h[39]]);
        crc16(&h[0..38]) == cs
    }

    // ── Color snapshot helpers ─────────────────────────────────────────────

    /// Append a `color_id_snapshot` EAV triple ([R, G, B]) to this entry.
    /// Call after construction; does not touch the fixed wire header.
    pub fn with_color_snapshot(mut self, rgb: [u8; 3], drift: f32) -> Self {
        self.eav.push(EavTriple::new(ATTR_COLOR_ID_SNAPSHOT, rgb.to_vec()));
        self.eav.push(EavTriple::new(ATTR_COLOR_DRIFT, drift.to_be_bytes().to_vec()));
        self
    }

    /// Append an `event_cause` EAV triple to this entry.
    pub fn with_event_cause(mut self, cause: EventCause) -> Self {
        self.eav.push(EavTriple::new(ATTR_EVENT_CAUSE, vec![cause.to_byte()]));
        self
    }

    /// Append a `source_kaki` EAV triple (16 raw bytes of the originating KAKI).
    pub fn with_source_kaki(mut self, kaki_bytes: [u8; 16]) -> Self {
        self.eav.push(EavTriple::new(ATTR_SOURCE_KAKI, kaki_bytes.to_vec()));
        self
    }

    /// Read back the color_id_snapshot stored in EAV, if present.
    pub fn color_id_snapshot(&self) -> Option<[u8; 3]> {
        self.eav
            .iter()
            .find(|t| t.attr_hash == ATTR_COLOR_ID_SNAPSHOT)
            .and_then(|t| t.value.as_slice().try_into().ok())
    }

    /// Read back the color_drift stored in EAV, if present.
    pub fn color_drift(&self) -> Option<f32> {
        self.eav
            .iter()
            .find(|t| t.attr_hash == ATTR_COLOR_DRIFT)
            .and_then(|t| {
                let arr: [u8; 4] = t.value.as_slice().try_into().ok()?;
                Some(f32::from_be_bytes(arr))
            })
    }

    /// Read back the EventCause stored in EAV, if present.
    pub fn event_cause(&self) -> Option<EventCause> {
        self.eav
            .iter()
            .find(|t| t.attr_hash == ATTR_EVENT_CAUSE)
            .and_then(|t| t.value.first().copied())
            .and_then(EventCause::from_byte)
    }

    /// Reconstruct a `JournalEntry` from header bytes + pre-parsed EAV triples.
    pub fn from_header_bytes(
        raw: [u8; 40],
        eav: Vec<EavTriple>,
    ) -> Result<Self> {
        let stored_cs   = u16::from_be_bytes([raw[38], raw[39]]);
        let computed_cs = crc16(&raw[0..38]);
        if stored_cs != computed_cs {
            return Err(BahywayError::ChecksumMismatch {
                expected: computed_cs,
                actual:   stored_cs,
            });
        }
        let ek = EventKaki::try_from_kaki(
            enkidb_kaki::Kaki::from_bytes(raw[0..16].try_into().unwrap())?
        ).map_err(|_| BahywayError::InvalidKakiType(raw[6]))?;
        let tk = IdentityKaki::try_from_kaki(
            enkidb_kaki::Kaki::from_bytes(raw[16..32].try_into().unwrap())?
        ).map_err(|_| BahywayError::InvalidKakiType(raw[22]))?;
        let epoch = u32::from_be_bytes(raw[32..36].try_into().unwrap());
        Ok(JournalEntry { event_kaki: ek, target_kaki: tk, epoch, eav })
    }

    // ── Full entry (header + EAV) serialization ──────────────────────────────
    //
    // Same wire layout `enkidb-persist::disk_journal::serialize_entry`/
    // `parse_one` already use for the on-disk WAL (40-byte header + per-triple
    // `attr_hash[4] + value_len[1] + value[value_len]`), exposed here as a
    // standalone pair so any caller needing a self-contained JournalEntry
    // encoding -- not just the WAL's own append-only file -- can reuse the
    // same real, tested format instead of inventing another one. Unlike the
    // WAL's own `parse_one`, this does not expect or consume a trailing
    // COMMIT_MARKER byte: callers that need crash-safety framing (like the
    // WAL) add it themselves; callers whose transport already guarantees
    // frame integrity (like `enkidb-replication`'s KANĀKU-sealed frames)
    // don't need a second one.

    /// Serialize this entry's full header + EAV payload. No commit marker.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(40 + self.eav.len() * 6);
        out.extend_from_slice(&self.header_bytes());
        for triple in &self.eav {
            let vlen = triple.value.len().min(255) as u8;
            out.extend_from_slice(&triple.attr_hash.to_be_bytes());
            out.push(vlen);
            out.extend_from_slice(&triple.value[..vlen as usize]);
        }
        out
    }

    /// Parse one entry (header + EAV payload) starting at the front of
    /// `bytes`. Returns the entry and how many bytes it consumed, so a
    /// caller can slice past it if more data follows. `None` on any
    /// malformed/truncated/checksum-mismatched input.
    pub fn from_bytes(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 40 { return None; }
        let mut header = [0u8; 40];
        header.copy_from_slice(&bytes[0..40]);

        let stored_cs   = u16::from_be_bytes([header[38], header[39]]);
        let computed_cs = crc16(&header[0..38]);
        if stored_cs != computed_cs { return None; }

        let eav_count = u16::from_be_bytes([header[36], header[37]]) as usize;
        let mut cursor = 40usize;
        let mut eav = Vec::with_capacity(eav_count);
        for _ in 0..eav_count {
            if cursor + 5 > bytes.len() { return None; }
            let attr_hash = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().ok()?);
            let vlen      = bytes[cursor + 4] as usize;
            cursor       += 5;
            if cursor + vlen > bytes.len() { return None; }
            let value = bytes[cursor..cursor + vlen].to_vec();
            cursor   += vlen;
            eav.push(EavTriple::new(attr_hash, value));
        }

        let entry = Self::from_header_bytes(header, eav).ok()?;
        Some((entry, cursor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{KakiRole, mint::KakiMinter};
    use bahyway_core::TribeId;

    fn make_entry() -> JournalEntry {
        let m  = KakiMinter::new(TribeId::from_u16(0x0001));
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let ik = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let eav = vec![EavTriple::new(0xDEAD_BEEF, b"GOLDEN".to_vec())];
        JournalEntry::new(ek, ik, 42, eav)
    }

    #[test]
    fn header_checksum_valid() {
        let e = make_entry();
        assert!(e.verify_header());
    }

    #[test]
    fn round_trip_header() {
        let e   = make_entry();
        let raw = e.header_bytes();
        let e2  = JournalEntry::from_header_bytes(raw, e.eav.clone()).unwrap();
        assert_eq!(e.epoch, e2.epoch);
        assert_eq!(e.event_kaki, e2.event_kaki);
    }

    #[test]
    fn color_snapshot_round_trip() {
        let e = make_entry()
            .with_color_snapshot([255, 128, 64], 0.37)
            .with_event_cause(EventCause::SdbValidationPass);

        let rgb = e.color_id_snapshot().expect("snapshot missing");
        assert_eq!(rgb, [255, 128, 64]);

        let drift = e.color_drift().expect("drift missing");
        assert!((drift - 0.37).abs() < 1e-5, "drift mismatch: {drift}");

        let cause = e.event_cause().expect("cause missing");
        assert_eq!(cause, EventCause::SdbValidationPass);
    }

    #[test]
    fn header_consistent_after_color_eav() {
        // Adding color EAV triples updates eav_count + CRC; the header must
        // stay internally self-consistent (verify_header passes) and the
        // identity fields (KAKI bytes, epoch) must be unchanged.
        let e = make_entry();
        let ek_bytes = *e.event_kaki.bytes();
        let tk_bytes = *e.target_kaki.bytes();
        let epoch    = e.epoch;

        let colored = e
            .with_color_snapshot([10, 20, 30], 0.0)
            .with_event_cause(EventCause::KakiBorn);

        assert!(colored.verify_header(), "header checksum broken after EAV addition");
        let h = colored.header_bytes();
        assert_eq!(&h[0..16],  ek_bytes.as_ref());
        assert_eq!(&h[16..32], tk_bytes.as_ref());
        assert_eq!(u32::from_be_bytes(h[32..36].try_into().unwrap()), epoch);
        // eav_count should now be 1 (original) + 2 (snapshot) + 1 (cause) = 4
        assert_eq!(u16::from_be_bytes([h[36], h[37]]), 4);
    }

    #[test]
    fn to_bytes_from_bytes_round_trips_full_entry() {
        let e = make_entry();
        let bytes = e.to_bytes();
        let (parsed, consumed) = JournalEntry::from_bytes(&bytes).expect("should parse");

        assert_eq!(consumed, bytes.len());
        assert_eq!(parsed.epoch, e.epoch);
        assert_eq!(parsed.event_kaki.bytes(), e.event_kaki.bytes());
        assert_eq!(parsed.target_kaki.bytes(), e.target_kaki.bytes());
        assert_eq!(parsed.eav.len(), e.eav.len());
        assert_eq!(parsed.eav[0].attr_hash, e.eav[0].attr_hash);
        assert_eq!(parsed.eav[0].value, e.eav[0].value);
    }

    #[test]
    fn from_bytes_reports_bytes_consumed_for_trailing_data() {
        let e = make_entry();
        let mut bytes = e.to_bytes();
        let entry_len = bytes.len();
        bytes.extend_from_slice(b"trailing-junk");

        let (_parsed, consumed) = JournalEntry::from_bytes(&bytes).expect("should parse");
        assert_eq!(consumed, entry_len, "must not consume the trailing bytes");
    }

    #[test]
    fn from_bytes_rejects_truncated_input() {
        let e = make_entry();
        let bytes = e.to_bytes();
        assert!(JournalEntry::from_bytes(&bytes[..bytes.len() - 1]).is_none());
        assert!(JournalEntry::from_bytes(&bytes[..10]).is_none());
        assert!(JournalEntry::from_bytes(&[]).is_none());
    }

    #[test]
    fn from_bytes_rejects_corrupted_header_checksum() {
        let e = make_entry();
        let mut bytes = e.to_bytes();
        bytes[39] ^= 0xFF;
        assert!(JournalEntry::from_bytes(&bytes).is_none());
    }

    #[test]
    fn to_bytes_from_bytes_round_trips_empty_eav() {
        let m  = KakiMinter::new(TribeId::from_u16(0x0003));
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let ik = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let e  = JournalEntry::new(ek, ik, 7, vec![]);

        let bytes = e.to_bytes();
        let (parsed, consumed) = JournalEntry::from_bytes(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(parsed.eav.len(), 0);
    }
}
