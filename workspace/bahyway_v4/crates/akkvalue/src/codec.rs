//! Sovereign binary codec for AkkValue — no external deps, no serde.
//!
//! Wire format: `[type_tag: u8] [payload bytes...]`
//!
//! The format is forward-compatible: unknown type tags are surfaced as
//! `CodecError::UnknownTypeTag` so callers can skip them. Variable-length
//! payloads are always prefixed by a 4-byte big-endian length (`u32`).

use crate::{
    types::{
        AkkCoordinate, AkkDate, AkkHijriDate, AkkNamePartLabel, AkkNameVector, AkkNationalId,
        AkkPhone, CipherAlgorithm, PipelineStatus, PolicyVerdict,
    },
    AkkValue,
};

// ── Type tags ─────────────────────────────────────────────────────────────────

const TAG_NULL: u8 = 0x00;
const TAG_BOOL: u8 = 0x01;
const TAG_INT: u8 = 0x02;
const TAG_FLOAT: u8 = 0x03;
const TAG_TEXT: u8 = 0x04;
const TAG_BYTES: u8 = 0x05;
const TAG_UUID: u8 = 0x06;
const TAG_NATIONAL_ID: u8 = 0x07;
const TAG_PHONE: u8 = 0x08;
const TAG_TIMESTAMP: u8 = 0x09;
const TAG_DATE: u8 = 0x0A;
const TAG_HIJRI_DATE: u8 = 0x0B;
const TAG_DURATION: u8 = 0x0C;
const TAG_COORDINATE: u8 = 0x0D;
const TAG_COUNTRY_CODE: u8 = 0x0E;
const TAG_DOMAIN_BYTE: u8 = 0x0F;
const TAG_QUALITY_SCORE: u8 = 0x10;
const TAG_KAKI_PK: u8 = 0x11;
const TAG_AKKADIAN_ROOT: u8 = 0x12;
const TAG_NAME_VECTOR: u8 = 0x13;
const TAG_LANG_CODE: u8 = 0x14;
const TAG_EMBEDDING: u8 = 0x15;
const TAG_PROBABILITY: u8 = 0x16;
const TAG_LABEL: u8 = 0x17;
const TAG_CONFIDENCE: u8 = 0x18;
const TAG_PIPELINE_STATUS: u8 = 0x19;
const TAG_STEP_INDEX: u8 = 0x1A;
const TAG_POLICY_VERDICT: u8 = 0x1B;
const TAG_CIPHER_ALGORITHM: u8 = 0x1C;
const TAG_SEAL_SIGNATURE: u8 = 0x1D;
const TAG_LIST: u8 = 0x1E;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CodecError {
    Truncated { expected: usize, got: usize },
    InvalidUtf8,
    UnknownTypeTag(u8),
    UnknownDiscriminant { tag: u8, discriminant: u8 },
    InvalidLength { max: usize, got: usize },
}

impl core::fmt::Display for CodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Truncated { expected, got } => {
                write!(f, "codec: need {expected} bytes, only {got} available")
            }
            Self::InvalidUtf8 => write!(f, "codec: invalid UTF-8 in Text/String field"),
            Self::UnknownTypeTag(t) => write!(f, "codec: unknown type tag {t:#04x}"),
            Self::UnknownDiscriminant { tag, discriminant } => write!(
                f,
                "codec: unknown discriminant {discriminant:#04x} for tag {tag:#04x}"
            ),
            Self::InvalidLength { max, got } => {
                write!(f, "codec: field length {got} exceeds max {max}")
            }
        }
    }
}

// ── Encode ────────────────────────────────────────────────────────────────────

/// Encode an `AkkValue` into a sovereign byte vector.
pub fn encode(value: &AkkValue) -> Vec<u8> {
    let mut buf = Vec::with_capacity(16);
    encode_into(value, &mut buf);
    buf
}

fn encode_into(value: &AkkValue, buf: &mut Vec<u8>) {
    match value {
        AkkValue::Null => {
            buf.push(TAG_NULL);
        }
        AkkValue::Bool(b) => {
            buf.push(TAG_BOOL);
            buf.push(if *b { 1 } else { 0 });
        }
        AkkValue::Int(i) => {
            buf.push(TAG_INT);
            buf.extend_from_slice(&i.to_be_bytes());
        }
        AkkValue::Float(f) => {
            buf.push(TAG_FLOAT);
            buf.extend_from_slice(&f.to_be_bytes());
        }
        AkkValue::Text(s) => {
            buf.push(TAG_TEXT);
            push_str(buf, s);
        }
        AkkValue::Bytes(b) => {
            buf.push(TAG_BYTES);
            push_bytes(buf, b);
        }
        AkkValue::Uuid(u) => {
            buf.push(TAG_UUID);
            buf.extend_from_slice(u);
        }
        AkkValue::NationalId(n) => {
            buf.push(TAG_NATIONAL_ID);
            push_str(buf, &n.country_code);
            push_str(buf, &n.number);
        }
        AkkValue::Phone(p) => {
            buf.push(TAG_PHONE);
            buf.extend_from_slice(&p.country_code.to_be_bytes());
            push_str(buf, &p.number);
        }
        AkkValue::Timestamp(ts) => {
            buf.push(TAG_TIMESTAMP);
            buf.extend_from_slice(&ts.to_be_bytes());
        }
        AkkValue::Date(d) => {
            buf.push(TAG_DATE);
            buf.extend_from_slice(&d.year.to_be_bytes());
            buf.push(d.month);
            buf.push(d.day);
        }
        AkkValue::HijriDate(h) => {
            buf.push(TAG_HIJRI_DATE);
            buf.extend_from_slice(&h.year.to_be_bytes());
            buf.push(h.month);
            buf.push(h.day);
        }
        AkkValue::Duration(s) => {
            buf.push(TAG_DURATION);
            buf.extend_from_slice(&s.to_be_bytes());
        }
        AkkValue::Coordinate(c) => {
            buf.push(TAG_COORDINATE);
            buf.extend_from_slice(&c.lat.to_be_bytes());
            buf.extend_from_slice(&c.lon.to_be_bytes());
            match c.alt {
                Some(alt) => {
                    buf.push(1);
                    buf.extend_from_slice(&alt.to_be_bytes());
                }
                None => buf.push(0),
            }
        }
        AkkValue::CountryCode(cc) => {
            buf.push(TAG_COUNTRY_CODE);
            push_str(buf, cc);
        }
        AkkValue::DomainByte(b) => {
            buf.push(TAG_DOMAIN_BYTE);
            buf.push(*b);
        }
        AkkValue::QualityScore(q) => {
            buf.push(TAG_QUALITY_SCORE);
            buf.push(*q);
        }
        AkkValue::KakiPk(pk) => {
            buf.push(TAG_KAKI_PK);
            buf.extend_from_slice(pk);
        }
        AkkValue::AkkadianRoot(r) => {
            buf.push(TAG_AKKADIAN_ROOT);
            push_str(buf, r);
        }
        AkkValue::NameVector(nv) => {
            buf.push(TAG_NAME_VECTOR);
            let count = nv.parts.len() as u32;
            buf.extend_from_slice(&count.to_be_bytes());
            for (label, part) in &nv.parts {
                buf.push(name_part_label_tag(label));
                push_str(buf, part);
            }
        }
        AkkValue::LangCode(l) => {
            buf.push(TAG_LANG_CODE);
            push_str(buf, l);
        }
        AkkValue::Embedding(e) => {
            buf.push(TAG_EMBEDDING);
            let count = e.len() as u32;
            buf.extend_from_slice(&count.to_be_bytes());
            for &f in e {
                buf.extend_from_slice(&f.to_be_bytes());
            }
        }
        AkkValue::Probability(p) => {
            buf.push(TAG_PROBABILITY);
            buf.extend_from_slice(&p.to_be_bytes());
        }
        AkkValue::Label(l) => {
            buf.push(TAG_LABEL);
            push_str(buf, l);
        }
        AkkValue::Confidence(c) => {
            buf.push(TAG_CONFIDENCE);
            buf.extend_from_slice(&c.to_be_bytes());
        }
        AkkValue::PipelineStatus(s) => {
            buf.push(TAG_PIPELINE_STATUS);
            match s {
                PipelineStatus::Pending => buf.push(0x00),
                PipelineStatus::Running => buf.push(0x01),
                PipelineStatus::Completed => buf.push(0x02),
                PipelineStatus::Failed(r) => {
                    buf.push(0x03);
                    push_str(buf, r);
                }
                PipelineStatus::Cancelled => buf.push(0x04),
            }
        }
        AkkValue::StepIndex(i) => {
            buf.push(TAG_STEP_INDEX);
            buf.extend_from_slice(&i.to_be_bytes());
        }
        AkkValue::PolicyVerdict(v) => {
            buf.push(TAG_POLICY_VERDICT);
            buf.push(match v {
                PolicyVerdict::Allow => 0x00,
                PolicyVerdict::Deny => 0x01,
                PolicyVerdict::Escalate => 0x02,
                PolicyVerdict::Redact => 0x03,
                PolicyVerdict::Audit => 0x04,
            });
        }
        AkkValue::CipherAlgorithm(a) => {
            buf.push(TAG_CIPHER_ALGORITHM);
            buf.push(match a {
                CipherAlgorithm::ChaCha20Poly1305 => 0x00,
                CipherAlgorithm::Aes256Gcm => 0x01,
                CipherAlgorithm::Ed25519 => 0x02,
                CipherAlgorithm::Dilithium3 => 0x03,
            });
        }
        AkkValue::SealSignature(s) => {
            buf.push(TAG_SEAL_SIGNATURE);
            push_bytes(buf, s);
        }
        AkkValue::List(items) => {
            buf.push(TAG_LIST);
            let count = items.len() as u32;
            buf.extend_from_slice(&count.to_be_bytes());
            for item in items {
                encode_into(item, buf);
            }
        }
    }
}

fn push_str(buf: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    buf.extend_from_slice(&(b.len() as u32).to_be_bytes());
    buf.extend_from_slice(b);
}

fn push_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(data);
}

fn name_part_label_tag(label: &AkkNamePartLabel) -> u8 {
    match label {
        AkkNamePartLabel::Given => 0x00,
        AkkNamePartLabel::Family => 0x01,
        AkkNamePartLabel::Father => 0x02,
        AkkNamePartLabel::Grandfather => 0x03,
        AkkNamePartLabel::Tribe => 0x04,
        AkkNamePartLabel::Honorific => 0x05,
        AkkNamePartLabel::Kunyah => 0x06,
        AkkNamePartLabel::Laqab => 0x07,
    }
}

// ── Decode ────────────────────────────────────────────────────────────────────

/// Decode one `AkkValue` from `bytes[pos..]`.
/// Returns `(value, new_pos)` — the number of bytes consumed.
pub fn decode(bytes: &[u8], pos: usize) -> Result<(AkkValue, usize), CodecError> {
    let tag = read_u8(bytes, pos)?;
    let p = pos + 1;
    match tag {
        TAG_NULL => Ok((AkkValue::Null, p)),
        TAG_BOOL => {
            let b = read_u8(bytes, p)?;
            Ok((AkkValue::Bool(b != 0), p + 1))
        }
        TAG_INT => {
            let i = read_i64(bytes, p)?;
            Ok((AkkValue::Int(i), p + 8))
        }
        TAG_FLOAT => {
            let f = read_f64(bytes, p)?;
            Ok((AkkValue::Float(f), p + 8))
        }
        TAG_TEXT => {
            let (s, np) = read_str(bytes, p)?;
            Ok((AkkValue::Text(s), np))
        }
        TAG_BYTES => {
            let (b, np) = read_bytes_vec(bytes, p)?;
            Ok((AkkValue::Bytes(b), np))
        }
        TAG_UUID => {
            need(bytes, p, 16)?;
            let mut u = [0u8; 16];
            u.copy_from_slice(&bytes[p..p + 16]);
            Ok((AkkValue::Uuid(u), p + 16))
        }
        TAG_NATIONAL_ID => {
            let (cc, p2) = read_str(bytes, p)?;
            let (num, p3) = read_str(bytes, p2)?;
            Ok((AkkValue::NationalId(AkkNationalId::new(cc, num)), p3))
        }
        TAG_PHONE => {
            let cc = read_u16(bytes, p)?;
            let (num, p2) = read_str(bytes, p + 2)?;
            Ok((AkkValue::Phone(AkkPhone::new(cc, num)), p2))
        }
        TAG_TIMESTAMP => {
            let ts = read_i64(bytes, p)?;
            Ok((AkkValue::Timestamp(ts), p + 8))
        }
        TAG_DATE => {
            need(bytes, p, 6)?;
            let year = i32::from_be_bytes(bytes[p..p + 4].try_into().unwrap());
            let month = bytes[p + 4];
            let day = bytes[p + 5];
            Ok((AkkValue::Date(AkkDate::new(year, month, day)), p + 6))
        }
        TAG_HIJRI_DATE => {
            need(bytes, p, 4)?;
            let year = u16::from_be_bytes(bytes[p..p + 2].try_into().unwrap());
            let month = bytes[p + 2];
            let day = bytes[p + 3];
            Ok((
                AkkValue::HijriDate(AkkHijriDate::new(year, month, day)),
                p + 4,
            ))
        }
        TAG_DURATION => {
            let d = read_u64(bytes, p)?;
            Ok((AkkValue::Duration(d), p + 8))
        }
        TAG_COORDINATE => {
            need(bytes, p, 17)?; // 8+8+1 minimum
            let lat = read_f64(bytes, p)?;
            let lon = read_f64(bytes, p + 8)?;
            let has_alt = bytes[p + 16];
            if has_alt != 0 {
                need(bytes, p + 17, 8)?;
                let alt = read_f64(bytes, p + 17)?;
                let mut c = AkkCoordinate::new(lat, lon);
                c.alt = Some(alt);
                Ok((AkkValue::Coordinate(c), p + 25))
            } else {
                Ok((AkkValue::Coordinate(AkkCoordinate::new(lat, lon)), p + 17))
            }
        }
        TAG_COUNTRY_CODE => {
            let (s, np) = read_str(bytes, p)?;
            Ok((AkkValue::CountryCode(s), np))
        }
        TAG_DOMAIN_BYTE => {
            let b = read_u8(bytes, p)?;
            Ok((AkkValue::DomainByte(b), p + 1))
        }
        TAG_QUALITY_SCORE => {
            let q = read_u8(bytes, p)?;
            Ok((AkkValue::QualityScore(q), p + 1))
        }
        TAG_KAKI_PK => {
            need(bytes, p, 16)?;
            let mut pk = [0u8; 16];
            pk.copy_from_slice(&bytes[p..p + 16]);
            Ok((AkkValue::KakiPk(pk), p + 16))
        }
        TAG_AKKADIAN_ROOT => {
            let (s, np) = read_str(bytes, p)?;
            Ok((AkkValue::AkkadianRoot(s), np))
        }
        TAG_NAME_VECTOR => {
            let count = read_u32(bytes, p)? as usize;
            let mut np = p + 4;
            let mut nv = AkkNameVector::new();
            for _ in 0..count {
                let label_byte = read_u8(bytes, np)?;
                np += 1;
                let label = name_part_label_from_byte(TAG_NAME_VECTOR, label_byte)?;
                let (part, p2) = read_str(bytes, np)?;
                np = p2;
                nv.push(label, part);
            }
            Ok((AkkValue::NameVector(nv), np))
        }
        TAG_LANG_CODE => {
            let (s, np) = read_str(bytes, p)?;
            Ok((AkkValue::LangCode(s), np))
        }
        TAG_EMBEDDING => {
            let count = read_u32(bytes, p)? as usize;
            need(bytes, p + 4, count * 4)?;
            let mut floats = Vec::with_capacity(count);
            let mut np = p + 4;
            for _ in 0..count {
                let f = f32::from_be_bytes(bytes[np..np + 4].try_into().unwrap());
                floats.push(f);
                np += 4;
            }
            Ok((AkkValue::Embedding(floats), np))
        }
        TAG_PROBABILITY => {
            let f = read_f64(bytes, p)?;
            Ok((AkkValue::Probability(f), p + 8))
        }
        TAG_LABEL => {
            let (s, np) = read_str(bytes, p)?;
            Ok((AkkValue::Label(s), np))
        }
        TAG_CONFIDENCE => {
            let f = read_f64(bytes, p)?;
            Ok((AkkValue::Confidence(f), p + 8))
        }
        TAG_PIPELINE_STATUS => {
            let disc = read_u8(bytes, p)?;
            match disc {
                0x00 => Ok((AkkValue::PipelineStatus(PipelineStatus::Pending), p + 1)),
                0x01 => Ok((AkkValue::PipelineStatus(PipelineStatus::Running), p + 1)),
                0x02 => Ok((AkkValue::PipelineStatus(PipelineStatus::Completed), p + 1)),
                0x03 => {
                    let (reason, np) = read_str(bytes, p + 1)?;
                    Ok((AkkValue::PipelineStatus(PipelineStatus::Failed(reason)), np))
                }
                0x04 => Ok((AkkValue::PipelineStatus(PipelineStatus::Cancelled), p + 1)),
                d => Err(CodecError::UnknownDiscriminant {
                    tag,
                    discriminant: d,
                }),
            }
        }
        TAG_STEP_INDEX => {
            let i = read_u32(bytes, p)?;
            Ok((AkkValue::StepIndex(i), p + 4))
        }
        TAG_POLICY_VERDICT => {
            let d = read_u8(bytes, p)?;
            let v = match d {
                0x00 => PolicyVerdict::Allow,
                0x01 => PolicyVerdict::Deny,
                0x02 => PolicyVerdict::Escalate,
                0x03 => PolicyVerdict::Redact,
                0x04 => PolicyVerdict::Audit,
                d => {
                    return Err(CodecError::UnknownDiscriminant {
                        tag,
                        discriminant: d,
                    })
                }
            };
            Ok((AkkValue::PolicyVerdict(v), p + 1))
        }
        TAG_CIPHER_ALGORITHM => {
            let d = read_u8(bytes, p)?;
            let a = match d {
                0x00 => CipherAlgorithm::ChaCha20Poly1305,
                0x01 => CipherAlgorithm::Aes256Gcm,
                0x02 => CipherAlgorithm::Ed25519,
                0x03 => CipherAlgorithm::Dilithium3,
                d => {
                    return Err(CodecError::UnknownDiscriminant {
                        tag,
                        discriminant: d,
                    })
                }
            };
            Ok((AkkValue::CipherAlgorithm(a), p + 1))
        }
        TAG_SEAL_SIGNATURE => {
            let (b, np) = read_bytes_vec(bytes, p)?;
            Ok((AkkValue::SealSignature(b), np))
        }
        TAG_LIST => {
            let count = read_u32(bytes, p)? as usize;
            let mut np = p + 4;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let (item, p2) = decode(bytes, np)?;
                np = p2;
                items.push(item);
            }
            Ok((AkkValue::List(items), np))
        }
        t => Err(CodecError::UnknownTypeTag(t)),
    }
}

// ── Decode helpers ────────────────────────────────────────────────────────────

#[inline]
fn need(bytes: &[u8], pos: usize, n: usize) -> Result<(), CodecError> {
    let available = bytes.len().saturating_sub(pos);
    if available < n {
        Err(CodecError::Truncated {
            expected: n,
            got: available,
        })
    } else {
        Ok(())
    }
}

fn read_u8(bytes: &[u8], pos: usize) -> Result<u8, CodecError> {
    need(bytes, pos, 1)?;
    Ok(bytes[pos])
}

fn read_u16(bytes: &[u8], pos: usize) -> Result<u16, CodecError> {
    need(bytes, pos, 2)?;
    Ok(u16::from_be_bytes(bytes[pos..pos + 2].try_into().unwrap()))
}

fn read_u32(bytes: &[u8], pos: usize) -> Result<u32, CodecError> {
    need(bytes, pos, 4)?;
    Ok(u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()))
}

fn read_u64(bytes: &[u8], pos: usize) -> Result<u64, CodecError> {
    need(bytes, pos, 8)?;
    Ok(u64::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap()))
}

fn read_i64(bytes: &[u8], pos: usize) -> Result<i64, CodecError> {
    need(bytes, pos, 8)?;
    Ok(i64::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap()))
}

fn read_f64(bytes: &[u8], pos: usize) -> Result<f64, CodecError> {
    need(bytes, pos, 8)?;
    Ok(f64::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap()))
}

fn read_str(bytes: &[u8], pos: usize) -> Result<(String, usize), CodecError> {
    let len = read_u32(bytes, pos)? as usize;
    need(bytes, pos + 4, len)?;
    let s = core::str::from_utf8(&bytes[pos + 4..pos + 4 + len])
        .map_err(|_| CodecError::InvalidUtf8)?
        .to_string();
    Ok((s, pos + 4 + len))
}

fn read_bytes_vec(bytes: &[u8], pos: usize) -> Result<(Vec<u8>, usize), CodecError> {
    let len = read_u32(bytes, pos)? as usize;
    need(bytes, pos + 4, len)?;
    Ok((bytes[pos + 4..pos + 4 + len].to_vec(), pos + 4 + len))
}

fn name_part_label_from_byte(tag: u8, b: u8) -> Result<AkkNamePartLabel, CodecError> {
    Ok(match b {
        0x00 => AkkNamePartLabel::Given,
        0x01 => AkkNamePartLabel::Family,
        0x02 => AkkNamePartLabel::Father,
        0x03 => AkkNamePartLabel::Grandfather,
        0x04 => AkkNamePartLabel::Tribe,
        0x05 => AkkNamePartLabel::Honorific,
        0x06 => AkkNamePartLabel::Kunyah,
        0x07 => AkkNamePartLabel::Laqab,
        d => {
            return Err(CodecError::UnknownDiscriminant {
                tag,
                discriminant: d,
            })
        }
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AkkCoordinate;

    fn round_trip(v: AkkValue) -> AkkValue {
        let bytes = encode(&v);
        let (decoded, pos) = decode(&bytes, 0).expect("decode failed");
        assert_eq!(pos, bytes.len(), "not all bytes consumed");
        decoded
    }

    #[test]
    fn null_round_trip() {
        assert_eq!(round_trip(AkkValue::Null), AkkValue::Null);
    }

    #[test]
    fn bool_round_trip() {
        assert_eq!(round_trip(AkkValue::Bool(true)), AkkValue::Bool(true));
        assert_eq!(round_trip(AkkValue::Bool(false)), AkkValue::Bool(false));
    }

    #[test]
    fn int_round_trip() {
        for v in [0i64, i64::MIN, i64::MAX, -1, 42, 1_700_000_000] {
            assert_eq!(round_trip(AkkValue::Int(v)), AkkValue::Int(v));
        }
    }

    #[test]
    fn float_round_trip() {
        for v in [0.0f64, f64::NAN, f64::INFINITY, -f64::INFINITY, 3.14159] {
            let enc = encode(&AkkValue::Float(v));
            let (AkkValue::Float(decoded), _) = decode(&enc, 0).unwrap() else {
                panic!()
            };
            if v.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded, v);
            }
        }
    }

    #[test]
    fn text_round_trip() {
        let cases = ["", "hello", "مرحبا", "𒀊𒆪"];
        for s in cases {
            let v = round_trip(AkkValue::Text(s.to_string()));
            assert_eq!(v, AkkValue::Text(s.to_string()));
        }
    }

    #[test]
    fn bytes_round_trip() {
        let b = vec![0u8, 1, 127, 128, 255];
        assert_eq!(round_trip(AkkValue::Bytes(b.clone())), AkkValue::Bytes(b));
    }

    #[test]
    fn uuid_round_trip() {
        let id = [
            0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9,
        ];
        assert_eq!(round_trip(AkkValue::Uuid(id)), AkkValue::Uuid(id));
    }

    #[test]
    fn timestamp_round_trip() {
        assert_eq!(
            round_trip(AkkValue::Timestamp(1_700_000_000)),
            AkkValue::Timestamp(1_700_000_000)
        );
        assert_eq!(round_trip(AkkValue::Timestamp(-1)), AkkValue::Timestamp(-1));
    }

    #[test]
    fn date_round_trip() {
        use crate::types::AkkDate;
        let d = AkkDate::new(2024, 3, 15);
        assert_eq!(round_trip(AkkValue::Date(d.clone())), AkkValue::Date(d));
    }

    #[test]
    fn coordinate_round_trip_with_alt() {
        let mut c = AkkCoordinate::new(32.0031, 44.3282);
        c.alt = Some(37.5);
        assert_eq!(
            round_trip(AkkValue::Coordinate(c.clone())),
            AkkValue::Coordinate(c)
        );
    }

    #[test]
    fn coordinate_round_trip_no_alt() {
        let c = AkkCoordinate::new(-33.8688, 151.2093);
        assert_eq!(
            round_trip(AkkValue::Coordinate(c.clone())),
            AkkValue::Coordinate(c)
        );
    }

    #[test]
    fn quality_score_round_trip() {
        assert_eq!(
            round_trip(AkkValue::QualityScore(240)),
            AkkValue::QualityScore(240)
        );
        assert_eq!(
            round_trip(AkkValue::QualityScore(0)),
            AkkValue::QualityScore(0)
        );
    }

    #[test]
    fn kaki_pk_round_trip() {
        let pk = [1u8; 16];
        assert_eq!(round_trip(AkkValue::KakiPk(pk)), AkkValue::KakiPk(pk));
    }

    #[test]
    fn embedding_round_trip() {
        let emb = vec![0.1f32, 0.2, 0.9, -0.5];
        let v = round_trip(AkkValue::Embedding(emb.clone()));
        if let AkkValue::Embedding(decoded) = v {
            assert_eq!(decoded.len(), emb.len());
            for (a, b) in decoded.iter().zip(emb.iter()) {
                assert!((a - b).abs() < 1e-6, "embedding mismatch: {a} vs {b}");
            }
        } else {
            panic!("wrong type");
        }
    }

    #[test]
    fn pipeline_status_failed_round_trip() {
        let s = PipelineStatus::Failed("disk full".into());
        assert_eq!(
            round_trip(AkkValue::PipelineStatus(s.clone())),
            AkkValue::PipelineStatus(s)
        );
    }

    #[test]
    fn list_round_trip() {
        let lst = AkkValue::List(vec![
            AkkValue::Int(1),
            AkkValue::Text("x".into()),
            AkkValue::Null,
        ]);
        assert_eq!(round_trip(lst.clone()), lst);
    }

    #[test]
    fn nested_list_round_trip() {
        let inner = AkkValue::List(vec![AkkValue::Bool(true), AkkValue::Int(99)]);
        let outer = AkkValue::List(vec![inner, AkkValue::Float(1.0)]);
        assert_eq!(round_trip(outer.clone()), outer);
    }

    #[test]
    fn truncated_bytes_returns_error() {
        let bytes = encode(&AkkValue::Int(42));
        // Truncate to just 4 bytes — missing last 4 of i64
        assert!(decode(&bytes[..4], 0).is_err());
    }

    #[test]
    fn unknown_tag_returns_error() {
        let bytes = [0xFF];
        assert!(matches!(
            decode(&bytes, 0),
            Err(CodecError::UnknownTypeTag(0xFF))
        ));
    }

    #[test]
    fn all_scalar_variants_round_trip() {
        let samples = vec![
            AkkValue::DomainByte(0xAB),
            AkkValue::StepIndex(u32::MAX),
            AkkValue::Probability(0.999),
            AkkValue::Confidence(0.001),
            AkkValue::Duration(3600),
            AkkValue::CountryCode("IQ".into()),
            AkkValue::LangCode("ar-IQ".into()),
            AkkValue::Label("grave".into()),
            AkkValue::AkkadianRoot("ktb".into()),
        ];
        for v in samples {
            let rt = round_trip(v.clone());
            assert_eq!(rt, v);
        }
    }
}
