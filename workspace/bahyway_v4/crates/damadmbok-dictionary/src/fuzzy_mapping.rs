//! FuzzyDimension ↔ DAMA-DMBOK term mapping.
//!
//! Maps each of BahyWay's 8 fuzzy quality dimensions (D1–D8) to its
//! corresponding DAMA-DMBOK term code, BahyWay field, and description.

/// A mapping from a FuzzyEngine dimension to a DAMA-DMBOK term.
#[derive(Debug, Clone, Copy)]
pub struct DimensionMapping {
    /// Dimension index (1–8).
    pub dimension:       u8,
    /// DAMA-DMBOK term code (e.g. "DQ.COMPLETENESS").
    pub dama_code:       &'static str,
    /// The BahyWay field or concept being scored.
    pub bahyway_field:   &'static str,
    /// Human-readable description of what this dimension measures.
    pub description:     &'static str,
}

/// All 8 fuzzy dimension mappings.
pub static DIMENSION_MAPPINGS: &[DimensionMapping] = &[
    DimensionMapping {
        dimension:     1,
        dama_code:     "DQ.COMPLETENESS",
        bahyway_field: "particle.name / required_fields",
        description:   "Scores 0.0–1.0 based on presence and non-emptiness of all required name fields.",
    },
    DimensionMapping {
        dimension:     2,
        dama_code:     "DQ.CONSISTENCY",
        bahyway_field: "arabic_bert_similarity",
        description:   "Measures cross-field semantic consistency using Arabic BERT cosine similarity.",
    },
    DimensionMapping {
        dimension:     3,
        dama_code:     "DQ.VALIDITY",
        bahyway_field: "identity_id_format",
        description:   "Validates national ID / identifier format against tribe-specific regex rules.",
    },
    DimensionMapping {
        dimension:     4,
        dama_code:     "DQ.VALIDITY",
        bahyway_field: "date_fields",
        description:   "Validates date fields for correct format (ISO-8601) and logical range (not future birth).",
    },
    DimensionMapping {
        dimension:     5,
        dama_code:     "DQ.ACCURACY",
        bahyway_field: "reference_data_match",
        description:   "Cross-references attribute values against authoritative reference data tables.",
    },
    DimensionMapping {
        dimension:     6,
        dama_code:     "DQ.FRESHNESS",
        bahyway_field: "last_update_epoch",
        description:   "Applies exponential decay δₜ = e^−λt to score recency of the last update event.",
    },
    DimensionMapping {
        dimension:     7,
        dama_code:     "DQ.UNIQUENESS",
        bahyway_field: "duplicate_proximity_score",
        description:   "Inverted duplicate detection score — high proximity to existing records lowers score.",
    },
    DimensionMapping {
        dimension:     8,
        dama_code:     "DQ.CONFORMITY",
        bahyway_field: "template_schema_conformity",
        description:   "Scores adherence to the registered VGCA template schema for the particle type.",
    },
];

/// Look up a dimension mapping by dimension number (1-based).
pub fn dimension_to_dama(d: u8) -> Option<&'static DimensionMapping> {
    DIMENSION_MAPPINGS.iter().find(|m| m.dimension == d)
}

/// Return all dimension mappings for a given DAMA code.
pub fn dimensions_for_dama(dama_code: &str) -> Vec<&'static DimensionMapping> {
    DIMENSION_MAPPINGS.iter().filter(|m| m.dama_code == dama_code).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_8_dimensions_present() {
        assert_eq!(DIMENSION_MAPPINGS.len(), 8);
        for i in 1u8..=8 {
            assert!(
                dimension_to_dama(i).is_some(),
                "Missing dimension D{i}"
            );
        }
    }

    #[test]
    fn dimension_1_is_completeness() {
        let m = dimension_to_dama(1).unwrap();
        assert_eq!(m.dama_code, "DQ.COMPLETENESS");
    }

    #[test]
    fn dimension_7_is_uniqueness() {
        let m = dimension_to_dama(7).unwrap();
        assert_eq!(m.dama_code, "DQ.UNIQUENESS");
    }

    #[test]
    fn dimension_out_of_range_returns_none() {
        assert!(dimension_to_dama(0).is_none());
        assert!(dimension_to_dama(9).is_none());
    }

    #[test]
    fn dimensions_for_validity_finds_d3_and_d4() {
        let hits = dimensions_for_dama("DQ.VALIDITY");
        assert_eq!(hits.len(), 2);
        let nums: Vec<u8> = hits.iter().map(|m| m.dimension).collect();
        assert!(nums.contains(&3));
        assert!(nums.contains(&4));
    }

    #[test]
    fn all_mappings_have_non_empty_fields() {
        for m in DIMENSION_MAPPINGS {
            assert!(!m.dama_code.is_empty());
            assert!(!m.bahyway_field.is_empty());
            assert!(!m.description.is_empty());
        }
    }
}
