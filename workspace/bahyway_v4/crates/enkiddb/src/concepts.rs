//! concepts — a real, deterministic concept registry + text scanner.
//!
//! Granted 2026-07-19, foundation for NISABA's access-grant exposure
//! preview: before the Architect grants a stakeholder group access to a
//! `meta.collection` (e.g. documentation about a BeeMDM gate), this is
//! what lets NISABA show which OTHER internal concepts a document in
//! that collection actually mentions — the "what sequence of impact
//! would this reveal" question, answered from real text, not a guess.
//!
//! Pure substring/word-boundary matching, no regex crate, no NLP —
//! consistent with "no tree in the Orbits." A concept "mentions" a
//! document if its name appears as a standalone word (not a substring of
//! a longer word) anywhere in the document's title, headers, body, or
//! code.

/// What kind of internal concept a [`ConceptEntry`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConceptKind {
    /// One of BeeMDM's Hepta gates (G1..G7).
    Gate,
    /// A workspace crate name.
    Crate,
    /// A sovereign name (Tigris, Euphrates, IRKALLA, ...).
    SovereignName,
}

impl ConceptKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConceptKind::Gate => "gate",
            ConceptKind::Crate => "crate",
            ConceptKind::SovereignName => "sovereign-name",
        }
    }
}

/// One registered internal concept a document's text can mention.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConceptEntry {
    pub name: String,
    pub kind: ConceptKind,
}

/// The set of internal concepts the scanner looks for. Built by the
/// caller — this crate does not hardcode a full crate list itself (that
/// would create an unwanted dependency on `enkimdb`'s catalog); see
/// [`ConceptRegistry::with_known_gates`] for the 7 sealed Hepta Gate names
/// this codebase actually confirms today.
#[derive(Debug, Clone, Default)]
pub struct ConceptRegistry {
    entries: Vec<ConceptEntry>,
}

impl ConceptRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_entry(mut self, name: impl Into<String>, kind: ConceptKind) -> Self {
        self.entries.push(ConceptEntry {
            name: name.into(),
            kind,
        });
        self
    }

    /// Seed from a real crate-name list — e.g. EnkiMDB's own
    /// `scan_crates` output, kept as the caller's responsibility so this
    /// crate never has to depend on `enkimdb` just to build a registry.
    pub fn extend_crates(mut self, crate_names: impl IntoIterator<Item = String>) -> Self {
        for name in crate_names {
            self.entries.push(ConceptEntry {
                name,
                kind: ConceptKind::Crate,
            });
        }
        self
    }

    pub fn extend_sovereign_names(mut self, names: impl IntoIterator<Item = String>) -> Self {
        for name in names {
            self.entries.push(ConceptEntry {
                name,
                kind: ConceptKind::SovereignName,
            });
        }
        self
    }

    /// All 7 Hepta Gates, from the sealed `bahyway_core::hepta_gate::
    /// HeptaGate` ruling (GATE-1, 2026-06-18/07-07) — the merge of the
    /// prior conflicting "doc" and "SVG" gate-name lists into one sealed
    /// source. Corrected 2026-07-19: an earlier version of this function
    /// used `bee_mdm_bus::GateLoads::g1_adad`/`g7_nergal` as the source of
    /// truth for two gates only (Adad at G1, Nergal at G7) — those
    /// accessor names predate the sealed ruling and are themselves wrong
    /// (they match neither this enum's numbering, where Adad is G2, nor
    /// any gate is named Nergal at all) and are unused everywhere else in
    /// the codebase. This now scans for all 7 real, sealed gate names.
    pub fn with_known_gates(self) -> Self {
        bahyway_core::hepta_gate::HeptaGate::all()
            .into_iter()
            .fold(self, |registry, gate| {
                registry.with_entry(gate.akkadian_name(), ConceptKind::Gate)
            })
    }

    /// Every sovereign name this Ecosystem has actually sealed for one of
    /// its components — the same "give the real crate a display name
    /// without renaming it" pattern used throughout (see
    /// `enkiddb::readnode::SOVEREIGN_NAME`, `enkimdb::readnode::
    /// SOVEREIGN_NAME`, `storage-sector::SOVEREIGN_NAME`,
    /// `enkidb-indexes::SOVEREIGN_NAME`, `bahyway-fabric::NUZI`) plus one
    /// Architect-confirmed name (2026-07-19) that does not yet have its
    /// own coded constant anywhere: **Nergal** — "my Internal Anti-Virus
    /// and the Shelled Defender for the Firewall of www.bahyway.com" —
    /// the sovereign AV/firewall engine, distinct from IRKALLA (the jail
    /// it feeds) and from any of the 7 Hepta Gates (see
    /// `with_known_gates`'s doc comment for the earlier mistake of
    /// treating "Nergal" as a gate name). Same status IRKALLA had before
    /// PB-188 gave it a real `SOVEREIGN_NAME` const: a real, deliberately
    /// chosen name with live UI usage (`dubsar-visualizer::panels::nergal`),
    /// not yet backed by its own coded identity — a natural next step if
    /// the Architect wants that pattern applied here too.
    pub fn with_known_sovereign_names(self) -> Self {
        self.with_entry("Tigris", ConceptKind::SovereignName)
            .with_entry("Euphrates", ConceptKind::SovereignName)
            .with_entry("Anu", ConceptKind::SovereignName)
            .with_entry("IRKALLA", ConceptKind::SovereignName)
            .with_entry("NUZI", ConceptKind::SovereignName)
            .with_entry("Nergal", ConceptKind::SovereignName)
    }

    pub fn entries(&self) -> &[ConceptEntry] {
        &self.entries
    }

    /// Scan `text` for whole-word, case-insensitive mentions of every
    /// registered concept. Deterministic: the same text always yields the
    /// same mentions, in registry order, each concept at most once.
    pub fn scan_mentions(&self, text: &str) -> Vec<ConceptEntry> {
        let lower = text.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| contains_word(&lower, &entry.name.to_lowercase()))
            .cloned()
            .collect()
    }
}

/// Whether `needle` appears in `haystack` as a standalone word — not
/// preceded or followed by another alphanumeric/`_`/`-` byte. Both
/// arguments are expected lowercase already (callers control that).
fn contains_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let hbytes = haystack.as_bytes();
    for (idx, _) in haystack.match_indices(needle) {
        let before_ok = idx == 0 || !is_word_byte(hbytes[idx - 1]);
        let after_idx = idx + needle.len();
        let after_ok = after_idx >= hbytes.len() || !is_word_byte(hbytes[after_idx]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> ConceptRegistry {
        ConceptRegistry::new()
            .with_known_gates()
            .with_known_sovereign_names()
            .extend_crates(["enkiddb".to_string(), "enkimdb".to_string()])
    }

    #[test]
    fn finds_a_whole_word_mention_case_insensitively() {
        let hits = registry().scan_mentions("The Enlil gate governs SLA sign-off.");
        assert_eq!(
            hits,
            vec![ConceptEntry {
                name: "ENLIL".into(),
                kind: ConceptKind::Gate
            }]
        );
    }

    #[test]
    fn does_not_match_a_substring_of_a_longer_word() {
        // "irkallas" should not count as a mention of "IRKALLA".
        let hits = registry().scan_mentions("The irkallastation was renamed.");
        assert!(hits.is_empty());
    }

    #[test]
    fn finds_all_seven_sealed_hepta_gate_names() {
        // Every name from bahyway_core::hepta_gate::HeptaGate (the sealed
        // 2026-06-18/07-07 GATE-1 ruling), one mention each.
        let text = "APSU intake, ADAD validation, SHEDU enrichment, MUMMU harmonization, \
                     ENKIDU review, DUBSAR cleansing, ENLIL governance.";
        let hits = registry().scan_mentions(text);
        let names: Vec<&str> = hits.iter().map(|c| c.name.as_str()).collect();
        for expected in [
            "APSU", "ADAD", "SHEDU", "MUMMU", "ENKIDU", "DUBSAR", "ENLIL",
        ] {
            assert!(
                names.contains(&expected),
                "missing sealed gate name {expected}"
            );
        }
        assert!(hits.iter().all(|c| c.kind == ConceptKind::Gate));
    }

    #[test]
    fn finds_all_known_sovereign_names_including_nergal() {
        // Nergal is the sovereign AV/firewall defender (Architect-
        // confirmed 2026-07-19) — a SovereignName, never a Hepta Gate.
        let text =
            "Tigris, Euphrates, Anu, IRKALLA, NUZI, and Nergal are all real sovereign names.";
        let hits = registry().scan_mentions(text);
        let names: Vec<&str> = hits.iter().map(|c| c.name.as_str()).collect();
        for expected in ["Tigris", "Euphrates", "Anu", "IRKALLA", "NUZI", "Nergal"] {
            assert!(
                names.contains(&expected),
                "missing sovereign name {expected}"
            );
        }
        assert!(hits.iter().all(|c| c.kind == ConceptKind::SovereignName));
    }

    #[test]
    fn finds_multiple_distinct_concepts_in_one_document() {
        let text = "EnkiDDB (Tigris) ingests documents; EnkiMDB (Euphrates) catalogs crates. \
                     Governance happens at the Enlil gate, tribe IRKALLA.";
        let hits = registry().scan_mentions(text);
        let names: Vec<&str> = hits.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"ENLIL"));
        assert!(names.contains(&"enkiddb"));
        assert!(names.contains(&"enkimdb"));
        assert!(names.contains(&"Tigris"));
        assert!(names.contains(&"Euphrates"));
        assert!(names.contains(&"IRKALLA"));
        assert_eq!(
            hits.len(),
            6,
            "Enlil among the 7 gates, plus 5 non-gate concepts, are mentioned"
        );
    }

    #[test]
    fn no_mentions_returns_empty() {
        let hits = registry().scan_mentions("Nothing relevant here.");
        assert!(hits.is_empty());
    }

    #[test]
    fn each_concept_reported_at_most_once_even_if_mentioned_repeatedly() {
        let hits = registry().scan_mentions("Enlil. Enlil again. Still Enlil.");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn scan_is_deterministic_across_repeated_calls() {
        let text = "Tigris and IRKALLA and Enlil, together.";
        let a = registry().scan_mentions(text);
        let b = registry().scan_mentions(text);
        assert_eq!(a, b);
    }
}
