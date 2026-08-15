#![forbid(unsafe_code)]
//! ESARHADDON KAKI particle class definitions.
//!
//! Three sovereign particle classes as specified in BC-QUAKE-001 §4.

/// KAKI particle class for earthquake rescue data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KakiClass {
    /// 0x01 / KISHIB — Survivor Identity particle.
    SurvivorIdentity = 0x01,
    /// 0x02 / ZIKRU — Seismic Event particle.
    SeismicEvent     = 0x02,
    /// 0x02 / PARZU — Structural State particle (SMI snapshot).
    StructuralState  = 0x03,
}

/// Seismic event particle — accelerometer / geophone reading.
#[derive(Debug, Clone)]
pub struct SeismicEvent {
    /// Peak ground acceleration in g.
    pub pga_g: f32,
    /// P-wave arrival epoch (seconds since earthquake T+0).
    pub p_wave_epoch_s: u16,
    /// S-wave arrival epoch (seconds since earthquake T+0).
    pub s_wave_epoch_s: u16,
    /// Estimated aftershock magnitude (Richter).
    pub magnitude: f32,
    /// Focal depth in km.
    pub depth_km: f32,
    /// HeptaMap E7 cell reference.
    pub hepta_cell: u16,
}

/// Survivor signal particle — fused identity from sensor aggregator.
#[derive(Debug, Clone)]
pub struct SurvivorSignal {
    /// Bayesian fused confidence percentage (0–100).
    pub confidence_pct: u8,
    /// HeptaMap E7 cell where survivor signal was detected.
    pub hepta_cell: u16,
    /// Estimated depth below rubble surface in metres.
    pub depth_m: f32,
    /// Detection epoch (seconds since earthquake T+0).
    pub epoch_s: u16,
    /// Signal sources contributing to this fused particle.
    pub source_count: u8,
}

/// Structural state particle — SMI snapshot per HeptaMap cell.
#[derive(Debug, Clone)]
pub struct StructuralState {
    /// HeptaMap E7 cell this snapshot covers.
    pub hepta_cell: u16,
    /// Structural Mortality Index score (0.0–1.0).
    pub smi: f32,
    /// Snapshot epoch (seconds since earthquake T+0).
    pub epoch_s: u16,
    /// ColourID B11 hue (0 = GOLDEN safe, 240 = DEAD critical).
    pub colour_id: u8,
}
