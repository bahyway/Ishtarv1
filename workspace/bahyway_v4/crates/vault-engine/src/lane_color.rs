//! # LaneColor — Sovereign Quality Lane Classification
//!
//! The six BahyWay quality lanes, applied to file system entities.
//! Maps directly to KAKI B11 quality thresholds (ADR-001):
//!
//! | Lane | B11 equivalent | Meaning |
//! |------|---------------|---------|
//! | Gem      | ≥ 200 | Golden record — no evidence of problems |
//! | Active   | ≥ 100 | Healthy working collection |
//! | Fuzzy    | 59-99 | Needs attention — minor issues detected |
//! | Sec      | any   | Encrypted / archived — old but healthy |
//! | Dead     | < 59  | Gray-Rot confirmed — evidence exists |
//! | Critical | < 59  | Dead with urgency — immediate action required |

/// Sovereign quality lane for a file or collection.
/// Assigned ONLY from evidence — never from heuristics without backing data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LaneColor {
    /// Gem — Golden Record. Very recent, near-zero problems, known sovereign folder.
    Gem,
    /// Active — Healthy working collection. Low problem ratio.
    Active,
    /// Fuzzy — Needs attention. 10–29% problem ratio.
    Fuzzy,
    /// Sec — Old but healthy. Archived, encrypted, low-activity.
    Sec,
    /// Dead — Gray-Rot confirmed. ≥ 60% of files have `DeadEvidence`.
    Dead,
    /// Critical — 30–59% problem ratio. Immediate attention required.
    Critical,
}

impl LaneColor {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Gem => "Gem",
            Self::Active => "Active",
            Self::Fuzzy => "Fuzzy",
            Self::Sec => "Sec",
            Self::Dead => "Dead",
            Self::Critical => "Critical",
        }
    }

    /// CSS hex color for DOM / Canvas 2D rendering (dubsar-canvas)
    pub fn css_color(&self) -> &'static str {
        match self {
            Self::Gem => "#C9A84C",      // Gold
            Self::Active => "#2DD4BF",   // Teal
            Self::Fuzzy => "#F59E0B",    // Amber
            Self::Sec => "#3B82F6",      // Blue
            Self::Dead => "#6B7280",     // Gray
            Self::Critical => "#EF4444", // Red
        }
    }

    /// True if this lane indicates a health problem requiring action.
    pub fn is_problematic(&self) -> bool {
        matches!(self, Self::Dead | Self::Critical | Self::Fuzzy)
    }

    /// Approximate KAKI B11 quality score for this lane (ADR-001).
    pub fn quality_b11(&self) -> u8 {
        match self {
            Self::Gem => 220,
            Self::Active => 140,
            Self::Fuzzy => 80,
            Self::Sec => 120,
            Self::Dead => 30,
            Self::Critical => 50,
        }
    }
}

impl std::fmt::Display for LaneColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_and_critical_are_problematic() {
        assert!(LaneColor::Dead.is_problematic());
        assert!(LaneColor::Critical.is_problematic());
        assert!(LaneColor::Fuzzy.is_problematic());
    }

    #[test]
    fn gem_active_sec_are_not_problematic() {
        assert!(!LaneColor::Gem.is_problematic());
        assert!(!LaneColor::Active.is_problematic());
        assert!(!LaneColor::Sec.is_problematic());
    }

    #[test]
    fn gem_quality_is_highest() {
        assert!(LaneColor::Gem.quality_b11() > LaneColor::Active.quality_b11());
        assert!(LaneColor::Active.quality_b11() > LaneColor::Dead.quality_b11());
    }

    #[test]
    fn display_matches_label() {
        assert_eq!(format!("{}", LaneColor::Gem), "Gem");
        assert_eq!(format!("{}", LaneColor::Dead), "Dead");
    }
}
