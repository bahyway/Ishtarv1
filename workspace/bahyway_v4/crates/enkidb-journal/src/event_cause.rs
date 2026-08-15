//! EventCause — the sovereign reason every JournalEntry was written.
//!
//! One u8 discriminant is stored as an EAV triple (ATTR_EVENT_CAUSE) so that
//! HeptaScript can answer "why did this event happen?" without re-reading
//! gate logs.  The discriminant is stable (never re-ordered — new variants
//! always appended).

/// Every possible cause that generates a JournalEntry in the StoryWay log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventCause {
    // ── Particle lifecycle ─────────────────────────────────────────────────
    KakiBorn                   = 0x01, // KAKI minted; particle enters existence

    // ── Pauli Exclusion Gate outcomes (§7, four-gate sequence) ────────────
    AdadPass                   = 0x10, // ADAD temporal gate — passed
    AdadFail                   = 0x11, // ADAD temporal gate — rejected (duplicate breath)
    AnuPass                    = 0x12, // ANU authority gate — passed
    AnuFail                    = 0x13, // ANU authority gate — rejected (insufficient authority)
    MardukPass                 = 0x14, // MARDUK transform-lock gate — passed
    MardukFail                 = 0x15, // MARDUK transform-lock gate — rejected (lock conflict)
    ShamashPass                = 0x16, // SHAMASH state gate — passed (particle alive)
    ShamashFail                = 0x17, // SHAMASH state gate — rejected (dead particle)

    // ── Orbital assignment ─────────────────────────────────────────────────
    OrbitAssigned              = 0x20, // PA-14 orbit ring computed and recorded

    // ── Musarû security events ─────────────────────────────────────────────
    MusaruMalwareDetected      = 0x30, // pre-extraction ZIP byte scan hit malware signature
    MusaruProximityDegradation = 0x31, // particle color degraded by proximity to infected zone
    MusaruLateQuarantine       = 0x32, // post-extraction late discovery; batch moved to EnkiQDB

    // ── Diagnosis Engine color events ──────────────────────────────────────
    DiagnosisWatch             = 0x40, // color drift first detected (watch level)
    DiagnosisWarning           = 0x41, // drift crossed warning threshold
    DiagnosisCritical          = 0x42, // drift crossed critical threshold — R channel dominant

    // ── Five-tier EnkiDB tier transitions ─────────────────────────────────
    SdbValidationPass          = 0x50, // EnkiSDB sweep passed; particle promoted to EnkiODB
    SdbValidationFail          = 0x51, // EnkiSDB sweep failed; particle sent to EnkiQDB
    EnkidbTransactionCommit    = 0x52, // GUI transaction committed through EnkiDB→EnkiODB
    QuarantineMove             = 0x53, // particle archived in EnkiQDB (permanent)
    ArchiveMove                = 0x54, // particle archived in EnkiDW (read-only cold storage)

    // ── BlackBox Station / Storage Sector / Steward loop-back ──────────────
    BlackBoxRoutedHarmful      = 0x60, // BlackBox scan confirmed harmful; routed to Storage Sector
    BlackBoxRoutedFuzzy        = 0x61, // BlackBox scan inconclusive; routed to EnkiQDB for Steward review
    StorageSectorMove          = 0x62, // particle sealed in the hardware-isolated Storage Sector (terminal)
    StewardResolvedRequeue     = 0x63, // Data Steward cleared a fuzzy particle; requeued into EnkiSDB

    // ── NINSUN advisory / Data Steward review (Stage 2b, §BC-NINSUN-001) ───
    NinsunAdvisoryConfirmed    = 0x64, // Data Steward confirmed a NINSUN_REFINE proposal
    NinsunAdvisoryRejected     = 0x65, // Data Steward rejected a NINSUN_REFINE proposal

    // ── Error Registry / Journal + Unified Attribute Type Registry ─────────
    // (2026-07-24, EnkiMDB internal registries -- see enkidb_journal::
    // error_registry and template_engine::attr_type). An ErrorType is a
    // Particle: it gets a real Identity-Kaki (role=Parzu) on registration;
    // every later firing is a fresh Event-Kaki whose JournalEntry.target_kaki
    // is that same Identity-Kaki -- exactly the existing "N events, one
    // target" shape enkidb-journal already uses elsewhere, not a new
    // mechanism.
    ErrorTypeRegistered        = 0x70, // a new ErrorType Particle born (Registry write)
    ErrorOccurred              = 0x71, // an ErrorOccurrence logged against an existing ErrorType
    ErrorTypeUnknown           = 0x72, // an occurrence referenced an ErrorType that was never registered
    AttrTypeRegistered         = 0x73, // a new canonical AttrType Particle born (Registry write)

    // ── Passport audit trail (2026-07-29) ──────────────────────────────────
    PassportMinted             = 0x74, // a SargonPassport was issued; audit-metadata-only record born

    // ── Shakkanakku run-confirmation registry (2026-07-29) ─────────────────
    ShakkanakkuRunRecorded     = 0x75, // one Shakkanakku corpus run's outcome + operator identity recorded

    // ── KAKI-at-authoring-time (ADR-014, 2026-07-30) ────────────────────────
    // A playbook is a Particle: it gets a real Identity-Kaki (role=Parzu) the
    // moment it's a real, numbered file in playbooks/ -- see
    // enkimdb::pb_emitter::PbEmitter / enkimdb::writenode::WriteNode::ingest_pb.
    PbRegistered               = 0x76, // a new, numbered playbook Particle born (Registry write)
    // A document's supersession is an APPEND on its OWN existing Identity-
    // Kaki (never a new Identity, never a delete) -- see
    // enkiddb::emitter::DocumentEmitter::emit_supersession /
    // enkiddb::writenode::WriteNode::supersede_document. This is the
    // mechanism ADR-014 Decision 2 promised: "why did v4.5 replace v4.4"
    // becomes a queryable hist.reason on this event, not lost history.
    DocumentSuperseded         = 0x77, // a document's prior version was recorded as superseded, with a reason

    // ── Girsu IDE "mark as final" KAKI generator (2026-07-30) ──────────────
    // A .akk/.way/.tmpl tablet mints its own Identity-Kaki (role=Parzu) the
    // moment the Architect marks it final in Girsu -- see
    // enkimdb::tablet::TabletEmitter / enkimdb::writenode::WriteNode::
    // ingest_tablet. One cause covers all three kinds; tablet.kind (an EAV
    // attribute, not a separate cause per language) is what distinguishes
    // them at query time.
    TabletRegistered           = 0x78, // a .akk/.way/.tmpl tablet Particle born (Girsu "mark as final")

    // ── ODB -> EnkiDB Golden Records promotion (2026-07-31) ─────────────────
    // An Active EnkiODB particle was committed into EnkiDb as a real Golden
    // Record via PermanentStore -- the corrected law: ODB never routes
    // straight to EnkiDW (only EnkiDB's own snapshot/partition job feeds
    // EnkiDW); this is the ODB -> EnkiDB(Golden) hop specifically. See
    // enkiodb::odb_store::OdbStore::promote_to_golden.
    OdbPromotedToGolden        = 0x79, // an Active ODB particle was committed to EnkiDb as a Golden Record

    // ── Data Steward's third QDB-review outcome (2026-07-31) ────────────────
    // QuarantineReviewQueue (data-steward-station) already had two of its
    // three real outcomes: resolve_clean -> EnkiSDB (StewardResolvedRequeue,
    // 0x63) and resolve_confirmed_harmful -> Storage Sector
    // (StorageSectorMove, 0x62). This is the third: the Steward judges the
    // quarantined particle clean enough to skip re-validation entirely and
    // land straight in EnkiODB as Active.
    StewardPromotedToOdb       = 0x7A, // Steward judged a quarantined particle clean -> straight to ODB, no re-validation

    // ── Pattern-as-Particle migration (2026-07-31, ADR-016) ─────────────────
    // A discovered pattern (nisaba::discovery::DiscoveredPattern) was minted
    // as a real EnkiMDB particle -- replacing enki-pattern's own separate
    // Hot/Warm/Cold/Crystallized store, per the law that no particle lives
    // outside the 7 Types EnkiDB. See enkimdb::pattern_emitter::PatternEmitter
    // / enkimdb::writenode::WriteNode::ingest_pattern.
    PatternRegistered          = 0x7B, // a discovered Pattern Particle born (EnkiMDB write)

    // ── Multi-location playbook catalog (2026-08-02) ────────────────────────
    // A neutral cross-reference between two already-minted documents --
    // e.g. "found in location X" or "same PB number as Y, unreconciled" --
    // deliberately DISTINCT from DocumentSuperseded (0x77): this cause
    // never implies either side replaces the other. Used by
    // enkiddb::writenode::WriteNode::mint_link_edge, the mechanism
    // shakkanakku::pb_catalog uses to graph playbook sightings across
    // multiple historical/backup locations without silently picking a
    // winner -- that stays the Architect's call.
    DocumentCrossReferenced    = 0x7C, // a neutral link between two documents (location, unreconciled collision)
}

impl EventCause {
    /// Encode to a single EAV value byte (the u8 discriminant).
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Decode from a single EAV value byte.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::KakiBorn),
            0x10 => Some(Self::AdadPass),
            0x11 => Some(Self::AdadFail),
            0x12 => Some(Self::AnuPass),
            0x13 => Some(Self::AnuFail),
            0x14 => Some(Self::MardukPass),
            0x15 => Some(Self::MardukFail),
            0x16 => Some(Self::ShamashPass),
            0x17 => Some(Self::ShamashFail),
            0x20 => Some(Self::OrbitAssigned),
            0x30 => Some(Self::MusaruMalwareDetected),
            0x31 => Some(Self::MusaruProximityDegradation),
            0x32 => Some(Self::MusaruLateQuarantine),
            0x40 => Some(Self::DiagnosisWatch),
            0x41 => Some(Self::DiagnosisWarning),
            0x42 => Some(Self::DiagnosisCritical),
            0x50 => Some(Self::SdbValidationPass),
            0x51 => Some(Self::SdbValidationFail),
            0x52 => Some(Self::EnkidbTransactionCommit),
            0x53 => Some(Self::QuarantineMove),
            0x54 => Some(Self::ArchiveMove),
            0x60 => Some(Self::BlackBoxRoutedHarmful),
            0x61 => Some(Self::BlackBoxRoutedFuzzy),
            0x62 => Some(Self::StorageSectorMove),
            0x63 => Some(Self::StewardResolvedRequeue),
            0x64 => Some(Self::NinsunAdvisoryConfirmed),
            0x65 => Some(Self::NinsunAdvisoryRejected),
            0x70 => Some(Self::ErrorTypeRegistered),
            0x71 => Some(Self::ErrorOccurred),
            0x72 => Some(Self::ErrorTypeUnknown),
            0x73 => Some(Self::AttrTypeRegistered),
            0x74 => Some(Self::PassportMinted),
            0x75 => Some(Self::ShakkanakkuRunRecorded),
            0x76 => Some(Self::PbRegistered),
            0x77 => Some(Self::DocumentSuperseded),
            0x78 => Some(Self::TabletRegistered),
            0x79 => Some(Self::OdbPromotedToGolden),
            0x7A => Some(Self::StewardPromotedToOdb),
            0x7B => Some(Self::PatternRegistered),
            0x7C => Some(Self::DocumentCrossReferenced),
            _    => None,
        }
    }

    /// True if this cause represents a gate failure (particle rejected).
    pub fn is_gate_failure(self) -> bool {
        matches!(
            self,
            Self::AdadFail | Self::AnuFail | Self::MardukFail | Self::ShamashFail
                | Self::SdbValidationFail
        )
    }

    /// True if this cause moves a particle into quarantine (permanent).
    pub fn is_quarantine(self) -> bool {
        matches!(
            self,
            Self::MusaruMalwareDetected | Self::MusaruLateQuarantine | Self::QuarantineMove
                | Self::BlackBoxRoutedHarmful | Self::BlackBoxRoutedFuzzy | Self::StorageSectorMove
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_all_variants() {
        let variants = [
            EventCause::KakiBorn,
            EventCause::AdadPass, EventCause::AdadFail,
            EventCause::AnuPass,  EventCause::AnuFail,
            EventCause::MardukPass, EventCause::MardukFail,
            EventCause::ShamashPass, EventCause::ShamashFail,
            EventCause::OrbitAssigned,
            EventCause::MusaruMalwareDetected,
            EventCause::MusaruProximityDegradation,
            EventCause::MusaruLateQuarantine,
            EventCause::DiagnosisWatch, EventCause::DiagnosisWarning, EventCause::DiagnosisCritical,
            EventCause::SdbValidationPass, EventCause::SdbValidationFail,
            EventCause::EnkidbTransactionCommit,
            EventCause::QuarantineMove,
            EventCause::ArchiveMove,
            EventCause::BlackBoxRoutedHarmful,
            EventCause::BlackBoxRoutedFuzzy,
            EventCause::StorageSectorMove,
            EventCause::StewardResolvedRequeue,
            EventCause::NinsunAdvisoryConfirmed,
            EventCause::NinsunAdvisoryRejected,
            EventCause::ErrorTypeRegistered,
            EventCause::ErrorOccurred,
            EventCause::ErrorTypeUnknown,
            EventCause::AttrTypeRegistered,
            EventCause::PassportMinted,
            EventCause::ShakkanakkuRunRecorded,
            EventCause::PbRegistered,
            EventCause::DocumentSuperseded,
            EventCause::TabletRegistered,
            EventCause::OdbPromotedToGolden,
            EventCause::StewardPromotedToOdb,
            EventCause::PatternRegistered,
            EventCause::DocumentCrossReferenced,
        ];
        for v in variants {
            assert_eq!(EventCause::from_byte(v.to_byte()), Some(v));
        }
    }

    #[test]
    fn unknown_byte_returns_none() {
        assert_eq!(EventCause::from_byte(0xFF), None);
        assert_eq!(EventCause::from_byte(0x00), None);
    }

    #[test]
    fn gate_failure_flags() {
        assert!(EventCause::AdadFail.is_gate_failure());
        assert!(EventCause::SdbValidationFail.is_gate_failure());
        assert!(!EventCause::AdadPass.is_gate_failure());
    }

    #[test]
    fn quarantine_flags() {
        assert!(EventCause::MusaruMalwareDetected.is_quarantine());
        assert!(EventCause::MusaruLateQuarantine.is_quarantine());
        assert!(EventCause::QuarantineMove.is_quarantine());
        assert!(!EventCause::ArchiveMove.is_quarantine());
    }
}
