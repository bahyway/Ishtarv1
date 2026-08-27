#![forbid(unsafe_code)]
//! # ASHNAN 𒀭𒀳 — Sovereign Climate-Agriculture-Livestock Intelligence
//!
//! Named for the Sumerian goddess of grain — her presence meant abundance,
//! her absence meant famine. ASHNAN is the BahyWay.Ecosystem v4.0
//! climate-agriculture-livestock intelligence module (BC-AGRI-001),
//! completing the ENKI-TERRA suite alongside NANSHE and ESARHADDON.
//!
//! ## Three Threat Domains
//! | Domain | Primary Threat | ASHNAN Engine |
//! |--------|---------------|---------------|
//! | SOIL & CROP | Drought, river reduction, heat stress | Crop Mortality Index (CMI) |
//! | PEST | Warming winters, extended breeding range | Incursion Risk Index (IRI) |
//! | LIVESTOCK | Heat stress, forage decline, disease | Livestock Mortality Index (LMI) |
//!
//! ## KAKI Particle Classes
//! - `Soil/Crop Event` — 0x02/ZIKRU — soil moisture, NDVI, weather station
//! - `Pest Identity`   — 0x01/KISHIB — trap catch, species ID, outbreak report
//! - `Livestock Event` — 0x02/PARZU — herd sensor, veterinary report, mortality
//! - `External Ingest` — 0x03 CrossTribe — FAO/WOAH/met-service with provenance EAV
//!
//! ## Confidence Tier Architecture
//! | Tier | Source Class | Treatment |
//! |------|-------------|-----------|
//! | TIER 0 | Sovereign sensor (Diyala pilot) | Full GOLDEN/FUZZY/DEAD, direct TIAMAT input |
//! | TIER 1 | Official direct feed | High confidence, cross-validated |
//! | TIER 2 | Official bulletin (FAO, WOAH) | Moderate, lag tracked |
//! | TIER 3 | Modelled / third-hand | Advisory only |
//!
//! ## Implementation Roadmap (BC-AGRI-001)
//! | Phase | Deliverable | Playbooks |
//! |-------|-------------|-----------|
//! | 0 | KISPU HeadStore fix (shared prerequisite) | 98 |
//! | 1 | ASHNAN KAKI schema + provenance EAV ontology | 156–160 |
//! | 2 | Diyala agricultural sensor pilot deployment | 161–166 |
//! | 3 | TIAMAT CMI formula | 167–171 |
//! | 4 | TIAMAT IRI + degree-day accumulation | 172–175 |
//! | 5 | TIAMAT LMI + THI + KIBRATU HIDDEN_PATTERN cascade | 176–180 |
//! | 6 | International ingestion adapters (FAO/WOAH/met) | 181–186 |
//! | 7 | DubSar Theater ASHNAN scenes | 187–192 |
//! | 8 | Regional extension framework (Jordan/Syria/Fertile Crescent) | 193–197 |

pub mod cmi;
pub mod colour;
pub mod error;
pub mod iri;
pub mod kaki;
pub mod lmi;
pub mod provenance;

pub use cmi::{Cmi, CmiState};
pub use colour::colour_id_b11;
pub use error::AshnanError;
pub use iri::{Iri, IriState};
pub use kaki::{AshnanKakiClass, ExternalIngestionParticle};
pub use lmi::{Lmi, LmiState};
pub use provenance::{ConfidenceTier, ProvenanceBlock};
