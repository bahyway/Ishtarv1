//! ea-agent-oracle — EaAgent Early Warning System and Stability Oracle.
//!
//! The Oracle mode of EaAgent: proactive monitoring, not reactive answering.
//! It watches tribes continuously and warns BEFORE collapse happens.
//!
//! Three oracle capabilities:
//!   1. CollapsePredictor  — forecasts tribe collapse from spectral radius trend
//!   2. PauliMonitor       — continuous Pauli Exclusion surveillance
//!   3. GemRateOracle      — tracks GEM rate vs ADR-004 target (35.4%)
//!
//! All warnings are KAKI-stamped AgentDecisions — immutable audit trail.
#![forbid(unsafe_code)]

pub mod collapse_predictor;
pub mod pauli_monitor;
pub mod gem_oracle;
pub mod oracle_engine;

pub use collapse_predictor::{CollapsePredictor, CollapseRisk, TrendPoint};
pub use pauli_monitor::{PauliMonitor, MonitorReport};
pub use gem_oracle::{GemOracle, GemReport};
pub use oracle_engine::{OracleEngine, OracleReport, OracleAlert, AlertLevel};
