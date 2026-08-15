//! story-engine — StoryEngine: CQRS read-side projection (§3.3, §3.6)
//!
//! The StoryEngine projects a particle's current state by replaying its
//! Event-Kaki history from the Journal.  When snapshots exist it starts
//! from the most recent snapshot and applies only the delta events.
//!
//! This is the ONLY path through which "current state" is obtained.
//! No state is ever stored as a mutable value — projections are computed
//! on demand from the immutable Journal.

pub mod projection;
pub mod projected_state;
pub mod story_engine;

pub use story_engine::StoryEngine;
pub use projected_state::ProjectedState;

pub mod prelude {
    pub use super::story_engine::StoryEngine;
    pub use super::projected_state::ProjectedState;
}
