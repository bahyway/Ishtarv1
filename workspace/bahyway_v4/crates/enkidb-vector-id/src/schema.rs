//! VectorIdKind — the schema classification for operational mechanisms.

/// Classifies which Default schema a VectorId belongs to.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VectorIdKind {
    /// Default.Jobs — snapshot jobs, schedulers, cron-like mechanisms (§3.6, §5.5).
    Job,
    /// Default.Indexes — the seven native index definitions (§9.3).
    Index,
    /// Default.Services — internal services (Musarû, VGCA, Score Engine, etc.).
    Service,
}

impl core::fmt::Display for VectorIdKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            VectorIdKind::Job     => "Default.Jobs",
            VectorIdKind::Index   => "Default.Indexes",
            VectorIdKind::Service => "Default.Services",
        };
        f.write_str(s)
    }
}

/// A VectorId with its schema classification and a human-readable name.
#[derive(Clone, Debug)]
pub struct VectorEntry {
    pub id:   super::VectorId,
    pub kind: VectorIdKind,
    pub name: String,
}

impl VectorEntry {
    pub fn new(kind: VectorIdKind, name: impl Into<String>) -> Self {
        VectorEntry { id: super::VectorId::generate(), kind, name: name.into() }
    }

    /// Create the well-known NeverSnapshot job entry.
    pub fn never_snapshot_job() -> Self {
        VectorEntry {
            id:   super::VectorId::NEVER_SNAPSHOT,
            kind: VectorIdKind::Job,
            name: "NeverSnapshot".to_string(),
        }
    }
}
