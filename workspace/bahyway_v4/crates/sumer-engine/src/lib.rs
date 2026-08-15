// ============================================================
// BahyWay.Ecosystem v4.0 — SumerEngine 𒁾
// The Sovereign Sumerian Layer: one crate, three projections
// of the same civilization — script, geography, mathematics —
// plus the Vec7DKey sovereign key derivation built on them.
//
// The same clay tablet carries cuneiform signs, a city, and
// mathematics. Separating them would be reductionist.
// Keeping them unified is the sovereign approach.
// ============================================================
#![forbid(unsafe_code)]

pub mod sign;
pub mod meszl;
pub mod wahshiyya;
pub mod geography;
pub mod key;
pub mod seed;

pub use sign::{fnv1a64, SignTradition, SovereignSign};
pub use meszl::MesZLSign;
pub use wahshiyya::{Planet, WahshiyyaSign};
pub use geography::{AncientCity, NajafSumerianContext, SumerianPeriod, ANCIENT_CITIES};
pub use key::{TraditionProfile, Vec7DKey, DEFAULT_STRETCH_ROUNDS};

// Re-export the single sovereign definitions from kinetic-engine.
pub use kinetic_engine::{HeptaDimension, PlimptonAnchors, Vec7D};

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn both_traditions_cover_all_seven_dimensions() {
        let meszl_dims: Vec<HeptaDimension> = meszl::dimensional_signs()
            .iter()
            .filter_map(|s| s.dimension)
            .collect();
        assert_eq!(meszl_dims.len(), 7);
        let wah_anchors = wahshiyya::planetary_anchor_signs();
        let wah_dims: Vec<HeptaDimension> =
            wah_anchors.iter().filter_map(|s| s.dimension()).collect();
        assert_eq!(wah_dims.len(), 7);
    }

    #[test]
    fn mixed_tradition_key_generation_end_to_end() {
        let dub = meszl::by_name("DUB").expect("DUB sign present");
        let sar = meszl::by_name("SAR").expect("SAR sign present");
        let anchors = wahshiyya::planetary_anchor_signs();
        let saturn = &anchors[0];
        let k = Vec7DKey::generate_with_rounds(dub, sar, saturn, 128);
        assert_eq!(k.profile, TraditionProfile::Mixed);
        assert_eq!(k.to_hex().len(), 64);
        assert!(k.verify_with_rounds(dub, sar, saturn, 128));
    }
}
