//! WriteNode — EnkiMDB's write path: real, append-only WAL.
//!
//! Identical pattern to `enkiddb::writenode::WriteNode` (which itself
//! mirrors EnkiDB's own port-7001 write node) — deliberately, since both
//! are the same "particle producer + real Journal" shape. A real
//! `enkidb_journal::Journal` is the WAL; every ingested artifact profile
//! is durable in it before `ingest_artifact` returns.

use enkidb_ingest::bridge::particle_to_eav_triple;
use enkidb_journal::entry::JournalEntry;
use enkidb_journal::{ErrorOccurrenceSpec, ErrorTypeSpec, EventCause, Journal};
use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
use template_engine::AttrTypeSpec;

use crate::artifact::ArtifactProfile;
use crate::emitter::ArtifactEmitter;
use crate::passport_record::PassportRecordSpec;
use crate::pattern::PatternProfile;
use crate::pattern_emitter::PatternEmitter;
use crate::pb::PbProfile;
use crate::pb_emitter::PbEmitter;
use crate::registry_emitter::RegistryEmitter;
use crate::run_record::AnuGovernorRunRecordSpec;
use crate::tablet::TabletProfile;
use crate::tablet_emitter::TabletEmitter;

/// EnkiMDB's write node: mints artifact Identity-Kakis, journals their
/// particles as one EAV-bearing Event-Kaki per artifact. The Journal held
/// here is the real WAL.
pub struct WriteNode {
    journal: Journal,
    minter: KakiMinter,
}

impl WriteNode {
    pub fn new(minter: KakiMinter, shard_count: u16) -> Self {
        WriteNode {
            journal: Journal::new(shard_count),
            minter,
        }
    }

    /// Ingest one artifact profile: mint its Identity-Kaki, emit its
    /// particles, journal them as a single Event-Kaki-bearing entry at
    /// `epoch`. Returns the artifact's Identity-Kaki.
    pub fn ingest_artifact(&mut self, profile: &ArtifactProfile, epoch: u32) -> IdentityKaki {
        let emitter = ArtifactEmitter::new(&self.minter);
        let (artifact_kaki, particles) = emitter.emit(profile);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(JournalEntry::new(event_kaki, artifact_kaki, epoch, eav))
            .expect("append to an in-memory Journal is infallible in this shard config");

        artifact_kaki
    }

    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    pub fn artifact_count(&self) -> usize {
        self.journal.all_entries().count()
    }

    // ── PB registry (ADR-014, 2026-07-30) ───────────────────────────────
    //
    // "Mint at authoring time" for playbooks: a numbered PB file is a
    // Particle the moment it's real (committed to playbooks/), not
    // deferred until it first runs. Same "particle producer + real
    // Journal" shape as ingest_artifact -- PbEmitter takes PbEmitter's
    // place, EventCause::PbRegistered marks why.

    /// Mint a real, numbered playbook's Identity-Kaki (role=Parzu),
    /// journal its `pb.*` particles under `EventCause::PbRegistered`.
    /// Callers (AnuGovernor's corpus scan) are responsible for calling
    /// this only for a PB that doesn't already have a minted Identity —
    /// this method itself always mints a fresh one, it does not check.
    pub fn ingest_pb(&mut self, profile: &PbProfile, epoch: u32) -> IdentityKaki {
        let emitter = PbEmitter::new(&self.minter);
        let (pb_kaki, particles) = emitter.emit(profile);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Parzu))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, pb_kaki, epoch, eav)
                    .with_event_cause(EventCause::PbRegistered),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        pb_kaki
    }

    // ── Pattern-as-Particle (2026-07-31, ADR-016) ───────────────────────
    //
    // Migrates enki-pattern's own separate Hot/Warm/Cold/Crystallized
    // store into EnkiMDB, per the law that no particle lives outside the
    // 7 Types EnkiDB. Same "particle producer + real Journal" shape as
    // ingest_pb -- PatternEmitter takes PbEmitter's place,
    // EventCause::PatternRegistered marks why.

    /// Mint a real Identity-Kaki (role=Parzu) for a discovered pattern,
    /// journal its `pattern.*` particles (including the pattern's own
    /// deterministic Pattern-KAKI, hex-encoded, and its age-derived
    /// storage tier) under `EventCause::PatternRegistered`.
    pub fn ingest_pattern(&mut self, profile: &PatternProfile, epoch: u32) -> IdentityKaki {
        let emitter = PatternEmitter::new(&self.minter);
        let (pattern_kaki, particles) = emitter.emit(profile);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Parzu))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, pattern_kaki, epoch, eav)
                    .with_event_cause(EventCause::PatternRegistered),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        pattern_kaki
    }

    // ── Girsu IDE "mark as final" KAKI generator (2026-07-30) ───────────
    //
    // The Architect marks a .akk/.way/.tmpl tablet final in Girsu; that
    // moment mints its Identity-Kaki. Same "particle producer + real
    // Journal" shape as ingest_pb -- TabletEmitter takes PbEmitter's
    // place, EventCause::TabletRegistered marks why. tablet.kind (an EAV
    // attribute) distinguishes akk/way/tmpl at query time, not a
    // separate EventCause per language.

    /// Mint a real tablet's Identity-Kaki (role=Parzu), journal its
    /// `<kind>.*` particles under `EventCause::TabletRegistered`. Callers
    /// (Girsu's "mark as final" trigger) are responsible for calling this
    /// only for a tablet that doesn't already have a minted Identity --
    /// this method itself always mints a fresh one, it does not check.
    pub fn ingest_tablet(&mut self, profile: &TabletProfile, epoch: u32) -> IdentityKaki {
        let emitter = TabletEmitter::new(&self.minter);
        let (tablet_kaki, particles) = emitter.emit(profile);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Parzu))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, tablet_kaki, epoch, eav)
                    .with_event_cause(EventCause::TabletRegistered),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        tablet_kaki
    }

    // ── Error Registry / Journal (2026-07-24) ──────────────────────────
    //
    // Same "particle producer + real Journal" shape as `ingest_artifact`
    // above. The Registry write (a new ErrorType) mints a fresh
    // Identity-Kaki, exactly like an artifact. The Journal write (an
    // occurrence) deliberately does NOT mint a fresh identity -- it
    // targets the ErrorType's existing one, the same "N events, one
    // target" shape `Journal::read_particle_history` already supports.

    /// Register a new ErrorType — mints its Identity-Kaki (role=Parzu),
    /// journals it under `EventCause::ErrorTypeRegistered`. Returns the
    /// Identity-Kaki callers must keep (or look up again by scanning
    /// `error_type.code`) to log occurrences against it.
    pub fn ingest_error_type(&mut self, spec: &ErrorTypeSpec, epoch: u32) -> IdentityKaki {
        let emitter = RegistryEmitter::new(&self.minter);
        let (type_kaki, particles) = emitter.emit_error_type(spec);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Parzu))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, type_kaki, epoch, eav)
                    .with_event_cause(EventCause::ErrorTypeRegistered),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        type_kaki
    }

    /// Log one firing of an already-registered ErrorType. `type_kaki`
    /// must be the Identity-Kaki `ingest_error_type` returned when that
    /// ErrorType was registered — never minted fresh here.
    pub fn log_error_occurrence(
        &mut self,
        type_kaki: IdentityKaki,
        occurrence: &ErrorOccurrenceSpec,
        epoch: u32,
    ) {
        let emitter = RegistryEmitter::new(&self.minter);
        let particles = emitter.emit_error_occurrence(type_kaki, occurrence);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, type_kaki, epoch, eav)
                    .with_event_cause(EventCause::ErrorOccurred),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");
    }

    // ── Unified Attribute Type Registry (2026-07-24) ───────────────────

    /// Register a new canonical AttrType — mints its Identity-Kaki
    /// (role=Parzu), journals it under `EventCause::AttrTypeRegistered`.
    pub fn ingest_attr_type(&mut self, spec: &AttrTypeSpec, epoch: u32) -> IdentityKaki {
        let emitter = RegistryEmitter::new(&self.minter);
        let (type_kaki, particles) = emitter.emit_attr_type(spec);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Parzu))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, type_kaki, epoch, eav)
                    .with_event_cause(EventCause::AttrTypeRegistered),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        type_kaki
    }

    // ── Passport audit trail (2026-07-29) ──────────────────────────────
    //
    // "Every minted Passport is also a new Particle... saved in EnkiMDB
    // in Passport_schema" (Architect's own instruction). Callers mint
    // this only for a passport that has ALREADY been issued through the
    // real Gilgamesh/Sargon minting path and already passed
    // `verify_seal()` -- this method does not itself authorize minting a
    // passport, it only records that one was minted. Read access to
    // `passport.*` is a query-time authorization concern (which realm/
    // privilege_level may run a HeptaScript QUERY against this
    // namespace), not a write-time one -- tracked separately, not
    // silently assumed solved here.

    /// Ingest one passport's audit record — mints its own Identity-Kaki
    /// (role=Zikru), journals it under `EventCause::PassportMinted`.
    /// `spec` carries only non-secret metadata (see
    /// `PassportRecordSpec`'s own doc comment) — never the passport's
    /// seal or any key material.
    pub fn ingest_passport_record(
        &mut self,
        spec: &PassportRecordSpec,
        epoch: u32,
    ) -> IdentityKaki {
        let emitter = RegistryEmitter::new(&self.minter);
        let (record_kaki, particles) = emitter.emit_passport_record(spec);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, record_kaki, epoch, eav)
                    .with_event_cause(EventCause::PassportMinted),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        record_kaki
    }

    // ── AnuGovernor run-confirmation registry (2026-07-29) ─────────────
    //
    // Daily-workflow request: "run AnuGovernor, get report, debug,
    // re-run" was missing a queryable confirmation registry that also
    // records who ran it and at what privilege level, specifically so an
    // authority block is distinguishable from a genuine technical
    // failure. Callers build `spec` with `operator_*` fields all `None`
    // when the run was unauthenticated (`vault_check_enabled=false`) --
    // never a fabricated identity.

    /// Ingest one AnuGovernor run's confirmation record — mints its own
    /// Identity-Kaki (role=Zikru), journals it under
    /// `EventCause::AnuGovernorRunRecorded`.
    pub fn ingest_anu_governor_run_record(
        &mut self,
        spec: &AnuGovernorRunRecordSpec,
        epoch: u32,
    ) -> IdentityKaki {
        let emitter = RegistryEmitter::new(&self.minter);
        let (record_kaki, particles) = emitter.emit_anu_governor_run(spec);

        let eav = particles.iter().map(particle_to_eav_triple).collect();
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(KakiRole::Zikru))
            .expect("KakiMinter::event always mints kaki_type=Event");

        self.journal
            .append(
                JournalEntry::new(event_kaki, record_kaki, epoch, eav)
                    .with_event_cause(EventCause::AnuGovernorRunRecorded),
            )
            .expect("append to an in-memory Journal is infallible in this shard config");

        record_kaki
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactKind;
    use bahyway_core::TribeId;

    fn write_node() -> WriteNode {
        WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF02)), 64)
    }

    fn profile(name: &str) -> ArtifactProfile {
        ArtifactProfile {
            name: name.to_string(),
            kind: ArtifactKind::Crate,
            path: format!("crates/{name}"),
            version: Some("4.0.2".to_string()),
        }
    }

    #[test]
    fn ingest_artifact_journals_a_real_entry() {
        let mut wn = write_node();
        let kaki = wn.ingest_artifact(&profile("enkiddb"), 1);

        assert_eq!(wn.artifact_count(), 1);
        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history.len(), 1);
        assert!(!history[0].eav.is_empty());
    }

    #[test]
    fn ingesting_two_artifacts_mints_two_distinct_identities() {
        let mut wn = write_node();
        let a = wn.ingest_artifact(&profile("enkiddb"), 1);
        let b = wn.ingest_artifact(&profile("enkimdb"), 2);

        assert_ne!(a.bytes(), b.bytes());
        assert_eq!(wn.artifact_count(), 2);
    }

    #[test]
    fn eav_triples_survive_the_particle_to_journal_bridge() {
        let mut wn = write_node();
        let kaki = wn.ingest_artifact(&profile("enkiddb"), 5);

        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history[0].epoch, 5);
        // name + kind + path + version = 4 triples
        assert_eq!(history[0].eav.len(), 4);
    }

    // ── Pattern-as-Particle ──────────────────────────────────────────────

    fn discovered_pattern() -> nisaba::discovery::DiscoveredPattern {
        use enkidb_kaki::{FixedCoord7D, PatternType};
        use nisaba::cluster::GaCluster;
        use nisaba::discovery::NisabaDiscovery;
        let cluster = GaCluster::new(
            FixedCoord7D {
                d: [1_000, 2_000, 500, 1, 2, 3, 0],
            },
            vec![],
            0.9,
            0.9,
            42,
        );
        NisabaDiscovery::discover(&cluster, PatternType::CrowdFlow).unwrap()
    }

    #[test]
    fn ingest_pattern_journals_a_real_entry_under_the_correct_cause() {
        let mut wn = write_node();
        let profile = PatternProfile::new(discovered_pattern(), 0);
        let kaki = wn.ingest_pattern(&profile, 1);

        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history.len(), 1);
        assert!(!history[0].eav.is_empty());
        assert_eq!(
            history[0].event_cause(),
            Some(EventCause::PatternRegistered)
        );
    }

    #[test]
    fn ingesting_two_patterns_mints_two_distinct_identities() {
        let mut wn = write_node();
        let a = wn.ingest_pattern(&PatternProfile::new(discovered_pattern(), 0), 1);
        let b = wn.ingest_pattern(&PatternProfile::new(discovered_pattern(), 0), 2);
        assert_ne!(a.bytes(), b.bytes());
    }

    // ── Passport audit trail ────────────────────────────────────────────

    fn real_passport(privilege_level: u8, realm: &str) -> kupru::SargonPassport {
        let keypair = kupru::SealKeyPair::generate().unwrap();
        let naru = kupru::NaruLayer {
            subject_kaki: [7u8; 16],
            akkadian_name: kupru::AkkadianName::dubsar(),
            linguistic_proof: kupru::LinguisticProof::create("test-linguistic-phrase").unwrap(),
            realm: realm.into(),
            mudu_score: 5,
        };
        let istar = if privilege_level >= 7 {
            kupru::IshtarLayer::architect(realm)
        } else {
            kupru::IshtarLayer::gardener(realm)
        };
        kupru::SargonPassport::issue(naru, istar, [1u8; 16], &keypair, &[0x42u8; 32]).unwrap()
    }

    #[test]
    fn ingest_passport_record_journals_a_real_entry_under_passport_minted() {
        let mut wn = write_node();
        let passport = real_passport(1, "bahyway");
        let spec = PassportRecordSpec::from_passport(&passport);

        let kaki = wn.ingest_passport_record(&spec, 1);

        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].event_cause(), Some(EventCause::PassportMinted));
        // 8 passport.* triples + 1 event_cause triple `with_event_cause` appends.
        assert_eq!(history[0].eav.len(), 9);
    }

    #[test]
    fn passport_record_kaki_is_zikru_role() {
        let mut wn = write_node();
        let spec = PassportRecordSpec::from_passport(&real_passport(7, "bahyway"));
        let kaki = wn.ingest_passport_record(&spec, 1);
        assert_eq!(kaki.role(), KakiRole::Zikru);
    }

    // ── AnuGovernor run-confirmation registry ───────────────────────────

    fn sample_run_spec() -> AnuGovernorRunRecordSpec {
        AnuGovernorRunRecordSpec {
            run_id: "20260729T120000Z".to_string(),
            started_at: 1_000,
            finished_at: 1_060,
            playbook_count: 3,
            ok_count: 3,
            warned_count: 0,
            failed_count: 0,
            skipped_count: 0,
            outcome: "Completed".to_string(),
            blocked_reason: None,
            operator_subject_kaki_hex: Some("cc".repeat(16)),
            operator_realm: Some("bahyway".to_string()),
            operator_privilege_level: Some(7),
            os_username: Some("architect".to_string()),
            os_groups_csv: Some("architect,wheel,bahyway-architect".to_string()),
            os_bahyway_role: Some("bahyway-architect".to_string()),
            report_path: "docs/SHEDU/anu_governor_reports/report_20260729T120000Z.md".to_string(),
        }
    }

    #[test]
    fn ingest_anu_governor_run_record_journals_a_real_entry_under_run_recorded() {
        let mut wn = write_node();
        let kaki = wn.ingest_anu_governor_run_record(&sample_run_spec(), 1);

        let history = wn.journal().read_particle_history(&kaki);
        assert_eq!(history.len(), 1);
        assert_eq!(
            history[0].event_cause(),
            Some(EventCause::AnuGovernorRunRecorded)
        );
        // 10 base fields + 3 operator_* + 3 os_* + 1 event_cause triple = 17.
        assert_eq!(history[0].eav.len(), 17);
    }

    #[test]
    fn run_record_kaki_is_zikru_role() {
        let mut wn = write_node();
        let kaki = wn.ingest_anu_governor_run_record(&sample_run_spec(), 1);
        assert_eq!(kaki.role(), KakiRole::Zikru);
    }
}
