//! AssetDescriptor — sovereign asset identity and metadata.
//!
//! An asset in BahyWay is a particle.  Its identity lives in a 32-bit
//! asset_hash (the lower 32 bits of its KAKI uuid_hash field).
//! The ticker string is a display label only — never used as a key.

/// Asset class classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetClass {
    /// Listed equity (stock, ETF).
    Equity,
    /// Fixed income (bond, sukuk).
    FixedIncome,
    /// Foreign exchange pair.
    Fx,
    /// Commodity (oil, gold, wheat).
    Commodity,
    /// Crypto asset.
    Crypto,
    /// Real-estate investment trust.
    Reit,
    /// Sovereign index or benchmark.
    Index,
}

impl AssetClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Equity => "EQUITY",
            Self::FixedIncome => "FIXED_INCOME",
            Self::Fx => "FX",
            Self::Commodity => "COMMODITY",
            Self::Crypto => "CRYPTO",
            Self::Reit => "REIT",
            Self::Index => "INDEX",
        }
    }
}

/// Exchange / market identifier (sovereign u8 code).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketId(pub u8);

impl MarketId {
    pub const NYSE: MarketId = MarketId(0x01);
    pub const NASDAQ: MarketId = MarketId(0x02);
    pub const LSE: MarketId = MarketId(0x03);
    pub const ISX: MarketId = MarketId(0x10); // Iraq Stock Exchange
    pub const ADX: MarketId = MarketId(0x11); // Abu Dhabi
    pub const DFM: MarketId = MarketId(0x12); // Dubai
    pub const TADAWUL: MarketId = MarketId(0x13); // Saudi
    pub const CRYPTO: MarketId = MarketId(0xFF);
}

/// Sovereign asset descriptor — the display metadata for one asset particle.
///
/// The `asset_hash` is the primary key used in `TickRecord` and `OhlcBar`.
/// It is the lower 32 bits of the asset's KAKI uuid_hash.
#[derive(Debug, Clone)]
pub struct AssetDescriptor {
    /// Sovereign 32-bit asset identity key.
    pub asset_hash: u32,
    /// Human-readable ticker symbol (display only, never a key).
    pub ticker: &'static str,
    /// Full asset name.
    pub name: &'static str,
    /// Asset class.
    pub asset_class: AssetClass,
    /// Exchange / market.
    pub market: MarketId,
    /// Fixed-point price scale: price in ticks / price_scale = real price.
    /// e.g. price_scale = 100 means prices stored in cents.
    pub price_scale: u32,
    /// Currency code (ISO 4217 numeric, e.g. 840 = USD, 368 = IQD).
    pub currency_id: u16,
}

impl AssetDescriptor {
    pub fn new(
        asset_hash: u32,
        ticker: &'static str,
        name: &'static str,
        asset_class: AssetClass,
        market: MarketId,
        price_scale: u32,
        currency_id: u16,
    ) -> Self {
        Self {
            asset_hash,
            ticker,
            name,
            asset_class,
            market,
            price_scale,
            currency_id,
        }
    }

    /// Real price from raw fixed-point tick price.
    pub fn to_real_price(&self, raw: i64) -> f64 {
        raw as f64 / self.price_scale as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_class_labels_non_empty() {
        for cls in [
            AssetClass::Equity,
            AssetClass::FixedIncome,
            AssetClass::Fx,
            AssetClass::Commodity,
            AssetClass::Crypto,
            AssetClass::Reit,
            AssetClass::Index,
        ] {
            assert!(!cls.label().is_empty());
        }
    }

    #[test]
    fn to_real_price_correct() {
        let asset = AssetDescriptor::new(
            0x1234,
            "AAPL",
            "Apple Inc.",
            AssetClass::Equity,
            MarketId::NASDAQ,
            100,
            840,
        );
        assert!((asset.to_real_price(15050) - 150.50).abs() < 1e-9);
    }

    #[test]
    fn market_id_constants_distinct() {
        assert_ne!(MarketId::NYSE.0, MarketId::NASDAQ.0);
        assert_ne!(MarketId::ISX.0, MarketId::NYSE.0);
    }
}
