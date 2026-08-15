//! navi-photog-bridge — connects `photogrammetry-ingest` (drone point
//! clouds) to `navi-engine` (the sovereign routing engine), so that a
//! drone survey of a burial ground becomes a real, routable map:
//! `ParticleMap::place`d as `MapKind::GraveMarker` particles, wired with
//! candidate shortcut-path edges, and immediately queryable by
//! `MachineRouter`'s already-existing, already-tested A* — no routing
//! logic is duplicated or reimplemented here.
//!
//! This is the piece named as missing when drone photogrammetry was
//! first added to this workspace: not structure-from-motion (still
//! correctly out of scope — a separate, mature field with real existing
//! engines), and not a new renderer (`navi-engine`'s own `HubbleTile`
//! Z0-Z6 zoom system already IS the "minimize rendering" structure —
//! `particles_in_tile` already filters by `zoom_min`; nothing new was
//! needed there either). What was actually missing, and what this crate
//! builds: the DATA bridge between an already-reconstructed point cloud
//! and that already-real map + router.
#![forbid(unsafe_code)]

pub mod bridge;
pub mod geo_convert;
