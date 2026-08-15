// ── Baghdad Heptagram Sectors ─────────────────────────────────────
// 7 sovereign sectors mapped to planetary symbols (H7-00 … H7-06).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BaghdadSector {
    GreenZone   = 0, // H7-00 SUN
    AlKadhimiya = 1, // H7-01 MOON
    SadrCity    = 2, // H7-02 MERCURY — reference density 1.00
    Karrada     = 3, // H7-03 VENUS
    Rashid      = 4, // H7-04 MARS
    AlJadria    = 5, // H7-05 JUPITER
    AlMansour   = 6, // H7-06 SATURN
}

impl BaghdadSector {
    pub const ALL: [BaghdadSector; 7] = [
        BaghdadSector::GreenZone,
        BaghdadSector::AlKadhimiya,
        BaghdadSector::SadrCity,
        BaghdadSector::Karrada,
        BaghdadSector::Rashid,
        BaghdadSector::AlJadria,
        BaghdadSector::AlMansour,
    ];

    /// Population density index relative to SadrCity (=1.00).
    pub fn density_index(self) -> f32 {
        match self {
            BaghdadSector::GreenZone   => 0.30,
            BaghdadSector::AlKadhimiya => 0.85,
            BaghdadSector::SadrCity    => 1.00,
            BaghdadSector::Karrada     => 0.75,
            BaghdadSector::Rashid      => 0.70,
            BaghdadSector::AlJadria    => 0.60,
            BaghdadSector::AlMansour   => 0.80,
        }
    }

    pub fn planet_symbol(self) -> &'static str {
        match self {
            BaghdadSector::GreenZone   => "SUN",
            BaghdadSector::AlKadhimiya => "MOON",
            BaghdadSector::SadrCity    => "MERCURY",
            BaghdadSector::Karrada     => "VENUS",
            BaghdadSector::Rashid      => "MARS",
            BaghdadSector::AlJadria    => "JUPITER",
            BaghdadSector::AlMansour   => "SATURN",
        }
    }

    pub fn heptagram_id(self) -> &'static str {
        match self {
            BaghdadSector::GreenZone   => "H7-00",
            BaghdadSector::AlKadhimiya => "H7-01",
            BaghdadSector::SadrCity    => "H7-02",
            BaghdadSector::Karrada     => "H7-03",
            BaghdadSector::Rashid      => "H7-04",
            BaghdadSector::AlJadria    => "H7-05",
            BaghdadSector::AlMansour   => "H7-06",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BaghdadSector::GreenZone   => "Green Zone",
            BaghdadSector::AlKadhimiya => "Al-Kadhimiya",
            BaghdadSector::SadrCity    => "Sadr City",
            BaghdadSector::Karrada     => "Karrada",
            BaghdadSector::Rashid      => "Rashid",
            BaghdadSector::AlJadria    => "Al-Jadria",
            BaghdadSector::AlMansour   => "Al-Mansour",
        }
    }

    /// Normalized tribe index [0.0, 1.0] for KAKI D2 encoding.
    pub fn norm(self) -> f32 {
        self as u8 as f32 / 6.0
    }

    /// KAKI tribe_id for this sector's deployment, reserved range
    /// 0x3000-0x3006 (2026-07-10) -- follows the precedent already set
    /// by EnkiduLLM (0x1001-0x10FF, 0x1200) and ESARHADDON
    /// (0x2000-0x20FF): each sovereign domain gets its own reserved
    /// tribe range rather than sharing the general particle namespace.
    pub fn tribe_id(self) -> u16 {
        0x3000 + self as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sectors_have_seven_members() {
        assert_eq!(BaghdadSector::ALL.len(), 7);
    }

    #[test]
    fn density_indices_in_range() {
        for s in BaghdadSector::ALL {
            let d = s.density_index();
            assert!(d >= 0.0 && d <= 1.0, "density {d} out of range for {:?}", s);
        }
    }

    #[test]
    fn sadr_city_is_maximum_density() {
        assert_eq!(BaghdadSector::SadrCity.density_index(), 1.00);
    }

    #[test]
    fn norm_in_range() {
        for s in BaghdadSector::ALL {
            let n = s.norm();
            assert!(n >= 0.0 && n <= 1.0);
        }
    }

    #[test]
    fn heptagram_ids_unique() {
        let ids: Vec<_> = BaghdadSector::ALL.iter().map(|s| s.heptagram_id()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len());
    }

    #[test]
    fn tribe_ids_are_unique_and_in_reserved_range() {
        let ids: Vec<u16> = BaghdadSector::ALL.iter().map(|s| s.tribe_id()).collect();
        for id in &ids {
            assert!((0x3000..=0x3006).contains(id), "tribe_id {id:#06x} outside reserved range");
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len());
    }

    #[test]
    fn green_zone_is_tribe_0x3000() {
        assert_eq!(BaghdadSector::GreenZone.tribe_id(), 0x3000);
    }
}
