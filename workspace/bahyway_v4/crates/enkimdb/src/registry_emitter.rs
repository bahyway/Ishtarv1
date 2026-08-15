//! RegistryEmitter — mints one Identity-Kaki per Error Registry / Unified
//! Attribute Type Registry entry and emits it as real
//! `enkidb_particles::Particle` EAV rows, namespaced under
//! `error_type.*` / `attr_type.*`. Same shape as `ArtifactEmitter`
//! (`artifact.*`) — a second namespace on the same EnkiMDB store, not a
//! new mechanism.

use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_journal::{ErrorTypeSpec, ErrorOccurrenceSpec};
use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};
use enkidb_particles::Particle;
use template_engine::AttrTypeSpec;

use crate::passport_record::PassportRecordSpec;
use crate::run_record::AnuGovernorRunRecordSpec;

pub struct RegistryEmitter<'a> {
    minter: &'a KakiMinter,
}

impl<'a> RegistryEmitter<'a> {
    pub fn new(minter: &'a KakiMinter) -> Self {
        Self { minter }
    }

    /// Mint an Identity-Kaki (role=Parzu — "logic, template, axiom, or
    /// rule") for a new ErrorType registration and emit its EAV rows.
    pub fn emit_error_type(&self, spec: &ErrorTypeSpec) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Parzu))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();

        let particles = vec![
            Particle::base(kaki, "error_type.code", AkkValue::Text(spec.code.to_string()), now),
            Particle::base(kaki, "error_type.summary", AkkValue::Text(spec.summary.to_string()), now),
            Particle::base(kaki, "error_type.severity", AkkValue::Int(spec.severity.to_byte() as i64), now),
            Particle::base(kaki, "error_type.defined_by", AkkValue::Text(spec.defined_by.to_string()), now),
        ];

        (kaki, particles)
    }

    /// Emit the EAV rows for one ErrorOccurrence firing. Does NOT mint an
    /// Identity-Kaki — an occurrence targets an already-registered
    /// ErrorType's existing identity (the caller supplies it as
    /// `JournalEntry::target_kaki`); this only builds the occurrence's
    /// own attribute rows.
    pub fn emit_error_occurrence(&self, error_type_kaki: IdentityKaki, occurrence: &ErrorOccurrenceSpec) -> Vec<Particle> {
        let now = now_secs();
        vec![
            Particle::base(error_type_kaki, "error_occurrence.source", AkkValue::Text(occurrence.source.to_string()), now),
            Particle::base(error_type_kaki, "error_occurrence.detail", AkkValue::Text(occurrence.detail.clone()), now),
        ]
    }

    /// Mint an Identity-Kaki (role=Parzu) for a new AttrType registration
    /// and emit its EAV rows.
    pub fn emit_attr_type(&self, spec: &AttrTypeSpec) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Parzu))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();

        let particles = vec![
            Particle::base(kaki, "attr_type.name", AkkValue::Text(spec.name.to_string()), now),
            Particle::base(kaki, "attr_type.representation", AkkValue::Text(spec.attr_type.as_str().to_string()), now),
            Particle::base(kaki, "attr_type.attr_hash", AkkValue::Int(spec.attr_hash as i64), now),
            Particle::base(kaki, "attr_type.version", AkkValue::Int(spec.version as i64), now),
        ];

        (kaki, particles)
    }

    // ── Passport audit trail (2026-07-29) ──────────────────────────────
    //
    // "Passport_schema" in the Architect's own naming -- the `passport.*`
    // EAV namespace on this same store, same shape as `error_type.*` /
    // `attr_type.*`. Role=Zikru ("record/entity in tribe domain"): a
    // minted passport is a concrete, one-off audit record of an issued
    // credential, not a type/template definition (Parzu, like ErrorType/
    // AttrType) and not an external file/blob (Kishib, like a crate or
    // playbook artifact) -- matching the Architect's own 3-category
    // taxonomy ("Internal Files, External Files, Files Records").

    /// Mint an Identity-Kaki (role=Zikru) for one issued passport's audit
    /// record and emit its EAV rows. `spec` must already be built from a
    /// passport whose `verify_seal()` passed -- this never re-verifies,
    /// it only records.
    pub fn emit_passport_record(&self, spec: &PassportRecordSpec) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Zikru))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();

        let particles = vec![
            Particle::base(kaki, "passport.passport_id", AkkValue::Text(spec.passport_id.clone()), now),
            Particle::base(kaki, "passport.subject_kaki_hex", AkkValue::Text(spec.subject_kaki_hex.clone()), now),
            Particle::base(kaki, "passport.issuer_kaki_hex", AkkValue::Text(spec.issuer_kaki_hex.clone()), now),
            Particle::base(kaki, "passport.realm", AkkValue::Text(spec.realm.clone()), now),
            Particle::base(kaki, "passport.privilege_level", AkkValue::Int(spec.privilege_level as i64), now),
            Particle::base(kaki, "passport.scopes_csv", AkkValue::Text(spec.scopes_csv.clone()), now),
            Particle::base(kaki, "passport.created_at", AkkValue::Int(spec.created_at as i64), now),
            Particle::base(kaki, "passport.expires_at", AkkValue::Int(spec.expires_at as i64), now),
        ];

        (kaki, particles)
    }

    // ── AnuGovernor run-confirmation registry (2026-07-29) ─────────────
    //
    // `anu_governor_run.*` EAV namespace. Role=Zikru: a run's outcome is a
    // concrete, one-off record of something that happened, same category
    // as a passport mint -- not a type/template (Parzu) and not an
    // external file (Kishib). `operator_*` fields are stored as empty
    // strings / -1 when `None` on the spec (AkkValue has no null variant
    // here) -- callers reading this back treat empty
    // `operator_subject_kaki_hex` as "unauthenticated run", matching
    // `outcome == "BlockedByAuthority"` or a `vault_check_enabled=false`
    // run.

    /// Mint an Identity-Kaki (role=Zikru) for one AnuGovernor run's
    /// confirmation record and emit its EAV rows.
    pub fn emit_anu_governor_run(&self, spec: &AnuGovernorRunRecordSpec) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Zikru))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();

        let mut particles = vec![
            Particle::base(kaki, "anu_governor_run.run_id", AkkValue::Text(spec.run_id.clone()), now),
            Particle::base(kaki, "anu_governor_run.started_at", AkkValue::Int(spec.started_at as i64), now),
            Particle::base(kaki, "anu_governor_run.finished_at", AkkValue::Int(spec.finished_at as i64), now),
            Particle::base(kaki, "anu_governor_run.playbook_count", AkkValue::Int(spec.playbook_count as i64), now),
            Particle::base(kaki, "anu_governor_run.ok_count", AkkValue::Int(spec.ok_count as i64), now),
            Particle::base(kaki, "anu_governor_run.warned_count", AkkValue::Int(spec.warned_count as i64), now),
            Particle::base(kaki, "anu_governor_run.failed_count", AkkValue::Int(spec.failed_count as i64), now),
            Particle::base(kaki, "anu_governor_run.skipped_count", AkkValue::Int(spec.skipped_count as i64), now),
            Particle::base(kaki, "anu_governor_run.outcome", AkkValue::Text(spec.outcome.clone()), now),
            Particle::base(kaki, "anu_governor_run.report_path", AkkValue::Text(spec.report_path.clone()), now),
        ];

        if let Some(reason) = &spec.blocked_reason {
            particles.push(Particle::base(kaki, "anu_governor_run.blocked_reason", AkkValue::Text(reason.clone()), now));
        }
        if let Some(hex) = &spec.operator_subject_kaki_hex {
            particles.push(Particle::base(kaki, "anu_governor_run.operator_subject_kaki_hex", AkkValue::Text(hex.clone()), now));
        }
        if let Some(realm) = &spec.operator_realm {
            particles.push(Particle::base(kaki, "anu_governor_run.operator_realm", AkkValue::Text(realm.clone()), now));
        }
        if let Some(level) = spec.operator_privilege_level {
            particles.push(Particle::base(kaki, "anu_governor_run.operator_privilege_level", AkkValue::Int(level as i64), now));
        }
        if let Some(user) = &spec.os_username {
            particles.push(Particle::base(kaki, "anu_governor_run.os_username", AkkValue::Text(user.clone()), now));
        }
        if let Some(groups) = &spec.os_groups_csv {
            particles.push(Particle::base(kaki, "anu_governor_run.os_groups_csv", AkkValue::Text(groups.clone()), now));
        }
        if let Some(role) = &spec.os_bahyway_role {
            particles.push(Particle::base(kaki, "anu_governor_run.os_bahyway_role", AkkValue::Text(role.clone()), now));
        }

        (kaki, particles)
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_journal::ErrorSeverity;
    use template_engine::AttrType;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x00E3))
    }

    #[test]
    fn emits_error_type_identity_with_parzu_role() {
        let m = minter();
        let spec = ErrorTypeSpec::new(
            "LEMNI_CONSONANT_COUNT",
            "Identity phrase needs at least 3 consonants.",
            ErrorSeverity::Error,
            "kupru",
        );
        let (kaki, particles) = RegistryEmitter::new(&m).emit_error_type(&spec);
        assert_eq!(kaki.role(), KakiRole::Parzu);
        assert_eq!(particles.len(), 4);
        assert!(particles.iter().any(|p| p.attribute == "error_type.code"));
    }

    #[test]
    fn error_occurrence_targets_the_existing_error_type_kaki() {
        let m = minter();
        let type_spec = ErrorTypeSpec::new("X", "x", ErrorSeverity::Warning, "kupru");
        let (type_kaki, _) = RegistryEmitter::new(&m).emit_error_type(&type_spec);

        let occurrence = ErrorOccurrenceSpec {
            source: "sargon-passport-manager",
            detail: "1 of 3 shares gathered".to_string(),
        };
        let particles = RegistryEmitter::new(&m).emit_error_occurrence(type_kaki, &occurrence);

        assert_eq!(particles.len(), 2);
        for p in &particles {
            assert_eq!(p.entity, type_kaki, "occurrence rows must target the ErrorType's own identity");
        }
    }

    #[test]
    fn emits_attr_type_identity_with_parzu_role() {
        let m = minter();
        let spec = AttrTypeSpec::new(0x1001, "price_usd", AttrType::FixedPointScaledInteger { scale: 2 }, 1);
        let (kaki, particles) = RegistryEmitter::new(&m).emit_attr_type(&spec);
        assert_eq!(kaki.role(), KakiRole::Parzu);
        assert!(particles.iter().any(|p| p.attribute == "attr_type.representation"));
    }

    fn sample_passport_spec() -> PassportRecordSpec {
        PassportRecordSpec {
            passport_id: "00000000-0000-4000-8000-000000000000".to_string(),
            subject_kaki_hex: "aa".repeat(16),
            issuer_kaki_hex: "bb".repeat(16),
            realm: "bahyway".to_string(),
            privilege_level: 5,
            scopes_csv: "bahyway:read,story:read".to_string(),
            created_at: 1_000,
            expires_at: 1_000 + 54 * 3600,
        }
    }

    #[test]
    fn emits_passport_record_identity_with_zikru_role() {
        let m = minter();
        let spec = sample_passport_spec();
        let (kaki, particles) = RegistryEmitter::new(&m).emit_passport_record(&spec);
        assert_eq!(kaki.role(), KakiRole::Zikru);
        assert_eq!(particles.len(), 8);
        assert!(particles.iter().any(|p| p.attribute == "passport.privilege_level"));
    }

    #[test]
    fn passport_record_never_carries_seal_or_key_fields() {
        // Explicit negative check: no attribute name may reference the
        // seal (hmac/nonce/content_hash) or any key material -- this
        // audit trail is metadata-only, per this module's own header.
        let m = minter();
        let (_, particles) = RegistryEmitter::new(&m).emit_passport_record(&sample_passport_spec());
        for p in &particles {
            let a = p.attribute.to_lowercase();
            assert!(!a.contains("hmac") && !a.contains("nonce") && !a.contains("content_hash")
                && !a.contains("signing_key") && !a.contains("private"),
                "passport.* particle leaked seal/key-shaped attribute: {a}");
        }
    }

    fn sample_run_spec(operator: Option<(&str, &str, u8)>) -> AnuGovernorRunRecordSpec {
        let (subject_kaki_hex, realm, privilege_level) = match operator {
            Some((k, r, l)) => (Some(k.to_string()), Some(r.to_string()), Some(l)),
            None => (None, None, None),
        };
        AnuGovernorRunRecordSpec {
            run_id: "20260729T120000Z".to_string(),
            started_at: 1_000,
            finished_at: 1_060,
            playbook_count: 5,
            ok_count: 4,
            warned_count: 1,
            failed_count: 0,
            skipped_count: 0,
            outcome: "Completed".to_string(),
            blocked_reason: None,
            operator_subject_kaki_hex: subject_kaki_hex,
            operator_realm: realm,
            operator_privilege_level: privilege_level,
            os_username: Some("architect".to_string()),
            os_groups_csv: Some("architect,wheel,bahyway-architect".to_string()),
            os_bahyway_role: Some("bahyway-architect".to_string()),
            report_path: "docs/SHEDU/anu_governor_reports/report_20260729T120000Z.md".to_string(),
        }
    }

    #[test]
    fn emits_run_record_identity_with_zikru_role() {
        let m = minter();
        let spec = sample_run_spec(Some(("aa".repeat(16).as_str(), "bahyway", 7)));
        let (kaki, particles) = RegistryEmitter::new(&m).emit_anu_governor_run(&spec);
        assert_eq!(kaki.role(), KakiRole::Zikru);
        // 10 base fields + 3 operator_* + 3 os_* = 16
        assert_eq!(particles.len(), 16);
        assert!(particles.iter().any(|p| p.attribute == "anu_governor_run.outcome"));
    }

    #[test]
    fn unauthenticated_run_omits_operator_particles_entirely() {
        let m = minter();
        let mut spec = sample_run_spec(None);
        spec.outcome = "BlockedByAuthority".to_string();
        spec.blocked_reason = Some("vault_check_enabled=false: run was unauthenticated".to_string());
        let (_, particles) = RegistryEmitter::new(&m).emit_anu_governor_run(&spec);
        assert!(!particles.iter().any(|p| p.attribute.starts_with("anu_governor_run.operator_")));
        assert!(particles.iter().any(|p| p.attribute == "anu_governor_run.blocked_reason"));
        // OS identity is independent of vault-passport authentication --
        // it must still be present on an unauthenticated (no-vault) run.
        assert!(particles.iter().any(|p| p.attribute == "anu_governor_run.os_username"));
    }

    #[test]
    fn blocked_by_authority_outcome_and_privilege_are_both_queryable() {
        let m = minter();
        let mut spec = sample_run_spec(Some(("bb".repeat(16).as_str(), "bahyway", 3)));
        spec.outcome = "BlockedByAuthority".to_string();
        spec.blocked_reason = Some("operator privilege_level=3 below required minimum=5".to_string());
        let (_, particles) = RegistryEmitter::new(&m).emit_anu_governor_run(&spec);
        let outcome = particles.iter().find(|p| p.attribute == "anu_governor_run.outcome").unwrap();
        assert!(matches!(&outcome.value, AkkValue::Text(t) if t == "BlockedByAuthority"));
        let level = particles.iter().find(|p| p.attribute == "anu_governor_run.operator_privilege_level").unwrap();
        assert!(matches!(level.value, AkkValue::Int(3)));
    }
}
