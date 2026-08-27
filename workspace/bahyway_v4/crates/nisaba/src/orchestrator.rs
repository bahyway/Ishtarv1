//! orchestrator — NISABA as the Architect's Internal Interface.
//!
//! Granted 2026-07-18: NISABA orchestrates the ecosystem's observation
//! surface — StoryEngine's high-priority alerts, the Data Steward's
//! review queue, and LamassuEngine's topological (TDA) readings — into
//! one continuously-updated digest the Architect can read or query.
//!
//! Extended 2026-07-31, per the Architect's own instruction ("Nisaba
//! needs to be in all processes... so it reports to me only what other
//! services and processes... miss or predict or alarm before it becomes
//! a great crash or big risk impact"): `observe_etl_tier_counts` reads
//! `bee_mdm_bus::EtlTierCounts` (the BeeMDM ETL pipeline's own real-time
//! status bus) and raises cross-cutting pipeline-health risks into a
//! SEPARATE, Architect-only `architect_digest` — never mixed into the
//! Data Steward's per-particle `steward_queue`, since these are two
//! genuinely different kinds of finding (one particle's diagnosis vs.
//! the whole pipeline's operational health).
//!
//! CSR-08 (Architect Sovereignty), reaffirmed for this role: "No organ
//! grows without DUB.SAR 𒁾 explicit confirmation." This orchestrator
//! *observes and proposes*; it never executes a write. Concretely:
//!   - `StewardStation::receive` (called here) enqueues a `Diagnosis` for
//!     human review — it is documented as "Zero writes to the database."
//!   - `LamassuEngine::scan_tribe` only reads a sampled point cloud and
//!     classifies what GeoEngine's persistence module already computed.
//!   - Nothing in this module ever mints a Kaki, submits an Event-Kaki
//!     through AdadGate, or otherwise mutates ecosystem state. Corrective
//!     action stays exactly where CSR-08 already puts it: the steward (the
//!     Architect) reviewing the queue and acting through AdadGate.
//!
//! This mirrors the propose-not-ratify shape `ninsun-agent` already uses
//! for the same reason, rather than inventing a new autonomy pattern.
//!
//! ## Bounded self-decision (granted 2026-07-18)
//! With a verified `autonomy::NisabaGrant` installed for this
//! orchestrator's own `NisabaEnvironment`, NISABA may resolve Watch-band
//! findings herself instead of enqueueing them for the Architect — the
//! "small details" tier. Nothing else changes: HighPriority findings
//! (Warning/Critical) always reach the Architect regardless of any grant,
//! and auto-resolution only ever changes which queue a finding lands in —
//! it never mints a Kaki, submits an Event-Kaki, or touches storage. See
//! `autonomy`'s doc comment for why a grant cannot be forged by the
//! client running this code.
use alert_engine::alert::Alert;
use data_steward_station::StewardStation;
use diagnosis_templates::Diagnosis;
use lamassu_engine::{LamassuEngine, TribeReading};
use story_engine::ProjectedState;
use story_sentinel::{StoryBand, StorySentinelFinding};

use crate::autonomy::{AutonomyClass, GrantClaim, NisabaEnvironment, NisabaGrant};
use kupru::akkadian_seal::SealVerifier;

/// One Watch-band finding NISABA resolved herself under an active grant,
/// instead of enqueueing it for the Architect.
#[derive(Debug, Clone)]
pub struct AutonomousResolution {
    pub uuid_hash: u32,
    pub narration: String,
}

/// One line of NISABA's running narration — what she would say if asked
/// "what have you seen." Used to build the daily digest and to answer
/// direct questions in the UrOS hologram panel.
#[derive(Debug, Clone)]
pub struct NisabaNote {
    pub narration: String,
}

/// Severity of an Architect-only pipeline-health finding. Distinct from
/// `story_sentinel::StoryBand`: that's a per-particle diagnosis band fed
/// to the Data Steward's queue; this is about the PIPELINE's OWN
/// operational health (backlogs, malware hits, an overdue sweep) — a
/// different axis entirely, reported only to the Architect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchitectSeverity {
    Advisory,
    HighPriority,
}

/// One cross-cutting operational-risk finding, reported to the Architect
/// only — never mixed into the Data Steward's per-particle queue.
#[derive(Debug, Clone)]
pub struct ArchitectFinding {
    pub severity: ArchitectSeverity,
    pub narration: String,
}

/// NISABA's continuously-updated view of the Ecosystem: alerts raised
/// from StoryEngine projections, the steward's review queue they feed,
/// and the topological (TDA) readings LamassuEngine has taken per Tribe.
///
/// Everything here is additive/observational. There is no method that
/// deletes a particle, mutates a Kaki, or writes to storage.
pub struct NisabaOrchestrator {
    environment: NisabaEnvironment,
    active_grant: Option<GrantClaim>,
    lamassu: LamassuEngine,
    steward: StewardStation,
    tribe_readings: Vec<TribeReading>,
    notes: Vec<NisabaNote>,
    autonomous_resolutions: Vec<AutonomousResolution>,
    architect_digest: Vec<ArchitectFinding>,
    sdb_pending_backlog_threshold: usize,
}

/// Default EnkiSDB pending-backlog threshold for
/// `observe_etl_tier_counts`'s stall-risk finding — a real, but
/// deliberately conservative, starting point; callers with real
/// production sweep-interval/throughput numbers should override via
/// `with_backlog_threshold`.
pub const DEFAULT_SDB_BACKLOG_THRESHOLD: usize = 10_000;

impl NisabaOrchestrator {
    pub fn new(environment: NisabaEnvironment, max_epsilon: f64, r_max: f64, h_max: f64) -> Self {
        NisabaOrchestrator {
            environment,
            active_grant: None,
            lamassu: LamassuEngine::new(max_epsilon, r_max, h_max),
            steward: StewardStation::new(),
            tribe_readings: Vec::new(),
            notes: Vec::new(),
            autonomous_resolutions: Vec::new(),
            architect_digest: Vec::new(),
            sdb_pending_backlog_threshold: DEFAULT_SDB_BACKLOG_THRESHOLD,
        }
    }

    /// Override the EnkiSDB pending-backlog threshold `observe_etl_tier_counts`
    /// flags against — real deployments should size this to their own
    /// sweep interval and expected ingest rate, not the crate default.
    pub fn with_backlog_threshold(mut self, threshold: usize) -> Self {
        self.sdb_pending_backlog_threshold = threshold;
        self
    }

    /// Which environment this orchestrator was constructed for. A grant
    /// only installs if it was issued for this exact environment.
    pub fn environment(&self) -> NisabaEnvironment {
        self.environment
    }

    /// Attempt to install an autonomy grant. Verifies the Architect's
    /// signature against `verifier` and that the grant's environment
    /// matches this orchestrator's own — a grant that fails either check
    /// installs nothing and this returns `false`. There is no other way
    /// to widen what NISABA may do herself.
    pub fn grant_autonomy(&mut self, grant: &NisabaGrant, verifier: &dyn SealVerifier) -> bool {
        if !grant.verify(verifier) {
            return false;
        }
        if grant.claim.environment != self.environment {
            return false;
        }
        self.active_grant = Some(grant.claim);
        true
    }

    /// Revoke any installed grant. Always callable by the Architect;
    /// never needs a signature to narrow autonomy, only to widen it.
    pub fn revoke_autonomy(&mut self) {
        self.active_grant = None;
    }

    pub fn active_autonomy(&self) -> Option<(NisabaEnvironment, AutonomyClass)> {
        self.active_grant.map(|c| (c.environment, c.class))
    }

    fn can_autonomously_resolve(&self, band: StoryBand) -> bool {
        band == StoryBand::Watch
            && matches!(
                self.active_grant,
                Some(GrantClaim {
                    class: AutonomyClass::WatchBandOnly,
                    ..
                })
            )
    }

    /// Scan a batch of StoryEngine projections. A Watch-band finding is
    /// auto-resolved (logged, never enqueued) only when a verified grant
    /// for this environment covers it. Every HighPriority finding always
    /// narrates and enqueues on the StewardStation for the Architect's
    /// review — no grant can change that; `can_autonomously_resolve` only
    /// ever checks the `Watch` band.
    pub fn observe_story_projections(&mut self, projections: &[ProjectedState]) -> usize {
        let mut raised = 0;
        for projected in projections {
            if let Some(StorySentinelFinding {
                alert,
                band,
                narration,
            }) = story_sentinel::scan(projected)
            {
                if self.can_autonomously_resolve(band) {
                    self.notes.push(NisabaNote {
                        narration: format!("[AUTO-RESOLVED · {:?}] {narration}", self.environment),
                    });
                    self.autonomous_resolutions.push(AutonomousResolution {
                        uuid_hash: alert.uuid_hash,
                        narration,
                    });
                } else {
                    self.steward.receive(&alert);
                    self.notes.push(NisabaNote { narration });
                    raised += 1;
                }
            }
        }
        raised
    }

    /// Sample and read one Tribe's topology through LamassuEngine.
    /// `particles` must already be a downsampled landmark subset — see
    /// `bahyway_algebra::persistence`'s O(n^3) caveat.
    pub fn observe_tribe_topology(&mut self, tribe_id: u16, particles: &[([u8; 16], f64)]) {
        let reading = self.lamassu.scan_tribe(tribe_id, particles);
        let narration = format!(
            "Tribe {tribe_id:#06x}: {} signature ({} component(s), {} particle(s) sampled).",
            reading.signature.label(),
            reading.component_count,
            reading.sample_size,
        );
        self.notes.push(NisabaNote { narration });
        self.tribe_readings.push(reading);
    }

    /// Observe the BeeMDM ETL pipeline's own real-time status bus
    /// (`bee_mdm_bus::EtlTierCounts`) and flag cross-cutting operational
    /// risks — malware hits, an overloaded validation-sweep backlog —
    /// into the ARCHITECT-ONLY digest (`architect_digest()`), distinct
    /// from the Data Steward's per-particle diagnosis queue
    /// (`steward_queue()`). This is the Architect's own instruction:
    /// "Nisaba needs to be in all processes... so it reports to me only
    /// what other services and processes... miss or predict or alarm
    /// before it becomes a great crash or big risk impact." Returns how
    /// many new findings were raised.
    pub fn observe_etl_tier_counts(&mut self, counts: &bee_mdm_bus::EtlTierCounts) -> usize {
        let mut raised = 0;

        if counts.sdb_malware_hits > 0 {
            self.architect_digest.push(ArchitectFinding {
                severity: ArchitectSeverity::HighPriority,
                narration: format!(
                    "EnkiSDB: {} malware hit(s) detected — Musarû flagged incoming particles before validation.",
                    counts.sdb_malware_hits
                ),
            });
            raised += 1;
        }

        if counts.qdb_malware_count > 0 {
            self.architect_digest.push(ArchitectFinding {
                severity: ArchitectSeverity::HighPriority,
                narration: format!(
                    "EnkiQDB: {} particle(s) confirmed malware in quarantine — verify Storage Sector routing.",
                    counts.qdb_malware_count
                ),
            });
            raised += 1;
        }

        if counts.sdb_pending >= self.sdb_pending_backlog_threshold {
            self.architect_digest.push(ArchitectFinding {
                severity: ArchitectSeverity::HighPriority,
                narration: format!(
                    "EnkiSDB backlog: {} particle(s) pending (threshold {}), {} tick(s) until the next ValidationSweep — \
                     risk of pipeline stall if arrivals keep outpacing the sweep interval.",
                    counts.sdb_pending, self.sdb_pending_backlog_threshold, counts.ticks_until_sweep
                ),
            });
            raised += 1;
        }

        raised
    }

    /// The Architect-only digest — cross-cutting operational-risk
    /// findings, never mixed into the Data Steward's per-particle queue
    /// (`steward_queue()`). Read-only; cleared only by `clear()`.
    pub fn architect_digest(&self) -> &[ArchitectFinding] {
        &self.architect_digest
    }

    /// The steward's current review queue — read-only, exactly what a
    /// human steward would see, never mutated by this orchestrator beyond
    /// enqueueing new diagnoses via `observe_story_projections`.
    pub fn steward_queue(&self) -> &[Diagnosis] {
        self.steward.queue()
    }

    /// Every topology reading taken so far, most recent last.
    pub fn tribe_readings(&self) -> &[TribeReading] {
        &self.tribe_readings
    }

    /// The full running narration log — "what NISABA has seen."
    pub fn notes(&self) -> &[NisabaNote] {
        &self.notes
    }

    /// Watch-band findings NISABA resolved herself under an active grant,
    /// most recent last — the Architect's audit trail for "small detail"
    /// self-decisions, distinct from the steward queue.
    pub fn autonomous_resolutions(&self) -> &[AutonomousResolution] {
        &self.autonomous_resolutions
    }

    /// A snapshot digest: counts plus the narration log. This is the
    /// buildable, honest interpretation of "daily... without stop" for a
    /// desktop app — a discrete summary composed on demand from whatever
    /// has been observed since the orchestrator was created or last
    /// cleared, not a literal background timer.
    pub fn digest(&self) -> NisabaDigest {
        NisabaDigest {
            steward_queue_len: self.steward.queue_len(),
            tribe_readings_count: self.tribe_readings.len(),
            note_count: self.notes.len(),
            autonomous_resolutions_count: self.autonomous_resolutions.len(),
            architect_findings_count: self.architect_digest.len(),
        }
    }

    /// Clear the steward queue, topology log, and autonomous-resolution
    /// audit trail after the Architect has reviewed them — does not touch
    /// the ecosystem itself, only NISABA's own bookkeeping. Never clears
    /// the active grant — revoking autonomy is a separate, deliberate act.
    pub fn clear(&mut self) {
        self.steward.clear();
        self.tribe_readings.clear();
        self.notes.clear();
        self.autonomous_resolutions.clear();
        self.architect_digest.clear();
    }
}

/// A compact summary of NISABA's current observation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NisabaDigest {
    pub steward_queue_len: usize,
    pub tribe_readings_count: usize,
    pub note_count: usize,
    pub autonomous_resolutions_count: usize,
    pub architect_findings_count: usize,
}

/// Re-exported for callers who only need the Alert type without pulling
/// in alert-engine directly.
pub type StoryAlert = Alert;

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::{ParticleState, TribeId};
    use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};
    use kupru::akkadian_seal::SealKeyPair;

    fn projected(state: ParticleState, events_seen: usize, uuid_hash: u32) -> ProjectedState {
        let minter = KakiMinter::new(TribeId::from_u16(0x0001));
        let identity = minter.mint_identity(uuid_hash, KakiRole::Zikru);
        let identity = IdentityKaki::try_from_kaki(identity).unwrap();
        let mut projected = ProjectedState::new(identity);
        projected.state = state;
        projected.events_seen = events_seen;
        projected
    }

    // fuzzy + orphan (events_seen=0) lands in story-sentinel's Watch band —
    // see story-sentinel's own `orphan_identity_with_fuzzy_state_still_escalates`.
    fn watch_band_projected(uuid_hash: u32) -> ProjectedState {
        projected(ParticleState::Fuzzy, 0, uuid_hash)
    }

    fn new_orch(env: NisabaEnvironment) -> NisabaOrchestrator {
        NisabaOrchestrator::new(env, 1.2, 10.0, 5.0)
    }

    #[test]
    fn new_orchestrator_has_empty_digest() {
        let nisaba = new_orch(NisabaEnvironment::Test);
        let digest = nisaba.digest();
        assert_eq!(digest.steward_queue_len, 0);
        assert_eq!(digest.tribe_readings_count, 0);
        assert_eq!(digest.note_count, 0);
        assert_eq!(digest.autonomous_resolutions_count, 0);
        assert!(nisaba.active_autonomy().is_none());
    }

    #[test]
    fn observing_a_dead_projection_raises_one_alert_and_enqueues_steward() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let projections = vec![projected(ParticleState::Dead, 1, 0xC0FFEE)];
        let raised = nisaba.observe_story_projections(&projections);
        assert_eq!(raised, 1);
        assert_eq!(nisaba.steward_queue().len(), 1);
        assert_eq!(nisaba.notes().len(), 1);
        assert!(nisaba.notes()[0].narration.contains("DEAD"));
    }

    #[test]
    fn observing_a_golden_projection_raises_nothing() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let projections = vec![projected(ParticleState::Golden, 5, 0xC0FFEE)];
        let raised = nisaba.observe_story_projections(&projections);
        assert_eq!(raised, 0);
        assert_eq!(nisaba.steward_queue().len(), 0);
    }

    #[test]
    fn observing_tribe_topology_appends_a_reading_and_note() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        nisaba.observe_tribe_topology(0x0001, &[]);
        assert_eq!(nisaba.tribe_readings().len(), 1);
        assert_eq!(nisaba.notes().len(), 1);
        assert!(nisaba.notes()[0].narration.contains("DEAD"));
    }

    #[test]
    fn clear_resets_everything_but_not_the_grant() {
        let architect_key = SealKeyPair::generate().unwrap();
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let grant = NisabaGrant::issue(
            &architect_key,
            GrantClaim {
                environment: NisabaEnvironment::Test,
                class: AutonomyClass::WatchBandOnly,
                issued_at: 1,
            },
        )
        .unwrap();
        assert!(nisaba.grant_autonomy(&grant, &architect_key.verifier()));

        nisaba.observe_story_projections(&[projected(ParticleState::Dead, 1, 0xC0FFEE)]);
        nisaba.observe_tribe_topology(0x0001, &[]);
        nisaba.clear();
        let digest = nisaba.digest();
        assert_eq!(digest.steward_queue_len, 0);
        assert_eq!(digest.tribe_readings_count, 0);
        assert_eq!(digest.note_count, 0);
        assert_eq!(digest.autonomous_resolutions_count, 0);
        assert!(
            nisaba.active_autonomy().is_some(),
            "clear() must not silently revoke autonomy"
        );
    }

    #[test]
    fn without_a_grant_watch_band_findings_still_go_to_the_steward() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let raised = nisaba.observe_story_projections(&[watch_band_projected(0xC0FFEE)]);
        assert_eq!(raised, 1);
        assert_eq!(nisaba.steward_queue().len(), 1);
        assert_eq!(nisaba.digest().autonomous_resolutions_count, 0);
    }

    #[test]
    fn with_a_valid_grant_watch_band_findings_are_auto_resolved() {
        let architect_key = SealKeyPair::generate().unwrap();
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let grant = NisabaGrant::issue(
            &architect_key,
            GrantClaim {
                environment: NisabaEnvironment::Test,
                class: AutonomyClass::WatchBandOnly,
                issued_at: 1,
            },
        )
        .unwrap();
        assert!(nisaba.grant_autonomy(&grant, &architect_key.verifier()));

        let raised = nisaba.observe_story_projections(&[watch_band_projected(0xC0FFEE)]);
        assert_eq!(
            raised, 0,
            "auto-resolved findings are not raised to the Architect"
        );
        assert_eq!(nisaba.steward_queue().len(), 0);
        assert_eq!(nisaba.digest().autonomous_resolutions_count, 1);
    }

    #[test]
    fn even_with_a_valid_grant_high_priority_findings_always_reach_the_steward() {
        let architect_key = SealKeyPair::generate().unwrap();
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let grant = NisabaGrant::issue(
            &architect_key,
            GrantClaim {
                environment: NisabaEnvironment::Test,
                class: AutonomyClass::WatchBandOnly,
                issued_at: 1,
            },
        )
        .unwrap();
        assert!(nisaba.grant_autonomy(&grant, &architect_key.verifier()));

        // Dead -> HighPriority/Critical, never auto-resolvable regardless of grant.
        let raised =
            nisaba.observe_story_projections(&[projected(ParticleState::Dead, 1, 0xC0FFEE)]);
        assert_eq!(raised, 1);
        assert_eq!(nisaba.steward_queue().len(), 1);
        assert_eq!(nisaba.digest().autonomous_resolutions_count, 0);
    }

    #[test]
    fn a_grant_for_the_wrong_environment_does_not_install() {
        let architect_key = SealKeyPair::generate().unwrap();
        let mut nisaba = new_orch(NisabaEnvironment::Prod);
        let test_grant = NisabaGrant::issue(
            &architect_key,
            GrantClaim {
                environment: NisabaEnvironment::Test,
                class: AutonomyClass::WatchBandOnly,
                issued_at: 1,
            },
        )
        .unwrap();
        assert!(!nisaba.grant_autonomy(&test_grant, &architect_key.verifier()));
        assert!(nisaba.active_autonomy().is_none());

        // Prod stays fully propose-only: even a Watch-band finding still
        // goes to the steward, since no grant is installed.
        let raised = nisaba.observe_story_projections(&[watch_band_projected(0xC0FFEE)]);
        assert_eq!(raised, 1);
    }

    #[test]
    fn a_forged_grant_does_not_install() {
        let architect_key = SealKeyPair::generate().unwrap();
        let impostor_key = SealKeyPair::generate().unwrap();
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let forged = NisabaGrant::issue(
            &impostor_key,
            GrantClaim {
                environment: NisabaEnvironment::Test,
                class: AutonomyClass::WatchBandOnly,
                issued_at: 1,
            },
        )
        .unwrap();
        // Verified against the real Architect key, not the impostor's.
        assert!(!nisaba.grant_autonomy(&forged, &architect_key.verifier()));
        assert!(nisaba.active_autonomy().is_none());
    }

    // ── Architect-only ETL pipeline-health digest ───────────────────────

    fn healthy_etl_counts() -> bee_mdm_bus::EtlTierCounts {
        bee_mdm_bus::EtlTierCounts {
            sdb_pending: 5,
            sdb_promoted: 100,
            sdb_quarantined: 0,
            sdb_malware_hits: 0,
            sdb_tribal_rgb: [0, 0, 0],
            qdb_total: 0,
            qdb_malware_count: 0,
            qdb_tribal_rgb: [0, 0, 0],
            odb_total: 100,
            odb_tribal_rgb: [0, 0, 0],
            dw_total: 0,
            dw_tribal_rgb: [0, 0, 0],
            db_total: 50,
            db_tribal_rgb: [0, 0, 0],
            current_tick: 10,
            sweep_interval: 900,
            ticks_until_sweep: 890,
        }
    }

    #[test]
    fn a_healthy_pipeline_raises_no_architect_findings() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let raised = nisaba.observe_etl_tier_counts(&healthy_etl_counts());
        assert_eq!(raised, 0);
        assert!(nisaba.architect_digest().is_empty());
    }

    #[test]
    fn sdb_malware_hits_raise_a_high_priority_architect_finding() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let mut counts = healthy_etl_counts();
        counts.sdb_malware_hits = 3;
        let raised = nisaba.observe_etl_tier_counts(&counts);
        assert_eq!(raised, 1);
        assert_eq!(nisaba.architect_digest().len(), 1);
        assert_eq!(
            nisaba.architect_digest()[0].severity,
            ArchitectSeverity::HighPriority
        );
        assert!(nisaba.architect_digest()[0].narration.contains("malware"));
    }

    #[test]
    fn qdb_confirmed_malware_raises_a_finding() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let mut counts = healthy_etl_counts();
        counts.qdb_malware_count = 1;
        assert_eq!(nisaba.observe_etl_tier_counts(&counts), 1);
    }

    #[test]
    fn sdb_backlog_past_threshold_raises_a_stall_risk_finding() {
        let mut nisaba = new_orch(NisabaEnvironment::Test).with_backlog_threshold(100);
        let mut counts = healthy_etl_counts();
        counts.sdb_pending = 150;
        let raised = nisaba.observe_etl_tier_counts(&counts);
        assert_eq!(raised, 1);
        assert!(nisaba.architect_digest()[0].narration.contains("backlog"));
    }

    #[test]
    fn architect_digest_is_never_mixed_into_the_steward_queue() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let mut counts = healthy_etl_counts();
        counts.sdb_malware_hits = 1;
        nisaba.observe_etl_tier_counts(&counts);
        assert_eq!(nisaba.architect_digest().len(), 1);
        assert_eq!(
            nisaba.steward_queue().len(),
            0,
            "ETL pipeline findings must never land in the Data Steward's per-particle queue"
        );
    }

    #[test]
    fn clear_also_clears_the_architect_digest() {
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let mut counts = healthy_etl_counts();
        counts.sdb_malware_hits = 1;
        nisaba.observe_etl_tier_counts(&counts);
        assert_eq!(nisaba.digest().architect_findings_count, 1);
        nisaba.clear();
        assert_eq!(nisaba.digest().architect_findings_count, 0);
    }

    #[test]
    fn revoke_autonomy_returns_to_fully_propose_only() {
        let architect_key = SealKeyPair::generate().unwrap();
        let mut nisaba = new_orch(NisabaEnvironment::Test);
        let grant = NisabaGrant::issue(
            &architect_key,
            GrantClaim {
                environment: NisabaEnvironment::Test,
                class: AutonomyClass::WatchBandOnly,
                issued_at: 1,
            },
        )
        .unwrap();
        assert!(nisaba.grant_autonomy(&grant, &architect_key.verifier()));
        nisaba.revoke_autonomy();
        assert!(nisaba.active_autonomy().is_none());

        let raised = nisaba.observe_story_projections(&[watch_band_projected(0xC0FFEE)]);
        assert_eq!(
            raised, 1,
            "after revocation, everything goes back to the steward"
        );
    }
}
