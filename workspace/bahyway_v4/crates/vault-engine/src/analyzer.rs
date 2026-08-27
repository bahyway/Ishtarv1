//! # Analyzer — Sovereign Dead-File Evidence Engine
//!
//! Finds REAL problems in the scanned file system.
//! Every "Dead" classification must have evidence here.
//! The sovereign rule: no Gray without evidence.

use super::file_walker::{FileRecord, ScanSummary};
use std::collections::HashMap;
use std::path::PathBuf;

/// Evidence that a file is dead or corrupt.
/// Without evidence, we NEVER classify as Dead.
#[derive(Debug, Clone)]
pub struct DeadEvidence {
    pub path: PathBuf,
    pub reason: DeadReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadReason {
    ZeroByte,           // file has 0 bytes
    DuplicateCandidate, // same size + same extension as another file
    VeryOld,            // > 5 years untouched AND > 0 bytes (archive candidate)
    OrphanExtension,    // extension unknown, isolated file
}

impl DeadReason {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ZeroByte => "Zero-byte file",
            Self::DuplicateCandidate => "Duplicate candidate",
            Self::VeryOld => "Very old (>5 years)",
            Self::OrphanExtension => "Orphan extension",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AnalysisReport {
    pub summary: ScanSummary,
    pub dead_evidence: Vec<DeadEvidence>,
    pub reclaimable_bytes: u64,
    pub duplicate_groups: u64,
    pub zero_byte_count: u64,
    pub old_files_count: u64,
    pub orphan_count: u64,
    pub duration_ms: u64,
}

impl AnalysisReport {
    pub fn has_problems(&self) -> bool {
        !self.dead_evidence.is_empty()
    }

    pub fn problems_in(&self, collection_path: &std::path::Path) -> usize {
        self.dead_evidence
            .iter()
            .filter(|e| e.path.starts_with(collection_path))
            .count()
    }
}

/// Run the real analysis pass.
/// Returns a report with evidence-backed findings only.
pub fn analyze_scan(files: &[FileRecord], summary: &ScanSummary) -> AnalysisReport {
    let start = std::time::Instant::now();
    let mut evidence = Vec::new();
    let mut reclaimable = 0_u64;

    // ── 1. Zero-byte files — always suspicious ──────────────────────────────
    let mut zero_count = 0_u64;
    for f in files {
        if f.is_zero {
            evidence.push(DeadEvidence {
                path: f.path.clone(),
                reason: DeadReason::ZeroByte,
            });
            zero_count += 1;
        }
    }

    // ── 2. Duplicate candidates — same size + extension ─────────────────────
    let mut by_size_ext: HashMap<(u64, String), Vec<&FileRecord>> = HashMap::new();
    for f in files.iter().filter(|f| !f.is_zero) {
        by_size_ext
            .entry((f.size, f.extension.clone()))
            .or_default()
            .push(f);
    }
    let mut dup_groups = 0_u64;
    for group in by_size_ext.values() {
        if group.len() >= 2 {
            dup_groups += 1;
            for f in group.iter().skip(1) {
                evidence.push(DeadEvidence {
                    path: f.path.clone(),
                    reason: DeadReason::DuplicateCandidate,
                });
                reclaimable += f.size;
            }
        }
    }

    // ── 3. Very old files (> 5 years untouched) ─────────────────────────────
    let mut old_count = 0_u64;
    for f in files {
        if !f.is_zero && f.age_days() > 5.0 * 365.0 {
            evidence.push(DeadEvidence {
                path: f.path.clone(),
                reason: DeadReason::VeryOld,
            });
            old_count += 1;
        }
    }

    // ── 4. Orphan extensions — single-occurrence unknown extensions ──────────
    let mut orphan_count = 0_u64;
    for f in files.iter().filter(|f| !f.is_zero) {
        if summary.extension_counts.get(&f.extension).copied() == Some(1)
            && !is_common_extension(&f.extension)
        {
            evidence.push(DeadEvidence {
                path: f.path.clone(),
                reason: DeadReason::OrphanExtension,
            });
            orphan_count += 1;
        }
    }

    AnalysisReport {
        summary: summary.clone(),
        dead_evidence: evidence,
        reclaimable_bytes: reclaimable,
        duplicate_groups: dup_groups,
        zero_byte_count: zero_count,
        old_files_count: old_count,
        orphan_count,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

fn is_common_extension(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "pdf"
            | "docx"
            | "xlsx"
            | "pptx"
            | "jpg"
            | "jpeg"
            | "png"
            | "gif"
            | "svg"
            | "webp"
            | "mp3"
            | "mp4"
            | "mov"
            | "avi"
            | "wav"
            | "flac"
            | "zip"
            | "tar"
            | "gz"
            | "7z"
            | "rar"
            | "rs"
            | "py"
            | "js"
            | "ts"
            | "html"
            | "css"
            | "json"
            | "xml"
            | "yaml"
            | "toml"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "(none)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_walker::walk_folder;

    fn make_temp_dir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "vault_ana_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn empty_scan_produces_empty_report() {
        let dir = make_temp_dir();
        let (files, summary) = walk_folder(&dir);
        let report = analyze_scan(&files, &summary);
        assert!(!report.has_problems());
        assert_eq!(report.zero_byte_count, 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn zero_byte_file_produces_evidence() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("empty.txt"), b"").unwrap();
        let (files, summary) = walk_folder(&dir);
        let report = analyze_scan(&files, &summary);
        assert!(report.has_problems());
        assert_eq!(report.zero_byte_count, 1);
        assert_eq!(report.dead_evidence[0].reason, DeadReason::ZeroByte);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn duplicate_files_produce_evidence() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("a.pdf"), b"same content").unwrap();
        std::fs::write(dir.join("b.pdf"), b"same content").unwrap(); // same size + ext
        let (files, summary) = walk_folder(&dir);
        let report = analyze_scan(&files, &summary);
        assert!(report.duplicate_groups >= 1);
        let dup_evidence: Vec<_> = report
            .dead_evidence
            .iter()
            .filter(|e| e.reason == DeadReason::DuplicateCandidate)
            .collect();
        assert!(
            !dup_evidence.is_empty(),
            "duplicate files must produce DuplicateCandidate evidence"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn healthy_unique_file_produces_no_evidence() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("unique.rs"), b"fn main() {}").unwrap();
        let (files, summary) = walk_folder(&dir);
        let report = analyze_scan(&files, &summary);
        // A single .rs file with content — no zero byte, no duplicate, no orphan (rs is common)
        let rs_evidence: Vec<_> = report
            .dead_evidence
            .iter()
            .filter(|e| e.path.extension().map(|x| x == "rs").unwrap_or(false))
            .collect();
        assert!(
            rs_evidence.is_empty(),
            "healthy unique .rs file must have no evidence"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn problems_in_returns_count_for_correct_path() {
        let dir = make_temp_dir();
        std::fs::write(dir.join("empty.txt"), b"").unwrap(); // zero byte in root
        let (files, summary) = walk_folder(&dir);
        let report = analyze_scan(&files, &summary);
        assert!(report.problems_in(&dir) >= 1);
        // Different path → no problems
        assert_eq!(
            report.problems_in(std::path::Path::new("/nonexistent/path")),
            0
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reclaimable_bytes_counts_duplicate_extras() {
        let dir = make_temp_dir();
        let content = b"sovereign data"; // 14 bytes
        std::fs::write(dir.join("a.bin"), content).unwrap();
        std::fs::write(dir.join("b.bin"), content).unwrap();
        let (files, summary) = walk_folder(&dir);
        let report = analyze_scan(&files, &summary);
        assert_eq!(
            report.reclaimable_bytes, 14,
            "reclaimable = size of one duplicate = 14 bytes"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
