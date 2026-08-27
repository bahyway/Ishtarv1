//! Particle <-> byte record codec for the Data File.
//!
//! Wire format (no length prefix here — the Index carries offset+len):
//!   [entity_kaki: 16 bytes]
//!   [attr_len: u16 BE][attr_bytes: attr_len bytes]
//!   [value: akkvalue::codec self-describing encoding]
//!   [timestamp: u64 BE]
//!   [shard: u32 BE]
//!   [proof_ctx: u32 BE]
//!   [mat_tag: u32 BE]

use akkvalue::{decode_value, encode_value};
use enkidb_kaki::{IdentityKaki, Kaki};
use enkidb_particles::Particle;

#[derive(Debug)]
pub enum RecordError {
    Truncated,
    InvalidKaki,
    InvalidUtf8,
    Codec(akkvalue::CodecError),
}

impl From<akkvalue::CodecError> for RecordError {
    fn from(e: akkvalue::CodecError) -> Self {
        RecordError::Codec(e)
    }
}

/// Serialize a particle into its on-disk record bytes.
pub fn encode_particle(p: &Particle) -> Vec<u8> {
    let mut buf = Vec::with_capacity(64);
    buf.extend_from_slice(p.entity.bytes());

    let attr_bytes = p.attribute.as_bytes();
    buf.extend_from_slice(&(attr_bytes.len() as u16).to_be_bytes());
    buf.extend_from_slice(attr_bytes);

    buf.extend_from_slice(&encode_value(&p.value));

    buf.extend_from_slice(&p.timestamp.to_be_bytes());
    buf.extend_from_slice(&p.shard.to_be_bytes());
    buf.extend_from_slice(&p.proof_ctx.to_be_bytes());
    buf.extend_from_slice(&p.mat_tag.to_be_bytes());

    buf
}

/// Deserialize a record's bytes (exactly as sliced via the Index's offset+len) into a Particle.
pub fn decode_particle(bytes: &[u8]) -> Result<Particle, RecordError> {
    if bytes.len() < 16 + 2 {
        return Err(RecordError::Truncated);
    }
    let mut kaki_bytes = [0u8; 16];
    kaki_bytes.copy_from_slice(&bytes[0..16]);
    let kaki = Kaki::from_bytes(kaki_bytes).map_err(|_| RecordError::InvalidKaki)?;
    let entity = IdentityKaki::try_from_kaki(kaki).map_err(|_| RecordError::InvalidKaki)?;

    let mut pos = 16;
    let attr_len = u16::from_be_bytes(
        bytes[pos..pos + 2]
            .try_into()
            .map_err(|_| RecordError::Truncated)?,
    ) as usize;
    pos += 2;
    if bytes.len() < pos + attr_len {
        return Err(RecordError::Truncated);
    }
    let attribute = String::from_utf8(bytes[pos..pos + attr_len].to_vec())
        .map_err(|_| RecordError::InvalidUtf8)?;
    pos += attr_len;

    let (value, new_pos) = decode_value(bytes, pos)?;
    pos = new_pos;

    if bytes.len() < pos + 8 + 4 + 4 + 4 {
        return Err(RecordError::Truncated);
    }
    let timestamp = u64::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap());
    pos += 8;
    let shard = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap());
    pos += 4;
    let proof_ctx = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap());
    pos += 4;
    let mat_tag = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap());

    Ok(Particle::new(
        entity, attribute, value, timestamp, shard, proof_ctx, mat_tag,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use akkvalue::AkkValue;
    use bahyway_core::TribeId;
    use enkidb_kaki::{KakiMinter, KakiRole};

    fn make_particle() -> Particle {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        Particle::new(
            e,
            "quality.score",
            AkkValue::QualityScore(200),
            1_700_000_000,
            3,
            7,
            1,
        )
    }

    #[test]
    fn round_trip_preserves_all_fields() {
        let p = make_particle();
        let bytes = encode_particle(&p);
        let back = decode_particle(&bytes).unwrap();

        assert_eq!(back.entity, p.entity);
        assert_eq!(back.attribute, p.attribute);
        assert_eq!(back.value, p.value);
        assert_eq!(back.timestamp, p.timestamp);
        assert_eq!(back.shard, p.shard);
        assert_eq!(back.proof_ctx, p.proof_ctx);
        assert_eq!(back.mat_tag, p.mat_tag);
    }

    #[test]
    fn truncated_bytes_rejected() {
        let p = make_particle();
        let bytes = encode_particle(&p);
        let truncated = &bytes[..bytes.len() - 3];
        assert!(decode_particle(truncated).is_err());
    }

    #[test]
    fn long_attribute_name_round_trips() {
        let m = KakiMinter::new(TribeId::from_u16(0x0002));
        let e = IdentityKaki::try_from_kaki(m.identity(KakiRole::Kishib)).unwrap();
        let long_attr = "a".repeat(500);
        let p = Particle::base(e, long_attr.clone(), AkkValue::Text("x".into()), 0);
        let bytes = encode_particle(&p);
        let back = decode_particle(&bytes).unwrap();
        assert_eq!(back.attribute, long_attr);
    }
}
