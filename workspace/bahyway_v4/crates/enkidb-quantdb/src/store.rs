//! QuantDbStore — the sovereign quantitative tick archive.
//!
//! Append-only in-memory store indexed by (asset_hash, epoch).
//! In production this backs to enkidb-persist sovereign pages.
//! Here the store is a Vec<TickRecord> per asset, sorted by epoch+sequence.

use crate::ohlc::{OhlcAggregator, OhlcBar};
use crate::tick::TickRecord;
use std::collections::HashMap;

/// The sovereign quantitative time-series store.
///
/// One store per market session.  Ticks are appended and never mutated.
/// Queries return slices or owned Vecs — no external iterators needed.
pub struct QuantDbStore {
    /// Per-asset tick storage: asset_hash → Vec<TickRecord> (ordered by epoch, sequence).
    ticks: HashMap<u32, Vec<TickRecord>>,
    /// Total tick count across all assets.
    total: usize,
}

impl QuantDbStore {
    pub fn new() -> Self {
        QuantDbStore {
            ticks: HashMap::new(),
            total: 0,
        }
    }

    /// Append a tick.  Ticks for the same asset should arrive in
    /// non-decreasing (epoch, sequence) order; the store does not sort.
    pub fn push_tick(&mut self, tick: TickRecord) {
        self.ticks.entry(tick.asset_hash).or_default().push(tick);
        self.total += 1;
    }

    /// Append a batch of ticks efficiently.
    pub fn push_batch(&mut self, ticks: &[TickRecord]) {
        for t in ticks {
            self.push_tick(*t);
        }
    }

    /// All ticks for a given asset in insertion order.
    pub fn ticks_for_asset(&self, asset_hash: u32) -> &[TickRecord] {
        self.ticks
            .get(&asset_hash)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Ticks for an asset in the epoch range [from, to] inclusive.
    pub fn ticks_in_range(&self, asset_hash: u32, from: u32, to: u32) -> Vec<&TickRecord> {
        self.ticks_for_asset(asset_hash)
            .iter()
            .filter(|t| t.epoch >= from && t.epoch <= to)
            .collect()
    }

    /// Latest tick for an asset (highest epoch + sequence), or None.
    pub fn latest_tick(&self, asset_hash: u32) -> Option<&TickRecord> {
        self.ticks.get(&asset_hash)?.last()
    }

    /// Latest price for an asset, or None.
    pub fn latest_price(&self, asset_hash: u32) -> Option<i64> {
        self.latest_tick(asset_hash).map(|t| t.raw_price)
    }

    /// Build OHLC bars for an asset using a given window size.
    pub fn build_ohlc(&self, asset_hash: u32, window_size: u32) -> Vec<OhlcBar> {
        let mut agg = OhlcAggregator::new(asset_hash, window_size);
        for tick in self.ticks_for_asset(asset_hash) {
            agg.ingest(tick);
        }
        agg.flush();
        agg.completed_bars().to_vec()
    }

    /// Number of distinct assets in the store.
    pub fn asset_count(&self) -> usize {
        self.ticks.len()
    }

    /// Total tick count across all assets.
    pub fn total_ticks(&self) -> usize {
        self.total
    }

    /// All known asset hashes.
    pub fn asset_hashes(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.ticks.keys().copied().collect();
        v.sort();
        v
    }

    /// Tick count for a specific asset.
    pub fn tick_count(&self, asset_hash: u32) -> usize {
        self.ticks.get(&asset_hash).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for QuantDbStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(asset: u32, price: i64, epoch: u32) -> TickRecord {
        TickRecord::new(asset, price, 1000, 1, epoch)
    }

    #[test]
    fn push_and_retrieve_ticks() {
        let mut store = QuantDbStore::new();
        store.push_tick(t(0xA, 100, 1));
        store.push_tick(t(0xA, 110, 2));
        assert_eq!(store.tick_count(0xA), 2);
        assert_eq!(store.total_ticks(), 2);
    }

    #[test]
    fn ticks_in_range_filters_correctly() {
        let mut store = QuantDbStore::new();
        for epoch in 1u32..=10 {
            store.push_tick(t(0xA, epoch as i64 * 10, epoch));
        }
        let range = store.ticks_in_range(0xA, 3, 6);
        assert_eq!(range.len(), 4);
        assert_eq!(range[0].epoch, 3);
        assert_eq!(range[3].epoch, 6);
    }

    #[test]
    fn latest_price_is_last_inserted() {
        let mut store = QuantDbStore::new();
        store.push_tick(t(0xB, 200, 5));
        store.push_tick(t(0xB, 250, 7));
        assert_eq!(store.latest_price(0xB), Some(250));
    }

    #[test]
    fn unknown_asset_returns_empty() {
        let store = QuantDbStore::new();
        assert_eq!(store.ticks_for_asset(0xDEAD).len(), 0);
        assert_eq!(store.latest_price(0xDEAD), None);
    }

    #[test]
    fn multiple_assets_isolated() {
        let mut store = QuantDbStore::new();
        store.push_tick(t(0x01, 100, 1));
        store.push_tick(t(0x02, 200, 1));
        store.push_tick(t(0x01, 110, 2));
        assert_eq!(store.asset_count(), 2);
        assert_eq!(store.tick_count(0x01), 2);
        assert_eq!(store.tick_count(0x02), 1);
    }

    #[test]
    fn build_ohlc_produces_bars() {
        let mut store = QuantDbStore::new();
        for epoch in 1u32..=25 {
            store.push_tick(t(0xC, 100 + epoch as i64, epoch));
        }
        let bars = store.build_ohlc(0xC, 5);
        assert_eq!(bars.len(), 5); // epochs 1-5, 6-10, 11-15, 16-20, 21-25
    }

    #[test]
    fn asset_hashes_sorted() {
        let mut store = QuantDbStore::new();
        store.push_tick(t(0x30, 1, 1));
        store.push_tick(t(0x10, 1, 1));
        store.push_tick(t(0x20, 1, 1));
        let hashes = store.asset_hashes();
        assert_eq!(hashes, vec![0x10, 0x20, 0x30]);
    }

    #[test]
    fn push_batch_equivalent_to_individual_push() {
        let mut s1 = QuantDbStore::new();
        let mut s2 = QuantDbStore::new();
        let ticks = vec![t(0xA, 1, 1), t(0xA, 2, 2), t(0xA, 3, 3)];
        for tick in &ticks {
            s1.push_tick(*tick);
        }
        s2.push_batch(&ticks);
        assert_eq!(s1.total_ticks(), s2.total_ticks());
        assert_eq!(s1.latest_price(0xA), s2.latest_price(0xA));
    }
}
