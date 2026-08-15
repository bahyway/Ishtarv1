// ============================================================================
// compare-tribe-schema/src/schema_version.rs
// Versioned schema snapshot — one record per file/tribe version.
// Carries full field metadata for deep V1→V2 comparison.
// ============================================================================
#![allow(missing_docs)]

/// EAV field data types supported by the schema version model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Integer,
    Decimal,
    Date,
    Boolean,
    Binary,
}

impl FieldType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "text" | "varchar" | "string" | "char" => Some(FieldType::Text),
            "integer" | "int" | "bigint" | "smallint" | "number" => Some(FieldType::Integer),
            "decimal" | "float" | "numeric" | "real" | "double" => Some(FieldType::Decimal),
            "date" | "datetime" | "timestamp" => Some(FieldType::Date),
            "boolean" | "bool" => Some(FieldType::Boolean),
            "binary" | "blob" | "bytes" => Some(FieldType::Binary),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            FieldType::Text    => "Text",
            FieldType::Integer => "Integer",
            FieldType::Decimal => "Decimal",
            FieldType::Date    => "Date",
            FieldType::Boolean => "Boolean",
            FieldType::Binary  => "Binary",
        }
    }
}

/// Full metadata for one schema column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMeta {
    /// Column name (ASCII, used as attr_hash seed).
    pub name:       String,
    /// FNV-1a 32-bit hash of `name` — matches KakiGenerator's attr_hash convention.
    pub attr_hash:  u32,
    pub data_type:  FieldType,
    /// Max allowed length; None = unlimited / not applicable.
    pub max_length: Option<u16>,
    /// 1-based column position in the file.
    pub position:   u16,
    /// If true, absence flags a quality issue.
    pub mandatory:  bool,
    /// If true, NULL values are permitted.
    pub nullable:   bool,
}

impl FieldMeta {
    pub fn new(
        name:       impl Into<String>,
        data_type:  FieldType,
        max_length: Option<u16>,
        position:   u16,
        mandatory:  bool,
        nullable:   bool,
    ) -> Self {
        let name = name.into();
        let attr_hash = fnv1a_32(name.as_bytes());
        FieldMeta { name, attr_hash, data_type, max_length, position, mandatory, nullable }
    }

    /// Parse from a colon-separated annotation string:
    /// `name:type:length:position:mandatory:nullable`
    ///
    /// Example: `"national_id:integer:12:3:true:false"`
    /// Use `-` or `0` for unlimited/absent length.
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(6, ':').collect();
        if parts.len() < 6 {
            return Err(format!("need 6 colon-separated parts, got {} in: {s}", parts.len()));
        }
        let name = parts[0].trim().to_string();
        if name.is_empty() { return Err("field name is empty".to_string()); }

        let data_type = FieldType::from_str(parts[1])
            .ok_or_else(|| format!("unknown type '{}' for field '{name}'", parts[1]))?;

        let max_length = {
            let l = parts[2].trim();
            if l == "-" || l == "0" {
                None
            } else {
                Some(l.parse::<u16>().map_err(|_| format!("bad length '{l}' for field '{name}'"))?)
            }
        };

        let position = parts[3].trim().parse::<u16>()
            .map_err(|_| format!("bad position '{}' for field '{name}'", parts[3]))?;

        let mandatory = parts[4].trim().eq_ignore_ascii_case("true");
        let nullable  = parts[5].trim().eq_ignore_ascii_case("true");

        Ok(FieldMeta::new(name, data_type, max_length, position, mandatory, nullable))
    }
}

/// A complete versioned schema snapshot for one tribe/file.
#[derive(Debug, Clone)]
pub struct SchemaVersion {
    pub version:       u32,
    pub tribe_name:    String,
    /// Journal epoch when this version was captured.
    pub created_epoch: u32,
    pub fields:        Vec<FieldMeta>,
}

impl SchemaVersion {
    pub fn new(
        version:       u32,
        tribe_name:    impl Into<String>,
        created_epoch: u32,
        fields:        Vec<FieldMeta>,
    ) -> Self {
        SchemaVersion { version, tribe_name: tribe_name.into(), created_epoch, fields }
    }

    /// Find a field by exact column name.
    pub fn field_by_name(&self, name: &str) -> Option<&FieldMeta> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Number of mandatory fields.
    pub fn mandatory_count(&self) -> usize {
        self.fields.iter().filter(|f| f.mandatory).count()
    }

    /// Number of optional (non-mandatory) fields.
    pub fn optional_count(&self) -> usize {
        self.fields.iter().filter(|f| !f.mandatory).count()
    }
}

/// FNV-1a 32-bit — deterministic column name → attr_hash.
/// Same algorithm as KakiGenerator to ensure consistent attribute addressing.
pub(crate) fn fnv1a_32(data: &[u8]) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for &b in data {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn civil_v1() -> SchemaVersion {
        SchemaVersion::new(1, "civil.registry", 1000, vec![
            FieldMeta::new("national_id",   FieldType::Integer, Some(12), 1, true,  false),
            FieldMeta::new("full_name",     FieldType::Text,    Some(200), 2, true,  false),
            FieldMeta::new("birth_date",    FieldType::Date,    None,      3, true,  false),
            FieldMeta::new("governorate",   FieldType::Text,    Some(64),  4, false, true),
        ])
    }

    #[test]
    fn mandatory_optional_counts() {
        let v = civil_v1();
        assert_eq!(v.mandatory_count(), 3);
        assert_eq!(v.optional_count(),  1);
    }

    #[test]
    fn field_by_name_found() {
        let v = civil_v1();
        let f = v.field_by_name("national_id").unwrap();
        assert_eq!(f.data_type,  FieldType::Integer);
        assert_eq!(f.position,   1);
        assert!(f.mandatory);
    }

    #[test]
    fn field_by_name_missing() {
        let v = civil_v1();
        assert!(v.field_by_name("nonexistent").is_none());
    }

    #[test]
    fn parse_field_meta_round_trip() {
        let src = "national_id:integer:12:1:true:false";
        let f   = FieldMeta::parse(src).unwrap();
        assert_eq!(f.name,       "national_id");
        assert_eq!(f.data_type,  FieldType::Integer);
        assert_eq!(f.max_length, Some(12));
        assert_eq!(f.position,   1);
        assert!(f.mandatory);
        assert!(!f.nullable);
    }

    #[test]
    fn parse_field_meta_unlimited_length() {
        let f = FieldMeta::parse("bio:text:-:5:false:true").unwrap();
        assert_eq!(f.max_length, None);
        assert!(!f.mandatory);
        assert!(f.nullable);
    }

    #[test]
    fn parse_field_meta_bad_type_returns_err() {
        assert!(FieldMeta::parse("x:garbage:10:1:true:false").is_err());
    }

    #[test]
    fn attr_hash_is_deterministic() {
        let h1 = fnv1a_32(b"national_id");
        let h2 = fnv1a_32(b"national_id");
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_names_different_hashes() {
        assert_ne!(fnv1a_32(b"national_id"), fnv1a_32(b"full_name"));
    }
}
