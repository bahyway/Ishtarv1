//! eridu-supervisor — lifecycle and health management (§11.3).
//!
//! Extended in v4.0.1 with HardwareHealthReport — Shedu telemetry integration.

pub mod hardware_health;
pub mod supervisor;

pub use hardware_health::HardwareHealthReport;
pub use supervisor::{EriduSupervisor, HealthStatus};
