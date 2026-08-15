//! AkkValue — 31-variant sovereign EAV value type.
//! Covers: scalar / identity / temporal / geographic / domain / linguistic /
//!         ML / pipeline / security types.
//! No external dependencies. No serde. No uuid crate. Pure Rust.

use core::fmt;

use crate::types::{
    AkkCoordinate, AkkDate, AkkHijriDate, AkkNationalId, AkkNameVector, AkkPhone,
    CipherAlgorithm, PipelineStatus, PolicyVerdict,
};

// ── AkkValue ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AkkValue {
    // Scalar
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),

    // Identity
    Uuid([u8; 16]),           // RFC 4122 UUID stored as raw bytes
    NationalId(AkkNationalId),
    Phone(AkkPhone),

    // Temporal
    Timestamp(i64),           // UNIX seconds (UTC)
    Date(AkkDate),
    HijriDate(AkkHijriDate),
    Duration(u64),            // seconds

    // Geographic
    Coordinate(AkkCoordinate),
    CountryCode(String),      // ISO 3166-1 alpha-2

    // Domain (KAKI)
    DomainByte(u8),
    QualityScore(u8),         // 0–240 (QUALITY_DIVISOR=240.0; ADR-001)
    KakiPk([u8; 16]),

    // Linguistic
    AkkadianRoot(String),     // 3-radical root string
    NameVector(AkkNameVector),
    LangCode(String),         // BCP-47

    // ML / Analytics
    Embedding(Vec<f32>),
    Probability(f64),         // 0.0..=1.0
    Label(String),
    Confidence(f64),          // 0.0..=1.0

    // Pipeline
    PipelineStatus(PipelineStatus),
    StepIndex(u32),

    // Security / Crypto
    PolicyVerdict(PolicyVerdict),
    CipherAlgorithm(CipherAlgorithm),
    SealSignature(Vec<u8>),

    // Structural
    List(Vec<AkkValue>),
}

impl AkkValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null              => "Null",
            Self::Bool(_)           => "Bool",
            Self::Int(_)            => "Int",
            Self::Float(_)          => "Float",
            Self::Text(_)           => "Text",
            Self::Bytes(_)          => "Bytes",
            Self::Uuid(_)           => "Uuid",
            Self::NationalId(_)     => "NationalId",
            Self::Phone(_)          => "Phone",
            Self::Timestamp(_)      => "Timestamp",
            Self::Date(_)           => "Date",
            Self::HijriDate(_)      => "HijriDate",
            Self::Duration(_)       => "Duration",
            Self::Coordinate(_)     => "Coordinate",
            Self::CountryCode(_)    => "CountryCode",
            Self::DomainByte(_)     => "DomainByte",
            Self::QualityScore(_)   => "QualityScore",
            Self::KakiPk(_)         => "KakiPk",
            Self::AkkadianRoot(_)   => "AkkadianRoot",
            Self::NameVector(_)     => "NameVector",
            Self::LangCode(_)       => "LangCode",
            Self::Embedding(_)      => "Embedding",
            Self::Probability(_)    => "Probability",
            Self::Label(_)          => "Label",
            Self::Confidence(_)     => "Confidence",
            Self::PipelineStatus(_) => "PipelineStatus",
            Self::StepIndex(_)      => "StepIndex",
            Self::PolicyVerdict(_)  => "PolicyVerdict",
            Self::CipherAlgorithm(_)=> "CipherAlgorithm",
            Self::SealSignature(_)  => "SealSignature",
            Self::List(_)           => "List",
        }
    }

    pub fn is_null(&self)  -> bool     { matches!(self, Self::Null) }
    pub fn as_bool(&self)  -> Option<bool>  { if let Self::Bool(v)  = self { Some(*v) } else { None } }
    pub fn as_int(&self)   -> Option<i64>   { if let Self::Int(v)   = self { Some(*v) } else { None } }
    pub fn as_float(&self) -> Option<f64>   { if let Self::Float(v) = self { Some(*v) } else { None } }
    pub fn as_text(&self)  -> Option<&str>  { if let Self::Text(v)  = self { Some(v)  } else { None } }
    pub fn as_bytes(&self) -> Option<&[u8]> { if let Self::Bytes(v) = self { Some(v)  } else { None } }
    pub fn as_uuid(&self)  -> Option<&[u8; 16]> { if let Self::Uuid(v) = self { Some(v) } else { None } }
    pub fn as_kaki_pk(&self) -> Option<&[u8; 16]> { if let Self::KakiPk(v) = self { Some(v) } else { None } }
    pub fn as_timestamp(&self) -> Option<i64> { if let Self::Timestamp(v) = self { Some(*v) } else { None } }
    pub fn as_coordinate(&self) -> Option<&AkkCoordinate> {
        if let Self::Coordinate(v) = self { Some(v) } else { None }
    }
}

impl Default for AkkValue { fn default() -> Self { Self::Null } }

impl fmt::Display for AkkValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null                 => write!(f, "∅"),
            Self::Bool(b)              => write!(f, "{b}"),
            Self::Int(i)               => write!(f, "{i}"),
            Self::Float(v)             => write!(f, "{v}"),
            Self::Text(s)              => write!(f, "{s}"),
            Self::Bytes(b)             => write!(f, "[{} bytes]", b.len()),
            Self::Uuid(u)              => write!(f, "{}", fmt_uuid(u)),
            Self::NationalId(n)        => write!(f, "{}/{}", n.country_code, n.number),
            Self::Phone(p)             => write!(f, "+{} {}", p.country_code, p.number),
            Self::Timestamp(ts)        => write!(f, "ts:{ts}"),
            Self::Date(d)              => write!(f, "{d}"),
            Self::HijriDate(h)         => write!(f, "{h}"),
            Self::Duration(s)          => write!(f, "{s}s"),
            Self::Coordinate(c)        => write!(f, "({},{})", c.lat, c.lon),
            Self::CountryCode(cc)      => write!(f, "cc:{cc}"),
            Self::DomainByte(b)        => write!(f, "dom:{b:#04x}"),
            Self::QualityScore(q)      => write!(f, "q:{q}"),
            Self::KakiPk(pk)           => write!(f, "kaki:{}", hex_16(pk)),
            Self::AkkadianRoot(r)      => write!(f, "root:{r}"),
            Self::NameVector(nv)       => write!(f, "name[{}]", nv.parts.len()),
            Self::LangCode(l)          => write!(f, "lang:{l}"),
            Self::Embedding(e)         => write!(f, "emb[{}]", e.len()),
            Self::Probability(p)       => write!(f, "p:{p:.4}"),
            Self::Label(l)             => write!(f, "lbl:{l}"),
            Self::Confidence(c)        => write!(f, "conf:{c:.4}"),
            Self::PipelineStatus(s)    => write!(f, "{s}"),
            Self::StepIndex(i)         => write!(f, "step:{i}"),
            Self::PolicyVerdict(v)     => write!(f, "{v}"),
            Self::CipherAlgorithm(a)   => write!(f, "{a}"),
            Self::SealSignature(s)     => write!(f, "seal[{}]", s.len()),
            Self::List(l)              => write!(f, "list[{}]", l.len()),
        }
    }
}

fn hex_16(b: &[u8; 16]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// RFC 4122 canonical UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
fn fmt_uuid(b: &[u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2],  b[3],
        b[4], b[5],
        b[6], b[7],
        b[8], b[9],
        b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_default() {
        assert!(AkkValue::default().is_null());
    }

    #[test]
    fn scalar_accessors() {
        assert_eq!(AkkValue::Bool(true).as_bool(), Some(true));
        assert_eq!(AkkValue::Int(42).as_int(), Some(42));
        assert_eq!(AkkValue::Float(3.14).as_float(), Some(3.14));
        assert_eq!(AkkValue::Text("hi".into()).as_text(), Some("hi"));
        assert_eq!(AkkValue::Bytes(vec![1, 2]).as_bytes(), Some([1u8, 2].as_slice()));
    }

    #[test]
    fn uuid_is_16_bytes() {
        let id = [1u8; 16];
        let v = AkkValue::Uuid(id);
        assert_eq!(v.as_uuid(), Some(&id));
        assert_eq!(v.type_name(), "Uuid");
    }

    #[test]
    fn uuid_display_has_dashes() {
        let id = [0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe,
                  0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];
        let s = AkkValue::Uuid(id).to_string();
        assert_eq!(s, "deadbeef-cafe-babe-0011-223344556677");
    }

    #[test]
    fn kaki_pk_display() {
        let kaki = [0u8; 16];
        let s = AkkValue::KakiPk(kaki).to_string();
        assert!(s.starts_with("kaki:"));
        assert_eq!(s.len(), "kaki:".len() + 32);
    }

    #[test]
    fn quality_score_in_range() {
        let q = AkkValue::QualityScore(240);
        assert_eq!(q.type_name(), "QualityScore");
        assert_eq!(q.to_string(), "q:240");
    }

    #[test]
    fn coordinate_display() {
        use crate::types::AkkCoordinate;
        let c = AkkValue::Coordinate(AkkCoordinate::new(32.0, 44.3));
        let s = c.to_string();
        assert!(s.contains("32") && s.contains("44.3"));
    }

    #[test]
    fn list_display_shows_length() {
        let lst = AkkValue::List(vec![AkkValue::Int(1), AkkValue::Int(2)]);
        assert_eq!(lst.to_string(), "list[2]");
    }

    #[test]
    fn timestamp_display() {
        let v = AkkValue::Timestamp(1_700_000_000);
        assert_eq!(v.to_string(), "ts:1700000000");
    }

    #[test]
    fn null_display_is_symbol() {
        assert_eq!(AkkValue::Null.to_string(), "∅");
    }

    #[test]
    fn all_type_names_non_empty() {
        let samples: &[AkkValue] = &[
            AkkValue::Null,
            AkkValue::Bool(false),
            AkkValue::Int(0),
            AkkValue::Float(0.0),
            AkkValue::Text(String::new()),
            AkkValue::Bytes(vec![]),
            AkkValue::Uuid([0u8; 16]),
            AkkValue::Timestamp(0),
            AkkValue::Duration(0),
            AkkValue::DomainByte(0),
            AkkValue::QualityScore(0),
            AkkValue::KakiPk([0u8; 16]),
            AkkValue::StepIndex(0),
        ];
        for v in samples {
            assert!(!v.type_name().is_empty(), "empty type_name for {v:?}");
        }
    }
}
