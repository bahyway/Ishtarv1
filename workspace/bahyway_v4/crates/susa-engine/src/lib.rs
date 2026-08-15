//! SusaEngine — the sovereign import THRESHOLD.
//! The single gateway where external artifacts cross into the
//! sovereign store: documents -> EnkiDDB, service/app structures +
//! metadata -> EnkiMDB. Susa = the guardian of the boundary;
//! artifacts cross here, are analyzed in a temporary laboratory,
//! and only MINT a KAKI at Commit (downstream in enkidb-ingest).
//!
//! SOVEREIGN LAWS ENFORCED:
//!   - NO JSON anywhere. Staging manifests are sovereign EAV rows.
//!   - NO KAKI minting here. Susa PROPOSES; enkidb-ingest mints.
//!   - NO RGB/quality bytes. Quality/freshness are EAV attributes.
//!   - Categories are NEVER originated by Susa. It only SUGGESTS an
//!     existing Template (advisory, NINSUN-class) or alerts "no match".
//!   - Pure std only. No walkdir, no sha256, no serde, no tokio.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// The destination sovereign store for a crossing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Threshold {
    EnkiDDB,
    EnkiMDB,
}

impl Threshold {
    pub fn store_name(&self) -> &'static str {
        match self {
            Threshold::EnkiDDB => "EnkiDDB",
            Threshold::EnkiMDB => "EnkiMDB",
        }
    }
}

/// Source artifact format detected at the threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Markdown,
    Html,
    RustSource,
    Svg,
    Unsupported,
}

pub fn detect_format(filename: &str) -> SourceFormat {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".md") { SourceFormat::Markdown }
    else if lower.ends_with(".html") || lower.ends_with(".htm") { SourceFormat::Html }
    else if lower.ends_with(".rs") { SourceFormat::RustSource }
    else if lower.ends_with(".svg") { SourceFormat::Svg }
    else { SourceFormat::Unsupported }
}

/// Sovereign pure-std content checksum (FNV-1a 64) — NO sha256 crate.
pub fn sovereign_checksum(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Pure-std recursive directory walk (NO walkdir crate).
/// Returns file paths under `root`, depth-first.
pub fn sovereign_walk(root: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.is_dir() {
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() { stack.push(path); }
                else { out.push(path); }
            }
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StagingStatus {
    Uploaded,
    Analyzed,
    PendingReview,
    Approved,
    Rejected,
    Duplicate,
    Committed,
}

/// A Template suggestion — ADVISORY ONLY. Susa never originates.
#[derive(Debug, Clone, PartialEq)]
pub enum CategorySuggestion {
    MatchedTemplate { template_name: String, similarity: f32 },
    NoMatchAlertArchitect,
}

#[derive(Debug, Clone)]
pub struct StagedArtifact {
    pub staging_id: u32,
    pub source_path: String,
    pub format: SourceFormat,
    pub checksum: u64,
    pub status: StagingStatus,
    pub suggestion: Option<CategorySuggestion>,
    pub eav: BTreeMap<String, String>,
}

impl StagedArtifact {
    pub fn new(staging_id: u32, source_path: &str, format: SourceFormat, checksum: u64) -> Self {
        StagedArtifact {
            staging_id, source_path: source_path.to_string(), format, checksum,
            status: StagingStatus::Uploaded, suggestion: None, eav: BTreeMap::new(),
        }
    }

    /// Record an analysis fact as EAV. Guard: reject v3.5 RGB keys.
    pub fn set_eav(&mut self, key: &str, value: &str) -> Result<(), String> {
        let k = key.to_ascii_lowercase();
        if k.contains("quality_byte") || k.contains("red") || k.contains("green") || k.contains("blue") {
            return Err(format!("forbidden v3.5 attribute '{}': quality/colour live as EAV, never bytes", key));
        }
        self.eav.insert(key.to_string(), value.to_string());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StagingBatch {
    pub batch_name: String,
    pub threshold: Threshold,
    pub artifacts: Vec<StagedArtifact>,
    next_id: u32,
    seen_checksums: std::collections::HashSet<u64>,
}

impl StagingBatch {
    pub fn create(batch_name: &str, threshold: Threshold) -> Self {
        StagingBatch {
            batch_name: batch_name.to_string(), threshold,
            artifacts: Vec::new(), next_id: 1,
            seen_checksums: std::collections::HashSet::new(),
        }
    }

    pub fn add(&mut self, source_path: &str, content: &[u8]) -> u32 {
        let format = detect_format(source_path);
        let checksum = sovereign_checksum(content);
        let id = self.next_id;
        self.next_id += 1;
        let mut art = StagedArtifact::new(id, source_path, format, checksum);
        if self.seen_checksums.contains(&checksum) {
            art.status = StagingStatus::Duplicate;
        } else {
            self.seen_checksums.insert(checksum);
        }
        self.artifacts.push(art);
        id
    }

    pub fn suggest(&mut self, staging_id: u32, suggestion: CategorySuggestion) {
        if let Some(a) = self.artifacts.iter_mut().find(|a| a.staging_id == staging_id) {
            if a.status != StagingStatus::Duplicate {
                a.suggestion = Some(suggestion);
                a.status = StagingStatus::Analyzed;
            }
        }
    }

    /// Architect confirms (the sovereign act). Blocks when no match.
    pub fn approve(&mut self, staging_id: u32) -> Result<(), String> {
        let a = self.artifacts.iter_mut().find(|a| a.staging_id == staging_id)
            .ok_or("staging id not found")?;
        match &a.suggestion {
            Some(CategorySuggestion::MatchedTemplate { .. }) => {
                a.status = StagingStatus::Approved; Ok(())
            }
            Some(CategorySuggestion::NoMatchAlertArchitect) =>
                Err("no Template matched — Architect must author/bind one before approval".into()),
            None => Err("artifact not analyzed yet".into()),
        }
    }

    /// The commit MANIFEST: approved artifacts ready to cross.
    /// KAKI minting happens DOWNSTREAM in enkidb-ingest::bridge.
    pub fn commit_manifest(&self) -> Vec<&StagedArtifact> {
        self.artifacts.iter().filter(|a| a.status == StagingStatus::Approved).collect()
    }

    pub fn duplicates(&self) -> Vec<&StagedArtifact> {
        self.artifacts.iter().filter(|a| a.status == StagingStatus::Duplicate).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_serves_both_stores() {
        assert_eq!(Threshold::EnkiDDB.store_name(), "EnkiDDB");
        assert_eq!(Threshold::EnkiMDB.store_name(), "EnkiMDB");
    }

    #[test]
    fn format_detection_covers_all_sovereign_types() {
        assert_eq!(detect_format("a.md"), SourceFormat::Markdown);
        assert_eq!(detect_format("b.HTML"), SourceFormat::Html);
        assert_eq!(detect_format("kaki.rs"), SourceFormat::RustSource);
        assert_eq!(detect_format("dash.svg"), SourceFormat::Svg);
        assert_eq!(detect_format("x.bin"), SourceFormat::Unsupported);
    }

    #[test]
    fn checksum_is_deterministic_and_sensitive() {
        assert_eq!(sovereign_checksum(b"hello"), sovereign_checksum(b"hello"));
        assert_ne!(sovereign_checksum(b"hello"), sovereign_checksum(b"hellp"));
    }

    #[test]
    fn duplicate_detected_by_checksum() {
        let mut b = StagingBatch::create("batch1", Threshold::EnkiDDB);
        b.add("docs/a.md", b"same content");
        b.add("docs/copy.md", b"same content");
        assert_eq!(b.duplicates().len(), 1);
    }

    #[test]
    fn eav_rejects_forbidden_rgb_attributes() {
        let mut a = StagedArtifact::new(1, "a.md", SourceFormat::Markdown, 0);
        assert!(a.set_eav("concept.primary", "algebra").is_ok());
        assert!(a.set_eav("green_byte", "200").is_err());
        assert!(a.set_eav("quality_byte", "200").is_err());
    }

    #[test]
    fn susa_never_originates_category_no_match_blocks_approval() {
        let mut b = StagingBatch::create("batch1", Threshold::EnkiDDB);
        let id = b.add("docs/novel.md", b"unprecedented content");
        b.suggest(id, CategorySuggestion::NoMatchAlertArchitect);
        assert!(b.approve(id).is_err());
    }

    #[test]
    fn matched_template_allows_approval_and_commit() {
        let mut b = StagingBatch::create("batch1", Threshold::EnkiDDB);
        let id = b.add("docs/etl.md", b"dataflow pipeline etl content");
        b.suggest(id, CategorySuggestion::MatchedTemplate {
            template_name: "PARZU_etl_v2".into(), similarity: 0.87,
        });
        assert!(b.approve(id).is_ok());
        assert_eq!(b.commit_manifest().len(), 1);
    }

    #[test]
    fn duplicates_never_reach_commit_manifest() {
        let mut b = StagingBatch::create("batch1", Threshold::EnkiMDB);
        let id1 = b.add("svc/a.rs", b"pub fn main() {}");
        b.add("svc/a_copy.rs", b"pub fn main() {}");
        b.suggest(id1, CategorySuggestion::MatchedTemplate {
            template_name: "PARZU_service_v1".into(), similarity: 0.9,
        });
        b.approve(id1).unwrap();
        assert_eq!(b.commit_manifest().len(), 1);
    }

    #[test]
    fn enkimdb_threshold_accepts_rust_sources() {
        let mut b = StagingBatch::create("services", Threshold::EnkiMDB);
        let id = b.add("crates/kaki/src/lib.rs", b"pub struct KakiV4([u8;16]);");
        assert_eq!(b.artifacts[0].format, SourceFormat::RustSource);
        b.suggest(id, CategorySuggestion::MatchedTemplate {
            template_name: "PARZU_rust_crate_v1".into(), similarity: 0.95,
        });
        assert!(b.approve(id).is_ok());
    }
}
