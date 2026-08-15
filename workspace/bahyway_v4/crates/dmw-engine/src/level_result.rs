//! level_result.rs — Rich per-level result and full journey record.
//!
//! `LevelResult` carries before/after color, problem/solution SQL, and
//! a StoryWay narrative entry for each of the 7 analysis levels.
//!
//! `AnalysisJourney` is the root output from a full 7-level pass:
//! it aggregates all LevelResults, the final ColorId fingerprint, and
//! the complete plain-language narrative.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::colorid::ColorId;
use crate::journey::JourneyLevelId;

// ── Level Result ──────────────────────────────────────────────────────────────

/// Rich result for a single analysis level.
#[derive(Debug, Clone)]
pub struct LevelResult {
    /// Which level this result belongs to.
    pub level: JourneyLevelId,
    /// Whether a problem was detected at this level.
    pub has_problem: bool,
    /// Short title for the detected problem (empty if no problem).
    pub problem_title:  String,
    /// Full description of the problem.
    pub problem_detail: String,
    /// Example SQL exhibiting the problem (empty if no problem).
    pub problem_sql: String,
    /// Short title for the recommended solution.
    pub solution_title:  String,
    /// Full description of the fix.
    pub solution_detail: String,
    /// Example SQL after the fix is applied.
    pub solution_sql: String,
    /// ColorId before the fix is applied.
    pub color_before: ColorId,
    /// ColorId after the fix is applied (projected healthy state).
    pub color_after: ColorId,
    /// AkkadianAOL rule reference.
    pub akkadi_rule: String,
    /// StoryWay-ready plain-language narrative entry.
    pub storyway_entry: String,
}

impl LevelResult {
    /// Create a passing (no-problem) result for a level.
    pub fn passing(
        level:          JourneyLevelId,
        solution_title: impl Into<String>,
        detail:         impl Into<String>,
        color:          ColorId,
        akkadi_rule:    impl Into<String>,
    ) -> Self {
        let detail_str = detail.into();
        LevelResult {
            level,
            has_problem:    false,
            problem_title:  String::new(),
            problem_detail: String::new(),
            problem_sql:    String::new(),
            solution_title: solution_title.into(),
            solution_detail: detail_str.clone(),
            solution_sql:   String::new(),
            color_before:   color,
            color_after:    color,
            akkadi_rule:    akkadi_rule.into(),
            storyway_entry: format!(
                "Level {} {} — PASS: {}",
                level.number(), level.akkadian_name(), detail_str
            ),
        }
    }

    /// Create a failing (problem detected) result for a level.
    #[allow(clippy::too_many_arguments)]
    pub fn failing(
        level:          JourneyLevelId,
        problem_title:  impl Into<String>,
        problem_detail: impl Into<String>,
        problem_sql:    impl Into<String>,
        solution_title: impl Into<String>,
        solution_detail: impl Into<String>,
        solution_sql:   impl Into<String>,
        color_before:   ColorId,
        color_after:    ColorId,
        akkadi_rule:    impl Into<String>,
    ) -> Self {
        let prob = problem_detail.into();
        let sol  = solution_detail.into();
        LevelResult {
            level,
            has_problem:    true,
            problem_title:  problem_title.into(),
            problem_detail: prob.clone(),
            problem_sql:    problem_sql.into(),
            solution_title: solution_title.into(),
            solution_detail: sol.clone(),
            solution_sql:   solution_sql.into(),
            color_before,
            color_after,
            akkadi_rule:    akkadi_rule.into(),
            storyway_entry: format!(
                "Level {} {} — PROBLEM: {} | FIX: {}",
                level.number(), level.akkadian_name(), prob, sol
            ),
        }
    }

    /// Alignment gain from color_before → color_after (0.0–1.0).
    pub fn alignment_gain(&self) -> f32 {
        let before = self.color_before.health_score();
        let after  = self.color_after.health_score();
        (after - before).max(0.0)
    }
}

// ── Analysis Journey ──────────────────────────────────────────────────────────

/// Complete result of a full 7-level journey analysis.
#[derive(Debug, Clone)]
pub struct AnalysisJourney {
    /// Query identifier.
    pub query_id: String,
    /// One result per level (always 7 entries).
    pub levels: Vec<LevelResult>,
    /// Final ColorId fingerprint after all levels are analysed.
    pub final_color_id: ColorId,
    /// Number of levels where a problem was detected.
    pub problem_count: usize,
    /// Total alignment gain if all recommendations are applied (0.0–1.0).
    pub alignment_gain: f32,
    /// Full plain-language narrative (StoryWay-ready).
    pub full_narrative: String,
}

impl AnalysisJourney {
    /// Build from a completed set of level results.
    pub fn from_results(query_id: impl Into<String>, levels: Vec<LevelResult>) -> Self {
        assert_eq!(levels.len(), 7, "AnalysisJourney requires exactly 7 LevelResults");

        let query_id       = query_id.into();
        let problem_count  = levels.iter().filter(|l| l.has_problem).count();
        let alignment_gain = levels.iter().map(|l| l.alignment_gain()).sum::<f32>().min(1.0);

        // Final color: merge worst across all color_before entries
        let final_color_id = levels.iter().fold(ColorId::healthy(), |acc, l| {
            acc.merge_worst(l.color_before)
        });

        let full_narrative = build_narrative(&query_id, &levels, problem_count);

        AnalysisJourney {
            query_id,
            levels,
            final_color_id,
            problem_count,
            alignment_gain,
            full_narrative,
        }
    }

    /// Returns true when any level detected a critical problem.
    pub fn has_critical_problem(&self) -> bool {
        self.levels.iter().any(|l| l.has_problem && l.color_before.red > 180)
    }

    /// Get the result for a specific level.
    pub fn level(&self, id: JourneyLevelId) -> &LevelResult {
        &self.levels[id as usize]
    }
}

fn build_narrative(query_id: &str, levels: &[LevelResult], problem_count: usize) -> String {
    let mut parts = vec![
        format!("Journey Analysis: {} | {} problem(s) detected", query_id, problem_count),
        String::new(),
    ];
    for l in levels {
        parts.push(l.storyway_entry.clone());
    }
    parts.join("\n")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn passing_result(level: JourneyLevelId) -> LevelResult {
        LevelResult::passing(
            level, "All good", "No issues detected",
            ColorId::healthy(), "none",
        )
    }

    fn failing_result(level: JourneyLevelId) -> LevelResult {
        LevelResult::failing(
            level,
            "Problem title", "Problem detail", "-- bad sql",
            "Solution title", "Solution detail", "-- good sql",
            ColorId::critical(), ColorId::healthy(),
            "test_rule",
        )
    }

    fn seven_passing() -> Vec<LevelResult> {
        JourneyLevelId::all().map(passing_result).to_vec()
    }

    #[test]
    fn passing_result_has_no_problem() {
        let r = passing_result(JourneyLevelId::ME);
        assert!(!r.has_problem);
        assert!(r.problem_title.is_empty());
    }

    #[test]
    fn failing_result_has_problem() {
        let r = failing_result(JourneyLevelId::GU);
        assert!(r.has_problem);
        assert!(!r.problem_title.is_empty());
        assert!(!r.solution_title.is_empty());
    }

    #[test]
    fn passing_alignment_gain_is_zero() {
        let r = passing_result(JourneyLevelId::A);
        assert_eq!(r.alignment_gain(), 0.0);
    }

    #[test]
    fn failing_alignment_gain_is_positive() {
        let r = failing_result(JourneyLevelId::A);
        assert!(r.alignment_gain() > 0.0);
    }

    #[test]
    fn storyway_entry_contains_level_info() {
        let r = passing_result(JourneyLevelId::ME);
        assert!(r.storyway_entry.contains("ME"));
        assert!(r.storyway_entry.contains("PASS"));
    }

    #[test]
    fn failing_storyway_contains_problem_and_fix() {
        let r = failing_result(JourneyLevelId::GU);
        assert!(r.storyway_entry.contains("PROBLEM"));
        assert!(r.storyway_entry.contains("FIX"));
    }

    #[test]
    fn journey_from_seven_levels() {
        let levels = seven_passing();
        let j = AnalysisJourney::from_results("Q001", levels);
        assert_eq!(j.levels.len(), 7);
        assert_eq!(j.problem_count, 0);
    }

    #[test]
    fn journey_counts_problems_correctly() {
        let mut levels = seven_passing();
        levels[2] = failing_result(JourneyLevelId::SAG);
        levels[4] = failing_result(JourneyLevelId::IZI);
        let j = AnalysisJourney::from_results("Q002", levels);
        assert_eq!(j.problem_count, 2);
    }

    #[test]
    fn journey_alignment_gain_for_failing_levels() {
        let mut levels = seven_passing();
        levels[0] = failing_result(JourneyLevelId::ME);
        let j = AnalysisJourney::from_results("Q003", levels);
        assert!(j.alignment_gain > 0.0);
    }

    #[test]
    fn level_accessor_returns_correct_level() {
        let levels = seven_passing();
        let j = AnalysisJourney::from_results("Q004", levels);
        assert_eq!(j.level(JourneyLevelId::URU).level, JourneyLevelId::URU);
    }

    #[test]
    fn full_narrative_non_empty() {
        let j = AnalysisJourney::from_results("Q005", seven_passing());
        assert!(!j.full_narrative.is_empty());
        assert!(j.full_narrative.contains("Journey Analysis"));
    }

    #[test]
    fn akkadian_name_in_storyway_entry() {
        let r = passing_result(JourneyLevelId::IZI);
        assert!(r.storyway_entry.contains("IZI"));
    }
}
