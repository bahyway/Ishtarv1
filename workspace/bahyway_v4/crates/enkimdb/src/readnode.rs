//! ReadNode — EnkiMDB's read path: materialized Data Files, no journal.
//!
//! Identical shape to `enkiddb::readnode` — thin wrapper over the real
//! `enkidb-readnode` crate (ADR-012). See that module's doc comments for
//! the full rationale; repeated only where it matters operationally here:
//! `materialize_now` must be called after new writes to refresh the Read
//! Node's view, and each materialization generation needs a *fresh* base
//! path — `enkidb_datafile::DataFileWriter::open` always appends, never
//! truncates, so re-materializing to the same path duplicates records
//! (proven by test in `enkiddb::readnode`; not re-proven here to avoid
//! duplicating that test, since it is a property of the shared
//! `enkidb-datafile` crate, not of this one).

use std::io;
use std::path::Path;

pub use enkidb_readnode::cached::{CachedReadNode, CachedReadNodeError};
pub use enkidb_readnode::generation::Generation;
pub use enkidb_readnode::materialize::MaterializeStats;
pub use enkidb_readnode::readnode::{ReadNode, ReadNodeError};

use crate::writenode::WriteNode;

/// EnkiMDB's sovereign name — Euphrates, per the Architect's naming
/// (2026-07-12). See `enkiddb::readnode::SOVEREIGN_NAME` for the same
/// pattern applied to EnkiDDB/Tigris; this does not rename the crate or
/// the EnkiMDB type.
pub const SOVEREIGN_NAME: &str = "euphrates";

/// Materialize `write_node`'s current Journal state into Data Files at
/// `entities_base` / `eav_index_base`. Prefer [`materialize_version`] —
/// it builds a fresh, collision-proof path per version automatically.
pub fn materialize_now(
    write_node: &WriteNode,
    entities_base: impl AsRef<Path>,
    eav_index_base: impl AsRef<Path>,
) -> io::Result<MaterializeStats> {
    enkidb_readnode::materialize::materialize(write_node.journal(), entities_base, eav_index_base)
}

/// Materialize `write_node` as a named, versioned Euphrates generation
/// under `root` (e.g. `root/euphrates/4.2/{entities,eav}`) — the entry
/// point DubSar Theater's version picker should use.
pub fn materialize_version(
    write_node: &WriteNode,
    root: impl AsRef<Path>,
    version: &str,
) -> io::Result<(Generation, MaterializeStats)> {
    enkidb_readnode::generation::materialize_generation(
        write_node.journal(),
        root,
        SOVEREIGN_NAME,
        version,
    )
}

/// Every Euphrates version materialized so far under `root`, sorted.
pub fn list_versions(root: impl AsRef<Path>) -> io::Result<Vec<String>> {
    enkidb_readnode::generation::list_generations(root, SOVEREIGN_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactKind;
    use crate::artifact::ArtifactProfile;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;
    use std::fs;
    use std::path::PathBuf;

    fn tmp_base(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkimdb_readnode_{name}"));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        dir.join("base")
    }

    fn profile(name: &str) -> ArtifactProfile {
        ArtifactProfile {
            name: name.to_string(),
            kind: ArtifactKind::Crate,
            path: format!("crates/{name}"),
            version: Some("4.0.2".to_string()),
        }
    }

    fn tmp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkimdb_readnode_versioned_{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn euphrates_versions_are_listable_and_independently_comparable() {
        let root = tmp_root("compare");

        let mut wn_v41 = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF12)), 64);
        wn_v41.ingest_artifact(&profile("bahyway-algebra"), 1);
        let (gen_v41, _) = materialize_version(&wn_v41, &root, "4.1").unwrap();

        let mut wn_v42 = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF13)), 64);
        wn_v42.ingest_artifact(&profile("bahyway-algebra"), 1);
        wn_v42.ingest_artifact(&profile("algebra-arsenal"), 2);
        let (gen_v42, _) = materialize_version(&wn_v42, &root, "4.2").unwrap();

        assert_eq!(SOVEREIGN_NAME, "euphrates");
        assert_eq!(list_versions(&root).unwrap(), vec!["4.1", "4.2"]);

        let mut rn_v41 = gen_v41.open().unwrap();
        let mut rn_v42 = gen_v42.open().unwrap();
        assert_eq!(
            rn_v41.entity_count(),
            1,
            "Euphrates v4.1 catalogs one artifact"
        );
        assert_eq!(
            rn_v42.entity_count(),
            2,
            "Euphrates v4.2 has grown to two artifacts"
        );

        let only_in_v42 = rn_v42
            .query("WHO T.E\nWHERE E[artifact.name] = \"algebra-arsenal\"")
            .unwrap();
        assert_eq!(only_in_v42.matched.len(), 1);
        let absent_from_v41 = rn_v41
            .query("WHO T.E\nWHERE E[artifact.name] = \"algebra-arsenal\"")
            .unwrap();
        assert_eq!(
            absent_from_v41.matched.len(),
            0,
            "algebra-arsenal postdates Euphrates v4.1's generation"
        );
    }

    #[test]
    fn two_node_round_trip_write_then_read() {
        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF03)), 64);
        let kaki = wn.ingest_artifact(&profile("bahyway-algebra"), 1);

        let base = tmp_base("round_trip");
        let stats = materialize_now(
            &wn,
            base.with_file_name("entities"),
            base.with_file_name("eav"),
        )
        .unwrap();
        assert_eq!(stats.entities, 1);

        let mut rn =
            ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        assert_eq!(rn.entity_count(), 1);

        let res = rn
            .query("WHO T.E\nWHERE E[artifact.name] = \"bahyway-algebra\"")
            .unwrap();
        assert_eq!(res.matched.len(), 1);
        assert_eq!(*res.matched[0].entity.bytes(), *kaki.bytes());
    }

    #[test]
    fn read_node_sees_nothing_before_materialization() {
        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF04)), 64);
        wn.ingest_artifact(&profile("kinetic-engine"), 1);
        assert_eq!(wn.artifact_count(), 1);

        let base = tmp_base("before_materialize");
        let opened = ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav"));
        assert!(opened.is_err());
    }
}
