//! TickRecord — one sovereign price tick for an asset particle.
//!
//! Prices are stored as i64 fixed-point integers (real = raw / price_scale).
//! This avoids all floating-point precision issues in financial storage.
//!
//! Sovereign constraints:
//! - No f64 in storage.  Only in display / computation via AssetDescriptor.
//! - Epoch = u32 BahyWay sovereign tick.  Not Unix seconds.
//! - Volume = u64 (shares, contracts, or base-currency units × 1000).

/// One price tick for an asset.
///
/// Stored as fixed-point to avoid floating-point accumulation errors.
/// Use `AssetDescriptor::to_real_price(raw)` to recover the human price.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickRecord {
    /// Sovereign 32-bit asset identity (lower 32 bits of KAKI uuid_hash).
    pub asset_hash:  u32,
    /// Fixed-point price: real_price = raw_price / asset.price_scale.
    pub raw_price:   i64,
    /// Volume traded at this tick (units × 1000 for fractional support).
    pub raw_volume:  u64,
    /// Sequence number within the epoch (for same-epoch ordering).
    pub sequence:    u32,
    /// BahyWay sovereign epoch at time of this tick.
    pub epoch:       u32,
    /// Bid–ask spread in price ticks (0 if not available).
    pub spread_ticks: u32,
    /// Trade flags: bit 0 = buy-initiated, bit 1 = block trade.
    pub flags:       u8,
}

impl TickRecord {
    /// Construct a minimal tick (no spread, no flags).
    pub fn new(asset_hash: u32, raw_price: i64, raw_volume: u64, sequence: u32, epoch: u32) -> Self {
        Self { asset_hash, raw_price, raw_volume, sequence, epoch, spread_ticks: 0, flags: 0 }
    }

    /// Construct a full tick with spread and flags.
    pub fn full(
        asset_hash: u32, raw_price: i64, raw_volume: u64,
        sequence: u32, epoch: u32, spread_ticks: u32, flags: u8,
    ) -> Self {
        Self { asset_hash, raw_price, raw_volume, sequence, epoch, spread_ticks, flags }
    }

    /// Returns `true` if the flags indicate a buy-initiated trade.
    pub fn is_buy(&self) -> bool { self.flags & 0x01 != 0 }

    /// Returns `true` if this is a block trade.
    pub fn is_block(&self) -> bool { self.flags & 0x02 != 0 }

    /// Real volume (divides by 1000 for fractional-unit assets).
    pub fn real_volume(&self) -> f64 { self.raw_volume as f64 / 1000.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tick_has_zero_spread_and_flags() {
        let t = TickRecord::new(0xABCD, 15000, 500_000, 1, 42);
        assert_eq!(t.spread_ticks, 0);
        assert_eq!(t.flags, 0);
        assert!(!t.is_buy());
        assert!(!t.is_block());
    }

    #[test]
    fn full_tick_flags_decoded() {
        let t = TickRecord::full(0x1234, 20000, 1_000_000, 5, 10, 3, 0x03);
        assert!(t.is_buy());
        assert!(t.is_block());
    }

    #[test]
    fn real_volume_divides_by_1000() {
        let t = TickRecord::new(0x1, 100, 1500, 1, 1);
        assert!((t.real_volume() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn tick_ordering_by_epoch_then_sequence() {
        let t1 = TickRecord::new(0x1, 100, 0, 1, 5);
        let t2 = TickRecord::new(0x1, 101, 0, 2, 5);
        let t3 = TickRecord::new(0x1, 99,  0, 1, 6);
        // t1 < t2 (same epoch, lower sequence)
        assert!(t1.epoch == t2.epoch && t1.sequence < t2.sequence);
        // t3 is in a later epoch
        assert!(t3.epoch > t1.epoch);
    }

    #[test]
    fn negative_price_allowed_for_derivatives() {
        let t = TickRecord::new(0xFF, -500, 1000, 1, 1);
        assert_eq!(t.raw_price, -500);
    }
}
