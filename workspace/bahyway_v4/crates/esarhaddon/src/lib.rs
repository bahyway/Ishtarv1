#![forbid(unsafe_code)]
//! # ESARHADDON 𒀭𒀸𒋩 — Sovereign Earthquake Rescue Intelligence
//!
//! Named for Aššur-ahhe-iddina, the Assyrian king who rebuilt Babylon from rubble.
//! ESARHADDON is the BahyWay.Ecosystem v4.0 earthquake rescue intelligence module,
//! part of the ENKI-TERRA environmental intelligence suite (BC-QUAKE-001).
//!
//! ## Capabilities
//! - Seismic KAKI particle ingestion (accelerometer, geophone, tiltmeter, crack monitors)
//! - Survivor Signal Aggregator — Bayesian fusion of acoustic/thermal/RF/robot signals
//! - TIAMAT Engine 5 Structural Mortality Index (SMI) — secondary collapse prediction
//! - HeptaMap E7 rescue grid — multi-team GREEN/AMBER/RED/BLACK zone coordination
//! - Atmospheric threat modelling — CH₄/CO/H₂S/PM2.5 kill-zone dispersion (NANSHE bridge)
//! - DubSar Theater scenes — RUBBLE FIELD · SMI HEATMAP · SURVIVOR ORBIT · GAS PLUME · TIAMAT FORECAST
//!
//! ## KAKI Particle Classes
//! - `0x01 / KISHIB` — Survivor Identity particle (acoustic/thermal/RF/robot signal)
//! - `0x02 / ZIKRU` — Seismic Event particle (accelerometer, geophone, aftershock)
//! - `0x02 / PARZU` — Structural State particle (SMI snapshot per HeptaMap cell, 60 s cadence)
//!
//! ## Pipeline Position
//! ENKI-TERRA Suite | Module 3 | TIAMAT Engine 5 + ESARHADDON SMI
//! 100 % offline sovereign field unit — zero cloud dependency.
//!
//! ## Implementation Roadmap (BC-QUAKE-001)
//! | Phase | Deliverable | Playbooks |
//! |-------|-------------|-----------|
//! | 0 | KISPU HeadStore fix (shared prerequisite) | 98 |
//! | 1 | ESARHADDON KAKI schema + EAV ontology | 119–122 |
//! | 2 | Survivor Signal Aggregator | 123–127 |
//! | 3 | TIAMAT SMI + KIBRATU PARALLEL_FAILURE cascade | 128–133 |
//! | 4 | HeptaMap rescue grid integration | 134–138 |
//! | 5 | T+1/6/24hr collapse forecast + NANSHE gas bridge | 139–143 |
//! | 6 | DubSar Theater scenes | 144–150 |
//! | 7 | Field ruggedisation + NĀRU audit journal | 151–155 |

pub mod kaki;
pub mod smi;
pub mod survivor;
pub mod grid;
pub mod atmosphere;
pub mod error;

pub use error::EsarhaddonError;
pub use kaki::{KakiClass, SeismicEvent, SurvivorSignal, StructuralState};
pub use smi::{Smi, SmiState};
pub use survivor::SurvivorAggregator;
pub use grid::{RescueZone, ZoneClass};
pub use atmosphere::AtmosphericThreat;
