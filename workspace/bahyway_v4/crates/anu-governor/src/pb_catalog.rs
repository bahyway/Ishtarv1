//! pb_catalog — multi-location playbook catalog: gathers every real
//! playbook file across every known historical/backup location (this
//! repo's own corpus, the Architect's DailyWork repo, the retired
//! eriduous-vdi's rsynced mirror, dated external-drive backups, ...) and
//! mints each DISTINCT content as a real EnkiDDB document in
//! `enkiddb::PLAYBOOK_CATALOG_TRIBE_ID` — deliberately NOT the sealed
//! corpus tribes (`pb_mint::PB_TRIBE_ID`, `enkiddb::PB_DOCS_TRIBE_ID`), so
//! an unreconciled backup copy never silently mixes with the current,
//! accepted playbook set.
//!
//! Three real, distinct things get graphed, all navigable from DubSar
//! Theater's Graph Explorer (Ctrl+K):
//!   - One marker document per named location, linked to every playbook
//!     found there ("found-in") — so "what's in the VMLAB backup" is a
//!     real graph query, not a filename guess.
//!   - Real content-hash dedup: the SAME bytes seen in five locations
//!     mint ONCE, with five "found-in" links — never five particles.
//!   - Real, honest collision surfacing: two DIFFERENT-content files that
//!     both claim the same PB number get a neutral
//!     `same-PB-number-as (unreconciled)` link to each other — never an
//!     automatic pick of a winner. That stays the Architect's call,
//!     exercised later via the real `supersede_document`/promotion into
//!     this repo's own `playbooks/` once a collision is actually ruled on.
//!
//! Every newly-catalogued document starts tagged `proposed_status =
//! unreviewed` (folded into its body text, queryable like any other
//! content) — this module never writes a "what does this PB do" summary.
//! That's a deliberate, separate, later phase (once the content is
//! actually readable end to end by whoever/whatever does that review),
//! not something minted blind here.

use bahyway_core::TribeId;
use enkidb_kaki::{IdentityKaki, KakiMinter};
use enkiddb::document::{BodyElement, BodyType, DocumentStructure};
use enkiddb::DocumentParser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// One named location to walk for candidate playbook files.
#[derive(Debug, Clone)]
pub struct CatalogLocation {
    pub name: String,
    pub root: PathBuf,
}

/// `pub(crate)` so `gate_review` can read this same on-disk registry
/// shape without duplicating its schema -- gate suggestions are scanned
/// straight from the paths/kakis `pb_catalog` already recorded.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RegistryLine {
    pub(crate) sha256: String,
    pub(crate) kaki_hex: String,
    pub(crate) first_seen_path: String,
    pub(crate) pb_number: Option<u32>,
    pub(crate) minted_at: String,
}

/// One catalogued playbook, title re-derived the same way
/// `catalog_playbooks` itself derives it when minting/re-minting --
/// everything a search/listing UI needs without requiring a live,
/// published EnkiDDB generation (e.g. before anything has ever been
/// gate/domain-tagged, or if `current` hasn't been promoted yet).
///
/// Carries `full_text` (the file's real, complete content -- the WHY
/// text, task bodies, everything) alongside `title`/`source_path`
/// specifically so a caller can search the ENTIRE playbook, not just its
/// title line: a name like "Ezida" showing up only in a playbook's WHY
/// THIS EXISTS prose, never in its filename or its header's title line,
/// is real and expected -- restricting search to title/path alone would
/// silently miss it. `full_text` is never itself serialized into an API
/// response (callers select which fields to expose); it exists purely as
/// search input.
#[derive(Debug, Clone)]
pub struct CatalogedPlaybook {
    pub kaki_hex: String,
    pub title: String,
    pub source_path: String,
    pub full_text: String,
}

/// Every playbook `registry_path` (`catalog_playbooks`'s own content
/// registry) has ever recorded, title and full text included. Re-reads
/// and re-parses each file on every call -- correct (content can't go
/// stale between calls) and, at this catalog's real size (hundreds, not
/// millions, of playbooks), fast enough for an interactive search box;
/// no caching layer here is warranted yet.
pub fn list_cataloged_playbooks(registry_path: &Path) -> Vec<CatalogedPlaybook> {
    std::fs::read_to_string(registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .map(|r| {
            let path = PathBuf::from(&r.first_seen_path);
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let full_text = std::fs::read_to_string(&path).unwrap_or_default();
            let title = DocumentParser::parse_playbook_header(&full_text)
                .map(|s| s.title)
                .filter(|t| !t.trim().is_empty())
                .unwrap_or(stem);
            CatalogedPlaybook { kaki_hex: r.kaki_hex, title, source_path: r.first_seen_path, full_text }
        })
        .collect()
}

/// When (and from where) `kaki_hex` was first catalogued -- a real,
/// durable fact `catalog_playbooks` already recorded, read back here for
/// StoryEngine's chronicle (`(minted_at, first_seen_path)`). `None` if
/// this kaki isn't in the registry at all.
pub fn catalog_event_for(registry_path: &Path, kaki_hex: &str) -> Option<(String, String)> {
    std::fs::read_to_string(registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .find(|r| r.kaki_hex == kaki_hex)
        .map(|r| (r.minted_at, r.first_seen_path))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// `playbook_269_x.yml` -> 269, `bw4_playbook_79_nergal_av.yml` -> 79,
/// `05_beemdm_monitor.yml` -> 5, `playbook_86_fix_draw_args(1).yml` -> 86,
/// `playbook_97_enlil_query_server_OLD.yml` -> 97. Best-effort only —
/// this is a HINT for collision-detection grouping, never trusted for
/// identity (the content hash is that). Returns `None` for names with no
/// leading/prefixed number at all.
pub(crate) fn guess_pb_number(file_stem: &str) -> Option<u32> {
    let s = file_stem.strip_prefix("bw4_").unwrap_or(file_stem);
    let s = s.strip_prefix("playbook_").unwrap_or(s);
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

/// True for any file this catalog considers a playbook candidate —
/// `.yml`/`.yaml` extension is the only hard requirement; naming
/// convention is deliberately NOT enforced here (unlike
/// `enkimdb::pb::scan_pbs`'s strict `playbook_<N>_...` law for the sealed
/// corpus), since the whole point of this scan is finding files that
/// never conformed to that shape in the first place.
fn is_playbook_candidate(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
        Some("yml") | Some("yaml")
    )
}

/// Recursively walk `root` for playbook-candidate files, bounded to
/// `max_depth` levels so a location with unexpectedly deep nesting can't
/// run away — 8 is generous for the deepest known case (a dated VMLAB
/// snapshot's `Forge/bahyway_v4/.../files(N)/*.yml`). Skips hidden
/// directories (`.git` etc.); never follows symlinks.
fn walk_playbook_files(root: &Path, max_depth: u32) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_inner(root, max_depth, &mut out);
    out
}

fn walk_inner(dir: &Path, depth_left: u32, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.filter_map(|e| e.ok()) {
        let Ok(file_type) = entry.file_type() else { continue };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            if depth_left > 0 {
                walk_inner(&path, depth_left - 1, out);
            }
        } else if file_type.is_file() && is_playbook_candidate(&path) {
            out.push(path);
        }
    }
}

/// Walk every named `locations` entry for real playbook files, dedupe by
/// content sha256 (the SAME bytes found in five locations mints once,
/// with five "found-in" links — never five particles), surface real
/// same-number/different-content collisions as neutral cross-reference
/// links, and materialize the result into a real, HeptaScript-queryable
/// EnkiDDB Tigris generation under `enkiddb::PLAYBOOK_CATALOG_TRIBE_ID`.
///
/// Quiet-on-rerun, same discipline as `pb_mint`/`pb_doc_mint`: content
/// already in `registry_path` mints nothing new, but a NEW location
/// sighting of already-known content still gets its own "found-in" link,
/// so re-running this after adding one more backup location is safe and
/// cheap — not a full re-scan from zero.
///
/// A location whose `root` doesn't exist (e.g. an unmounted external
/// drive) is reported in the returned log, never silently skipped.
pub fn catalog_playbooks(
    locations: &[CatalogLocation],
    registry_path: &Path,
    enkiddb_output_root: &Path,
) -> Vec<String> {
    let mut log = Vec::new();

    let known: HashMap<String, RegistryLine> = std::fs::read_to_string(registry_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<RegistryLine>(l).ok())
        .map(|r| (r.sha256.clone(), r))
        .collect();

    let minter = KakiMinter::new(TribeId::from_u16(enkiddb::PLAYBOOK_CATALOG_TRIBE_ID));
    let mut write_node = enkiddb::WriteNode::new(minter, 64);
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f").to_string();
    let mut epoch = 1u32;

    if let Some(p) = registry_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut registry_file = std::fs::OpenOptions::new().create(true).append(true).open(registry_path).ok();

    let mut location_kakis: HashMap<String, IdentityKaki> = HashMap::new();
    // Rehydrate identities already minted in a PREVIOUS run so this run's
    // links can point at them — cross-run linking is the same pattern
    // `supersede_document` already relies on (its own doc comment: "old
    // ... possibly on a different WriteNode/run").
    let mut hash_kakis: HashMap<String, IdentityKaki> = known
        .iter()
        .filter_map(|(sha, r)| {
            let bytes = hex::decode(&r.kaki_hex).ok()?;
            let arr: [u8; 16] = bytes.try_into().ok()?;
            let kaki = enkidb_kaki::Kaki::from_bytes(arr).ok()?;
            let identity = IdentityKaki::try_from_kaki(kaki).ok()?;
            Some((sha.clone(), identity))
        })
        .collect();
    // number -> (first doc identity, its title, a log-friendly description)
    // seen THIS RUN with that number, for collision links.
    let mut number_first_seen: HashMap<u32, (IdentityKaki, String, String)> = HashMap::new();
    let mut minted_this_run = 0usize;

    for loc in locations {
        if !loc.root.is_dir() {
            log.push(format!(
                "⚠ location '{}' not reachable at {} -- skipped, not silently ignored",
                loc.name,
                loc.root.display()
            ));
            continue;
        }

        let loc_title = format!("Playbook catalog location: {}", loc.name);
        let loc_kaki = *location_kakis.entry(loc.name.clone()).or_insert_with(|| {
            let k = write_node.mint_marker(&loc_title, "playbook-catalog-location", epoch);
            epoch += 1;
            k
        });

        for file in walk_playbook_files(&loc.root, 8) {
            let Ok(bytes) = std::fs::read(&file) else { continue };
            let hash = sha256_hex(&bytes);
            let stem = file.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let pb_number = guess_pb_number(&stem);

            let Ok(text) = std::fs::read_to_string(&file) else { continue };
            // Scanned and parsed for EVERY file, known-hash or not.
            //
            // FIXED: `enkidb-readnode::Generation` is a complete,
            // independent snapshot of whatever journal it was built from
            // (proved by `enkidb-readnode::generation::tests::
            // two_versions_of_the_same_sovereign_name_are_independently_addressable`,
            // "v4.2's generation must not see v4.0's documents"). This used
            // to scan/journal a document's own content ONLY the first time
            // its hash was seen -- a known-hash file on any LATER run just
            // reused its identity with no re-journaling at all. That meant
            // the second time this function is ever rerun over an
            // unchanged (or partially overlapping) location set -- adding
            // one more backup location is exactly the expected workflow --
            // the newly materialized generation would carry every fresh
            // "found-in"/"contains" edge but be MISSING meta.title/body
            // content for every previously-catalogued playbook, since only
            // brand-new content ever got (re-)journaled. Every file now
            // gets its full content re-scanned and re-journaled every run,
            // regardless of whether its hash is new or already known, so
            // every materialized generation stays self-sufficient.
            let scan = enkiddb::scan_document(text.as_bytes());
            if !scan.clean {
                log.push(format!("⚠ skipped {} ({}): {}", file.display(), loc.name, scan.detail));
                continue;
            }
            let mut structure = DocumentParser::parse_playbook_header(&text)
                .unwrap_or_else(|| DocumentStructure { title: stem.clone(), ..Default::default() });
            if structure.title.trim().is_empty() {
                structure.title = stem.clone();
            }
            let doc_title = structure.title.clone();
            let next_order = structure.body.len() as i64;
            structure.body.push(BodyElement {
                element_type: BodyType::Paragraph,
                content: format!(
                    "[playbook-catalog] source_path={} content_sha256={} proposed_status=unreviewed",
                    file.display(),
                    hash
                ),
                order: next_order,
            });

            let doc_kaki = if let Some(existing) = hash_kakis.get(&hash) {
                let existing = *existing;
                write_node.reingest_document_categorized(existing, &structure, epoch, "playbook-record-candidate");
                epoch += 1;
                existing
            } else {
                let kaki = write_node.ingest_document_categorized(&structure, epoch, "playbook-record-candidate");
                epoch += 1;
                minted_this_run += 1;

                let line = RegistryLine {
                    sha256: hash.clone(),
                    kaki_hex: hex::encode(kaki.bytes()),
                    first_seen_path: file.display().to_string(),
                    pb_number,
                    minted_at: chrono::Utc::now().to_rfc3339(),
                };
                if let Some(f) = &mut registry_file {
                    if let Ok(json) = serde_json::to_string(&line) {
                        let _ = writeln!(f, "{json}");
                    }
                }
                hash_kakis.insert(hash.clone(), kaki);
                log.push(format!("𒁾 catalogued {} ({}) → {kaki}", file.display(), loc.name));
                kaki
            };

            // Each direction gets its OWN dedicated edge entity
            // (`WriteNode::mint_link_edge`), never a repeated particle on
            // the shared `doc_kaki`/`loc_kaki` identity -- a location
            // linked to hundreds of playbooks (or one playbook found in
            // several locations) would otherwise collapse to just the
            // LAST edge written, per `apply_entry_to_map`'s last-write-
            // wins fold. This is what actually makes "browse outward from
            // a location" work: Graph Explorer's second query,
            // `WHERE E[link.source_title] = "<title>"`, finds every edge
            // whose source is that exact title, however many there are.
            write_node.mint_link_edge(
                doc_kaki,
                &doc_title,
                loc_kaki,
                &loc_title,
                &format!("found-in:{}", loc.name),
                epoch,
            );
            epoch += 1;
            write_node.mint_link_edge(
                loc_kaki,
                &loc_title,
                doc_kaki,
                &doc_title,
                &format!("contains:{stem}"),
                epoch,
            );
            epoch += 1;

            if let Some(n) = pb_number {
                match number_first_seen.get(&n) {
                    Some((first_kaki, first_title, first_desc)) if *first_kaki != doc_kaki => {
                        write_node.mint_link_edge(
                            doc_kaki,
                            &doc_title,
                            *first_kaki,
                            first_title,
                            &format!("same-PB-number-as (unreconciled): PB-{n}, first seen {first_desc}"),
                            epoch,
                        );
                        epoch += 1;
                        log.push(format!(
                            "⚠ collision: PB-{n} has multiple distinct contents ({} vs first-seen {first_desc})",
                            file.display()
                        ));
                    }
                    Some(_) => {}
                    None => {
                        number_first_seen
                            .insert(n, (doc_kaki, doc_title.clone(), format!("{} ({})", file.display(), loc.name)));
                    }
                }
            }
        }
    }

    if write_node.document_count() > 0 {
        match enkiddb::materialize_version(&write_node, enkiddb_output_root, &stamp) {
            Ok((generation, stats)) => log.push(format!(
                "𒁾 EnkiDDB Tigris generation {stamp} materialized: {} entities → {}",
                stats.entities,
                generation.entities_path.display()
            )),
            Err(e) => log.push(format!("⚠ EnkiDDB materialize failed: {e}")),
        }
    }
    log.push(format!("✓ {minted_this_run} new unique playbook(s) catalogued this run"));
    log
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("shk_pb_catalog_test_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_pb(dir: &Path, name: &str, why: &str) {
        std::fs::write(
            dir.join(name),
            format!("# =====\n# PB-910 -- Scratch test playbook\n#\n# WHY THIS EXISTS: {why}\n# =====\n- name: \"scratch\"\n  hosts: localhost\n"),
        )
        .unwrap();
    }

    #[test]
    fn catalogs_a_real_new_playbook_from_one_location() {
        let dir = scratch_dir("one_loc");
        write_pb(&dir, "playbook_910_scratch.yml", "a real first reason");
        let registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let locations = vec![CatalogLocation { name: "test-loc".into(), root: dir.clone() }];
        let log = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert!(log.iter().any(|l| l.contains("catalogued") && l.contains("playbook_910_scratch")), "log: {log:?}");
    }

    #[test]
    fn dedupes_identical_content_across_two_locations_into_one_particle() {
        let dir = scratch_dir("dedupe");
        let loc_a = dir.join("a");
        let loc_b = dir.join("b");
        std::fs::create_dir_all(&loc_a).unwrap();
        std::fs::create_dir_all(&loc_b).unwrap();
        write_pb(&loc_a, "playbook_911_scratch.yml", "identical content");
        write_pb(&loc_b, "playbook_911_scratch.yml", "identical content");
        let registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let locations = vec![
            CatalogLocation { name: "loc-a".into(), root: loc_a },
            CatalogLocation { name: "loc-b".into(), root: loc_b },
        ];
        let log = catalog_playbooks(&locations, &registry, &enkiddb_root);
        let minted_count = log.iter().filter(|l| l.contains("𒁾 catalogued")).count();
        assert_eq!(minted_count, 1, "identical content in two locations must mint exactly once: {log:?}");
        assert!(log.iter().any(|l| l.contains("found-in:loc-a") || l.contains("loc-a")), "log: {log:?}");
    }

    #[test]
    fn flags_a_real_same_number_different_content_collision() {
        let dir = scratch_dir("collision");
        let loc_a = dir.join("a");
        let loc_b = dir.join("b");
        std::fs::create_dir_all(&loc_a).unwrap();
        std::fs::create_dir_all(&loc_b).unwrap();
        write_pb(&loc_a, "playbook_912_scratch.yml", "the FIRST version");
        write_pb(&loc_b, "playbook_912_scratch.yml", "a COMPLETELY DIFFERENT version");
        let registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let locations = vec![
            CatalogLocation { name: "loc-a".into(), root: loc_a },
            CatalogLocation { name: "loc-b".into(), root: loc_b },
        ];
        let log = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert!(log.iter().any(|l| l.contains("collision: PB-912")), "log: {log:?}");
    }

    #[test]
    fn unreachable_location_is_reported_not_silently_skipped() {
        let dir = scratch_dir("unreachable");
        let registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let locations = vec![CatalogLocation { name: "missing-drive".into(), root: dir.join("does_not_exist") }];
        let log = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert!(log.iter().any(|l| l.contains("not reachable") && l.contains("missing-drive")), "log: {log:?}");
    }

    // Real end-to-end regression test for the exact bug a live user hit:
    // Graph Explorer expanding a location title showed only ONE connected
    // playbook no matter how many were actually catalogued there, because
    // every "contains" edge used to be written as a repeated particle on
    // the SAME shared location entity (last-write-wins on read back).
    // Materializes for real and queries back through the real ReadNode --
    // the same `WHERE E[link.source_title] = "..."` shape Graph Explorer's
    // second query now issues -- to prove every edge survives, not just
    // that a log line was printed.
    #[test]
    fn every_contains_edge_from_one_location_survives_a_real_read_node_query() {
        let dir = scratch_dir("multi_edge_readback");
        write_pb(&dir, "playbook_920_scratch.yml", "first");
        write_pb(&dir, "playbook_921_scratch.yml", "second");
        write_pb(&dir, "playbook_922_scratch.yml", "third");
        let registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");

        let locations = vec![CatalogLocation { name: "multi-loc".into(), root: dir.clone() }];
        let log = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert_eq!(log.iter().filter(|l| l.contains("𒁾 catalogued")).count(), 3, "log: {log:?}");

        let tigris_root = enkiddb_root.join("tigris");
        let generation_dir = std::fs::read_dir(&tigris_root)
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .expect("catalog_playbooks must materialize exactly one generation")
            .path();
        let mut read_node =
            enkiddb::ReadNode::open(generation_dir.join("entities"), generation_dir.join("eav")).unwrap();

        // NOTE: no "QUERY:" prefix here -- that's a wire-protocol tag
        // enkiddb-read-server strips before ever calling parse_query (see
        // bin/enkiddb-read-server/src/main.rs's `strip_prefix("QUERY:")`);
        // calling `ReadNode::query` directly, as this test does, means
        // starting straight from the grammar's own `who` clause.
        let query = "WHO T.E\nWHAT E[link.target, link.target_title, link.description]\nWHERE E[link.source_title] = \"Playbook catalog location: multi-loc\"";
        let result = read_node.query(query).unwrap();

        let contains_edges: Vec<&str> = result
            .matched
            .iter()
            .filter_map(|m| {
                m.projected.iter().find(|(k, _)| k == "link.description").and_then(|(_, v)| match v {
                    akkvalue::AkkValue::Text(s) if s.starts_with("contains:") => Some(s.as_str()),
                    _ => None,
                })
            })
            .collect();
        assert_eq!(
            contains_edges.len(),
            3,
            "all three contains-edges from the SAME location must be independently queryable, not \
             collapsed to just the last one written: {contains_edges:?}"
        );
    }

    // Real end-to-end regression test for the generation-completeness bug
    // found while designing the gate-orbit feature: `enkidb-readnode`'s own
    // test proves each materialized `Generation` is a complete, INDEPENDENT
    // snapshot ("v4.2's generation must not see v4.0's documents") -- but
    // this function used to skip re-journaling a document's own content
    // (title/body) on every run after its first, only ever re-minting fresh
    // edges. So the SECOND real run over the same (or an expanded) location
    // set -- adding one more backup location is the expected workflow --
    // would materialize a generation missing meta.title for every
    // previously-catalogued playbook, breaking Graph Explorer's title
    // search for anything not freshly re-minted that run. Runs
    // catalog_playbooks TWICE over the identical location, opens ONLY the
    // SECOND generation, and confirms the first run's playbook is still
    // found by its exact title.
    #[test]
    fn a_second_run_still_produces_a_self_sufficient_generation_for_first_run_content() {
        let dir = scratch_dir("two_run_completeness");
        write_pb(&dir, "playbook_930_scratch.yml", "unchanged across both runs");
        let registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        let locations = vec![CatalogLocation { name: "stable-loc".into(), root: dir.clone() }];

        let first = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert_eq!(first.iter().filter(|l| l.contains("𒁾 catalogued")).count(), 1, "log: {first:?}");

        let second = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert!(!second.iter().any(|l| l.contains("𒁾 catalogued")), "second run must mint nothing new: {second:?}");

        let tigris_root = enkiddb_root.join("tigris");
        let mut generation_names: Vec<String> = std::fs::read_dir(&tigris_root)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        generation_names.sort();
        assert_eq!(generation_names.len(), 2, "each run must materialize its own generation: {generation_names:?}");
        let latest_dir = tigris_root.join(generation_names.last().unwrap());

        let mut read_node = enkiddb::ReadNode::open(latest_dir.join("entities"), latest_dir.join("eav")).unwrap();
        // NOTE: `write_pb`'s fixture content always parses to the same
        // title, "PB-910 -- Scratch test playbook" (its first non-
        // separator header line -- see
        // `enkiddb::DocumentParser::parse_playbook_header`), regardless of
        // the filename/why-text passed to it.
        let query = "WHO T.E\nWHAT E[meta.title]\nWHERE E[meta.title] = \"PB-910 -- Scratch test playbook\"";
        let result = read_node.query(query).unwrap();
        assert_eq!(
            result.matched.len(),
            1,
            "the SECOND run's generation must still carry the first run's playbook content, not just its edges"
        );
    }

    #[test]
    fn rerun_with_no_new_locations_mints_nothing_new() {
        let dir = scratch_dir("rerun");
        write_pb(&dir, "playbook_913_scratch.yml", "stable reason");
        let registry = dir.join("chronicle/pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        let locations = vec![CatalogLocation { name: "test-loc".into(), root: dir.clone() }];

        let first = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert!(first.iter().any(|l| l.contains("𒁾 catalogued")));

        let second = catalog_playbooks(&locations, &registry, &enkiddb_root);
        assert!(!second.iter().any(|l| l.contains("𒁾 catalogued")), "second run must mint nothing new: {second:?}");
    }

    /// The bug `anu_governor_web::playbook_why_text` exists to work around:
    /// `body.paragraph` is a single EAV scalar on the parent entity
    /// (last-write-wins), and this module's own tracking marker
    /// (`[playbook-catalog] source_path=...`) is appended as the LAST body
    /// paragraph -- so `body.paragraph` alone only ever shows that marker,
    /// never the playbook's real WHY text, even though the real text was
    /// genuinely journaled.
    ///
    /// It survives un-collapsed one level down, in the document's child
    /// sections -- but NOT as one bare section whose title equals the
    /// parent's own title: `DocumentParser::parse_playbook_header`
    /// recognizes "PREFIX: text" lines (`WHY THIS EXISTS:`, etc.) as real
    /// headers, so a W5H2-style playbook mints one section PER such
    /// prefix, each titled `"{parent title} § {PREFIX}"` -- there's no
    /// single fixed title to guess. What DOES reliably identify every
    /// section belonging to this playbook is its real `link.target` KakiPk
    /// bytes, matched against the parent's own kaki in Rust (HeptaScript's
    /// WHERE clause has no KakiPk-literal comparison at all).
    #[test]
    fn the_real_why_text_survives_via_section_link_target_matching_despite_body_paragraph_collapsing() {
        let dir = scratch_dir("why_text");
        let loc = dir.join("loc");
        std::fs::create_dir_all(&loc).unwrap();
        std::fs::write(
            loc.join("playbook_920_geo.yml"),
            "# =====\n# PB-920 -- Geo Engine\n#\n# WHY THIS EXISTS: real geodesy math the ecosystem needs.\n#\n# It adds UTM<->WGS84 conversion no other crate provides.\n# =====\n- name: \"scratch\"\n  hosts: localhost\n",
        )
        .unwrap();

        let registry = dir.join("pb_catalog_registry.jsonl");
        let enkiddb_root = dir.join("enkiddb_data");
        catalog_playbooks(&[CatalogLocation { name: "loc".into(), root: loc }], &registry, &enkiddb_root);

        let catalogued = list_cataloged_playbooks(&registry);
        let pb = catalogued.iter().find(|p| p.source_path.contains("920")).unwrap();
        let parent_bytes: [u8; 16] = hex::decode(&pb.kaki_hex).unwrap().try_into().unwrap();

        let tigris_root = enkiddb_root.join("tigris");
        let mut gens: Vec<String> = std::fs::read_dir(&tigris_root)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        gens.sort();
        let latest = tigris_root.join(gens.last().unwrap());
        let mut read_node = enkiddb::ReadNode::open(latest.join("entities"), latest.join("eav")).unwrap();

        // The parent entity's own `body.paragraph` really does collapse to
        // the catalog's bookkeeping marker -- proving the bug is real, not
        // hypothetical.
        let parent_q = format!("WHO T.E\nWHAT E[body.paragraph]\nWHERE E[meta.title] = \"{}\"", pb.title);
        let parent_r = read_node.query(&parent_q).unwrap();
        let saw_marker = parent_r.matched.iter().any(|m| {
            m.projected.iter().any(|(k, v)| {
                k == "body.paragraph"
                    && matches!(v, akkvalue::AkkValue::Text(s) if s.starts_with("[playbook-catalog]"))
            })
        });
        assert!(saw_marker, "the parent's body.paragraph must collapse to the catalog marker: {parent_r:?}");

        // Every real child section is recoverable by matching its
        // link.target bytes against the parent -- not collapsed, not
        // shadowed.
        let section_q = "WHO T.E\nWHAT E[link.target, body.text]\nWHERE E[link.description] = \"section-of\"";
        let section_r = read_node.query(section_q).unwrap();
        let mut found_text = String::new();
        for m in &section_r.matched {
            let mut target: Option<[u8; 16]> = None;
            let mut text: Option<String> = None;
            for (k, v) in &m.projected {
                match (k.as_str(), v) {
                    ("link.target", akkvalue::AkkValue::KakiPk(b)) => target = Some(*b),
                    ("body.text", akkvalue::AkkValue::Text(t)) => text = Some(t.clone()),
                    _ => {}
                }
            }
            if target == Some(parent_bytes) {
                if let Some(t) = text {
                    found_text.push_str(&t);
                    found_text.push('\n');
                }
            }
        }
        assert!(found_text.contains("real geodesy math the ecosystem needs"), "found_text: {found_text}");
        assert!(found_text.contains("UTM<->WGS84 conversion"), "found_text: {found_text}");
    }
}
