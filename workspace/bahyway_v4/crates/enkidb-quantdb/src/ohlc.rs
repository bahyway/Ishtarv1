//! OhlcBar — Open/High/Low/Close/Volume bar aggregation.
//!
//! The OhlcAggregator consumes TickRecords and produces OhlcBars over a
//! fixed epoch window (e.g. 10 epochs = one "candle").
//!
//! All prices remain fixed-point i64 — no floating-point in the bar.
//! Conversion to real prices is done externally via AssetDescriptor.

use crate::tick::TickRecord;

/// One OHLC bar for an asset over a fixed epoch window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OhlcBar {
    /// Sovereign asset identity.
    pub asset_hash:   u32,
    /// First epoch in this bar's window.
    pub epoch_open:   u32,
    /// Last epoch in this bar's window (inclusive).
    pub epoch_close:  u32,
    /// First trade price in this window.
    pub open:         i64,
    /// Highest price in this window.
    pub high:         i64,
    /// Lowest price in this window.
    pub low:          i64,
    /// Last trade price in this window.
    pub close:        i64,
    /// Total volume × 1000 in this window.
    pub raw_volume:   u64,
    /// Number of ticks aggregated.
    pub tick_count:   u32,
    /// Volume-weighted average price × price_scale (VWAP raw).
    pub vwap_raw:     i64,
}

impl OhlcBar {
    /// Returns `true` if close > open (bullish candle).
    pub fn is_bullish(&self) -> bool { self.close > self.open }

    /// Bar body size in price ticks (absolute value of close − open).
    pub fn body_size(&self) -> i64 { (self.close - self.open).abs() }

    /// Total wick size: (high − low) − body_size.
    pub fn wick_size(&self) -> i64 { (self.high - self.low) - self.body_size() }

    /// Real volume (raw_volume / 1000).
    pub fn real_volume(&self) -> f64 { self.raw_volume as f64 / 1000.0 }

    /// Price change from open to close (signed).
    pub fn price_change(&self) -> i64 { self.close - self.open }
}

/// Streaming OHLC aggregator — consumes ticks and builds bars over a window.
pub struct OhlcAggregator {
    pub asset_hash:   u32,
    /// Epoch window size per bar (e.g. 10 = bar covers epochs N..N+9).
    pub window_size:  u32,
    current:          Option<OhlcBarBuilder>,
    /// Completed bars, oldest first.
    completed:        Vec<OhlcBar>,
}

struct OhlcBarBuilder {
    asset_hash:  u32,
    epoch_open:  u32,
    open:        i64,
    high:        i64,
    low:         i64,
    close:       i64,
    raw_volume:  u64,
    tick_count:  u32,
    vwap_num:    i128,  // sum of (price × volume) for VWAP
}

impl OhlcBarBuilder {
    fn new(asset_hash: u32, tick: &TickRecord) -> Self {
        OhlcBarBuilder {
            asset_hash,
            epoch_open: tick.epoch,
            open:       tick.raw_price,
            high:       tick.raw_price,
            low:        tick.raw_price,
            close:      tick.raw_price,
            raw_volume: tick.raw_volume,
            tick_count: 1,
            vwap_num:   tick.raw_price as i128 * tick.raw_volume as i128,
        }
    }

    fn update(&mut self, tick: &TickRecord) {
        if tick.raw_price > self.high { self.high = tick.raw_price; }
        if tick.raw_price < self.low  { self.low  = tick.raw_price; }
        self.close       = tick.raw_price;
        self.raw_volume += tick.raw_volume;
        self.tick_count += 1;
        self.vwap_num   += tick.raw_price as i128 * tick.raw_volume as i128;
    }

    fn finish(self, epoch_close: u32) -> OhlcBar {
        let vwap_raw = if self.raw_volume > 0 {
            (self.vwap_num / self.raw_volume as i128) as i64
        } else {
            self.close
        };
        OhlcBar {
            asset_hash:  self.asset_hash,
            epoch_open:  self.epoch_open,
            epoch_close,
            open:        self.open,
            high:        self.high,
            low:         self.low,
            close:       self.close,
            raw_volume:  self.raw_volume,
            tick_count:  self.tick_count,
            vwap_raw,
        }
    }
}

impl OhlcAggregator {
    /// Create a new aggregator for `asset_hash` with bars of `window_size` epochs.
    pub fn new(asset_hash: u32, window_size: u32) -> Self {
        OhlcAggregator { asset_hash, window_size, current: None, completed: Vec::new() }
    }

    /// Ingest one tick.  Completes the current bar if the tick's epoch falls
    /// outside the current window.
    pub fn ingest(&mut self, tick: &TickRecord) {
        debug_assert_eq!(tick.asset_hash, self.asset_hash, "wrong asset");

        match &self.current {
            None => {
                self.current = Some(OhlcBarBuilder::new(self.asset_hash, tick));
            }
            Some(b) => {
                let bar_end = b.epoch_open + self.window_size - 1;
                if tick.epoch > bar_end {
                    // Close current bar and start a new one
                    let builder = self.current.take().unwrap();
                    self.completed.push(builder.finish(bar_end));
                    self.current = Some(OhlcBarBuilder::new(self.asset_hash, tick));
                } else {
                    self.current.as_mut().unwrap().update(tick);
                }
            }
        }
    }

    /// Peek at the currently-open (incomplete) bar.
    pub fn current_bar(&self) -> Option<OhlcBar> {
        self.current.as_ref().map(|b| OhlcBar {
            asset_hash:  b.asset_hash,
            epoch_open:  b.epoch_open,
            epoch_close: b.epoch_open + self.window_size - 1,
            open:        b.open,
            high:        b.high,
            low:         b.low,
            close:       b.close,
            raw_volume:  b.raw_volume,
            tick_count:  b.tick_count,
            vwap_raw:    if b.raw_volume > 0 {
                (b.vwap_num / b.raw_volume as i128) as i64
            } else { b.close },
        })
    }

    /// Flush the current open bar into the completed list.
    pub fn flush(&mut self) {
        if let Some(builder) = self.current.take() {
            let epoch_close = builder.epoch_open + self.window_size - 1;
            self.completed.push(builder.finish(epoch_close));
        }
    }

    /// All completed bars (oldest first).
    pub fn completed_bars(&self) -> &[OhlcBar] { &self.completed }

    /// Total bars completed (not including the open bar).
    pub fn bar_count(&self) -> usize { self.completed.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick::TickRecord;

    fn tick(price: i64, vol: u64, epoch: u32) -> TickRecord {
        TickRecord::new(0xABCD, price, vol, 1, epoch)
    }

    #[test]
    fn single_tick_creates_open_bar() {
        let mut agg = OhlcAggregator::new(0xABCD, 10);
        agg.ingest(&tick(15000, 1000, 1));
        let bar = agg.current_bar().unwrap();
        assert_eq!(bar.open,  15000);
        assert_eq!(bar.high,  15000);
        assert_eq!(bar.low,   15000);
        assert_eq!(bar.close, 15000);
        assert_eq!(bar.tick_count, 1);
    }

    #[test]
    fn high_low_track_correctly() {
        let mut agg = OhlcAggregator::new(0xABCD, 10);
        agg.ingest(&tick(100, 1000, 1));
        agg.ingest(&tick(120, 1000, 3));
        agg.ingest(&tick(90,  1000, 7));
        agg.ingest(&tick(110, 1000, 9));
        let bar = agg.current_bar().unwrap();
        assert_eq!(bar.high, 120);
        assert_eq!(bar.low,  90);
        assert_eq!(bar.open, 100);
        assert_eq!(bar.close, 110);
    }

    #[test]
    fn window_boundary_closes_bar() {
        let mut agg = OhlcAggregator::new(0xABCD, 10);
        agg.ingest(&tick(100, 1000, 1));   // bar 1: epochs 1-10
        agg.ingest(&tick(110, 1000, 11));  // epoch 11 starts bar 2
        assert_eq!(agg.bar_count(), 1);
        let completed = &agg.completed_bars()[0];
        assert_eq!(completed.open, 100);
        assert_eq!(completed.close, 100);
    }

    #[test]
    fn vwap_computed_correctly() {
        let mut agg = OhlcAggregator::new(0xABCD, 10);
        // 2 ticks: price 100 vol 1000, price 200 vol 1000 → VWAP = 150
        agg.ingest(&tick(100, 1000, 1));
        agg.ingest(&tick(200, 1000, 5));
        let bar = agg.current_bar().unwrap();
        assert_eq!(bar.vwap_raw, 150);
    }

    #[test]
    fn bullish_bar_detected() {
        let mut agg = OhlcAggregator::new(0xABCD, 10);
        agg.ingest(&tick(100, 1000, 1));
        agg.ingest(&tick(110, 1000, 5));
        assert!(agg.current_bar().unwrap().is_bullish());
    }

    #[test]
    fn bearish_bar_detected() {
        let mut agg = OhlcAggregator::new(0xABCD, 10);
        agg.ingest(&tick(110, 1000, 1));
        agg.ingest(&tick(100, 1000, 5));
        assert!(!agg.current_bar().unwrap().is_bullish());
    }

    #[test]
    fn flush_moves_open_bar_to_completed() {
        let mut agg = OhlcAggregator::new(0xABCD, 10);
        agg.ingest(&tick(100, 1000, 1));
        assert_eq!(agg.bar_count(), 0);
        agg.flush();
        assert_eq!(agg.bar_count(), 1);
        assert!(agg.current_bar().is_none());
    }

    #[test]
    fn multiple_windows_produce_multiple_bars() {
        let mut agg = OhlcAggregator::new(0xABCD, 5);
        for epoch in [1u32, 6, 11, 16, 21] {
            agg.ingest(&tick(100 + epoch as i64, 1000, epoch));
        }
        // epochs 1,6,11,16 each start new bars; epoch 21 is open
        assert_eq!(agg.bar_count(), 4);
    }

    #[test]
    fn body_and_wick_sizes_correct() {
        let bar = OhlcBar {
            asset_hash: 1, epoch_open: 1, epoch_close: 10,
            open: 100, high: 130, low: 80, close: 120,
            raw_volume: 0, tick_count: 0, vwap_raw: 0,
        };
        assert_eq!(bar.body_size(), 20);  // |120-100|
        assert_eq!(bar.wick_size(), 30);  // (130-80) - 20
    }

    #[test]
    fn price_change_signed() {
        let up = OhlcBar { asset_hash: 1, epoch_open: 1, epoch_close: 5,
            open: 100, high: 110, low: 95, close: 108,
            raw_volume: 0, tick_count: 0, vwap_raw: 0 };
        let dn = OhlcBar { asset_hash: 1, epoch_open: 1, epoch_close: 5,
            open: 100, high: 102, low: 85, close: 90,
            raw_volume: 0, tick_count: 0, vwap_raw: 0 };
        assert_eq!(up.price_change(),  8);
        assert_eq!(dn.price_change(), -10);
    }
}
