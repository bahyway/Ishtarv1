#![forbid(unsafe_code)]
//! HeptaMap E7 rescue grid — zone classification and multi-team coordination.
//!
//! BC-QUAKE-001 §8: each HeptaMap cell receives a sovereign zone class
//! derived from its SMI score, governing rescue team entry protocol.

use crate::smi::SmiState;

/// Zone classification for a HeptaMap E7 cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneClass {
    /// GREEN — SMI < 0.25 — full team entry, structural sensors active.
    Green,
    /// AMBER — SMI 0.25–0.60 — 2-person max, 20-min timed rotations.
    Amber,
    /// RED — SMI 0.60–0.85 — robot and acoustic only, no human entry.
    Red,
    /// BLACK — SMI > 0.85 — immediate evacuation, PUZRU broadcast.
    Black,
}

impl ZoneClass {
    /// Derive zone class from SMI EAV state.
    pub fn from_smi_state(state: SmiState) -> Self {
        match state {
            SmiState::Golden => Self::Green,
            SmiState::Fuzzy => Self::Amber,
            SmiState::Dead => Self::Red,
            SmiState::DeadCritical => Self::Black,
        }
    }
}

/// A single rescue zone — one HeptaMap cell with its current classification.
#[derive(Debug, Clone)]
pub struct RescueZone {
    /// HeptaMap E7 cell identifier.
    pub hepta_cell: u16,
    /// Current zone classification.
    pub class: ZoneClass,
    /// Rescue team tribe_id currently assigned to this zone (0 = unassigned).
    pub assigned_team: u16,
    /// SMI score at last classification update.
    pub smi_score: f32,
}

impl RescueZone {
    pub fn new(hepta_cell: u16, smi_score: f32, state: SmiState) -> Self {
        Self {
            hepta_cell,
            class: ZoneClass::from_smi_state(state),
            assigned_team: 0,
            smi_score,
        }
    }

    /// Returns true if human entry is permitted under sovereign rescue protocol.
    pub fn human_entry_permitted(&self) -> bool {
        matches!(self.class, ZoneClass::Green | ZoneClass::Amber)
    }

    /// Returns true if an evacuation PUZRU broadcast must be triggered.
    pub fn requires_puzru(&self) -> bool {
        self.class == ZoneClass::Black
    }
}
