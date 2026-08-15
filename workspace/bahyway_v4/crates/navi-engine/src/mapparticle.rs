//! MapParticle — the sovereign atom of the NaviEngine map.
//!
//! Everything visible on the map is a `MapParticle` with a KAKI identity:
//! road segments, junctions, pipelines, POIs, landmarks, and grave markers.
//!
//! `MapKind` encodes the particle's structural type with routing-relevant
//! properties as enum fields (road class, pipeline type, flow direction).
//! Additional enrichment lives in the `EavStore` attached to each particle.
//!
//! `zoom_min` — the minimum HubbleZoom level at which this particle becomes
//! visible.  A motorway junction appears at Z1; an individual grave headstone
//! at Z5.  The `HubbleTile` query engine uses this to filter particles.

#![forbid(unsafe_code)]

use bahyway_core::TribeId;
use enkidb_kaki::{Kaki, KakiMinter, KakiRole};

use crate::eav::EavStore;
use crate::particle::{NaviCoord, NaviParticleState};

// ── Road classification ───────────────────────────────────────────────────────

/// Functional road class — determines routing cost multiplier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadClass {
    Motorway,    // 0.60× — fastest arterial
    Primary,     // 0.75× — national/regional highway
    Secondary,   // 0.90× — inter-district road
    Tertiary,    // 1.00× — local street
    Local,       // 1.10× — residential / access road
    Track,       // 1.40× — unpaved / rough
    Footway,     // 2.00× — pedestrian only (no vehicles)
}

impl RoadClass {
    pub fn cost_multiplier(self) -> f32 {
        match self {
            RoadClass::Motorway   => 0.60,
            RoadClass::Primary    => 0.75,
            RoadClass::Secondary  => 0.90,
            RoadClass::Tertiary   => 1.00,
            RoadClass::Local      => 1.10,
            RoadClass::Track      => 1.40,
            RoadClass::Footway    => 2.00,
        }
    }

    pub fn is_vehicle_accessible(self) -> bool {
        !matches!(self, RoadClass::Footway)
    }

    pub fn as_u8(self) -> u8 {
        match self {
            RoadClass::Motorway  => 0,
            RoadClass::Primary   => 1,
            RoadClass::Secondary => 2,
            RoadClass::Tertiary  => 3,
            RoadClass::Local     => 4,
            RoadClass::Track     => 5,
            RoadClass::Footway   => 6,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => RoadClass::Motorway,
            1 => RoadClass::Primary,
            2 => RoadClass::Secondary,
            3 => RoadClass::Tertiary,
            4 => RoadClass::Local,
            5 => RoadClass::Track,
            _ => RoadClass::Footway,
        }
    }
}

// ── Pipeline classification ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineKind {
    WaterSupply,
    Sewage,
    Drainage,
    Gas,
    Power,
    Irrigation,
    Telecoms,
}

impl PipelineKind {
    pub fn label(self) -> &'static str {
        match self {
            PipelineKind::WaterSupply => "Water Supply",
            PipelineKind::Sewage      => "Sewage",
            PipelineKind::Drainage    => "Drainage",
            PipelineKind::Gas         => "Gas",
            PipelineKind::Power       => "Power",
            PipelineKind::Irrigation  => "Irrigation",
            PipelineKind::Telecoms    => "Telecoms",
        }
    }
}

// ── Flow direction ────────────────────────────────────────────────────────────

/// Flow direction for pipeline segments — determines traversal validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowDir {
    /// Pressure-fed — from source to consumer (pump → tap).
    UpStream,
    /// Gravity-fed — toward drain / treatment plant.
    DownStream,
    /// Flow can occur in either direction (e.g. ring main).
    Bidirectional,
}

// ── Point-of-Interest category ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoiCategory {
    Mosque,
    Hospital,
    School,
    Government,
    Cemetery,
    Industrial,
    Commercial,
    Residential,
    Station,        // transport hub
    WaterTower,
    PumpStation,
    Other,
}

// ── MapKind — structural type of a map particle ───────────────────────────────

/// The structural type of a `MapParticle`.  Routing-relevant sub-properties
/// are encoded as enum fields; further detail lives in the `EavStore`.
#[derive(Debug, Clone, PartialEq)]
pub enum MapKind {
    /// Road or path intersection / routing node.
    Junction,
    /// A directed road segment between two junctions.
    Road {
        class:   RoadClass,
        one_way: bool,
    },
    /// A utility pipeline segment.
    Pipeline {
        kind: PipelineKind,
        flow: FlowDir,
    },
    /// Named point of interest.
    Poi { category: PoiCategory },
    /// Named landmark for orientation (mosque, tower, etc.).
    Landmark,
    /// Administrative or sovereign boundary node.
    Boundary,
    /// Cemetery grave marker — integration point for NajafEngine.
    GraveMarker,
}

impl MapKind {
    pub fn is_road(&self)     -> bool { matches!(self, MapKind::Road { .. }) }
    pub fn is_pipeline(&self) -> bool { matches!(self, MapKind::Pipeline { .. }) }
    pub fn is_junction(&self) -> bool { matches!(self, MapKind::Junction) }

    /// Routing cost multiplier imposed by the particle's structural type.
    pub fn routing_multiplier(&self) -> f32 {
        match self {
            MapKind::Road { class, .. }     => class.cost_multiplier(),
            MapKind::Junction               => 1.00,
            MapKind::Pipeline { .. }        => 1.00,
            MapKind::Poi { .. }             => 1.20,
            MapKind::Landmark               => 1.10,
            MapKind::Boundary               => 1.30,
            MapKind::GraveMarker            => 0.95, // pilgrimage priority
        }
    }

    /// True when a vehicle can traverse this particle kind.
    pub fn vehicle_accessible(&self) -> bool {
        match self {
            MapKind::Road { class, .. } => class.is_vehicle_accessible(),
            MapKind::Junction           => true,
            _                           => false,
        }
    }

    /// True when a pedestrian can traverse this particle kind.
    pub fn pedestrian_accessible(&self) -> bool {
        match self {
            MapKind::Road { .. }    => true,
            MapKind::Junction       => true,
            MapKind::GraveMarker    => true,
            MapKind::Poi { .. }     => true,
            MapKind::Landmark       => true,
            _                       => false,
        }
    }
}

// ── MapParticle ───────────────────────────────────────────────────────────────

/// The sovereign atom of the map.  Every feature — road, pipeline, POI —
/// is represented as a `MapParticle` with an immutable KAKI identity.
pub struct MapParticle {
    pub particle_id: u32,
    /// Immutable sovereign identity — minted once at registration.
    pub kaki:        Kaki,
    pub coord:       NaviCoord,
    pub kind:        MapKind,
    pub tribe:       TribeId,
    /// Minimum HubbleZoom level at which this particle becomes visible.
    pub zoom_min:    u8,
    pub eav:         EavStore,
    pub state:       NaviParticleState,
}

impl MapParticle {
    /// Construct and mint a new map particle.
    pub fn new(
        particle_id: u32,
        coord:       NaviCoord,
        kind:        MapKind,
        tribe:       TribeId,
        zoom_min:    u8,
    ) -> Self {
        let minter = KakiMinter::new(tribe);
        let kaki   = minter.mint_identity(particle_id, KakiRole::Zikru);
        MapParticle {
            particle_id,
            kaki,
            coord,
            kind,
            tribe,
            zoom_min,
            eav:   EavStore::new(),
            state: NaviParticleState::Active,
        }
    }

    pub fn with_eav(mut self, eav: EavStore) -> Self { self.eav = eav; self }
    pub fn with_state(mut self, state: NaviParticleState) -> Self { self.state = state; self }

    pub fn is_passable(&self) -> bool {
        matches!(self.state, NaviParticleState::Active)
    }

    pub fn is_road(&self)     -> bool { self.kind.is_road() }
    pub fn is_pipeline(&self) -> bool { self.kind.is_pipeline() }
    pub fn is_junction(&self) -> bool { self.kind.is_junction() }

    /// Intrinsic routing cost — structural kind × EAV surface quality.
    pub fn routing_cost(&self) -> f32 {
        self.kind.routing_multiplier()
    }

    /// KAKI hash (first 4 bytes of the KAKI uuid field) — used as map key.
    pub fn kaki_hash(&self) -> u32 { self.kaki.uuid_hash() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use crate::particle::NaviCoord;

    fn tribe() -> TribeId { TribeId::from_u16(0x0001) }
    fn coord() -> NaviCoord { NaviCoord::new(31.995, 44.320, 0.0) }

    fn road(id: u32, class: RoadClass) -> MapParticle {
        MapParticle::new(id, coord(), MapKind::Road { class, one_way: false }, tribe(), 2)
    }

    fn junction(id: u32) -> MapParticle {
        MapParticle::new(id, coord(), MapKind::Junction, tribe(), 2)
    }

    #[test]
    fn new_particle_is_active() {
        let p = road(1, RoadClass::Primary);
        assert!(p.is_passable());
        assert_eq!(p.state, NaviParticleState::Active);
    }

    #[test]
    fn kaki_minted_with_correct_tribe() {
        let p = road(1, RoadClass::Motorway);
        assert_eq!(p.kaki.tribe_id(), tribe());
    }

    #[test]
    fn kaki_checksum_valid() {
        let p = junction(42);
        assert!(p.kaki.verify_checksum());
    }

    #[test]
    fn different_ids_produce_different_kakis() {
        let p1 = junction(100);
        let p2 = junction(101);
        assert_ne!(p1.kaki_hash(), p2.kaki_hash());
    }

    #[test]
    fn road_class_cost_order() {
        assert!(RoadClass::Motorway.cost_multiplier() < RoadClass::Primary.cost_multiplier());
        assert!(RoadClass::Primary.cost_multiplier() < RoadClass::Secondary.cost_multiplier());
        assert!(RoadClass::Track.cost_multiplier() < RoadClass::Footway.cost_multiplier());
    }

    #[test]
    fn road_class_roundtrip_u8() {
        for v in 0u8..=6 {
            assert_eq!(RoadClass::from_u8(v).as_u8(), v);
        }
    }

    #[test]
    fn footway_not_vehicle_accessible() {
        assert!(!RoadClass::Footway.is_vehicle_accessible());
        assert!(RoadClass::Motorway.is_vehicle_accessible());
    }

    #[test]
    fn map_kind_predicates() {
        let road = MapKind::Road { class: RoadClass::Primary, one_way: false };
        let pipe = MapKind::Pipeline { kind: PipelineKind::WaterSupply, flow: FlowDir::UpStream };
        let junc = MapKind::Junction;
        assert!(road.is_road());
        assert!(!road.is_pipeline());
        assert!(pipe.is_pipeline());
        assert!(!pipe.is_road());
        assert!(junc.is_junction());
    }

    #[test]
    fn vehicle_accessible_excludes_footway() {
        let foot = MapKind::Road { class: RoadClass::Footway, one_way: false };
        assert!(!foot.vehicle_accessible());
        let pri = MapKind::Road { class: RoadClass::Primary, one_way: false };
        assert!(pri.vehicle_accessible());
    }

    #[test]
    fn pedestrian_accessible_on_roads_and_graves() {
        let r = MapKind::Road { class: RoadClass::Footway, one_way: false };
        assert!(r.pedestrian_accessible());
        assert!(MapKind::GraveMarker.pedestrian_accessible());
        assert!(!MapKind::Pipeline { kind: PipelineKind::Sewage, flow: FlowDir::DownStream }.pedestrian_accessible());
    }

    #[test]
    fn gravemarker_has_lower_routing_cost_than_tertiary() {
        let gm      = MapKind::GraveMarker;
        let tertiary = MapKind::Road { class: RoadClass::Tertiary, one_way: false };
        // GraveMarker 0.95 < Tertiary 1.00
        assert!(gm.routing_multiplier() < tertiary.routing_multiplier());
    }

    #[test]
    fn with_state_builder() {
        let p = junction(5).with_state(NaviParticleState::Dead);
        assert!(!p.is_passable());
    }

    #[test]
    fn pipeline_label_non_empty() {
        for kind in [
            PipelineKind::WaterSupply, PipelineKind::Sewage, PipelineKind::Drainage,
            PipelineKind::Gas, PipelineKind::Power, PipelineKind::Irrigation, PipelineKind::Telecoms,
        ] {
            assert!(!kind.label().is_empty());
        }
    }

    #[test]
    fn zoom_min_stored() {
        let p = MapParticle::new(99, coord(), MapKind::GraveMarker, tribe(), 5);
        assert_eq!(p.zoom_min, 5);
    }

    #[test]
    fn eav_attached_via_builder() {
        let mut eav = EavStore::new();
        eav.set(crate::eav::EavAttr::optional(crate::eav::ATTR_NAME, crate::eav::AttrValue::Text("Test".into())));
        let p = junction(10).with_eav(eav);
        assert_eq!(p.eav.get_text(crate::eav::ATTR_NAME), Some("Test"));
    }
}
