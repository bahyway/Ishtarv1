//! Bridge: Particle (AkkValue) ↔ EavTriple (Vec<u8>) for EnkiDb.
//!
//! The bridge converts the high-level Particle type (which carries AkkValue)
//! into the low-level EavTriple format that the Journal + EavIndex accept.
//!
//! attr_hash: CRC-16 of the attribute name string (cast to u32).
//!            This is stable — the same string always hashes the same way.
//!
//! value_bytes: sovereign AkkValue binary codec (akkvalue::codec).

use akkvalue::codec::encode as codec_encode;
use akkvalue::AkkValue;
use bahyway_crc::crc16;
use enkidb_journal::entry::EavTriple;
use enkidb_particles::Particle;

/// Compute the attribute hash for a given attribute name string.
/// Uses CRC-16/CCITT over the UTF-8 bytes, cast to u32.
#[inline]
pub fn attr_hash(attribute: &str) -> u32 {
    crc16(attribute.as_bytes()) as u32
}

/// Convert a Particle into an EavTriple for the Journal.
pub fn particle_to_eav_triple(p: &Particle) -> EavTriple {
    let hash  = attr_hash(&p.attribute);
    let bytes = codec_encode(&p.value);
    EavTriple::new(hash, bytes)
}

/// Convert a (attribute_name, AkkValue) pair directly into an EavTriple.
pub fn attr_value_to_eav_triple(attribute: &str, value: &AkkValue) -> EavTriple {
    EavTriple::new(attr_hash(attribute), codec_encode(value))
}

/// Decode the value from an EavTriple's bytes back into AkkValue.
/// Returns `AkkValue::Null` if bytes are empty or decoding fails.
pub fn eav_triple_to_value(triple: &EavTriple) -> AkkValue {
    if triple.value.is_empty() {
        return AkkValue::Null;
    }
    akkvalue::codec::decode(&triple.value, 0)
        .map(|(v, _)| v)
        .unwrap_or(AkkValue::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use akkvalue::AkkValue;

    #[test]
    fn attr_hash_is_deterministic() {
        assert_eq!(attr_hash("quality.score"), attr_hash("quality.score"));
        assert_ne!(attr_hash("quality.score"), attr_hash("quality.lane"));
    }

    #[test]
    fn round_trip_text_value() {
        let triple = attr_value_to_eav_triple("name.given", &AkkValue::Text("Bahaa".into()));
        let decoded = eav_triple_to_value(&triple);
        assert_eq!(decoded, AkkValue::Text("Bahaa".into()));
    }

    #[test]
    fn round_trip_int_value() {
        let triple = attr_value_to_eav_triple("birth.year", &AkkValue::Int(1985));
        assert_eq!(eav_triple_to_value(&triple), AkkValue::Int(1985));
    }

    #[test]
    fn round_trip_null() {
        let triple = attr_value_to_eav_triple("maybe", &AkkValue::Null);
        assert_eq!(eav_triple_to_value(&triple), AkkValue::Null);
    }

    #[test]
    fn round_trip_coordinate() {
        use akkvalue::AkkCoordinate;
        let coord = AkkValue::Coordinate(AkkCoordinate::new(32.0031, 44.3282));
        let triple = attr_value_to_eav_triple("location", &coord);
        assert_eq!(eav_triple_to_value(&triple), coord);
    }

    #[test]
    fn empty_bytes_decodes_as_null() {
        let triple = EavTriple::new(0, vec![]);
        assert_eq!(eav_triple_to_value(&triple), AkkValue::Null);
    }
}
