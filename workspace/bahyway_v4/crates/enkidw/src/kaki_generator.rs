//! KakiGenerator — converts raw TSV/CSV rows into Journal-ready particles.
//!
//! Each data row produces one Identity-Kaki + one Event-Kaki + EAV triples.
//!
//! Column mapping rules:
//!   - Column named "epoch"  → epoch field (parsed as u32; defaults to 1)
//!   - Column named "state"  → ATTR_STATE (0x1A4B), encoded with `encode_state`
//!   - All other columns     → EAV triple where attr_hash = fnv1a_32(column_name)
//!
//! The uuid_hash for the Identity-Kaki is derived from the first data column
//! value using FNV-1a so the same source row always produces the same particle.

use bahyway_core::ParticleState;
use enkidb_journal::entry::EavTriple;
use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
use story_engine::projection::{encode_state, ATTR_STATE};

/// One raw data row from a CSV/TSV file.
#[derive(Debug, Clone)]
pub struct RawRecord {
    /// Column names in order.
    pub headers: Vec<String>,
    /// Column values in the same order as headers.
    pub values:  Vec<Vec<u8>>,
    /// Sequence number for the row (used as epoch fallback).
    pub seq:     u32,
}

/// A fully-generated particle ready for `PersistedDb::commit`.
#[derive(Debug)]
pub struct GeneratedEntry {
    pub particle:   IdentityKaki,
    pub event_kaki: EventKaki,
    pub epoch:      u32,
    pub eav:        Vec<EavTriple>,
}

/// Generate one particle + event from a raw data row.
pub fn generate(minter: &KakiMinter, record: &RawRecord) -> GeneratedEntry {
    let mut epoch = record.seq.max(1);
    let mut eav   = Vec::with_capacity(record.headers.len());

    // Derive uuid_hash from the first column value (deterministic)
    let seed_value = record.values.first().map(|v| v.as_slice()).unwrap_or(b"");
    let uuid_hash  = fnv1a_32(seed_value);

    for (col_name, val) in record.headers.iter().zip(record.values.iter()) {
        match col_name.to_lowercase().as_str() {
            "epoch" | "ep" => {
                if let Ok(s) = std::str::from_utf8(val) {
                    if let Ok(n) = s.trim().parse::<u32>() {
                        epoch = n;
                    }
                }
                // epoch column is not stored as EAV
            }
            "state" | "status" => {
                let state = decode_state_str(val);
                eav.push(EavTriple::new(ATTR_STATE, encode_state(state).to_vec()));
            }
            other => {
                if !val.is_empty() {
                    let attr_hash = fnv1a_32(other.as_bytes());
                    eav.push(EavTriple::new(attr_hash, val.clone()));
                }
            }
        }
    }

    // If no state column was present, default to Fuzzy
    let has_state = eav.iter().any(|t| t.attr_hash == ATTR_STATE);
    if !has_state {
        eav.push(EavTriple::new(ATTR_STATE, encode_state(ParticleState::Fuzzy).to_vec()));
    }

    let particle   = IdentityKaki::try_from_kaki(
        minter.mint_identity(uuid_hash, KakiRole::Zikru)
    ).unwrap_or_else(|_| IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap());

    let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();

    GeneratedEntry { particle, event_kaki, epoch, eav }
}

/// Parse a TSV byte slice into a list of `RawRecord`s.
/// First line is treated as the header row.
pub fn parse_tsv(data: &[u8]) -> Vec<RawRecord> {
    parse_delimited(data, b'\t')
}

/// Parse a CSV byte slice into a list of `RawRecord`s.
pub fn parse_csv(data: &[u8]) -> Vec<RawRecord> {
    parse_delimited(data, b',')
}

fn parse_delimited(data: &[u8], sep: u8) -> Vec<RawRecord> {
    let text = match std::str::from_utf8(data) {
        Ok(s)  => s,
        Err(_) => return Vec::new(),
    };
    let mut lines = text.lines();
    let Some(header_line) = lines.next() else { return Vec::new(); };

    let headers: Vec<String> = split_bytes(header_line.as_bytes(), sep)
        .into_iter()
        .map(|b| String::from_utf8_lossy(b).trim().to_string())
        .collect();

    if headers.is_empty() { return Vec::new(); }

    lines.enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, line)| {
            let mut values: Vec<Vec<u8>> = split_bytes(line.as_bytes(), sep)
                .into_iter()
                .map(|b| {
                    let s = std::str::from_utf8(b).unwrap_or("").trim();
                    s.as_bytes().to_vec()
                })
                .collect();
            // Pad or truncate to match header count
            values.resize(headers.len(), Vec::new());
            RawRecord { headers: headers.clone(), values, seq: (i + 1) as u32 }
        })
        .collect()
}

fn split_bytes(line: &[u8], sep: u8) -> Vec<&[u8]> {
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, &b) in line.iter().enumerate() {
        if b == sep { parts.push(&line[start..i]); start = i + 1; }
    }
    parts.push(&line[start..]);
    parts
}

fn decode_state_str(val: &[u8]) -> ParticleState {
    match std::str::from_utf8(val).unwrap_or("").trim().to_lowercase().as_str() {
        "golden" | "1" | "active" | "ok"   => ParticleState::Golden,
        "dead"   | "0" | "inactive" | "off" => ParticleState::Dead,
        _                                   => ParticleState::Fuzzy,
    }
}

/// FNV-1a 32-bit hash — used to derive attr_hash from column names.
pub fn fnv1a_32(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for &b in data {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0001))
    }

    #[test]
    fn parse_tsv_single_row() {
        let data = b"name\tepoch\tstate\nAli_Karim\t1\tGolden\n";
        let rows = parse_tsv(data);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].headers, &["name", "epoch", "state"]);
        assert_eq!(rows[0].values[0], b"Ali_Karim");
    }

    #[test]
    fn generate_golden_particle() {
        let data = b"name\tepoch\tstate\nAli_Karim\t5\tGolden\n";
        let rows = parse_tsv(data);
        let m    = minter();
        let ge   = generate(&m, &rows[0]);
        assert_eq!(ge.epoch, 5);
        let state_eav = ge.eav.iter().find(|e| e.attr_hash == ATTR_STATE).unwrap();
        let state = decode_state_str(&state_eav.value);
        assert_eq!(state, ParticleState::Golden);
    }

    #[test]
    fn deterministic_uuid_hash_for_same_name() {
        let data = b"name\nAli_Karim\n";
        let rows = parse_tsv(data);
        let m1   = minter();
        let m2   = minter();
        let ge1  = generate(&m1, &rows[0]);
        let ge2  = generate(&m2, &rows[0]);
        assert_eq!(ge1.particle.uuid_hash(), ge2.particle.uuid_hash());
    }

    #[test]
    fn missing_state_defaults_to_fuzzy() {
        let data = b"name\nSomeone\n";
        let rows = parse_tsv(data);
        let m    = minter();
        let ge   = generate(&m, &rows[0]);
        let state_eav = ge.eav.iter().find(|e| e.attr_hash == ATTR_STATE).unwrap();
        let state = decode_state_str(&state_eav.value);
        assert_eq!(state, ParticleState::Fuzzy);
    }

    #[test]
    fn extra_columns_become_eav_triples() {
        let data = b"name\tcity\nAli\tBaghdad\n";
        let rows = parse_tsv(data);
        let m    = minter();
        let ge   = generate(&m, &rows[0]);
        let city_hash = fnv1a_32(b"city");
        assert!(ge.eav.iter().any(|e| e.attr_hash == city_hash && e.value == b"Baghdad"));
    }

    #[test]
    fn parse_csv_works() {
        let data = b"name,epoch,state\nFatima,2,Golden\n";
        let rows = parse_csv(data);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].values[0], b"Fatima");
    }

    #[test]
    fn empty_optional_column_not_stored_as_eav() {
        // Optional column "notes" is empty — must NOT produce an EAV triple.
        // If it did, DQ.VALIDITY would flag every record with any blank optional field.
        let data = b"name\tnotes\nAli\t\n";
        let rows = parse_tsv(data);
        let m    = minter();
        let ge   = generate(&m, &rows[0]);
        let notes_hash = fnv1a_32(b"notes");
        assert!(
            !ge.eav.iter().any(|t| t.attr_hash == notes_hash),
            "empty optional column must not produce an EAV triple"
        );
    }
}
