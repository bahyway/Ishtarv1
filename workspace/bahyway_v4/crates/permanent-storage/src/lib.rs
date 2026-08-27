//! permanent-storage — append-only Golden Record commit station (§10.7).

pub mod store;
pub use store::{CommitStats, PermanentStore};
