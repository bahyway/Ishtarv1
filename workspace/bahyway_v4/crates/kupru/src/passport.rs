//! `SargonPassport` — The sovereign credential (ŠARRUKIN ṬUPPU).
//!
//! Four-layer credential carried by every stakeholder, service, and
//! autonomous `.akk` agent in BahyWay.Ecosystem.
//!
//! ```text
//! QUPPU  (Basket)  → Bytes 0-3   UUID / passport identity
//! NĀRU   (River)   → Bytes 4-7   Tribe / realm
//! IŠTAR  (Favor)   → Bytes 8-11  Domain + quality scopes
//! KUPRU  (Bitumen) → Bytes 12-15 Freshness + checksum + Mamdani score
//! ```

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    akkadian_root::{AkkadianName, LinguisticProof},
    akkadian_seal::SealKeyPair,
    passport_seal::{passport_seal, passport_verify_seal},
    sargon_kdf::{validate_credential_timing, SargonKdf, SATTATU_MAX},
    sovereign_hash::default_hasher,
    KupruError, KupruResult,
};

// ── Layer 1: QUPPU (Basket / Outer Container) ─────────────────────────────────

/// QUPPU — outer container. Identifies the passport and its lifespan.
///
/// *"She set me in a basket of rushes."* — Sargon Legend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuppuLayer {
    /// Unique passport identifier (UUID v4).
    pub passport_id: Uuid,
    /// Unix timestamp (seconds) when this passport was issued.
    pub created_at: u64,
    /// Unix timestamp — always `created_at + SATTATU_MAX`.
    pub expires_at: u64,
    /// Issuer's KAKI PK (16 bytes).
    pub issuer_kaki: [u8; 16],
}

/// Renewal buffer -- reissue once 80% of a passport's lifetime has
/// elapsed, never at the last second. Integer math (not float) so this
/// is bit-identical across platforms. Sargon's passports die at the
/// SATTATU_MAX cliff with zero grace period (validate_credential_timing
/// rejects the instant `now > expires_at`), so a caller that waits until
/// expiry to renew risks a real access gap -- this exists so nothing has
/// to guess when "soon" is.
const RENEWAL_NUMERATOR: u64 = 4;
const RENEWAL_DENOMINATOR: u64 = 5; // 4/5 = 80%

impl QuppuLayer {
    pub fn new(issuer_kaki: [u8; 16]) -> Self {
        let now = SargonKdf::sovereign_now();
        Self {
            passport_id: Uuid::new_v4(),
            created_at: now,
            expires_at: now + SATTATU_MAX,
            issuer_kaki,
        }
    }

    pub fn validate_timing(&self) -> KupruResult<()> {
        validate_credential_timing(self.created_at, self.expires_at)
    }

    fn renewal_threshold(&self) -> u64 {
        let lifetime = self.expires_at.saturating_sub(self.created_at);
        self.created_at + (lifetime * RENEWAL_NUMERATOR) / RENEWAL_DENOMINATOR
    }

    /// True once 80% of this passport's lifetime has elapsed -- the
    /// recommended moment to call `SargonPassport::issue()` again and
    /// swap in the fresh result, well before the hard expiry cliff.
    pub fn renewal_due(&self) -> bool {
        SargonKdf::sovereign_now() >= self.renewal_threshold()
    }

    /// Seconds remaining until renewal is recommended (0 if already due).
    /// For SATTATU_MAX's default 54h, this starts at ~43.2 hours.
    pub fn seconds_until_renewal_due(&self) -> u64 {
        self.renewal_threshold()
            .saturating_sub(SargonKdf::sovereign_now())
    }
}

// ── Layer 2: KUPRU (Bitumen Seal / Cryptographic Integrity) ───────────────────

/// KUPRU — the cryptographic seal. Prevents tampering with inner layers.
///
/// *"With bitumen she sealed my lid."* — Sargon Legend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KupruLayer {
    /// SovereignHash digest of the inner content (NĀRU + IŠTAR layers).
    pub content_hash: Vec<u8>,
    /// HMAC-keyed digest for integrity across all layers.
    pub hmac: Vec<u8>,
    /// Random nonce used during keyed hashing (16 bytes).
    pub nonce: [u8; 16],
    /// Hash algorithm identifier (from `HashAlgorithm` repr).
    pub algorithm_id: u8,
}

impl KupruLayer {
    pub fn compute(inner_bytes: &[u8], integrity_key: &[u8]) -> KupruResult<Self> {
        use rand::RngCore;
        let hasher = default_hasher();

        let mut nonce = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce);

        let content_hash = hasher.digest(inner_bytes);
        let hmac = hasher.keyed_digest(integrity_key, inner_bytes)?;
        let algorithm_id = hasher.algorithm() as u8;

        Ok(Self {
            content_hash,
            hmac,
            nonce,
            algorithm_id,
        })
    }

    pub fn verify(&self, inner_bytes: &[u8], integrity_key: &[u8]) -> KupruResult<()> {
        let hasher = default_hasher();
        let expected_hash = hasher.digest(inner_bytes);
        let expected_hmac = hasher.keyed_digest(integrity_key, inner_bytes)?;

        use subtle::ConstantTimeEq;
        let hash_ok: bool = self.content_hash.ct_eq(&expected_hash).into();
        let hmac_ok: bool = self.hmac.ct_eq(&expected_hmac).into();

        if hash_ok && hmac_ok {
            Ok(())
        } else {
            Err(KupruError::SealBroken(
                "NAKRU: KUPRU layer integrity check failed".into(),
            ))
        }
    }
}

// ── Layer 3: NĀRU (River / Identity Channel) ──────────────────────────────────

/// NĀRU — the identity river. Carries who the passport belongs to.
///
/// *"The river bore me up and carried me to Akki."* — Sargon Legend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaruLayer {
    /// Subject's KAKI 16-byte PK.
    pub subject_kaki: [u8; 16],
    /// Akkadian identity of the subject.
    pub akkadian_name: AkkadianName,
    /// Linguistic proof of known Akkadian phrase.
    pub linguistic_proof: LinguisticProof,
    /// Realm / tenant identifier (maps to client schema in EnkiDB).
    pub realm: String,
    /// MUDÛ score of the linguistic proof (0–7).
    pub mudu_score: u8,
}

// ── Layer 4: IŠTAR (Granting Favor / Authorization) ───────────────────────────

/// IŠTAR — authorization and permissions.
///
/// *"Ishtar granted me her love, and for four and fifty years
/// I exercised kingship."* — Sargon Legend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IshtarLayer {
    /// Granted permission scopes in Akkadian notation (IŠRETU).
    pub isretu_scopes: Vec<String>,
    /// Resource access matrix: resource → allowed operations.
    pub access_matrix: std::collections::HashMap<String, Vec<String>>,
    /// Delegation chain — KAKI PKs of prior issuers.
    pub nadanu_chain: Vec<[u8; 16]>,
    /// Maximum privilege level: 1=gardener … 7=king (Sargon's career arc).
    pub privilege_level: u8,
}

impl IshtarLayer {
    /// Minimal read-only gardener permissions (entry level).
    pub fn gardener(realm: &str) -> Self {
        Self {
            isretu_scopes: vec![format!("{realm}:read"), "story:read".into()],
            access_matrix: std::collections::HashMap::new(),
            nadanu_chain: vec![],
            privilege_level: 1, // NUKARIB (gardener)
        }
    }

    /// Full sovereign access — DUB.SAR architect level.
    pub fn architect(realm: &str) -> Self {
        Self {
            isretu_scopes: vec![
                format!("{realm}:*"),
                "kaki:*".into(),
                "enki:*".into(),
                "story:*".into(),
                "zero:*".into(), // v4.0: no "Way" suffix
            ],
            access_matrix: std::collections::HashMap::new(),
            nadanu_chain: vec![],
            privilege_level: 7, // ŠARRU (king)
        }
    }
}

// ── SargonPassport ────────────────────────────────────────────────────────────

/// The complete Sargon Passport — four layers + outer AkkadianSeal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SargonPassport {
    pub quppu: QuppuLayer,
    pub kupru: KupruLayer,
    pub naru: NaruLayer,
    pub istar: IshtarLayer,
    pub outer_seal: Vec<u8>,
    pub issuer_verifying_key: Vec<u8>,
}

impl SargonPassport {
    /// Issue a new passport. The only constructor — a passport cannot
    /// exist without a valid seal.
    pub fn issue(
        naru: NaruLayer,
        istar: IshtarLayer,
        issuer_kaki: [u8; 16],
        issuer_keypair: &SealKeyPair,
        integrity_key: &[u8; 32],
    ) -> KupruResult<Self> {
        let quppu = QuppuLayer::new(issuer_kaki);

        let inner =
            serde_json::to_vec(&(&naru, &istar)).map_err(|e| KupruError::Io(e.to_string()))?;

        let kupru = KupruLayer::compute(&inner, integrity_key)?;

        // Fix 4 (passport_seal.rs): canonical binary, never JSON -- JSON
        // field ordering isn't guaranteed across serde versions, which
        // would silently invalidate every previously-issued passport on
        // a dependency bump. This was already documented as mandatory
        // ("the ONLY function that should be called") but SargonPassport
        // itself never actually called it until now.
        let outer_seal = passport_seal(&quppu, &kupru, &naru, &istar, issuer_keypair)?;

        Ok(Self {
            quppu,
            kupru,
            naru,
            istar,
            outer_seal,
            issuer_verifying_key: issuer_keypair.verifying_key_bytes().to_vec(),
        })
    }

    /// Verify the passport seal and timing. Does not check scopes.
    pub fn verify_seal(&self) -> KupruResult<()> {
        self.quppu.validate_timing()?;

        let ok = passport_verify_seal(
            &self.quppu,
            &self.kupru,
            &self.naru,
            &self.istar,
            &self.outer_seal,
            &self.issuer_verifying_key,
        )?;

        if ok {
            Ok(())
        } else {
            Err(KupruError::SealBroken(
                "NAKRU: outer passport seal is invalid".into(),
            ))
        }
    }

    /// True if the named scope (or a wildcard covering it) is granted.
    pub fn has_scope(&self, scope: &str) -> bool {
        self.istar
            .isretu_scopes
            .iter()
            .any(|s| s == scope || (s.ends_with(":*") && scope.starts_with(&s[..s.len() - 1])))
    }

    pub fn subject_kaki(&self) -> [u8; 16] {
        self.naru.subject_kaki
    }

    pub fn passport_id(&self) -> String {
        self.quppu.passport_id.to_string()
    }

    /// True once this passport should be reissued (80% of its lifetime
    /// elapsed) -- check this on a schedule (e.g. once per login-gate
    /// use, or on a periodic timer for long-running services) and call
    /// `SargonPassport::issue()` again with the same NaruLayer/IshtarLayer
    /// when it flips true. There is no "extend" operation by design --
    /// renewal is always a fresh issuance with a fresh QUPPU/outer_seal,
    /// never a mutation of the existing one.
    pub fn renewal_due(&self) -> bool {
        self.quppu.renewal_due()
    }

    /// Seconds remaining until renewal is recommended (0 if already due).
    pub fn seconds_until_renewal_due(&self) -> u64 {
        self.quppu.seconds_until_renewal_due()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::akkadian_root::{AkkadianName, LinguisticProof};

    fn issue_test_passport() -> (SargonPassport, SealKeyPair) {
        let keypair = SealKeyPair::generate().unwrap();
        let integrity_key = [0x42u8; 32];
        let naru = NaruLayer {
            subject_kaki: [7u8; 16],
            akkadian_name: AkkadianName::dubsar(),
            linguistic_proof: LinguisticProof::create("sarru-kibrat-arbaim").unwrap(),
            realm: "bahyway-test".into(),
            mudu_score: 5,
        };
        let istar = IshtarLayer::gardener("bahyway-test");
        let passport = SargonPassport::issue(naru, istar, [1u8; 16], &keypair, &integrity_key)
            .expect("issuance must succeed");
        (passport, keypair)
    }

    /// A legitimately-issued passport must verify.
    #[test]
    fn genuine_passport_verifies() {
        let (passport, _keypair) = issue_test_passport();
        assert!(passport.verify_seal().is_ok());
    }

    /// Flipping ANY single bit in ANY layer after issuance must be caught --
    /// this is the actual "hard to breach" property: the outer Ed25519
    /// signature covers all four layers via canonical binary encoding, so
    /// there is no field an attacker can silently modify.
    #[test]
    fn tampering_with_privilege_level_is_detected() {
        let (mut passport, _keypair) = issue_test_passport();
        assert_eq!(passport.istar.privilege_level, 1); // gardener
        passport.istar.privilege_level = 7; // attacker tries to self-promote to king
        assert!(
            passport.verify_seal().is_err(),
            "privilege escalation must invalidate the seal"
        );
    }

    #[test]
    fn tampering_with_subject_kaki_is_detected() {
        let (mut passport, _keypair) = issue_test_passport();
        passport.naru.subject_kaki = [0xFFu8; 16]; // attacker tries to claim someone else's identity
        assert!(
            passport.verify_seal().is_err(),
            "identity substitution must invalidate the seal"
        );
    }

    #[test]
    fn tampering_with_expiry_is_detected() {
        let (mut passport, _keypair) = issue_test_passport();
        passport.quppu.expires_at = SargonKdf::sovereign_now() + 999_999_999; // attacker extends TTL
        assert!(
            passport.verify_seal().is_err(),
            "TTL extension must invalidate the seal"
        );
    }

    /// An attacker who doesn't hold the issuer's private key cannot produce
    /// a passport that verifies against the real issuer's public key --
    /// this is the actual forgery-resistance property (not just tamper
    /// detection on an already-issued passport).
    #[test]
    fn forged_passport_from_different_keypair_fails_verification() {
        let (mut genuine, _real_keypair) = issue_test_passport();
        let attacker_keypair = SealKeyPair::generate().unwrap();

        // Attacker re-signs the (possibly modified) content with their OWN
        // key, then claims it came from the real issuer by swapping in the
        // real issuer's public key bytes.
        let forged_seal = passport_seal(
            &genuine.quppu,
            &genuine.kupru,
            &genuine.naru,
            &genuine.istar,
            &attacker_keypair,
        )
        .unwrap();
        genuine.outer_seal = forged_seal;
        // issuer_verifying_key still says it's the real issuer -- but the
        // signature was produced by attacker_keypair's private key, which
        // does not match. Verification must fail.
        assert!(
            genuine.verify_seal().is_err(),
            "a seal from the wrong private key must never verify"
        );
    }

    #[test]
    fn expired_passport_fails_timing_check() {
        let (mut passport, _keypair) = issue_test_passport();
        // Simulate a passport issued in the past whose TTL has lapsed.
        passport.quppu.expires_at = SargonKdf::sovereign_now() - 10;
        assert!(
            passport.verify_seal().is_err(),
            "an expired passport must fail verify_seal()"
        );
    }

    #[test]
    fn scope_matching_respects_wildcards() {
        let istar = IshtarLayer::architect("bahyway");
        let passport_scopes = istar.isretu_scopes.clone();
        assert!(passport_scopes.iter().any(|s| s == "kaki:*"));

        let (passport, _keypair) = {
            let keypair = SealKeyPair::generate().unwrap();
            let naru = NaruLayer {
                subject_kaki: [9u8; 16],
                akkadian_name: AkkadianName::dubsar(),
                linguistic_proof: LinguisticProof::create("architect-phrase").unwrap(),
                realm: "bahyway".into(),
                mudu_score: 7,
            };
            let p = SargonPassport::issue(naru, istar, [1u8; 16], &keypair, &[0x11u8; 32]).unwrap();
            (p, keypair)
        };

        assert!(passport.has_scope("kaki:read")); // covered by "kaki:*"
        assert!(passport.has_scope("kaki:write")); // covered by "kaki:*"
        assert!(!passport.has_scope("unrelated:read"));
    }

    #[test]
    fn gardener_scope_is_read_only() {
        let (passport, _keypair) = issue_test_passport();
        assert!(passport.has_scope("bahyway-test:read"));
        assert!(!passport.has_scope("bahyway-test:write"));
        assert!(!passport.has_scope("kaki:*"));
    }

    #[test]
    fn freshly_issued_passport_does_not_need_renewal() {
        let (passport, _keypair) = issue_test_passport();
        assert!(!passport.renewal_due());
        // Fresh passport: ~80% of SATTATU_MAX (54h) remains until renewal.
        assert!(passport.seconds_until_renewal_due() > (SATTATU_MAX * 4) / 5 - 5);
    }

    #[test]
    fn passport_past_80_percent_lifetime_needs_renewal() {
        let (mut passport, _keypair) = issue_test_passport();
        // Simulate a passport 85% through its original 54h window --
        // shift BOTH created_at and expires_at back together so the
        // SATTATU_MAX window size is preserved, not stretched.
        let now = SargonKdf::sovereign_now();
        let elapsed = (SATTATU_MAX * 85) / 100;
        passport.quppu.created_at = now - elapsed;
        passport.quppu.expires_at = passport.quppu.created_at + SATTATU_MAX;

        assert!(passport.renewal_due());
        assert_eq!(passport.seconds_until_renewal_due(), 0);
        // Still genuinely valid -- renewal is advance warning, not expiry.
        assert!(SargonKdf::sovereign_now() < passport.quppu.expires_at);
    }
}
