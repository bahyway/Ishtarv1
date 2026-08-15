//! GraveParticle — a sovereign burial site in Wadi al-Salam.
//!
//! Each grave holds an immutable KAKI (sovereign identity) minted once at
//! registration time.  Coordinates follow NaviEngine convention (lat/lon/alt).
//! Epoch is the Islamic calendar year of interment (e.g. 1400 = 1979 CE).
//!
//! # Discovery Extension
//!
//! `GraveCondition` and `DiscoverySource` track satellite/drone AI findings.
//! `identity_confidence` uses the sovereign QUALITY_DIVISOR=240 scale:
//! 240=confirmed, 150=probable, 60=speculative, 0=unidentified.

use bahyway_core::TribeId;
use enkidb_kaki::{KakiMinter, KakiRole, Kaki};
use navi_engine::NaviCoord;

use crate::sector::NajafSector;

pub type GraveId = u32;

/// Sovereign quality divisor — must match ADR-001 (never 255).
pub const QUALITY_DIVISOR: u8 = 240;

// ── GraveState ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraveState {
    /// Grave contains remains — pilgrimage destination.
    Occupied,
    /// Pre-purchased, awaiting use — still a valid destination.
    Reserved,
    /// Unoccupied and not reserved — no pilgrimage purpose.
    Available,
    /// Temporarily inaccessible (restoration, authority restriction).
    Sealed,
}

impl GraveState {
    pub fn is_accessible(self) -> bool {
        matches!(self, GraveState::Occupied | GraveState::Reserved)
    }
}

// ── GraveCondition ────────────────────────────────────────────────────────────

/// Physical condition of the grave structure as assessed by AI discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraveCondition {
    /// Structure fully intact — headstone and surround visible.
    Intact,
    /// Partially damaged — headstone collapsed or surround eroded.
    Partial,
    /// Structure destroyed — only ground disturbance remains.
    Destroyed,
    /// Not yet assessed — default for manually registered graves.
    Unknown,
}

impl GraveCondition {
    pub fn is_damaged(self) -> bool {
        matches!(self, GraveCondition::Partial | GraveCondition::Destroyed)
    }
}

// ── DiscoverySource ───────────────────────────────────────────────────────────

/// Attribution: how the grave was located / identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverySource {
    /// Field survey — physical inspection by staff.
    Manual,
    /// Satellite imagery analysis (AI inference).
    Satellite,
    /// Drone-captured overhead or oblique imagery.
    Drone,
    /// Historical records / archive maps.
    Historical,
    /// Civil registry cross-reference.
    CivilRegistry,
}

// ── GraveParticle ─────────────────────────────────────────────────────────────

pub struct GraveParticle {
    pub id:    GraveId,
    pub coord: NaviCoord,
    pub sector: NajafSector,
    pub tribe: TribeId,
    /// Islamic Hijri year of interment (e.g. 1400).
    pub epoch: u32,
    pub state: GraveState,
    /// Immutable sovereign 16-byte identity — never reassigned.
    pub kaki:  Kaki,

    // ── Discovery extensions ──────────────────────────────────────────────────
    /// Physical condition assessed by AI imagery analysis.
    pub condition: GraveCondition,
    /// Identity confidence on QUALITY_DIVISOR=240 scale (0=unknown, 240=confirmed).
    pub identity_confidence: u8,
    /// KAKI hash of the linked civil-registry particle (if resolved).
    pub civil_registry_kaki: Option<u32>,
    /// KAKI hash of the satellite/drone image tile particle.
    pub image_tile_kaki: Option<u32>,
    /// How this grave was discovered / last updated.
    pub discovery_source: DiscoverySource,
    /// Grave owner's Arabic name from inscription or registry (if identified).
    pub owner_name_arabic: Option<String>,
    /// Hijri year of death from inscription or civil record (if known).
    pub death_hijri_year: Option<u32>,
}

impl GraveParticle {
    /// Register a new grave. Mints a KAKI automatically.
    pub fn new(
        id:     GraveId,
        coord:  NaviCoord,
        sector: NajafSector,
        tribe:  TribeId,
        epoch:  u32,
    ) -> Self {
        let minter = KakiMinter::new(tribe);
        let kaki   = minter.mint_identity(id, KakiRole::Zikru);
        GraveParticle {
            id, coord, sector, tribe, epoch,
            state:               GraveState::Occupied,
            kaki,
            condition:           GraveCondition::Unknown,
            identity_confidence: 0,
            civil_registry_kaki: None,
            image_tile_kaki:     None,
            discovery_source:    DiscoverySource::Manual,
            owner_name_arabic:   None,
            death_hijri_year:    None,
        }
    }

    /// Create with an explicit state (used in seed data).
    pub fn with_state(mut self, state: GraveState) -> Self {
        self.state = state;
        self
    }

    pub fn with_condition(mut self, condition: GraveCondition) -> Self {
        self.condition = condition;
        self
    }

    /// Set identity confidence (clamped to QUALITY_DIVISOR=240).
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.identity_confidence = confidence.min(QUALITY_DIVISOR);
        self
    }

    pub fn with_civil_registry(mut self, kaki_hash: u32) -> Self {
        self.civil_registry_kaki = Some(kaki_hash);
        self
    }

    pub fn with_image_tile(mut self, kaki_hash: u32) -> Self {
        self.image_tile_kaki = Some(kaki_hash);
        self
    }

    pub fn with_discovery_source(mut self, source: DiscoverySource) -> Self {
        self.discovery_source = source;
        self
    }

    pub fn with_owner_name(mut self, name: impl Into<String>) -> Self {
        self.owner_name_arabic = Some(name.into());
        self
    }

    pub fn with_death_year(mut self, hijri_year: u32) -> Self {
        self.death_hijri_year = Some(hijri_year);
        self
    }

    pub fn is_accessible(&self) -> bool { self.state.is_accessible() }
    pub fn is_identified(&self) -> bool { self.identity_confidence > 0 }

    pub fn seal(&mut self)    { self.state = GraveState::Sealed; }
    pub fn release(&mut self) { self.state = GraveState::Available; }
    pub fn occupy(&mut self)  { self.state = GraveState::Occupied; }
    pub fn reserve(&mut self) { self.state = GraveState::Reserved; }

    /// Intrinsic routing cost — sacred weight of the zone.
    pub fn intrinsic_cost(&self) -> f32 {
        self.sector.sacred_weight()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navi_engine::NaviCoord;
    use bahyway_core::TribeId;

    fn coord() -> NaviCoord { NaviCoord::new(31.995, 44.320, 0.0) }
    fn tribe() -> TribeId   { TribeId::from_u16(0x0001) }

    fn grave(id: GraveId, sector: NajafSector) -> GraveParticle {
        GraveParticle::new(id, coord(), sector, tribe(), 1400)
    }

    #[test]
    fn new_grave_is_occupied() {
        let g = grave(1, NajafSector::Shuhadaa);
        assert_eq!(g.state, GraveState::Occupied);
        assert!(g.is_accessible());
    }

    #[test]
    fn seal_makes_inaccessible() {
        let mut g = grave(2, NajafSector::Awliya);
        g.seal();
        assert!(!g.is_accessible());
        assert_eq!(g.state, GraveState::Sealed);
    }

    #[test]
    fn release_makes_available_not_accessible() {
        let mut g = grave(3, NajafSector::Huffaz);
        g.release();
        assert_eq!(g.state, GraveState::Available);
        assert!(!g.is_accessible());
    }

    #[test]
    fn reserve_is_accessible() {
        let mut g = grave(4, NajafSector::Momineen);
        g.reserve();
        assert!(g.is_accessible());
        assert_eq!(g.state, GraveState::Reserved);
    }

    #[test]
    fn occupy_restores_occupied() {
        let mut g = grave(5, NajafSector::Ulamaa);
        g.seal();
        g.occupy();
        assert!(g.is_accessible());
        assert_eq!(g.state, GraveState::Occupied);
    }

    #[test]
    fn intrinsic_cost_matches_sector_weight() {
        let g = grave(6, NajafSector::Shuhadaa);
        assert!((g.intrinsic_cost() - NajafSector::Shuhadaa.sacred_weight()).abs() < f32::EPSILON);
    }

    #[test]
    fn kaki_tribe_matches_grave_tribe() {
        let g = grave(7, NajafSector::Anbiya);
        assert_eq!(g.kaki.tribe_id(), tribe());
    }

    #[test]
    fn kaki_checksum_valid() {
        let g = grave(8, NajafSector::Entrance);
        assert!(g.kaki.verify_checksum());
    }

    #[test]
    fn with_state_builder() {
        let g = GraveParticle::new(9, coord(), NajafSector::Awliya, tribe(), 1420)
            .with_state(GraveState::Reserved);
        assert!(g.is_accessible());
        assert_eq!(g.state, GraveState::Reserved);
    }

    #[test]
    fn grave_state_is_accessible_matrix() {
        assert!(GraveState::Occupied.is_accessible());
        assert!(GraveState::Reserved.is_accessible());
        assert!(!GraveState::Available.is_accessible());
        assert!(!GraveState::Sealed.is_accessible());
    }

    #[test]
    fn epoch_stored_correctly() {
        let g = grave(10, NajafSector::Momineen);
        assert_eq!(g.epoch, 1400);
    }

    #[test]
    fn different_graves_have_different_kakis() {
        let g1 = grave(100, NajafSector::Shuhadaa);
        let g2 = grave(101, NajafSector::Shuhadaa);
        // uuid_hash differs because id (mint_identity argument) differs
        assert_ne!(g1.kaki.uuid_hash(), g2.kaki.uuid_hash());
    }

    // ── Discovery extension tests ─────────────────────────────────────────────

    #[test]
    fn new_grave_defaults_unknown_condition() {
        let g = grave(20, NajafSector::Shuhadaa);
        assert_eq!(g.condition, GraveCondition::Unknown);
        assert_eq!(g.identity_confidence, 0);
        assert!(g.civil_registry_kaki.is_none());
        assert!(g.image_tile_kaki.is_none());
        assert_eq!(g.discovery_source, DiscoverySource::Manual);
        assert!(g.owner_name_arabic.is_none());
        assert!(g.death_hijri_year.is_none());
    }

    #[test]
    fn with_condition_builder() {
        let g = grave(21, NajafSector::Awliya).with_condition(GraveCondition::Partial);
        assert_eq!(g.condition, GraveCondition::Partial);
        assert!(g.condition.is_damaged());
    }

    #[test]
    fn grave_condition_damaged_matrix() {
        assert!(!GraveCondition::Intact.is_damaged());
        assert!(GraveCondition::Partial.is_damaged());
        assert!(GraveCondition::Destroyed.is_damaged());
        assert!(!GraveCondition::Unknown.is_damaged());
    }

    #[test]
    fn with_confidence_clamped_to_240() {
        let g = grave(22, NajafSector::Huffaz).with_confidence(255);
        assert_eq!(g.identity_confidence, 240, "must clamp to QUALITY_DIVISOR");
    }

    #[test]
    fn with_confidence_exact_240() {
        let g = grave(23, NajafSector::Momineen).with_confidence(240);
        assert_eq!(g.identity_confidence, 240);
        assert!(g.is_identified());
    }

    #[test]
    fn with_civil_registry_and_image_tile() {
        let g = grave(24, NajafSector::Ulamaa)
            .with_civil_registry(0xABCD_1234)
            .with_image_tile(0xFEED_BEEF);
        assert_eq!(g.civil_registry_kaki, Some(0xABCD_1234));
        assert_eq!(g.image_tile_kaki, Some(0xFEED_BEEF));
    }

    #[test]
    fn with_owner_and_death_year() {
        let g = grave(25, NajafSector::Anbiya)
            .with_owner_name("عبد الله الحسيني")
            .with_death_year(1398);
        assert_eq!(g.owner_name_arabic.as_deref(), Some("عبد الله الحسيني"));
        assert_eq!(g.death_hijri_year, Some(1398));
    }

    #[test]
    fn satellite_discovery_source() {
        let g = grave(26, NajafSector::Entrance)
            .with_discovery_source(DiscoverySource::Satellite);
        assert_eq!(g.discovery_source, DiscoverySource::Satellite);
    }

    #[test]
    fn full_discovery_chain() {
        let g = GraveParticle::new(27, coord(), NajafSector::Shuhadaa, tribe(), 1380)
            .with_condition(GraveCondition::Partial)
            .with_confidence(150)
            .with_discovery_source(DiscoverySource::Drone)
            .with_image_tile(0x1A2B_3C4D)
            .with_owner_name("محمد الكاظمي")
            .with_death_year(1380);
        assert!(g.condition.is_damaged());
        assert_eq!(g.identity_confidence, 150);
        assert!(g.is_identified());
        assert_eq!(g.discovery_source, DiscoverySource::Drone);
    }
}
