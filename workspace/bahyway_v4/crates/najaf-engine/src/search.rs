//! GraveRegistry — searchable sovereign collection of burial sites.
//!
//! All lookups are in-process. No external storage — the registry is loaded
//! at startup from the sovereign data source and kept in memory.

use bahyway_core::TribeId;
use navi_engine::{haversine_m, NaviCoord};
use std::collections::HashMap;

use crate::grave::{GraveCondition, GraveId, GraveParticle};
use crate::sector::NajafSector;

pub struct GraveRegistry {
    graves: HashMap<GraveId, GraveParticle>,
}

impl GraveRegistry {
    pub fn new() -> Self {
        GraveRegistry {
            graves: HashMap::new(),
        }
    }

    pub fn register(&mut self, grave: GraveParticle) {
        self.graves.insert(grave.id, grave);
    }

    pub fn get(&self, id: GraveId) -> Option<&GraveParticle> {
        self.graves.get(&id)
    }

    pub fn get_mut(&mut self, id: GraveId) -> Option<&mut GraveParticle> {
        self.graves.get_mut(&id)
    }

    pub fn count(&self) -> usize {
        self.graves.len()
    }

    pub fn by_sector(&self, sector: NajafSector) -> Vec<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.sector == sector)
            .collect()
    }

    pub fn by_tribe(&self, tribe: TribeId) -> Vec<&GraveParticle> {
        self.graves.values().filter(|g| g.tribe == tribe).collect()
    }

    pub fn by_epoch_range(&self, from: u32, to: u32) -> Vec<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.epoch >= from && g.epoch <= to)
            .collect()
    }

    pub fn accessible(&self) -> Vec<&GraveParticle> {
        self.graves.values().filter(|g| g.is_accessible()).collect()
    }

    pub fn accessible_in_sector(&self, sector: NajafSector) -> Vec<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.sector == sector && g.is_accessible())
            .collect()
    }

    /// Returns the nearest grave (by haversine distance) to the given coordinate.
    /// Returns `None` if the registry is empty.
    pub fn nearest(&self, coord: &NaviCoord) -> Option<&GraveParticle> {
        self.graves.values().min_by(|a, b| {
            let da = haversine_m(a.coord.lat, a.coord.lon, coord.lat, coord.lon);
            let db = haversine_m(b.coord.lat, b.coord.lon, coord.lat, coord.lon);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Returns the nearest *accessible* grave to the given coordinate.
    pub fn nearest_accessible(&self, coord: &NaviCoord) -> Option<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.is_accessible())
            .min_by(|a, b| {
                let da = haversine_m(a.coord.lat, a.coord.lon, coord.lat, coord.lon);
                let db = haversine_m(b.coord.lat, b.coord.lon, coord.lat, coord.lon);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn all_ids(&self) -> Vec<GraveId> {
        self.graves.keys().copied().collect()
    }

    // ── Discovery queries ─────────────────────────────────────────────────────

    /// Return graves matching a specific physical condition.
    pub fn by_condition(&self, condition: GraveCondition) -> Vec<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.condition == condition)
            .collect()
    }

    /// Return graves whose identity confidence is below `threshold`.
    ///
    /// Use with `QUALITY_DIVISOR=240`: threshold=60 finds speculative or lower.
    pub fn unidentified(&self, threshold: u8) -> Vec<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.identity_confidence < threshold)
            .collect()
    }

    /// Return graves that are physically damaged (Partial or Destroyed condition).
    pub fn damaged(&self) -> Vec<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.condition.is_damaged())
            .collect()
    }

    /// Return damaged graves that are also unidentified (confidence < threshold).
    pub fn damaged_and_unidentified(&self, threshold: u8) -> Vec<&GraveParticle> {
        self.graves
            .values()
            .filter(|g| g.condition.is_damaged() && g.identity_confidence < threshold)
            .collect()
    }
}

impl Default for GraveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grave::{DiscoverySource, GraveCondition, GraveParticle};
    use bahyway_core::TribeId;
    use navi_engine::NaviCoord;

    fn tribe1() -> TribeId {
        TribeId::from_u16(0x0001)
    }
    fn tribe2() -> TribeId {
        TribeId::from_u16(0x0002)
    }

    fn make_grave(
        id: GraveId,
        sector: NajafSector,
        tribe: TribeId,
        epoch: u32,
        lat: f32,
    ) -> GraveParticle {
        GraveParticle::new(id, NaviCoord::new(lat, 44.320, 0.0), sector, tribe, epoch)
    }

    fn populate() -> GraveRegistry {
        let mut r = GraveRegistry::new();
        r.register(make_grave(1, NajafSector::Shuhadaa, tribe1(), 1400, 31.990));
        r.register(make_grave(2, NajafSector::Shuhadaa, tribe1(), 1410, 31.991));
        r.register(make_grave(3, NajafSector::Awliya, tribe1(), 1420, 31.992));
        r.register(make_grave(4, NajafSector::Momineen, tribe2(), 1430, 31.993));
        r.register(make_grave(5, NajafSector::Anbiya, tribe2(), 1440, 31.994));
        r
    }

    #[test]
    fn register_and_count() {
        let r = populate();
        assert_eq!(r.count(), 5);
    }

    #[test]
    fn get_existing() {
        let r = populate();
        assert!(r.get(1).is_some());
        assert!(r.get(99).is_none());
    }

    #[test]
    fn by_sector_correct_count() {
        let r = populate();
        assert_eq!(r.by_sector(NajafSector::Shuhadaa).len(), 2);
        assert_eq!(r.by_sector(NajafSector::Awliya).len(), 1);
        assert_eq!(r.by_sector(NajafSector::Ulamaa).len(), 0);
    }

    #[test]
    fn by_tribe_correct_count() {
        let r = populate();
        assert_eq!(r.by_tribe(tribe1()).len(), 3);
        assert_eq!(r.by_tribe(tribe2()).len(), 2);
    }

    #[test]
    fn by_epoch_range() {
        let r = populate();
        let in_range = r.by_epoch_range(1400, 1420);
        assert_eq!(in_range.len(), 3); // 1400, 1410, 1420
        let none = r.by_epoch_range(1500, 1600);
        assert_eq!(none.len(), 0);
    }

    #[test]
    fn accessible_returns_occupied_and_reserved() {
        let mut r = populate();
        r.get_mut(1).unwrap().seal(); // now inaccessible
        r.get_mut(2).unwrap().reserve(); // still accessible
        let acc = r.accessible();
        assert_eq!(acc.len(), 4); // 2 (reserved), 3, 4, 5
        assert!(!acc.iter().any(|g| g.id == 1));
    }

    #[test]
    fn accessible_in_sector() {
        let mut r = populate();
        r.get_mut(1).unwrap().seal();
        let acc = r.accessible_in_sector(NajafSector::Shuhadaa);
        assert_eq!(acc.len(), 1); // only grave 2 remains
    }

    #[test]
    fn nearest_returns_closest() {
        let r = populate();
        // Target exactly at grave 3's lat
        let target = NaviCoord::new(31.992, 44.320, 0.0);
        let near = r.nearest(&target).unwrap();
        assert_eq!(near.id, 3);
    }

    #[test]
    fn nearest_accessible_skips_sealed() {
        let mut r = populate();
        r.get_mut(3).unwrap().seal(); // seal the closest
        let target = NaviCoord::new(31.992, 44.320, 0.0);
        let near = r.nearest_accessible(&target).unwrap();
        assert_ne!(near.id, 3);
    }

    #[test]
    fn nearest_empty_registry_returns_none() {
        let r = GraveRegistry::new();
        let target = NaviCoord::new(31.990, 44.320, 0.0);
        assert!(r.nearest(&target).is_none());
    }

    // ── Discovery query tests ─────────────────────────────────────────────────

    fn discovery_registry() -> GraveRegistry {
        let mut r = GraveRegistry::new();
        r.register(
            make_grave(10, NajafSector::Shuhadaa, tribe1(), 1400, 31.990)
                .with_condition(GraveCondition::Intact)
                .with_confidence(240)
                .with_discovery_source(DiscoverySource::Manual),
        );
        r.register(
            make_grave(11, NajafSector::Awliya, tribe1(), 1410, 31.991)
                .with_condition(GraveCondition::Partial)
                .with_confidence(150)
                .with_discovery_source(DiscoverySource::Satellite),
        );
        r.register(
            make_grave(12, NajafSector::Huffaz, tribe2(), 1420, 31.992)
                .with_condition(GraveCondition::Destroyed)
                .with_confidence(0)
                .with_discovery_source(DiscoverySource::Drone),
        );
        r.register(
            make_grave(13, NajafSector::Momineen, tribe2(), 1430, 31.993)
                .with_condition(GraveCondition::Unknown)
                .with_confidence(60)
                .with_discovery_source(DiscoverySource::Historical),
        );
        r
    }

    #[test]
    fn by_condition_intact() {
        let r = discovery_registry();
        let intact = r.by_condition(GraveCondition::Intact);
        assert_eq!(intact.len(), 1);
        assert_eq!(intact[0].id, 10);
    }

    #[test]
    fn by_condition_destroyed() {
        let r = discovery_registry();
        let destroyed = r.by_condition(GraveCondition::Destroyed);
        assert_eq!(destroyed.len(), 1);
        assert_eq!(destroyed[0].id, 12);
    }

    #[test]
    fn unidentified_below_threshold() {
        let r = discovery_registry();
        // threshold=60: confidence<60 → only grave 12 (confidence=0)
        let unid = r.unidentified(60);
        assert_eq!(unid.len(), 1);
        assert_eq!(unid[0].id, 12);
    }

    #[test]
    fn unidentified_below_240_returns_all_but_confirmed() {
        let r = discovery_registry();
        // threshold=240: all with confidence<240 → graves 11(150), 12(0), 13(60)
        let unid = r.unidentified(240);
        assert_eq!(unid.len(), 3);
        assert!(!unid.iter().any(|g| g.id == 10));
    }

    #[test]
    fn damaged_returns_partial_and_destroyed() {
        let r = discovery_registry();
        let damaged = r.damaged();
        assert_eq!(damaged.len(), 2);
        let ids: Vec<u32> = damaged.iter().map(|g| g.id).collect();
        assert!(ids.contains(&11));
        assert!(ids.contains(&12));
    }

    #[test]
    fn damaged_and_unidentified_combined() {
        let r = discovery_registry();
        // Damaged (Partial/Destroyed) AND confidence < 60 → only grave 12 (Destroyed, confidence=0)
        let hits = r.damaged_and_unidentified(60);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, 12);
    }
}
