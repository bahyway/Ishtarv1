//! Akkadian ecosystem name constants — canonical v4.0 crate identity map.
//!
//! Every BahyWay v4.0 crate has an Akkadian identity name that reflects its
//! purpose semantically.  These constants are the single source of truth for
//! the naming convention sealed in the Nabu Notebook document (2026-05).

/// Akkadian name → purpose constants for the 13 principal crates.
///
/// These are declared as `&str` constants rather than an enum so downstream
/// crates can reference individual names without pulling in the full map.

/// Nabu (𒀭𒀜) — god of scribes, wisdom, and writing.
/// Role: AkkadianAOL v4.0 compiler + Notebook engine (`.akk` files).
pub const CRATE_NABU: &str = "nabu";

/// Zakāru (𒍣𒆪) — "to name, to declare, to speak."
/// Role: Sovereign event log / StoryWay journal.
pub const CRATE_ZAKARU: &str = "zakaru";

/// Lamassu (𒀭𒆳𒍪) — protective spirit / sovereign guardian.
/// Role: ZeroWay runtime enforcement (replaces zeroway-v353).
pub const CRATE_LAMASSU: &str = "lamassu";

/// Ištar (𒀭𒈹) — goddess of war and love.
/// Role: ABAC policy engine (replaces istar-wayv353).
pub const CRATE_ISHTAR: &str = "ishtar";

/// Isdu (𒅖𒁺) — "foundation, pillar."
/// Role: Core data-structure library (replaces data-structure-wayv353).
pub const CRATE_ISDU: &str = "isdu";

/// Ebbēru (𒂗𒁉) — "to cross over, to purify."
/// Role: Data cleansing engine (replaces cleansing-wayv353).
pub const CRATE_EBBERU: &str = "ebberu";

/// Kakkabu (𒆳𒆪𒁀) — "star, celestial body."
/// Role: Orbital / particle visualizer (replaces pollution-wayv353).
pub const CRATE_KAKKABU: &str = "kakkabu";

/// Tēmu (𒋾𒄠) — "report, intelligence, AI knowledge."
/// Role: MetaWay AI documentation agent (replaces metawayv353).
pub const CRATE_TEMU: &str = "temu";

/// Šēdu (𒊺𒁺) — "protective spirit / guardian of the vault."
/// Role: Sovereign storage vault (replaces vaultway-v353).
pub const CRATE_SHEDU: &str = "shedu";

/// Quppu — "cage, secure container."
/// Role: Credential store (replaces quppu-wayv353).
pub const CRATE_QUPPU: &str = "quppu";

/// Kupru — "tar, pitch, sealing compound."
/// Role: Cipher / cryptographic layer (replaces kupru-wayv353).
pub const CRATE_KUPRU: &str = "kupru";

/// HeptaScript — "seven-dimensional query language."
/// Role: `.hepta` query language compiler (placeholder until post-demo).
pub const CRATE_HEPTASCRIPT: &str = "heptascript";

/// Way — "the path, sovereign security language."
/// Role: `.way` security/policy language compiler (placeholder until post-demo).
pub const CRATE_WAY: &str = "way";

/// File extensions — locked per collision check (2026-05).

/// AkkadianAOL / Nabu source files.
pub const EXT_AKK: &str = ".akk";
/// HeptaScript query files (`.hepta` — not `.hep`, which collides with IBM/Brother tooling).
pub const EXT_HEPTA: &str = ".hepta";
/// Way security/policy declaration files.
pub const EXT_WAY: &str = ".way";
/// Nabu Notebook files (typed fences, 10 cell kinds).
pub const EXT_AKKNB: &str = ".akknb";

/// Canonical slice of all ecosystem crate names, in dependency order
/// (lower-layer first).
pub const ECOSYSTEM_CRATES: &[&str] = &[
    CRATE_ISDU,
    CRATE_KUPRU,
    CRATE_QUPPU,
    CRATE_ISHTAR,
    CRATE_LAMASSU,
    CRATE_EBBERU,
    CRATE_ZAKARU,
    CRATE_KAKKABU,
    CRATE_SHEDU,
    CRATE_NABU,
    CRATE_TEMU,
    CRATE_HEPTASCRIPT,
    CRATE_WAY,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_empty_names() {
        for name in ECOSYSTEM_CRATES {
            assert!(!name.is_empty(), "empty crate name in ECOSYSTEM_CRATES");
        }
    }

    #[test]
    fn no_duplicate_names() {
        let mut seen = std::collections::HashSet::new();
        for name in ECOSYSTEM_CRATES {
            assert!(seen.insert(*name), "duplicate crate name: {name}");
        }
    }

    #[test]
    fn extensions_start_with_dot() {
        for ext in [EXT_AKK, EXT_HEPTA, EXT_WAY, EXT_AKKNB] {
            assert!(ext.starts_with('.'), "extension missing dot: {ext}");
        }
    }

    #[test]
    fn hepta_extension_not_hep() {
        assert_eq!(EXT_HEPTA, ".hepta");
        assert_ne!(EXT_HEPTA, ".hep");
    }

    #[test]
    fn principal_names_correct() {
        assert_eq!(CRATE_NABU,       "nabu");
        assert_eq!(CRATE_ZAKARU,     "zakaru");
        assert_eq!(CRATE_LAMASSU,    "lamassu");
        assert_eq!(CRATE_ISHTAR,     "ishtar");
        assert_eq!(CRATE_ISDU,       "isdu");
        assert_eq!(CRATE_EBBERU,     "ebberu");
        assert_eq!(CRATE_KAKKABU,    "kakkabu");
        assert_eq!(CRATE_TEMU,       "temu");
        assert_eq!(CRATE_SHEDU,      "shedu");
        assert_eq!(CRATE_QUPPU,      "quppu");
        assert_eq!(CRATE_KUPRU,      "kupru");
        assert_eq!(CRATE_HEPTASCRIPT,"heptascript");
        assert_eq!(CRATE_WAY,        "way");
    }

    #[test]
    fn ecosystem_count() {
        assert_eq!(ECOSYSTEM_CRATES.len(), 13);
    }
}
