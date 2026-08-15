//! ReadNode — EnkiDDB's read path: materialized Data Files, no journal.
//!
//! Thin wrapper over the real `enkidb-readnode` crate (ADR-012), the same
//! Read Node machinery already proven for EnkiDB itself (port 7001).
//! `materialize_now` performs the one-time O(n) pass over a `WriteNode`'s
//! Journal into two Data Files; `ReadNode::open` afterward is O(1) and
//! every query thereafter touches only the entities it matches — no
//! journal replay, matching the "Read as Data files, Write as Journal"
//! CQRS split.

use std::io;
use std::path::Path;

pub use enkidb_readnode::generation::Generation;
pub use enkidb_readnode::materialize::MaterializeStats;
pub use enkidb_readnode::readnode::{ReadNode, ReadNodeError};

use crate::writenode::WriteNode;

/// EnkiDDB's sovereign name — Tigris, per the Architect's naming
/// (2026-07-12). Not a crate/type rename: `enkiddb`/`EnkiDDB` are
/// unchanged identifiers; this is the display/versioning namespace
/// `materialize_version`/`list_versions` use under the hood, the same
/// relationship "GeoEngine" has to `bahyway-algebra`.
pub const SOVEREIGN_NAME: &str = "tigris";

/// Materialize `write_node`'s current Journal state into Data Files at
/// `entities_base` / `eav_index_base`. Call this whenever the Read Node's
/// view needs to catch up with new writes — it is not automatic, exactly
/// like EnkiDB's own Read Node (ADR-012 never re-materializes implicitly).
///
/// **Real constraint, discovered and proven by test in this module:**
/// `enkidb_datafile::DataFileWriter::open` always opens in append mode —
/// it never truncates. Calling this twice at the *same* path does not
/// refresh the Read Node's view; it appends a second, duplicate copy of
/// every entity. Each materialization generation needs a fresh base path.
/// Prefer [`materialize_version`] over this function directly — it builds
/// that fresh path for you and makes the result listable and comparable
/// (e.g. "Tigris v4.0" vs "Tigris v4.1" in DubSar Theater's version
/// picker) instead of leaving path management to the caller.
pub fn materialize_now(
    write_node: &WriteNode,
    entities_base: impl AsRef<Path>,
    eav_index_base: impl AsRef<Path>,
) -> io::Result<MaterializeStats> {
    enkidb_readnode::materialize::materialize(write_node.journal(), entities_base, eav_index_base)
}

/// Materialize `write_node` as a named, versioned Tigris generation under
/// `root` (e.g. `root/tigris/4.0/{entities,eav}`). This is the entry
/// point DubSar Theater's version picker should use: the returned
/// [`Generation`] is independently openable and queryable, and
/// [`list_versions`] enumerates every version materialized so far.
pub fn materialize_version(
    write_node: &WriteNode,
    root: impl AsRef<Path>,
    version: &str,
) -> io::Result<(Generation, MaterializeStats)> {
    enkidb_readnode::generation::materialize_generation(write_node.journal(), root, SOVEREIGN_NAME, version)
}

/// Every Tigris version materialized so far under `root`, sorted (e.g.
/// `["4.0", "4.1", "4.2"]`). Empty, not an error, if none exist yet.
pub fn list_versions(root: impl AsRef<Path>) -> io::Result<Vec<String>> {
    enkidb_readnode::generation::list_generations(root, SOVEREIGN_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::DocumentParser;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;
    use std::fs;
    use std::path::PathBuf;

    fn tmp_base(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_readnode_{name}"));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);
        dir.join("base")
    }

    fn tmp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_readnode_versioned_{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn tigris_versions_are_listable_and_independently_comparable() {
        // The Architect's ask, proven end to end: "EnkiDDB (Tigris v4.0)"
        // and a later "Tigris v4.1" must both exist, be listable, and
        // disagree in content, so a stakeholder can genuinely compare them.
        let root = tmp_root("compare");

        let mut wn_v40 = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF10)), 64);
        let doc_v40 = DocumentParser::parse_markdown("# Release Notes v4.0\n\nFirst cut.\n");
        wn_v40.ingest_document(&doc_v40, 1);
        let (gen_v40, _) = materialize_version(&wn_v40, &root, "4.0").unwrap();

        let mut wn_v41 = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF11)), 64);
        let doc_v41a = DocumentParser::parse_markdown("# Release Notes v4.1\n\nRevised.\n");
        let doc_v41b = DocumentParser::parse_markdown("# Migration Guide\n\nNew in 4.1.\n");
        wn_v41.ingest_document(&doc_v41a, 1);
        wn_v41.ingest_document(&doc_v41b, 2);
        let (gen_v41, _) = materialize_version(&wn_v41, &root, "4.1").unwrap();

        assert_eq!(SOVEREIGN_NAME, "tigris");
        assert_eq!(list_versions(&root).unwrap(), vec!["4.0", "4.1"]);

        let mut rn_v40 = gen_v40.open().unwrap();
        let mut rn_v41 = gen_v41.open().unwrap();
        assert_eq!(rn_v40.entity_count(), 1, "Tigris v4.0 has one document");
        assert_eq!(rn_v41.entity_count(), 2, "Tigris v4.1 has grown to two documents");

        let found_in_v41_only = rn_v41.query("WHO T.E\nWHERE E[meta.title] = \"Migration Guide\"").unwrap();
        assert_eq!(found_in_v41_only.matched.len(), 1);
        let absent_from_v40 = rn_v40.query("WHO T.E\nWHERE E[meta.title] = \"Migration Guide\"").unwrap();
        assert_eq!(absent_from_v40.matched.len(), 0, "v4.0's generation predates the Migration Guide");
    }

    #[test]
    fn two_node_round_trip_write_then_read() {
        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF08)), 64);
        let doc = DocumentParser::parse_markdown("# EnkiDDB\n\nDocumentation lives here.\n");
        let doc_kaki = wn.ingest_document(&doc, 1);

        let base = tmp_base("round_trip");
        let stats = materialize_now(&wn, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        assert_eq!(stats.entities, 1);

        let mut rn = ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        assert_eq!(rn.entity_count(), 1);

        let attr = "meta.title";
        let query = format!("WHO T.E\nWHERE E[{attr}] = \"EnkiDDB\"");
        let res = rn.query(&query).unwrap();
        assert_eq!(res.matched.len(), 1);
        assert_eq!(*res.matched[0].entity.bytes(), *doc_kaki.bytes());
    }

    #[test]
    fn read_node_sees_nothing_before_materialization() {
        // Prove the CQRS split is real: writes are durable in the Journal
        // immediately, but invisible to the Read Node until materialize_now
        // runs -- the two nodes are genuinely decoupled, not a facade.
        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF09)), 64);
        let doc = DocumentParser::parse_markdown("# Before\n");
        wn.ingest_document(&doc, 1);
        assert_eq!(wn.document_count(), 1, "the write node itself sees the document immediately");

        let base = tmp_base("before_materialize");
        // No materialize_now call yet -- the read node's files don't exist.
        let opened = ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav"));
        assert!(opened.is_err(), "Read Node must not exist before its Data Files are materialized");
    }

    #[test]
    fn materializing_to_a_fresh_generation_picks_up_new_writes() {
        // Each generation gets its own base path -- re-materializing to the
        // SAME path would append duplicates (enkidb-datafile's writer is
        // append-only, never truncates; see materialize_now's doc comment).
        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF0A)), 64);

        let a = DocumentParser::parse_markdown("# Alpha\n");
        wn.ingest_document(&a, 1);
        let gen1 = tmp_base("two_pass_gen1");
        materialize_now(&wn, gen1.with_file_name("entities"), gen1.with_file_name("eav")).unwrap();
        {
            let rn = ReadNode::open(gen1.with_file_name("entities"), gen1.with_file_name("eav")).unwrap();
            assert_eq!(rn.entity_count(), 1);
        }

        let b = DocumentParser::parse_markdown("# Beta\n");
        wn.ingest_document(&b, 2);
        let gen2 = tmp_base("two_pass_gen2");
        materialize_now(&wn, gen2.with_file_name("entities"), gen2.with_file_name("eav")).unwrap();
        let rn = ReadNode::open(gen2.with_file_name("entities"), gen2.with_file_name("eav")).unwrap();
        assert_eq!(rn.entity_count(), 2, "the new generation must include both write node documents");
    }

    #[test]
    fn rematerializing_to_the_same_path_appends_rather_than_replaces() {
        // Documents the real constraint directly, so nobody re-discovers it
        // the hard way: enkidb-datafile's DataFileWriter::open always opens
        // in append mode (OpenOptions::append(true), never truncates).
        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(0xFF0B)), 64);
        let a = DocumentParser::parse_markdown("# Alpha\n");
        wn.ingest_document(&a, 1);

        let base = tmp_base("same_path_appends");
        materialize_now(&wn, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        materialize_now(&wn, base.with_file_name("entities"), base.with_file_name("eav")).unwrap();

        let rn = ReadNode::open(base.with_file_name("entities"), base.with_file_name("eav")).unwrap();
        assert_eq!(rn.entity_count(), 2, "same-path re-materialize duplicates the one entity written so far");
    }
}
