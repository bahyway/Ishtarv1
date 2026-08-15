//! idu-prober — Shamash IDU Probe: CrossTribe-Kaki effective state (§8.3)
//!
//! The IDU Probing Rule:
//!   1. Read the persistent tribes_id EAV attribute to identify anchor particles.
//!   2. For each anchor, ask the StoryEngine to project its current state.
//!   3. Compose the link's effective state: weakest anchor wins.
//!
//! Properties (§8.3):
//!   - Zero writes during probe — CrossTribe-Kaki is NEVER modified.
//!   - Honors strict CQRS — anchor states obtained via StoryEngine projection.
//!   - Time-travel-safe — project_at(T) instead of project().
//!   - No stored cross-tribe state (§10, forbidden op #8).

pub mod probe;
pub mod crosstribe;

pub use probe::{IduProbe, IduProbeResult};
pub use crosstribe::CrossTribeLinkState;

pub mod prelude {
    pub use super::probe::{IduProbe, IduProbeResult};
    pub use super::crosstribe::CrossTribeLinkState;
}
