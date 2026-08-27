//! enkidb-vector-id — Vector IDs for operational mechanisms (§5.5)
//!
//! Operational mechanisms (schedulers, snapshot jobs, indexes) are NOT particles
//! and do NOT earn KAKIs.  They are identified by Vector IDs stored in:
//!   Default.Jobs    — snapshot jobs, cron schedulers, system services
//!   Default.Indexes — the seven native index definitions
//!
//! Vector IDs are self-generated at creation, never recycled, never mutated.

pub mod schema;
pub mod vector_id;

pub use schema::VectorIdKind;
pub use vector_id::VectorId;

pub mod prelude {
    pub use super::schema::VectorIdKind;
    pub use super::vector_id::VectorId;
}
