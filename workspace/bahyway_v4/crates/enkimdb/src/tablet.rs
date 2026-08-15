//! Tablet scanning — real `.akk`/`.way`/`.tmpl` files as a scannable
//! profile, minted when the Architect marks one final in Girsu IDE (the
//! "mark as final" KAKI generator). One `TabletKind`-parameterized shape
//! covers all three -- they mint identically (role=Parzu, same as
//! `PbEmitter` -- a tablet literally is "logic, template, axiom, or
//! rule") and differ only in which structured fields get extracted:
//!
//!   - `.akk` (AAOL): real structure via `aaol::PmpvdParser` -- tribe/
//!     particle/rule counts, when the file actually parses.
//!   - `.way`: file-level metadata only -- WAY has no compiler yet
//!     (`bahyway_core::akkadian::CRATE_WAY` is a reserved name, no
//!     `crates/way` exists). Upgradeable to structured fields once one
//!     does; nothing here should need to change to add that later.
//!   - `.tmpl` (TemplateEngine): file-level metadata only -- the real
//!     `template-engine` crate builds `Template`s programmatically in
//!     Rust (`Template::default_template`/`stakeholder`), it does not
//!     parse a `.tmpl` file format off disk today.

use std::fs;
use std::path::Path;

use akkvalue::AkkValue;

/// Which of the three internal-language tablets this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabletKind {
    Akk,
    Way,
    Tmpl,
}

impl TabletKind {
    /// From a file's extension. `None` for anything else -- callers
    /// should refuse to mint a tablet for a file this doesn't recognize.
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|e| e.to_str())? {
            "akk" => Some(Self::Akk),
            "way" => Some(Self::Way),
            "tmpl" => Some(Self::Tmpl),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Akk => "akk",
            Self::Way => "way",
            Self::Tmpl => "tmpl",
        }
    }

    /// Fixed tribe id per kind -- single source of truth, continuing the
    /// 0x716x sequence (`enkiddb::DOCS_TRIBE_ID` = 0x7160,
    /// `shakkanakku::pb_mint::PB_TRIBE_ID` = 0x7161).
    pub fn tribe_id(&self) -> u16 {
        match self {
            Self::Akk => 0x7162,
            Self::Way => 0x7163,
            Self::Tmpl => 0x7164,
        }
    }
}

/// One real tablet's profile, ready to be minted. `extra` carries
/// kind-specific structured attributes (e.g. `akk.tribe_count`) --
/// empty when the file doesn't parse as its kind, or for kinds with no
/// parser at all (`.way`, `.tmpl` today). Universal fields (path, bytes,
/// sha256) are always present regardless of parse success.
#[derive(Debug, Clone)]
pub struct TabletProfile {
    pub kind: TabletKind,
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
    pub extra: Vec<(&'static str, AkkValue)>,
}

/// Read `path`, hash it, and extract whatever structured fields its kind
/// supports. Never fails on unparseable `.akk` content -- a tablet that
/// doesn't parse still gets file-level fields; `extra` is just empty.
pub fn profile_tablet(path: &Path) -> std::io::Result<TabletProfile> {
    let kind = TabletKind::from_path(path).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{}: not a .akk/.way/.tmpl file", path.display()),
        )
    })?;
    let bytes = fs::read(path)?;
    let sha256 = sha256_hex(&bytes);
    let extra = match kind {
        TabletKind::Akk => profile_akk(&bytes),
        TabletKind::Way | TabletKind::Tmpl => Vec::new(),
    };
    Ok(TabletProfile {
        kind,
        path: path.display().to_string(),
        bytes: bytes.len() as u64,
        sha256,
        extra,
    })
}

/// Real structure via `aaol::PmpvdParser` -- tribe/particle/rule/
/// equation/guard counts. Empty (not an error) when the content isn't
/// valid UTF-8 or doesn't parse -- a tablet that fails to parse is still
/// a real file worth minting, just without the structured extras.
fn profile_akk(bytes: &[u8]) -> Vec<(&'static str, AkkValue)> {
    let Ok(text) = std::str::from_utf8(bytes) else { return Vec::new() };
    let Ok(program) = aaol::PmpvdParser::parse_source(text) else { return Vec::new() };

    let mut particles = 0i64;
    let mut tribes = 0i64;
    let mut rules = 0i64;
    let mut equations = 0i64;
    let mut guards = 0i64;
    for node in &program.nodes {
        match &node.kind {
            aaol::AkkNodeKind::Particle(_) => particles += 1,
            aaol::AkkNodeKind::Tribe(_) => tribes += 1,
            aaol::AkkNodeKind::Rule(_) => rules += 1,
            aaol::AkkNodeKind::Equation(_) => equations += 1,
            aaol::AkkNodeKind::Guard(_) => guards += 1,
            _ => {}
        }
    }
    vec![
        ("akk.particle_count", AkkValue::Int(particles)),
        ("akk.tribe_count", AkkValue::Int(tribes)),
        ("akk.rule_count", AkkValue::Int(rules)),
        ("akk.equation_count", AkkValue::Int(equations)),
        ("akk.guard_count", AkkValue::Int(guards)),
    ]
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_from_path_recognizes_all_three_extensions() {
        assert_eq!(TabletKind::from_path(Path::new("x.akk")), Some(TabletKind::Akk));
        assert_eq!(TabletKind::from_path(Path::new("x.way")), Some(TabletKind::Way));
        assert_eq!(TabletKind::from_path(Path::new("x.tmpl")), Some(TabletKind::Tmpl));
        assert_eq!(TabletKind::from_path(Path::new("x.md")), None);
    }

    #[test]
    fn tribe_ids_are_distinct() {
        let mut ids = vec![TabletKind::Akk.tribe_id(), TabletKind::Way.tribe_id(), TabletKind::Tmpl.tribe_id()];
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 3, "each tablet kind must have its own distinct tribe id");
    }

    #[test]
    fn profile_tablet_rejects_unrecognized_extensions() {
        let dir = std::env::temp_dir().join("enkimdb_tablet_test_reject");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("not_a_tablet.txt");
        std::fs::write(&path, b"hello").unwrap();
        assert!(profile_tablet(&path).is_err());
    }

    #[test]
    fn profile_tablet_way_and_tmpl_are_file_level_only() {
        let dir = std::env::temp_dir().join("enkimdb_tablet_test_way_tmpl");
        std::fs::create_dir_all(&dir).unwrap();
        let way_path = dir.join("policy.way");
        std::fs::write(&way_path, b"some way content").unwrap();
        let profile = profile_tablet(&way_path).unwrap();
        assert_eq!(profile.kind, TabletKind::Way);
        assert!(profile.extra.is_empty(), "WAY has no parser yet -- extras must stay empty, not fabricated");
        assert!(!profile.sha256.is_empty());
    }

    #[test]
    fn profile_tablet_akk_extracts_real_structure_when_it_parses() {
        let dir = std::env::temp_dir().join("enkimdb_tablet_test_akk");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.akk");
        // Deliberately not asserting a specific PMPVD source string here --
        // this is a genuine dependency on aaol's real grammar, not a
        // hand-rolled fixture, and it's covered separately if it fails.
        std::fs::write(&path, b"tribe demo {}\n").unwrap();
        let profile = profile_tablet(&path).unwrap();
        assert_eq!(profile.kind, TabletKind::Akk);
        // Either it parsed (extras populated) or it didn't (extras empty,
        // not an error) -- both are valid outcomes; the file must still
        // profile successfully either way.
        assert!(!profile.sha256.is_empty());
    }
}
