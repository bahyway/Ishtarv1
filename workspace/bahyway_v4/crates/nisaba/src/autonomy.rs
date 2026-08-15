//! autonomy — NISABA's bounded self-decision grant.
//!
//! Granted 2026-07-18, in response to the Architect's request for a
//! "robot control" on the Ecosystem: NISABA may resolve small-detail
//! findings herself, in a scoped, environment-gated, provably
//! Architect-only way, while everything High Priority / Major Effect
//! still always waits for the Architect's ratification (CSR-08).
//!
//! The whole safety story rests on one fact: a `NisabaGrant` only
//! verifies if it was signed with the Architect's `kupru::SealKeyPair`
//! signing key. That key is never shipped anywhere this code runs — only
//! its public `SovereignVerifier` counterpart is. So "the client can
//! never configure NISABA as it wishes" is not a policy this module
//! promises to enforce; it is a fact about what Ed25519 signatures make
//! possible to forge (nothing, without the key). Widening NISABA's
//! autonomy is always, unconditionally, an act only the Architect can
//! perform — this module cannot mint the grants it verifies.
//!
//! Reuses `kupru::akkadian_seal` (the Ecosystem's existing sealed
//! signature layer, already used elsewhere for sovereign authority)
//! rather than inventing a new trust primitive for this one purpose.
use kupru::akkadian_seal::{AkkadianSeal, SealVerifier};
use kupru::KupruResult;

/// Which deployment this orchestrator instance is running as. A grant
/// issued for one environment never applies to another — a Test grant
/// cannot widen Dev or Prod autonomy, even if somehow presented there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NisabaEnvironment {
    Test,
    Dev,
    Prod,
}

impl NisabaEnvironment {
    fn tag(self) -> u8 {
        match self {
            NisabaEnvironment::Test => 0,
            NisabaEnvironment::Dev => 1,
            NisabaEnvironment::Prod => 2,
        }
    }
}

/// What kind of self-decision the grant authorizes. Only one class
/// exists today, deliberately: NISABA may resolve Watch-severity
/// findings herself — the "small details" tier — and nothing wider.
/// Any finding that reaches Warning or Critical (StoryEngine's
/// HighPriority band) always goes to the Architect, with or without a
/// grant; this enum has no variant that could change that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutonomyClass {
    /// Auto-resolve Watch-band findings by logging them as self-handled
    /// instead of enqueueing to the Data Steward. Never writes to
    /// ecosystem storage, never submits an Event-Kaki through AdadGate —
    /// only changes which queue a finding lands in.
    WatchBandOnly,
}

impl AutonomyClass {
    fn tag(self) -> u8 {
        match self {
            AutonomyClass::WatchBandOnly => 0,
        }
    }
}

/// The exact claim the Architect's signature covers — environment,
/// class, and an issue timestamp, canonically encoded so nothing about
/// what was authorized is ambiguous or extendable after the fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrantClaim {
    pub environment: NisabaEnvironment,
    pub class: AutonomyClass,
    pub issued_at: u64,
}

impl GrantClaim {
    /// The bytes actually signed/verified. A fixed domain-separation tag
    /// prefix stops a signature meant for some other message type from
    /// ever being replayed as a valid grant.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16 + 1 + 1 + 8);
        buf.extend_from_slice(b"NISABA_GRANT_V1\0");
        buf.push(self.environment.tag());
        buf.push(self.class.tag());
        buf.extend_from_slice(&self.issued_at.to_be_bytes());
        buf
    }
}

/// A grant as it travels: the claim plus the Architect's signature over
/// `claim.canonical_bytes()`. Constructing one with a valid signature
/// requires the Architect's signing key; verifying one requires only the
/// public `SovereignVerifier`, which is safe to embed anywhere.
#[derive(Debug, Clone)]
pub struct NisabaGrant {
    pub claim: GrantClaim,
    pub signature: Vec<u8>,
}

impl NisabaGrant {
    /// Mint a grant. Only callable by whoever holds the Architect's
    /// `AkkadianSeal` signing key (e.g. `kupru::SealKeyPair`) — this
    /// function does not itself gate who may call it; the signature it
    /// produces is what gates whether the result verifies anywhere else.
    pub fn issue(signer: &dyn AkkadianSeal, claim: GrantClaim) -> KupruResult<Self> {
        let signature = signer.seal(&claim.canonical_bytes())?;
        Ok(NisabaGrant { claim, signature })
    }

    /// Verify this grant against a public verifier. Anyone — including
    /// the client — can call this; nobody without the Architect's
    /// signing key can produce a grant that returns `true` here.
    pub fn verify(&self, verifier: &dyn SealVerifier) -> bool {
        verifier.verify(&self.claim.canonical_bytes(), &self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kupru::akkadian_seal::SealKeyPair;

    fn claim(env: NisabaEnvironment) -> GrantClaim {
        GrantClaim { environment: env, class: AutonomyClass::WatchBandOnly, issued_at: 1_700_000_000 }
    }

    #[test]
    fn architect_signed_grant_verifies() {
        let architect_key = SealKeyPair::generate().unwrap();
        let grant = NisabaGrant::issue(&architect_key, claim(NisabaEnvironment::Test)).unwrap();
        assert!(grant.verify(&architect_key.verifier()));
    }

    #[test]
    fn grant_signed_by_a_different_key_does_not_verify() {
        let architect_key = SealKeyPair::generate().unwrap();
        let impostor_key = SealKeyPair::generate().unwrap();
        // The "client" forges a grant with their own key, but only the
        // Architect's public verifier is ever trusted downstream.
        let forged = NisabaGrant::issue(&impostor_key, claim(NisabaEnvironment::Prod)).unwrap();
        assert!(!forged.verify(&architect_key.verifier()));
    }

    #[test]
    fn tampered_claim_does_not_verify() {
        let architect_key = SealKeyPair::generate().unwrap();
        let mut grant = NisabaGrant::issue(&architect_key, claim(NisabaEnvironment::Test)).unwrap();
        // Someone edits the environment after signing, trying to widen a
        // Test grant into a Prod one.
        grant.claim.environment = NisabaEnvironment::Prod;
        assert!(!grant.verify(&architect_key.verifier()));
    }

    #[test]
    fn different_environments_produce_different_canonical_bytes() {
        let a = claim(NisabaEnvironment::Test).canonical_bytes();
        let b = claim(NisabaEnvironment::Prod).canonical_bytes();
        assert_ne!(a, b);
    }
}
