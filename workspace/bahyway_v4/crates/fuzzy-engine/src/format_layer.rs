// ============================================================================
// fuzzy-engine/src/format_layer.rs
// ADR-006 format layer multipliers
// serde removed — pure std (BahyWay v4 zero-dependency constraint)
// ============================================================================
#![allow(missing_docs)]

/// The 5 sovereign format layers (ADR-006).
/// Each layer carries a score multiplier that scales the raw B11 quality byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatLayer {
    /// Official civil registration data — highest confidence. Multiplier 1.0×.
    CivilRegistry,
    /// Medical / health record data. Multiplier 0.95×.
    MedicalRecord,
    /// Pipeline-ingested batch data. Multiplier 0.90×.
    PipelineData,
    /// Cuneiform / dictionary reference data. Multiplier 0.85×.
    CuneiformDictionary,
    /// Generic EAV catch-all. Multiplier 0.75×.
    GenericEav,
}

impl FormatLayer {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            FormatLayer::CivilRegistry => "CivilRegistry",
            FormatLayer::MedicalRecord => "MedicalRecord",
            FormatLayer::PipelineData => "PipelineData",
            FormatLayer::CuneiformDictionary => "CuneiformDictionary",
            FormatLayer::GenericEav => "GenericEav",
        }
    }

    /// Minimum required fuzzy dimensions for this layer (as dimension indices 1-based).
    /// Returns a bitmask: bit N-1 set ⟹ dimension N is required.
    pub fn required_dimensions(self) -> u8 {
        match self {
            // D1, D2, D3, D4, D6, D8 required (bits 0,1,2,3,5,7)
            FormatLayer::CivilRegistry => 0b_1010_1111,
            // D1, D2, D4, D6 required (bits 0,1,3,5)
            FormatLayer::MedicalRecord => 0b_0010_1011,
            // D1, D6 required (bits 0,5)
            FormatLayer::PipelineData => 0b_0010_0001,
            // D1, D2, D8 required (bits 0,1,7)
            FormatLayer::CuneiformDictionary => 0b_1000_0011,
            // D1 only (bit 0)
            FormatLayer::GenericEav => 0b_0000_0001,
        }
    }

    /// Score multiplier applied to the raw centroid defuzz result.
    pub fn score_multiplier(self) -> f32 {
        match self {
            FormatLayer::CivilRegistry => 1.00,
            FormatLayer::MedicalRecord => 0.95,
            FormatLayer::PipelineData => 0.90,
            FormatLayer::CuneiformDictionary => 0.85,
            FormatLayer::GenericEav => 0.75,
        }
    }

    /// All 5 layers in descending confidence order.
    pub fn all() -> [FormatLayer; 5] {
        [
            FormatLayer::CivilRegistry,
            FormatLayer::MedicalRecord,
            FormatLayer::PipelineData,
            FormatLayer::CuneiformDictionary,
            FormatLayer::GenericEav,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multipliers_in_descending_order() {
        let layers = FormatLayer::all();
        for i in 1..layers.len() {
            assert!(
                layers[i - 1].score_multiplier() > layers[i].score_multiplier(),
                "{:?} multiplier should exceed {:?}",
                layers[i - 1],
                layers[i]
            );
        }
    }

    #[test]
    fn civil_registry_requires_id_and_date() {
        let req = FormatLayer::CivilRegistry.required_dimensions();
        // D3 = bit 2, D4 = bit 3
        assert!(
            req & 0b_0000_0100 != 0,
            "CivilRegistry must require D3 (ID)"
        );
        assert!(
            req & 0b_0000_1000 != 0,
            "CivilRegistry must require D4 (date)"
        );
    }

    #[test]
    fn generic_eav_requires_only_d1() {
        let req = FormatLayer::GenericEav.required_dimensions();
        assert_eq!(req, 0b_0000_0001, "GenericEav requires only D1");
    }

    #[test]
    fn all_returns_five_distinct_layers() {
        let layers = FormatLayer::all();
        assert_eq!(layers.len(), 5);
        // All distinct
        for i in 0..layers.len() {
            for j in (i + 1)..layers.len() {
                assert_ne!(layers[i], layers[j]);
            }
        }
    }

    #[test]
    fn labels_are_non_empty() {
        for layer in FormatLayer::all() {
            assert!(!layer.label().is_empty());
        }
    }
}
