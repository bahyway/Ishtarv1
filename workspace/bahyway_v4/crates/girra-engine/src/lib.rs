//! girra-engine — the lamp of the ecosystem.
//!
//! From the iPhone-session PB-166 review, renamed from "NuskuEngine":
//! `nusku-engine`/`nusku-score`/`nusku-fuzzy` already exist in this
//! workspace and mean something else real (body-scan security-threat
//! classification, not monitoring). Girra (fire and light) carries the
//! same "observability as illumination" metaphor the original design
//! intended, without the collision.
//!
//! ARCHITECTURE (sovereignty decisions carried over from the review):
//!   - Metrics ARE particles, in intent: every sample is meant to become a
//!     KAKI Event particle (kaki_type 0x02) destined for EnkiODB.
//!     Monitoring is not a separate universe with its own TSDB -- it is
//!     another orbit. The EMIT wiring to enkidb-ingest is separate, later
//!     work (this crate's in-memory registry is the live window; the
//!     durable EnkiODB write path is not implemented here).
//!   - Host collector reads /proc directly (std only -- no sysinfo crate).
//!   - Engine metrics arrive over a minimal TCP line protocol:
//!       metric_name value unix_millis\n
//!   - UI: eframe/egui native window, the SAME version dubsar-visualizer
//!     already uses (0.29) -- one sovereign GUI stack, not two.

pub mod collector;
pub mod ingest;
pub mod metrics;

pub use metrics::{gauge_laws, Band, Registry, Sample, Thresholds};
