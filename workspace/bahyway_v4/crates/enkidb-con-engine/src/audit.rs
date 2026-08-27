#![forbid(unsafe_code)]
//! NĀRU WAL journal — 64-byte fixed-size binary entries with CRC-16.

use crate::error::ConError;
use std::time::{SystemTime, UNIX_EPOCH};

// Entry layout (64 bytes):
// [0..4]   entry_seq: u32 LE
// [4..8]   timestamp_epoch: u32 LE
// [8..12]  tribe_id: u32 LE
// [12..16] operation_code: u32 LE
// [16..48] surrogate_kaki: [u8; 32]
// [48..62] reserved: [u8; 14]
// [62..64] crc16: u16 LE (CRC-16 of bytes [0..62])

pub struct NaruEntry {
    entry_seq: u32,
    timestamp_epoch: u32,
    tribe_id: u32,
    operation_code: u32,
    surrogate_kaki: [u8; 32],
    reserved: [u8; 14],
}

impl NaruEntry {
    pub fn new(seq: u32, tribe_id: u32, op_code: u32) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);
        Self {
            entry_seq: seq,
            timestamp_epoch: ts,
            tribe_id,
            operation_code: op_code,
            surrogate_kaki: [0u8; 32],
            reserved: [0u8; 14],
        }
    }

    pub fn serialize(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        buf[0..4].copy_from_slice(&self.entry_seq.to_le_bytes());
        buf[4..8].copy_from_slice(&self.timestamp_epoch.to_le_bytes());
        buf[8..12].copy_from_slice(&self.tribe_id.to_le_bytes());
        buf[12..16].copy_from_slice(&self.operation_code.to_le_bytes());
        buf[16..48].copy_from_slice(&self.surrogate_kaki);
        buf[48..62].copy_from_slice(&self.reserved);
        let crc = bahyway_crc::crc16(&buf[0..62]);
        buf[62..64].copy_from_slice(&crc.to_le_bytes());
        buf
    }

    pub fn verify(bytes: &[u8; 64]) -> bool {
        let stored_crc = u16::from_le_bytes([bytes[62], bytes[63]]);
        let computed = bahyway_crc::crc16(&bytes[0..62]);
        computed == stored_crc
    }
}

pub struct NaruJournal {
    entries: Vec<NaruEntry>,
    max_entries: usize,
}

impl NaruJournal {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn append(&mut self, tribe_id: u32, op_code: u32) -> Result<(), ConError> {
        if self.entries.len() >= self.max_entries {
            return Err(ConError::AuditJournalFull);
        }
        let seq = self.entries.len() as u32;
        self.entries.push(NaruEntry::new(seq, tribe_id, op_code));
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn verify_all(&self) -> bool {
        self.entries.iter().all(|e| {
            let bytes = e.serialize();
            NaruEntry::verify(&bytes)
        })
    }
}
