//! enkidb-quantdb — Sovereign Quantitative Time-Series Store
//!
//! Layer 2 sovereign storage for asset particles.  Stores tick data and
//! OHLC bars with epoch-indexed access.  All asset identity is rooted in
//! a 16-byte sovereign KAKI token — no external ticker string ownership.
//!
//! # Design
//!
//! ```text
//! Tick feed → QuantDbStore::push_tick()
//!           → epoch-indexed TickRecord storage
//!           → OhlcAggregator::ingest() → OhlcBar per window
//!           → TimeSeriesWindow::range(from, to) → slice
//! ```
//!
//! # Sovereign Constraints
//! - No external deps.  No Polars, no Arrow, no chrono.
//! - Epoch = u32 BahyWay sovereign tick (not Unix timestamp).
//! - All prices are i64 fixed-point: value = raw / price_scale.
//! - Asset identity = 16-byte KAKI hash (never a string ticker).
//!
//! # Quick Start
//!
//! ```rust
//! use enkidb_quantdb::{QuantDbStore, TickRecord, OhlcAggregator};
//!
//! let mut store = QuantDbStore::new();
//! store.push_tick(TickRecord::new(0xABCD_1234u32, 150_00, 1000, 1, 1));
//! store.push_tick(TickRecord::new(0xABCD_1234u32, 151_50, 2000, 1, 2));
//!
//! let ticks = store.ticks_for_asset(0xABCD_1234u32);
//! assert_eq!(ticks.len(), 2);
//!
//! let mut agg = OhlcAggregator::new(0xABCD_1234u32, 10);
//! for t in ticks { agg.ingest(t); }
//! let bar = agg.current_bar().unwrap();
//! assert!(bar.high >= bar.low);
//! ```

#![forbid(unsafe_code)]

pub mod tick;
pub mod ohlc;
pub mod store;
pub mod window;
pub mod asset;

pub use tick::TickRecord;
pub use ohlc::{OhlcBar, OhlcAggregator};
pub use store::QuantDbStore;
pub use window::TimeSeriesWindow;
pub use asset::{AssetDescriptor, AssetClass, MarketId};

pub mod prelude {
    pub use crate::{
        TickRecord, OhlcBar, OhlcAggregator,
        QuantDbStore, TimeSeriesWindow,
        AssetDescriptor, AssetClass, MarketId,
    };
}
