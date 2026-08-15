//! death_legacy.rs — PB-160-Dead_Split1: Particle Death & Legacy Law.
//!
//! Split 1 (Theory Implementation, per the design doc's own Two-Split
//! method): a compilable type skeleton, nothing more. Death is a verdict,
//! not a measurement — a particle already carries `ParticleState::Dead`
//! (see `particle_state.rs`), and this module refines WHAT KIND of dead,
//! purely additively. No existing `ParticleState` match site anywhere in
//! the ~40 dependent crates is touched or needs updating.
//!
//! Split 2 (deferred, out of scope here): live NUZI traversal, live Lamassu
//! before/after phase-space computation, the KISPU-sealed
//! DeadExpired→DeadSealed atomic commit, EnkiDW's dual GoldenRecord/
//! DeathCertificate schema, and the HeptaScript settlement-grammar
//! compiler (WITNESS/PROVE over DeathCertificate). None of that exists
//! here — this module is the rustc-verified type skeleton, matching
//! exactly what Split 1 promised.

use crate::particle_state::ParticleState;

/// Refines `ParticleState::Dead`. Meaningful only when the particle's state
/// is Dead — a Golden or Fuzzy particle has no disposition. Death is a
/// pronouncement (the DataSteward's sovereign ruling, CSR-08), never
/// automatic: TDA/Lamassu only advises (latent H₁ survives → Recoverable;
/// only H₀ dust remains → Foreclosed). The disposition itself is the verdict.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DeathDisposition {
    /// Fallen, UNJUDGED. Recoverable. Lives in EnkiODB, awaits the
    /// DataSteward.
    Expired = 0x01,
    /// Pronounced dead by the DataSteward: sealed, final. Lives in EnkiDW
    /// with its DeathCertificate.
    Sealed = 0x02,
}

impl DeathDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            DeathDisposition::Expired => "DEAD_EXPIRED",
            DeathDisposition::Sealed => "DEAD_SEALED",
        }
    }

    /// Only Expired is recoverable — Sealed is final by law.
    pub fn is_recoverable(self) -> bool {
        matches!(self, DeathDisposition::Expired)
    }
}

impl core::fmt::Display for DeathDisposition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A particle's coarse-grained `ParticleState` plus, when Dead, the finer
/// disposition. Composition point between the existing 3-state law and the
/// Dead_Split1 refinement: callers who only care about Golden/Fuzzy/Dead
/// keep using `ParticleState` directly and are unaffected by this type;
/// callers settling a death read `DeathState`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DeathState {
    pub state: ParticleState,
    /// `Some` only when `state == Dead`. Invariant enforced by `new`.
    pub disposition: Option<DeathDisposition>,
}

impl DeathState {
    /// Construct, enforcing the invariant: a disposition may exist only for
    /// a Dead particle.
    pub fn new(
        state: ParticleState,
        disposition: Option<DeathDisposition>,
    ) -> Result<Self, &'static str> {
        if disposition.is_some() && state != ParticleState::Dead {
            return Err("disposition set on a non-Dead particle");
        }
        Ok(DeathState { state, disposition })
    }

    /// True only once the DataSteward has pronounced (Sealed) — the death
    /// certificate boundary. `Expired` (fallen, unjudged) is not yet a
    /// pronouncement.
    pub fn is_pronounced(&self) -> bool {
        matches!(self.disposition, Some(DeathDisposition::Sealed))
    }
}

/// One of the seven typed residues a pronounced death leaves along a Hepta
/// coupling axis. Order and naming match this ecosystem's own D1..D7 EAV
/// dimension convention.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InheritanceChannel {
    /// D1 — bindings/references to the dead particle's uuid_hash.
    Identity = 1,
    /// D2 — its role/mass in its tribe's collective.
    Belonging = 2,
    /// D3 — epochs/sequences it anchored.
    Temporal = 3,
    /// D4 — proofs/validations that relied on it.
    Integrity = 4,
    /// D5 — templates/baselines it authored (outlive the author).
    QualityEav = 5,
    /// D6 — collective attractor it installed / shapes it anticipated.
    OrbitalPosition = 6,
    /// D7 — colour derivations downstream of it.
    ColourId = 7,
}

impl InheritanceChannel {
    pub const ALL: [InheritanceChannel; 7] = [
        InheritanceChannel::Identity,
        InheritanceChannel::Belonging,
        InheritanceChannel::Temporal,
        InheritanceChannel::Integrity,
        InheritanceChannel::QualityEav,
        InheritanceChannel::OrbitalPosition,
        InheritanceChannel::ColourId,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            InheritanceChannel::Identity => "IDENTITY",
            InheritanceChannel::Belonging => "BELONGING",
            InheritanceChannel::Temporal => "TEMPORAL",
            InheritanceChannel::Integrity => "INTEGRITY",
            InheritanceChannel::QualityEav => "QUALITY_EAV",
            InheritanceChannel::OrbitalPosition => "ORBITAL_POSITION",
            InheritanceChannel::ColourId => "COLOUR_ID",
        }
    }
}

impl core::fmt::Display for InheritanceChannel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How one channel's residue was (or wasn't) settled at pronouncement.
/// `Orphaned { flagged: false }` anywhere is forbidden by law — unaccounted
/// legacy is hidden opacity, influence with no visible source. Callers must
/// escalate an unflagged orphan, never let it stand silently.
#[derive(Clone, Debug, PartialEq)]
pub enum Resolution {
    /// This channel carried no residue for this particle — nothing to
    /// settle.
    NotApplicable,
    /// Residue was reassigned/transferred to a named successor.
    Transferred { to_uuid_hash: u32 },
    /// Residue was archived as-is; no successor, but accounted for.
    Archived,
    /// Load-bearing residue with no settlement yet. `flagged: true` means
    /// it has been surfaced for steward attention; `flagged: false` is the
    /// unlawful state a pronouncement may not leave behind.
    Orphaned { flagged: bool },
}

/// One channel's settlement record, part of a DeathCertificate's legacy.
#[derive(Clone, Debug, PartialEq)]
pub struct LegacyResidue {
    pub channel: InheritanceChannel,
    pub resolution: Resolution,
}

/// The full seven-channel legacy settlement for one pronounced death. Law:
/// no load-bearing channel may be left unresolved-and-unflagged. A
/// pronouncement is a settlement, not a delete.
#[derive(Clone, Debug, PartialEq)]
pub struct LegacySettlement {
    pub residues: [LegacyResidue; 7],
}

impl LegacySettlement {
    /// Build from per-channel resolutions, supplied in `InheritanceChannel::ALL`
    /// order (Identity, Belonging, Temporal, Integrity, QualityEav,
    /// OrbitalPosition, ColourId).
    pub fn new(resolutions: [Resolution; 7]) -> Self {
        let mut iter = resolutions.into_iter();
        let residues = InheritanceChannel::ALL.map(|channel| LegacyResidue {
            channel,
            resolution: iter.next().expect("exactly 7 resolutions, one per channel"),
        });
        LegacySettlement { residues }
    }

    /// Law check: is every channel either settled, or an explicitly flagged
    /// orphan? An unflagged orphan anywhere fails this.
    pub fn is_lawful(&self) -> bool {
        self.residues.iter().all(|r| match &r.resolution {
            Resolution::Orphaned { flagged } => *flagged,
            _ => true,
        })
    }

    /// Channels still carrying an unflagged orphan — the settlement is not
    /// yet lawful until this is empty.
    pub fn unresolved_load_bearing(&self) -> Vec<InheritanceChannel> {
        self.residues
            .iter()
            .filter(|r| matches!(&r.resolution, Resolution::Orphaned { flagged: false }))
            .map(|r| r.channel)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disposition_labels() {
        assert_eq!(DeathDisposition::Expired.as_str(), "DEAD_EXPIRED");
        assert_eq!(DeathDisposition::Sealed.as_str(), "DEAD_SEALED");
        assert!(DeathDisposition::Expired.is_recoverable());
        assert!(!DeathDisposition::Sealed.is_recoverable());
    }

    #[test]
    fn death_state_rejects_disposition_on_live_particle() {
        let err = DeathState::new(ParticleState::Golden, Some(DeathDisposition::Expired));
        assert!(err.is_err());
    }

    #[test]
    fn death_state_allows_no_disposition_on_dead_particle_pre_pronouncement() {
        let ds = DeathState::new(ParticleState::Dead, None).unwrap();
        assert!(!ds.is_pronounced());
    }

    #[test]
    fn death_state_pronounced_only_when_sealed() {
        let expired = DeathState::new(ParticleState::Dead, Some(DeathDisposition::Expired)).unwrap();
        assert!(!expired.is_pronounced());
        let sealed = DeathState::new(ParticleState::Dead, Some(DeathDisposition::Sealed)).unwrap();
        assert!(sealed.is_pronounced());
    }

    #[test]
    fn seven_channels_exactly() {
        assert_eq!(InheritanceChannel::ALL.len(), 7);
    }

    #[test]
    fn settlement_lawful_when_all_settled() {
        let s = LegacySettlement::new([
            Resolution::NotApplicable,
            Resolution::Transferred { to_uuid_hash: 0xDEAD },
            Resolution::Archived,
            Resolution::NotApplicable,
            Resolution::Archived,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
        ]);
        assert!(s.is_lawful());
        assert!(s.unresolved_load_bearing().is_empty());
    }

    #[test]
    fn unflagged_orphan_is_unlawful() {
        let s = LegacySettlement::new([
            Resolution::NotApplicable,
            Resolution::Orphaned { flagged: false },
            Resolution::Archived,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
        ]);
        assert!(!s.is_lawful());
        assert_eq!(s.unresolved_load_bearing(), vec![InheritanceChannel::Belonging]);
    }

    #[test]
    fn flagged_orphan_is_lawful() {
        let s = LegacySettlement::new([
            Resolution::Orphaned { flagged: true },
            Resolution::NotApplicable,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
            Resolution::NotApplicable,
        ]);
        assert!(s.is_lawful());
    }
}
