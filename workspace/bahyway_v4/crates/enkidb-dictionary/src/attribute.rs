//! DamaAttribute — a single entry in the TĒMĒNU dictionary.

/// The data type of an EAV attribute value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AttrDataType {
    /// Boolean (1 byte: 0x00 = false, 0x01 = true).
    Bool = 0x01,
    /// Signed 64-bit integer.
    Int64 = 0x02,
    /// IEEE 754 64-bit float.
    Float64 = 0x03,
    /// UTF-8 string (variable length, null-terminated in storage).
    Text = 0x04,
    /// 16-byte KAKI reference.
    KakiRef = 0x05,
    /// Raw bytes (domain-specific encoding).
    Bytes = 0x06,
    /// JSON-encoded value (for extensible SHU-GUR attributes).
    Json = 0x07,
}

impl AttrDataType {
    pub fn as_str(self) -> &'static str {
        match self {
            AttrDataType::Bool => "Bool",
            AttrDataType::Int64 => "Int64",
            AttrDataType::Float64 => "Float64",
            AttrDataType::Text => "Text",
            AttrDataType::KakiRef => "KakiRef",
            AttrDataType::Bytes => "Bytes",
            AttrDataType::Json => "Json",
        }
    }
}

/// A single attribute entry in the TĒMĒNU dictionary.
///
/// The `dama_id` is the u32 attr_hash — the primary key in both the
/// in-memory HashMap and the on-disk EAV index (Index 6, §9.3).
#[derive(Clone, Debug)]
pub struct DamaAttribute {
    /// CRC-16 of the canonical name string (zero-padded to u32).
    pub dama_id: u32,
    /// Canonical human-readable name (ASCII, snake_case).
    pub name: &'static str,
    /// The data type of this attribute's values.
    pub data_type: AttrDataType,
    /// Whether this attribute is mandatory (ILKUM core) or optional (SHU-GUR).
    pub mandatory: bool,
    /// Human-readable description for the MUMMU brain and DubSar IDE.
    pub description: &'static str,
}

impl DamaAttribute {
    pub const fn new(
        dama_id: u32,
        name: &'static str,
        data_type: AttrDataType,
        mandatory: bool,
        description: &'static str,
    ) -> Self {
        Self {
            dama_id,
            name,
            data_type,
            mandatory,
            description,
        }
    }
}
