//! Directory ingestion — walk a folder tree of Markdown files and mint
//! every one of them into KAKI-sealed particles in one pass.
//!
//! This is the practical entry point for cataloging a large, scattered
//! pile of documentation folders: point it at a root directory, get back
//! one Identity-Kaki + particle set per `.md` file found, with the
//! original filesystem path preserved as `meta.source_path`. Every
//! document is also categorized (`meta.collection`, [`infer_collection`])
//! and split into RAG-indexable sections (`emitter::emit_sections`) in the
//! same pass — see [`IngestedDocument::sections`] and `crate::rag`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_kaki::{IdentityKaki, KakiMinter};
use enkidb_particles::Particle;

use crate::document::DocumentStructure;
use crate::emitter::DocumentEmitter;
use crate::orbit::DocOrbit;
use crate::parser::DocumentParser;

/// One ingested document: its minted identity, source path, particles,
/// inferred collection, and its own RAG sections (each a child
/// Identity-Kaki linked back to `kaki`, with `body.summary`/`body.text`).
pub struct IngestedDocument {
    pub kaki: IdentityKaki,
    pub source_path: PathBuf,
    pub particles: Vec<Particle>,
    pub collection: String,
    pub sections: Vec<(IdentityKaki, Vec<Particle>)>,
}

/// Classify a document's KIND from its filesystem path — the
/// `meta.collection` taxonomy, orthogonal to `DocOrbit`. Two rules, tried
/// in order:
///
/// 1. **The Architect's canonical numbered taxonomy** (`docs/00_codex/`,
///    `docs/02_identity/`, `docs/_start_here/`, `docs/components/`, ...):
///    if the file sits inside a subfolder of any `docs/` directory, that
///    subfolder's name — numeric prefix and leading underscore stripped,
///    kebab-cased — *is* the collection (`02_identity` -> `"identity"`,
///    `06_governance_parzu` -> `"governance-parzu"`). This is the
///    intended primary path once documents live under that structure:
///    the folder layout and the EAV taxonomy are the same thing, by
///    construction, not two schemes that can drift apart.
/// 2. **Filename heuristics**, for anything not (yet) organized that way
///    — a bare file directly in `docs/`'s root, or with no `docs` segment
///    in its path at all (e.g. `playbooks/`). Falls back to `"general"`
///    for anything matching neither (never an error — every document
///    gets some collection).
pub fn infer_collection(path: &Path) -> String {
    let file_stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if path
        .to_string_lossy()
        .to_lowercase()
        .contains("playbooks/")
        || file_stem.starts_with("playbook_")
    {
        return "playbook-record".to_string();
    }

    if let Some(collection) = docs_subfolder_collection(path) {
        return collection;
    }

    if file_stem.contains("glossary") {
        "glossary".to_string()
    } else if file_stem.contains("architecture") {
        "architecture-reference".to_string()
    } else if file_stem.contains("transparency") {
        "concept-law".to_string()
    } else {
        "general".to_string()
    }
}

/// If `path` sits inside a subfolder of a `docs/` directory, return that
/// subfolder's name as a collection tag (numeric prefix and leading
/// underscore stripped, underscores kebab-cased). `None` for a file
/// sitting directly in `docs/`'s own root, or with no `docs` segment in
/// its path — those fall back to `infer_collection`'s filename heuristics.
/// Only the immediate subfolder is used (deeper nesting, e.g.
/// `docs/_meta/01_templates/x.md`, still collapses to `"meta"`) — one
/// level of granularity per the Architect's own numbered layout.
fn docs_subfolder_collection(path: &Path) -> Option<String> {
    let components: Vec<&str> = path.components().filter_map(|c| c.as_os_str().to_str()).collect();
    let docs_idx = components.iter().rposition(|c| c.eq_ignore_ascii_case("docs"))?;
    let folder = components.get(docs_idx + 1)?;

    // The file itself must be at least one more segment past `folder` --
    // otherwise `folder` IS the filename (a root-level doc, not one in a
    // subfolder).
    if docs_idx + 2 >= components.len() {
        return None;
    }

    let stripped = folder
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start_matches('_');
    if stripped.is_empty() {
        return None;
    }
    Some(stripped.to_lowercase().replace('_', "-"))
}

/// Errors that can occur while walking the directory tree, or while
/// enforcing [`crate::authorship`]'s team-membership gate on it.
#[derive(Debug)]
pub enum IngestError {
    Io(std::io::Error),
    /// A file's git commit author is not on the team allowlist (or has
    /// no determinable author at all) — see
    /// [`crate::writenode::WriteNode::ingest_directory_categorized_checked`].
    UnauthorizedCreator { path: PathBuf, author: Option<String> },
    /// A file's raw bytes matched a known malware/webshell/dropper
    /// signature — see [`crate::security::scan_document`]. Fails closed:
    /// the file is never parsed or journaled.
    SecurityRejected { path: PathBuf, detail: String },
}

impl From<std::io::Error> for IngestError {
    fn from(e: std::io::Error) -> Self {
        IngestError::Io(e)
    }
}

impl std::fmt::Display for IngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IngestError::Io(e) => write!(f, "io error: {e}"),
            IngestError::UnauthorizedCreator { path, author } => write!(
                f,
                "unauthorized creator for {}: {}",
                path.display(),
                author.as_deref().unwrap_or("<unknown -- no git history>")
            ),
            IngestError::SecurityRejected { path, detail } => {
                write!(f, "security scan rejected {}: {detail}", path.display())
            }
        }
    }
}

impl std::error::Error for IngestError {}

/// Recursively find every `.md` file under `root`, parse it, and mint it
/// into particles. Returns one `IngestedDocument` per file found.
pub fn ingest_directory(
    minter: &KakiMinter,
    root: &Path,
) -> Result<Vec<IngestedDocument>, IngestError> {
    let mut results = Vec::new();
    let mut paths = Vec::new();
    collect_markdown_files(root, &mut paths)?;

    let emitter = DocumentEmitter::new(minter);
    for path in paths {
        let text = fs::read_to_string(&path)?;
        let structure: DocumentStructure = DocumentParser::parse_markdown(&text);
        let (kaki, mut particles) = emitter.emit_document(&structure);

        particles.push(Particle::base(
            kaki,
            DocOrbit::Meta.attr("source_path"),
            AkkValue::Text(path.display().to_string()),
            now_secs(),
        ));

        let collection = infer_collection(&path);
        particles.push(emitter.emit_collection(kaki, &collection));
        let sections = emitter.emit_sections(kaki, &structure.title, &structure);

        results.push(IngestedDocument {
            kaki,
            source_path: path,
            particles,
            collection,
            sections,
        });
    }

    Ok(results)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// The public "scan" stage: every `.md` file under `root`, recursively,
/// sorted, depth-first — the exact same walk `ingest_directory` and
/// `WriteNode::ingest_directory_categorized` use, exposed on its own so a
/// caller (e.g. `enkiddb-cli`) can inspect what would be ingested before
/// committing to categorizing or running it.
pub fn scan_markdown_directory(root: &Path) -> Result<Vec<PathBuf>, IngestError> {
    let mut paths = Vec::new();
    collect_markdown_files(root, &mut paths)?;
    Ok(paths)
}

/// Recursively collect every `.md` file under `dir`, sorted, depth-first.
/// `pub(crate)` -- `writenode::WriteNode::ingest_directory_categorized` reuses
/// this same walk so the two directory-ingestion entry points (this
/// module's particle-only `ingest_directory` and the Journal-writing one on
/// `WriteNode`) never drift apart on which files they consider documents.
pub(crate) fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), IngestError> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use std::fs;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x00E1))
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("enkiddb_ingest_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        dir
    }

    #[test]
    fn ingests_all_markdown_files_recursively() {
        let dir = scratch_dir("recursive");
        fs::write(dir.join("root.md"), "# Root\n\nRoot body.\n").unwrap();
        fs::write(dir.join("sub/nested.md"), "# Nested\n\nNested body.\n").unwrap();
        fs::write(dir.join("ignore.txt"), "not markdown").unwrap();

        let m = minter();
        let docs = ingest_directory(&m, &dir).unwrap();

        assert_eq!(docs.len(), 2);
        let titles: Vec<String> = docs
            .iter()
            .flat_map(|d| d.particles.iter())
            .filter(|p| p.attribute == "meta.title")
            .filter_map(|p| match &p.value {
                AkkValue::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect();
        assert!(titles.contains(&"Root".to_string()));
        assert!(titles.contains(&"Nested".to_string()));
    }

    #[test]
    fn each_document_carries_its_source_path() {
        let dir = scratch_dir("source_path");
        fs::write(dir.join("doc.md"), "# Doc\n\nBody.\n").unwrap();

        let m = minter();
        let docs = ingest_directory(&m, &dir).unwrap();

        assert_eq!(docs.len(), 1);
        let has_path = docs[0]
            .particles
            .iter()
            .any(|p| p.attribute == "meta.source_path");
        assert!(has_path);
        assert_eq!(docs[0].source_path, dir.join("doc.md"));
    }

    #[test]
    fn infer_collection_classifies_known_path_conventions() {
        assert_eq!(infer_collection(Path::new("playbooks/playbook_176_foo.yml")), "playbook-record");
        assert_eq!(infer_collection(Path::new("docs/components/KAKI_V4.md")), "components");
        assert_eq!(infer_collection(Path::new("docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md")), "glossary");
        assert_eq!(infer_collection(Path::new("docs/BAHYWAY_V4_ARCHITECTURE_REFERENCE.md")), "architecture-reference");
        assert_eq!(infer_collection(Path::new("docs/TRANSPARENCY_STANDARD.md")), "concept-law");
        assert_eq!(infer_collection(Path::new("docs/random_notes.md")), "general");
    }

    #[test]
    fn infer_collection_maps_the_canonical_numbered_taxonomy() {
        // The Architect's real docs/ layout: numbered top-level folders
        // (docs/NN_name/) and leading-underscore folders (docs/_name/) both
        // become the collection tag directly, prefix stripped, kebab-cased.
        assert_eq!(infer_collection(Path::new("docs/00_codex/axioms.md")), "codex");
        assert_eq!(infer_collection(Path::new("docs/02_identity/identity_kaki.md")), "identity");
        assert_eq!(infer_collection(Path::new("docs/06_governance_parzu/parzu_laws.md")), "governance-parzu");
        assert_eq!(infer_collection(Path::new("docs/14_decisions_adr/adr_001_no_external_db.md")), "decisions-adr");
        assert_eq!(infer_collection(Path::new("docs/_start_here/for_architects.md")), "start-here");
        assert_eq!(infer_collection(Path::new("docs/_diagrams/high_council_diagram.md")), "diagrams");
        // Nested deeper than one level still collapses to the top folder.
        assert_eq!(infer_collection(Path::new("docs/_meta/01_templates/help_page_template.md")), "meta");
        // A file directly in docs/'s own root has no subfolder to use --
        // falls through to the filename heuristics instead.
        assert_eq!(infer_collection(Path::new("docs/README.md")), "general");
    }

    #[test]
    fn ingested_documents_carry_a_collection_and_their_own_sections() {
        let dir = scratch_dir("categorized");
        fs::create_dir_all(dir.join("docs/components")).unwrap();
        fs::write(
            dir.join("docs/components/widget.md"),
            "# Widget\n\n## Overview\n\nThe widget does things.\n\n## Usage\n\nCall it like this.\n",
        )
        .unwrap();

        let m = minter();
        let docs = ingest_directory(&m, &dir).unwrap();

        assert_eq!(docs.len(), 1);
        let doc = &docs[0];
        assert_eq!(doc.collection, "components");
        assert!(doc.particles.iter().any(|p| p.attribute == "meta.collection"));
        assert_eq!(doc.sections.len(), 2, "Overview + Usage sections");
        for (section_kaki, section_particles) in &doc.sections {
            assert_ne!(section_kaki.bytes(), doc.kaki.bytes());
            assert!(section_particles.iter().any(|p| p.attribute == "body.summary"));
            assert!(section_particles.iter().any(|p| p.attribute == "body.text"));
            let link = section_particles.iter().find(|p| p.attribute == "link.target").unwrap();
            match link.value {
                AkkValue::KakiPk(bytes) => assert_eq!(bytes, *doc.kaki.bytes()),
                _ => panic!("expected KakiPk"),
            }
        }
    }

    #[test]
    fn empty_directory_yields_no_documents() {
        let dir = scratch_dir("empty");
        let m = minter();
        let docs = ingest_directory(&m, &dir).unwrap();
        assert!(docs.is_empty());
    }

    #[test]
    fn scan_markdown_directory_finds_files_without_ingesting_them() {
        let dir = scratch_dir("scan_only");
        fs::write(dir.join("root.md"), "# Root\n\nBody.\n").unwrap();
        fs::write(dir.join("sub/nested.md"), "# Nested\n\nBody.\n").unwrap();
        fs::write(dir.join("ignore.txt"), "not markdown").unwrap();

        let found = scan_markdown_directory(&dir).unwrap();
        assert_eq!(found, vec![dir.join("root.md"), dir.join("sub/nested.md")]);
    }
}
