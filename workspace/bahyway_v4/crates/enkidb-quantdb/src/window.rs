//! TimeSeriesWindow — epoch-ranged slice query over a tick series.
//!
//! Wraps a reference to a `QuantDbStore` and provides ergonomic windowed
//! access for quant-engine to build return series, rolling stats, etc.

use crate::ohlc::OhlcBar;
use crate::store::QuantDbStore;
use crate::tick::TickRecord;

/// A view over an asset's tick history within an epoch window.
///
/// Used by `quant-engine` to feed `ReturnSeries` and `RollingStats`.
pub struct TimeSeriesWindow<'a> {
    store: &'a QuantDbStore,
    asset_hash: u32,
    from_epoch: u32,
    to_epoch: u32,
}

impl<'a> TimeSeriesWindow<'a> {
    /// Create a window over `[from_epoch, to_epoch]` inclusive.
    pub fn new(store: &'a QuantDbStore, asset_hash: u32, from_epoch: u32, to_epoch: u32) -> Self {
        Self {
            store,
            asset_hash,
            from_epoch,
            to_epoch,
        }
    }

    /// Full history window (all ticks for this asset).
    pub fn full(store: &'a QuantDbStore, asset_hash: u32) -> Self {
        Self::new(store, asset_hash, 0, u32::MAX)
    }

    /// All tick prices as a Vec<i64> in chronological order.
    pub fn prices(&self) -> Vec<i64> {
        self.store
            .ticks_in_range(self.asset_hash, self.from_epoch, self.to_epoch)
            .iter()
            .map(|t| t.raw_price)
            .collect()
    }

    /// All ticks in the window.
    pub fn ticks(&self) -> Vec<&TickRecord> {
        self.store
            .ticks_in_range(self.asset_hash, self.from_epoch, self.to_epoch)
    }

    /// OHLC bars for this window using the given bar window size.
    pub fn ohlc_bars(&self, bar_window_size: u32) -> Vec<OhlcBar> {
        use crate::ohlc::OhlcAggregator;
        let mut agg = OhlcAggregator::new(self.asset_hash, bar_window_size);
        for tick in self.ticks() {
            agg.ingest(tick);
        }
        agg.flush();
        agg.completed_bars().to_vec()
    }

    /// Tick count in this window.
    pub fn len(&self) -> usize {
        self.store
            .ticks_in_range(self.asset_hash, self.from_epoch, self.to_epoch)
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// First price in the window, or None.
    pub fn first_price(&self) -> Option<i64> {
        self.store
            .ticks_in_range(self.asset_hash, self.from_epoch, self.to_epoch)
            .first()
            .map(|t| t.raw_price)
    }

    /// Last price in the window, or None.
    pub fn last_price(&self) -> Option<i64> {
        self.store
            .ticks_in_range(self.asset_hash, self.from_epoch, self.to_epoch)
            .last()
            .map(|t| t.raw_price)
    }

    /// Absolute price change from first to last tick.
    pub fn price_change(&self) -> Option<i64> {
        let first = self.first_price()?;
        let last = self.last_price()?;
        Some(last - first)
    }

    /// Asset hash this window covers.
    pub fn asset_hash(&self) -> u32 {
        self.asset_hash
    }

    /// Epoch range [from, to].
    pub fn epoch_range(&self) -> (u32, u32) {
        (self.from_epoch, self.to_epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick::TickRecord;

    fn build_store() -> QuantDbStore {
        let mut s = QuantDbStore::new();
        for epoch in 1u32..=20 {
            s.push_tick(TickRecord::new(0xAA, epoch as i64 * 10, 1000, 1, epoch));
        }
        s
    }

    #[test]
    fn full_window_contains_all_ticks() {
        let store = build_store();
        let w = TimeSeriesWindow::full(&store, 0xAA);
        assert_eq!(w.len(), 20);
    }

    #[test]
    fn range_window_filters_correctly() {
        let store = build_store();
        let w = TimeSeriesWindow::new(&store, 0xAA, 5, 10);
        assert_eq!(w.len(), 6);
        assert_eq!(w.first_price(), Some(50));
        assert_eq!(w.last_price(), Some(100));
    }

    #[test]
    fn price_change_computed() {
        let store = build_store();
        let w = TimeSeriesWindow::new(&store, 0xAA, 1, 5);
        assert_eq!(w.price_change(), Some(40)); // 50 - 10
    }

    #[test]
    fn prices_vec_correct_length() {
        let store = build_store();
        let w = TimeSeriesWindow::new(&store, 0xAA, 1, 10);
        assert_eq!(w.prices().len(), 10);
    }

    #[test]
    fn empty_window_is_empty() {
        let store = build_store();
        let w = TimeSeriesWindow::new(&store, 0xAA, 100, 200);
        assert!(w.is_empty());
        assert_eq!(w.price_change(), None);
    }

    #[test]
    fn ohlc_bars_generated() {
        let store = build_store();
        let w = TimeSeriesWindow::full(&store, 0xAA);
        let bars = w.ohlc_bars(5);
        assert_eq!(bars.len(), 4); // 20 ticks / 5-epoch windows = 4 bars
    }
}
