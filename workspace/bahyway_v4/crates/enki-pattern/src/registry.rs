#![forbid(unsafe_code)]
//! PatternRegistry — in-memory pattern store for ENKI-PATTERN.
//!
//! The registry maintains all patterns indexed by their Pattern-KAKI.
//! It auto-emits NARUDU-PATTERN events on every mutation.

use std::collections::HashMap;
use enkidb_kaki::Kaki;
use crate::narudu::NaruduEvent;
use crate::tier::{PatternRecord, StorageTier};
use crate::nash::NashState;

/// In-memory pattern registry with NARUDU event emission.
pub struct PatternRegistry {
    records: HashMap<[u8; 16], PatternRecord>,
    event_log: Vec<NaruduEvent>,
    current_orbital: u64,
}

impl PatternRegistry {
    pub fn new(current_orbital: u64) -> Self {
        Self {
            records: HashMap::new(),
            event_log: Vec::new(),
            current_orbital,
        }
    }

    /// Advance the registry to a new orbital (used for tier age calculations).
    pub fn tick(&mut self, orbital: u64) {
        self.current_orbital = orbital;
        // Update orbital_now on all records
        for rec in self.records.values_mut() {
            rec.orbital_now = orbital;
        }
    }

    /// Publish a new pattern to the registry.
    /// Emits a `Published` NARUDU event.
    pub fn publish(&mut self, mut record: PatternRecord) {
        record.orbital_now = self.current_orbital;
        let key = *record.pattern_kaki.bytes();
        let event = NaruduEvent::published(&record);
        self.records.insert(key, record);
        self.event_log.push(event);
    }

    /// Update the Nash state for an existing pattern.
    /// Emits a `NashShift` event if the delta is ≥ 0.10.
    pub fn update_nash(&mut self, pattern_kaki: Kaki, nash: NashState) -> bool {
        let key = *pattern_kaki.bytes();
        if let Some(rec) = self.records.get_mut(&key) {
            let delta = (nash.equilibrium_score - rec.nash_equilibrium_score).abs();
            rec.nash_equilibrium_score = nash.equilibrium_score;
            rec.orbital_now = self.current_orbital;
            if delta >= 0.10 {
                let event = NaruduEvent::nash_shift(
                    pattern_kaki, nash, self.current_orbital,
                );
                self.event_log.push(event);
                return true; // event emitted
            }
        }
        false
    }

    /// Deprecate a pattern by KAKI.  Emits a `Deprecated` NARUDU event.
    pub fn deprecate(&mut self, pattern_kaki: Kaki, reason: &'static str) -> bool {
        use enkidb_kaki::PatternLifecycle;
        let key = *pattern_kaki.bytes();
        if let Some(rec) = self.records.get_mut(&key) {
            rec.lifecycle = PatternLifecycle::Deprecated;
            let event = NaruduEvent::deprecated(pattern_kaki, self.current_orbital, reason);
            self.event_log.push(event);
            return true;
        }
        false
    }

    /// Look up a pattern by KAKI.
    pub fn get(&self, pattern_kaki: &Kaki) -> Option<&PatternRecord> {
        self.records.get(pattern_kaki.bytes())
    }

    /// All patterns in a given storage tier at the current orbital.
    pub fn by_tier(&self, tier: StorageTier) -> impl Iterator<Item = &PatternRecord> {
        self.records.values().filter(move |r| r.storage_tier() == tier)
    }

    /// All patterns currently in Canonical lifecycle.
    pub fn canonical(&self) -> impl Iterator<Item = &PatternRecord> {
        self.records.values().filter(|r| r.is_canonical())
    }

    /// Drain all pending NARUDU events (clears the internal log).
    pub fn drain_events(&mut self) -> Vec<NaruduEvent> {
        std::mem::take(&mut self.event_log)
    }

    /// Total pattern count.
    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::narudu::NaruduEventKind;
    use enkidb_kaki::{derive_pattern_kaki, FixedCoord7D, PatternLifecycle, PatternType};

    fn make_record(n: u16, orbital: u64) -> PatternRecord {
        let k = derive_pattern_kaki(
            PatternType::CrowdFlow,
            FixedCoord7D { d: [n as i32, 0, 0, 0, 0, 0, 0] },
            [0u8; 32],
            8_500,
            orbital,
        );
        PatternRecord {
            pattern_kaki: k,
            pattern_type: PatternType::CrowdFlow,
            lifecycle: PatternLifecycle::Stable,
            confidence_millipercent: 8_500,
            constituent_count: 10,
            constituent_merkle_root: [0u8; 32],
            orbital_published: orbital,
            orbital_now: orbital,
            nash_equilibrium_score: 0.0,
        }
    }

    #[test]
    fn publish_and_retrieve() {
        let mut reg = PatternRegistry::new(5);
        let rec = make_record(1, 5);
        let kaki = rec.pattern_kaki;
        reg.publish(rec);
        assert!(reg.get(&kaki).is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn nash_shift_emits_event_above_delta() {
        let mut reg = PatternRegistry::new(5);
        let rec = make_record(2, 5);
        let kaki = rec.pattern_kaki;
        reg.publish(rec);

        let nash = NashState { equilibrium_score: 0.50, anarchy_score: 1.1, deviation_count: 10, computed_at_orbital: 6 };
        let emitted = reg.update_nash(kaki, nash);
        assert!(emitted, "delta 0.50 should trigger NashShift event");

        let events = reg.drain_events();
        let kinds: Vec<_> = events.iter().map(|e| &e.kind).collect();
        assert!(kinds.contains(&&NaruduEventKind::NashShift));
    }

    #[test]
    fn deprecate_emits_event() {
        let mut reg = PatternRegistry::new(10);
        let rec = make_record(3, 10);
        let kaki = rec.pattern_kaki;
        reg.publish(rec);
        reg.drain_events(); // clear publish event

        let ok = reg.deprecate(kaki, "superseded by newer pattern");
        assert!(ok);
        let events = reg.drain_events();
        assert_eq!(events[0].kind, NaruduEventKind::Deprecated);
    }

    #[test]
    fn tier_classification_after_tick() {
        let mut reg = PatternRegistry::new(100);
        let rec = make_record(4, 100);
        reg.publish(rec);

        // Hot at orbital 100
        let hot_count = reg.by_tier(StorageTier::Hot).count();
        assert_eq!(hot_count, 1);

        // After 50 orbitals → Cold
        reg.tick(150);
        let hot_count = reg.by_tier(StorageTier::Hot).count();
        let cold_count = reg.by_tier(StorageTier::Cold).count();
        assert_eq!(hot_count, 0);
        assert_eq!(cold_count, 1);
    }
}
