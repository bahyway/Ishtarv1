//! # lane_classifier — Sovereign Evidence-to-Lane Mapping
//!
//! THE SOVEREIGN RULE: No Gray without evidence.
//!
//! Every collection and every file gets a lane classification based on
//! REAL evidence from the analysis. A collection cannot be Dead unless the
//! analyzer found real problems in it. A file cannot be Dead unless it has
//! DeadEvidence.

use super::analyzer::AnalysisReport;
use super::collection::Collection;
use super::file_walker::FileRecord;
use crate::LaneColor;

/// Classify a collection based on analysis evidence.
/// Returns the dominant lane color and the ratio of dead files.
pub fn classify_collection(col: &Collection, report: &AnalysisReport) -> (LaneColor, f32) {
    let dead_in_col = report.problems_in(&col.path) as f32;
    let total       = col.file_count as f32;
    if total == 0.0 { return (LaneColor::Fuzzy, 0.0); }

    let dead_ratio = dead_in_col / total;

    let lane = if dead_ratio >= 0.60 {
        LaneColor::Dead          // > 60% problems — collection collapsed
    } else if dead_ratio >= 0.30 {
        LaneColor::Critical      // 30-59% problems — critical state
    } else if dead_ratio >= 0.10 {
        LaneColor::Fuzzy         // 10-29% problems — needs attention
    } else if col.avg_age_days < 30.0 && dead_ratio < 0.02 {
        if is_gem_folder_name(&col.name) {
            LaneColor::Gem
        } else {
            LaneColor::Active
        }
    } else {
        LaneColor::Active        // healthy working collection
    };

    (lane, dead_ratio)
}

/// Classify a single file. Returns Dead ONLY if evidence exists.
pub fn classify_file(file: &FileRecord, report: &AnalysisReport) -> LaneColor {
    if file.is_zero { return LaneColor::Dead; }

    let has_evidence = report.dead_evidence.iter().any(|e| e.path == file.path);
    if has_evidence { return LaneColor::Dead; }

    if file.age_days() < 90.0 {
        LaneColor::Active
    } else if file.age_days() < 365.0 {
        LaneColor::Fuzzy
    } else {
        LaneColor::Sec  // old but healthy — archived
    }
}

/// Heuristic: is this folder name likely to contain golden records?
fn is_gem_folder_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(lower.as_str(),
        "contracts" | "invoices" | "masters" | "originals" |
        "gold" | "golden" | "gem" | "publish" | "released" |
        "final" | "production")
    || lower.contains("contract")
    || lower.contains("invoice")
    || lower.contains("master")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_walker::FileRecord;
    use crate::analyzer::{AnalysisReport, DeadEvidence, DeadReason};
    use std::path::PathBuf;

    fn empty_report() -> AnalysisReport {
        AnalysisReport::default()
    }

    fn make_file(name: &str, size: u64, is_zero: bool) -> FileRecord {
        FileRecord {
            path: PathBuf::from(name),
            size,
            extension: "txt".to_string(),
            modified: None,
            is_zero,
            parent_dir: PathBuf::from("."),
        }
    }

    fn make_collection(name: &str, file_count: u64, avg_age_days: f32) -> Collection {
        Collection {
            id: "col_0001".to_string(),
            name: name.to_string(),
            path: PathBuf::from(format!("/root/{}", name)),
            files: Vec::new(),
            total_bytes: 0,
            file_count,
            zero_byte_cnt: 0,
            avg_age_days,
            dominant_ext: "txt".to_string(),
        }
    }

    #[test]
    fn empty_collection_classifies_as_fuzzy() {
        let col    = make_collection("empty", 0, 0.0);
        let report = empty_report();
        let (lane, ratio) = classify_collection(&col, &report);
        assert_eq!(lane, LaneColor::Fuzzy);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn collection_with_no_problems_classifies_active() {
        let col    = make_collection("docs", 100, 200.0);
        let report = empty_report();
        let (lane, _) = classify_collection(&col, &report);
        assert_eq!(lane, LaneColor::Active);
    }

    #[test]
    fn gem_folder_name_classifies_gem_when_recent() {
        let col    = make_collection("contracts", 50, 5.0); // recent, few days old
        let report = empty_report();
        let (lane, _) = classify_collection(&col, &report);
        assert_eq!(lane, LaneColor::Gem, "contracts folder < 30 days old with no problems → Gem");
    }

    #[test]
    fn high_problem_ratio_classifies_dead() {
        let col = make_collection("trash", 10, 500.0);
        let mut report = empty_report();
        // 7 out of 10 files have evidence → 70% ratio → Dead
        for i in 0..7 {
            report.dead_evidence.push(DeadEvidence {
                path: PathBuf::from(format!("/root/trash/file{}.txt", i)),
                reason: DeadReason::ZeroByte,
            });
        }
        let (lane, ratio) = classify_collection(&col, &report);
        assert_eq!(lane, LaneColor::Dead);
        assert!(ratio >= 0.60);
    }

    #[test]
    fn medium_problem_ratio_classifies_critical() {
        let col = make_collection("mixed", 10, 500.0);
        let mut report = empty_report();
        for i in 0..4 {
            report.dead_evidence.push(DeadEvidence {
                path: PathBuf::from(format!("/root/mixed/file{}.txt", i)),
                reason: DeadReason::DuplicateCandidate,
            });
        }
        let (lane, ratio) = classify_collection(&col, &report);
        assert_eq!(lane, LaneColor::Critical, "40% dead ratio must be Critical");
        assert!(ratio >= 0.30 && ratio < 0.60);
    }

    #[test]
    fn zero_byte_file_classifies_dead() {
        let file   = make_file("empty.txt", 0, true);
        let report = empty_report();
        assert_eq!(classify_file(&file, &report), LaneColor::Dead);
    }

    #[test]
    fn file_with_evidence_classifies_dead() {
        let file = make_file("dup.txt", 100, false);
        let mut report = empty_report();
        report.dead_evidence.push(DeadEvidence {
            path: PathBuf::from("dup.txt"),
            reason: DeadReason::DuplicateCandidate,
        });
        assert_eq!(classify_file(&file, &report), LaneColor::Dead);
    }

    #[test]
    fn healthy_recent_file_classifies_active() {
        let file   = make_file("new.txt", 100, false);
        let report = empty_report();
        // age_days() = 0.0 (modified = None → returns 0.0) → Active
        assert_eq!(classify_file(&file, &report), LaneColor::Active);
    }

    #[test]
    fn is_gem_folder_name_recognizes_sovereign_folders() {
        assert!(is_gem_folder_name("contracts"));
        assert!(is_gem_folder_name("invoices"));
        assert!(is_gem_folder_name("masters"));
        assert!(is_gem_folder_name("FINAL"));
        assert!(is_gem_folder_name("my_master_copy"));
    }

    #[test]
    fn is_gem_folder_name_rejects_regular_folders() {
        assert!(!is_gem_folder_name("documents"));
        assert!(!is_gem_folder_name("downloads"));
        assert!(!is_gem_folder_name("tmp"));
    }
}
