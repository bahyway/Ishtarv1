//! HeptaGate — the 7 Hepta Gates, GATE-1 ruling (merged, 2026-07-07).
//!
//! This resolves the long-standing "doc vs SVG" conflict flagged repeatedly
//! across this ecosystem's own documents (GLS-001 §4 "⚔ GATE-1 conflict";
//! MAN-001's BeeMDM entry criteria; PBCOLLECTIONS_Master_Index PB-162):
//! two source materials assigned different per-gate pipeline functions to
//! the same 7 named gates.
//!
//! Two axes were being conflated under one "GATE-1" label, and this module
//! keeps them explicitly separate:
//!
//! 1. **Sector** — which of the 7 Forge2 architectural domains a gate
//!    belongs to. Already sealed 2026-06-18, recovered from session
//!    history and confirmed against GL-001's own "Forge2 (7-sector
//!    architecture)" glossary entry: `sector()` below.
//! 2. **Pipeline function** — what a particle's passage through that gate
//!    *does*, in the BeeMDM ETL sense. This is what "doc vs SVG" actually
//!    disagreed about. Architect's ruling (2026-07-07): **merge, not
//!    choose one** — `pipeline_function()` below combines both sources'
//!    verb for each gate rather than discarding either.
//!
//! | # | Akkadian | Sector (sealed 2026-06-18)  | Pipeline function (merged 2026-07-07)          |
//! |---|----------|------------------------------|--------------------------------------------------|
//! | 1 | APSU     | Storage                      | Identity & Security Intake                       |
//! | 2 | ADAD     | ETL / Ingestion              | Structural Validation                            |
//! | 3 | SHEDU    | Security                     | Comparative Enrichment                           |
//! | 4 | MUMMU    | Algebra                      | Schema Harmonization & Quarantine                |
//! | 5 | ENKIDU   | AI Agents                    | AI-Assisted Match/Dedupe & Steward Review        |
//! | 6 | DUBSAR   | Languages                    | Cleansing & Quality Scoring                      |
//! | 7 | ENLIL    | Governance                   | Governance & SLA Sign-off                        |
//!
//! Source lists being merged, for the record (GLS-001 §4):
//! - "Doc" column:  Security, Structure, Compare, BlackBox/Quarantine, Steward, Cleanse, Quality/SLA
//! - "SVG" column:  Identity, Validation, Enrichment, Schema Harmonization, Match/Dedupe, Quality Scoring, Governance
//!
//! Note this `pipeline_function` axis is deliberately independent from the
//! separate É-DUBBA BeeMDM stage chain (MASHSHARU→NĀMZITUM→PASHIRU→
//! KISPU→UTNAPISHTIM→NISABA→NERGAL_GATE, itself still under a three-way
//! documentation conflict of its own per the 2026-07-07 verification
//! batches) — that chain names the *stations* data physically visits;
//! this enum names the *sectors* those stations are governed by. Both are
//! real and both matter; they are not the same list and this module does
//! not claim to reconcile them.

#![allow(clippy::module_name_repetitions)]

/// One of the 7 Hepta Gates — the top-level sector/authority structure
/// every particle, engine, and document in BahyWay.Ecosystem answers to.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum HeptaGate {
    Apsu = 1,
    Adad = 2,
    Shedu = 3,
    Mummu = 4,
    Enkidu = 5,
    Dubsar = 6,
    Enlil = 7,
}

impl HeptaGate {
    pub fn all() -> [HeptaGate; 7] {
        [
            HeptaGate::Apsu,
            HeptaGate::Adad,
            HeptaGate::Shedu,
            HeptaGate::Mummu,
            HeptaGate::Enkidu,
            HeptaGate::Dubsar,
            HeptaGate::Enlil,
        ]
    }

    /// 1-indexed gate number (G1..G7), matching every source document's
    /// own numbering.
    pub fn gate_number(self) -> u8 {
        self as u8
    }

    /// The Akkadian/logical name — unchanged since the 2026-06-18 seal.
    pub fn akkadian_name(self) -> &'static str {
        match self {
            HeptaGate::Apsu => "APSU",
            HeptaGate::Adad => "ADAD",
            HeptaGate::Shedu => "SHEDU",
            HeptaGate::Mummu => "MUMMU",
            HeptaGate::Enkidu => "ENKIDU",
            HeptaGate::Dubsar => "DUBSAR",
            HeptaGate::Enlil => "ENLIL",
        }
    }

    /// The physical/functional sector name — sealed 2026-06-18: "Both are
    /// Perfect; the god name is the Logical name, the sector name is the
    /// physical (Functional) Name." Confirmed independently in GL-001's
    /// "Forge2 (7-sector architecture)" glossary entry.
    pub fn sector(self) -> &'static str {
        match self {
            HeptaGate::Apsu => "Storage",
            HeptaGate::Adad => "ETL / Ingestion",
            HeptaGate::Shedu => "Security",
            HeptaGate::Mummu => "Algebra",
            HeptaGate::Enkidu => "AI Agents",
            HeptaGate::Dubsar => "Languages",
            HeptaGate::Enlil => "Governance",
        }
    }

    /// The merged BeeMDM pipeline-step function — Architect's ruling,
    /// 2026-07-07: combine the "doc" and "SVG" verb for each gate rather
    /// than pick one. See the module-level table for both source columns.
    pub fn pipeline_function(self) -> &'static str {
        match self {
            HeptaGate::Apsu => "Identity & Security Intake",
            HeptaGate::Adad => "Structural Validation",
            HeptaGate::Shedu => "Comparative Enrichment",
            HeptaGate::Mummu => "Schema Harmonization & Quarantine",
            HeptaGate::Enkidu => "AI-Assisted Match/Dedupe & Steward Review",
            HeptaGate::Dubsar => "Cleansing & Quality Scoring",
            HeptaGate::Enlil => "Governance & SLA Sign-off",
        }
    }

    pub fn from_gate_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(HeptaGate::Apsu),
            2 => Some(HeptaGate::Adad),
            3 => Some(HeptaGate::Shedu),
            4 => Some(HeptaGate::Mummu),
            5 => Some(HeptaGate::Enkidu),
            6 => Some(HeptaGate::Dubsar),
            7 => Some(HeptaGate::Enlil),
            _ => None,
        }
    }

    pub fn from_akkadian_name(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "APSU" => Some(HeptaGate::Apsu),
            "ADAD" => Some(HeptaGate::Adad),
            "SHEDU" => Some(HeptaGate::Shedu),
            "MUMMU" => Some(HeptaGate::Mummu),
            "ENKIDU" => Some(HeptaGate::Enkidu),
            "DUBSAR" => Some(HeptaGate::Dubsar),
            "ENLIL" => Some(HeptaGate::Enlil),
            _ => None,
        }
    }
}

impl core::fmt::Display for HeptaGate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "G{} {} ({})", self.gate_number(), self.akkadian_name(), self.sector())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_seven_in_order() {
        let gates = HeptaGate::all();
        assert_eq!(gates.len(), 7);
        for (i, g) in gates.iter().enumerate() {
            assert_eq!(g.gate_number(), (i + 1) as u8);
        }
    }

    #[test]
    fn sector_mapping_matches_2026_06_18_seal() {
        assert_eq!(HeptaGate::Apsu.sector(), "Storage");
        assert_eq!(HeptaGate::Adad.sector(), "ETL / Ingestion");
        assert_eq!(HeptaGate::Shedu.sector(), "Security");
        assert_eq!(HeptaGate::Mummu.sector(), "Algebra");
        assert_eq!(HeptaGate::Enkidu.sector(), "AI Agents");
        assert_eq!(HeptaGate::Dubsar.sector(), "Languages");
        assert_eq!(HeptaGate::Enlil.sector(), "Governance");
    }

    #[test]
    fn pipeline_function_is_populated_for_every_gate() {
        for g in HeptaGate::all() {
            assert!(!g.pipeline_function().is_empty());
        }
    }

    #[test]
    fn round_trip_gate_number() {
        for g in HeptaGate::all() {
            assert_eq!(HeptaGate::from_gate_number(g.gate_number()), Some(g));
        }
        assert_eq!(HeptaGate::from_gate_number(0), None);
        assert_eq!(HeptaGate::from_gate_number(8), None);
    }

    #[test]
    fn round_trip_akkadian_name_case_insensitive() {
        assert_eq!(HeptaGate::from_akkadian_name("shedu"), Some(HeptaGate::Shedu));
        assert_eq!(HeptaGate::from_akkadian_name("ENLIL"), Some(HeptaGate::Enlil));
        assert_eq!(HeptaGate::from_akkadian_name("nope"), None);
    }

    #[test]
    fn display_format() {
        assert_eq!(HeptaGate::Adad.to_string(), "G2 ADAD (ETL / Ingestion)");
    }
}
