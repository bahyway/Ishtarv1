#![forbid(unsafe_code)]

// ============================================================================
// hepta/src/kaki.rs — KAKI_7D 16-byte sovereign spatial primary key
//
// BIT LAYOUT (big-endian, 7 spatial dimensions + 32-bit sovereign identity hash):
//
//  Bits  0–17  (18 bits) → D1: Longitude   fixed-point, 0=−180°, 262143=+180°
//  Bits 18–35  (18 bits) → D2: Latitude    fixed-point, 0=−90°,  262143=+90°
//  Bits 36–53  (18 bits) → D3: Elevation   offset binary, −131071..+131071 m
//  Bits 54–71  (18 bits) → D4: Temporal    Unix epoch >> 6, relative to year 2000
//  Bits 72–79   (8 bits) → D5: SemanticTier  0–255
//  Bits 80–89  (10 bits) → D6: HeptaSector  3+3+3+1 bits (primary/sub1/sub2/reserved)
//  Bits 90–95   (6 bits) → D7: Density      0=perimeter, 63=core
//  Bits 96–127 (32 bits) → Identity Hash    FNV-1a 32-bit of geo_bytes || project_id
//
// Geo bytes 0..11 pack D1–D7 (96 bits); bytes 12..15 = identity hash.
// ============================================================================

use crate::error::HeptaError;
use core::fmt;

// ─── Epoch constant ──────────────────────────────────────────────────────────

/// Year-2000 origin offset: Unix 946_684_800 seconds, right-shifted 6 bits.
const EPOCH_OFFSET: u64 = 946_684_800 >> 6;

// ─── Coordinate System Tag ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordSystem {
    Kaki7d,       // KAKI_7D
    Kaki7dSacred, // KAKI_7D_SACRED
    Wgs84,        // WGS84
    Procedural,   // PROCEDURAL
}

impl CoordSystem {
    // Used only via this crate's own dot-call convention (both here and in
    // parser.rs), not the FromStr trait -- renaming or implementing FromStr
    // are both real API decisions, not mechanical fixes.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, HeptaError> {
        match s {
            "KAKI_7D" => Ok(Self::Kaki7d),
            "KAKI_7D_SACRED" => Ok(Self::Kaki7dSacred),
            "WGS84" => Ok(Self::Wgs84),
            "PROCEDURAL" => Ok(Self::Procedural),
            other => Err(HeptaError::UnknownCoordSystem(other.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Kaki7d => "KAKI_7D",
            Self::Kaki7dSacred => "KAKI_7D_SACRED",
            Self::Wgs84 => "WGS84",
            Self::Procedural => "PROCEDURAL",
        }
    }
}

// ─── Planet Node ─────────────────────────────────────────────────────────────

/// The 7 Sumerian celestial bodies plus the ANCHOR origin.
/// Index 0 is the ANCHOR (Sun/Utu) — origin of the heptagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlanetNode {
    Sun = 0,     // Utu     — ANCHOR origin
    Moon = 1,    // Nanna
    Mercury = 2, // Enki
    Venus = 3,   // Inanna
    Mars = 4,    // Nergal
    Jupiter = 5, // Enlil
    Saturn = 6,  // Ninurta
    Void = 7,    // Domain-specific or structural node
}

impl PlanetNode {
    /// Map a planet label string to a PlanetNode (domain labels → Void).
    pub fn from_label(s: &str) -> Self {
        match s {
            "SUN" => Self::Sun,
            "MOON" => Self::Moon,
            "MERCURY" => Self::Mercury,
            "VENUS" => Self::Venus,
            "MARS" => Self::Mars,
            "JUPITER" => Self::Jupiter,
            "SATURN" => Self::Saturn,
            _ => Self::Void,
        }
    }

    pub fn from_index(idx: u8) -> Self {
        match idx % 7 {
            0 => Self::Sun,
            1 => Self::Moon,
            2 => Self::Mercury,
            3 => Self::Venus,
            4 => Self::Mars,
            5 => Self::Jupiter,
            6 => Self::Saturn,
            _ => Self::Void,
        }
    }

    pub fn sector_index(self) -> u8 {
        self as u8
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sun => "SUN",
            Self::Moon => "MOON",
            Self::Mercury => "MERCURY",
            Self::Venus => "VENUS",
            Self::Mars => "MARS",
            Self::Jupiter => "JUPITER",
            Self::Saturn => "SATURN",
            Self::Void => "VOID",
        }
    }

    /// {7/2}: connect to the node 2 steps clockwise.
    pub fn chord_72_target(self) -> Self {
        Self::from_index((self as u8 + 2) % 7)
    }

    /// {7/3}: connect to the node 3 steps clockwise.
    pub fn chord_73_target(self) -> Self {
        Self::from_index((self as u8 + 3) % 7)
    }
}

// ─── Chord Type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordType {
    /// {7/2} — skips 1 vertex, ~9.9% path reduction vs perimeter walk.
    Chord72,
    /// {7/3} — skips 2 vertices, ~25.1% path reduction vs perimeter walk.
    Chord73,
}

impl ChordType {
    pub fn from_step(step: u8) -> Result<Self, HeptaError> {
        match step {
            2 => Ok(Self::Chord72),
            3 => Ok(Self::Chord73),
            s => Err(HeptaError::InvalidChordStep(s)),
        }
    }

    /// Path reduction factor relative to walking the same number of perimeter segments.
    /// {7/2}: saves (2×seg − chord72) / (2×seg) ≈ 9.9%
    /// {7/3}: saves (3×seg − chord73) / (3×seg) ≈ 25.1%
    pub fn path_reduction_factor(self) -> f64 {
        match self {
            Self::Chord72 => 0.099,
            Self::Chord73 => 0.251,
        }
    }
}

// ─── Sector Index (D6) ───────────────────────────────────────────────────────

/// 10-bit hierarchical sector index.
/// primary (3 bits, 0–7): planet hub
/// sub1    (3 bits, 0–7): first Hobble-Zoom level (0 = not zoomed)
/// sub2    (3 bits, 0–7): second Hobble-Zoom level (0 = not zoomed)
/// reserved(1 bit):  always 0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeptaSector {
    pub primary: u8,
    pub sub1: u8,
    pub sub2: u8,
}

impl HeptaSector {
    pub fn new(primary: u8, sub1: u8, sub2: u8) -> Result<Self, HeptaError> {
        if primary > 7 {
            return Err(HeptaError::SectorOutOfRange(primary));
        }
        if sub1 > 7 {
            return Err(HeptaError::SectorOutOfRange(sub1));
        }
        if sub2 > 7 {
            return Err(HeptaError::SectorOutOfRange(sub2));
        }
        Ok(Self {
            primary,
            sub1,
            sub2,
        })
    }

    pub fn root(primary: u8) -> Result<Self, HeptaError> {
        Self::new(primary, 0, 0)
    }

    /// Pack into 10 bits: [primary(3)][sub1(3)][sub2(3)][reserved(1)]
    pub fn to_bits(self) -> u16 {
        ((self.primary as u16) << 7) | ((self.sub1 as u16) << 4) | ((self.sub2 as u16) << 1)
    }

    pub fn from_bits(bits: u16) -> Self {
        Self {
            primary: ((bits >> 7) & 0b111) as u8,
            sub1: ((bits >> 4) & 0b111) as u8,
            sub2: ((bits >> 1) & 0b111) as u8,
        }
    }

    /// Zoom into sub-sector `idx` at the deepest unfilled level.
    /// sub1 is filled first, then sub2. A third zoom returns MaxZoomReached.
    pub fn zoom(&self, idx: u8) -> Result<Self, HeptaError> {
        if idx > 7 {
            return Err(HeptaError::SectorOutOfRange(idx));
        }
        if self.sub1 == 0 {
            Self::new(self.primary, idx, 0)
        } else if self.sub2 == 0 {
            Self::new(self.primary, self.sub1, idx)
        } else {
            Err(HeptaError::MaxZoomReached)
        }
    }

    pub fn planet(self) -> PlanetNode {
        PlanetNode::from_index(self.primary)
    }
}

// ─── KAKI_7D Primary Key ─────────────────────────────────────────────────────

/// 16-byte (128-bit) sovereign spatial primary key.
/// Bytes  0..11 = 96-bit packed geometric dimensions (D1–D7).
/// Bytes 12..15 = 32-bit sovereign identity hash (FNV-1a of geo || project_id).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Kaki7dPk([u8; 16]);

impl Kaki7dPk {
    // ── Construction ────────────────────────────────────────────────────────

    /// Each parameter is a distinct required dimension of the 128-bit key
    /// (D1-D7 plus the project-id salt) -- not artificial padding, so
    /// bundling into a params struct is a real API-design tradeoff rather
    /// than a mechanical fix.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lon_deg: f64,
        lat_deg: f64,
        elev_m: f64,
        epoch_sec: u64,
        tier: u8,
        sector: HeptaSector,
        density: u8,
        project_id: &str,
    ) -> Result<Self, HeptaError> {
        if !(-180.0..=180.0).contains(&lon_deg) {
            return Err(HeptaError::LongitudeOutOfRange(lon_deg));
        }
        if !(-90.0..=90.0).contains(&lat_deg) {
            return Err(HeptaError::LatitudeOutOfRange(lat_deg));
        }
        if density > 63 {
            return Err(HeptaError::DensityOutOfRange(density));
        }

        // D1: longitude → 18-bit (0=−180°, 262143=+180°)
        let d1_lon = ((lon_deg + 180.0) / 360.0 * (0x3_FFFF as f64)) as u32 & 0x3_FFFF;

        // D2: latitude → 18-bit (0=−90°, 262143=+90°)
        let d2_lat = ((lat_deg + 90.0) / 180.0 * (0x3_FFFF as f64)) as u32 & 0x3_FFFF;

        // D3: elevation → 18-bit offset binary (−131071 m to +131071 m)
        let elev_c = elev_m.clamp(-131_071.0, 131_071.0);
        let d3_elev = ((elev_c + 131_071.0) / 262_142.0 * (0x3_FFFF as f64)) as u32 & 0x3_FFFF;

        // D4: temporal → 18-bit, epoch in 64-second buckets relative to year 2000
        let epoch_bucket = (epoch_sec >> 6).saturating_sub(EPOCH_OFFSET);
        let d4_time = (epoch_bucket & 0x3_FFFF) as u32;

        // D5–D7
        let d5_tier = tier as u32;
        let d6_sector = sector.to_bits() as u32;
        let d7_density = (density as u32) & 0x3F;

        // Pack all 96 bits into a u128.
        // D1 is at bits 95..78, D7 is at bits 5..0.
        let packed: u128 = ((d1_lon as u128) << (18 + 18 + 18 + 8 + 10 + 6))
            | ((d2_lat as u128) << (18 + 18 + 8 + 10 + 6))
            | ((d3_elev as u128) << (18 + 8 + 10 + 6))
            | ((d4_time as u128) << (8 + 10 + 6))
            | ((d5_tier as u128) << (10 + 6))
            | ((d6_sector as u128) << 6)
            | (d7_density as u128);

        // Extract bytes 4..16 of the big-endian u128 (= bits 95..0 = 12 geo bytes).
        let packed_be = packed.to_be_bytes();
        let geo_bytes = &packed_be[4..16];

        // Identity hash: FNV-1a 32-bit of (geo_bytes || project_id bytes).
        let hash = fnv1a32(geo_bytes, project_id.as_bytes());
        let id_bytes = hash.to_be_bytes();

        let mut pk = [0u8; 16];
        pk[0..12].copy_from_slice(geo_bytes);
        pk[12..16].copy_from_slice(&id_bytes);
        Ok(Self(pk))
    }

    // ── Decode ──────────────────────────────────────────────────────────────

    fn packed_96(&self) -> u128 {
        let mut buf = [0u8; 16];
        buf[4..16].copy_from_slice(&self.0[0..12]);
        u128::from_be_bytes(buf)
    }

    pub fn longitude(&self) -> f64 {
        let d1 = ((self.packed_96() >> (18 + 18 + 18 + 8 + 10 + 6)) & 0x3_FFFF) as f64;
        d1 / (0x3_FFFF as f64) * 360.0 - 180.0
    }

    pub fn latitude(&self) -> f64 {
        let d2 = ((self.packed_96() >> (18 + 18 + 8 + 10 + 6)) & 0x3_FFFF) as f64;
        d2 / (0x3_FFFF as f64) * 180.0 - 90.0
    }

    pub fn elevation_m(&self) -> f64 {
        let d3 = ((self.packed_96() >> (18 + 8 + 10 + 6)) & 0x3_FFFF) as f64;
        d3 / (0x3_FFFF as f64) * 262_142.0 - 131_071.0
    }

    pub fn temporal_epoch(&self) -> u64 {
        let d4 = ((self.packed_96() >> (8 + 10 + 6)) & 0x3_FFFF) as u64;
        (d4 + EPOCH_OFFSET) << 6
    }

    pub fn semantic_tier(&self) -> u8 {
        ((self.packed_96() >> (10 + 6)) & 0xFF) as u8
    }

    pub fn hepta_sector(&self) -> HeptaSector {
        let d6 = ((self.packed_96() >> 6) & 0x3FF) as u16;
        HeptaSector::from_bits(d6)
    }

    pub fn density(&self) -> u8 {
        (self.packed_96() & 0x3F) as u8
    }

    pub fn identity_hash(&self) -> [u8; 4] {
        let mut h = [0u8; 4];
        h.copy_from_slice(&self.0[12..16]);
        h
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(34);
        s.push_str("0x");
        for b in &self.0 {
            let hi = (b >> 4) as usize;
            let lo = (b & 0x0F) as usize;
            const HEX: &[u8] = b"0123456789ABCDEF";
            s.push(HEX[hi] as char);
            s.push(HEX[lo] as char);
        }
        s
    }
}

impl fmt::Debug for Kaki7dPk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kaki7dPk({})", self.to_hex())
    }
}

impl fmt::Display for Kaki7dPk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// ─── Sovereign Identity Hash ──────────────────────────────────────────────────

/// FNV-1a 32-bit hash over two slices concatenated.
/// Replaces BLAKE3 with a zero-dep sovereign hash.
fn fnv1a32(a: &[u8], b: &[u8]) -> u32 {
    const BASIS: u32 = 2_166_136_261;
    const PRIME: u32 = 16_777_619;
    let mut h = BASIS;
    for &byte in a.iter().chain(b.iter()) {
        h ^= byte as u32;
        h = h.wrapping_mul(PRIME);
    }
    h
}

// ─── CRC-16/CCITT (kept for compatibility) ────────────────────────────────────

/// CRC-16/CCITT (poly 0x1021, init 0xFFFF) — no external dep.
#[allow(dead_code)]
fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planet_round_trip() {
        for (label, idx) in [
            ("SUN", 0u8),
            ("MOON", 1),
            ("MERCURY", 2),
            ("VENUS", 3),
            ("MARS", 4),
            ("JUPITER", 5),
            ("SATURN", 6),
        ] {
            let p = PlanetNode::from_label(label);
            assert_eq!(p.sector_index(), idx);
            assert_eq!(p.as_str(), label);
        }
    }

    #[test]
    fn planet_void_for_unknown_label() {
        assert_eq!(PlanetNode::from_label("SADR-CITY"), PlanetNode::Void);
    }

    #[test]
    fn chord_72_73_targets_are_distinct() {
        let p = PlanetNode::Sun;
        assert_ne!(p.chord_72_target(), p.chord_73_target());
        assert_eq!(p.chord_72_target(), PlanetNode::Mercury); // (0+2)%7=2
        assert_eq!(p.chord_73_target(), PlanetNode::Venus); // (0+3)%7=3
    }

    #[test]
    fn coord_system_from_str() {
        assert_eq!(
            CoordSystem::from_str("KAKI_7D").unwrap(),
            CoordSystem::Kaki7d
        );
        assert_eq!(
            CoordSystem::from_str("KAKI_7D_SACRED").unwrap(),
            CoordSystem::Kaki7dSacred
        );
        assert_eq!(CoordSystem::from_str("WGS84").unwrap(), CoordSystem::Wgs84);
        assert_eq!(
            CoordSystem::from_str("PROCEDURAL").unwrap(),
            CoordSystem::Procedural
        );
        assert!(CoordSystem::from_str("XY").is_err());
        assert!(CoordSystem::from_str("UNKNOWN").is_err());
    }

    #[test]
    fn chord_type_from_step() {
        assert_eq!(ChordType::from_step(2).unwrap(), ChordType::Chord72);
        assert_eq!(ChordType::from_step(3).unwrap(), ChordType::Chord73);
        assert!(ChordType::from_step(1).is_err());
        assert!(ChordType::from_step(4).is_err());
    }

    #[test]
    fn hepta_sector_root_accepts_0_to_7() {
        assert!(HeptaSector::root(0).is_ok());
        assert!(HeptaSector::root(7).is_ok());
        assert!(HeptaSector::root(8).is_err());
    }

    #[test]
    fn sector_bits_roundtrip() {
        let s = HeptaSector::new(5, 3, 7).unwrap();
        let bits = s.to_bits();
        let decoded = HeptaSector::from_bits(bits);
        assert_eq!(decoded.primary, 5);
        assert_eq!(decoded.sub1, 3);
        assert_eq!(decoded.sub2, 7);
    }

    #[test]
    fn sector_zoom_fills_sub1_then_sub2() {
        let root = HeptaSector::root(2).unwrap();
        let z1 = root.zoom(5).unwrap();
        assert_eq!(z1.primary, 2);
        assert_eq!(z1.sub1, 5);
        assert_eq!(z1.sub2, 0);
        let z2 = z1.zoom(3).unwrap();
        assert_eq!(z2.sub1, 5);
        assert_eq!(z2.sub2, 3);
        assert!(z2.zoom(1).is_err()); // MaxZoomReached
    }

    #[test]
    fn kaki7d_length_is_16_bytes() {
        let sector = HeptaSector::root(1).unwrap();
        let pk = Kaki7dPk::new(0.0, 0.0, 0.0, 0, 0, sector, 0, "test").unwrap();
        assert_eq!(pk.as_bytes().len(), 16);
    }

    #[test]
    fn kaki7d_hex_format() {
        let sector = HeptaSector::root(0).unwrap();
        let pk = Kaki7dPk::new(44.3148, 31.9957, 0.0, 0, 0, sector, 0, "test").unwrap();
        assert!(pk.to_hex().starts_with("0x"));
        assert_eq!(pk.to_hex().len(), 34); // "0x" + 32 hex chars
    }

    #[test]
    fn kaki7d_roundtrip_coordinates() {
        let sector = HeptaSector::root(0).unwrap();
        let pk = Kaki7dPk::new(
            44.3148,
            31.9957,
            30.0,
            1_700_000_000,
            1,
            sector,
            5,
            "NajafWay",
        )
        .unwrap();
        assert!(
            (pk.longitude() - 44.3148).abs() < 0.002,
            "lon roundtrip: {} ≠ 44.3148",
            pk.longitude()
        );
        assert!(
            (pk.latitude() - 31.9957).abs() < 0.001,
            "lat roundtrip: {} ≠ 31.9957",
            pk.latitude()
        );
    }

    #[test]
    fn kaki7d_sector_roundtrip() {
        let sector = HeptaSector::new(3, 2, 1).unwrap();
        let pk = Kaki7dPk::new(44.36, 33.33, 0.0, 1_700_000_000, 0, sector, 10, "Baghdad").unwrap();
        let decoded = pk.hepta_sector();
        assert_eq!(decoded.primary, 3);
        assert_eq!(decoded.sub1, 2);
        assert_eq!(decoded.sub2, 1);
    }

    #[test]
    fn kaki7d_density_roundtrip() {
        let sector = HeptaSector::root(0).unwrap();
        let pk = Kaki7dPk::new(0.0, 0.0, 0.0, 0, 0, sector, 42, "test").unwrap();
        assert_eq!(pk.density(), 42);
    }

    #[test]
    fn kaki7d_density_out_of_range_rejected() {
        let sector = HeptaSector::root(0).unwrap();
        assert!(Kaki7dPk::new(0.0, 0.0, 0.0, 0, 0, sector, 64, "test").is_err());
    }

    #[test]
    fn kaki7d_longitude_out_of_range_rejected() {
        let sector = HeptaSector::root(0).unwrap();
        assert!(Kaki7dPk::new(200.0, 0.0, 0.0, 0, 0, sector, 0, "test").is_err());
    }

    #[test]
    fn kaki7d_latitude_out_of_range_rejected() {
        let sector = HeptaSector::root(0).unwrap();
        assert!(Kaki7dPk::new(0.0, -91.0, 0.0, 0, 0, sector, 0, "test").is_err());
    }
}
