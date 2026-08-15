#![forbid(unsafe_code)]
//! compute_dims — convert EAV triples + ClientDqProfile → FuzzyDimensions.
//!
//! This is the bridge between a client's configurable DQ profile and the
//! Mamdani fuzzy engine.  For each of the 8 sovereign dimensions:
//!
//!   D1 name completeness:   fraction of non-nullable Name-tagged attrs present
//!   D2 name consistency:    approximated as D1 (NLP not available in sovereign stack)
//!   D3 identity validity:   1.0 if all Identity-tagged mandatory attrs present
//!   D4 date validity:       1.0 / 0.5 based on Date-tagged attr presence
//!   D5 geo precision:       1.0 / 0.3 based on Geography-tagged attr presence
//!   D6 source trust:        from profile.source_trust
//!   D7 duplicate proximity: 0.0 (per-record dup check needs full DB scan — skipped)
//!   D8 arabic density:      fraction of Arabic codepoints across all EAV values

use fuzzy_engine::FuzzyDimensions;
use enkidb_journal::entry::EavTriple;

use crate::profile::{ClientDqProfile, DimTag};

/// Compute fuzzy input dimensions from one record's EAV set and the client profile.
pub fn compute_dims(eav: &[EavTriple], profile: &ClientDqProfile) -> FuzzyDimensions {
    let present: std::collections::HashSet<u32> =
        eav.iter().map(|t| t.attr_hash).collect();

    // D1 — name completeness: fraction of non-nullable Name attrs that are present
    let d1 = {
        let name_mandatory: Vec<u32> = profile.attrs.iter()
            .filter(|a| matches!(a.dim_tag, DimTag::Name) && !a.nullable)
            .map(|a| a.attr_hash)
            .collect();
        if name_mandatory.is_empty() {
            // No explicitly tagged Name columns; fall back to general completeness
            general_fill_rate(&present, profile)
        } else {
            name_mandatory.iter().filter(|h| present.contains(h)).count() as f32
                / name_mandatory.len() as f32
        }
    };

    // D2 — name consistency: can't compute Arabic BERT similarity in sovereign stack.
    // Approximate as D1 (same completeness score).
    let d2 = d1;

    // D3 — identity validity: all mandatory Identity attrs must be present (strict AND)
    let d3 = {
        let id_mandatory: Vec<u32> = profile.attrs.iter()
            .filter(|a| matches!(a.dim_tag, DimTag::Identity) && !a.nullable)
            .map(|a| a.attr_hash)
            .collect();
        if id_mandatory.is_empty() {
            1.0f32
        } else if id_mandatory.iter().all(|h| present.contains(h)) {
            1.0
        } else {
            let found = id_mandatory.iter().filter(|h| present.contains(h)).count();
            found as f32 / id_mandatory.len() as f32
        }
    };

    // D4 — date validity: any Date-tagged attr present = 1.0; none = 0.5 if optional, 0.0 if mandatory
    let d4 = {
        let date_attrs: Vec<_> = profile.attrs.iter()
            .filter(|a| matches!(a.dim_tag, DimTag::Date))
            .collect();
        if date_attrs.is_empty() {
            1.0f32 // no date columns configured → not penalised
        } else {
            let any_present = date_attrs.iter().any(|a| present.contains(&a.attr_hash));
            if any_present {
                1.0
            } else {
                // All date attrs absent — penalise if any is mandatory
                let has_mandatory = date_attrs.iter().any(|a| !a.nullable);
                if has_mandatory { 0.0 } else { 0.5 }
            }
        }
    };

    // D5 — geographic precision: similar logic to D4
    let d5 = {
        let geo_attrs: Vec<_> = profile.attrs.iter()
            .filter(|a| matches!(a.dim_tag, DimTag::Geography))
            .collect();
        if geo_attrs.is_empty() {
            1.0f32
        } else {
            let any_present = geo_attrs.iter().any(|a| present.contains(&a.attr_hash));
            if any_present {
                1.0
            } else {
                let has_mandatory = geo_attrs.iter().any(|a| !a.nullable);
                if has_mandatory { 0.0 } else { 0.3 }
            }
        }
    };

    // D6 — source trust: from the profile's organisational setting
    let d6 = profile.fuzzy_source_trust();

    // D7 — duplicate proximity: would require scanning the full DB; default 0.0 (assume unique)
    let d7 = 0.0f32;

    // D8 — Arabic density: fraction of Arabic Unicode codepoints across all EAV values
    let d8 = arabic_density_from_eav(eav);

    FuzzyDimensions {
        d1_name_completeness:   d1,
        d2_name_consistency:    d2,
        d3_id_validity:         d3,
        d4_date_validity:       d4,
        d5_geo_precision:       d5,
        d6_source_trust:        d6,
        d7_duplicate_proximity: d7,
        d8_arabic_density:      d8,
        d9_orbital_trust_penalty: 0.0,
    }
}

/// Fraction of non-nullable attrs (any dim) that are present in EAV.
fn general_fill_rate(
    present: &std::collections::HashSet<u32>,
    profile: &ClientDqProfile,
) -> f32 {
    let mandatory: Vec<u32> = profile.attrs.iter()
        .filter(|a| !a.nullable)
        .map(|a| a.attr_hash)
        .collect();
    if mandatory.is_empty() { return 1.0; }
    mandatory.iter().filter(|h| present.contains(h)).count() as f32
        / mandatory.len() as f32
}

/// Count Arabic Unicode codepoints (U+0600–U+06FF) across all EAV values.
fn arabic_density_from_eav(eav: &[EavTriple]) -> f32 {
    let mut total_chars = 0usize;
    let mut arabic_chars = 0usize;
    for triple in eav {
        if let Ok(s) = std::str::from_utf8(&triple.value) {
            for c in s.chars() {
                total_chars += 1;
                if c >= '\u{0600}' && c <= '\u{06FF}' {
                    arabic_chars += 1;
                }
            }
        }
    }
    if total_chars == 0 { 0.0 } else {
        (arabic_chars as f32 / total_chars as f32).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::ClientDqProfile;

    fn eav(pairs: &[(&str, &[u8])]) -> Vec<EavTriple> {
        pairs.iter().map(|(name, val)| {
            let h = {
                let mut h: u32 = 0x811C_9DC5;
                for &b in name.as_bytes() { h ^= b as u32; h = h.wrapping_mul(0x0100_0193); }
                h
            };
            EavTriple::new(h, val.to_vec())
        }).collect()
    }

    fn profile_cemetery() -> ClientDqProfile {
        ClientDqProfile::default_from_schema(
            "najaf_001",
            &["name".to_string(), "national_id".to_string()],
            &["date_of_death".to_string(), "cemetery_section".to_string(), "notes".to_string()],
        )
    }

    #[test]
    fn full_record_scores_high_d1_and_d3() {
        let p   = profile_cemetery();
        let eavs = eav(&[
            ("name",        b"Ali Karim"),
            ("national_id", b"190273649281"),
        ]);
        let dims = compute_dims(&eavs, &p);
        assert!(dims.d1_name_completeness >= 1.0, "name fully filled → d1 = 1.0");
        assert!(dims.d3_id_validity >= 1.0,       "id present → d3 = 1.0");
    }

    #[test]
    fn missing_mandatory_name_lowers_d1() {
        let p   = profile_cemetery();
        let eavs = eav(&[("national_id", b"190273649281")]); // name absent
        let dims = compute_dims(&eavs, &p);
        assert!(dims.d1_name_completeness < 1.0, "missing name must lower d1");
    }

    #[test]
    fn missing_optional_date_gives_partial_d4() {
        let p    = profile_cemetery();
        let eavs = eav(&[("name", b"Ali"), ("national_id", b"123")]); // no date_of_death
        let dims = compute_dims(&eavs, &p);
        assert!(dims.d4_date_validity < 1.0, "absent optional date → d4 < 1.0");
        assert!(dims.d4_date_validity > 0.0, "absent optional date → d4 > 0.0 (nullable)");
    }

    #[test]
    fn arabic_text_produces_high_d8() {
        let p    = profile_cemetery();
        let eavs = eav(&[
            ("name", "علي كريم".as_bytes()),
        ]);
        let dims = compute_dims(&eavs, &p);
        assert!(dims.d8_arabic_density > 0.5, "Arabic name → d8 > 0.5, got {}", dims.d8_arabic_density);
    }

    #[test]
    fn latin_only_text_gives_low_d8() {
        let p    = profile_cemetery();
        let eavs = eav(&[("name", b"Ali Karim")]);
        let dims = compute_dims(&eavs, &p);
        assert!(dims.d8_arabic_density < 0.1, "Latin-only name → d8 near 0");
    }

    #[test]
    fn empty_eav_produces_valid_dims() {
        let p    = profile_cemetery();
        let dims = compute_dims(&[], &p);
        // Should not panic; all dims are 0.0 or defaults
        assert!(dims.d1_name_completeness >= 0.0);
        assert!(dims.d3_id_validity       >= 0.0);
    }
}
