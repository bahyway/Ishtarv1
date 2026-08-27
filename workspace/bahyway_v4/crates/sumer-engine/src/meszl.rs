//! MesZL Phase-1 sign set — the 7 dimensional signs (ME GU SAG A
//! IZI UD URU) plus 8 core signs most referenced across BahyWay.
//!
//! MesZL reference numbers are PHASE-1 PROVISIONAL and must be
//! verified against Borger (2nd ed. 2004) before the Phase-2
//! 898-sign corpus (sourced from CDLI / ePSD2 / ORACC / Unicode,
//! all open — MesZL numbers used as reference identifiers only).

use crate::sign::{SignTradition, SovereignSign};
use kinetic_engine::HeptaDimension;

/// A Sumerian cuneiform sign in the MesZL tradition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MesZLSign {
    pub meszl: u32,
    pub glyph: char,
    pub name: &'static str,
    pub phonetic_primary: &'static str,
    pub dimension: Option<HeptaDimension>,
}

impl SovereignSign for MesZLSign {
    fn tradition(&self) -> SignTradition {
        SignTradition::MesZL
    }
    fn reference(&self) -> u32 {
        self.meszl
    }
    fn phonetic(&self) -> &str {
        self.phonetic_primary
    }
    fn dimension(&self) -> Option<HeptaDimension> {
        self.dimension
    }
}

/// The 15 Phase-1 signs.
pub const PHASE1_SIGNS: [MesZLSign; 15] = [
    // ── The seven dimensional signs ──────────────────────────
    MesZLSign {
        meszl: 532,
        glyph: '𒈨',
        name: "ME",
        phonetic_primary: "me",
        dimension: Some(HeptaDimension::ME),
    },
    MesZLSign {
        meszl: 559,
        glyph: '𒄖',
        name: "GU",
        phonetic_primary: "gu",
        dimension: Some(HeptaDimension::GU),
    },
    MesZLSign {
        meszl: 184,
        glyph: '𒊕',
        name: "SAG",
        phonetic_primary: "sag",
        dimension: Some(HeptaDimension::SAG),
    },
    MesZLSign {
        meszl: 839,
        glyph: '𒀀',
        name: "A",
        phonetic_primary: "a",
        dimension: Some(HeptaDimension::A),
    },
    MesZLSign {
        meszl: 172,
        glyph: '𒉈',
        name: "IZI",
        phonetic_primary: "izi",
        dimension: Some(HeptaDimension::IZI),
    },
    MesZLSign {
        meszl: 596,
        glyph: '𒌓',
        name: "UD",
        phonetic_primary: "ud",
        dimension: Some(HeptaDimension::UD),
    },
    MesZLSign {
        meszl: 71,
        glyph: '𒌷',
        name: "URU",
        phonetic_primary: "uru",
        dimension: Some(HeptaDimension::URU),
    },
    // ── The eight core signs ─────────────────────────────────
    MesZLSign {
        meszl: 10,
        glyph: '𒀭',
        name: "AN",
        phonetic_primary: "an",
        dimension: None,
    },
    MesZLSign {
        meszl: 242,
        glyph: '𒁾',
        name: "DUB",
        phonetic_primary: "dub",
        dimension: None,
    },
    MesZLSign {
        meszl: 541,
        glyph: '𒊬',
        name: "SAR",
        phonetic_primary: "sar",
        dimension: None,
    },
    MesZLSign {
        meszl: 164,
        glyph: '𒂗',
        name: "EN",
        phonetic_primary: "en",
        dimension: None,
    },
    MesZLSign {
        meszl: 737,
        glyph: '𒆠',
        name: "KI",
        phonetic_primary: "ki",
        dimension: None,
    },
    MesZLSign {
        meszl: 266,
        glyph: '𒈗',
        name: "LUGAL",
        phonetic_primary: "lugal",
        dimension: None,
    },
    MesZLSign {
        meszl: 495,
        glyph: '𒂍',
        name: "E2",
        phonetic_primary: "e",
        dimension: None,
    },
    MesZLSign {
        meszl: 143,
        glyph: '𒉣',
        name: "NUN",
        phonetic_primary: "nun",
        dimension: None,
    },
];

/// The seven dimensional signs, in ME..URU order.
pub fn dimensional_signs() -> Vec<MesZLSign> {
    PHASE1_SIGNS
        .iter()
        .filter(|s| s.dimension.is_some())
        .copied()
        .collect()
}

/// Look a Phase-1 sign up by its name.
pub fn by_name(name: &str) -> Option<&'static MesZLSign> {
    PHASE1_SIGNS.iter().find(|s| s.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifteen_signs_with_unique_references_and_glyphs() {
        let mut refs: Vec<u32> = PHASE1_SIGNS.iter().map(|s| s.meszl).collect();
        refs.sort_unstable();
        refs.dedup();
        assert_eq!(refs.len(), 15);
        let mut glyphs: Vec<char> = PHASE1_SIGNS.iter().map(|s| s.glyph).collect();
        glyphs.sort_unstable();
        glyphs.dedup();
        assert_eq!(glyphs.len(), 15);
    }

    #[test]
    fn all_references_within_meszl_range() {
        for s in PHASE1_SIGNS.iter() {
            assert!(s.meszl >= 1 && s.meszl <= 898);
        }
    }

    #[test]
    fn seven_dimensional_signs_cover_all_dimensions_in_order() {
        let dims: Vec<HeptaDimension> = dimensional_signs()
            .iter()
            .filter_map(|s| s.dimension)
            .collect();
        assert_eq!(dims, HeptaDimension::ALL.to_vec());
    }

    #[test]
    fn dub_and_sar_are_present_for_the_scribe() {
        // DUB.SAR 𒁾 — the identity of the Architect.
        assert!(by_name("DUB").is_some());
        assert!(by_name("SAR").is_some());
    }

    #[test]
    fn seeds_are_unique_and_position_sensitive() {
        let mut seeds: Vec<u64> = PHASE1_SIGNS.iter().map(|s| s.seed(0)).collect();
        seeds.sort_unstable();
        seeds.dedup();
        assert_eq!(seeds.len(), 15, "all Phase-1 seeds unique");
        let dub = by_name("DUB").unwrap();
        assert_ne!(dub.seed(0), dub.seed(1), "position changes the seed");
    }
}
