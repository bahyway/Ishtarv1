//! Alert — a flagged particle with drift diagnosis for the Data Steward Station.

/// Severity of the drift pattern detected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertSeverity {
    /// Mild drift — monitor; no immediate action required.
    Watch,
    /// Significant drift — steward review recommended.
    Warning,
    /// Critical drift — immediate steward intervention required.
    Critical,
}

/// Likely cause of the drift pattern, as diagnosed by a Diagnosis Template (§4.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DriftCause {
    /// Data-entry or manufacturing error.
    Defect,
    /// Deliberate distortion — anomalous drift trajectory.
    Fraud,
    /// Bad data propagated from a corrupted source particle.
    Infection,
    /// Natural aging — freshness decay without data problems.
    FreshnessDecay,
    /// Unknown cause — steward should investigate.
    Unknown,
}

/// A flagged particle emitted by the AlertEngine to the Data Steward Station.
#[derive(Clone, Debug)]
pub struct Alert {
    /// uuid_hash of the flagged particle.
    pub uuid_hash:  u32,
    pub severity:   AlertSeverity,
    pub cause:      DriftCause,
    /// Measured drift distance from the particle's last healthy color state.
    pub drift_distance: f32,
}

impl Alert {
    pub fn new(uuid_hash: u32, severity: AlertSeverity, cause: DriftCause, drift_distance: f32) -> Self {
        Alert { uuid_hash, severity, cause, drift_distance }
    }
}
