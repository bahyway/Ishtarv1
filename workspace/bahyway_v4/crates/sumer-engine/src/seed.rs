//! Seed manifest — the meszl.* EAV rows ready for enkidb-ingest.
//!
//! V4.0 LAW: this module emits ATTRIBUTE ROWS ONLY.
//!   * KAKI minting happens exclusively in enkidb-ingest::bridge,
//!     under the locked v4.0 canonical layout — never here.
//!   * There is NO meszl.quality attribute: the quality byte does
//!     not exist in v4.0. ColourID is GeoEngine EAV law.
//!   * Ingest path: manifest -> enkidb-ingest -> E-DUBBA seal ->
//!     KISPU-style atomic commit.

use crate::meszl::{MesZLSign, PHASE1_SIGNS};
use crate::sign::SovereignSign;

/// The EAV rows for one MesZL sign, keyed by attribute name.
pub fn meszl_eav_rows(sign: &MesZLSign) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> = vec![
        ("meszl.number".to_string(), sign.meszl.to_string()),
        ("meszl.glyph".to_string(), sign.glyph.to_string()),
        ("meszl.name_primary".to_string(), sign.name.to_string()),
        (
            "meszl.unicode_hex".to_string(),
            format!("U+{:X}", sign.glyph as u32),
        ),
        (
            "meszl.phonetic.0".to_string(),
            sign.phonetic_primary.to_string(),
        ),
        ("meszl.phonetic_count".to_string(), "1".to_string()),
        (
            "meszl.tradition".to_string(),
            format!("0x{:02X}", sign.tradition().tag()),
        ),
    ];
    if let Some(d) = sign.dimension {
        rows.push(("meszl.dimension".to_string(), d.name().to_string()));
        rows.push(("meszl.akkadianAOL".to_string(), "true".to_string()));
    }
    rows
}

/// The full Phase-1 manifest: (meszl_number, rows) per sign.
pub fn phase1_manifest() -> Vec<(u32, Vec<(String, String)>)> {
    PHASE1_SIGNS
        .iter()
        .map(|s| (s.meszl, meszl_eav_rows(s)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meszl;

    #[test]
    fn manifest_covers_all_phase1_signs() {
        let m = phase1_manifest();
        assert_eq!(m.len(), 15);
    }

    #[test]
    fn no_quality_attribute_anywhere_v4_law() {
        for (_, rows) in phase1_manifest() {
            for (key, _) in rows {
                assert!(
                    !key.contains("quality"),
                    "v4.0 law: the quality byte does not exist"
                );
                assert!(
                    !key.contains("colour") && !key.contains("color"),
                    "ColourID is GeoEngine EAV law — never seeded here"
                );
            }
        }
    }

    #[test]
    fn dimensional_signs_carry_dimension_rows() {
        let me = meszl::by_name("ME").unwrap();
        let rows = meszl_eav_rows(me);
        assert!(rows
            .iter()
            .any(|(k, v)| k == "meszl.dimension" && v == "ME"));
        assert!(rows
            .iter()
            .any(|(k, v)| k == "meszl.akkadianAOL" && v == "true"));
    }

    #[test]
    fn core_signs_carry_no_dimension_rows() {
        let an = meszl::by_name("AN").unwrap();
        let rows = meszl_eav_rows(an);
        assert!(!rows.iter().any(|(k, _)| k == "meszl.dimension"));
    }

    #[test]
    fn glyph_unicode_hex_is_correct_for_dub() {
        let dub = meszl::by_name("DUB").unwrap();
        let rows = meszl_eav_rows(dub);
        let hex = rows
            .iter()
            .find(|(k, _)| k == "meszl.unicode_hex")
            .map(|(_, v)| v.clone())
            .unwrap();
        assert_eq!(hex, format!("U+{:X}", dub.glyph as u32));
    }
}
