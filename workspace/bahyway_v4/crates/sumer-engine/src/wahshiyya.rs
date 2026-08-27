//! Ibn Wahshiyya — Iraqi scholar, Baghdad, ~930 CE.
//! From Shawq al-Mustaham: the seven planetary scripts, four
//! Phase-1 signs per planet (28 total). The planets bind to the
//! Heptagon dimensions:
//!   Saturn>ME  Jupiter>GU  Mars>SAG  Venus>A
//!   Sun>IZI    Mercury>UD  Moon>URU
//! Sign names use the Arabic planet names of the tradition:
//!   Zuhal, Mushtari, Mirrikh, Zuhara, Shams, Utarid, Qamar.

use crate::sign::{SignTradition, SovereignSign};
use kinetic_engine::HeptaDimension;

/// The seven classical planets of the Wahshiyya tradition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Planet {
    Saturn,
    Jupiter,
    Mars,
    Venus,
    Sun,
    Mercury,
    Moon,
}

impl Planet {
    pub const ALL: [Planet; 7] = [
        Planet::Saturn,
        Planet::Jupiter,
        Planet::Mars,
        Planet::Venus,
        Planet::Sun,
        Planet::Mercury,
        Planet::Moon,
    ];

    pub fn dimension(&self) -> HeptaDimension {
        match self {
            Planet::Saturn => HeptaDimension::ME,
            Planet::Jupiter => HeptaDimension::GU,
            Planet::Mars => HeptaDimension::SAG,
            Planet::Venus => HeptaDimension::A,
            Planet::Sun => HeptaDimension::IZI,
            Planet::Mercury => HeptaDimension::UD,
            Planet::Moon => HeptaDimension::URU,
        }
    }

    pub fn arabic_name(&self) -> &'static str {
        match self {
            Planet::Saturn => "zuhal",
            Planet::Jupiter => "mushtari",
            Planet::Mars => "mirrikh",
            Planet::Venus => "zuhara",
            Planet::Sun => "shams",
            Planet::Mercury => "utarid",
            Planet::Moon => "qamar",
        }
    }

    pub fn index(&self) -> u32 {
        match self {
            Planet::Saturn => 0,
            Planet::Jupiter => 1,
            Planet::Mars => 2,
            Planet::Venus => 3,
            Planet::Sun => 4,
            Planet::Mercury => 5,
            Planet::Moon => 6,
        }
    }
}

/// One Ibn Wahshiyya planetary sign.
#[derive(Debug, Clone, PartialEq)]
pub struct WahshiyyaSign {
    pub planet: Planet,
    /// Ordinal within the planet's script, 1..=4 (Phase 1).
    pub ordinal: u8,
    pub name: String,
}

impl WahshiyyaSign {
    pub fn new(planet: Planet, ordinal: u8) -> Self {
        assert!((1..=4).contains(&ordinal), "Phase-1 ordinal is 1..=4");
        let name = format!("{}-{}", planet.arabic_name(), ordinal);
        WahshiyyaSign {
            planet,
            ordinal,
            name,
        }
    }
}

impl SovereignSign for WahshiyyaSign {
    fn tradition(&self) -> SignTradition {
        SignTradition::IbnWahshiyya
    }
    fn reference(&self) -> u32 {
        self.planet.index() * 4 + self.ordinal as u32
    }
    fn phonetic(&self) -> &str {
        &self.name
    }
    fn dimension(&self) -> Option<HeptaDimension> {
        Some(self.planet.dimension())
    }
}

/// All 28 Phase-1 Wahshiyya signs (7 planets x 4).
pub fn all_signs() -> Vec<WahshiyyaSign> {
    let mut v = Vec::with_capacity(28);
    for p in Planet::ALL.iter() {
        for o in 1u8..=4u8 {
            v.push(WahshiyyaSign::new(*p, o));
        }
    }
    v
}

/// The 7 planetary anchor signs — ordinal 1 of each planet,
/// one per Heptagon dimension.
pub fn planetary_anchor_signs() -> Vec<WahshiyyaSign> {
    Planet::ALL
        .iter()
        .map(|p| WahshiyyaSign::new(*p, 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meszl;

    #[test]
    fn twenty_eight_signs_with_unique_references() {
        let signs = all_signs();
        assert_eq!(signs.len(), 28);
        let mut refs: Vec<u32> = signs.iter().map(|s| s.reference()).collect();
        refs.sort_unstable();
        refs.dedup();
        assert_eq!(refs.len(), 28);
    }

    #[test]
    fn planetary_anchors_cover_all_seven_dimensions() {
        let dims: Vec<HeptaDimension> = planetary_anchor_signs()
            .iter()
            .filter_map(|s| s.dimension())
            .collect();
        assert_eq!(dims, HeptaDimension::ALL.to_vec());
    }

    #[test]
    fn saturn_anchors_me_and_moon_anchors_uru() {
        assert_eq!(Planet::Saturn.dimension(), HeptaDimension::ME);
        assert_eq!(Planet::Moon.dimension(), HeptaDimension::URU);
    }

    #[test]
    fn cross_tradition_seeds_never_collide_in_phase1() {
        let wah: Vec<u64> = all_signs().iter().map(|s| s.seed(0)).collect();
        let mes: Vec<u64> = meszl::PHASE1_SIGNS.iter().map(|s| s.seed(0)).collect();
        for w in wah.iter() {
            assert!(
                !mes.contains(w),
                "Wahshiyya(0x11) != MesZL(0x10) guaranteed"
            );
        }
    }

    #[test]
    #[should_panic(expected = "Phase-1 ordinal")]
    fn ordinal_out_of_range_is_rejected() {
        let _ = WahshiyyaSign::new(Planet::Sun, 5);
    }
}
