//! Generation — a named, versioned Read Node materialization.
//!
//! `materialize()` (see `crate::materialize`) requires a fresh base path
//! per call, because `enkidb-datafile`'s writer always appends and never
//! truncates — calling it twice at the same path silently duplicates
//! every entity. This module turns that constraint into a real, checkable
//! naming scheme instead of leaving it as a caller convention to
//! remember: every materialization gets a `(sovereign_name, version)`
//! pair (e.g. `("tigris", "4.0")`, `("euphrates", "4.2")`) and lands at
//! `<root>/<sovereign_name>/<version>/{entities,eav}` — deterministic,
//! collision-proof by construction (two calls with the same pair always
//! resolve to the same path, so accidentally re-materializing the same
//! generation is now visible and diffable, not silently duplicated data),
//! and listable, so a caller (DubSar Theater's version picker, for
//! instance) can enumerate what generations exist without tracking paths
//! itself.
//!
//! `sovereign_name` is a plain namespace string — this crate does not
//! hardcode "tigris"/"euphrates"/anything else. Real crate/type names
//! (`enkiddb`, `enkimdb`, EnkiDDB, EnkiMDB) are unaffected; this is a
//! labeling and versioning layer on top, the same relationship "GeoEngine"
//! has to `bahyway-algebra`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use enkidb_journal::Journal;

use crate::materialize::{materialize, MaterializeStats};
use crate::readnode::ReadNode;

/// One materialized generation: a sovereign name (e.g. "tigris"), a
/// version string (e.g. "4.0"), and the Data File paths it lives at.
#[derive(Debug, Clone)]
pub struct Generation {
    pub sovereign_name: String,
    pub version: String,
    pub entities_path: PathBuf,
    pub eav_path: PathBuf,
}

impl Generation {
    fn at(root: &Path, sovereign_name: &str, version: &str) -> Self {
        let dir = root.join(sovereign_name).join(version);
        Generation {
            sovereign_name: sovereign_name.to_string(),
            version: version.to_string(),
            entities_path: dir.join("entities"),
            eav_path: dir.join("eav"),
        }
    }

    /// Open this generation's Data Files as a Read Node. Fails if this
    /// generation has never been materialized (see `materialize_generation`).
    pub fn open(&self) -> io::Result<ReadNode> {
        ReadNode::open(&self.entities_path, &self.eav_path)
    }
}

/// Materialize `journal`'s current state as generation `(sovereign_name,
/// version)` under `root`. Deterministic path — calling this again with
/// the *same* `(sovereign_name, version)` re-materializes into the exact
/// same location, so a caller who does that on purpose (rebuilding a
/// generation after fixing a bad write) gets the expected duplication
/// warning baked into `enkidb-datafile`'s append-only behavior, rather
/// than a silent surprise at a path they chose ad hoc.
pub fn materialize_generation(
    journal: &Journal,
    root: impl AsRef<Path>,
    sovereign_name: &str,
    version: &str,
) -> io::Result<(Generation, MaterializeStats)> {
    let root = root.as_ref();
    fs::create_dir_all(root.join(sovereign_name).join(version))?;
    let gen = Generation::at(root, sovereign_name, version);
    let stats = materialize(journal, &gen.entities_path, &gen.eav_path)?;
    Ok((gen, stats))
}

/// List every version materialized so far under `sovereign_name` (e.g.
/// `list_generations(root, "euphrates")` -> `["4.0", "4.1", "4.2"]`,
/// sorted). Empty if the sovereign name has no generations yet — not an
/// error, since "nothing materialized yet" is a normal starting state.
pub fn list_generations(root: impl AsRef<Path>, sovereign_name: &str) -> io::Result<Vec<String>> {
    let dir = root.as_ref().join(sovereign_name);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut versions: Vec<String> = fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    versions.sort();
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_kaki::{mint::KakiMinter, EventKaki, IdentityKaki, KakiRole};
    use std::fs;

    fn tmp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkidb_readnode_generation_{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn journal_with_one_entry(tribe: u16, title: &str) -> Journal {
        let mut jnl = Journal::new(64);
        let m = KakiMinter::new(TribeId::from_u16(tribe));
        let target = IdentityKaki::try_from_kaki(m.identity(KakiRole::Kishib)).unwrap();
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav = vec![EavTriple::new(
            bahyway_crc::crc16(b"meta.title") as u32,
            akkvalue::codec::encode(&akkvalue::AkkValue::Text(title.to_string())),
        )];
        jnl.append(JournalEntry::new(ek, target, 1, eav)).unwrap();
        jnl
    }

    #[test]
    fn two_versions_of_the_same_sovereign_name_are_independently_addressable() {
        let root = tmp_root("two_versions");

        let jnl_v40 = journal_with_one_entry(0xFF20, "v4.0 doc");
        let (gen_v40, stats_v40) = materialize_generation(&jnl_v40, &root, "euphrates", "4.1").unwrap();
        assert_eq!(stats_v40.entities, 1);

        // v4.2's journal has two documents, proving it is a genuinely
        // distinct dataset from v4.1's, not the same journal reused.
        let mut jnl_v42 = journal_with_one_entry(0xFF22, "v4.2 doc a");
        let m = KakiMinter::new(TribeId::from_u16(0xFF22));
        let target2 = IdentityKaki::try_from_kaki(m.identity(KakiRole::Kishib)).unwrap();
        let ek2 = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        jnl_v42.append(JournalEntry::new(
            ek2, target2, 2,
            vec![EavTriple::new(
                bahyway_crc::crc16(b"meta.title") as u32,
                akkvalue::codec::encode(&akkvalue::AkkValue::Text("v4.2 doc b".to_string())),
            )],
        )).unwrap();

        let (gen_v42, stats_v42) = materialize_generation(&jnl_v42, &root, "euphrates", "4.2").unwrap();
        assert_eq!(stats_v42.entities, 2);

        // Both generations are independently openable and disagree in entity count,
        // proving they are genuinely separate datasets, not the same path twice.
        let mut rn_v40 = gen_v40.open().unwrap();
        let mut rn_v42 = gen_v42.open().unwrap();
        assert_eq!(rn_v40.entity_count(), 1);
        assert_eq!(rn_v42.entity_count(), 2);

        let res_v40 = rn_v40.query("WHO T.E\nWHERE E[meta.title] = \"v4.0 doc\"").unwrap();
        assert_eq!(res_v40.matched.len(), 1);
        let res_v42 = rn_v42.query("WHO T.E\nWHERE E[meta.title] = \"v4.0 doc\"").unwrap();
        assert_eq!(res_v42.matched.len(), 0, "v4.2's generation must not see v4.0's documents");
    }

    #[test]
    fn list_generations_enumerates_materialized_versions_sorted() {
        let root = tmp_root("list_generations");
        let jnl = journal_with_one_entry(0xFF23, "doc");

        materialize_generation(&jnl, &root, "tigris", "4.1").unwrap();
        materialize_generation(&jnl, &root, "tigris", "4.0").unwrap();
        materialize_generation(&jnl, &root, "tigris", "4.2").unwrap();

        let versions = list_generations(&root, "tigris").unwrap();
        assert_eq!(versions, vec!["4.0", "4.1", "4.2"]);
    }

    #[test]
    fn list_generations_on_an_unmaterialized_sovereign_name_is_empty_not_an_error() {
        let root = tmp_root("empty_list");
        let versions = list_generations(&root, "nonexistent").unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn different_sovereign_names_never_collide_under_the_same_root() {
        let root = tmp_root("no_cross_collision");
        let jnl_a = journal_with_one_entry(0xFF24, "tigris doc");
        let jnl_b = journal_with_one_entry(0xFF25, "euphrates doc");

        materialize_generation(&jnl_a, &root, "tigris", "4.0").unwrap();
        materialize_generation(&jnl_b, &root, "euphrates", "4.0").unwrap();

        assert_eq!(list_generations(&root, "tigris").unwrap(), vec!["4.0"]);
        assert_eq!(list_generations(&root, "euphrates").unwrap(), vec!["4.0"]);
    }
}
