#![forbid(unsafe_code)]
//! # StewardLens — Sovereign Auditor for DataStewards
//!
//! StewardLens is a **read-only** observation and anomaly-detection layer.
//! It never writes to EnkiDB, never modifies particles, and never mints KAKIs.
//! Its sole job is to surface anomalies to the DataSteward so a human can decide.
//!
//! ## Four device tiers (SL-TIER-001):
//!
//! | Tier       | FPS | Features                                       |
//! |------------|-----|------------------------------------------------|
//! | Smartwatch | N/A | Haptic + 2-button alert only                   |
//! | Smartphone | 30  | WGPU 2D map + AR camera overlay                |
//! | Tablet     | 60  | WGPU 3D + AICouncil vote viewer                |
//! | Desktop    | 120 | Multi-monitor + HeptaScript editor             |
//!
//! ## Eight anomaly codes (SL-ANOMALY-001):
//! SA-001 through SA-008 — see `AnomalyCode` enum.

pub mod alert;
pub mod anomaly;
pub mod audit_view;
pub mod device;

pub use alert::{AlertSeverity, StewardAlert};
pub use anomaly::{AnomalyCode, AnomalyEvent};
pub use audit_view::AuditView;
pub use device::DeviceTier;
