//! idu-batching — IDU Probe Batching for production-scale dashboards (§8.4)
//!
//! Three-layer optimization that preserves the no-cache rule of §8.3:
//!
//! Layer 1 — Anchor-Set Deduplication:
//!   Collect unique anchor KAKIs across all links, project each once.
//! Layer 2 — Home-Node Parallelism (structural; in-process here):
//!   Independent particle projections are logically parallel.
//! Layer 3 — Time-Window Coalescing (request-scoped; discarded after batch):
//!   Results live only for the duration of one batch operation.
//!
//! A dashboard rendering 10,000 links with 2,000 unique anchors performs
//! ~2,000 projections, not ~40,000 (§8.4 cost characteristics).

pub mod batch;

pub use batch::{BatchProbe, BatchResult, LinkResult};

pub mod prelude {
    pub use super::batch::{BatchProbe, BatchResult, LinkResult};
}
