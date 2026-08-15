//! quant-engine — Sovereign Quantitative Analytics Engine
//!
//! Layer 5 analytics crate.  Consumes sovereign tick data from
//! `enkidb-quantdb` and produces institutional-grade quantitative metrics
//! using only sovereign Rust algorithms (no Polars, no ndarray, no BLAS).
//!
//! # Module Map
//!
//! | Module | Provides |
//! |--------|----------|
//! | `returns` | Log-return series, period returns, cumulative return |
//! | `rolling` | Rolling mean, std-dev, min, max, Bollinger bands, EMA |
//! | `covariance` | Cross-asset covariance matrix, correlation, beta |
//! | `risk` | VaR (Historical), CVaR, Sharpe, Sortino, max drawdown |
//! | `factor` | VGCA 7D FSV → 5 sovereign quant factors |
//! | `particle` | Asset as a BahyWay particle with B11 quality lane |
//!
//! # Sovereign Constraints
//! - No external deps.  No ndarray, nalgebra, polars, or statrs.
//! - All computations in f64.  No unsafe code.
//! - Prices received as i64 fixed-point from enkidb-quantdb.
//! - Asset identity = u32 asset_hash (KAKI-derived). Never a string.
//! - B11 = (score × 240).round() as u8 — ADR-001, always 240, never 255.
//!
//! # Quick Start
//!
//! ```rust
//! use quant_engine::returns::ReturnSeries;
//! use quant_engine::risk::RiskReport;
//! use quant_engine::rolling::RollingWindow;
//!
//! // Simulated price series (fixed-point, scale=100 → cents)
//! let prices: Vec<i64> = vec![10000, 10150, 10080, 10300, 10250, 10400];
//! let scale = 100u32;
//!
//! let returns = ReturnSeries::from_prices(&prices, scale);
//! assert_eq!(returns.len(), 5); // n-1 returns from n prices
//!
//! let risk = RiskReport::from_returns(returns.log_returns(), 0.95);
//! assert!(risk.sharpe.is_finite());
//! ```

#![forbid(unsafe_code)]

pub mod returns;
pub mod rolling;
pub mod covariance;
pub mod risk;
pub mod factor;
pub mod particle;

pub use returns::{ReturnSeries, ReturnKind};
pub use rolling::{RollingWindow, BollingerBands, ExponentialMovingAverage};
pub use covariance::{CovMatrix, CorrelationMatrix};
pub use risk::{RiskReport, DrawdownSeries};
pub use factor::{QuantFactor, FactorModel, FactorScores};
pub use particle::{AssetParticle, AssetQualityLane};

pub mod prelude {
    pub use crate::{
        ReturnSeries, ReturnKind,
        RollingWindow, BollingerBands, ExponentialMovingAverage,
        CovMatrix, CorrelationMatrix,
        RiskReport, DrawdownSeries,
        QuantFactor, FactorModel, FactorScores,
        AssetParticle, AssetQualityLane,
    };
}
