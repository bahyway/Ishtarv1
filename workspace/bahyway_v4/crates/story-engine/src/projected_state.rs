//! ProjectedState — the result of a StoryEngine projection for one particle.
//!
//! Contains the seven mandatory universal EAV attributes plus any
//! tribe-specific attributes accumulated from the event stream (§4.2).

use std::collections::HashMap;
use bahyway_core::ParticleState;
use enkidb_kaki::IdentityKaki;

/// The projected current state of a particle at a given query time.
/// Computed by the StoryEngine — never stored as a mutable row.
#[derive(Debug, Clone)]
pub struct ProjectedState {
    pub particle:     IdentityKaki,
    /// Events replayed to produce this projection.
    pub events_seen:  usize,
    /// Whether a snapshot was used (true = snapshot-accelerated).
    pub from_snapshot: bool,
    /// The seven mandatory EAV attributes (§4.2) + tribe-specific attrs.
    /// Key = attr_hash (u32), Value = latest active value bytes.
    pub attributes:   HashMap<u32, Vec<u8>>,
    /// Projected particle state (derived from the `state` mandatory EAV attr).
    pub state:        ParticleState,
}

impl ProjectedState {
    pub fn new(particle: IdentityKaki) -> Self {
        ProjectedState {
            particle,
            events_seen:   0,
            from_snapshot: false,
            attributes:    HashMap::new(),
            state:         ParticleState::Fuzzy, // default until first Event-Kaki sets it
        }
    }

    /// Apply one EAV triple onto this projection (SINGLE_ROW latest-wins per §4.6).
    pub fn apply_single_row(&mut self, attr_hash: u32, value: Vec<u8>) {
        self.attributes.insert(attr_hash, value);
    }

    /// Get the current value of an attribute, if present.
    pub fn get(&self, attr_hash: u32) -> Option<&[u8]> {
        self.attributes.get(&attr_hash).map(Vec::as_slice)
    }
}
