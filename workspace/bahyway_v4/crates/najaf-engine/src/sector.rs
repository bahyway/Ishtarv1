//! NajafSector — the 7 sovereign zones of Wadi al-Salam cemetery.
//!
//! Each zone maps 1-to-1 onto the NaviEngine HeptaSector topology:
//!   Entrance=Centre(0)  Shuhadaa=North(1)    Awliya=NorthEast(2)
//!   Huffaz=SouthEast(3) Momineen=South(4)    Ulamaa=SouthWest(5)
//!   Anbiya=NorthWest(6)
//!
//! Sacred weights drive cost reduction for elevated zones; Entrance is neutral.
//! In the seven_node_map fixture: node_id = sector.index() + 1.

use navi_engine::HeptaSector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NajafSector {
    /// Main entrance — reception and orientation. HeptaSector::Centre.
    Entrance,
    /// Zone of the Martyrs (Al-Shuhadaa). HeptaSector::North.
    Shuhadaa,
    /// Zone of the Saints (Al-Awliya). HeptaSector::NorthEast.
    Awliya,
    /// Zone of the Memorisers of Quran (Al-Huffaz). HeptaSector::SouthEast.
    Huffaz,
    /// Zone of the Believers (Al-Momineen). HeptaSector::South.
    Momineen,
    /// Zone of the Scholars (Al-Ulamaa). HeptaSector::SouthWest.
    Ulamaa,
    /// Zone of the Prophets' lineage (Al-Anbiya). HeptaSector::NorthWest.
    Anbiya,
}

impl NajafSector {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => NajafSector::Entrance,
            1 => NajafSector::Shuhadaa,
            2 => NajafSector::Awliya,
            3 => NajafSector::Huffaz,
            4 => NajafSector::Momineen,
            5 => NajafSector::Ulamaa,
            6 => NajafSector::Anbiya,
            _ => NajafSector::Entrance,
        }
    }

    pub fn index(self) -> u8 {
        match self {
            NajafSector::Entrance => 0,
            NajafSector::Shuhadaa => 1,
            NajafSector::Awliya => 2,
            NajafSector::Huffaz => 3,
            NajafSector::Momineen => 4,
            NajafSector::Ulamaa => 5,
            NajafSector::Anbiya => 6,
        }
    }

    /// NaviGraph node id in the seven_node_map fixture: sector.index() + 1.
    pub fn fixture_node_id(self) -> u32 {
        self.index() as u32 + 1
    }

    pub fn to_hepta(self) -> HeptaSector {
        match self {
            NajafSector::Entrance => HeptaSector::Centre,
            NajafSector::Shuhadaa => HeptaSector::North,
            NajafSector::Awliya => HeptaSector::NorthEast,
            NajafSector::Huffaz => HeptaSector::SouthEast,
            NajafSector::Momineen => HeptaSector::South,
            NajafSector::Ulamaa => HeptaSector::SouthWest,
            NajafSector::Anbiya => HeptaSector::NorthWest,
        }
    }

    pub fn from_hepta(h: HeptaSector) -> Self {
        match h {
            HeptaSector::Centre => NajafSector::Entrance,
            HeptaSector::North => NajafSector::Shuhadaa,
            HeptaSector::NorthEast => NajafSector::Awliya,
            HeptaSector::SouthEast => NajafSector::Huffaz,
            HeptaSector::South => NajafSector::Momineen,
            HeptaSector::SouthWest => NajafSector::Ulamaa,
            HeptaSector::NorthWest => NajafSector::Anbiya,
        }
    }

    /// Sacred cost weight — lower = elevated pilgrimage priority.
    /// Entrance is neutral (1.0); martyrs and prophets receive reductions.
    pub fn sacred_weight(self) -> f32 {
        match self {
            NajafSector::Entrance => 1.00,
            NajafSector::Shuhadaa => 0.85,
            NajafSector::Awliya => 0.90,
            NajafSector::Huffaz => 0.95,
            NajafSector::Momineen => 1.00,
            NajafSector::Ulamaa => 0.92,
            NajafSector::Anbiya => 0.88,
        }
    }

    pub fn arabic_name(self) -> &'static str {
        match self {
            NajafSector::Entrance => "المدخل",
            NajafSector::Shuhadaa => "الشهداء",
            NajafSector::Awliya => "الأولياء",
            NajafSector::Huffaz => "الحفّاظ",
            NajafSector::Momineen => "المؤمنين",
            NajafSector::Ulamaa => "العلماء",
            NajafSector::Anbiya => "الأنبياء",
        }
    }

    pub fn english_name(self) -> &'static str {
        match self {
            NajafSector::Entrance => "Entrance",
            NajafSector::Shuhadaa => "Martyrs",
            NajafSector::Awliya => "Saints",
            NajafSector::Huffaz => "Memorisers",
            NajafSector::Momineen => "Believers",
            NajafSector::Ulamaa => "Scholars",
            NajafSector::Anbiya => "Prophets",
        }
    }

    pub fn all() -> [NajafSector; 7] {
        [
            NajafSector::Entrance,
            NajafSector::Shuhadaa,
            NajafSector::Awliya,
            NajafSector::Huffaz,
            NajafSector::Momineen,
            NajafSector::Ulamaa,
            NajafSector::Anbiya,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_roundtrip() {
        for i in 0..=6u8 {
            let s = NajafSector::from_u8(i);
            assert_eq!(s.index(), i);
        }
    }

    #[test]
    fn hepta_roundtrip() {
        for &s in NajafSector::all().iter() {
            assert_eq!(NajafSector::from_hepta(s.to_hepta()), s);
        }
    }

    #[test]
    fn fixture_node_id_offset_by_one() {
        for &s in NajafSector::all().iter() {
            assert_eq!(s.fixture_node_id(), s.index() as u32 + 1);
        }
    }

    #[test]
    fn sacred_weight_range() {
        for &s in NajafSector::all().iter() {
            let w = s.sacred_weight();
            assert!(w >= 0.80 && w <= 1.10, "weight {w} out of range for {s:?}");
        }
    }

    #[test]
    fn elevated_zones_below_one() {
        // Martyrs, Saints, Prophets get pilgrimage priority < 1.0
        assert!(NajafSector::Shuhadaa.sacred_weight() < 1.0);
        assert!(NajafSector::Awliya.sacred_weight() < 1.0);
        assert!(NajafSector::Anbiya.sacred_weight() < 1.0);
    }

    #[test]
    fn arabic_names_non_empty() {
        for &s in NajafSector::all().iter() {
            assert!(!s.arabic_name().is_empty(), "{s:?} has empty arabic name");
        }
    }

    #[test]
    fn english_names_non_empty() {
        for &s in NajafSector::all().iter() {
            assert!(!s.english_name().is_empty());
        }
    }

    #[test]
    fn from_u8_out_of_range_defaults_to_entrance() {
        assert_eq!(NajafSector::from_u8(99), NajafSector::Entrance);
    }

    #[test]
    fn all_returns_seven_sectors() {
        assert_eq!(NajafSector::all().len(), 7);
    }
}
