//! eridu-supervisor — lifecycle and health management (§11.3).
//!
//! Extended in v4.0.1 with HardwareHealthReport — Shedu telemetry integration.

pub mod supervisor;
pub mod hardware_health;

pub use supervisor::{EriduSupervisor, HealthStatus};
pub use hardware_health::HardwareHealthReport;
