#![forbid(unsafe_code)]
//! ClientDqProfile and AttrProfile — client-defined DQ rules per attribute.

use fuzzy_engine::{FormatLayer, SourceTrust};

/// Which FuzzyDimension an attribute contributes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimTag {
    /// D1 — name completeness (full name, family name, etc.)
    Name,
    /// D3 — identity validity (national ID, UUID, etc.)
    Identity,
    /// D4 — date validity (date of death, birth date, etc.)
    Date,
    /// D5 — geographic precision (cemetery, region, city, etc.)
    Geography,
    /// D8 — text / Arabic content density
    Content,
    /// No specific dimension — contributes to overall fill completeness only.
    General,
}

impl DimTag {
    pub fn label(self) -> &'static str {
        match self {
            DimTag::Name => "Name",
            DimTag::Identity => "Identity",
            DimTag::Date => "Date",
            DimTag::Geography => "Geography",
            DimTag::Content => "Content",
            DimTag::General => "General",
        }
    }

    pub fn parse_dim_tag(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "name" => DimTag::Name,
            "identity" => DimTag::Identity,
            "date" => DimTag::Date,
            "geography" => DimTag::Geography,
            "content" => DimTag::Content,
            _ => DimTag::General,
        }
    }

    /// Auto-detect DimTag from a column name heuristic.
    pub fn detect(col_name: &str) -> Self {
        let n = col_name.to_lowercase();
        if n.contains("name") || n == "full_name" || n.contains("ism") {
            return DimTag::Name;
        }
        if n.contains("id") || n.contains("uid") || n == "national_id" || n == "identifier" {
            return DimTag::Identity;
        }
        if n.contains("date") || n.contains("birth") || n.contains("death") || n.contains("year") {
            return DimTag::Date;
        }
        if n.contains("location")
            || n.contains("cemetery")
            || n.contains("region")
            || n.contains("zone")
            || n.contains("city")
            || n.contains("address")
            || n.contains("section")
            || n.contains("row")
            || n.contains("plot")
        {
            return DimTag::Geography;
        }
        if n.contains("note")
            || n.contains("remark")
            || n.contains("comment")
            || n.contains("description")
            || n.contains("text")
        {
            return DimTag::General;
        }
        DimTag::General
    }
}

/// Client-level format layer codes (maps to fuzzy_engine::FormatLayer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatLayerCode {
    CivilRegistry = 0,
    MedicalRecord = 1,
    PipelineData = 2,
    CuneiformDictionary = 3,
    GenericEav = 4,
}

impl FormatLayerCode {
    pub fn to_format_layer(self) -> FormatLayer {
        match self {
            FormatLayerCode::CivilRegistry => FormatLayer::CivilRegistry,
            FormatLayerCode::MedicalRecord => FormatLayer::MedicalRecord,
            FormatLayerCode::PipelineData => FormatLayer::PipelineData,
            FormatLayerCode::CuneiformDictionary => FormatLayer::CuneiformDictionary,
            FormatLayerCode::GenericEav => FormatLayer::GenericEav,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FormatLayerCode::CivilRegistry => "CivilRegistry",
            FormatLayerCode::MedicalRecord => "MedicalRecord",
            FormatLayerCode::PipelineData => "PipelineData",
            FormatLayerCode::CuneiformDictionary => "CuneiformDictionary",
            FormatLayerCode::GenericEav => "GenericEav",
        }
    }

    pub fn parse_format_layer_code(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "civilregistry" | "civil_registry" | "civil" => FormatLayerCode::CivilRegistry,
            "medicalrecord" | "medical_record" | "medical" => FormatLayerCode::MedicalRecord,
            "pipelinedata" | "pipeline_data" | "pipeline" => FormatLayerCode::PipelineData,
            "cuneiformdictionary" | "cuneiform" => FormatLayerCode::CuneiformDictionary,
            _ => FormatLayerCode::GenericEav,
        }
    }

    pub fn all() -> [FormatLayerCode; 5] {
        [
            FormatLayerCode::CivilRegistry,
            FormatLayerCode::MedicalRecord,
            FormatLayerCode::PipelineData,
            FormatLayerCode::CuneiformDictionary,
            FormatLayerCode::GenericEav,
        ]
    }
}

/// Client-level source trust codes (maps to fuzzy_engine::SourceTrust).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTrustLevel {
    Authoritative = 0,
    Official = 1,
    Community = 2,
    Unknown = 3,
}

impl SourceTrustLevel {
    pub fn to_source_trust(self) -> SourceTrust {
        match self {
            SourceTrustLevel::Authoritative => SourceTrust::Authoritative,
            SourceTrustLevel::Official => SourceTrust::Official,
            SourceTrustLevel::Community => SourceTrust::Community,
            SourceTrustLevel::Unknown => SourceTrust::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SourceTrustLevel::Authoritative => "Authoritative",
            SourceTrustLevel::Official => "Official",
            SourceTrustLevel::Community => "Community",
            SourceTrustLevel::Unknown => "Unknown",
        }
    }

    pub fn parse_source_trust_level(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "authoritative" => SourceTrustLevel::Authoritative,
            "official" => SourceTrustLevel::Official,
            "community" => SourceTrustLevel::Community,
            _ => SourceTrustLevel::Unknown,
        }
    }
}

/// Per-attribute DQ configuration set by the client.
#[derive(Debug, Clone)]
pub struct AttrProfile {
    /// Column name from the CSV header.
    pub attr_name: String,
    /// FNV-1a hash of attr_name — matches EavTriple.attr_hash.
    pub attr_hash: u32,
    /// If true: empty value is acceptable (optional column). If false: empty = DQ failure.
    pub nullable: bool,
    /// Minimum fraction of records across the batch that must have this attr (0.0–1.0).
    /// Only used for batch-level SLA reporting; per-record scoring uses `nullable`.
    pub fill_threshold: f32,
    /// Which fuzzy dimension this attribute contributes to.
    pub dim_tag: DimTag,
    /// Relative weight of this attribute in the overall quality score (0.0–1.0).
    pub weight: f32,
}

impl AttrProfile {
    pub fn new(attr_name: &str, nullable: bool) -> Self {
        let attr_hash = fnv1a_32(attr_name.as_bytes());
        let dim_tag = DimTag::detect(attr_name);
        AttrProfile {
            attr_name: attr_name.to_string(),
            attr_hash,
            nullable,
            fill_threshold: if nullable { 0.0 } else { 1.0 },
            dim_tag,
            weight: 1.0,
        }
    }
}

/// Full client DQ profile for one batch.
#[derive(Debug, Clone)]
pub struct ClientDqProfile {
    /// Base batch name (ZIP filename without extension).
    pub batch_name: String,
    /// Per-attribute rules.
    pub attrs: Vec<AttrProfile>,
    /// B11 threshold for Golden (Gem tier). Default: 200.
    pub gem_threshold: u8,
    /// B11 threshold for Tribe tier. Default: 140.
    pub tribe_threshold: u8,
    /// Format layer for score multiplier.
    pub format_layer: FormatLayerCode,
    /// Source trust level.
    pub source_trust: SourceTrustLevel,
}

impl ClientDqProfile {
    /// Build a default profile from BatchSchema output.
    /// Mandatory attrs → nullable=false. Optional attrs → nullable=true.
    /// DimTag auto-detected from column names.
    pub fn default_from_schema(
        batch_name: &str,
        mandatory_attrs: &[String],
        optional_attrs: &[String],
    ) -> Self {
        let mut attrs = Vec::with_capacity(mandatory_attrs.len() + optional_attrs.len());
        for col in mandatory_attrs {
            attrs.push(AttrProfile::new(col, false));
        }
        for col in optional_attrs {
            attrs.push(AttrProfile::new(col, true));
        }
        ClientDqProfile {
            batch_name: batch_name.to_string(),
            attrs,
            gem_threshold: 200,
            tribe_threshold: 140,
            format_layer: FormatLayerCode::CivilRegistry,
            source_trust: SourceTrustLevel::Official,
        }
    }

    /// Return the FormatLayer for the fuzzy engine.
    pub fn fuzzy_format_layer(&self) -> FormatLayer {
        self.format_layer.to_format_layer()
    }

    /// Return the SourceTrust for the fuzzy engine.
    pub fn fuzzy_source_trust(&self) -> SourceTrust {
        self.source_trust.to_source_trust()
    }

    /// Look up an AttrProfile by attr_hash. O(n) — profiles are small.
    pub fn find_attr(&self, attr_hash: u32) -> Option<&AttrProfile> {
        self.attrs.iter().find(|a| a.attr_hash == attr_hash)
    }

    /// Return all non-nullable attrs (mandatory for DQ).
    pub fn mandatory_attrs(&self) -> impl Iterator<Item = &AttrProfile> {
        self.attrs.iter().filter(|a| !a.nullable)
    }

    /// Return all nullable attrs (optional — empty is acceptable).
    pub fn optional_attrs(&self) -> impl Iterator<Item = &AttrProfile> {
        self.attrs.iter().filter(|a| a.nullable)
    }
}

fn fnv1a_32(data: &[u8]) -> u32 {
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

    #[test]
    fn dim_detect_name_column() {
        assert_eq!(DimTag::detect("first_name"), DimTag::Name);
        assert_eq!(DimTag::detect("father_name"), DimTag::Name);
        assert_eq!(DimTag::detect("full_name"), DimTag::Name);
    }

    #[test]
    fn dim_detect_identity_column() {
        assert_eq!(DimTag::detect("national_id"), DimTag::Identity);
        assert_eq!(DimTag::detect("uid"), DimTag::Identity);
    }

    #[test]
    fn dim_detect_date_column() {
        assert_eq!(DimTag::detect("date_of_death"), DimTag::Date);
        assert_eq!(DimTag::detect("birth_year"), DimTag::Date);
    }

    #[test]
    fn dim_detect_geography() {
        assert_eq!(DimTag::detect("cemetery_section"), DimTag::Geography);
        assert_eq!(DimTag::detect("city"), DimTag::Geography);
    }

    #[test]
    fn default_profile_classifies_correctly() {
        let mandatory = vec!["name".to_string(), "national_id".to_string()];
        let optional = vec!["date_of_death".to_string(), "notes".to_string()];
        let p = ClientDqProfile::default_from_schema("batch_001", &mandatory, &optional);

        assert_eq!(p.attrs.len(), 4);
        let name_attr = p.find_attr(fnv1a_32(b"name")).unwrap();
        assert!(!name_attr.nullable, "name must be non-nullable");

        let notes_attr = p.find_attr(fnv1a_32(b"notes")).unwrap();
        assert!(notes_attr.nullable, "notes must be nullable");
    }

    #[test]
    fn format_layer_bridges_correctly() {
        let p = ClientDqProfile::default_from_schema("x", &[], &[]);
        assert_eq!(p.fuzzy_format_layer(), FormatLayer::CivilRegistry);
    }
}
