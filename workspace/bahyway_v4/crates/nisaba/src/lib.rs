#![forbid(unsafe_code)]
//! # NISABA — Pattern Discovery Service
//!
//! NISABA monitors particle trajectories in 7D orbital space, applies Genetic
//! Algorithm (GA) clustering to detect emergent movement patterns, and mints
//! Pattern-KAKIs for confirmed clusters.
//!
//! ## Processing pipeline:
//! 1. **Trajectory sampling** — buffer sliding-window trajectory segments per particle
//! 2. **GA clustering** — group similar trajectories by centroid proximity in 7D
//! 3. **Confidence evaluation** — compute cluster confidence (size × coherence)
//! 4. **Pattern-KAKI derivation** — deterministic mint via `derive_pattern_kaki()`
//! 5. **Lifecycle assignment** — Emerging (≥0.60), Stable (≥0.85), escalate to council
//!
//! ## Fanout constraint (E7 Viazovska):
//! Each Pattern-KAKI can reference at most **126 constituent KAKIs** (kissing number).
//!
//! ## Orchestrator (granted 2026-07-18)
//! `orchestrator::NisabaOrchestrator` is the Architect's Internal
//! Interface: it wires together StoryEngine's high-priority alerts (via
//! `story-sentinel`), the Data Steward's review queue
//! (`data-steward-station::StewardStation`), and LamassuEngine's
//! topological (TDA) readings into one continuously-updated digest.
//! See `orchestrator`'s doc comment for the CSR-08 propose-not-ratify
//! discipline it follows.
//!
//! HONEST FLAG (carried over from the EriduOS UI panel rename, PB-189):
//! this crate's GA-clustering pattern discovery and the EriduOS hologram
//! agent panel share the name NISABA but were, until this orchestrator,
//! two unrelated real components. The orchestrator is what actually joins
//! them: it lives in this crate, and the UI panel now has a real backend
//! to read from instead of static strings.

pub mod cluster;
pub mod trajectory;
pub mod discovery;
pub mod orchestrator;
pub mod autonomy;

pub use cluster::{GaCluster, ClusterMetrics};
pub use trajectory::{TrajectorySegment, TrajectoryBuffer};
pub use discovery::{NisabaDiscovery, DiscoveredPattern};
pub use orchestrator::{NisabaOrchestrator, NisabaDigest, NisabaNote, AutonomousResolution};
pub use autonomy::{NisabaEnvironment, AutonomyClass, GrantClaim, NisabaGrant};
