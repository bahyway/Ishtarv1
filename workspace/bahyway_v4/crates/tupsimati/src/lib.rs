//! Tupsimati — the Tablet of Destinies.
//!
//! Configuration model for the seven EnkiDB databases, SLA definitions, and
//! the Tertu FIS (Mamdani fuzzy inference) rule base. Serializes to a
//! sovereign line format (no JSON, no serde) and seals with the real
//! `kupru::AkkadianSeal` (Ed25519) before it may decree.
//!
//! From the iPhone-session PB-169 review, landed with one correction: the
//! original design path-depended on a standalone `akkadian-seal` crate that
//! does not exist. `AkkadianSeal` is real, but it is a trait living inside
//! `kupru` (`kupru::akkadian_seal::{AkkadianSeal, SealKeyPair}`), already
//! proven in `kupru-gdext` for Sargon/Gilgamesh/DubSar login. This crate
//! depends on `kupru` directly and seals through that real trait.
//!
//! Guardrails carried over from the review, all still real and enforced
//! here: the tablet composes, it never rules -- applying a sealed tablet is
//! always a generated, Architect-run playbook, never a direct mutation.
//! `emit_playbook` refuses an unsealed tablet.

pub mod fis;
pub mod playbook;

use kupru::AkkadianSeal;

/// The seven, in pipeline order, with canonical ports.
pub const ENKIDB_SEVEN: [(&str, u16); 7] = [
    ("EnkiSDB", 7001),
    ("EnkiODB", 7002),
    ("EnkiQDB", 7003),
    ("EnkiDB", 7004),
    ("EnkiDW", 7005),
    ("EnkiMDB", 7006),
    ("EnkiDDB", 7007),
];

/// Visibility boundary (SUSA): Internal knobs never cross outward; Client
/// knobs are the gateway-safe subset.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scope {
    Internal,
    Client,
}

#[derive(Clone, Debug)]
pub struct DbConfig {
    pub name: String,
    pub port: u16,
    // Internal scope
    pub journal_segment_mb: u32,
    pub partition_rule: String,
    pub anu_hot_orbits: Vec<String>,
    pub page_pin_window_s: u32,
    pub retention_days: u16,
    // Client scope
    pub sla_read_ms: u32,
    pub sla_availability: f64,
}

#[derive(Default, Clone, Debug)]
pub struct Tablet {
    pub version: u32,
    pub created_millis: u64,
    pub dbs: Vec<DbConfig>,
    pub fis: fis::FisConfig,
    /// Ed25519 signature bytes over `to_lines()`, once sealed. `None` means
    /// this tablet may not yet decree.
    pub seal: Option<Vec<u8>>,
}

impl Tablet {
    /// Sovereign line serialization:
    ///   TUPSIMATI 1
    ///   DB EnkiODB 7002 SEG 128 PART day PIN 3600 RET 14 SLAMS 50 SLAAV 0.999
    pub fn to_lines(&self) -> String {
        let mut out = format!("TUPSIMATI {}\nCREATED {}\n", self.version, self.created_millis);
        for db in &self.dbs {
            out += &format!(
                "DB {} {} SEG {} PART {} PIN {} RET {} SLAMS {} SLAAV {}\n",
                db.name,
                db.port,
                db.journal_segment_mb,
                db.partition_rule,
                db.page_pin_window_s,
                db.retention_days,
                db.sla_read_ms,
                db.sla_availability
            );
            for o in &db.anu_hot_orbits {
                out += &format!("HOT {} {}\n", db.name, o);
            }
        }
        out += &self.fis.to_lines();
        out
    }

    /// Validation before sealing: refuse malformed destiny.
    pub fn validate(&self) -> Result<(), String> {
        let names: Vec<&str> = ENKIDB_SEVEN.iter().map(|(n, _)| *n).collect();
        for db in &self.dbs {
            if !names.contains(&db.name.as_str()) {
                return Err(format!("unknown database {}", db.name));
            }
            if db.journal_segment_mb == 0 || db.retention_days == 0 {
                return Err(format!("{}: zero-valued destiny forbidden", db.name));
            }
        }
        self.fis.validate()
    }

    /// Seal with a real `kupru::AkkadianSeal` implementor (e.g.
    /// `kupru::SealKeyPair`). Refuses to overwrite an existing seal --
    /// re-sealing means bumping `version` and sealing a new tablet.
    pub fn seal_with(&mut self, sealer: &impl AkkadianSeal) -> Result<(), String> {
        if self.seal.is_some() {
            return Err("tablet already sealed -- bump version for a new tablet".into());
        }
        self.validate()?;
        let bytes = self.to_lines().into_bytes();
        let seal = sealer.seal(&bytes).map_err(|e| format!("seal failed: {e:?}"))?;
        self.seal = Some(seal);
        Ok(())
    }

    pub fn is_sealed(&self) -> bool {
        self.seal.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kupru::SealKeyPair;

    fn sample_tablet() -> Tablet {
        Tablet {
            version: 1,
            created_millis: 1_000,
            dbs: vec![DbConfig {
                name: "EnkiODB".into(),
                port: 7002,
                journal_segment_mb: 128,
                partition_rule: "day".into(),
                anu_hot_orbits: vec!["nusku".into()],
                page_pin_window_s: 3600,
                retention_days: 14,
                sla_read_ms: 50,
                sla_availability: 0.999,
            }],
            fis: fis::FisConfig::default(),
            seal: None,
        }
    }

    #[test]
    fn validate_rejects_unknown_database() {
        let mut t = sample_tablet();
        t.dbs[0].name = "NotARealDb".into();
        assert!(t.validate().is_err());
    }

    #[test]
    fn validate_rejects_zero_valued_destiny() {
        let mut t = sample_tablet();
        t.dbs[0].journal_segment_mb = 0;
        assert!(t.validate().is_err());
    }

    #[test]
    fn seal_with_real_kupru_key_produces_a_verifiable_seal() {
        let mut t = sample_tablet();
        let key = SealKeyPair::generate().expect("keygen");
        assert!(!t.is_sealed());
        t.seal_with(&key).expect("seal");
        assert!(t.is_sealed());

        // The seal actually verifies against the tablet's own bytes, using
        // the real kupru trait -- not a placeholder.
        let bytes = t.to_lines().into_bytes();
        assert!(key.verify(&bytes, t.seal.as_ref().unwrap()));
    }

    #[test]
    fn cannot_reseal_an_already_sealed_tablet() {
        let mut t = sample_tablet();
        let key = SealKeyPair::generate().expect("keygen");
        t.seal_with(&key).unwrap();
        assert!(t.seal_with(&key).is_err());
    }

    #[test]
    fn to_lines_round_trips_the_hot_orbit_line() {
        let t = sample_tablet();
        let lines = t.to_lines();
        assert!(lines.contains("DB EnkiODB 7002 SEG 128 PART day PIN 3600 RET 14 SLAMS 50 SLAAV 0.999"));
        assert!(lines.contains("HOT EnkiODB nusku"));
    }
}
