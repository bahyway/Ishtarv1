//! gray_rot.rs — Gray-Rot Fragmentation Detector
//!
//! Gray-Rot is the sovereign term for physical database degradation.
//! When index fragmentation exceeds 20%, the particle turns gray —
//! the BahyWay threshold for mandatory maintenance action.
//!
//! Gray-Rot levels (4-tier model):
//!   Healthy  —  < 10%  fragmentation — no action
//!   Warning  — 10–20%  fragmentation — monitor
//!   GrayRot  — 20–40%  fragmentation — REORGANIZE
//!   Critical —  > 40%  fragmentation — REBUILD immediately
//!
//! The `alignment_penalty` bridges fragmentation into the DMW alignment score:
//!   penalty = (fragmentation_pct / 200.0).min(0.40)
//!   i.e. up to 40% alignment reduction at 80% fragmentation.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

// ── Gray-Rot Level ────────────────────────────────────────────────────────────

/// Four-tier fragmentation severity for the Gray-Rot model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GrayRotLevel {
    Healthy,
    Warning,
    GrayRot,
    Critical,
}

impl GrayRotLevel {
    pub fn from_fragmentation(pct: f32) -> Self {
        match pct as u32 {
            0..=9 => GrayRotLevel::Healthy,
            10..=19 => GrayRotLevel::Warning,
            20..=39 => GrayRotLevel::GrayRot,
            _ => GrayRotLevel::Critical,
        }
    }

    pub fn is_action_required(self) -> bool {
        matches!(self, GrayRotLevel::GrayRot | GrayRotLevel::Critical)
    }

    pub fn recommended_action(self) -> &'static str {
        match self {
            GrayRotLevel::Healthy => "No action needed",
            GrayRotLevel::Warning => "Monitor — consider REORGANIZE if growing",
            GrayRotLevel::GrayRot => "ALTER INDEX ... REORGANIZE",
            GrayRotLevel::Critical => "ALTER INDEX ... REBUILD WITH (ONLINE = ON)",
        }
    }

    /// Sovereign colour hex for the Gray-Rot level.
    pub fn color_hex(self) -> &'static str {
        match self {
            GrayRotLevel::Healthy => "#00D4C8",  // teal — sovereign health
            GrayRotLevel::Warning => "#FFD700",  // gold — amber alert
            GrayRotLevel::GrayRot => "#808080",  // gray — the rot begins
            GrayRotLevel::Critical => "#FF4444", // red  — critical decay
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GrayRotLevel::Healthy => "Healthy",
            GrayRotLevel::Warning => "Warning",
            GrayRotLevel::GrayRot => "Gray-Rot",
            GrayRotLevel::Critical => "Critical",
        }
    }
}

impl std::fmt::Display for GrayRotLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ── Gray-Rot Report ───────────────────────────────────────────────────────────

/// Fragmentation analysis result with DMW alignment penalty.
#[derive(Debug, Clone)]
pub struct GrayRotReport {
    pub fragmentation_pct: f32,
    pub level: GrayRotLevel,
    pub page_reads: u64,
    pub action_required: bool,
    pub recommended_action: String,
    /// Alignment penalty applied to the DMW score (0.0–0.40).
    pub alignment_penalty: f32,
}

impl GrayRotReport {
    /// Build a report from raw fragmentation percentage and I/O page reads.
    pub fn from_plan_data(fragmentation_pct: f32, page_reads: u64) -> Self {
        let level = GrayRotLevel::from_fragmentation(fragmentation_pct);
        let action_required = level.is_action_required();
        // Linear penalty — up to 0.40 at 80% fragmentation.
        let alignment_penalty = (fragmentation_pct / 200.0).min(0.40);

        GrayRotReport {
            fragmentation_pct,
            level,
            page_reads,
            action_required,
            recommended_action: level.recommended_action().to_string(),
            alignment_penalty,
        }
    }

    /// Build a report from v4.0's `FragmentationMap` (uses worst fragment).
    pub fn from_fragmentation_map(map: &crate::fragmentation::FragmentationMap) -> Self {
        let worst_pct = map
            .worst_fragment()
            .map(|f| f.fragmentation_pct)
            .unwrap_or(0.0);
        let page_count = map
            .worst_fragment()
            .map(|f| f.page_count as u64)
            .unwrap_or(0);
        Self::from_plan_data(worst_pct, page_count)
    }

    /// Plain-language narrative for this fragmentation state.
    pub fn narrative(&self) -> String {
        if self.action_required {
            format!(
                "Gray-Rot detected at {:.1}% fragmentation. \
                 Page reads: {}. Action: {}. \
                 Alignment penalty: -{:.0}%.",
                self.fragmentation_pct,
                self.page_reads,
                self.recommended_action,
                self.alignment_penalty * 100.0,
            )
        } else {
            format!(
                "Storage healthy at {:.1}% fragmentation. No action needed.",
                self.fragmentation_pct
            )
        }
    }

    /// Apply the alignment penalty to a current alignment score.
    pub fn penalised_alignment(&self, current: f32) -> f32 {
        (current - self.alignment_penalty).max(0.0)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fragmentation::{sales_order_frag_map, FragmentationMap};

    #[test]
    fn level_boundaries() {
        assert_eq!(GrayRotLevel::from_fragmentation(5.0), GrayRotLevel::Healthy);
        assert_eq!(GrayRotLevel::from_fragmentation(9.9), GrayRotLevel::Healthy);
        assert_eq!(
            GrayRotLevel::from_fragmentation(10.0),
            GrayRotLevel::Warning
        );
        assert_eq!(
            GrayRotLevel::from_fragmentation(19.9),
            GrayRotLevel::Warning
        );
        assert_eq!(
            GrayRotLevel::from_fragmentation(20.0),
            GrayRotLevel::GrayRot
        );
        assert_eq!(
            GrayRotLevel::from_fragmentation(39.9),
            GrayRotLevel::GrayRot
        );
        assert_eq!(
            GrayRotLevel::from_fragmentation(40.0),
            GrayRotLevel::Critical
        );
    }

    #[test]
    fn action_required_above_gray_rot_threshold() {
        assert!(!GrayRotLevel::Healthy.is_action_required());
        assert!(!GrayRotLevel::Warning.is_action_required());
        assert!(GrayRotLevel::GrayRot.is_action_required());
        assert!(GrayRotLevel::Critical.is_action_required());
    }

    #[test]
    fn alignment_penalty_increases_with_fragmentation() {
        let low = GrayRotReport::from_plan_data(5.0, 0);
        let high = GrayRotReport::from_plan_data(60.0, 0);
        assert!(high.alignment_penalty > low.alignment_penalty);
    }

    #[test]
    fn alignment_penalty_capped_at_40_percent() {
        let r = GrayRotReport::from_plan_data(100.0, 0);
        assert!(r.alignment_penalty <= 0.40);
    }

    #[test]
    fn narrative_mentions_fragmentation_pct() {
        let r = GrayRotReport::from_plan_data(35.0, 5000);
        assert!(
            r.narrative().contains("35.0"),
            "narrative={}",
            r.narrative()
        );
    }

    #[test]
    fn narrative_non_empty_for_healthy() {
        let r = GrayRotReport::from_plan_data(3.0, 100);
        assert!(!r.narrative().is_empty());
    }

    #[test]
    fn penalised_alignment_reduces_score() {
        let r = GrayRotReport::from_plan_data(40.0, 0);
        assert!(r.penalised_alignment(0.85) < 0.85);
    }

    #[test]
    fn penalised_alignment_floored_at_zero() {
        let r = GrayRotReport::from_plan_data(100.0, 0);
        assert_eq!(r.penalised_alignment(0.0), 0.0);
    }

    #[test]
    fn color_hex_starts_with_hash() {
        for level in [
            GrayRotLevel::Healthy,
            GrayRotLevel::Warning,
            GrayRotLevel::GrayRot,
            GrayRotLevel::Critical,
        ] {
            assert!(level.color_hex().starts_with('#'), "level={level:?}");
        }
    }

    #[test]
    fn labels_non_empty() {
        for level in [
            GrayRotLevel::Healthy,
            GrayRotLevel::Warning,
            GrayRotLevel::GrayRot,
            GrayRotLevel::Critical,
        ] {
            assert!(!level.label().is_empty());
        }
    }

    #[test]
    fn level_ordering() {
        assert!(GrayRotLevel::Critical > GrayRotLevel::Healthy);
        assert!(GrayRotLevel::GrayRot > GrayRotLevel::Warning);
    }

    #[test]
    fn from_fragmentation_map_uses_worst_index() {
        let map = sales_order_frag_map();
        let r = GrayRotReport::from_fragmentation_map(&map);
        // sales_order_frag_map has IX_Person_LastName at 38.4% → GrayRot
        assert!(r.fragmentation_pct >= 30.0, "pct={}", r.fragmentation_pct);
        assert!(r.action_required);
    }

    #[test]
    fn empty_map_is_healthy() {
        let map = FragmentationMap::new(vec![]);
        let r = GrayRotReport::from_fragmentation_map(&map);
        assert_eq!(r.level, GrayRotLevel::Healthy);
        assert_eq!(r.alignment_penalty, 0.0);
    }
}
